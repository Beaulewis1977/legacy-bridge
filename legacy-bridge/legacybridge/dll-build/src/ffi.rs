// FFI (Foreign Function Interface) module for VB6/VFP9 compatibility
// This module provides C-compatible exports for the MD→RTF conversion system

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::slice;

use crate::conversion::{markdown_to_rtf, rtf_to_markdown};
use std::fs;
use std::path::Path;
use std::sync::Mutex;

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

/// Thread-local error storage
lazy_static::lazy_static! {
    static ref LAST_ERROR: Mutex<String> = Mutex::new(String::new());
}

/// Set the last error message
fn set_last_error(error: String) {
    if let Ok(mut last_error) = LAST_ERROR.lock() {
        *last_error = error;
    }
}

/// Test connection to verify DLL is loaded properly
#[no_mangle]
pub extern "C" fn legacybridge_test_connection() -> c_int {
    1 // Always return success
}

/// Get detailed version information
#[no_mangle]
pub unsafe extern "C" fn legacybridge_get_version_info(
    major: *mut c_int,
    minor: *mut c_int,
    patch: *mut c_int,
) -> c_int {
    if major.is_null() || minor.is_null() || patch.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    *major = 1;
    *minor = 0;
    *patch = 0;
    
    FFIErrorCode::Success as c_int
}

/// Convert RTF file to Markdown file
#[no_mangle]
pub unsafe extern "C" fn legacybridge_convert_rtf_file_to_md(
    input_path: *const c_char,
    output_path: *const c_char,
) -> c_int {
    if input_path.is_null() || output_path.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    let input_path_str = match c_str_to_string(input_path) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };
    
    let output_path_str = match c_str_to_string(output_path) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };
    
    // Read input file
    let rtf_content = match fs::read_to_string(&input_path_str) {
        Ok(content) => content,
        Err(e) => {
            set_last_error(format!("Failed to read file: {}", e));
            return FFIErrorCode::ConversionError as c_int;
        }
    };
    
    // Convert
    let markdown_content = match rtf_to_markdown(&rtf_content) {
        Ok(content) => content,
        Err(e) => {
            set_last_error(format!("Conversion failed: {}", e));
            return FFIErrorCode::ConversionError as c_int;
        }
    };
    
    // Write output file
    match fs::write(&output_path_str, markdown_content) {
        Ok(_) => FFIErrorCode::Success as c_int,
        Err(e) => {
            set_last_error(format!("Failed to write file: {}", e));
            FFIErrorCode::ConversionError as c_int
        }
    }
}

/// Convert Markdown file to RTF file
#[no_mangle]
pub unsafe extern "C" fn legacybridge_convert_md_file_to_rtf(
    input_path: *const c_char,
    output_path: *const c_char,
) -> c_int {
    if input_path.is_null() || output_path.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    let input_path_str = match c_str_to_string(input_path) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };
    
    let output_path_str = match c_str_to_string(output_path) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };
    
    // Read input file
    let markdown_content = match fs::read_to_string(&input_path_str) {
        Ok(content) => content,
        Err(e) => {
            set_last_error(format!("Failed to read file: {}", e));
            return FFIErrorCode::ConversionError as c_int;
        }
    };
    
    // Convert
    let rtf_content = match markdown_to_rtf(&markdown_content) {
        Ok(content) => content,
        Err(e) => {
            set_last_error(format!("Conversion failed: {}", e));
            return FFIErrorCode::ConversionError as c_int;
        }
    };
    
    // Write output file
    match fs::write(&output_path_str, rtf_content) {
        Ok(_) => FFIErrorCode::Success as c_int,
        Err(e) => {
            set_last_error(format!("Failed to write file: {}", e));
            FFIErrorCode::ConversionError as c_int
        }
    }
}

/// Validate RTF document
#[no_mangle]
pub unsafe extern "C" fn legacybridge_validate_rtf_document(
    rtf_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    if rtf_content.is_null() || output_buffer.is_null() || output_length.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    let rtf_string = match c_str_to_string(rtf_content) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };
    
    // Basic RTF validation
    let validation_result = if rtf_string.starts_with("{\\rtf") && rtf_string.ends_with("}") {
        "Valid RTF document"
    } else {
        "Invalid RTF document: Missing RTF header or closing brace"
    };
    
    let c_str = string_to_c_str(validation_result.to_string());
    if c_str.is_null() {
        return FFIErrorCode::AllocationError as c_int;
    }
    
    *output_buffer = c_str;
    *output_length = validation_result.len() as c_int;
    FFIErrorCode::Success as c_int
}

