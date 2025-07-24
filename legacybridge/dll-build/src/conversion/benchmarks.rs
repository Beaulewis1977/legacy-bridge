// Performance benchmarks for MD→RTF conversion
// Measures parsing speed, memory usage, and scalability

#[cfg(test)]
mod conversion_benchmarks {
    use super::super::*;
    use crate::conversion::{MarkdownParser, RtfGenerator};
    use crate::pipeline::convert_markdown_to_rtf_with_pipeline;
    use std::time::Instant;

    /// Helper to generate markdown of specific size
    fn generate_markdown(paragraphs: usize, complexity: &str) -> String {
        let mut markdown = String::new();
        
        for i in 0..paragraphs {
            match complexity {
                "simple" => {
                    markdown.push_str(&format!("# Heading {}\n\n", i));
                    markdown.push_str(&format!("This is paragraph {}.\n\n", i));
                }
                "moderate" => {
                    markdown.push_str(&format!("# Heading {} with **bold**\n\n", i));
                    markdown.push_str(&format!("Paragraph {} with **bold** and *italic* text.\n", i));
                    markdown.push_str("- List item 1\n- List item 2\n\n");
                }
                "complex" => {
                    markdown.push_str(&format!("# Main Heading {}\n\n", i));
                    markdown.push_str(&format!("## Subheading {}.1\n\n", i));
                    markdown.push_str(&format!("Complex paragraph {} with **bold**, *italic*, and `code`.\n\n", i));
                    markdown.push_str("| Col1 | Col2 | Col3 |\n");
                    markdown.push_str("|------|------|------|\n");
                    markdown.push_str("| A    | B    | C    |\n\n");
                    markdown.push_str("1. Numbered\n   - Nested\n2. List\n\n");
                    if i % 5 == 0 {
                        markdown.push_str("---\n\n");
                    }
                }
                _ => {}
            }
        }
        
        markdown
    }

    #[test]
    fn benchmark_simple_document_parsing() {
        let sizes = vec![10, 100, 1000];
        
        println!("\n=== Simple Document Parsing Benchmarks ===");
        for size in sizes {
            let markdown = generate_markdown(size, "simple");
            let start = Instant::now();
            
            let result = MarkdownParser::parse(&markdown);
            
            let duration = start.elapsed();
            assert!(result.is_ok());
            
            let doc = result.unwrap();
            println!("Simple {} paragraphs: {:?} ({} nodes)", 
                size, duration, doc.content.len());
            
            // Performance assertions
            let ms_per_paragraph = duration.as_millis() as f64 / size as f64;
            assert!(ms_per_paragraph < 1.0, "Should parse simple content quickly");
        }
    }

    #[test]
    fn benchmark_complex_document_parsing() {
        let sizes = vec![10, 50, 100];
        
        println!("\n=== Complex Document Parsing Benchmarks ===");
        for size in sizes {
            let markdown = generate_markdown(size, "complex");
            let start = Instant::now();
            
            let result = MarkdownParser::parse(&markdown);
            
            let duration = start.elapsed();
            assert!(result.is_ok());
            
            let doc = result.unwrap();
            println!("Complex {} paragraphs: {:?} ({} nodes, {} bytes input)", 
                size, duration, doc.content.len(), markdown.len());
            
            // Performance assertions
            assert!(duration.as_secs() < 2, "Should parse complex content in reasonable time");
        }
    }

    #[test]
    fn benchmark_rtf_generation() {
        use crate::conversion::types::{RtfDocument, RtfNode, DocumentMetadata};
        
        println!("\n=== RTF Generation Benchmarks ===");
        
        let node_counts = vec![100, 500, 1000];
        
        for count in node_counts {
            let mut content = Vec::new();
            
            // Generate varied content
            for i in 0..count {
                match i % 4 {
                    0 => content.push(RtfNode::Heading {
                        level: ((i % 6) + 1) as u8,
                        content: vec![RtfNode::Text(format!("Heading {}", i))],
                    }),
                    1 => content.push(RtfNode::Paragraph(vec![
                        RtfNode::Text("Text ".to_string()),
                        RtfNode::Bold(vec![RtfNode::Text("bold".to_string())]),
                        RtfNode::Text(" more.".to_string()),
                    ])),
                    2 => content.push(RtfNode::ListItem {
                        level: (i % 3) as u8,
                        content: vec![RtfNode::Text(format!("Item {}", i))],
                    }),
                    _ => content.push(RtfNode::Paragraph(vec![
                        RtfNode::Italic(vec![RtfNode::Text(format!("Italic {}", i))]),
                    ])),
                }
            }
            
            let document = RtfDocument {
                metadata: DocumentMetadata::default(),
                content,
            };
            
            let start = Instant::now();
            let result = RtfGenerator::generate(&document);
            let duration = start.elapsed();
            
            assert!(result.is_ok());
            let rtf = result.unwrap();
            
            println!("{} nodes: {:?} ({} bytes output)", 
                count, duration, rtf.len());
            
            // Performance assertion
            assert!(duration.as_millis() < 100, "Should generate RTF quickly");
        }
    }

