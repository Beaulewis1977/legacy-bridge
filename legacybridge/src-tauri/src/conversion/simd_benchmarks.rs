// SIMD Optimization Benchmarks
// Comprehensive benchmarking suite for string processing operations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::prelude::*;
use std::time::Duration;

// Import the modules we'll be optimizing
use crate::conversion::rtf_lexer;
use crate::conversion::markdown_parser_optimized::OptimizedMarkdownParser;
use crate::conversion::rtf_parser_optimized::OptimizedRtfParser;

/// Generate test documents of various sizes
fn generate_rtf_document(size_kb: usize) -> String {
    let mut rng = thread_rng();
    let mut doc = String::with_capacity(size_kb * 1024);
    
    // RTF header
    doc.push_str(r"{\rtf1\ansi\deff0 {\fonttbl {\f0 Times New Roman;}}");
    doc.push_str(r"{\colortbl;\red0\green0\blue0;\red255\green0\blue0;}");
    
    // Generate paragraphs
    let paragraph_count = size_kb * 10; // Roughly 10 paragraphs per KB
    for i in 0..paragraph_count {
        doc.push_str(r"\par ");
        
        // Mix of plain text and formatted text
        if i % 3 == 0 {
            doc.push_str(r"{\b Bold text ");
            doc.push_str(&format!("paragraph {} ", i));
            doc.push_str(r"}\par ");
        } else if i % 3 == 1 {
            doc.push_str(r"{\i Italic text ");
            doc.push_str(&format!("content {} ", i));
            doc.push_str(r"}\par ");
        } else {
            doc.push_str("Plain text with special chars: ");
            // Add some special characters that need escaping
            doc.push_str(r"\{braces\} and \backslash ");
            doc.push_str(&format!("number {} ", i));
        }
        
        // Add some Unicode via hex codes
        if i % 5 == 0 {
            doc.push_str(r"\'e9\'e8\'ea "); // French accents
        }
    }
    
    doc.push('}');
    doc
}

fn generate_markdown_document(size_kb: usize) -> String {
    let mut rng = thread_rng();
    let mut doc = String::with_capacity(size_kb * 1024);
    
    let paragraph_count = size_kb * 10;
    for i in 0..paragraph_count {
        match i % 6 {
            0 => {
                doc.push_str(&format!("# Heading {}\n\n", i));
                doc.push_str("This is a paragraph with **bold** and *italic* text.\n\n");
            }
            1 => {
                doc.push_str(&format!("## Subheading {}\n\n", i));
                doc.push_str("A paragraph with `inline code` and [links](http://example.com).\n\n");
            }
            2 => {
                doc.push_str("- List item one\n");
                doc.push_str("- List item two with **emphasis**\n");
                doc.push_str("- List item three\n\n");
            }
            3 => {
                doc.push_str("1. Ordered item\n");
                doc.push_str("2. Another ordered item\n");
                doc.push_str("3. Third item with *italic*\n\n");
            }
            4 => {
                doc.push_str("| Header 1 | Header 2 | Header 3 |\n");
                doc.push_str("|----------|----------|----------|\n");
                doc.push_str("| Cell 1   | Cell 2   | Cell 3   |\n\n");
            }
            _ => {
                doc.push_str("Regular paragraph with ");
                // Add special markdown characters
                doc.push_str("asterisks * and underscores _ and brackets [].\n\n");
            }
        }
    }
    
    doc
}

/// Benchmark RTF character search operations
fn bench_rtf_char_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("rtf_char_search");
    group.measurement_time(Duration::from_secs(10));
    
    for size_kb in [1, 10, 100, 1000].iter() {
        let rtf_doc = generate_rtf_document(*size_kb);
        let bytes = rtf_doc.as_bytes();
        
        group.throughput(Throughput::Bytes(bytes.len() as u64));
        
        // Baseline: Scalar search for RTF control characters
        group.bench_with_input(
            BenchmarkId::new("scalar", size_kb),
            &bytes,
            |b, input| {
                b.iter(|| {
                    let mut count = 0;
                    for byte in input.iter() {
                        match *byte {
                            b'\\' | b'{' | b'}' => count += 1,
                            _ => {}
                        }
                    }
                    black_box(count)
                });
            },
        );
        
        // TODO: Add SIMD implementation benchmark here
    }
    
    group.finish();
}

/// Benchmark RTF tokenization
fn bench_rtf_tokenization(c: &mut Criterion) {
    let mut group = c.benchmark_group("rtf_tokenization");
    group.measurement_time(Duration::from_secs(10));
    
    for size_kb in [1, 10, 100, 1000].iter() {
        let rtf_doc = generate_rtf_document(*size_kb);
        
        group.throughput(Throughput::Bytes(rtf_doc.len() as u64));
        
        group.bench_with_input(
            BenchmarkId::new("current", size_kb),
            &rtf_doc,
            |b, input| {
                b.iter(|| {
                    let tokens = rtf_lexer::tokenize(black_box(input)).unwrap();
                    black_box(tokens)
                });
            },
        );
        
        // TODO: Add SIMD-optimized tokenizer benchmark
    }
    
    group.finish();
}

/// Benchmark Markdown special character detection
fn bench_markdown_char_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown_char_search");
    group.measurement_time(Duration::from_secs(10));
    
    for size_kb in [1, 10, 100, 1000].iter() {
        let md_doc = generate_markdown_document(*size_kb);
        let bytes = md_doc.as_bytes();
        
        group.throughput(Throughput::Bytes(bytes.len() as u64));
        
        // Baseline: Scalar search for Markdown special characters
        group.bench_with_input(
            BenchmarkId::new("scalar", size_kb),
            &bytes,
            |b, input| {
                b.iter(|| {
                    let mut count = 0;
                    for byte in input.iter() {
                        match *byte {
                            b'*' | b'_' | b'#' | b'[' | b']' | b'`' => count += 1,
                            _ => {}
                        }
                    }
                    black_box(count)
                });
            },
        );
        
        // TODO: Add SIMD implementation benchmark
    }
    
    group.finish();
}

