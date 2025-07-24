use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use legacybridge::conversion::{markdown_parser::MarkdownParser, rtf_generator::RtfGenerator};
use std::time::Duration;
use std::fs;
use std::path::Path;

// Test document sizes
const SIZES: &[(usize, &str)] = &[
    (10 * 1024, "10KB"),
    (100 * 1024, "100KB"),
    (1024 * 1024, "1MB"),
    (10 * 1024 * 1024, "10MB"),
];

// Performance targets (in milliseconds)
struct PerformanceTarget {
    size_bytes: usize,
    max_duration_ms: u64,
}

const PERFORMANCE_TARGETS: &[PerformanceTarget] = &[
    PerformanceTarget { size_bytes: 10 * 1024, max_duration_ms: 1 },      // 10KB < 1ms
    PerformanceTarget { size_bytes: 100 * 1024, max_duration_ms: 10 },   // 100KB < 10ms
    PerformanceTarget { size_bytes: 1024 * 1024, max_duration_ms: 100 }, // 1MB < 100ms
    PerformanceTarget { size_bytes: 10 * 1024 * 1024, max_duration_ms: 1000 }, // 10MB < 1s
];

// Generate test documents
fn generate_markdown_document(size: usize) -> String {
    let mut doc = String::with_capacity(size);
    
    doc.push_str("# Performance Test Document\n\n");
    doc.push_str("## Executive Summary\n\n");
    doc.push_str("This document is generated for performance testing purposes.\n\n");
    
    let paragraph = "This is a test paragraph with **bold text**, *italic text*, and `inline code`. \
                     It contains [links](https://example.com) and various formatting options. \
                     The paragraph is designed to be representative of real-world content.\n\n";
    
    let paragraph_size = paragraph.len();
    let paragraphs_needed = (size - doc.len()) / paragraph_size;
    
    for i in 0..paragraphs_needed {
        if i % 20 == 0 {
            doc.push_str(&format!("### Section {}\n\n", i / 20 + 1));
        }
        
        doc.push_str(paragraph);
        
        // Add variety to the content
        if i % 5 == 0 {
            doc.push_str("- List item one with **emphasis**\n");
            doc.push_str("- List item two with *subtlety*\n");
            doc.push_str("  - Nested item for complexity\n");
            doc.push_str("- List item three\n\n");
        }
        
        if i % 10 == 0 {
            doc.push_str("| Column 1 | Column 2 | Column 3 |\n");
            doc.push_str("|----------|----------|----------|\n");
            doc.push_str("| Data 1   | Data 2   | Data 3   |\n");
            doc.push_str("| Data 4   | Data 5   | Data 6   |\n\n");
        }
    }
    
    doc
}

fn generate_rtf_document(size: usize) -> String {
    let header = r"{\rtf1\ansi\deff0 {\fonttbl{\f0 Times New Roman;}}";
    let footer = "}";
    let mut content = String::from(header);
    
    let paragraph = r"\f0\fs24 This is a test paragraph with \b bold\b0 and \i italic\i0 text. ";
    let paragraph_size = paragraph.len();
    let paragraphs_needed = (size - header.len() - footer.len()) / paragraph_size;
    
    for i in 0..paragraphs_needed {
        content.push_str(paragraph);
        if i % 10 == 0 {
            content.push_str(r"\par\par");
        } else {
            content.push_str(r"\par ");
        }
    }
    
    content.push_str(footer);
    content
}

// Benchmark RTF to Markdown conversion
fn benchmark_rtf_to_markdown(c: &mut Criterion) {
    let mut group = c.benchmark_group("rtf_to_markdown");
    group.measurement_time(Duration::from_secs(10));
    
    for &(size, name) in SIZES {
        let rtf_content = generate_rtf_document(size);
        
        group.bench_with_input(
            BenchmarkId::new("parse", name),
            &rtf_content,
            |b, content| {
                b.iter(|| {
                    let result = black_box(legacybridge::convert_rtf_to_markdown(black_box(content)));
                    assert!(result.is_ok());
                })
            },
        );
    }
    
    group.finish();
}

// Benchmark Markdown to RTF conversion
fn benchmark_markdown_to_rtf(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown_to_rtf");
    group.measurement_time(Duration::from_secs(10));
    
    for &(size, name) in SIZES {
        let markdown_content = generate_markdown_document(size);
        
        group.bench_with_input(
            BenchmarkId::new("generate", name),
            &markdown_content,
            |b, content| {
                b.iter(|| {
                    let result = black_box(legacybridge::convert_markdown_to_rtf(black_box(content)));
                    assert!(result.is_ok());
                })
            },
        );
    }
    
    group.finish();
}

// Benchmark memory allocation patterns
fn benchmark_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");
    
    // Test string building strategies
    group.bench_function("string_push", |b| {
        b.iter(|| {
            let mut s = String::new();
            for i in 0..1000 {
                s.push_str(&format!("Item {}", i));
            }
            black_box(s);
        })
    });
    
    group.bench_function("string_with_capacity", |b| {
        b.iter(|| {
            let mut s = String::with_capacity(10000);
            for i in 0..1000 {
                s.push_str(&format!("Item {}", i));
            }
            black_box(s);
        })
    });
    
    group.bench_function("vec_collect", |b| {
        b.iter(|| {
            let s: String = (0..1000)
                .map(|i| format!("Item {}", i))
                .collect();
            black_box(s);
        })
    });
    
    group.finish();
}