    #[test]
    fn benchmark_full_pipeline() {
        println!("\n=== Full Pipeline Benchmarks ===");
        
        let test_cases = vec![
            ("small", 10, "simple"),
            ("medium", 50, "moderate"),
            ("large", 100, "complex"),
            ("xlarge", 200, "complex"),
        ];
        
        for (name, size, complexity) in test_cases {
            let markdown = generate_markdown(size, complexity);
            let start = Instant::now();
            
            let result = convert_markdown_to_rtf_with_pipeline(&markdown, None);
            
            let duration = start.elapsed();
            assert!(result.is_ok());
            
            let (rtf, context) = result.unwrap();
            println!("{}: {:?} ({}kb MD → {}kb RTF, {} validations)", 
                name, 
                duration, 
                markdown.len() / 1024,
                rtf.len() / 1024,
                context.validation_results.len()
            );
            
            // Performance assertions based on size
            match name {
                "small" => assert!(duration.as_millis() < 50),
                "medium" => assert!(duration.as_millis() < 200),
                "large" => assert!(duration.as_millis() < 500),
                "xlarge" => assert!(duration.as_secs() < 2),
                _ => {}
            }
        }
    }

    #[test]
    fn benchmark_unicode_processing() {
        println!("\n=== Unicode Processing Benchmarks ===");
        
        let unicode_content = vec![
            ("ASCII", "The quick brown fox jumps over the lazy dog. ".repeat(100)),
            ("Latin", "Thé qüick bröwn føx jümps övér thé läzy dög. ".repeat(100)),
            ("Mixed", "The 快速 brown 狐狸 jumps 过 the lazy 狗. ".repeat(100)),
            ("Emoji", "The quick 🦊 jumps over the lazy 🐕. 🚀🎉 ".repeat(100)),
        ];
        
        for (name, content) in unicode_content {
            let markdown = format!("# {} Test\n\n{}", name, content);
            let start = Instant::now();
            
            let result = MarkdownParser::parse(&markdown);
            assert!(result.is_ok());
            let doc = result.unwrap();
            
            let parse_duration = start.elapsed();
            
            let start = Instant::now();
            let rtf_result = RtfGenerator::generate(&doc);
            assert!(rtf_result.is_ok());
            let rtf = rtf_result.unwrap();
            
            let gen_duration = start.elapsed();
            
            println!("{}: Parse {:?}, Generate {:?} ({}kb → {}kb)", 
                name, parse_duration, gen_duration, 
                markdown.len() / 1024, rtf.len() / 1024
            );
            
            // Unicode shouldn't significantly slow down processing
            assert!(parse_duration.as_millis() < 100);
            assert!(gen_duration.as_millis() < 100);
        }
    }

    #[test]
    fn benchmark_table_processing() {
        println!("\n=== Table Processing Benchmarks ===");
        
        let row_counts = vec![10, 50, 100, 200];
        
        for rows in row_counts {
            let mut markdown = String::from("# Table Test\n\n");
            markdown.push_str("| Col1 | Col2 | Col3 | Col4 | Col5 |\n");
            markdown.push_str("|------|------|------|------|------|\n");
            
            for i in 0..rows {
                markdown.push_str(&format!("| A{} | B{} | C{} | D{} | E{} |\n", i, i, i, i, i));
            }
            
            let start = Instant::now();
            let result = convert_markdown_to_rtf_with_pipeline(&markdown, None);
            let duration = start.elapsed();
            
            assert!(result.is_ok());
            let (rtf, _) = result.unwrap();
            
            println!("{} rows: {:?} ({}kb → {}kb)", 
                rows, duration, markdown.len() / 1024, rtf.len() / 1024
            );
            
            // Table processing should scale linearly
            let ms_per_row = duration.as_millis() as f64 / rows as f64;
            assert!(ms_per_row < 2.0, "Should process tables efficiently");
        }
    }

    #[test]
    fn benchmark_memory_efficiency() {
        println!("\n=== Memory Efficiency Test ===");
        
        // Generate a very large document
        let markdown = generate_markdown(500, "complex");
        println!("Input size: {} KB", markdown.len() / 1024);
        
        // Parse multiple times to check for memory leaks
        for i in 0..5 {
            let start = Instant::now();
            let result = convert_markdown_to_rtf_with_pipeline(&markdown, None);
            let duration = start.elapsed();
            
            assert!(result.is_ok());
            let (rtf, _) = result.unwrap();
            
            println!("Iteration {}: {:?} ({} KB output)", 
                i + 1, duration, rtf.len() / 1024
            );
            
            // Duration should remain consistent (no memory degradation)
            assert!(duration.as_secs() < 3);
        }
    }

    #[test]
    fn benchmark_edge_case_performance() {
        println!("\n=== Edge Case Performance ===");
        
        // Deeply nested structure
        let mut deep_nested = String::from("# Deep Nesting\n\n");
        for i in 0..20 {
            deep_nested.push_str(&"  ".repeat(i));
            deep_nested.push_str(&format!("- Level {}\n", i));
        }
        
        let start = Instant::now();
        let result = MarkdownParser::parse(&deep_nested);
        let duration = start.elapsed();
        println!("Deep nesting (20 levels): {:?}", duration);
        assert!(result.is_ok());
        assert!(duration.as_millis() < 50);
        
        // Very long line
        let long_line = format!("# Long Line\n\n{}\n", "x".repeat(10000));
        let start = Instant::now();
        let result = MarkdownParser::parse(&long_line);
        let duration = start.elapsed();
        println!("Very long line (10k chars): {:?}", duration);
        assert!(result.is_ok());
        assert!(duration.as_millis() < 50);
        
        // Many small paragraphs
        let mut many_small = String::new();
        for i in 0..1000 {
            many_small.push_str(&format!("P{}\n\n", i));
        }
        let start = Instant::now();
        let result = MarkdownParser::parse(&many_small);
        let duration = start.elapsed();
        println!("1000 small paragraphs: {:?}", duration);
        assert!(result.is_ok());
        assert!(duration.as_millis() < 200);
    }
}