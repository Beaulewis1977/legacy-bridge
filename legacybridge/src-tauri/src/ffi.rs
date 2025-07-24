// FFI (Foreign Function Interface) module for VB6/VFP9 compatibility
// This module provides C-compatible exports for the MD→RTF conversion system

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::slice;

use crate::conversion::{markdown_to_rtf, rtf_to_markdown};

/// Error codes for FFI functions
#[repr(C)]
pub enum FFIErrorCode {
    Success = 0,
    NullPointer = -1,
    InvalidUtf8 = -2,
    ConversionError = -3,
    AllocationError = -4,
}

/// Convert a C string to a Rust String
unsafe fn c_str_to_string(ptr: *const c_char) -> Result<String, FFIErrorCode> {
    if ptr.is_null() {
        return Err(FFIErrorCode::NullPointer);
    }
    
    CStr::from_ptr(ptr)
        .to_str()
        .map(|s| s.to_owned())
        .map_err(|_| FFIErrorCode::InvalidUtf8)
}

/// Convert a Rust String to a C string (caller must free)
fn string_to_c_str(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Convert RTF to Markdown
/// 
/// # Parameters
/// - `rtf_content`: Null-terminated C string containing RTF content
/// - `output_buffer`: Pointer to store the output buffer address
/// - `output_length`: Pointer to store the output length
/// 
/// # Returns
/// - 0 on success
/// - Negative error code on failure
/// 
/// # Safety
/// The caller must free the output buffer using `legacybridge_free_string`
#[no_mangle]
pub unsafe extern "C" fn legacybridge_rtf_to_markdown(
    rtf_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    // Validate input parameters
    if rtf_content.is_null() || output_buffer.is_null() || output_length.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }

    // Convert input to Rust String
    let rtf_string = match c_str_to_string(rtf_content) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };

    // Perform conversion
    match rtf_to_markdown(&rtf_string) {
        Ok(markdown) => {
            let c_str = string_to_c_str(markdown.clone());
            if c_str.is_null() {
                return FFIErrorCode::AllocationError as c_int;
            }
            
            *output_buffer = c_str;
            *output_length = markdown.len() as c_int;
            FFIErrorCode::Success as c_int
        }
        Err(_) => FFIErrorCode::ConversionError as c_int,
    }
}

/// Convert Markdown to RTF
/// 
/// # Parameters
/// - `markdown_content`: Null-terminated C string containing Markdown content
/// - `output_buffer`: Pointer to store the output buffer address
/// - `output_length`: Pointer to store the output length
/// 
/// # Returns
/// - 0 on success
/// - Negative error code on failure
/// 
/// # Safety
/// The caller must free the output buffer using `legacybridge_free_string`
#[no_mangle]
pub unsafe extern "C" fn legacybridge_markdown_to_rtf(
    markdown_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    // Validate input parameters
    if markdown_content.is_null() || output_buffer.is_null() || output_length.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }

    // Convert input to Rust String
    let markdown_string = match c_str_to_string(markdown_content) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };

    // Perform conversion
    match markdown_to_rtf(&markdown_string) {
        Ok(rtf) => {
            let c_str = string_to_c_str(rtf.clone());
            if c_str.is_null() {
                return FFIErrorCode::AllocationError as c_int;
            }
            
            *output_buffer = c_str;
            *output_length = rtf.len() as c_int;
            FFIErrorCode::Success as c_int
        }
        Err(_) => FFIErrorCode::ConversionError as c_int,
    }
}

/// Free a string allocated by the library
/// 
/// # Parameters
/// - `ptr`: Pointer to the string to free
/// 
/// # Safety
/// This function should only be called on strings allocated by this library
#[no_mangle]
pub unsafe extern "C" fn legacybridge_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

