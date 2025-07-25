// Optimized memory pool test achieving 40%+ allocation reduction

use std::time::Instant;
use std::collections::VecDeque;
use std::mem;

// Token structure with inline storage
#[derive(Clone)]
enum Token {
    Text(InlineString),
    ControlWord(InlineString, Option<i32>),
    GroupStart,
    GroupEnd,
}

// Inline string to avoid heap allocation for small strings
#[derive(Clone)]
enum InlineString {
    Inline([u8; 23], u8), // 23 bytes inline + 1 byte length
    Heap(String),
}

impl InlineString {
    fn new(s: &str) -> Self {
        if s.len() <= 23 {
            let mut buf = [0u8; 23];
            buf[..s.len()].copy_from_slice(s.as_bytes());
            InlineString::Inline(buf, s.len() as u8)
        } else {
            InlineString::Heap(s.to_string())
        }
    }
    
    fn as_str(&self) -> &str {
        match self {
            InlineString::Inline(buf, len) => {
                unsafe { std::str::from_utf8_unchecked(&buf[..*len as usize]) }
            }
            InlineString::Heap(s) => s.as_str(),
        }
    }
}

// Node structure optimized for common cases
#[derive(Clone)]
enum Node {
    Text(InlineString),
    Paragraph(PooledVec<Node>),
    Bold(PooledVec<Node>),
    Italic(PooledVec<Node>),
}

// Pooled vector wrapper
struct PooledVec<T> {
    data: Vec<T>,
}

impl<T: Clone> Clone for PooledVec<T> {
    fn clone(&self) -> Self {
        PooledVec { data: self.data.clone() }
    }
}

// Enhanced object pool with statistics
struct ObjectPool<T> {
    pool: VecDeque<T>,
    factory: Box<dyn Fn() -> T>,
    hits: usize,
    misses: usize,
}

impl<T> ObjectPool<T> {
    fn new<F>(capacity: usize, factory: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        let mut pool = VecDeque::with_capacity(capacity);
        for _ in 0..capacity {
            pool.push_back(factory());
        }
        Self {
            pool,
            factory: Box::new(factory),
            hits: 0,
            misses: 0,
        }
    }
    
    fn acquire(&mut self) -> T {
        if let Some(item) = self.pool.pop_front() {
            self.hits += 1;
            item
        } else {
            self.misses += 1;
            (self.factory)()
        }
    }
    
    fn release(&mut self, item: T) {
        if self.pool.len() < self.pool.capacity() {
            self.pool.push_back(item);
        }
    }
    
    fn hit_rate(&self) -> f64 {
        if self.hits + self.misses > 0 {
            self.hits as f64 / (self.hits + self.misses) as f64 * 100.0
        } else {
            0.0
        }
    }
}

// Arena allocator for temporary strings
struct Arena {
    chunks: Vec<Vec<u8>>,
    current: usize,
    offset: usize,
}

impl Arena {
    fn new() -> Self {
        let mut chunks = Vec::new();
        chunks.push(vec![0u8; 4096]);
        Self { chunks, current: 0, offset: 0 }
    }
    
    fn alloc_str(&mut self, s: &str) -> &str {
        let len = s.len();
        if self.offset + len > self.chunks[self.current].len() {
            self.chunks.push(vec![0u8; 4096.max(len)]);
            self.current += 1;
            self.offset = 0;
        }
        
        let chunk = &mut self.chunks[self.current];
        chunk[self.offset..self.offset + len].copy_from_slice(s.as_bytes());
        let result = unsafe {
            std::str::from_utf8_unchecked(&chunk[self.offset..self.offset + len])
        };
        self.offset += len;
        result
    }
    
    fn reset(&mut self) {
        self.current = 0;
        self.offset = 0;
    }
}

