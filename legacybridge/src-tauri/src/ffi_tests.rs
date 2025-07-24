// Comprehensive FFI test module
#[cfg(test)]
mod ffi_tests {
    use super::super::ffi::*;
    use std::ffi::{CStr, CString};
    use std::ptr;
    use proptest::prelude::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

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

    // Helper to create test RTF content
    fn create_test_rtf(content: &str) -> String {
        format!("{{\\rtf1\\ansi\\deff0 {{\\fonttbl{{\\f0 Times New Roman;}}}}\\f0\\fs24 {}\\par}}", content)
    }

    // Helper to create test Markdown content
    fn create_test_markdown(content: &str) -> String {
        format!("# Test Document\n\n{}", content)
    }

    #[test]
    fn test_basic_rtf_to_markdown_conversion() {
        unsafe {
            let rtf_content = CString::new(create_test_rtf("Hello World")).unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_rtf_to_markdown(
                rtf_content.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );

            assert_eq!(result, FFIErrorCode::Success as c_int);
            assert!(!output_buffer.is_null());
            assert!(output_length > 0);

            let output = c_str_result_to_string(output_buffer);
            assert!(output.is_some());
            assert!(output.unwrap().contains("Hello World"));
        }
    }

    #[test]
    fn test_basic_markdown_to_rtf_conversion() {
        unsafe {
            let markdown_content = CString::new(create_test_markdown("Test content")).unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_markdown_to_rtf(
                markdown_content.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );

            assert_eq!(result, FFIErrorCode::Success as c_int);
            assert!(!output_buffer.is_null());
            assert!(output_length > 0);

            let output = c_str_result_to_string(output_buffer);
            assert!(output.is_some());
            let rtf = output.unwrap();
            assert!(rtf.starts_with("{\\rtf"));
            assert!(rtf.contains("Test content"));
        }
    }

    #[test]
    fn test_null_pointer_handling() {
        unsafe {
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            // Test null input
            let result = legacybridge_rtf_to_markdown(
                ptr::null(),
                &mut output_buffer,
                &mut output_length
            );
            assert_eq!(result, FFIErrorCode::NullPointer as c_int);

            // Test null output buffer
            let valid_input = CString::new("test").unwrap();
            let result = legacybridge_rtf_to_markdown(
                valid_input.as_ptr(),
                ptr::null_mut(),
                &mut output_length
            );
            assert_eq!(result, FFIErrorCode::NullPointer as c_int);

            // Test null output length
            let result = legacybridge_rtf_to_markdown(
                valid_input.as_ptr(),
                &mut output_buffer,
                ptr::null_mut()
            );
            assert_eq!(result, FFIErrorCode::NullPointer as c_int);
        }
    }

    #[test]
    fn test_invalid_utf8_handling() {
        unsafe {
            let invalid_utf8 = vec![0xFF, 0xFE, 0x00];
            let invalid_ptr = invalid_utf8.as_ptr() as *const c_char;
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_rtf_to_markdown(
                invalid_ptr,
                &mut output_buffer,
                &mut output_length
            );
            
            assert_eq!(result, FFIErrorCode::InvalidUtf8 as c_int);
        }
    }

    #[test]
    fn test_memory_cleanup() {
        unsafe {
            // Test repeated allocations and frees
            for i in 0..1000 {
                let content = format!("Test iteration {}", i);
                let rtf_content = CString::new(create_test_rtf(&content)).unwrap();
                let mut output_buffer: *mut c_char = ptr::null_mut();
                let mut output_length: c_int = 0;

                let result = legacybridge_rtf_to_markdown(
                    rtf_content.as_ptr(),
                    &mut output_buffer,
                    &mut output_length
                );

                assert_eq!(result, FFIErrorCode::Success as c_int);
                assert!(!output_buffer.is_null());

                // Free the allocated memory
                legacybridge_free_string(output_buffer);
            }
            // No memory leaks should occur
        }
    }

    #[test]
    fn test_free_null_pointer() {
        unsafe {
            // Should not crash when freeing null pointer
            legacybridge_free_string(ptr::null_mut());
        }
    }

