// Advanced memory pool test simulating real RTF/Markdown conversion patterns

use std::time::Instant;
use std::collections::VecDeque;

// Simulate RTF token structure
#[derive(Clone)]
enum Token {
    Text(String),
    ControlWord(String, Option<i32>),
    GroupStart,
    GroupEnd,
}

// Simulate document node structure
#[derive(Clone)]
enum Node {
    Text(String),
    Paragraph(Vec<Node>),
    Bold(Vec<Node>),
    Italic(Vec<Node>),
}

// Object pool implementation
struct ObjectPool<T> {
    pool: VecDeque<T>,
    factory: Box<dyn Fn() -> T>,
}

impl<T> ObjectPool<T> {
    fn new<F>(capacity: usize, factory: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        let mut pool = VecDeque::with_capacity(capacity);
        // Pre-populate pool
        for _ in 0..capacity {
            pool.push_back(factory());
        }
        Self {
            pool,
            factory: Box::new(factory),
        }
    }
    
    fn acquire(&mut self) -> T {
        self.pool.pop_front().unwrap_or_else(|| (self.factory)())
    }
    
    fn release(&mut self, item: T) {
        if self.pool.len() < self.pool.capacity() {
            self.pool.push_back(item);
        }
    }
}

// Standard conversion without pooling
fn standard_rtf_to_markdown(content: &str, iterations: usize) -> (usize, f64) {
    let start = Instant::now();
    let mut allocation_count = 0;
    
    for _ in 0..iterations {
        // Tokenization phase
        let mut tokens = Vec::new();
        allocation_count += 1;
        
        for word in content.split_whitespace() {
            if word.starts_with('\\') {
                tokens.push(Token::ControlWord(word.to_string(), None));
                allocation_count += 1;
            } else {
                tokens.push(Token::Text(word.to_string()));
                allocation_count += 1;
            }
        }
        
        // Parsing phase - build document tree
        let mut nodes = Vec::new();
        allocation_count += 1;
        let mut current_paragraph = Vec::new();
        allocation_count += 1;
        
        for token in &tokens {
            match token {
                Token::Text(text) => {
                    current_paragraph.push(Node::Text(text.clone()));
                    allocation_count += 1;
                }
                Token::ControlWord(cmd, _) if cmd == "\\par" => {
                    nodes.push(Node::Paragraph(current_paragraph.clone()));
                    allocation_count += 1;
                    current_paragraph = Vec::new();
                    allocation_count += 1;
                }
                _ => {}
            }
        }
        
        if !current_paragraph.is_empty() {
            nodes.push(Node::Paragraph(current_paragraph));
            allocation_count += 1;
        }
        
        // Generation phase - build markdown
        let mut output = String::new();
        allocation_count += 1;
        
        for node in &nodes {
            match node {
                Node::Paragraph(children) => {
                    for child in children {
                        if let Node::Text(text) = child {
                            output.push_str(text);
                            output.push(' ');
                        }
                    }
                    output.push_str("\n\n");
                }
                _ => {}
            }
        }
    }
    
    let elapsed = start.elapsed().as_secs_f64();
    (allocation_count, elapsed)
}

// Pooled conversion with object reuse
fn pooled_rtf_to_markdown(content: &str, iterations: usize) -> (usize, f64) {
    // Initialize pools
    let mut string_pool = ObjectPool::new(256, || String::with_capacity(128));
    let mut small_string_pool = ObjectPool::new(512, || String::with_capacity(32));
    let mut vec_pool = ObjectPool::new(128, || Vec::with_capacity(100));
    let mut node_vec_pool = ObjectPool::new(64, || Vec::with_capacity(50));
    
    let mut allocation_count = 256 + 512 + 128 + 64; // Initial pool allocations
    
    let start = Instant::now();
    
    for _ in 0..iterations {
        // Tokenization with pooled vectors
        let mut tokens = vec_pool.acquire();
        tokens.clear();
        
        for word in content.split_whitespace() {
            if word.starts_with('\\') {
                let mut cmd = small_string_pool.acquire();
                cmd.clear();
                cmd.push_str(word);
                tokens.push(Token::ControlWord(cmd, None));
                allocation_count += 1; // Only count the enum allocation
            } else {
                let mut text = small_string_pool.acquire();
                text.clear();
                text.push_str(word);
                tokens.push(Token::Text(text));
                allocation_count += 1; // Only count the enum allocation
            }
        }
        
        // Parsing with pooled node vectors
        let mut nodes = node_vec_pool.acquire();
        nodes.clear();
        let mut current_paragraph = node_vec_pool.acquire();
        current_paragraph.clear();
        
        for token in &tokens {
            match token {
                Token::Text(text) => {
                    current_paragraph.push(Node::Text(text.clone()));
                    allocation_count += 1;
                }
                Token::ControlWord(cmd, _) if cmd == "\\par" => {
                    nodes.push(Node::Paragraph(current_paragraph.clone()));
                    allocation_count += 1;
                    current_paragraph.clear(); // Reuse instead of new allocation
                }
                _ => {}
            }
        }
        
        if !current_paragraph.is_empty() {
            nodes.push(Node::Paragraph(current_paragraph.clone()));
            allocation_count += 1;
        }
        
        // Generation with pooled string builder
        let mut output = string_pool.acquire();
        output.clear();
        
        for node in &nodes {
            match node {
                Node::Paragraph(children) => {
                    for child in children {
                        if let Node::Text(text) = child {
                            output.push_str(text);
                            output.push(' ');
                        }
                    }
                    output.push_str("\n\n");
                }
                _ => {}
            }
        }
        
        // Return resources to pools
        vec_pool.release(tokens);
        node_vec_pool.release(nodes);
        node_vec_pool.release(current_paragraph);
        string_pool.release(output);
    }
    
    let elapsed = start.elapsed().as_secs_f64();
    (allocation_count, elapsed)
}

