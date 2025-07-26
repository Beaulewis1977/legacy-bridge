// Fuzzing test suite for RTF parser security
//
// This module implements fuzzing tests to discover edge cases and vulnerabilities
// in the RTF parsing and generation pipeline.

#[cfg(test)]
mod fuzzing_tests {
    use crate::conversion::{
        rtf_lexer::tokenize,
        secure_parser::SecureRtfParser,
        secure_generator::SecureRtfGenerator,
        markdown_to_rtf,
        rtf_to_markdown,
        input_validation::InputValidator,
        security::SecurityLimits,
    };
    use rand::{Rng, SeedableRng, rngs::StdRng};
    use std::time::{Duration, Instant};

    // Seed for reproducible fuzzing
    const FUZZ_SEED: u64 = 0x1337BEEF;
    const FUZZ_ITERATIONS: usize = 1000;
    const TIMEOUT_DURATION: Duration = Duration::from_secs(5);

    #[test]
    fn fuzz_rtf_parser_random_input() {
        let mut rng = StdRng::seed_from_u64(FUZZ_SEED);
        let mut failures = Vec::new();

        for i in 0..FUZZ_ITERATIONS {
            let rtf = generate_random_rtf(&mut rng, 1000);
            
            let start = Instant::now();
            let result = std::panic::catch_unwind(|| {
                tokenize(&rtf).and_then(|tokens| SecureRtfParser::parse(tokens))
            });
            let duration = start.elapsed();

            // Check for panics
            if result.is_err() {
                failures.push(format!("Iteration {}: Parser panicked on input", i));
                continue;
            }

            // Check for timeouts
            if duration > TIMEOUT_DURATION {
                failures.push(format!("Iteration {}: Parser timeout ({}ms)", i, duration.as_millis()));
            }
        }

        assert!(failures.is_empty(), "Fuzzing failures:\n{}", failures.join("\n"));
    }

    #[test]
    fn fuzz_rtf_control_word_combinations() {
        let mut rng = StdRng::seed_from_u64(FUZZ_SEED);
        let control_words = vec![
            "\\rtf", "\\ansi", "\\deff", "\\fonttbl", "\\f", "\\fswiss",
            "\\par", "\\b", "\\i", "\\ul", "\\fs", "\\cf", "\\cb",
            "\\ql", "\\qr", "\\qc", "\\qj", "\\li", "\\ri", "\\fi",
            "\\tab", "\\line", "\\page", "\\sect", "\\trowd", "\\cell",
            "\\row", "\\pard", "\\plain", "\\s", "\\*", "\\u",
        ];

        for _ in 0..FUZZ_ITERATIONS {
            let mut rtf = String::from("{\\rtf1 ");
            
            // Generate random combination of control words
            let num_controls = rng.gen_range(1..50);
            for _ in 0..num_controls {
                let idx = rng.gen_range(0..control_words.len());
                rtf.push_str(control_words[idx]);
                
                // Maybe add parameter
                if rng.gen_bool(0.5) {
                    let param = rng.gen_range(-1000..1000);
                    rtf.push_str(&param.to_string());
                }
                rtf.push(' ');
            }
            
            rtf.push_str("Test text}");
            
            // Should not panic or hang
            let _ = rtf_to_markdown(&rtf);
        }
    }

    #[test]
    fn fuzz_markdown_parser_random_input() {
        let mut rng = StdRng::seed_from_u64(FUZZ_SEED);
        let mut failures = Vec::new();

        for i in 0..FUZZ_ITERATIONS {
            let markdown = generate_random_markdown(&mut rng, 1000);
            
            let start = Instant::now();
            let result = std::panic::catch_unwind(|| {
                markdown_to_rtf(&markdown)
            });
            let duration = start.elapsed();

            if result.is_err() {
                failures.push(format!("Iteration {}: Markdown parser panicked", i));
                continue;
            }

            if duration > TIMEOUT_DURATION {
                failures.push(format!("Iteration {}: Markdown parser timeout", i));
            }
        }

        assert!(failures.is_empty(), "Markdown fuzzing failures:\n{}", failures.join("\n"));
    }

