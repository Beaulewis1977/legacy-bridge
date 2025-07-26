// Performance security tests for RTF conversion
//
// Tests to ensure security controls don't degrade performance
// and that performance limits prevent DoS attacks

#[cfg(test)]
mod performance_security_tests {
    use crate::conversion::{
        rtf_to_markdown,
        markdown_to_rtf,
        secure_parser::SecureRtfParser,
        secure_generator::SecureRtfGenerator,
        rtf_lexer::tokenize,
        types::{RtfDocument, RtfNode, DocumentMetadata},
        security::SecurityLimits,
    };
    use std::time::{Duration, Instant};
    use std::sync::Arc;
    use std::thread;

    // Performance thresholds
    const SMALL_DOC_MAX_TIME: Duration = Duration::from_millis(50);
    const MEDIUM_DOC_MAX_TIME: Duration = Duration::from_millis(200);
    const LARGE_DOC_MAX_TIME: Duration = Duration::from_secs(1);
    const THROUGHPUT_MIN_MB_PER_SEC: f64 = 10.0;

    #[test]
    fn test_secure_parsing_performance() {
        let test_cases = vec![
            ("Small", generate_document(10, 5)),
            ("Medium", generate_document(100, 10)),
            ("Large", generate_document(1000, 20)),
        ];

        for (size, rtf) in test_cases {
            let iterations = match size {
                "Small" => 1000,
                "Medium" => 100,
                "Large" => 10,
                _ => 100,
            };

            let start = Instant::now();
            for _ in 0..iterations {
                let tokens = tokenize(&rtf).unwrap();
                let _ = SecureRtfParser::parse(tokens);
            }
            let total_duration = start.elapsed();
            let avg_duration = total_duration / iterations as u32;

            let max_allowed = match size {
                "Small" => SMALL_DOC_MAX_TIME,
                "Medium" => MEDIUM_DOC_MAX_TIME,
                "Large" => LARGE_DOC_MAX_TIME,
                _ => MEDIUM_DOC_MAX_TIME,
            };

            assert!(
                avg_duration < max_allowed,
                "{} document parsing too slow: {:?} (max: {:?})",
                size, avg_duration, max_allowed
            );

            println!("{} document: {:?} avg", size, avg_duration);
        }
    }

    #[test]
    fn test_secure_generation_performance() {
        let test_docs = vec![
            ("Small", create_test_document(10)),
            ("Medium", create_test_document(100)),
            ("Large", create_test_document(1000)),
        ];

        for (size, doc) in test_docs {
            let iterations = match size {
                "Small" => 1000,
                "Medium" => 100,
                "Large" => 10,
                _ => 100,
            };

            let start = Instant::now();
            for _ in 0..iterations {
                let _ = SecureRtfGenerator::generate(&doc);
            }
            let total_duration = start.elapsed();
            let avg_duration = total_duration / iterations as u32;

            let max_allowed = match size {
                "Small" => SMALL_DOC_MAX_TIME,
                "Medium" => MEDIUM_DOC_MAX_TIME,
                "Large" => LARGE_DOC_MAX_TIME,
                _ => MEDIUM_DOC_MAX_TIME,
            };

            assert!(
                avg_duration < max_allowed,
                "{} document generation too slow: {:?}",
                size, avg_duration
            );

            println!("{} generation: {:?} avg", size, avg_duration);
        }
    }

    #[test]
    fn test_throughput_under_security() {
        // Test that security controls maintain acceptable throughput
        let mb_of_text = "Lorem ipsum dolor sit amet. ".repeat(40_000); // ~1MB
        let rtf = format!(r"{{\rtf1 {}}}", mb_of_text);
        let rtf_size_mb = rtf.len() as f64 / (1024.0 * 1024.0);

        let start = Instant::now();
        let result = rtf_to_markdown(&rtf);
        let duration = start.elapsed();

        assert!(result.is_ok(), "Large document should process successfully");

        let throughput = rtf_size_mb / duration.as_secs_f64();
        assert!(
            throughput >= THROUGHPUT_MIN_MB_PER_SEC,
            "Throughput too low: {:.2} MB/s (min: {} MB/s)",
            throughput, THROUGHPUT_MIN_MB_PER_SEC
        );

        println!("Throughput: {:.2} MB/s", throughput);
    }

