// Enhanced FFI Edge Case and Security Tests
#[cfg(test)]
mod ffi_edge_case_tests {
    use super::super::ffi::*;
    use std::ffi::{CStr, CString};
    use std::ptr;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use proptest::prelude::*;

    // Helper function to convert C string result to Rust String
    unsafe fn c_str_result_to_string(ptr: *mut c_char) -> Option<String> {
        if ptr.is_null() {
            None
        } else {
            let result = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            legacybridge_free_string(ptr);
            Some(result)
        }
    }

    // ===== Memory Safety Tests =====
    
    #[test]
    fn test_double_free_protection() {
        unsafe {
            let rtf_content = CString::new("{\\rtf1 test}").unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_rtf_to_markdown(
                rtf_content.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );

            assert_eq!(result, FFIErrorCode::Success as c_int);
            
            // First free - should succeed
            legacybridge_free_string(output_buffer);
            
            // Second free - should not crash
            legacybridge_free_string(output_buffer);
        }
    }

    #[test]
    fn test_buffer_overflow_protection() {
        unsafe {
            // Create a very large input
            let large_content = "a".repeat(10 * 1024 * 1024); // 10MB
            let rtf_content = CString::new(format!("{{\\rtf1 {}}}", large_content)).unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_rtf_to_markdown(
                rtf_content.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );

            // Should either succeed or return appropriate error
            assert!(result == FFIErrorCode::Success as c_int || 
                   result == FFIErrorCode::ConversionError as c_int);

            if !output_buffer.is_null() {
                legacybridge_free_string(output_buffer);
            }
        }
    }

    #[test]
    fn test_stack_overflow_protection() {
        unsafe {
            // Create deeply nested RTF structure
            let mut nested = String::from("{\\rtf1");
            for _ in 0..10000 {
                nested.push_str("{\\b ");
            }
            nested.push_str("test");
            for _ in 0..10000 {
                nested.push_str("}");
            }
            nested.push_str("}");

            if let Ok(rtf_content) = CString::new(nested) {
                let mut output_buffer: *mut c_char = ptr::null_mut();
                let mut output_length: c_int = 0;

                let result = legacybridge_rtf_to_markdown(
                    rtf_content.as_ptr(),
                    &mut output_buffer,
                    &mut output_length
                );

                // Should handle without stack overflow
                assert!(result == FFIErrorCode::Success as c_int || 
                       result == FFIErrorCode::ConversionError as c_int);

                if !output_buffer.is_null() {
                    legacybridge_free_string(output_buffer);
                }
            }
        }
    }

    // ===== Encoding and Character Set Tests =====

    #[test]
    fn test_various_encodings() {
        unsafe {
            let test_strings = vec![
                "ASCII test",
                "UTF-8: 你好世界 🌍",
                "Latin-1: café naïve",
                "Cyrillic: Привет мир",
                "Arabic: مرحبا بالعالم",
                "Hebrew: שלום עולם",
                "Emoji: 🚀 💻 🎉 ❤️",
                "Control chars: \n\r\t\0",
                "Zero-width: ‌‍",
            ];

            for test_str in test_strings {
                // Skip if contains null byte
                if test_str.contains('\0') {
                    continue;
                }

                if let Ok(rtf_content) = CString::new(format!("{{\\rtf1 {}}}", test_str)) {
                    let mut output_buffer: *mut c_char = ptr::null_mut();
                    let mut output_length: c_int = 0;

                    let result = legacybridge_rtf_to_markdown(
                        rtf_content.as_ptr(),
                        &mut output_buffer,
                        &mut output_length
                    );

                    assert!(result == FFIErrorCode::Success as c_int || 
                           result == FFIErrorCode::InvalidUtf8 as c_int);

                    if !output_buffer.is_null() {
                        legacybridge_free_string(output_buffer);
                    }
                }
            }
        }
    }