    #[test]
    fn fuzz_unicode_edge_cases() {
        let mut rng = StdRng::seed_from_u64(FUZZ_SEED);
        
        // Test various Unicode ranges
        let unicode_ranges = vec![
            (0x0000, 0x007F),   // Basic Latin
            (0x0080, 0x00FF),   // Latin-1 Supplement
            (0x0100, 0x017F),   // Latin Extended-A
            (0x0400, 0x04FF),   // Cyrillic
            (0x4E00, 0x9FFF),   // CJK Unified Ideographs
            (0xD800, 0xDFFF),   // Surrogates (invalid)
            (0xFFF0, 0xFFFF),   // Specials
            (0x10000, 0x10FFFF), // Supplementary planes
        ];

        for (start, end) in unicode_ranges {
            for _ in 0..100 {
                let codepoint = rng.gen_range(start..=end);
                
                // Skip invalid surrogates
                if (0xD800..=0xDFFF).contains(&codepoint) {
                    continue;
                }
                
                if let Some(ch) = char::from_u32(codepoint) {
                    let rtf = format!("{{\\rtf1 Test {} test}}", ch);
                    let markdown = format!("Test {} test", ch);
                    
                    // Should handle without panicking
                    let _ = rtf_to_markdown(&rtf);
                    let _ = markdown_to_rtf(&markdown);
                }
            }
        }
    }

    #[test]
    fn fuzz_nested_structures() {
        let mut rng = StdRng::seed_from_u64(FUZZ_SEED);
        
        for _ in 0..100 {
            let depth = rng.gen_range(1..100);
            let mut rtf = String::from("{\\rtf1 ");
            
            // Create random nesting pattern
            for _ in 0..depth {
                match rng.gen_range(0..3) {
                    0 => rtf.push('{'),
                    1 => {
                        rtf.push('}');
                        // Ensure we don't close too many
                        rtf.push('{');
                    }
                    _ => rtf.push_str("text "),
                }
            }
            
            // Balance braces
            let open_count = rtf.matches('{').count();
            let close_count = rtf.matches('}').count();
            for _ in 0..(open_count.saturating_sub(close_count)) {
                rtf.push('}');
            }
            
            // Should handle gracefully
            let _ = tokenize(&rtf);
        }
    }

    #[test]
    fn fuzz_numeric_parameters() {
        let mut rng = StdRng::seed_from_u64(FUZZ_SEED);
        
        let test_values = vec![
            i32::MIN,
            i32::MIN + 1,
            -1_000_000,
            -1,
            0,
            1,
            1_000_000,
            i32::MAX - 1,
            i32::MAX,
        ];

        for value in test_values {
            let rtf = format!("{{\\rtf1 \\fs{} \\li{} \\ri{} Text}}", value, value, value);
            
            // Should handle all values safely
            let result = tokenize(&rtf);
            if let Ok(tokens) = result {
                let _ = SecureRtfParser::parse(tokens);
            }
        }

        // Random values
        for _ in 0..1000 {
            let value = rng.gen::<i32>();
            let rtf = format!("{{\\rtf1 \\fs{} Text}}", value);
            let _ = tokenize(&rtf);
        }
    }

    #[test]
    fn fuzz_malformed_structures() {
        let malformed_patterns = vec![
            "{\\rtf1",                    // Missing closing brace
            "\\rtf1 text}",               // Missing opening brace
            "{\\rtf1 {{{{}}}}}}",         // Unbalanced braces
            "{\\rtf1 \\",                 // Incomplete control word
            "{\\rtf1 \\fs}",              // Control word without parameter
            "{\\rtf1 \\fs-}",             // Invalid parameter
            "{\\rtf1 \\fsABC}",           // Non-numeric parameter
            "{\\rtf1 \0}",                // Null byte
            "{\\rtf1 \\x00}",             // Hex null
            "{\\rtf1 \\par\\par\\par}",   // Repeated control words
            "{{{{{{{rtf1}}}}}}}",         // Excessive nesting
            "",                           // Empty input
            "{}",                         // Empty group
            "{\\}",                       // Escaped closing brace only
        ];

        for pattern in malformed_patterns {
            // Should handle without panicking
            let _ = tokenize(pattern);
            let _ = rtf_to_markdown(pattern);
        }
    }

