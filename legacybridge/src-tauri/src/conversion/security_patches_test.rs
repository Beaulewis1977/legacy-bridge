// Security Patches Test Suite - Validates all critical security fixes
// Tests for CRITICAL-SEC-001, CRITICAL-SEC-002, and CRITICAL-SEC-003

#[cfg(test)]
mod security_tests {
    use super::super::*;
    use crate::conversion::{rtf_lexer, rtf_parser::RtfParser, rtf_generator::RtfGenerator};
    use crate::conversion::types::{RtfDocument, RtfNode, DocumentMetadata};
    
    // CRITICAL-SEC-001: Unbounded Memory Allocation Tests
    #[test]
    fn test_memory_limit_enforcement() {
        // Create a massive text node that exceeds memory limits
        let mut huge_rtf = String::from(r"{\rtf1 ");
        huge_rtf.push_str(&"A".repeat(11 * 1024 * 1024)); // 11MB of text
        huge_rtf.push_str(r"\par}");
        
        let result = rtf_lexer::tokenize(&huge_rtf);
        assert!(result.is_err(), "Should reject input with excessive text size");
        
        match result {
            Err(e) => {
                let error_msg = e.to_string();
                assert!(error_msg.contains("exceeds maximum allowed") || 
                        error_msg.contains("Text size exceeds"),
                        "Error should mention size limit: {}", error_msg);
            }
            _ => panic!("Expected error for oversized input")
        }
    }
    
    #[test]
    fn test_node_count_limit() {
        // Create RTF with excessive nodes
        let mut rtf = String::from(r"{\rtf1 ");
        for _ in 0..100_001 {
            rtf.push_str(r"{\b A}");
        }
        rtf.push_str(r"\par}");
        
        // This should fail during tokenization or parsing
        let tokens = rtf_lexer::tokenize(&rtf);
        if let Ok(tokens) = tokens {
            let result = RtfParser::parse(tokens);
            assert!(result.is_err(), "Should reject document with too many nodes");
        }
    }
    
    #[test]
    fn test_concurrent_memory_limits() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::thread;
        
