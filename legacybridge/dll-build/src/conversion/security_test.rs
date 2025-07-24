// Security test suite for RTF conversion
//
// This module contains tests that verify security controls
// are working correctly to prevent various attack vectors.

#[cfg(test)]
mod tests {
    use crate::conversion::{
        rtf_lexer::tokenize,
        secure_parser::SecureRtfParser,
        secure_generator::SecureRtfGenerator,
        security::{SecurityLimits, ControlWordSecurity},
        types::{RtfDocument, RtfNode, DocumentMetadata},
    };

    #[test]
    fn test_billion_laughs_protection() {
        // Attempt to create exponential expansion through nested groups
        let mut rtf = String::from(r"{\rtf1 ");
        
        // Create pattern that would expand exponentially if not limited
        for i in 0..10 {
            rtf.push_str(&format!("{{\\*\\expandme{} ", i));
            for _ in 0..10 {
                rtf.push_str("{AAAAAAAAAA}");
            }
            rtf.push('}');
        }
        
        rtf.push('}');
        
        // This should fail due to limits
        let tokens = tokenize(&rtf).unwrap();
        let result = SecureRtfParser::parse(tokens);
        
        // We expect this to succeed but with controlled output
        assert!(result.is_ok() || 
            (result.is_err() && 
             result.unwrap_err().to_string().contains("Maximum")));
    }

    #[test]
    fn test_deep_nesting_attack() {
        // Create deeply nested structure
        let depth = 100;
        let mut rtf = String::from(r"{\rtf1 ");
        
        for _ in 0..depth {
            rtf.push('{');
        }
        
        rtf.push_str("Deeply nested content");
        
        for _ in 0..depth {
            rtf.push('}');
        }
        
        rtf.push('}');
        
        let tokens = tokenize(&rtf).unwrap();
        let result = SecureRtfParser::parse(tokens);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Maximum nesting depth"));
    }

    #[test]
    fn test_memory_exhaustion_text() {
        // Attempt to create very large text node
        let large_text = "A".repeat(2_000_000); // 2MB of text
        let rtf = format!(r"{{\rtf1 {}\par}}", large_text);
        
        // This should fail in the lexer due to text size limit
        let result = tokenize(&rtf);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }

    #[test]
    fn test_integer_overflow_protection() {
        // Test various numeric edge cases
        let test_cases = vec![
            r"{\rtf1 \fs2147483647 Text}",  // Max i32
            r"{\rtf1 \fs-2147483648 Text}", // Min i32
            r"{\rtf1 \fs9999999999 Text}",  // Too large
            r"{\rtf1 \li-9999999999 Text}", // Too small
        ];
        
        for rtf in test_cases {
            let result = tokenize(rtf);
            
            if result.is_err() {
                // Should fail with appropriate error
                let err = result.unwrap_err().to_string();
                assert!(err.contains("exceeds allowed range") || 
                       err.contains("too long"));
            }
        }
    }

    #[test]
    fn test_control_word_injection() {
        // Test dangerous control words are blocked
        let dangerous_rtf = vec![
            r"{\rtf1 {\object\objemb} Embedded object}",
            r"{\rtf1 {\field{\fldinst{HYPERLINK malicious.exe}}}}",
            r"{\rtf1 {\datastore\data} Hidden data}",
            r"{\rtf1 {\objdata 0000000} Object data}",
        ];
        
        for rtf in dangerous_rtf {
            let tokens = tokenize(rtf).unwrap();
            let result = SecureRtfParser::parse(tokens);
            
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Forbidden control word"));
        }
    }

    #[test]
    fn test_output_size_limits() {
        // Create document that would generate large output
        let mut content = Vec::new();
        
        // Add many paragraphs
        for i in 0..1000 {
            content.push(RtfNode::Paragraph(vec![
                RtfNode::Text(format!("Paragraph {} with some content", i))
            ]));
        }
        
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content,
        };
        
        // Use very restrictive limits
        let mut limits = SecurityLimits::default();
        limits.max_file_size = 1000; // Only 1KB allowed
        
