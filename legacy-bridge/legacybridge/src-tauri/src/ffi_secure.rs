// Secure FFI (Foreign Function Interface) module for VB6/VFP9 compatibility
// This module provides C-compatible exports with secure error handling

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::slice;

use crate::conversion::{markdown_to_rtf, rtf_to_markdown};
use crate::conversion::secure_error_handling::{
    SecureError, SecureErrorCode, ErrorSanitizer, FfiErrorHandler
};
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tracing::{error, warn, info};

/// Thread-local error storage for secure error messages
lazy_static::lazy_static! {
    static ref LAST_ERROR: Mutex<Option<SecureError>> = Mutex::new(None);
}

/// Set the last error in a secure manner
fn set_last_secure_error(error: SecureError) {
    if let Ok(mut last_error) = LAST_ERROR.lock() {
        *last_error = Some(error);
    }
}

/// Convert a C string to a Rust String securely
unsafe fn c_str_to_string_secure(ptr: *const c_char) -> Result<String, SecureError> {
    if ptr.is_null() {
        return Err(SecureError {
            code: SecureErrorCode::InvalidInput,
            message: "Invalid input provided".to_string(),
            error_id: None,
        });
    }
    
    CStr::from_ptr(ptr)
        .to_str()
        .map(|s| s.to_owned())
        .map_err(|_| SecureError {
            code: SecureErrorCode::InvalidInput,
            message: "Invalid text encoding".to_string(),
            error_id: None,
        })
}

/// Convert RTF to Markdown with secure error handling
#[no_mangle]
pub unsafe extern "C" fn legacybridge_rtf_to_markdown_secure(
    rtf_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    // Validate input parameters
    if rtf_content.is_null() || output_buffer.is_null() || output_length.is_null() {
        let error = SecureError {
            code: SecureErrorCode::InvalidInput,
            message: "Invalid parameters provided".to_string(),
            error_id: None,
        };
        set_last_secure_error(error.clone());
        return error.code as c_int;
    }

    // Convert input to Rust String
    let rtf_string = match c_str_to_string_secure(rtf_content) {
        Ok(s) => s,
        Err(error) => {
            set_last_secure_error(error.clone());
            return error.code as c_int;
        }
    };

    // Perform conversion with error handling
    match rtf_to_markdown(&rtf_string) {
        Ok(markdown) => {
            let c_str = match CString::new(markdown.clone()) {
                Ok(s) => s.into_raw(),
                Err(_) => {
                    let error = SecureError {
                        code: SecureErrorCode::InternalError,
                        message: "Failed to process output".to_string(),
                        error_id: Some(generate_error_id()),
                    };
                    set_last_secure_error(error.clone());
                    return error.code as c_int;
                }
            };
            
            *output_buffer = c_str;
            *output_length = markdown.len() as c_int;
            SecureErrorCode::Success as c_int
        }
        Err(internal_error) => {
            // Log internal error securely
            let error_id = generate_error_id();
            error!(
                error_id = %error_id,
                error_type = "rtf_to_markdown",
                "Conversion failed"
            );
            
            let error = SecureError {
                code: SecureErrorCode::ConversionFailed,
                message: "Document conversion failed".to_string(),
                error_id: Some(error_id),
            };
            set_last_secure_error(error.clone());
            error.code as c_int
        }
    }
}

