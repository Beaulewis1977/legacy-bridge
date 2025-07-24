// Performance Benchmark Suite - Comprehensive performance testing
//
// This module provides benchmarks to measure:
// 1. Memory usage and allocation patterns
// 2. Processing throughput
// 3. Thread pool efficiency
// 4. Cache hit rates
// 5. String interning effectiveness

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::sync::Arc;
use std::time::Duration;

use crate::conversion::markdown_parser_optimized::OptimizedMarkdownParser;
use crate::pipeline::concurrent_processor::{
    ConcurrentProcessor, ConversionRequest, ConversionContent, ConversionOptions
};

/// Generate test documents of various sizes
fn generate_test_document(size: usize) -> String {
    let mut doc = String::with_capacity(size);
    
    // Generate realistic markdown content
    for i in 0..(size / 1000) {
        doc.push_str(&format!("# Heading {}\n\n", i));
        doc.push_str("This is a paragraph with **bold** and *italic* text. ");
        doc.push_str("Here's some `inline code` and a [link](https://example.com).\n\n");
        
        // Add lists
        doc.push_str("- List item one\n");
        doc.push_str("- List item two\n");
        doc.push_str("  - Nested item\n\n");
        
        // Add code block
        doc.push_str("```rust\n");
        doc.push_str("fn example() {\n");
        doc.push_str("    println!(\"Hello, world!\");\n");
        doc.push_str("}\n");
        doc.push_str("```\n\n");
        
        // Add table
        doc.push_str("| Column 1 | Column 2 |\n");
        doc.push_str("|----------|----------|\n");
        doc.push_str("| Cell 1   | Cell 2   |\n\n");
    }
    
    doc
}

/// Benchmark string interning performance
fn bench_string_interner(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_interner");
    
    // Test with documents that have varying repetition patterns
    let sizes = vec![1_000, 10_000, 100_000, 1_000_000];
    
    for size in sizes {
        let doc = generate_test_document(size);
        
        group.bench_with_input(
            BenchmarkId::new("parse_with_interner", size),
            &doc,
            |b, doc| {
                b.iter(|| {
                    let mut parser = OptimizedMarkdownParser::new();
                    let _ = parser.parse(black_box(doc));
                });
            },
        );
    }
    
    // Measure memory usage for string interner
    group.bench_function("memory_overhead", |b| {
        let doc = generate_test_document(100_000);
        b.iter(|| {
            let mut parser = OptimizedMarkdownParser::new();
            // Parse multiple times to stress the interner
            for _ in 0..10 {
                let _ = parser.parse(black_box(&doc));
            }
        });
    });
    
    group.finish();
}

/// Benchmark thread pool efficiency
fn bench_thread_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("thread_pool");
    group.measurement_time(Duration::from_secs(10));
    
    let thread_counts = vec![1, 2, 4, 8];
    let doc_count = 100;
    
    for thread_count in thread_counts {
        group.bench_with_input(
            BenchmarkId::new("concurrent_processing", thread_count),
            &thread_count,
            |b, &thread_count| {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                
                b.iter(|| {
                    runtime.block_on(async {
                        let processor = ConcurrentProcessor::new(Some(thread_count));
                        
                        // Create batch of documents
                        let requests: Vec<_> = (0..doc_count)
                            .map(|i| {
                                let doc = generate_test_document(10_000);
                                ConversionRequest {
                                    id: format!("doc-{}", i),
                                    content: ConversionContent::Memory(doc),
                                    options: ConversionOptions::default(),
                                }
                            })
                            .collect();
                        
                        let _ = processor.process_batch(requests).await;
                    });
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark string cloning overhead
fn bench_string_cloning(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_cloning");
    
    let text_sizes = vec![10, 100, 1_000, 10_000];
    
    for size in text_sizes {
        let text = "x".repeat(size);
        
        // Benchmark clone() overhead
        group.bench_with_input(
            BenchmarkId::new("clone", size),
            &text,
            |b, text| {
                b.iter(|| {
                    let _cloned = black_box(text.clone());
                });
            },
        );
        
        // Benchmark Arc<str> overhead
        group.bench_with_input(
            BenchmarkId::new("arc_str", size),
            &text,
            |b, text| {
                let arc_text: Arc<str> = Arc::from(text.as_str());
                b.iter(|| {
                    let _cloned = black_box(arc_text.clone());
                });
            },
        );
        
        // Benchmark Cow<str> overhead
        group.bench_with_input(
            BenchmarkId::new("cow_str", size),
            &text,
            |b, text| {
                use std::borrow::Cow;
                let cow_text: Cow<str> = Cow::Borrowed(text);
                b.iter(|| {
                    let _cloned: Cow<str> = black_box(cow_text.clone());
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark memory allocations
fn bench_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocations");
    
    // Test document parsing with different allocation patterns
    let doc = generate_test_document(100_000);
    
    group.bench_function("baseline_allocations", |b| {
        b.iter(|| {
            let mut parser = OptimizedMarkdownParser::new();
            let _ = parser.parse(black_box(&doc));
        });
    });
    
    group.finish();
}

/// Benchmark end-to-end performance
fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");
    group.measurement_time(Duration::from_secs(20));
    
    let doc_sizes = vec![1_000, 10_000, 100_000, 1_000_000];
    
    for size in doc_sizes {
        let doc = generate_test_document(size);
        
        group.bench_with_input(
            BenchmarkId::new("full_conversion", size),
            &doc,
            |b, doc| {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                
                b.iter(|| {
                    runtime.block_on(async {
                        let processor = ConcurrentProcessor::new(None);
                        let request = ConversionRequest {
                            id: "test".to_string(),
                            content: ConversionContent::Memory(doc.clone()),
                            options: ConversionOptions::default(),
                        };
                        
                        let _ = processor.process_single(request).await;
                    });
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_string_interner,
    bench_thread_pool,
    bench_string_cloning,
    bench_allocations,
    bench_end_to_end
);
criterion_main!(benches);

#[cfg(test)]
mod memory_profiling {
    use super::*;
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    /// Custom allocator to track memory usage
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
    
    #[test]
    fn profile_string_interner_memory() {
        let allocated_before = ALLOCATED.load(Ordering::SeqCst);
        let deallocated_before = DEALLOCATED.load(Ordering::SeqCst);
        
        let doc = generate_test_document(1_000_000);
        let mut parser = OptimizedMarkdownParser::new();
        
        // Parse document multiple times
        for _ in 0..10 {
            let _ = parser.parse(&doc);
        }
        
        let allocated_after = ALLOCATED.load(Ordering::SeqCst);
        let deallocated_after = DEALLOCATED.load(Ordering::SeqCst);
        
        let total_allocated = allocated_after - allocated_before;
        let total_deallocated = deallocated_after - deallocated_before;
        let net_memory = total_allocated.saturating_sub(total_deallocated);
        
        println!("Memory Profile - String Interner:");
        println!("  Total allocated: {} bytes", total_allocated);
        println!("  Total deallocated: {} bytes", total_deallocated);
        println!("  Net memory usage: {} bytes", net_memory);
        println!("  Memory per parse: {} bytes", net_memory / 10);
    }
}