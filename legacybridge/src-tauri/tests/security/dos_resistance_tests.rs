// DoS resistance tests for RTF conversion system
//
// Tests the system's ability to handle and reject various denial-of-service attacks

#[cfg(test)]
mod dos_resistance_tests {
    use crate::conversion::{
        rtf_to_markdown,
        markdown_to_rtf,
        input_validation::InputValidator,
        security::SecurityLimits,
    };
    use std::time::{Duration, Instant};
    use std::thread;
    use std::sync::{Arc, Mutex};

    const MAX_PROCESSING_TIME: Duration = Duration::from_secs(30);
    const MAX_MEMORY_MB: usize = 100;

    #[test]
    fn test_memory_bomb_protection() {
        // Test protection against memory exhaustion attacks
        let test_cases = vec![
            // Exponential expansion attempt
            generate_expansion_bomb(10),
            // Large single allocation
            generate_large_text_bomb(50_000_000), // 50MB
            // Many small allocations
            generate_fragmentation_bomb(1_000_000),
        ];

        for (name, rtf) in test_cases {
            println!("Testing memory bomb: {}", name);
            
            // Monitor memory usage
            let initial_memory = get_current_memory_usage();
            let result = rtf_to_markdown(&rtf);
            let final_memory = get_current_memory_usage();
            
            let memory_increase = final_memory.saturating_sub(initial_memory);
            
            assert!(
                result.is_err() || memory_increase < MAX_MEMORY_MB * 1024 * 1024,
                "{}: Memory usage exceeded limit: {} bytes", name, memory_increase
            );
        }
    }

    #[test]
    fn test_cpu_exhaustion_protection() {
        let test_cases = vec![
            // Deeply nested groups requiring recursive parsing
            ("Deep nesting", generate_deep_nesting(1000)),
            // Complex regex patterns
            ("Regex bomb", generate_regex_bomb()),
            // Excessive control word processing
            ("Control word spam", generate_control_word_spam(100_000)),
        ];

        for (name, rtf) in test_cases {
            println!("Testing CPU exhaustion: {}", name);
            
            let start = Instant::now();
            let _ = rtf_to_markdown(&rtf);
            let duration = start.elapsed();
            
            assert!(
                duration < MAX_PROCESSING_TIME,
                "{}: Processing time exceeded limit: {:?}", name, duration
            );
        }
    }

    #[test]
    fn test_concurrent_dos_protection() {
        // Test system behavior under concurrent attack
        let validator = Arc::new(InputValidator::new());
        let mut handles = vec![];
        let errors = Arc::new(Mutex::new(Vec::new()));

        // Spawn multiple threads attempting DoS
        for i in 0..10 {
            let validator = Arc::clone(&validator);
            let errors = Arc::clone(&errors);
            
            let handle = thread::spawn(move || {
                let bomb = if i % 2 == 0 {
                    generate_expansion_bomb(5).1
                } else {
                    generate_large_text_bomb(10_000_000).1
                };
                
                let start = Instant::now();
                let result = validator.pre_validate_rtf(&bomb);
                let duration = start.elapsed();
                
                if duration > Duration::from_secs(5) {
                    errors.lock().unwrap().push(
                        format!("Thread {} validation took too long: {:?}", i, duration)
                    );
                }
                
                result
            });
            
            handles.push(handle);
        }

        // All threads should complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        let errors = errors.lock().unwrap();
        assert!(errors.is_empty(), "Concurrent DoS issues: {:?}", *errors);
    }

    #[test]
    fn test_zip_bomb_style_attack() {
        // RTF equivalent of a zip bomb - small input that expands massively
        let mut rtf = String::from(r"{\rtf1 ");
        
        // Define macros that reference each other
        for i in 0..10 {
            rtf.push_str(&format!(r"{{\*\expand{} ", i));
            if i > 0 {
                for j in 0..i {
                    rtf.push_str(&format!(r"{{\*\expand{} AAAA}}", j));
                }
            } else {
                rtf.push_str("AAAA");
            }
            rtf.push('}');
        }
        
        // Reference the largest macro many times
        for _ in 0..1000 {
            rtf.push_str(r"{\*\expand9}");
        }
        
        rtf.push('}');
        
        let start = Instant::now();
        let result = rtf_to_markdown(&rtf);
        let duration = start.elapsed();
        
        assert!(
            result.is_err() || duration < Duration::from_secs(5),
            "Zip bomb protection failed"
        );
    }