/// Convert file with secure error handling
#[no_mangle]
pub unsafe extern "C" fn legacybridge_convert_file_secure(
    input_path: *const c_char,
    output_path: *const c_char,
) -> c_int {
    if input_path.is_null() || output_path.is_null() {
        let error = SecureError {
            code: SecureErrorCode::InvalidInput,
            message: "Invalid parameters provided".to_string(),
            error_id: None,
        };
        set_last_secure_error(error.clone());
        return error.code as c_int;
    }
    
    let input_path_str = match c_str_to_string_secure(input_path) {
        Ok(s) => s,
        Err(error) => {
            set_last_secure_error(error.clone());
            return error.code as c_int;
        }
    };
    
    let output_path_str = match c_str_to_string_secure(output_path) {
        Ok(s) => s,
        Err(error) => {
            set_last_secure_error(error.clone());
            return error.code as c_int;
        }
    };
    
    // Validate file extension without exposing paths
    let input_ext = Path::new(&input_path_str)
        .extension()
        .and_then(|e| e.to_str());
    
    let is_rtf_to_md = input_ext == Some("rtf");
    let is_md_to_rtf = input_ext == Some("md") || input_ext == Some("markdown");
    
    if !is_rtf_to_md && !is_md_to_rtf {
        let error = SecureError {
            code: SecureErrorCode::InvalidInput,
            message: "Unsupported file format".to_string(),
            error_id: None,
        };
        set_last_secure_error(error.clone());
        return error.code as c_int;
    }
    
    // Read input file
    let content = match fs::read_to_string(&input_path_str) {
        Ok(content) => content,
        Err(io_error) => {
            let error_id = generate_error_id();
            error!(
                error_id = %error_id,
                error_type = "file_read",
                "Failed to read input file"
            );
            
            let error = SecureError {
                code: SecureErrorCode::InvalidInput,
                message: "Cannot read input file".to_string(),
                error_id: Some(error_id),
            };
            set_last_secure_error(error.clone());
            return error.code as c_int;
        }
    };
    
    // Perform conversion
    let result = if is_rtf_to_md {
        rtf_to_markdown(&content)
    } else {
        markdown_to_rtf(&content)
    };
    
    match result {
        Ok(converted) => {
            // Write output file
            match fs::write(&output_path_str, converted) {
                Ok(_) => SecureErrorCode::Success as c_int,
                Err(io_error) => {
                    let error_id = generate_error_id();
                    error!(
                        error_id = %error_id,
                        error_type = "file_write",
                        "Failed to write output file"
                    );
                    
                    let error = SecureError {
                        code: SecureErrorCode::InvalidInput,
                        message: "Cannot write output file".to_string(),
                        error_id: Some(error_id),
                    };
                    set_last_secure_error(error.clone());
                    error.code as c_int
                }
            }
        }
        Err(conversion_error) => {
            let error_id = generate_error_id();
            error!(
                error_id = %error_id,
                error_type = "conversion",
                "Conversion failed"
            );
            
            let error = SecureError {
                code: SecureErrorCode::ConversionFailed,
                message: "Document conversion failed".to_string(),
                error_id: Some(error_id),
            };
            set_last_secure_error(error.clone());
            error.code as c_int
        }
    }
}

/// Get the last error message (secure version)
#[no_mangle]
pub unsafe extern "C" fn legacybridge_get_last_error_secure(
    buffer: *mut c_char,
    buffer_size: c_int,
) -> c_int {
    if buffer.is_null() || buffer_size <= 0 {
        return -1;
    }

    let error_msg = if let Ok(last_error) = LAST_ERROR.lock() {
        if let Some(ref error) = *last_error {
            format!(
                "{} (Code: {}, ID: {})",
                error.message,
                error.code as i32,
                error.error_id.as_ref().unwrap_or(&"N/A".to_string())
            )
        } else {
            "No error information available".to_string()
        }
    } else {
        "Error information unavailable".to_string()
    };
    
    let error_bytes = error_msg.as_bytes();
    let copy_len = std::cmp::min(error_bytes.len(), (buffer_size - 1) as usize);
    
    let buffer_slice = slice::from_raw_parts_mut(buffer as *mut u8, copy_len);
    buffer_slice.copy_from_slice(&error_bytes[..copy_len]);
    
    // Null terminate
    *buffer.add(copy_len) = 0;
    
    copy_len as c_int
}