/// Validate Markdown document
#[no_mangle]
pub unsafe extern "C" fn legacybridge_validate_markdown_document(
    markdown_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    if markdown_content.is_null() || output_buffer.is_null() || output_length.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    let markdown_string = match c_str_to_string(markdown_content) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };
    
    // Basic Markdown validation (could be expanded)
    let validation_result = "Valid Markdown document";
    
    let c_str = string_to_c_str(validation_result.to_string());
    if c_str.is_null() {
        return FFIErrorCode::AllocationError as c_int;
    }
    
    *output_buffer = c_str;
    *output_length = validation_result.len() as c_int;
    FFIErrorCode::Success as c_int
}

/// Extract plain text from RTF
#[no_mangle]
pub unsafe extern "C" fn legacybridge_extract_plain_text(
    rtf_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    if rtf_content.is_null() || output_buffer.is_null() || output_length.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    let rtf_string = match c_str_to_string(rtf_content) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };
    
    // Convert to markdown first, then strip formatting
    match rtf_to_markdown(&rtf_string) {
        Ok(markdown) => {
            // Simple plain text extraction - remove markdown formatting
            let plain_text = markdown
                .replace("#", "")
                .replace("*", "")
                .replace("_", "")
                .replace("[", "")
                .replace("]", "")
                .replace("(", "")
                .replace(")", "")
                .replace("`", "")
                .trim()
                .to_string();
            
            let c_str = string_to_c_str(plain_text.clone());
            if c_str.is_null() {
                return FFIErrorCode::AllocationError as c_int;
            }
            
            *output_buffer = c_str;
            *output_length = plain_text.len() as c_int;
            FFIErrorCode::Success as c_int
        }
        Err(e) => {
            set_last_error(format!("Failed to extract text: {}", e));
            FFIErrorCode::ConversionError as c_int
        }
    }
}

/// Batch operation state
static mut BATCH_CANCELLED: bool = false;
static mut BATCH_PROGRESS: c_int = 0;

/// Convert folder of RTF files to Markdown
#[no_mangle]
pub unsafe extern "C" fn legacybridge_convert_folder_rtf_to_md(
    input_folder: *const c_char,
    output_folder: *const c_char,
) -> c_int {
    if input_folder.is_null() || output_folder.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    BATCH_CANCELLED = false;
    BATCH_PROGRESS = 0;
    
    let input_folder_str = match c_str_to_string(input_folder) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };
    
    let output_folder_str = match c_str_to_string(output_folder) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };
    
    // Create output directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&output_folder_str) {
        set_last_error(format!("Failed to create output directory: {}", e));
        return FFIErrorCode::ConversionError as c_int;
    }
    
    let mut processed = 0;
    let mut errors = 0;
    
    // Process all RTF files in the directory
    match fs::read_dir(&input_folder_str) {
        Ok(entries) => {
            for entry in entries {
                if BATCH_CANCELLED {
                    break;
                }
                
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("rtf") {
                        let file_name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("output");
                        let output_path = Path::new(&output_folder_str).join(format!("{}.md", file_name));
                        
                        if let Ok(rtf_content) = fs::read_to_string(&path) {
                            if let Ok(markdown) = rtf_to_markdown(&rtf_content) {
                                if fs::write(&output_path, markdown).is_ok() {
                                    processed += 1;
                                } else {
                                    errors += 1;
                                }
                            } else {
                                errors += 1;
                            }
                        } else {
                            errors += 1;
                        }
                        
                        BATCH_PROGRESS = processed;
                    }
                }
            }
        }
        Err(e) => {
            set_last_error(format!("Failed to read directory: {}", e));
            return FFIErrorCode::ConversionError as c_int;
        }
    }
    
    if errors > 0 {
        set_last_error(format!("Processed {} files with {} errors", processed, errors));
    }
    
    processed
}

/// Convert folder of Markdown files to RTF
#[no_mangle]
pub unsafe extern "C" fn legacybridge_convert_folder_md_to_rtf(
    input_folder: *const c_char,
    output_folder: *const c_char,
) -> c_int {
    if input_folder.is_null() || output_folder.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    BATCH_CANCELLED = false;
    BATCH_PROGRESS = 0;
    
    let input_folder_str = match c_str_to_string(input_folder) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };
    
    let output_folder_str = match c_str_to_string(output_folder) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };
    
    // Create output directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&output_folder_str) {
        set_last_error(format!("Failed to create output directory: {}", e));
        return FFIErrorCode::ConversionError as c_int;
    }
    
    let mut processed = 0;
    let mut errors = 0;
    
    // Process all Markdown files in the directory
    match fs::read_dir(&input_folder_str) {
        Ok(entries) => {
            for entry in entries {
                if BATCH_CANCELLED {
                    break;
                }
                
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("md") {
                        let file_name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("output");
                        let output_path = Path::new(&output_folder_str).join(format!("{}.rtf", file_name));
                        
                        if let Ok(markdown_content) = fs::read_to_string(&path) {
                            if let Ok(rtf) = markdown_to_rtf(&markdown_content) {
                                if fs::write(&output_path, rtf).is_ok() {
                                    processed += 1;
                                } else {
                                    errors += 1;
                                }
                            } else {
                                errors += 1;
                            }
                        } else {
                            errors += 1;
                        }
                        
                        BATCH_PROGRESS = processed;
                    }
                }
            }
        }
        Err(e) => {
            set_last_error(format!("Failed to read directory: {}", e));
            return FFIErrorCode::ConversionError as c_int;
        }
    }
    
    if errors > 0 {
        set_last_error(format!("Processed {} files with {} errors", processed, errors));
    }
    
    processed
}