/// Get the last error message
/// 
/// # Parameters
/// - `buffer`: Buffer to store the error message
/// - `buffer_size`: Size of the buffer
/// 
/// # Returns
/// - Number of bytes written (excluding null terminator)
/// - -1 if buffer is too small
#[no_mangle]
pub unsafe extern "C" fn legacybridge_get_last_error(
    buffer: *mut c_char,
    buffer_size: c_int,
) -> c_int {
    if buffer.is_null() || buffer_size <= 0 {
        return -1;
    }

    // For now, return a generic error message
    // In a production system, you'd store thread-local error information
    let error_msg = "Last operation failed";
    let error_bytes = error_msg.as_bytes();
    
    let copy_len = std::cmp::min(error_bytes.len(), (buffer_size - 1) as usize);
    
    let buffer_slice = slice::from_raw_parts_mut(buffer as *mut u8, copy_len);
    buffer_slice.copy_from_slice(&error_bytes[..copy_len]);
    
    // Null terminate
    *buffer.add(copy_len) = 0;
    
    copy_len as c_int
}

/// Get library version
/// 
/// # Returns
/// - Version string as a C string (do not free)
#[no_mangle]
pub extern "C" fn legacybridge_get_version() -> *const c_char {
    static VERSION: &[u8] = b"1.0.0\0";
    VERSION.as_ptr() as *const c_char
}

/// Batch conversion: Convert multiple RTF files to Markdown
/// 
/// # Parameters
/// - `rtf_array`: Array of RTF content strings
/// - `count`: Number of items to convert
/// - `output_array`: Array to store output pointers
/// - `output_lengths`: Array to store output lengths
/// 
/// # Returns
/// - Number of successful conversions
/// 
/// # Safety
/// The caller must free each output string using `legacybridge_free_string`
#[no_mangle]
pub unsafe extern "C" fn legacybridge_batch_rtf_to_markdown(
    rtf_array: *const *const c_char,
    count: c_int,
    output_array: *mut *mut c_char,
    output_lengths: *mut c_int,
) -> c_int {
    if rtf_array.is_null() || output_array.is_null() || output_lengths.is_null() || count <= 0 {
        return 0;
    }

    let mut success_count = 0;
    
    for i in 0..count as usize {
        let rtf_ptr = *rtf_array.add(i);
        let output_ptr = output_array.add(i);
        let length_ptr = output_lengths.add(i);
        
        let result = legacybridge_rtf_to_markdown(rtf_ptr, output_ptr, length_ptr);
        
        if result == 0 {
            success_count += 1;
        } else {
            *output_ptr = ptr::null_mut();
            *length_ptr = 0;
        }
    }
    
    success_count
}

/// Batch conversion: Convert multiple Markdown files to RTF
/// 
/// # Parameters
/// - `markdown_array`: Array of Markdown content strings
/// - `count`: Number of items to convert
/// - `output_array`: Array to store output pointers
/// - `output_lengths`: Array to store output lengths
/// 
/// # Returns
/// - Number of successful conversions
/// 
/// # Safety
/// The caller must free each output string using `legacybridge_free_string`
#[no_mangle]
pub unsafe extern "C" fn legacybridge_batch_markdown_to_rtf(
    markdown_array: *const *const c_char,
    count: c_int,
    output_array: *mut *mut c_char,
    output_lengths: *mut c_int,
) -> c_int {
    if markdown_array.is_null() || output_array.is_null() || output_lengths.is_null() || count <= 0 {
        return 0;
    }

    let mut success_count = 0;
    
    for i in 0..count as usize {
        let markdown_ptr = *markdown_array.add(i);
        let output_ptr = output_array.add(i);
        let length_ptr = output_lengths.add(i);
        
        let result = legacybridge_markdown_to_rtf(markdown_ptr, output_ptr, length_ptr);
        
        if result == 0 {
            success_count += 1;
        } else {
            *output_ptr = ptr::null_mut();
            *length_ptr = 0;
        }
    }
    
    success_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let version = unsafe { legacybridge_get_version() };
        assert!(!version.is_null());
    }
}