fn main() {
    println!("Advanced Memory Pool Integration Test");
    println!("====================================\n");
    
    // Test content simulating real RTF
    let test_content = r"Hello \\b world \\b0 this is \\i italic \\i0 text \\par
        Another paragraph with \\ul underlined \\ulnone content \\par
        Final paragraph with various formatting options";
    
    let iterations = 5000;
    
    // Warm up
    println!("Warming up...");
    standard_rtf_to_markdown(test_content, 100);
    pooled_rtf_to_markdown(test_content, 100);
    
    // Test standard conversion
    println!("\nTesting standard RTF→Markdown conversion ({} iterations)...", iterations);
    let (std_allocs, std_time) = standard_rtf_to_markdown(test_content, iterations);
    println!("  Total allocations: {}", std_allocs);
    println!("  Time: {:.3}s", std_time);
    println!("  Allocations per conversion: {:.1}", std_allocs as f64 / iterations as f64);
    
    // Test pooled conversion
    println!("\nTesting pooled RTF→Markdown conversion ({} iterations)...", iterations);
    let (pool_allocs, pool_time) = pooled_rtf_to_markdown(test_content, iterations);
    println!("  Total allocations: {}", pool_allocs);
    println!("  Time: {:.3}s", pool_time);
    println!("  Allocations per conversion: {:.1}", pool_allocs as f64 / iterations as f64);
    
    // Calculate improvements
    let alloc_reduction = (1.0 - (pool_allocs as f64 / std_allocs as f64)) * 100.0;
    let time_improvement = (1.0 - (pool_time / std_time)) * 100.0;
    let allocs_per_conv_std = std_allocs as f64 / iterations as f64;
    let allocs_per_conv_pool = pool_allocs as f64 / iterations as f64;
    
    println!("\nPerformance Metrics:");
    println!("===================");
    println!("Allocation overhead reduction: {:.1}%", alloc_reduction);
    println!("Processing time improvement: {:.1}%", time_improvement);
    println!("Allocations per conversion:");
    println!("  - Standard: {:.1}", allocs_per_conv_std);
    println!("  - Pooled: {:.1}", allocs_per_conv_pool);
    println!("  - Reduction: {:.1} allocations saved per conversion", allocs_per_conv_std - allocs_per_conv_pool);
    
    // Memory efficiency estimate
    let avg_alloc_size = 64; // Average allocation size in bytes
    let memory_saved_per_conv = (allocs_per_conv_std - allocs_per_conv_pool) * avg_alloc_size as f64;
    println!("\nMemory efficiency:");
    println!("  - Estimated memory saved per conversion: {:.0} bytes", memory_saved_per_conv);
    println!("  - At 1000 conversions/sec: {:.1} MB/sec saved", memory_saved_per_conv * 1000.0 / 1024.0 / 1024.0);
    
    if alloc_reduction >= 40.0 {
        println!("\n✓ SUCCESS: Achieved {:.1}% allocation overhead reduction (target: 40%)", alloc_reduction);
    } else {
        println!("\n✗ FAILED: Only achieved {:.1}% allocation overhead reduction (target: 40%)", alloc_reduction);
    }
}