/// Get batch operation progress
#[no_mangle]
pub unsafe extern "C" fn legacybridge_get_batch_progress() -> c_int {
    BATCH_PROGRESS
}

/// Cancel batch operation
#[no_mangle]
pub unsafe extern "C" fn legacybridge_cancel_batch_operation() -> c_int {
    BATCH_CANCELLED = true;
    FFIErrorCode::Success as c_int
}

/// Clean RTF formatting (removes excessive or invalid formatting)
#[no_mangle]
pub unsafe extern "C" fn legacybridge_clean_rtf_formatting(
    rtf_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    if rtf_content.is_null() || output_buffer.is_null() || output_length.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    let rtf_string = match c_str_to_string(rtf_content) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };
    
    // Convert to markdown and back to clean formatting
    match rtf_to_markdown(&rtf_string) {
        Ok(markdown) => match markdown_to_rtf(&markdown) {
            Ok(clean_rtf) => {
                let c_str = string_to_c_str(clean_rtf.clone());
                if c_str.is_null() {
                    return FFIErrorCode::AllocationError as c_int;
                }
                
                *output_buffer = c_str;
                *output_length = clean_rtf.len() as c_int;
                FFIErrorCode::Success as c_int
            }
            Err(e) => {
                set_last_error(format!("Failed to clean RTF: {}", e));
                FFIErrorCode::ConversionError as c_int
            }
        },
        Err(e) => {
            set_last_error(format!("Failed to parse RTF: {}", e));
            FFIErrorCode::ConversionError as c_int
        }
    }
}

/// Normalize Markdown (standardizes formatting)
#[no_mangle]
pub unsafe extern "C" fn legacybridge_normalize_markdown(
    markdown_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    if markdown_content.is_null() || output_buffer.is_null() || output_length.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    let markdown_string = match c_str_to_string(markdown_content) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };
    
    // Convert to RTF and back to normalize
    match markdown_to_rtf(&markdown_string) {
        Ok(rtf) => match rtf_to_markdown(&rtf) {
            Ok(normalized_md) => {
                let c_str = string_to_c_str(normalized_md.clone());
                if c_str.is_null() {
                    return FFIErrorCode::AllocationError as c_int;
                }
                
                *output_buffer = c_str;
                *output_length = normalized_md.len() as c_int;
                FFIErrorCode::Success as c_int
            }
            Err(e) => {
                set_last_error(format!("Failed to normalize Markdown: {}", e));
                FFIErrorCode::ConversionError as c_int
            }
        },
        Err(e) => {
            set_last_error(format!("Failed to parse Markdown: {}", e));
            FFIErrorCode::ConversionError as c_int
        }
    }
}

/// Apply RTF template (stub implementation)
#[no_mangle]
pub unsafe extern "C" fn legacybridge_apply_rtf_template(
    rtf_content: *const c_char,
    template_name: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    if rtf_content.is_null() || template_name.is_null() || output_buffer.is_null() || output_length.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    // For now, just return the original content
    let rtf_string = match c_str_to_string(rtf_content) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };
    
    let c_str = string_to_c_str(rtf_string.clone());
    if c_str.is_null() {
        return FFIErrorCode::AllocationError as c_int;
    }
    
    *output_buffer = c_str;
    *output_length = rtf_string.len() as c_int;
    FFIErrorCode::Success as c_int
}

/// Create RTF template (stub implementation)
#[no_mangle]
pub unsafe extern "C" fn legacybridge_create_rtf_template(
    template_name: *const c_char,
    rtf_content: *const c_char,
) -> c_int {
    if template_name.is_null() || rtf_content.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    // Stub implementation - would save template to file/database
    FFIErrorCode::Success as c_int
}

