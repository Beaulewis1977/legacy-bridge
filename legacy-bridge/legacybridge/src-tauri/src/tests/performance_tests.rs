#[cfg(test)]
mod performance_tests {
    use crate::conversion::markdown_parser::MarkdownParser;
    use crate::conversion::markdown_parser_optimized::OptimizedMarkdownParser;
    use crate::pipeline::formatting_engine::FormattingEngine;
    use crate::pipeline::formatting_engine_optimized::OptimizedFormattingEngine;
    use crate::pipeline::concurrent_processor::{ConcurrentProcessor, ConversionRequest, ConversionContent, ConversionOptions};
    use std::time::Instant;

    /// Generate realistic test document
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

    #[test]
    fn benchmark_parser_comparison() {
        println!("\n=== Parser Performance Comparison ===");
        
        let sizes = vec![10, 100, 500, 1000];
        
        for size in sizes {
            let doc = generate_test_document(size);
            let doc_size_kb = doc.len() / 1024;
            
            // Test original parser
            let start = Instant::now();
            let _ = MarkdownParser::parse(&doc);
            let original_time = start.elapsed();
            
            // Test optimized parser
            let mut opt_parser = OptimizedMarkdownParser::new();
            let start = Instant::now();
            let _ = opt_parser.parse(&doc);
            let optimized_time = start.elapsed();
            
            let improvement = ((original_time.as_micros() as f64 - optimized_time.as_micros() as f64) 
                / original_time.as_micros() as f64) * 100.0;
            
            println!(
                "Document size: {}KB ({} paragraphs)",
                doc_size_kb, size
            );
            println!(
                "  Original:  {:>8.2}ms",
                original_time.as_secs_f64() * 1000.0
            );
            println!(
                "  Optimized: {:>8.2}ms ({:+.1}% improvement)",
                optimized_time.as_secs_f64() * 1000.0,
                improvement
            );
            println!();
        }
    }

    #[test]
    fn benchmark_memory_efficiency() {
        println!("\n=== Memory Efficiency Test ===");
        
        // Test with many small allocations
        let mut doc = String::new();
        for i in 0..1000 {
            doc.push_str(&format!("**Word{}** ", i));
        }
        
        let start = Instant::now();
        let mut parser = OptimizedMarkdownParser::new();
        let _ = parser.parse(&doc);
        let time = start.elapsed();
        
        println!("Processing 1000 formatted words:");
        println!("  Time: {:.2}ms", time.as_secs_f64() * 1000.0);
        println!("  Throughput: {:.0} words/sec", 1000.0 / time.as_secs_f64());
    }

    #[test]
    fn benchmark_deep_nesting() {
        println!("\n=== Deep Nesting Performance ===");
        
        // Create deeply nested list
        let mut doc = String::new();
        for depth in 0..20 {
            doc.push_str(&"  ".repeat(depth));
            doc.push_str(&format!("- Level {} item\n", depth));
            
            // Add some content at each level
            doc.push_str(&"  ".repeat(depth + 1));
            doc.push_str("Content with **formatting** and *emphasis*\n");
        }
        
        let mut parser = OptimizedMarkdownParser::new();
        let start = Instant::now();
        let result = parser.parse(&doc);
        let time = start.elapsed();
        
        assert!(result.is_ok());
        println!("Deep nesting (20 levels):");
        println!("  Time: {:.2}ms", time.as_secs_f64() * 1000.0);
    }

    #[test]
    fn benchmark_large_tables() {
        println!("\n=== Large Table Performance ===");
        
        let rows = 100;
        let cols = 20;
        
        let mut doc = String::from("| ");
        for col in 0..cols {
            doc.push_str(&format!("Column {} | ", col + 1));
        }
        doc.push_str("\n|");
        doc.push_str(&"----------|".repeat(cols));
        doc.push('\n');
        
        for row in 0..rows {
            doc.push_str("| ");
            for col in 0..cols {
                doc.push_str(&format!("R{}C{} | ", row + 1, col + 1));
            }
            doc.push('\n');
        }
        
        let mut parser = OptimizedMarkdownParser::new();
        let start = Instant::now();
        let result = parser.parse(&doc);
        let time = start.elapsed();
        
        assert!(result.is_ok());
        println!("Large table ({}x{}):", rows, cols);
        println!("  Time: {:.2}ms", time.as_secs_f64() * 1000.0);
        println!("  Cells/sec: {:.0}", (rows * cols) as f64 / time.as_secs_f64());
    }