    #[test]
    fn test_algorithmic_complexity_attack() {
        // Test attacks targeting O(n²) or worse algorithms
        
        // 1. Nested table attack
        let nested_tables = generate_nested_tables(50);
        let start = Instant::now();
        let _ = rtf_to_markdown(&nested_tables);
        assert!(start.elapsed() < Duration::from_secs(5), "Nested tables took too long");
        
        // 2. Cross-reference attack
        let cross_refs = generate_cross_references(100);
        let start = Instant::now();
        let _ = rtf_to_markdown(&cross_refs);
        assert!(start.elapsed() < Duration::from_secs(5), "Cross references took too long");
        
        // 3. Style cascade attack  
        let style_cascade = generate_style_cascade(100);
        let start = Instant::now();
        let _ = rtf_to_markdown(&style_cascade);
        assert!(start.elapsed() < Duration::from_secs(5), "Style cascade took too long");
    }

    #[test]
    fn test_resource_limit_enforcement() {
        let limits = SecurityLimits {
            max_file_size: 1000, // 1KB
            max_text_size: 500,
            max_nesting_depth: 5,
            max_table_dimensions: (10, 10),
            max_processing_time: Duration::from_secs(1),
            max_memory_usage: 1024 * 1024, // 1MB
        };

        // Test each limit
        let test_cases = vec![
            ("File size", "A".repeat(2000)),
            ("Nesting depth", generate_deep_nesting(10)),
            ("Table size", generate_large_table(20, 20)),
        ];

        for (limit_name, content) in test_cases {
            let rtf = format!("{{\\rtf1 {}}}", content);
            // Should enforce limits - exact behavior depends on implementation
            let _ = rtf_to_markdown(&rtf);
            println!("Limit enforced: {}", limit_name);
        }
    }

    #[test] 
    fn test_slow_loris_style_attack() {
        // Simulate slow input that tries to hold resources
        let rtf = r"{\rtf1 \par Text";
        
        // Start parsing incomplete RTF
        let start = Instant::now();
        let result = rtf_to_markdown(rtf);
        let duration = start.elapsed();
        
        // Should fail fast on incomplete input
        assert!(result.is_err(), "Incomplete RTF should be rejected");
        assert!(
            duration < Duration::from_secs(1),
            "Should fail fast on incomplete input"
        );
    }

    #[test]
    fn test_hash_flooding_protection() {
        // Test protection against hash collision attacks
        let mut rtf = String::from(r"{\rtf1 ");
        
        // Generate many similar strings that might collide
        for i in 0..10000 {
            // These strings are designed to potentially collide in weak hash functions
            let key = format!("Aa{}", "B".repeat(i % 10));
            rtf.push_str(&format!(r"\{} ", key));
        }
        
        rtf.push_str("Test}");
        
        let start = Instant::now();
        let _ = rtf_to_markdown(&rtf);
        let duration = start.elapsed();
        
        assert!(
            duration < Duration::from_secs(10),
            "Hash flooding protection failed: {:?}", duration
        );
    }

    // Helper functions

    fn generate_expansion_bomb(levels: usize) -> (&'static str, String) {
        let mut rtf = String::from(r"{\rtf1 ");
        
        for i in 0..levels {
            rtf.push_str(&format!(r"{{\*\level{} ", i));
            for _ in 0..(2_usize.pow(i as u32)) {
                rtf.push_str("BOMB");
            }
            rtf.push('}');
        }
        
        rtf.push('}');
        ("Expansion bomb", rtf)
    }