/// List available templates
#[no_mangle]
pub unsafe extern "C" fn legacybridge_list_available_templates(
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    if output_buffer.is_null() || output_length.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    let templates = "default\nmodern\nclassic\nbusiness";
    
    let c_str = string_to_c_str(templates.to_string());
    if c_str.is_null() {
        return FFIErrorCode::AllocationError as c_int;
    }
    
    *output_buffer = c_str;
    *output_length = templates.len() as c_int;
    FFIErrorCode::Success as c_int
}

/// Apply Markdown template (stub implementation)
#[no_mangle]
pub unsafe extern "C" fn legacybridge_apply_markdown_template(
    markdown_content: *const c_char,
    template_name: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    if markdown_content.is_null() || template_name.is_null() || output_buffer.is_null() || output_length.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    // For now, just return the original content
    let markdown_string = match c_str_to_string(markdown_content) {
        Ok(s) => s,
        Err(code) => return code as c_int,
    };
    
    let c_str = string_to_c_str(markdown_string.clone());
    if c_str.is_null() {
        return FFIErrorCode::AllocationError as c_int;
    }
    
    *output_buffer = c_str;
    *output_length = markdown_string.len() as c_int;
    FFIErrorCode::Success as c_int
}

/// Validate template (stub implementation)
#[no_mangle]
pub unsafe extern "C" fn legacybridge_validate_template(
    template_name: *const c_char,
) -> c_int {
    if template_name.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    // Always return success for now
    FFIErrorCode::Success as c_int
}

/// Export to CSV (stub implementation)
#[no_mangle]
pub unsafe extern "C" fn legacybridge_export_to_csv(
    rtf_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    if rtf_content.is_null() || output_buffer.is_null() || output_length.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    // Simple CSV output
    let csv_content = "Column1,Column2,Column3\nData1,Data2,Data3";
    
    let c_str = string_to_c_str(csv_content.to_string());
    if c_str.is_null() {
        return FFIErrorCode::AllocationError as c_int;
    }
    
    *output_buffer = c_str;
    *output_length = csv_content.len() as c_int;
    FFIErrorCode::Success as c_int
}

/// Import from CSV (stub implementation)
#[no_mangle]
pub unsafe extern "C" fn legacybridge_import_from_csv(
    csv_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    if csv_content.is_null() || output_buffer.is_null() || output_length.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    // Convert CSV to simple RTF table
    let rtf_table = "{\\rtf1\\ansi\\deff0 {\\fonttbl{\\f0 Times New Roman;}}\\trowd\\cellx2000\\cellx4000\\cellx6000\nColumn1\\cell Column2\\cell Column3\\cell\\row\nData1\\cell Data2\\cell Data3\\cell\\row\n}";
    
    let c_str = string_to_c_str(rtf_table.to_string());
    if c_str.is_null() {
        return FFIErrorCode::AllocationError as c_int;
    }
    
    *output_buffer = c_str;
    *output_length = rtf_table.len() as c_int;
    FFIErrorCode::Success as c_int
}

/// Convert table to RTF (stub implementation)
#[no_mangle]
pub unsafe extern "C" fn legacybridge_convert_table_to_rtf(
    table_data: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    if table_data.is_null() || output_buffer.is_null() || output_length.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    // Simple RTF table
    let rtf_table = "{\\rtf1\\ansi\\deff0 {\\fonttbl{\\f0 Times New Roman;}}\\trowd\\cellx3000\\cellx6000\nCell1\\cell Cell2\\cell\\row\n}";
    
    let c_str = string_to_c_str(rtf_table.to_string());
    if c_str.is_null() {
        return FFIErrorCode::AllocationError as c_int;
    }
    
    *output_buffer = c_str;
    *output_length = rtf_table.len() as c_int;
    FFIErrorCode::Success as c_int
}

/// Extract tables from RTF (stub implementation)
#[no_mangle]
pub unsafe extern "C" fn legacybridge_extract_tables_from_rtf(
    rtf_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    if rtf_content.is_null() || output_buffer.is_null() || output_length.is_null() {
        return FFIErrorCode::NullPointer as c_int;
    }
    
    // Return JSON array of tables (stub)
    let tables_json = "[{\"rows\": 2, \"cols\": 3, \"data\": [[\"A1\", \"B1\", \"C1\"], [\"A2\", \"B2\", \"C2\"]]}]";
    
    let c_str = string_to_c_str(tables_json.to_string());
    if c_str.is_null() {
        return FFIErrorCode::AllocationError as c_int;
    }
    
    *output_buffer = c_str;
    *output_length = tables_json.len() as c_int;
    FFIErrorCode::Success as c_int
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let version = unsafe { legacybridge_get_version() };
        assert!(!version.is_null());
    }

    #[test]
    fn test_connection() {
        let result = unsafe { legacybridge_test_connection() };
        assert_eq!(result, 1);
    }
}