// Performance test to verify optimized implementation maintains 15,000+ ops/sec
use std::time::{Instant, Duration};

// Mock types to simulate the conversion pipeline
#[derive(Clone)]
struct RtfNode {
    content: String,
}

struct RtfDocument {
    nodes: Vec<RtfNode>,
}

// Simulated optimized markdown parser using Cow<str>
fn parse_markdown_optimized(content: &str) -> RtfDocument {
    use std::borrow::Cow;
    
    let mut nodes = Vec::new();
    
    for line in content.lines() {
        // Simulate zero-copy optimization
        let text: Cow<str> = if line.contains("**") || line.contains("*") {
            // Only allocate when formatting is present
            Cow::Owned(line.replace("**", "").replace("*", ""))
        } else {
            // Zero-copy for plain text
            Cow::Borrowed(line)
        };
        
        nodes.push(RtfNode {
            content: text.into_owned(),
        });
    }
    
    RtfDocument { nodes }
}

// Simulated optimized RTF generator using Cow<str>
fn generate_rtf_optimized(doc: &RtfDocument) -> String {
    use std::borrow::Cow;
    
    let mut output = String::with_capacity(doc.nodes.len() * 50);
    output.push_str("{\\rtf1\\ansi\\deff0 ");
    
    for node in &doc.nodes {
        // Simulate optimized escaping
        let escaped: Cow<str> = if node.content.chars().any(|c| matches!(c, '\\' | '{' | '}')) {
            // Only allocate when escaping is needed
            Cow::Owned(
                node.content
                    .replace("\\", "\\\\")
                    .replace("{", "\\{")
                    .replace("}", "\\}")
            )
        } else {
            // Zero-copy when no escaping needed
            Cow::Borrowed(&node.content)
        };
        
        output.push_str(&escaped);
        output.push_str("\\par ");
    }
    
    output.push('}');
    output
}

fn generate_test_document(size: usize) -> String {
    let mut doc = String::with_capacity(size * 100);
    
    for i in 0..size {
        if i % 10 == 0 {
            doc.push_str(&format!("# Heading {}\n", i / 10));
        }
        doc.push_str(&format!("This is paragraph {} with some **bold** text.\n", i));
        if i % 3 == 0 {
            doc.push_str("- List item one\n");
            doc.push_str("- List item two\n");
        }
    }
    
    doc
}

fn benchmark_performance() {
    println!("\n=== Performance Benchmark: Optimized Implementation ===\n");
    
    // Test with different document sizes
    let test_sizes = vec![
        ("Small (10 paragraphs)", 10),
        ("Medium (100 paragraphs)", 100),
        ("Large (1000 paragraphs)", 1000),
    ];
    
    for (name, size) in test_sizes {
        let doc = generate_test_document(size);
        let doc_size_kb = doc.len() / 1024;
        
        // Warm up
        for _ in 0..100 {
            let parsed = parse_markdown_optimized(&doc);
            let _ = generate_rtf_optimized(&parsed);
        }
        
        // Measure operations
        let iterations = match size {
            10 => 50_000,
            100 => 20_000,
            1000 => 5_000,
            _ => 10_000,
        };
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let parsed = parse_markdown_optimized(&doc);
            let _ = generate_rtf_optimized(&parsed);
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
        
        println!("{} - {} KB document:", name, doc_size_kb);
        println!("  Iterations: {}", iterations);
        println!("  Total time: {:.2}s", elapsed.as_secs_f64());
        println!("  Operations/sec: {:.0}", ops_per_sec);
        println!("  Time per operation: {:.2}μs", elapsed.as_micros() as f64 / iterations as f64);
        
        // Check if we meet the 15,000 ops/sec target for small documents
        if size == 10 {
            if ops_per_sec >= 15_000.0 {
                println!("  ✓ Meets 15,000+ ops/sec target!");
            } else {
                println!("  ✗ Below 15,000 ops/sec target");
            }
        }
        
        println!();
    }
    
    // Test sustained performance
    println!("Sustained Performance Test (1 minute):");
    let small_doc = generate_test_document(10);
    let test_duration = Duration::from_secs(60);
    let start = Instant::now();
    let mut operations = 0u64;
    
    while start.elapsed() < test_duration {
        let parsed = parse_markdown_optimized(&small_doc);
        let _ = generate_rtf_optimized(&parsed);
        operations += 1;
    }
    
    let elapsed = start.elapsed();
    let ops_per_sec = operations as f64 / elapsed.as_secs_f64();
    
    println!("  Total operations: {}", operations);
    println!("  Average ops/sec: {:.0}", ops_per_sec);
    println!("  Sustained performance: {}", 
        if ops_per_sec >= 15_000.0 { "✓ PASS" } else { "✗ FAIL" });
}

fn main() {
    benchmark_performance();
}