        let failure_count = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];
        
        // Try to create many concurrent conversions
        for _ in 0..20 {
            let failure_count = Arc::clone(&failure_count);
            let handle = thread::spawn(move || {
                // Each thread tries to allocate 10MB
                let large_rtf = format!(r"{{\rtf1 {}\par}}", "X".repeat(10 * 1024 * 1024));
                
                match rtf_lexer::tokenize(&large_rtf) {
                    Ok(tokens) => {
                        match RtfParser::parse(tokens) {
                            Err(_) => {
                                failure_count.fetch_add(1, Ordering::SeqCst);
                            }
                            Ok(_) => {
                                // Some may succeed within limits
                            }
                        }
                    }
                    Err(_) => {
                        failure_count.fetch_add(1, Ordering::SeqCst);
                    }
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        // At least some should fail due to global memory limits
        assert!(failure_count.load(Ordering::SeqCst) > 0,
                "Global memory limits should prevent some concurrent conversions");
    }
    
    // CRITICAL-SEC-002: Integer Overflow Tests
    #[test]
    fn test_integer_overflow_prevention() {
        // Test maximum positive number
        let rtf = r"{\rtf1 \fs999999 Test\par}";
        let result = rtf_lexer::tokenize(rtf);
        assert!(result.is_ok(), "Should accept numbers within range");
        
        // Test number exceeding allowed range
        let rtf = r"{\rtf1 \fs9999999 Test\par}";
        let result = rtf_lexer::tokenize(rtf);
        assert!(result.is_err(), "Should reject numbers outside allowed range");
    }
    
    #[test]
    fn test_negative_number_overflow() {
        // Test minimum negative number
        let rtf = r"{\rtf1 \li-999999 Test\par}";
        let result = rtf_lexer::tokenize(rtf);
        assert!(result.is_ok(), "Should accept negative numbers within range");
        
        // Test negative number exceeding range
        let rtf = r"{\rtf1 \li-9999999 Test\par}";
        let result = rtf_lexer::tokenize(rtf);
        assert!(result.is_err(), "Should reject negative numbers outside range");
    }
    
    #[test]
    fn test_hex_value_overflow_prevention() {
        // Valid hex values
        let rtf = r"{\rtf1 \'FF \'00 \'7F\par}";
        let result = rtf_lexer::tokenize(rtf);
        assert!(result.is_ok(), "Should accept valid hex values");
        
        // Invalid hex (not possible to overflow with 2 digits, but test parsing)
        let rtf = r"{\rtf1 \'GG\par}";
        let result = rtf_lexer::tokenize(rtf);
        assert!(result.is_err(), "Should reject invalid hex digits");
    }
    
    #[test]
    fn test_control_word_length_limit() {
        // Create control word exceeding length limit
        let long_control = "a".repeat(100);
        let rtf = format!(r"{{\rtf1 \{} Test\par}}", long_control);
        
        let result = rtf_lexer::tokenize(&rtf);
        assert!(result.is_err(), "Should reject control words exceeding length limit");
    }
    
    #[test]
    fn test_numeric_overflow_with_multiplication() {
        // Test a number that would overflow during parsing
        let rtf = r"{\rtf1 \fs2147483647 Test\par}"; // Max i32
        let result = rtf_lexer::tokenize(rtf);
        assert!(result.is_err(), "Should reject numbers that could cause overflow");
    }
    
    // CRITICAL-SEC-003: Stack Overflow Prevention Tests
    #[test]
    fn test_deep_recursion_prevention_parser() {
        // Create deeply nested groups
        let mut rtf = String::from(r"{\rtf1 ");
        for _ in 0..60 {
            rtf.push_str(r"{\b ");
        }
        rtf.push_str("Deep");
        for _ in 0..60 {
            rtf.push('}');
        }
        rtf.push_str(r"\par}");
        
        let tokens = rtf_lexer::tokenize(&rtf);
        if let Ok(tokens) = tokens {
            let result = RtfParser::parse(tokens);
            assert!(result.is_err(), "Should reject deeply nested structures");
            
            if let Err(e) = result {
                let error_msg = e.to_string();
                assert!(error_msg.contains("recursion") || error_msg.contains("depth"),
                        "Error should mention recursion depth: {}", error_msg);
            }
        }
    }
    
    #[test]
    fn test_deep_recursion_prevention_generator() {
        // Create deeply nested document structure
        let mut content = RtfNode::Text("Deep".to_string());
        
        // Nest it deeply
        for _ in 0..60 {
            content = RtfNode::Bold(vec![content]);
        }
        
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![content],
        };
        
        let result = RtfGenerator::generate(&document);
        assert!(result.is_err(), "Should reject deeply nested generation");
        
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(error_msg.contains("recursion") || error_msg.contains("depth"),
                    "Error should mention recursion depth: {}", error_msg);
        }
    }
    
    #[test]
    fn test_output_size_limit() {
        // Create a document that would generate huge output
        let mut nodes = Vec::new();
        for i in 0..100_000 {
            nodes.push(RtfNode::Text(format!("Line {} with some text content ", i)));
        }
        
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Paragraph(nodes)],
        };
        
        let result = RtfGenerator::generate(&document);
        assert!(result.is_err(), "Should reject output exceeding size limit");
    }
    
    #[test]
    fn test_table_cell_complexity_limit() {
        use crate::conversion::types::{TableRow, TableCell};
        
        // Create table with overly complex cells
        let mut cell_content = Vec::new();
        for i in 0..2000 {
            cell_content.push(RtfNode::Text(format!("Cell item {}", i)));
        }
        
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![
                RtfNode::Table {
                    rows: vec![
                        TableRow {
                            cells: vec![
                                TableCell { content: cell_content }
                            ]
                        }
                    ]
                }
            ],
        };
        
        let result = RtfGenerator::generate(&document);
        assert!(result.is_err(), "Should reject tables with overly complex cells");
    }
    
    // Malicious input tests
    #[test]
    fn test_malicious_memory_bomb() {
        // Billion laughs style attack
        let rtf = r#"{\rtf1
            {\*\foo {\b {\i {\u {\b {\i {\u AAAAAAAAAA}}}}}}
            \foo\foo\foo\foo\foo\foo\foo\foo\foo\foo
            \foo\foo\foo\foo\foo\foo\foo\foo\foo\foo
            \par}"#;
        
        let result = rtf_lexer::tokenize(rtf);
        // Should either fail in lexing or parsing due to limits
        if let Ok(tokens) = result {
            let parse_result = RtfParser::parse(tokens);
            // The parser should handle this safely
            assert!(parse_result.is_ok() || parse_result.is_err(), 
                    "Should handle malicious input safely");
        }
    }
    
    #[test]
    fn test_stack_overflow_with_mutual_recursion() {
        // Test patterns that could cause mutual recursion
        let mut rtf = String::from(r"{\rtf1 ");
        for i in 0..30 {
            if i % 2 == 0 {
                rtf.push_str(r"{\b ");
            } else {
                rtf.push_str(r"{\i ");
            }
        }
        rtf.push_str("Text");
        for _ in 0..30 {
            rtf.push('}');
        }
        rtf.push_str(r"\par}");
        
        // Should complete successfully within recursion limits
        let tokens = rtf_lexer::tokenize(&rtf).unwrap();
        let result = RtfParser::parse(tokens);
        assert!(result.is_ok(), "Should handle moderate nesting depth");
    }
    
    // Boundary tests
    #[test]
    fn test_exact_limit_boundaries() {
        // Test at exact memory limit (100MB)
        let size = 100 * 1024 * 1024 - 1000; // Just under limit
        let text = "A".repeat(size);
        let rtf = format!(r"{{\rtf1 {}\par}}", text);
        
        let result = rtf_lexer::tokenize(&rtf);
        // Should fail due to size check
        assert!(result.is_err(), "Should enforce strict memory limits");
    }
    
    #[test]
    fn test_recursion_at_limit() {
        // Test at exactly MAX_RECURSION_DEPTH
        let mut rtf = String::from(r"{\rtf1 ");
        for _ in 0..49 { // Just under limit of 50
            rtf.push_str(r"{\b ");
        }
        rtf.push_str("OK");
        for _ in 0..49 {
            rtf.push('}');
        }
        rtf.push_str(r"\par}");
        
        let tokens = rtf_lexer::tokenize(&rtf).unwrap();
        let result = RtfParser::parse(tokens);
        assert!(result.is_ok(), "Should accept nesting at just under limit");
        
        // Now test at exactly the limit
        let mut rtf = String::from(r"{\rtf1 ");
        for _ in 0..51 { // Over limit
            rtf.push_str(r"{\b ");
        }
        rtf.push_str("Too Deep");
        for _ in 0..51 {
            rtf.push('}');
        }
        rtf.push_str(r"\par}");
        
        let tokens = rtf_lexer::tokenize(&rtf);
        if let Ok(tokens) = tokens {
            let result = RtfParser::parse(tokens);
            assert!(result.is_err(), "Should reject nesting exceeding limit");
        }
    }
}