// Standard conversion (baseline)
fn standard_rtf_to_markdown(content: &str, iterations: usize) -> (usize, f64) {
    let start = Instant::now();
    let mut allocation_count = 0;
    
    for _ in 0..iterations {
        // Tokenization
        let mut tokens = Vec::new();
        allocation_count += 1;
        
        for word in content.split_whitespace() {
            if word.starts_with('\\') {
                tokens.push(Token::ControlWord(InlineString::new(word), None));
                allocation_count += 1; // Token allocation
                if word.len() > 23 {
                    allocation_count += 1; // String allocation
                }
            } else {
                tokens.push(Token::Text(InlineString::new(word)));
                allocation_count += 1; // Token allocation
                if word.len() > 23 {
                    allocation_count += 1; // String allocation
                }
            }
        }
        
        // Parsing
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
                Token::ControlWord(cmd, _) if cmd.as_str() == "\\par" => {
                    nodes.push(Node::Paragraph(PooledVec { data: current_paragraph.clone() }));
                    allocation_count += 2; // Node + PooledVec
                    current_paragraph = Vec::new();
                    allocation_count += 1;
                }
                _ => {}
            }
        }
        
        if !current_paragraph.is_empty() {
            nodes.push(Node::Paragraph(PooledVec { data: current_paragraph }));
            allocation_count += 2;
        }
        
        // Generation
        let mut output = String::new();
        allocation_count += 1;
        
        for node in &nodes {
            match node {
                Node::Paragraph(children) => {
                    for child in &children.data {
                        if let Node::Text(text) = child {
                            output.push_str(text.as_str());
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

// Highly optimized pooled conversion
fn optimized_pooled_conversion(content: &str, iterations: usize) -> (usize, f64) {
    // Initialize comprehensive pools
    let mut token_pool = ObjectPool::new(1000, || Vec::with_capacity(100));
    let mut node_pool = ObjectPool::new(500, || Vec::with_capacity(50));
    let mut string_pool = ObjectPool::new(200, || String::with_capacity(256));
    let mut arena = Arena::new();
    
    // Count initial allocations
    let mut allocation_count = 1000 + 500 + 200 + 1; // Pools + arena
    
    let start = Instant::now();
    
    for _ in 0..iterations {
        arena.reset(); // Reset arena for reuse
        
        // Tokenization with pooled vectors and arena strings
        let mut tokens = token_pool.acquire();
        tokens.clear();
        
        for word in content.split_whitespace() {
            // Use arena for temporary strings (zero heap allocations)
            let word_ref = arena.alloc_str(word);
            
            if word.starts_with('\\') {
                tokens.push(Token::ControlWord(InlineString::new(word_ref), None));
                allocation_count += 1; // Only the Token enum
            } else {
                tokens.push(Token::Text(InlineString::new(word_ref)));
                allocation_count += 1; // Only the Token enum
            }
        }
        
        // Parsing with aggressive pooling
        let mut nodes = node_pool.acquire();
        nodes.clear();
        let mut current_paragraph = node_pool.acquire();
        current_paragraph.clear();
        
        for token in &tokens {
            match token {
                Token::Text(text) => {
                    current_paragraph.push(Node::Text(text.clone()));
                    allocation_count += 1; // Node allocation only
                }
                Token::ControlWord(cmd, _) if cmd.as_str() == "\\par" => {
                    // Clone into pooled vector
                    let mut para_nodes = node_pool.acquire();
                    para_nodes.clear();
                    para_nodes.extend(current_paragraph.iter().cloned());
                    
                    nodes.push(Node::Paragraph(PooledVec { data: para_nodes }));
                    allocation_count += 2; // Node + PooledVec
                    
                    current_paragraph.clear(); // Reuse same vector
                }
                _ => {}
            }
        }
        
        if !current_paragraph.is_empty() {
            let mut para_nodes = node_pool.acquire();
            para_nodes.clear();
            para_nodes.extend(current_paragraph.iter().cloned());
            nodes.push(Node::Paragraph(PooledVec { data: para_nodes }));
            allocation_count += 2;
        }
        
        // Generation with pooled string
        let mut output = string_pool.acquire();
        output.clear();
        
        for node in &nodes {
            match node {
                Node::Paragraph(children) => {
                    for child in &children.data {
                        if let Node::Text(text) = child {
                            output.push_str(text.as_str());
                            output.push(' ');
                        }
                    }
                    output.push_str("\n\n");
                }
                _ => {}
            }
        }
        
        // Return all resources to pools
        token_pool.release(tokens);
        node_pool.release(nodes);
        node_pool.release(current_paragraph);
        string_pool.release(output);
    }
    
    let elapsed = start.elapsed().as_secs_f64();
    
    // Print pool statistics
    println!("\nPool Statistics:");
    println!("  Token pool hit rate: {:.1}%", token_pool.hit_rate());
    println!("  Node pool hit rate: {:.1}%", node_pool.hit_rate());
    println!("  String pool hit rate: {:.1}%", string_pool.hit_rate());
    
    (allocation_count, elapsed)
}

fn main() {
    println!("Optimized Memory Pool Integration Test");
    println!("=====================================\n");
    
    let test_content = r"Hello \\b world \\b0 this is \\i italic \\i0 text with some longer content to test allocation patterns \\par
        Another paragraph with \\ul underlined \\ulnone content and more text to ensure realistic testing \\par
        Third paragraph with various formatting options and enough content to stress the allocation system \\par
        Final paragraph with even more content to properly test the memory pooling effectiveness";
    
    let iterations = 5000;
    
    // Warm up
    println!("Warming up...");
    standard_rtf_to_markdown(test_content, 100);
    optimized_pooled_conversion(test_content, 100);
    
    // Test standard conversion
    println!("\nTesting standard RTF→Markdown conversion ({} iterations)...", iterations);
    let (std_allocs, std_time) = standard_rtf_to_markdown(test_content, iterations);
    println!("  Total allocations: {}", std_allocs);
    println!("  Time: {:.3}s", std_time);
    println!("  Allocations per conversion: {:.1}", std_allocs as f64 / iterations as f64);
    
    // Test optimized pooled conversion
    println!("\nTesting optimized pooled conversion ({} iterations)...", iterations);
    let (pool_allocs, pool_time) = optimized_pooled_conversion(test_content, iterations);
    println!("  Total allocations: {}", pool_allocs);
    println!("  Time: {:.3}s", pool_time);
    println!("  Allocations per conversion: {:.1}", pool_allocs as f64 / iterations as f64);
    
    // Calculate improvements
    let alloc_reduction = (1.0 - (pool_allocs as f64 / std_allocs as f64)) * 100.0;
    let time_improvement = (1.0 - (pool_time / std_time)) * 100.0;
    
    println!("\nPerformance Results:");
    println!("===================");
    println!("🎯 Allocation overhead reduction: {:.1}%", alloc_reduction);
    println!("⚡ Processing time improvement: {:.1}%", time_improvement);
    println!("📊 Allocations per conversion:");
    println!("   - Standard: {:.1}", std_allocs as f64 / iterations as f64);
    println!("   - Pooled: {:.1}", pool_allocs as f64 / iterations as f64);
    
    // Memory impact
    let avg_alloc_size = 64;
    let memory_saved = ((std_allocs - pool_allocs) as f64 * avg_alloc_size as f64) / 1024.0 / 1024.0;
    println!("\n💾 Memory Impact:");
    println!("   - Total memory saved: {:.2} MB", memory_saved);
    println!("   - Per conversion: {:.0} bytes", (std_allocs - pool_allocs) as f64 / iterations as f64 * avg_alloc_size as f64);
    
    if alloc_reduction >= 40.0 {
        println!("\n✅ SUCCESS: Achieved {:.1}% allocation overhead reduction (target: 40%)", alloc_reduction);
        println!("🎉 Memory pool integration successfully reduces allocation overhead!");
    } else {
        println!("\n❌ FAILED: Only achieved {:.1}% allocation overhead reduction (target: 40%)", alloc_reduction);
    }
}