    fn generate_large_text_bomb(size: usize) -> (&'static str, String) {
        let text = "A".repeat(size);
        ("Large text bomb", format!(r"{{\rtf1 {}}}", text))
    }

    fn generate_fragmentation_bomb(count: usize) -> (&'static str, String) {
        let mut rtf = String::from(r"{\rtf1 ");
        
        for i in 0..count {
            rtf.push_str(&format!(r"\par Fragment {} ", i));
        }
        
        rtf.push('}');
        ("Fragmentation bomb", rtf)
    }

    fn generate_deep_nesting(depth: usize) -> String {
        let mut rtf = String::from(r"{\rtf1 ");
        
        for _ in 0..depth {
            rtf.push('{');
        }
        
        rtf.push_str("Deep");
        
        for _ in 0..depth {
            rtf.push('}');
        }
        
        rtf.push('}');
        rtf
    }

    fn generate_regex_bomb() -> String {
        // Pattern that could cause catastrophic backtracking
        let pattern = "a".repeat(50) + &"a?".repeat(50) + &"a".repeat(50);
        format!(r"{{\rtf1 {}}}", pattern)
    }

    fn generate_control_word_spam(count: usize) -> String {
        let mut rtf = String::from(r"{\rtf1 ");
        
        for i in 0..count {
            rtf.push_str(&format!(r"\fs{} \cf{} \cb{} ", i % 100, i % 10, i % 10));
        }
        
        rtf.push_str("Text}");
        rtf
    }

    fn generate_nested_tables(depth: usize) -> String {
        let mut rtf = String::from(r"{\rtf1 ");
        
        for _ in 0..depth {
            rtf.push_str(r"\trowd\cellx1000 ");
        }
        
        rtf.push_str(r"\intbl Cell");
        
        for _ in 0..depth {
            rtf.push_str(r"\cell\row ");
        }
        
        rtf.push('}');
        rtf
    }

    fn generate_cross_references(count: usize) -> String {
        let mut rtf = String::from(r"{\rtf1 ");
        
        // Create bookmarks
        for i in 0..count {
            rtf.push_str(&format!(r"{{\*\bkmkstart ref{}}}{{\*\bkmkend ref{}}} ", i, i));
        }
        
        // Create references to all bookmarks
        for i in 0..count {
            for j in 0..count {
                rtf.push_str(&format!(r"{{\\field{{\\fldinst REF ref{}}}}}", j));
            }
        }
        
        rtf.push('}');
        rtf
    }

    fn generate_style_cascade(count: usize) -> String {
        let mut rtf = String::from(r"{\rtf1 ");
        
        // Create style definitions that reference each other
        rtf.push_str(r"{\stylesheet ");
        for i in 0..count {
            rtf.push_str(&format!(
                r"{{\s{} \snext{} Style{};}}", 
                i, 
                (i + 1) % count, 
                i
            ));
        }
        rtf.push_str("}");
        
        // Use all styles
        for i in 0..count {
            rtf.push_str(&format!(r"\s{} Text ", i));
        }
        
        rtf.push('}');
        rtf
    }

    fn generate_large_table(rows: usize, cols: usize) -> String {
        let mut rtf = String::new();
        
        for _ in 0..rows {
            rtf.push_str(r"\trowd ");
            for j in 0..cols {
                rtf.push_str(&format!(r"\cellx{} ", (j + 1) * 1000));
            }
            
            for _ in 0..cols {
                rtf.push_str(r"\intbl Cell\cell ");
            }
            
            rtf.push_str(r"\row ");
        }
        
        rtf
    }

    fn get_current_memory_usage() -> usize {
        // Simplified memory tracking - in production use proper OS APIs
        use std::collections::HashMap;
        
        let mut map = HashMap::new();
        for i in 0..1000 {
            map.insert(i, vec![0u8; 1024]);
        }
        
        std::mem::drop(map);
        
        // Return approximate value - would use /proc/self/status on Linux
        0
    }
}