    #[test]
    fn test_invalid_utf8_sequences() {
        unsafe {
            // Invalid UTF-8 sequences
            let invalid_sequences = vec![
                vec![0xFF, 0xFE], // Invalid start
                vec![0xC0, 0x80], // Overlong encoding
                vec![0xED, 0xA0, 0x80], // Surrogate half
                vec![0xF5, 0x80, 0x80, 0x80], // Out of range
            ];

            for seq in invalid_sequences {
                let mut null_terminated = seq.clone();
                null_terminated.push(0);
                let ptr = null_terminated.as_ptr() as *const c_char;
                
                let mut output_buffer: *mut c_char = ptr::null_mut();
                let mut output_length: c_int = 0;

                let result = legacybridge_rtf_to_markdown(
                    ptr,
                    &mut output_buffer,
                    &mut output_length
                );

                assert_eq!(result, FFIErrorCode::InvalidUtf8 as c_int);
            }
        }
    }

    // ===== Concurrent Access Tests =====

    #[test]
    fn test_thread_safety_single_function() {
        let results = Arc::new(Mutex::new(Vec::new()));
        let mut threads = vec![];

        for i in 0..50 {
            let results_clone = Arc::clone(&results);
            let thread = thread::spawn(move || {
                unsafe {
                    let content = format!("Thread {} content", i);
                    let rtf = CString::new(format!("{{\\rtf1 {}}}", content)).unwrap();
                    let mut output_buffer: *mut c_char = ptr::null_mut();
                    let mut output_length: c_int = 0;

                    let result = legacybridge_rtf_to_markdown(
                        rtf.as_ptr(),
                        &mut output_buffer,
                        &mut output_length
                    );

                    let success = result == FFIErrorCode::Success as c_int;
                    results_clone.lock().unwrap().push((i, success));

                    if !output_buffer.is_null() {
                        legacybridge_free_string(output_buffer);
                    }
                }
            });
            threads.push(thread);
        }

        for thread in threads {
            thread.join().unwrap();
        }

        let results = results.lock().unwrap();
        assert_eq!(results.len(), 50);
        
        // All conversions should succeed
        for (_, success) in results.iter() {
            assert!(*success);
        }
    }

    #[test]
    fn test_concurrent_different_functions() {
        let barrier = Arc::new(std::sync::Barrier::new(4));
        let mut threads = vec![];

        // Thread 1: RTF to Markdown
        let b1 = Arc::clone(&barrier);
        threads.push(thread::spawn(move || {
            b1.wait();
            unsafe {
                for _ in 0..100 {
                    let rtf = CString::new("{\\rtf1 test}").unwrap();
                    let mut output_buffer: *mut c_char = ptr::null_mut();
                    let mut output_length: c_int = 0;

                    legacybridge_rtf_to_markdown(
                        rtf.as_ptr(),
                        &mut output_buffer,
                        &mut output_length
                    );

                    if !output_buffer.is_null() {
                        legacybridge_free_string(output_buffer);
                    }
                }
            }
        }));

        // Thread 2: Markdown to RTF
        let b2 = Arc::clone(&barrier);
        threads.push(thread::spawn(move || {
            b2.wait();
            unsafe {
                for _ in 0..100 {
                    let md = CString::new("# Test").unwrap();
                    let mut output_buffer: *mut c_char = ptr::null_mut();
                    let mut output_length: c_int = 0;

                    legacybridge_markdown_to_rtf(
                        md.as_ptr(),
                        &mut output_buffer,
                        &mut output_length
                    );

                    if !output_buffer.is_null() {
                        legacybridge_free_string(output_buffer);
                    }
                }
            }
        }));

        // Thread 3: Validation
        let b3 = Arc::clone(&barrier);
        threads.push(thread::spawn(move || {
            b3.wait();
            unsafe {
                for _ in 0..100 {
                    let rtf = CString::new("{\\rtf1 test}").unwrap();
                    let mut output_buffer: *mut c_char = ptr::null_mut();
                    let mut output_length: c_int = 0;

                    legacybridge_validate_rtf_document(
                        rtf.as_ptr(),
                        &mut output_buffer,
                        &mut output_length
                    );

                    if !output_buffer.is_null() {
                        legacybridge_free_string(output_buffer);
                    }
                }
            }
        }));

        // Thread 4: Version check
        let b4 = Arc::clone(&barrier);
        threads.push(thread::spawn(move || {
            b4.wait();
            unsafe {
                for _ in 0..100 {
                    let version = legacybridge_get_version();
                    assert!(!version.is_null());
                }
            }
        }));

        for thread in threads {
            thread.join().unwrap();
        }
    }

