// Simple memory profiling tool to measure string cloning overhead
use std::time::Instant;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// Custom allocator to track memory usage
struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            ALLOCATED.fetch_add(size, Ordering::SeqCst);
        }
        ptr
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        System.dealloc(ptr, layout);
        DEALLOCATED.fetch_add(size, Ordering::SeqCst);
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

fn generate_test_document(paragraphs: usize) -> String {
    let mut doc = String::with_capacity(paragraphs * 500);
    
    doc.push_str("# Enterprise Document Processing System\n\n");
    doc.push_str("## Executive Summary\n\n");
    doc.push_str("This document demonstrates the performance characteristics of our optimized RTF conversion pipeline.\n\n");
    
    for i in 0..paragraphs {
        // Section header
        if i % 10 == 0 {
            doc.push_str(&format!("### Section {}\n\n", i / 10 + 1));
        }
        
        // Mixed content paragraph
        doc.push_str(&format!(
            "Paragraph {} contains **bold text**, *italic text*, and `inline code`. \
            It demonstrates various formatting options including [links](https://example.com/page{}) \
            and complex nested structures. This ensures realistic processing loads.\n\n",
            i + 1, i
        ));
        
        // Add lists periodically
        if i % 5 == 0 {
            doc.push_str("Key points:\n");
            doc.push_str("- First point with **emphasis**\n");
            doc.push_str("- Second point with *subtlety*\n");
            doc.push_str("  - Nested point for complexity\n");
            doc.push_str("  - Another nested point\n");
            doc.push_str("- Third point for completeness\n\n");
        }
        
        // Add tables periodically
        if i % 7 == 0 {
            doc.push_str("| Metric | Value | Status |\n");
            doc.push_str("|--------|-------|--------|\n");
            doc.push_str(&format!("| Performance | {}ms | Good |\n", i * 2));
            doc.push_str(&format!("| Memory | {}MB | Optimal |\n", i / 10));
            doc.push_str(&format!("| CPU | {}% | Normal |\n\n", 25 + i % 50));
        }
    }
    
    doc
}

fn measure_string_operations() {
    println!("\n=== String Operation Memory Profiling ===\n");
    
    // Test 1: String cloning
    println!("Test 1: String Cloning");
    let medium_string = "x".repeat(1000);
    let large_string = "x".repeat(10000);
    let test_strings = vec![
        ("Small", "This is a small test string"),
        ("Medium", medium_string.as_str()),
        ("Large", large_string.as_str()),
    ];
    
    for (name, text) in test_strings {
        ALLOCATED.store(0, Ordering::SeqCst);
        DEALLOCATED.store(0, Ordering::SeqCst);
        
        let iterations = 1000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _cloned = text.to_string();
            // Force drop to measure deallocation
            std::mem::drop(_cloned);
        }
        
        let elapsed = start.elapsed();
        let allocated = ALLOCATED.load(Ordering::SeqCst);
        let deallocated = DEALLOCATED.load(Ordering::SeqCst);
        
        println!("  {} string ({} bytes):", name, text.len());
        println!("    Time: {:.2}ms", elapsed.as_secs_f64() * 1000.0);
        println!("    Allocated: {} KB", allocated / 1024);
        println!("    Deallocated: {} KB", deallocated / 1024);
        println!("    Net memory: {} KB", (allocated - deallocated) / 1024);
        println!("    Per operation: {} bytes\n", allocated / iterations);
    }
    
    // Test 2: Document processing simulation
    println!("Test 2: Document Processing Simulation");
    let doc_sizes = vec![10, 100, 1000];
    
    for size in doc_sizes {
        let doc = generate_test_document(size);
        
        ALLOCATED.store(0, Ordering::SeqCst);
        DEALLOCATED.store(0, Ordering::SeqCst);
        
        let start = Instant::now();
        
        // Simulate typical string operations in parsing
        let mut fragments = Vec::new();
        for line in doc.lines() {
            // Simulate parsing operations that clone strings
            if line.contains("**") {
                fragments.push(line.to_string());
            }
            if line.contains("*") && !line.contains("**") {
                fragments.push(line.to_string());
            }
            if line.contains("`") {
                fragments.push(line.to_string());
            }
            if line.starts_with("#") {
                fragments.push(line.to_string());
            }
            if line.starts_with("-") || line.starts_with("  -") {
                fragments.push(line.to_string());
            }
            if line.contains("|") {
                fragments.push(line.to_string());
            }
        }
        
        let elapsed = start.elapsed();
        let allocated = ALLOCATED.load(Ordering::SeqCst);
        let deallocated = DEALLOCATED.load(Ordering::SeqCst);
        
        println!("  Document with {} paragraphs ({} KB):", size, doc.len() / 1024);
        println!("    Time: {:.2}ms", elapsed.as_secs_f64() * 1000.0);
        println!("    Fragments collected: {}", fragments.len());
        println!("    Allocated: {} KB", allocated / 1024);
        println!("    Deallocated: {} KB", deallocated / 1024);
        println!("    Net memory: {} KB", (allocated - deallocated) / 1024);
        println!("    Memory per fragment: {} bytes\n", 
                if fragments.len() > 0 { allocated / fragments.len() } else { 0 });
    }
}

fn main() {
    measure_string_operations();
}