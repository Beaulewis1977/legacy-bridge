// Example of updated FFI functions using unified error handling
//
// This shows how to modify existing FFI functions to use the new error system

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;

use crate::conversion::{markdown_to_rtf, rtf_to_markdown};
use crate::ffi_error_bridge::{set_and_return_error, create_conversion_error, create_io_error};
use crate::conversion::unified_errors::LegacyBridgeError;

/// Updated RTF to Markdown conversion with unified error handling
#[no_mangle]
pub unsafe extern "C" fn legacybridge_rtf_to_markdown_v2(
    rtf_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    // Clear any previous error
    crate::conversion::unified_errors::clear_last_error();
    
    // Validate input parameters
    if rtf_content.is_null() || output_buffer.is_null() || output_length.is_null() {
        let error = LegacyBridgeError::SystemError {
            component: "FFI".to_string(),
            error_code: 1,
            description: "Null pointer passed to function".to_string(),
            internal_message: None,
        };
        return set_and_return_error(error);
    }

    // Convert input to Rust String
    let rtf_string = match CStr::from_ptr(rtf_content).to_str() {
        Ok(s) => s.to_string(),
        Err(e) => {
            let error = LegacyBridgeError::ValidationError {
                field: "rtf_content".to_string(),
                expected: "Valid UTF-8 string".to_string(),
                received: "Invalid UTF-8 sequence".to_string(),
                location: None,
            };
            return set_and_return_error(error);
        }
    };

    // Perform conversion with proper error handling
    match rtf_to_markdown(&rtf_string) {
        Ok(markdown) => {
            match CString::new(markdown.clone()) {
                Ok(c_string) => {
                    *output_buffer = c_string.into_raw();
                    *output_length = markdown.len() as c_int;
                    0 // Success
                }
                Err(_) => {
                    let error = LegacyBridgeError::SystemError {
                        component: "FFI".to_string(),
                        error_code: 2,
                        description: "Failed to allocate output string".to_string(),
                        internal_message: Some("CString::new failed".to_string()),
                    };
                    set_and_return_error(error)
                }
            }
        }
        Err(e) => {
            // Convert ConversionError to LegacyBridgeError
            let error: LegacyBridgeError = e.into();
            set_and_return_error(error)
        }
    }
}

/// Example: Convert file with detailed error reporting
#[no_mangle]
pub unsafe extern "C" fn legacybridge_convert_rtf_file_to_md_v2(
    input_path: *const c_char,
    output_path: *const c_char,
) -> c_int {
    use std::fs;
    use std::path::Path;
    
    // Clear any previous error
    crate::conversion::unified_errors::clear_last_error();
    
    if input_path.is_null() || output_path.is_null() {
        let error = LegacyBridgeError::SystemError {
            component: "FFI".to_string(),
            error_code: 1,
            description: "Null pointer passed to function".to_string(),
            internal_message: None,
        };
        return set_and_return_error(error);
    }
    
    // Convert paths
    let input_path_str = match CStr::from_ptr(input_path).to_str() {
        Ok(s) => s,
        Err(_) => {
            let error = LegacyBridgeError::ValidationError {
                field: "input_path".to_string(),
                expected: "Valid UTF-8 path".to_string(),
                received: "Invalid UTF-8 sequence".to_string(),
                location: None,
            };
            return set_and_return_error(error);
        }
    };
    
    let output_path_str = match CStr::from_ptr(output_path).to_str() {
        Ok(s) => s,
        Err(_) => {
            let error = LegacyBridgeError::ValidationError {
                field: "output_path".to_string(),
                expected: "Valid UTF-8 path".to_string(),
                received: "Invalid UTF-8 sequence".to_string(),
                location: None,
            };
            return set_and_return_error(error);
        }
    };
    
    // Check if input file exists
    if !Path::new(input_path_str).exists() {
        let error = LegacyBridgeError::IOError {
            operation: "read".to_string(),
            path: input_path_str.to_string(),
            cause: "File not found".to_string(),
            error_code: Some(2),
        };
        return set_and_return_error(error);
    }
    
    // Read input file
    let rtf_content = match fs::read_to_string(input_path_str) {
        Ok(content) => content,
        Err(e) => {
            let error = create_io_error("read", input_path_str, e);
            return set_and_return_error(error);
        }
    };
    
    // Convert with detailed error handling
    let markdown_content = match rtf_to_markdown(&rtf_content) {
        Ok(content) => content,
        Err(e) => {
            // Create conversion error with suggestions
            let error = create_conversion_error(
                "RTF",
                "Markdown",
                &format!("Failed to convert file: {}", input_path_str),
                vec![
                    "Check if the RTF file is valid".to_string(),
                    "Try opening the file in a text editor to verify it's not corrupted".to_string(),
                    "Ensure the file contains proper RTF headers (starts with {\\rtf)".to_string(),
                ],
            );
            return set_and_return_error(error);
        }
    };
    
    // Write output file
    match fs::write(output_path_str, markdown_content) {
        Ok(_) => 0, // Success
        Err(e) => {
            let error = create_io_error("write", output_path_str, e);
            set_and_return_error(error)
        }
    }
}

