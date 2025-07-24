// Memory profiling comparison between original and optimized implementations
use std::time::Instant;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::borrow::Cow;

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

// Original implementation simulation
fn simulate_original_processing(doc: &str) -> Vec<String> {
    let mut fragments = Vec::new();
    
    for line in doc.lines() {
        // Simulate the original behavior with excessive cloning
        if line.contains("**") {
            fragments.push(line.to_string()); // Clone
        }
        if line.contains("*") && !line.contains("**") {
            fragments.push(line.to_string()); // Clone
        }
        if line.contains("`") {
            fragments.push(line.to_string()); // Clone
        }
        if line.starts_with("#") {
            fragments.push(line.to_string()); // Clone
        }
        if line.starts_with("-") || line.starts_with("  -") {
            fragments.push(line.to_string()); // Clone
        }
        if line.contains("|") {
            fragments.push(line.to_string()); // Clone
        }
    }
    
    // Simulate RTF escaping with cloning
    let mut escaped_fragments = Vec::new();
    for fragment in &fragments {
        let escaped = fragment.chars()
            .map(|ch| match ch {
                '\\' => "\\\\".to_string(),
                '{' => "\\{".to_string(),
                '}' => "\\}".to_string(),
                '\n' => "\\par ".to_string(),
                c => c.to_string(),
            })
            .collect::<String>();
        escaped_fragments.push(escaped);
    }
    
    escaped_fragments
}

// Optimized implementation using Cow<str>
fn simulate_optimized_processing<'a>(doc: &'a str) -> Vec<Cow<'a, str>> {
    let mut fragments = Vec::new();
    
    for line in doc.lines() {
        // Use Cow<str> to avoid cloning when possible
        if line.contains("**") || line.contains("*") || line.contains("`") || 
           line.starts_with("#") || line.starts_with("-") || line.starts_with("  -") || 
           line.contains("|") {
            fragments.push(Cow::Borrowed(line)); // Zero-copy
        }
    }
    
    // Simulate optimized RTF escaping
    let mut escaped_fragments = Vec::new();
    for fragment in fragments {
        // Check if escaping is needed
        let needs_escape = fragment.chars().any(|ch| matches!(ch, '\\' | '{' | '}' | '\n'));
        
        if !needs_escape {
            escaped_fragments.push(fragment); // Zero-copy pass-through
        } else {
            // Only allocate when necessary
            let mut escaped = String::with_capacity(fragment.len() + fragment.len() / 4);
            for ch in fragment.chars() {
                match ch {
                    '\\' => escaped.push_str("\\\\"),
                    '{' => escaped.push_str("\\{"),
                    '}' => escaped.push_str("\\}"),
                    '\n' => escaped.push_str("\\par "),
                    c => escaped.push(c),
                }
            }
            escaped_fragments.push(Cow::Owned(escaped));
        }
    }
    
    escaped_fragments
}

fn measure_memory_operations() {
    println!("\n=== Memory Usage Comparison: Original vs Optimized ===\n");
    
    let doc_sizes = vec![10, 100, 1000];
    
    for size in doc_sizes {
        let doc = generate_test_document(size);
        println!("Document with {} paragraphs ({} KB):", size, doc.len() / 1024);
        
        // Test original implementation
        ALLOCATED.store(0, Ordering::SeqCst);
        DEALLOCATED.store(0, Ordering::SeqCst);
        
        let start = Instant::now();
        let _original_result = simulate_original_processing(&doc);
        let original_time = start.elapsed();
        
        let original_allocated = ALLOCATED.load(Ordering::SeqCst);
        let original_deallocated = DEALLOCATED.load(Ordering::SeqCst);
        let original_net = original_allocated.saturating_sub(original_deallocated);
        
        println!("  Original implementation:");
        println!("    Time: {:.2}ms", original_time.as_secs_f64() * 1000.0);
        println!("    Allocated: {} KB", original_allocated / 1024);
        println!("    Net memory: {} KB", original_net / 1024);
        
        // Test optimized implementation
        ALLOCATED.store(0, Ordering::SeqCst);
        DEALLOCATED.store(0, Ordering::SeqCst);
        
        let start = Instant::now();
        let _optimized_result = simulate_optimized_processing(&doc);
        let optimized_time = start.elapsed();
        
        let optimized_allocated = ALLOCATED.load(Ordering::SeqCst);
        let optimized_deallocated = DEALLOCATED.load(Ordering::SeqCst);
        let optimized_net = optimized_allocated.saturating_sub(optimized_deallocated);
        
        println!("  Optimized implementation:");
        println!("    Time: {:.2}ms", optimized_time.as_secs_f64() * 1000.0);
        println!("    Allocated: {} KB", optimized_allocated / 1024);
        println!("    Net memory: {} KB", optimized_net / 1024);
        
        // Calculate improvements
        let memory_reduction = if original_allocated > 0 {
            ((original_allocated as f64 - optimized_allocated as f64) / original_allocated as f64) * 100.0
        } else {
            0.0
        };
        
        let speed_improvement = if original_time.as_nanos() > 0 {
            ((original_time.as_nanos() as f64 - optimized_time.as_nanos() as f64) / original_time.as_nanos() as f64) * 100.0
        } else {
            0.0
        };
        
        println!("  Improvements:");
        println!("    Memory reduction: {:.1}%", memory_reduction);
        println!("    Speed improvement: {:.1}%", speed_improvement);
        println!();
    }
}

fn main() {
    measure_memory_operations();
}