    // ===== Input Validation Tests =====

    #[test]
    fn test_extremely_long_input() {
        unsafe {
            // Test with 100MB input
            let huge_content = "x".repeat(100 * 1024 * 1024);
            
            if let Ok(rtf) = CString::new(format!("{{\\rtf1 {}}}", huge_content)) {
                let mut output_buffer: *mut c_char = ptr::null_mut();
                let mut output_length: c_int = 0;

                let start = std::time::Instant::now();
                let result = legacybridge_rtf_to_markdown(
                    rtf.as_ptr(),
                    &mut output_buffer,
                    &mut output_length
                );
                let duration = start.elapsed();

                // Should complete within reasonable time (10 seconds)
                assert!(duration.as_secs() < 10);

                // Should handle gracefully
                assert!(result == FFIErrorCode::Success as c_int || 
                       result == FFIErrorCode::ConversionError as c_int);

                if !output_buffer.is_null() {
                    legacybridge_free_string(output_buffer);
                }
            }
        }
    }

    #[test]
    fn test_malicious_input_patterns() {
        unsafe {
            let malicious_patterns = vec![
                // Billion laughs attack pattern
                "{\\rtf1 {\\*\\lol lol}{\\*\\lol2 {\\lol}{\\lol}}}",
                // Excessive nesting
                &("{".repeat(1000) + "test" + &"}".repeat(1000)),
                // Invalid control sequences
                "{\\rtf1 \\invalid\\control\\sequences}",
                // Mixed valid/invalid
                "{\\rtf1 valid {\\invalid} more valid}",
                // Null bytes (will be skipped)
                "{\\rtf1 test\0hidden}",
                // Format string patterns
                "{\\rtf1 %s %d %x %n}",
                // Path traversal attempts
                "{\\rtf1 ../../etc/passwd}",
                // SQL injection patterns
                "{\\rtf1 '; DROP TABLE users; --}",
                // Script injection
                "{\\rtf1 <script>alert('xss')</script>}",
            ];

            for pattern in malicious_patterns {
                if pattern.contains('\0') {
                    continue;
                }

                if let Ok(rtf) = CString::new(pattern) {
                    let mut output_buffer: *mut c_char = ptr::null_mut();
                    let mut output_length: c_int = 0;

                    let result = legacybridge_rtf_to_markdown(
                        rtf.as_ptr(),
                        &mut output_buffer,
                        &mut output_length
                    );

                    // Should not crash or hang
                    assert!(result == FFIErrorCode::Success as c_int || 
                           result == FFIErrorCode::ConversionError as c_int);

                    if !output_buffer.is_null() {
                        // Verify no script/sql content in output
                        let output = c_str_result_to_string(output_buffer);
                        if let Some(text) = output {
                            assert!(!text.contains("<script"));
                            assert!(!text.contains("DROP TABLE"));
                            assert!(!text.contains("../"));
                        }
                    }
                }
            }
        }
    }

    // ===== Error Recovery Tests =====

    #[test]
    fn test_partial_rtf_documents() {
        unsafe {
            let partial_docs = vec![
                "{\\rtf1",           // Missing closing brace
                "{\\rtf1 test",      // Missing closing brace
                "\\rtf1 test}",      // Missing opening brace  
                "{\\rtf1 {\\b test", // Unclosed formatting
                "{\\rtf1 \\par",     // Incomplete paragraph
                "",                  // Empty input
                "not rtf at all",    // Plain text
            ];

            for doc in partial_docs {
                if let Ok(rtf) = CString::new(doc) {
                    let mut output_buffer: *mut c_char = ptr::null_mut();
                    let mut output_length: c_int = 0;

                    let result = legacybridge_rtf_to_markdown(
                        rtf.as_ptr(),
                        &mut output_buffer,
                        &mut output_length
                    );

                    // Should handle gracefully without crashing
                    assert!(result == FFIErrorCode::Success as c_int || 
                           result == FFIErrorCode::ConversionError as c_int);

                    if !output_buffer.is_null() {
                        legacybridge_free_string(output_buffer);
                    }
                }
            }
        }
    }