    #[test]
    fn test_rate_limiting_performance() {
        // Simulate rate limiting behavior
        let rtf = generate_document(50, 10);
        let mut durations = Vec::new();

        // Process multiple requests rapidly
        for _ in 0..20 {
            let start = Instant::now();
            let _ = rtf_to_markdown(&rtf);
            durations.push(start.elapsed());
        }

        // Check that processing time remains consistent
        let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        let max_deviation = durations.iter()
            .map(|d| {
                let diff = if *d > avg_duration {
                    *d - avg_duration
                } else {
                    avg_duration - *d
                };
                diff.as_secs_f64() / avg_duration.as_secs_f64()
            })
            .fold(0.0, f64::max);

        assert!(
            max_deviation < 0.5,
            "Performance variance too high: {:.2}%",
            max_deviation * 100.0
        );
    }

    #[test]
    fn test_concurrent_processing_performance() {
        let rtf = Arc::new(generate_document(100, 10));
        let thread_count = 8;
        let iterations_per_thread = 50;

        let start = Instant::now();
        let mut handles = vec![];

        for _ in 0..thread_count {
            let rtf = Arc::clone(&rtf);
            let handle = thread::spawn(move || {
                for _ in 0..iterations_per_thread {
                    let _ = rtf_to_markdown(&rtf);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let total_duration = start.elapsed();
        let total_operations = thread_count * iterations_per_thread;
        let ops_per_second = total_operations as f64 / total_duration.as_secs_f64();

        println!("Concurrent ops/sec: {:.2}", ops_per_second);
        
        // Should maintain reasonable performance under concurrent load
        assert!(
            ops_per_second > 100.0,
            "Concurrent performance too low: {:.2} ops/sec",
            ops_per_second
        );
    }

    #[test]
    fn test_security_validation_overhead() {
        // Measure overhead of security validation
        let rtf = generate_document(100, 10);
        
        // Time with security
        let start_secure = Instant::now();
        for _ in 0..100 {
            let _ = rtf_to_markdown(&rtf);
        }
        let secure_duration = start_secure.elapsed();

        // Time raw parsing (if available)
        let start_raw = Instant::now();
        for _ in 0..100 {
            let _ = tokenize(&rtf);
        }
        let raw_duration = start_raw.elapsed();

        let overhead_ratio = secure_duration.as_secs_f64() / raw_duration.as_secs_f64();
        
        println!("Security overhead: {:.2}x", overhead_ratio);
        
        // Security should not add more than 3x overhead
        assert!(
            overhead_ratio < 3.0,
            "Security overhead too high: {:.2}x",
            overhead_ratio
        );
    }

    #[test]
    fn test_memory_limit_performance() {
        // Test that memory limits are enforced efficiently
        let limits = SecurityLimits {
            max_file_size: 1024 * 1024, // 1MB
            max_text_size: 1024 * 1024,
            max_nesting_depth: 50,
            max_table_dimensions: (100, 100),
            max_processing_time: Duration::from_secs(5),
            max_memory_usage: 10 * 1024 * 1024, // 10MB
        };

        // Generate document that approaches but doesn't exceed limits
        let content = "Test content. ".repeat(50_000); // ~700KB
        let rtf = format!(r"{{\rtf1 {}}}", content);

        let start = Instant::now();
        let tokens = tokenize(&rtf).unwrap();
        let result = SecureRtfParser::parse_with_limits(tokens, limits.clone());
        let duration = start.elapsed();

        assert!(result.is_ok(), "Should process within limits");
        assert!(
            duration < Duration::from_secs(1),
            "Memory checking shouldn't slow processing"
        );
    }

    #[test]
    fn test_progressive_slowdown_detection() {
        // Test for algorithmic complexity attacks that cause progressive slowdown
        let mut durations = Vec::new();

        for i in 1..=10 {
            let rtf = generate_nested_document(i * 10);
            
            let start = Instant::now();
            let _ = rtf_to_markdown(&rtf);
            let duration = start.elapsed();
            
            durations.push((i * 10, duration));
        }

        // Check that processing time grows linearly, not exponentially
        for window in durations.windows(2) {
            let (size1, dur1) = window[0];
            let (size2, dur2) = window[1];
            
            let size_ratio = size2 as f64 / size1 as f64;
            let time_ratio = dur2.as_secs_f64() / dur1.as_secs_f64();
            
            // Time should grow at most quadratically with size
            assert!(
                time_ratio < size_ratio * size_ratio * 1.5,
                "Non-linear slowdown detected: size {}x, time {}x",
                size_ratio, time_ratio
            );
        }
    }

    #[test]
    fn test_timeout_enforcement() {
        // Test that processing timeouts are enforced
        let limits = SecurityLimits {
            max_processing_time: Duration::from_millis(100),
            ..Default::default()
        };

        // Create a document that would take long to process
        let complex_rtf = generate_complex_document();
        
        let start = Instant::now();
        let tokens = tokenize(&complex_rtf).unwrap();
        let result = SecureRtfParser::parse_with_limits(tokens, limits);
        let duration = start.elapsed();

        // Should timeout close to the limit
        assert!(
            duration < Duration::from_millis(200),
            "Timeout not enforced properly: {:?}",
            duration
        );

        if result.is_err() {
            let err = result.unwrap_err().to_string();
            assert!(
                err.contains("timeout") || err.contains("time limit"),
                "Should indicate timeout"
            );
        }
    }

    // Helper functions

    fn generate_document(paragraphs: usize, words_per_para: usize) -> String {
        let mut rtf = String::from(r"{\rtf1\ansi\deff0 {\fonttbl{\f0 Times New Roman;}}");
        
        for i in 0..paragraphs {
            rtf.push_str(&format!(r"\par\b Paragraph {}\b0\par ", i));
            
            for j in 0..words_per_para {
                rtf.push_str(&format!("Word{} ", j));
                
                // Add some formatting
                if j % 3 == 0 {
                    rtf.push_str(r"\i italic\i0 ");
                }
                if j % 5 == 0 {
                    rtf.push_str(r"\ul underline\ul0 ");
                }
            }
        }
        
        rtf.push('}');
        rtf
    }

    fn create_test_document(nodes: usize) -> RtfDocument {
        let mut content = Vec::new();
        
        for i in 0..nodes {
            match i % 4 {
                0 => content.push(RtfNode::Paragraph(vec![
                    RtfNode::Text(format!("Paragraph {}", i))
                ])),
                1 => content.push(RtfNode::Heading {
                    level: (i % 6 + 1) as u8,
                    content: vec![RtfNode::Text(format!("Heading {}", i))],
                }),
                2 => content.push(RtfNode::ListItem {
                    level: (i % 3) as u8,
                    ordered: i % 2 == 0,
                    content: vec![RtfNode::Text(format!("List item {}", i))],
                }),
                _ => content.push(RtfNode::Text(format!("Text node {}", i))),
            }
        }
        
        RtfDocument {
            metadata: DocumentMetadata::default(),
            content,
        }
    }

    fn generate_nested_document(depth: usize) -> String {
        let mut rtf = String::from(r"{\rtf1 ");
        
        for i in 0..depth {
            rtf.push_str(&format!(r"{{\b Level {} ", i));
        }
        
        rtf.push_str("Deep content");
        
        for _ in 0..depth {
            rtf.push('}');
        }
        
        rtf.push('}');
        rtf
    }

    fn generate_complex_document() -> String {
        let mut rtf = String::from(r"{\rtf1 ");
        
        // Add many different elements to increase processing time
        for i in 0..100 {
            rtf.push_str(&format!(
                r"\par\fs{} \cf{} \b Text{}\b0 \i More{}\i0 \ul Content{}\ul0 ",
                i % 72 + 8, i % 16, i, i, i
            ));
            
            // Add nested groups
            for _ in 0..5 {
                rtf.push_str(&format!(r"{{\fs{} Nested }}", i % 48 + 12));
            }
            
            // Add table rows
            if i % 10 == 0 {
                rtf.push_str(r"\trowd\cellx2000\cellx4000\intbl Cell1\cell Cell2\cell\row ");
            }
        }
        
        rtf.push('}');
        rtf
    }
}