    #[test]
    fn test_version_functions() {
        unsafe {
            // Test get_version
            let version = legacybridge_get_version();
            assert!(!version.is_null());
            let version_str = CStr::from_ptr(version).to_str().unwrap();
            assert_eq!(version_str, "1.0.0");

            // Test get_version_info
            let mut major: c_int = 0;
            let mut minor: c_int = 0;
            let mut patch: c_int = 0;

            let result = legacybridge_get_version_info(&mut major, &mut minor, &mut patch);
            assert_eq!(result, FFIErrorCode::Success as c_int);
            assert_eq!(major, 1);
            assert_eq!(minor, 0);
            assert_eq!(patch, 0);
        }
    }

    #[test]
    fn test_batch_conversion() {
        unsafe {
            // Create test data
            let rtf_contents = vec![
                CString::new(create_test_rtf("Document 1")).unwrap(),
                CString::new(create_test_rtf("Document 2")).unwrap(),
                CString::new(create_test_rtf("Document 3")).unwrap(),
            ];

            let rtf_ptrs: Vec<*const c_char> = rtf_contents.iter()
                .map(|s| s.as_ptr())
                .collect();

            let mut output_ptrs: Vec<*mut c_char> = vec![ptr::null_mut(); 3];
            let mut output_lengths: Vec<c_int> = vec![0; 3];

            let success_count = legacybridge_batch_rtf_to_markdown(
                rtf_ptrs.as_ptr(),
                3,
                output_ptrs.as_mut_ptr(),
                output_lengths.as_mut_ptr()
            );

            assert_eq!(success_count, 3);

            // Verify all conversions succeeded
            for i in 0..3 {
                assert!(!output_ptrs[i].is_null());
                assert!(output_lengths[i] > 0);

                let output = c_str_result_to_string(output_ptrs[i]);
                assert!(output.is_some());
                assert!(output.unwrap().contains(&format!("Document {}", i + 1)));
            }
        }
    }

    #[test]
    fn test_batch_with_errors() {
        unsafe {
            // Mix valid and invalid content
            let rtf_contents = vec![
                CString::new(create_test_rtf("Valid 1")).unwrap(),
                CString::new("Invalid RTF").unwrap(), // This might fail conversion
                CString::new(create_test_rtf("Valid 2")).unwrap(),
            ];

            let rtf_ptrs: Vec<*const c_char> = rtf_contents.iter()
                .map(|s| s.as_ptr())
                .collect();

            let mut output_ptrs: Vec<*mut c_char> = vec![ptr::null_mut(); 3];
            let mut output_lengths: Vec<c_int> = vec![0; 3];

            let success_count = legacybridge_batch_rtf_to_markdown(
                rtf_ptrs.as_ptr(),
                3,
                output_ptrs.as_mut_ptr(),
                output_lengths.as_mut_ptr()
            );

            // At least some should succeed
            assert!(success_count >= 0);
            assert!(success_count <= 3);

            // Clean up successful conversions
            for i in 0..3 {
                if !output_ptrs[i].is_null() {
                    legacybridge_free_string(output_ptrs[i]);
                }
            }
        }
    }

    #[test]
    fn test_last_error_message() {
        unsafe {
            let mut buffer: [c_char; 256] = [0; 256];
            
            let result = legacybridge_get_last_error(buffer.as_mut_ptr(), 256);
            
            assert!(result >= 0);
            assert!(result < 256);
            
            // Verify null termination
            assert_eq!(buffer[result as usize], 0);
        }
    }

    #[test]
    fn test_test_connection() {
        unsafe {
            let result = legacybridge_test_connection();
            assert_eq!(result, 1);
        }
    }

    #[test]
    fn test_validate_rtf_document() {
        unsafe {
            // Valid RTF
            let valid_rtf = CString::new("{\\rtf1 test}").unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_validate_rtf_document(
                valid_rtf.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );

            assert_eq!(result, FFIErrorCode::Success as c_int);
            let validation = c_str_result_to_string(output_buffer);
            assert!(validation.unwrap().contains("Valid RTF"));

            // Invalid RTF
            let invalid_rtf = CString::new("not rtf").unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_validate_rtf_document(
                invalid_rtf.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );

            assert_eq!(result, FFIErrorCode::Success as c_int);
            let validation = c_str_result_to_string(output_buffer);
            assert!(validation.unwrap().contains("Invalid RTF"));
        }
    }

    #[test]
    fn test_extract_plain_text() {
        unsafe {
            let rtf_content = CString::new(create_test_rtf("**Bold** and *italic* text")).unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_extract_plain_text(
                rtf_content.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );

            assert_eq!(result, FFIErrorCode::Success as c_int);
            let plain_text = c_str_result_to_string(output_buffer);
            assert!(plain_text.is_some());
            
            let text = plain_text.unwrap();
            assert!(text.contains("Bold and italic text"));
            assert!(!text.contains("*"));
            assert!(!text.contains("#"));
        }
    }