    #[test]
    fn test_error_message_retrieval() {
        unsafe {
            // Trigger an error
            let result = legacybridge_rtf_to_markdown(
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut()
            );

            assert_eq!(result, FFIErrorCode::NullPointer as c_int);

            // Get error message
            let mut error_buffer: [c_char; 512] = [0; 512];
            let msg_len = legacybridge_get_last_error(error_buffer.as_mut_ptr(), 512);

            assert!(msg_len > 0);
            assert!(msg_len < 512);

            let error_msg = CStr::from_ptr(error_buffer.as_ptr()).to_string_lossy();
            assert!(!error_msg.is_empty());
        }
    }

    // ===== Batch Operation Edge Cases =====

    #[test] 
    fn test_batch_with_mixed_validity() {
        unsafe {
            let contents = vec![
                CString::new("{\\rtf1 valid 1}").unwrap(),
                CString::new("").unwrap(), // Empty
                CString::new("{\\rtf1 valid 2}").unwrap(),
                // Invalid UTF-8 would be here but CString::new would fail
                CString::new("plain text").unwrap(),
                CString::new("{\\rtf1 valid 3}").unwrap(),
            ];

            let ptrs: Vec<*const c_char> = contents.iter().map(|s| s.as_ptr()).collect();
            let mut output_ptrs: Vec<*mut c_char> = vec![ptr::null_mut(); 5];
            let mut output_lengths: Vec<c_int> = vec![0; 5];

            let success_count = legacybridge_batch_rtf_to_markdown(
                ptrs.as_ptr(),
                5,
                output_ptrs.as_mut_ptr(),
                output_lengths.as_mut_ptr()
            );

            // Should process some successfully
            assert!(success_count > 0);
            assert!(success_count <= 5);

            // Check individual results
            for i in 0..5 {
                if !output_ptrs[i].is_null() {
                    // Successful conversion
                    assert!(output_lengths[i] > 0);
                    legacybridge_free_string(output_ptrs[i]);
                } else {
                    // Failed conversion
                    assert_eq!(output_lengths[i], 0);
                }
            }
        }
    }

    #[test]
    fn test_batch_with_null_entries() {
        unsafe {
            let valid = CString::new("{\\rtf1 test}").unwrap();
            let ptrs = vec![
                valid.as_ptr(),
                ptr::null(),
                valid.as_ptr(),
            ];

            let mut output_ptrs: Vec<*mut c_char> = vec![ptr::null_mut(); 3];
            let mut output_lengths: Vec<c_int> = vec![0; 3];

            let success_count = legacybridge_batch_rtf_to_markdown(
                ptrs.as_ptr(),
                3,
                output_ptrs.as_mut_ptr(),
                output_lengths.as_mut_ptr()
            );

            // Should process valid entries
            assert_eq!(success_count, 2);

            // First and third should succeed
            assert!(!output_ptrs[0].is_null());
            assert!(output_ptrs[1].is_null());
            assert!(!output_ptrs[2].is_null());

            legacybridge_free_string(output_ptrs[0]);
            legacybridge_free_string(output_ptrs[2]);
        }
    }

    // ===== Performance Boundary Tests =====