/// Example: Batch conversion with detailed progress and error reporting
#[no_mangle]
pub unsafe extern "C" fn legacybridge_batch_convert_with_errors(
    rtf_array: *const *const c_char,
    count: c_int,
    output_array: *mut *mut c_char,
    output_lengths: *mut c_int,
    error_array: *mut *mut c_char, // Array to store error JSONs
) -> c_int {
    if rtf_array.is_null() || output_array.is_null() || output_lengths.is_null() || 
       error_array.is_null() || count <= 0 {
        return -1;
    }

    let mut success_count = 0;
    
    for i in 0..count as usize {
        let rtf_ptr = *rtf_array.add(i);
        let output_ptr = output_array.add(i);
        let length_ptr = output_lengths.add(i);
        let error_ptr = error_array.add(i);
        
        // Clear previous error for this iteration
        crate::conversion::unified_errors::clear_last_error();
        
        // Validate input
        if rtf_ptr.is_null() {
            let error = LegacyBridgeError::ValidationError {
                field: format!("rtf_array[{}]", i),
                expected: "Non-null RTF content".to_string(),
                received: "Null pointer".to_string(),
                location: None,
            };
            crate::conversion::unified_errors::set_last_error(error);
            
            // Store error JSON
            if let Some(json) = crate::conversion::unified_errors::get_last_error_json() {
                if let Ok(c_string) = CString::new(json) {
                    *error_ptr = c_string.into_raw();
                }
            }
            
            *output_ptr = ptr::null_mut();
            *length_ptr = 0;
            continue;
        }
        
        // Convert to Rust string
        let rtf_string = match CStr::from_ptr(rtf_ptr).to_str() {
            Ok(s) => s,
            Err(_) => {
                let error = LegacyBridgeError::ValidationError {
                    field: format!("rtf_array[{}]", i),
                    expected: "Valid UTF-8 string".to_string(),
                    received: "Invalid UTF-8 sequence".to_string(),
                    location: None,
                };
                crate::conversion::unified_errors::set_last_error(error);
                
                if let Some(json) = crate::conversion::unified_errors::get_last_error_json() {
                    if let Ok(c_string) = CString::new(json) {
                        *error_ptr = c_string.into_raw();
                    }
                }
                
                *output_ptr = ptr::null_mut();
                *length_ptr = 0;
                continue;
            }
        };
        
        // Perform conversion
        match rtf_to_markdown(rtf_string) {
            Ok(markdown) => {
                match CString::new(markdown.clone()) {
                    Ok(c_string) => {
                        *output_ptr = c_string.into_raw();
                        *length_ptr = markdown.len() as c_int;
                        *error_ptr = ptr::null_mut(); // No error
                        success_count += 1;
                    }
                    Err(_) => {
                        let error = LegacyBridgeError::SystemError {
                            component: "FFI".to_string(),
                            error_code: 2,
                            description: format!("Failed to allocate output for item {}", i),
                            internal_message: None,
                        };
                        crate::conversion::unified_errors::set_last_error(error);
                        
                        if let Some(json) = crate::conversion::unified_errors::get_last_error_json() {
                            if let Ok(c_string) = CString::new(json) {
                                *error_ptr = c_string.into_raw();
                            }
                        }
                        
                        *output_ptr = ptr::null_mut();
                        *length_ptr = 0;
                    }
                }
            }
            Err(e) => {
                let error: LegacyBridgeError = e.into();
                crate::conversion::unified_errors::set_last_error(error);
                
                if let Some(json) = crate::conversion::unified_errors::get_last_error_json() {
                    if let Ok(c_string) = CString::new(json) {
                        *error_ptr = c_string.into_raw();
                    }
                }
                
                *output_ptr = ptr::null_mut();
                *length_ptr = 0;
            }
        }
    }
    
    success_count
}