    #[test]
    fn fuzz_size_limits() {
        let mut rng = StdRng::seed_from_u64(FUZZ_SEED);
        let validator = InputValidator::new();
        
        // Test various sizes around limits
        let sizes = vec![
            0,
            1,
            1000,
            10_000,
            100_000,
            1_000_000,
            10_000_000,
            10_485_760, // 10MB + 1KB
        ];

        for size in sizes {
            let content = generate_random_string(&mut rng, size);
            let rtf = format!("{{\\rtf1 {}}}", content);
            
            let result = validator.pre_validate_rtf(&rtf);
            
            // Should reject anything over 10MB
            if rtf.len() > 10 * 1024 * 1024 {
                assert!(result.is_err(), "Large input should be rejected");
            }
        }
    }

    #[test]
    fn fuzz_concurrent_parsing() {
        use std::thread;
        use std::sync::Arc;
        
        let mut rng = StdRng::seed_from_u64(FUZZ_SEED);
        let test_inputs: Vec<String> = (0..10)
            .map(|_| generate_random_rtf(&mut rng, 100))
            .collect();
        
        let test_inputs = Arc::new(test_inputs);
        let mut handles = vec![];

        // Spawn multiple threads parsing the same inputs
        for _ in 0..10 {
            let inputs = Arc::clone(&test_inputs);
            let handle = thread::spawn(move || {
                for input in inputs.iter() {
                    let _ = rtf_to_markdown(input);
                }
            });
            handles.push(handle);
        }

        // All threads should complete without issues
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    // Helper functions

    fn generate_random_rtf(rng: &mut StdRng, max_size: usize) -> String {
        let mut rtf = String::from("{\\rtf1 ");
        let target_size = rng.gen_range(10..max_size);
        
        while rtf.len() < target_size {
            match rng.gen_range(0..10) {
                0 => rtf.push_str("\\par "),
                1 => rtf.push_str("\\b "),
                2 => rtf.push_str("\\b0 "),
                3 => rtf.push_str("\\i "),
                4 => rtf.push_str("\\i0 "),
                5 => rtf.push('{'),
                6 => rtf.push('}'),
                7 => {
                    let num = rng.gen_range(-1000..1000);
                    rtf.push_str(&format!("\\fs{} ", num));
                }
                _ => {
                    let text_len = rng.gen_range(1..20);
                    rtf.push_str(&generate_random_string(rng, text_len));
                    rtf.push(' ');
                }
            }
        }
        
        // Balance braces
        let open = rtf.matches('{').count();
        let close = rtf.matches('}').count();
        for _ in 0..(open.saturating_sub(close)) {
            rtf.push('}');
        }
        
        rtf
    }

    fn generate_random_markdown(rng: &mut StdRng, max_size: usize) -> String {
        let mut markdown = String::new();
        let target_size = rng.gen_range(10..max_size);
        
        while markdown.len() < target_size {
            match rng.gen_range(0..10) {
                0 => markdown.push_str("\n# Heading\n"),
                1 => markdown.push_str("\n## Subheading\n"),
                2 => markdown.push_str("**bold** "),
                3 => markdown.push_str("*italic* "),
                4 => markdown.push_str("\n- List item\n"),
                5 => markdown.push_str("\n1. Numbered item\n"),
                6 => markdown.push_str("[link](http://example.com) "),
                7 => markdown.push_str("\n```\ncode block\n```\n"),
                8 => markdown.push_str("\n> Quote\n"),
                _ => {
                    let text_len = rng.gen_range(1..50);
                    markdown.push_str(&generate_random_string(rng, text_len));
                    markdown.push(' ');
                }
            }
        }
        
        markdown
    }

    fn generate_random_string(rng: &mut StdRng, len: usize) -> String {
        (0..len)
            .map(|_| {
                let idx = rng.gen_range(0..62);
                match idx {
                    0..=25 => (b'a' + idx as u8) as char,
                    26..=51 => (b'A' + (idx - 26) as u8) as char,
                    _ => (b'0' + (idx - 52) as u8) as char,
                }
            })
            .collect()
    }
}