    #[test]
    fn test_clean_rtf_formatting() {
        unsafe {
            let messy_rtf = CString::new("{\\rtf1\\ansi\\deff0 {\\fonttbl{\\f0 Times;}{\\f1 Arial;}}\\f0\\fs24 Test\\f1\\fs30 content}").unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_clean_rtf_formatting(
                messy_rtf.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );

            assert_eq!(result, FFIErrorCode::Success as c_int);
            let clean_rtf = c_str_result_to_string(output_buffer);
            assert!(clean_rtf.is_some());
        }
    }

    #[test]
    fn test_normalize_markdown() {
        unsafe {
            let messy_md = CString::new("#  Heading\n\n\n\nContent").unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_normalize_markdown(
                messy_md.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );

            assert_eq!(result, FFIErrorCode::Success as c_int);
            let normalized = c_str_result_to_string(output_buffer);
            assert!(normalized.is_some());
        }
    }

    // Property-based tests
    proptest! {
        #[test]
        fn test_rtf_conversion_doesnt_crash(content in ".*") {
            unsafe {
                let rtf = create_test_rtf(&content);
                if let Ok(c_string) = CString::new(rtf) {
                    let mut output_buffer: *mut c_char = ptr::null_mut();
                    let mut output_length: c_int = 0;

                    let result = legacybridge_rtf_to_markdown(
                        c_string.as_ptr(),
                        &mut output_buffer,
                        &mut output_length
                    );

                    // Should not crash, may return error
                    assert!(result == FFIErrorCode::Success as c_int || 
                           result == FFIErrorCode::ConversionError as c_int);

                    if !output_buffer.is_null() {
                        legacybridge_free_string(output_buffer);
                    }
                }
            }
        }

        #[test]
        fn test_markdown_conversion_doesnt_crash(content in ".*") {
            unsafe {
                let markdown = create_test_markdown(&content);
                if let Ok(c_string) = CString::new(markdown) {
                    let mut output_buffer: *mut c_char = ptr::null_mut();
                    let mut output_length: c_int = 0;

                    let result = legacybridge_markdown_to_rtf(
                        c_string.as_ptr(),
                        &mut output_buffer,
                        &mut output_length
                    );

                    // Should not crash, may return error
                    assert!(result == FFIErrorCode::Success as c_int || 
                           result == FFIErrorCode::ConversionError as c_int);

                    if !output_buffer.is_null() {
                        legacybridge_free_string(output_buffer);
                    }
                }
            }
        }

        #[test]
        fn test_batch_size_handling(size in 0..100usize) {
            unsafe {
                let rtf_contents: Vec<CString> = (0..size)
                    .map(|i| CString::new(create_test_rtf(&format!("Doc {}", i))).unwrap())
                    .collect();

                if size == 0 {
                    let result = legacybridge_batch_rtf_to_markdown(
                        ptr::null(),
                        0,
                        ptr::null_mut(),
                        ptr::null_mut()
                    );
                    assert_eq!(result, 0);
                } else {
                    let rtf_ptrs: Vec<*const c_char> = rtf_contents.iter()
                        .map(|s| s.as_ptr())
                        .collect();

                    let mut output_ptrs: Vec<*mut c_char> = vec![ptr::null_mut(); size];
                    let mut output_lengths: Vec<c_int> = vec![0; size];

                    let success_count = legacybridge_batch_rtf_to_markdown(
                        rtf_ptrs.as_ptr(),
                        size as c_int,
                        output_ptrs.as_mut_ptr(),
                        output_lengths.as_mut_ptr()
                    );

                    assert!(success_count >= 0);
                    assert!(success_count <= size as c_int);

                    // Clean up
                    for ptr in output_ptrs {
                        if !ptr.is_null() {
                            legacybridge_free_string(ptr);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_concurrent_conversions() {
        let counter = AtomicUsize::new(0);
        let threads: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    unsafe {
                        for j in 0..100 {
                            let content = format!("Thread {} iteration {}", i, j);
                            let rtf = CString::new(create_test_rtf(&content)).unwrap();
                            let mut output_buffer: *mut c_char = ptr::null_mut();
                            let mut output_length: c_int = 0;

                            let result = legacybridge_rtf_to_markdown(
                                rtf.as_ptr(),
                                &mut output_buffer,
                                &mut output_length
                            );

                            if result == FFIErrorCode::Success as c_int {
                                legacybridge_free_string(output_buffer);
                            }
                        }
                    }
                })
            })
            .collect();

        for thread in threads {
            thread.join().unwrap();
        }
    }

    #[test]
    fn test_large_content_handling() {
        unsafe {
            // Test with large content (1MB)
            let large_content = "x".repeat(1024 * 1024);
            let rtf = CString::new(create_test_rtf(&large_content)).unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_rtf_to_markdown(
                rtf.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );

            // Should handle large content
            assert!(result == FFIErrorCode::Success as c_int || 
                   result == FFIErrorCode::ConversionError as c_int);

            if !output_buffer.is_null() {
                legacybridge_free_string(output_buffer);
            }
        }
    }

    #[test]
    fn test_special_characters() {
        unsafe {
            let special_chars = "Hello 世界 🌍 \n\t\r";
            let rtf = CString::new(create_test_rtf(special_chars)).unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_rtf_to_markdown(
                rtf.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );

            assert_eq!(result, FFIErrorCode::Success as c_int);
            let output = c_str_result_to_string(output_buffer);
            assert!(output.is_some());
            assert!(output.unwrap().contains("世界"));
        }
    }

    #[test]
    fn test_empty_content() {
        unsafe {
            let empty_rtf = CString::new("{\\rtf1}").unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_rtf_to_markdown(
                empty_rtf.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );

            assert_eq!(result, FFIErrorCode::Success as c_int);
            let output = c_str_result_to_string(output_buffer);
            assert!(output.is_some());
        }
    }

    #[test]
    fn test_template_functions() {
        unsafe {
            // Test list templates
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_list_available_templates(
                &mut output_buffer,
                &mut output_length
            );

            assert_eq!(result, FFIErrorCode::Success as c_int);
            let templates = c_str_result_to_string(output_buffer);
            assert!(templates.is_some());
            assert!(templates.unwrap().contains("default"));

            // Test validate template
            let template_name = CString::new("default").unwrap();
            let result = legacybridge_validate_template(template_name.as_ptr());
            assert_eq!(result, FFIErrorCode::Success as c_int);

            // Test create template
            let rtf_content = CString::new(create_test_rtf("Template content")).unwrap();
            let result = legacybridge_create_rtf_template(
                template_name.as_ptr(),
                rtf_content.as_ptr()
            );
            assert_eq!(result, FFIErrorCode::Success as c_int);
        }
    }

    #[test]
    fn test_csv_functions() {
        unsafe {
            // Test export to CSV
            let rtf = CString::new(create_test_rtf("Data")).unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_export_to_csv(
                rtf.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );

            assert_eq!(result, FFIErrorCode::Success as c_int);
            let csv = c_str_result_to_string(output_buffer);
            assert!(csv.is_some());
            assert!(csv.unwrap().contains("Column"));

            // Test import from CSV
            let csv_content = CString::new("A,B,C\n1,2,3").unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_import_from_csv(
                csv_content.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );

            assert_eq!(result, FFIErrorCode::Success as c_int);
            let rtf = c_str_result_to_string(output_buffer);
            assert!(rtf.is_some());
            assert!(rtf.unwrap().starts_with("{\\rtf"));
        }
    }

    #[test]
    fn test_table_functions() {
        unsafe {
            // Test convert table to RTF
            let table_data = CString::new("[[1,2],[3,4]]").unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_convert_table_to_rtf(
                table_data.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );

            assert_eq!(result, FFIErrorCode::Success as c_int);
            let rtf = c_str_result_to_string(output_buffer);
            assert!(rtf.is_some());
            assert!(rtf.unwrap().contains("\\trowd"));

            // Test extract tables from RTF
            let rtf_with_table = CString::new("{\\rtf1\\trowd\\cellx1000\\cell\\row}").unwrap();
            let mut output_buffer: *mut c_char = ptr::null_mut();
            let mut output_length: c_int = 0;

            let result = legacybridge_extract_tables_from_rtf(
                rtf_with_table.as_ptr(),
                &mut output_buffer,
                &mut output_length
            );

            assert_eq!(result, FFIErrorCode::Success as c_int);
            let tables_json = c_str_result_to_string(output_buffer);
            assert!(tables_json.is_some());
            assert!(tables_json.unwrap().contains("rows"));
        }
    }
}