// Benchmark concurrent processing
fn benchmark_concurrent_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_processing");
    group.measurement_time(Duration::from_secs(20));
    
    let doc_100kb = generate_markdown_document(100 * 1024);
    let docs: Vec<String> = (0..100).map(|_| doc_100kb.clone()).collect();
    
    group.bench_function("sequential", |b| {
        b.iter(|| {
            for doc in &docs {
                let _ = black_box(legacybridge::convert_markdown_to_rtf(black_box(doc)));
            }
        })
    });
    
    group.bench_function("parallel_rayon", |b| {
        use rayon::prelude::*;
        b.iter(|| {
            docs.par_iter()
                .map(|doc| black_box(legacybridge::convert_markdown_to_rtf(black_box(doc))))
                .collect::<Vec<_>>();
        })
    });
    
    group.finish();
}

// Benchmark edge cases
fn benchmark_edge_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("edge_cases");
    
    // Very long single line
    let long_line = "word ".repeat(10000);
    group.bench_function("long_single_line", |b| {
        b.iter(|| {
            let _ = black_box(legacybridge::convert_markdown_to_rtf(black_box(&long_line)));
        })
    });
    
    // Many short lines
    let many_lines = "line\n".repeat(10000);
    group.bench_function("many_short_lines", |b| {
        b.iter(|| {
            let _ = black_box(legacybridge::convert_markdown_to_rtf(black_box(&many_lines)));
        })
    });
    
    // Deeply nested lists
    let mut nested = String::new();
    for i in 0..50 {
        nested.push_str(&"  ".repeat(i));
        nested.push_str("- Item\n");
    }
    group.bench_function("deeply_nested", |b| {
        b.iter(|| {
            let _ = black_box(legacybridge::convert_markdown_to_rtf(black_box(&nested)));
        })
    });
    
    // Heavy formatting
    let formatted = "**bold** *italic* `code` ***both*** ".repeat(1000);
    group.bench_function("heavy_formatting", |b| {
        b.iter(|| {
            let _ = black_box(legacybridge::convert_markdown_to_rtf(black_box(&formatted)));
        })
    });
    
    group.finish();
}

// Performance regression detection
fn verify_performance_targets(c: &mut Criterion) {
    println!("\n=== Performance Target Verification ===\n");
    
    let mut all_passed = true;
    
    for target in PERFORMANCE_TARGETS {
        let doc = generate_markdown_document(target.size_bytes);
        
        // Warm up
        for _ in 0..3 {
            let _ = legacybridge::convert_markdown_to_rtf(&doc);
        }
        
        // Measure
        let start = std::time::Instant::now();
        let _ = legacybridge::convert_markdown_to_rtf(&doc);
        let duration = start.elapsed();
        
        let passed = duration.as_millis() <= target.max_duration_ms as u128;
        all_passed &= passed;
        
        println!(
            "{}: {} (target: <{}ms) {}",
            format_size(target.size_bytes),
            format_duration(duration),
            target.max_duration_ms,
            if passed { "✓ PASS" } else { "✗ FAIL" }
        );
    }
    
    if !all_passed {
        panic!("Performance regression detected! Some targets were not met.");
    }
}

// Helper functions
fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{}KB", bytes / 1024)
    } else {
        format!("{}MB", bytes / (1024 * 1024))
    }
}

fn format_duration(duration: Duration) -> String {
    let millis = duration.as_secs_f64() * 1000.0;
    if millis < 1.0 {
        format!("{:.2}μs", duration.as_secs_f64() * 1_000_000.0)
    } else {
        format!("{:.2}ms", millis)
    }
}

// Benchmark groups
criterion_group!(
    benches,
    benchmark_rtf_to_markdown,
    benchmark_markdown_to_rtf,
    benchmark_memory_patterns,
    benchmark_concurrent_processing,
    benchmark_edge_cases
);

criterion_main!(benches);

// Custom benchmark runner for CI/CD integration
#[cfg(test)]
mod regression_tests {
    use super::*;
    
    #[test]
    fn test_performance_targets() {
        verify_performance_targets(&mut Criterion::default());
    }
    
    #[test]
    fn test_memory_usage() {
        // Test that memory usage stays within bounds
        let doc_1mb = generate_markdown_document(1024 * 1024);
        
        let before = get_current_memory_usage();
        let _ = legacybridge::convert_markdown_to_rtf(&doc_1mb);
        let after = get_current_memory_usage();
        
        let memory_increase_mb = (after - before) as f64 / (1024.0 * 1024.0);
        assert!(memory_increase_mb < 100.0, "Memory usage exceeded 100MB limit");
    }
    
    fn get_current_memory_usage() -> usize {
        // Platform-specific memory usage measurement
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            let status = fs::read_to_string("/proc/self/status").unwrap();
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    return parts[1].parse::<usize>().unwrap() * 1024; // Convert KB to bytes
                }
            }
        }
        0
    }
}