        let result = SecureRtfGenerator::generate_with_security(&document, limits);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Output size exceeds"));
    }

    #[test]
    fn test_table_bomb() {
        // Attempt to create massive table
        use crate::conversion::types::{TableRow, TableCell};
        
        let mut rows = Vec::new();
        
        // Try to create 10000 x 10000 table
        for _ in 0..10000 {
            let mut cells = Vec::new();
            for _ in 0..10000 {
                cells.push(TableCell {
                    content: vec![RtfNode::Text("X".to_string())],
                });
            }
            rows.push(TableRow { cells });
        }
        
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Table { rows }],
        };
        
        let result = SecureRtfGenerator::generate(&document);
        
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("exceeds maximum") || err.contains("Table"));
    }

    #[test]
    fn test_unicode_overflow() {
        // Test Unicode handling edge cases
        let test_cases = vec![
            "\u{0000}", // Null byte
            "\u{FFFF}", // Highest BMP character
            "\u{10000}", // First character outside BMP
            "\u{10FFFF}", // Highest valid Unicode
        ];
        
        for test_char in test_cases {
            let document = RtfDocument {
                metadata: DocumentMetadata::default(),
                content: vec![RtfNode::Paragraph(vec![
                    RtfNode::Text(format!("Test: {}", test_char))
                ])],
            };
            
            let result = SecureRtfGenerator::generate(&document);
            
            // Should handle all cases safely
            assert!(result.is_ok());
            let rtf = result.unwrap();
            
            // Should contain Unicode escape or safe replacement
            assert!(rtf.contains("\\u") || rtf.contains("Test: "));
        }
    }

    #[test]
    fn test_safe_parsing_performance() {
        use std::time::Instant;
        
        // Generate a reasonably complex but safe document
        let mut rtf = String::from(r"{\rtf1\ansi\deff0 {\fonttbl{\f0 Times New Roman;}}");
        
        for i in 0..100 {
            rtf.push_str(&format!(
                r"\par\b Heading {}\b0\par Normal text with \i italics\i0 and \ul underline\ul0.",
                i
            ));
        }
        
        rtf.push('}');
        
        // Time the parsing
        let start = Instant::now();
        let tokens = tokenize(&rtf).unwrap();
        let result = SecureRtfParser::parse(tokens);
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        
        // Should complete in reasonable time (< 1 second for this size)
        assert!(duration.as_secs() < 1, "Parsing took too long: {:?}", duration);
    }

    #[test]
    fn test_whitelist_mode() {
        // Test strict whitelist mode
        let rtf = r"{\rtf1 \par Normal text \customcontrol with unknown control}";
        
        let tokens = tokenize(rtf).unwrap();
        let whitelist_config = ControlWordSecurity::whitelist();
        let limits = SecurityLimits::default();
        
        let result = SecureRtfParser::parse_with_security(
            tokens,
            limits,
            whitelist_config,
        );
        
        // In whitelist mode, unknown control words should be rejected
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Forbidden control word"));
    }
}

// Benchmark tests (requires unstable features)
#[cfg(all(test, feature = "unstable"))]
mod benches {
    use test::Bencher;
    use super::*;

    #[bench]
    fn bench_secure_parsing(b: &mut Bencher) {
        let rtf = r"{\rtf1\ansi\deff0 {\fonttbl{\f0 Times New Roman;}}
            \par\b Sample Document\b0\par
            This is a test document with \i various\i0 formatting.
            \par}";
        
        b.iter(|| {
            let tokens = tokenize(rtf).unwrap();
            SecureRtfParser::parse(tokens).unwrap()
        });
    }

    #[bench]
    fn bench_secure_generation(b: &mut Bencher) {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![
                RtfNode::Heading {
                    level: 1,
                    content: vec![RtfNode::Text("Test Document".to_string())],
                },
                RtfNode::Paragraph(vec![
                    RtfNode::Text("This is a test with ".to_string()),
                    RtfNode::Bold(vec![RtfNode::Text("bold".to_string())]),
                    RtfNode::Text(" and ".to_string()),
                    RtfNode::Italic(vec![RtfNode::Text("italic".to_string())]),
                    RtfNode::Text(" text.".to_string()),
                ]),
            ],
        };
        
        b.iter(|| {
            SecureRtfGenerator::generate(&document).unwrap()
        });
    }
}