/// Batch conversion with secure error handling
#[no_mangle]
pub unsafe extern "C" fn legacybridge_batch_convert_secure(
    input_array: *const *const c_char,
    count: c_int,
    output_array: *mut *mut c_char,
    output_lengths: *mut c_int,
    is_rtf_to_md: c_int,
) -> c_int {
    if input_array.is_null() || output_array.is_null() || 
       output_lengths.is_null() || count <= 0 {
        let error = SecureError {
            code: SecureErrorCode::InvalidInput,
            message: "Invalid parameters provided".to_string(),
            error_id: None,
        };
        set_last_secure_error(error);
        return 0;
    }

    let mut success_count = 0;
    
    for i in 0..count as usize {
        let input_ptr = *input_array.add(i);
        let output_ptr = output_array.add(i);
        let length_ptr = output_lengths.add(i);
        
        // Process each item with individual error handling
        let result = if is_rtf_to_md != 0 {
            legacybridge_rtf_to_markdown_secure(input_ptr, output_ptr, length_ptr)
        } else {
            legacybridge_markdown_to_rtf_secure(input_ptr, output_ptr, length_ptr)
        };
        
        if result == 0 {
            success_count += 1;
        } else {
            // Set null for failed conversions
            *output_ptr = ptr::null_mut();
            *length_ptr = 0;
        }
    }
    
    success_count
}

/// Convert Markdown to RTF with secure error handling
#[no_mangle]
pub unsafe extern "C" fn legacybridge_markdown_to_rtf_secure(
    markdown_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    // Validate input parameters
    if markdown_content.is_null() || output_buffer.is_null() || output_length.is_null() {
        let error = SecureError {
            code: SecureErrorCode::InvalidInput,
            message: "Invalid parameters provided".to_string(),
            error_id: None,
        };
        set_last_secure_error(error.clone());
        return error.code as c_int;
    }

    // Convert input to Rust String
    let markdown_string = match c_str_to_string_secure(markdown_content) {
        Ok(s) => s,
        Err(error) => {
            set_last_secure_error(error.clone());
            return error.code as c_int;
        }
    };

    // Perform conversion with error handling
    match markdown_to_rtf(&markdown_string) {
        Ok(rtf) => {
            let c_str = match CString::new(rtf.clone()) {
                Ok(s) => s.into_raw(),
                Err(_) => {
                    let error = SecureError {
                        code: SecureErrorCode::InternalError,
                        message: "Failed to process output".to_string(),
                        error_id: Some(generate_error_id()),
                    };
                    set_last_secure_error(error.clone());
                    return error.code as c_int;
                }
            };
            
            *output_buffer = c_str;
            *output_length = rtf.len() as c_int;
            SecureErrorCode::Success as c_int
        }
        Err(internal_error) => {
            // Log internal error securely
            let error_id = generate_error_id();
            error!(
                error_id = %error_id,
                error_type = "markdown_to_rtf",
                "Conversion failed"
            );
            
            let error = SecureError {
                code: SecureErrorCode::ConversionFailed,
                message: "Document conversion failed".to_string(),
                error_id: Some(error_id),
            };
            set_last_secure_error(error.clone());
            error.code as c_int
        }
    }
}

/// Generate unique error ID for tracking
fn generate_error_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    use rand::Rng;
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let random: u32 = rand::thread_rng().gen();
    
    format!("ERR-{}-{:08X}", timestamp, random)
}

/// Test connection with secure response
#[no_mangle]
pub extern "C" fn legacybridge_test_connection_secure() -> c_int {
    1 // Always return success
}

/// Get version information securely
#[no_mangle]
pub extern "C" fn legacybridge_get_version_secure() -> *const c_char {
    static VERSION: &[u8] = b"1.0.0\0";
    VERSION.as_ptr() as *const c_char
}

/// Free a string allocated by the library
#[no_mangle]
pub unsafe extern "C" fn legacybridge_free_string_secure(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_error_generation() {
        let error = SecureError {
            code: SecureErrorCode::InvalidInput,
            message: "Test error".to_string(),
            error_id: Some(generate_error_id()),
        };
        
        assert!(error.error_id.is_some());
        assert!(error.error_id.unwrap().starts_with("ERR-"));
    }

    #[test]
    fn test_secure_connection() {
        let result = unsafe { legacybridge_test_connection_secure() };
        assert_eq!(result, 1);
    }
}