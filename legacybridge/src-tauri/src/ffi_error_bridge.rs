// FFI Error Bridge Module
//
// This module provides safe error propagation across the C FFI boundary
// with full error context preservation and JSON serialization.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use crate::conversion::unified_errors::{
    LegacyBridgeError, set_last_error, get_last_error_json, clear_last_error
};

/// Get the last error as a JSON string
/// 
/// Returns a JSON object with full error details including:
/// - error_type: The error category
/// - error_code: Numeric error code for programmatic handling
/// - message: Developer-facing error message
/// - user_message: User-friendly error message
/// - details: Full error context
/// - suggestions: Array of suggestions to fix the error
/// - recoverable: Whether the error can be recovered from
/// - timestamp: When the error occurred
///
/// # Safety
/// The caller must free the returned string using `legacybridge_free_string`
#[no_mangle]
pub unsafe extern "C" fn legacybridge_get_last_error_json() -> *mut c_char {
    match get_last_error_json() {
        Some(json) => match CString::new(json) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => ptr::null_mut(),
        },
        None => {
            // Return a default "no error" response
            let no_error = r#"{"error_type":"NoError","error_code":0,"message":"No error","user_message":"No error","details":null,"suggestions":[],"recoverable":true}"#;
            match CString::new(no_error) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => ptr::null_mut(),
            }
        }
    }
}

/// Clear the last error
#[no_mangle]
pub extern "C" fn legacybridge_clear_last_error() {
    clear_last_error();
}

/// Set a custom error from FFI side
///
/// # Parameters
/// - `error_type`: Type of error (ParseError, ConversionError, etc.)
/// - `message`: Error message
/// - `error_code`: Optional error code
///
/// # Safety
/// All string parameters must be valid null-terminated C strings
#[no_mangle]
pub unsafe extern "C" fn legacybridge_set_error(
    error_type: *const c_char,
    message: *const c_char,
    error_code: i32,
) -> i32 {
    if error_type.is_null() || message.is_null() {
        return -1;
    }

    let error_type_str = match CStr::from_ptr(error_type).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let message_str = match CStr::from_ptr(message).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    // Create appropriate error based on type
    let error = match error_type_str {
        "ParseError" => LegacyBridgeError::ParseError {
            message: message_str.to_string(),
            line: 0,
            column: 0,
            expected: None,
            found: None,
            file_path: None,
        },
        "ConversionError" => LegacyBridgeError::ConversionError {
            source_format: "unknown".to_string(),
            target_format: "unknown".to_string(),
            details: message_str.to_string(),
            recoverable: false,
            suggestions: vec![],
        },
        "IOError" => LegacyBridgeError::IOError {
            operation: "unknown".to_string(),
            path: "unknown".to_string(),
            cause: message_str.to_string(),
            error_code: Some(error_code),
        },
        "ValidationError" => LegacyBridgeError::ValidationError {
            field: "unknown".to_string(),
            expected: "valid input".to_string(),
            received: message_str.to_string(),
            location: None,
        },
        _ => LegacyBridgeError::SystemError {
            component: "FFI".to_string(),
            error_code: error_code as u32,
            description: message_str.to_string(),
            internal_message: None,
        },
    };

    set_last_error(error);
    0
}

/// Helper function to set error and return error code
pub(crate) fn set_and_return_error(error: LegacyBridgeError) -> i32 {
    let code = error.error_code();
    set_last_error(error);
    code
}

/// Convert a ConversionError to LegacyBridgeError and set it
pub(crate) fn handle_conversion_error(err: crate::conversion::types::ConversionError) -> i32 {
    let legacy_err: LegacyBridgeError = err.into();
    set_and_return_error(legacy_err)
}

/// Create a parse error with location information
pub(crate) fn create_parse_error(
    message: &str,
    line: u32,
    column: u32,
    file_path: Option<&str>,
) -> LegacyBridgeError {
    LegacyBridgeError::ParseError {
        message: message.to_string(),
        line,
        column,
        expected: None,
        found: None,
        file_path: file_path.map(|s| s.to_string()),
    }
}

/// Create an IO error with context
pub(crate) fn create_io_error(
    operation: &str,
    path: &str,
    cause: std::io::Error,
) -> LegacyBridgeError {
    LegacyBridgeError::IOError {
        operation: operation.to_string(),
        path: path.to_string(),
        cause: cause.to_string(),
        error_code: cause.raw_os_error(),
    }
}

/// Create a conversion error with suggestions
pub(crate) fn create_conversion_error(
    source_format: &str,
    target_format: &str,
    details: &str,
    suggestions: Vec<String>,
) -> LegacyBridgeError {
    LegacyBridgeError::ConversionError {
        source_format: source_format.to_string(),
        target_format: target_format.to_string(),
        details: details.to_string(),
        recoverable: !suggestions.is_empty(),
        suggestions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_bridge() {
        let error = create_parse_error("Unexpected token", 10, 5, Some("test.rtf"));
        set_last_error(error);
        
        let json = get_last_error_json().unwrap();
        assert!(json.contains("ParseError"));
        assert!(json.contains("\"line\":10"));
        assert!(json.contains("\"column\":5"));
        assert!(json.contains("test.rtf"));
    }

    #[test]
    fn test_io_error_creation() {
        use std::io::{Error, ErrorKind};
        
        let io_err = Error::new(ErrorKind::NotFound, "File not found");
        let error = create_io_error("read", "/path/to/file.rtf", io_err);
        
        assert_eq!(error.error_type(), "IOError");
        assert!(error.to_string().contains("read"));
        assert!(error.to_string().contains("/path/to/file.rtf"));
    }

    #[test]
    fn test_conversion_error_with_suggestions() {
        let error = create_conversion_error(
            "RTF",
            "Markdown",
            "Complex table structure not supported",
            vec![
                "Simplify table structure".to_string(),
                "Use nested lists instead".to_string(),
            ],
        );
        
        assert!(error.is_recoverable());
        assert_eq!(error.suggestions().len(), 2);
    }
}