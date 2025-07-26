// Performance validation test for LegacyBridge
// Testing the claimed 41,000+ conversions/second

use std::time::{Duration, Instant};
use std::fs;

const SMALL_DOC_SIZE: usize = 100; // 100 bytes
const MEDIUM_DOC_SIZE: usize = 10_000; // 10KB
const LARGE_DOC_SIZE: usize = 100_000; // 100KB

fn generate_markdown(size: usize) -> String {
    let mut doc = String::with_capacity(size);
    doc.push_str("# Test Document\n\n");
    
    let paragraph = "This is a test paragraph with **bold** and *italic* text. ";
    while doc.len() < size {
        doc.push_str(paragraph);
    }
    
    doc.truncate(size);
    doc
}

fn simulate_conversion(doc: &str) -> String {
    // Simulate a basic markdown to RTF conversion
    // Real conversion would involve parsing and formatting
    let mut rtf = String::with_capacity(doc.len() * 2);
    rtf.push_str("{\\rtf1\\ansi\\deff0 {\\fonttbl {\\f0 Times New Roman;}}");
    
    // Simple simulation - just process each character
    for ch in doc.chars() {
        match ch {
            '*' => rtf.push_str("\\b "),
            '\n' => rtf.push_str("\\par "),
            _ => rtf.push(ch),
        }
    }
    
    rtf.push_str("}");
    rtf
}

fn benchmark_conversions(doc_size: usize, iterations: usize) -> (Duration, f64) {
    let doc = generate_markdown(doc_size);
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = simulate_conversion(&doc);
    }
    let duration = start.elapsed();
    
    let ops_per_second = iterations as f64 / duration.as_secs_f64();
    (duration, ops_per_second)
}

fn main() {
    println!("LegacyBridge Performance Validation Test");
    println!("========================================\n");
    
    // Test different document sizes
    let test_cases = vec![
        ("Small (100B)", SMALL_DOC_SIZE, 10_000),
        ("Medium (10KB)", MEDIUM_DOC_SIZE, 1_000),
        ("Large (100KB)", LARGE_DOC_SIZE, 100),
    ];
    
    for (name, size, iterations) in test_cases {
        println!("Testing {} documents ({} iterations):", name, iterations);
        let (duration, ops_per_sec) = benchmark_conversions(size, iterations);
        
        println!("  Time: {:?}", duration);
        println!("  Operations/second: {:.0}", ops_per_sec);
        println!("  Time per operation: {:.3}ms", 1000.0 / ops_per_sec);
        
        // Check against claimed performance
        if size == SMALL_DOC_SIZE {
            println!("  vs Claimed 41,000 ops/sec: {:.1}%", (ops_per_sec / 41_000.0) * 100.0);
        }
        println!();
    }
    
    // Theoretical maximum test
    println!("Theoretical Maximum Test (minimal processing):");
    let mut count = 0u64;
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(1) {
        count += 1;
        // Minimal work - just increment
        std::hint::black_box(count);
    }
    println!("  Empty loop iterations/sec: {}", count);
    
    // Memory allocation test
    println!("\nMemory Allocation Test:");
    let start = Instant::now();
    let mut allocations = 0;
    while start.elapsed() < Duration::from_secs(1) {
        let _s = String::with_capacity(1000);
        allocations += 1;
    }
    println!("  String allocations/sec: {}", allocations);
}