/// Benchmark UTF-8 validation
fn bench_utf8_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("utf8_validation");
    group.measurement_time(Duration::from_secs(10));
    
    for size_kb in [1, 10, 100, 1000].iter() {
        let mut text = String::with_capacity(*size_kb * 1024);
        
        // Mix of ASCII and UTF-8
        for i in 0..(size_kb * 100) {
            if i % 10 == 0 {
                text.push_str("café résumé naïve ");
            } else {
                text.push_str("Regular ASCII text ");
            }
        }
        
        let bytes = text.as_bytes();
        group.throughput(Throughput::Bytes(bytes.len() as u64));
        
        // Baseline: Standard library UTF-8 validation
        group.bench_with_input(
            BenchmarkId::new("std", size_kb),
            &bytes,
            |b, input| {
                b.iter(|| {
                    let valid = std::str::from_utf8(black_box(input)).is_ok();
                    black_box(valid)
                });
            },
        );
        
        // TODO: Add SIMD UTF-8 validation benchmark
    }
    
    group.finish();
}

/// Benchmark whitespace normalization
fn bench_whitespace_normalization(c: &mut Criterion) {
    let mut group = c.benchmark_group("whitespace_norm");
    group.measurement_time(Duration::from_secs(10));
    
    for size_kb in [1, 10, 100, 1000].iter() {
        let mut text = String::with_capacity(*size_kb * 1024);
        
        // Text with various whitespace patterns
        for i in 0..(size_kb * 50) {
            match i % 4 {
                0 => text.push_str("Normal   spacing  "),
                1 => text.push_str("Tabs\t\there\t"),
                2 => text.push_str("Multiple\n\n\nnewlines"),
                _ => text.push_str("Mixed \t \n spaces"),
            }
        }
        
        group.throughput(Throughput::Bytes(text.len() as u64));
        
        // Baseline: Scalar whitespace normalization
        group.bench_with_input(
            BenchmarkId::new("scalar", size_kb),
            &text,
            |b, input| {
                b.iter(|| {
                    let mut result = String::with_capacity(input.len());
                    let mut prev_space = false;
                    
                    for ch in input.chars() {
                        if ch.is_whitespace() {
                            if !prev_space {
                                result.push(' ');
                                prev_space = true;
                            }
                        } else {
                            result.push(ch);
                            prev_space = false;
                        }
                    }
                    
                    black_box(result)
                });
            },
        );
        
        // TODO: Add SIMD whitespace normalization benchmark
    }
    
    group.finish();
}

/// Benchmark end-to-end parsing performance
fn bench_end_to_end_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end_parsing");
    group.measurement_time(Duration::from_secs(10));
    
    // RTF parsing
    for size_kb in [1, 10, 100].iter() {
        let rtf_doc = generate_rtf_document(*size_kb);
        
        group.throughput(Throughput::Bytes(rtf_doc.len() as u64));
        
        group.bench_with_input(
            BenchmarkId::new("rtf_current", size_kb),
            &rtf_doc,
            |b, input| {
                b.iter(|| {
                    let tokens = rtf_lexer::tokenize(black_box(input)).unwrap();
                    let document = OptimizedRtfParser::parse(tokens).unwrap();
                    black_box(document)
                });
            },
        );
        
        // TODO: Add SIMD-optimized version benchmark
    }
    
    // Markdown parsing
    for size_kb in [1, 10, 100].iter() {
        let md_doc = generate_markdown_document(*size_kb);
        
        group.throughput(Throughput::Bytes(md_doc.len() as u64));
        
        group.bench_with_input(
            BenchmarkId::new("markdown_current", size_kb),
            &md_doc,
            |b, input| {
                b.iter(|| {
                    let mut parser = OptimizedMarkdownParser::new();
                    let document = parser.parse(black_box(input)).unwrap();
                    black_box(document)
                });
            },
        );
        
        // TODO: Add SIMD-optimized version benchmark
    }
    
    group.finish();
}

/// Profile hot paths in parsing
fn bench_hot_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_paths");
    group.measurement_time(Duration::from_secs(5));
    
    // String interning performance
    let repeated_strings: Vec<String> = (0..1000)
        .map(|i| format!("repeated_string_{}", i % 50))
        .collect();
    
    group.bench_function("string_interning", |b| {
        b.iter(|| {
            let mut cache = std::collections::HashMap::new();
            for s in &repeated_strings {
                cache.entry(s.clone()).or_insert_with(|| s.clone());
            }
            black_box(cache)
        });
    });
    
    // Character classification
    let mixed_text = "Hello123 World_Test! Special#Chars$Here%";
    group.bench_function("char_classification", |b| {
        b.iter(|| {
            let mut alpha = 0;
            let mut digit = 0;
            let mut special = 0;
            
            for ch in mixed_text.chars() {
                if ch.is_alphabetic() {
                    alpha += 1;
                } else if ch.is_numeric() {
                    digit += 1;
                } else {
                    special += 1;
                }
            }
            
            black_box((alpha, digit, special))
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_rtf_char_search,
    bench_rtf_tokenization,
    bench_markdown_char_search,
    bench_utf8_validation,
    bench_whitespace_normalization,
    bench_end_to_end_parsing,
    bench_hot_paths
);
criterion_main!(benches);