    #[test]
    fn test_performance_with_complex_formatting() {
        unsafe {
            let mut complex_rtf = String::from("{\\rtf1\\ansi\\deff0 {\\fonttbl{\\f0 Times;}}\n");
            
            // Add lots of formatting changes
            for i in 0..1000 {
                complex_rtf.push_str(&format!(
                    "{{\\f0\\fs{} Word{} }}{{\\b\\i Text{} }}{{\\ul Underline{} }}\n",
                    20 + (i % 10), i, i, i
                ));
            }
            
            complex_rtf.push_str("}");

            let rtf = CString::new(complex_rtf).unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let start = std::time::Instant::now();
            let result = legacybridge_rtf_to_markdown(
                rtf.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );
            let duration = start.elapsed();

            assert_eq!(result, FFIErrorCode::Success as c_int);
            assert!(duration.as_millis() < 1000); // Should complete within 1 second

            if !output_buffer.is_null() {
                legacybridge_free_string(output_buffer);
            }
        }
    }

    // ===== Property-based tests for edge cases =====

    proptest! {
        #[test]
        fn test_random_bytes_dont_crash(bytes: Vec<u8>) {
            unsafe {
                // Ensure null termination
                let mut null_terminated = bytes.clone();
                null_terminated.push(0);
                
                // Remove any internal nulls
                null_terminated.retain(|&b| b != 0 || std::ptr::eq(&b, null_terminated.last().unwrap()));
                
                let ptr = null_terminated.as_ptr() as *const c_char;
                let mut output_buffer: *mut c_char = ptr::null_mut();
                let mut output_length: c_int = 0;

                let result = legacybridge_rtf_to_markdown(
                    ptr,
                    &mut output_buffer,
                    &mut output_length
                );

                // Should not crash, may return error
                assert!(result == FFIErrorCode::Success as c_int || 
                       result == FFIErrorCode::InvalidUtf8 as c_int ||
                       result == FFIErrorCode::ConversionError as c_int);

                if !output_buffer.is_null() {
                    legacybridge_free_string(output_buffer);
                }
            }
        }

        #[test]
        fn test_string_length_boundaries(s in ".*", len in 0..65536usize) {
            unsafe {
                let truncated: String = s.chars().take(len).collect();
                let rtf = format!("{{\\rtf1 {}}}", truncated);
                
                if let Ok(c_str) = CString::new(rtf) {
                    let mut output_buffer: *mut c_char = ptr::null_mut();
                    let mut output_length: c_int = 0;

                    let result = legacybridge_rtf_to_markdown(
                        c_str.as_ptr(),
                        &mut output_buffer,
                        &mut output_length
                    );

                    // Length should be handled correctly
                    assert!(result == FFIErrorCode::Success as c_int || 
                           result == FFIErrorCode::ConversionError as c_int);

                    if !output_buffer.is_null() {
                        assert!(output_length >= 0);
                        assert!(output_length <= c_int::MAX);
                        legacybridge_free_string(output_buffer);
                    }
                }
            }
        }
    }

    // ===== Resource Exhaustion Tests =====

    #[test]
    fn test_memory_exhaustion_handling() {
        unsafe {
            // Try to allocate many conversions simultaneously
            let mut buffers = Vec::new();
            
            for i in 0..10000 {
                let rtf = CString::new(format!("{{\\rtf1 Content {}}}", i)).unwrap();
                let mut output_buffer: *mut c_char = ptr::null_mut();
                let mut output_length: c_int = 0;

                let result = legacybridge_rtf_to_markdown(
                    rtf.as_ptr(),
                    &mut output_buffer,
                    &mut output_length
                );

                if result == FFIErrorCode::Success as c_int {
                    buffers.push(output_buffer);
                } else {
                    // Memory exhaustion - should fail gracefully
                    break;
                }
            }

            // Clean up
            for buffer in buffers {
                legacybridge_free_string(buffer);
            }
        }
    }

    #[test]
    fn test_version_info_boundaries() {
        unsafe {
            let mut major: c_int = c_int::MAX;
            let mut minor: c_int = c_int::MIN;
            let mut patch: c_int = 42;

            let result = legacybridge_get_version_info(&mut major, &mut minor, &mut patch);
            
            assert_eq!(result, FFIErrorCode::Success as c_int);
            assert!(major >= 0 && major < 1000);
            assert!(minor >= 0 && minor < 1000);  
            assert!(patch >= 0 && patch < 1000);
        }
    }
}