    #[tokio::test]
    async fn benchmark_concurrent_processing() {
        println!("\n=== Concurrent Processing Test ===");
        
        let processor = ConcurrentProcessor::new(Some(4));
        
        // Create batch of documents
        let batch_sizes = vec![10, 50, 100];
        
        for batch_size in batch_sizes {
            let requests: Vec<_> = (0..batch_size)
                .map(|i| ConversionRequest {
                    id: format!("doc-{}", i),
                    content: ConversionContent::Memory(generate_test_document(100)),
                    options: ConversionOptions::default(),
                })
                .collect();
            
            let start = Instant::now();
            let responses = processor.process_batch(requests).await;
            let time = start.elapsed();
            
            let successful = responses.iter().filter(|r| r.result.is_ok()).count();
            let total_bytes: usize = responses.iter().map(|r| r.metrics.input_size_bytes).sum();
            
            println!("Batch size: {} documents", batch_size);
            println!("  Total time: {:.2}s", time.as_secs_f64());
            println!("  Docs/sec: {:.1}", batch_size as f64 / time.as_secs_f64());
            println!("  Throughput: {:.1} MB/s", total_bytes as f64 / 1024.0 / 1024.0 / time.as_secs_f64());
            println!("  Success rate: {}%", (successful * 100) / batch_size);
            
            // Get processor metrics
            let metrics = processor.get_metrics();
            println!("  Avg time/doc: {:.1}ms", metrics.average_time_ms);
            println!("  Peak memory: {:.1}MB", metrics.peak_memory_mb);
            println!();
        }
    }

    #[test]
    fn benchmark_string_interning() {
        println!("\n=== String Interning Efficiency ===");
        
        // Document with many repeated strings
        let mut doc = String::new();
        let repeated_text = "This is a commonly repeated phrase in documents. ";
        
        for i in 0..500 {
            doc.push_str(&format!("## Section {}\n\n", i));
            doc.push_str(repeated_text);
            doc.push_str("**");
            doc.push_str(repeated_text);
            doc.push_str("**\n\n");
        }
        
        let mut parser = OptimizedMarkdownParser::new();
        let start = Instant::now();
        let _ = parser.parse(&doc);
        let time = start.elapsed();
        
        println!("Document with repeated strings:");
        println!("  Size: {}KB", doc.len() / 1024);
        println!("  Time: {:.2}ms", time.as_secs_f64() * 1000.0);
        println!("  Throughput: {:.1} MB/s", doc.len() as f64 / 1024.0 / 1024.0 / time.as_secs_f64());
    }

    #[test]
    fn stress_test_edge_cases() {
        println!("\n=== Edge Case Stress Test ===");
        
        // Very long single line
        let long_line = "word ".repeat(10000);
        let mut parser = OptimizedMarkdownParser::new();
        let start = Instant::now();
        let _ = parser.parse(&long_line);
        let time1 = start.elapsed();
        
        // Many short lines
        let many_lines = "line\n".repeat(10000);
        let start = Instant::now();
        let _ = parser.parse(&many_lines);
        let time2 = start.elapsed();
        
        // Heavy formatting
        let heavy_format = "**bold** *italic* `code` ".repeat(1000);
        let start = Instant::now();
        let _ = parser.parse(&heavy_format);
        let time3 = start.elapsed();
        
        println!("Edge cases:");
        println!("  Long line (50K words): {:.2}ms", time1.as_secs_f64() * 1000.0);
        println!("  Many lines (10K lines): {:.2}ms", time2.as_secs_f64() * 1000.0);
        println!("  Heavy formatting: {:.2}ms", time3.as_secs_f64() * 1000.0);
    }
}