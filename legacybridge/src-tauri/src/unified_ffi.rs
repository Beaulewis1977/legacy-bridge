// Unified FFI (Foreign Function Interface) module for VB6/VFP9 compatibility
// This module consolidates secure and standard FFI implementations with
// configurable security levels

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::sync::Mutex;

use crate::conversion::{
    ConversionConfig, ConversionConfigBuilder, SecurityLevel,
    rtf_to_markdown_with_config, markdown_to_rtf_with_config,
};

/// FFI configuration structure for C interop
#[repr(C)]
pub struct FFIConfig {
    /// Security level: 0=Standard, 1=Enhanced, 2=Paranoid
    pub security_level: c_int,
    /// Enable validation: 0=false, 1=true
    pub enable_validation: c_int,
    /// Enable logging: 0=false, 1=true
    pub enable_logging: c_int,
    /// Enable pipeline: 0=false, 1=true
    pub enable_pipeline: c_int,
    /// Enable auto recovery: 0=false, 1=true
    pub enable_auto_recovery: c_int,
    /// Timeout in seconds (0 = no timeout)
    pub timeout_seconds: c_int,
}

/// Error codes for FFI functions
#[repr(C)]
pub enum FFIErrorCode {
    Success = 0,
    NullPointer = -1,
    InvalidUtf8 = -2,
    ConversionError = -3,
    AllocationError = -4,
    InvalidConfig = -5,
    TimeoutError = -6,
}

/// Thread-local error storage
lazy_static::lazy_static! {
    static ref LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);
}

/// Set the last error message
fn set_last_error(error: String) {
    if let Ok(mut last_error) = LAST_ERROR.lock() {
        *last_error = Some(error);
    }
}

/// Convert FFI config to Rust config
fn ffi_config_to_rust(ffi_config: *const FFIConfig) -> ConversionConfig {
    if ffi_config.is_null() {
        return ConversionConfig::default();
    }
    
    unsafe {
        let config = &*ffi_config;
        let mut builder = ConversionConfigBuilder::new();
        
        // Set security level
        let security_level = match config.security_level {
            2 => SecurityLevel::paranoid(),
            1 => SecurityLevel::Enhanced {
                limits: crate::conversion::security::SecurityLimits::default(),
            },
            _ => SecurityLevel::Standard,
        };
        builder = builder.security_level(security_level);
        
        // Set other options
        builder = builder
            .validation(config.enable_validation != 0)
            .logging(config.enable_logging != 0)
            .pipeline(config.enable_pipeline != 0)
            .auto_recovery(config.enable_auto_recovery != 0);
            
        // Set timeout if specified
        if config.timeout_seconds > 0 {
            builder = builder.timeout(std::time::Duration::from_secs(config.timeout_seconds as u64));
        }
        
        builder.build()
    }
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

/// Unified RTF to Markdown conversion
/// 
/// # Parameters
/// - `rtf_content`: Null-terminated C string containing RTF content
/// - `output_buffer`: Pointer to store the output buffer address
/// - `output_length`: Pointer to store the output length
/// - `config`: Optional configuration (can be NULL for defaults)
/// 
/// # Returns
/// - 0 on success
/// - Negative error code on failure
/// 
/// # Safety
/// The caller must free the output buffer using `legacybridge_free_string`
#[no_mangle]
pub unsafe extern "C" fn legacybridge_rtf_to_markdown_unified(
    rtf_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
    config: *const FFIConfig,
) -> c_int {
    // Validate required parameters
    if rtf_content.is_null() || output_buffer.is_null() || output_length.is_null() {
        set_last_error("Invalid parameters: null pointer provided".to_string());
        return FFIErrorCode::NullPointer as c_int;
    }

    // Convert input to Rust String
    let rtf_string = match c_str_to_string(rtf_content) {
        Ok(s) => s,
        Err(code) => {
            set_last_error("Invalid UTF-8 in input".to_string());
            return code as c_int;
        }
    };

    // Convert configuration
    let rust_config = ffi_config_to_rust(config);

    // Perform conversion
    match rtf_to_markdown_with_config(&rtf_string, rust_config) {
        Ok(markdown) => {
            let c_str = string_to_c_str(markdown.clone());
            if c_str.is_null() {
                set_last_error("Failed to allocate output buffer".to_string());
                return FFIErrorCode::AllocationError as c_int;
            }
            
            *output_buffer = c_str;
            *output_length = markdown.len() as c_int;
            FFIErrorCode::Success as c_int
        }
        Err(e) => {
            set_last_error(format!("Conversion failed: {}", e));
            FFIErrorCode::ConversionError as c_int
        }
    }
}

/// Unified Markdown to RTF conversion
/// 
/// # Parameters
/// - `markdown_content`: Null-terminated C string containing Markdown content
/// - `output_buffer`: Pointer to store the output buffer address
/// - `output_length`: Pointer to store the output length
/// - `config`: Optional configuration (can be NULL for defaults)
/// 
/// # Returns
/// - 0 on success
/// - Negative error code on failure
/// 
/// # Safety
/// The caller must free the output buffer using `legacybridge_free_string`
#[no_mangle]
pub unsafe extern "C" fn legacybridge_markdown_to_rtf_unified(
    markdown_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
    config: *const FFIConfig,
) -> c_int {
    // Validate required parameters
    if markdown_content.is_null() || output_buffer.is_null() || output_length.is_null() {
        set_last_error("Invalid parameters: null pointer provided".to_string());
        return FFIErrorCode::NullPointer as c_int;
    }

    // Convert input to Rust String
    let markdown_string = match c_str_to_string(markdown_content) {
        Ok(s) => s,
        Err(code) => {
            set_last_error("Invalid UTF-8 in input".to_string());
            return code as c_int;
        }
    };

    // Convert configuration
    let rust_config = ffi_config_to_rust(config);

    // Perform conversion
    match markdown_to_rtf_with_config(&markdown_string, rust_config) {
        Ok(rtf) => {
            let c_str = string_to_c_str(rtf.clone());
            if c_str.is_null() {
                set_last_error("Failed to allocate output buffer".to_string());
                return FFIErrorCode::AllocationError as c_int;
            }
            
            *output_buffer = c_str;
            *output_length = rtf.len() as c_int;
            FFIErrorCode::Success as c_int
        }
        Err(e) => {
            set_last_error(format!("Conversion failed: {}", e));
            FFIErrorCode::ConversionError as c_int
        }
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

    let error_msg = LAST_ERROR.lock()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_else(|| "No error".to_string());

    let c_error = match CString::new(error_msg.clone()) {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let bytes = c_error.as_bytes_with_nul();
    if bytes.len() > buffer_size as usize {
        return -1;
    }

    std::ptr::copy_nonoverlapping(
        bytes.as_ptr(),
        buffer as *mut u8,
        bytes.len(),
    );

    (bytes.len() - 1) as c_int // Don't count null terminator
}

/// Create a default FFI configuration
#[no_mangle]
pub extern "C" fn legacybridge_create_default_config() -> FFIConfig {
    FFIConfig {
        security_level: 1,        // Enhanced
        enable_validation: 1,     // true
        enable_logging: 0,        // false
        enable_pipeline: 1,       // true
        enable_auto_recovery: 1,  // true
        timeout_seconds: 30,      // 30 seconds
    }
}

/// Create a high security FFI configuration
#[no_mangle]
pub extern "C" fn legacybridge_create_high_security_config() -> FFIConfig {
    FFIConfig {
        security_level: 2,        // Paranoid
        enable_validation: 1,     // true
        enable_logging: 1,        // true
        enable_pipeline: 0,       // false (direct conversion for better control)
        enable_auto_recovery: 0,  // false (fail on errors)
        timeout_seconds: 30,      // 30 seconds
    }
}

/// Create a high performance FFI configuration
#[no_mangle]
pub extern "C" fn legacybridge_create_high_performance_config() -> FFIConfig {
    FFIConfig {
        security_level: 0,        // Standard
        enable_validation: 0,     // false
        enable_logging: 0,        // false
        enable_pipeline: 1,       // true
        enable_auto_recovery: 1,  // true
        timeout_seconds: 0,       // no timeout
    }
}

// Backward compatibility wrappers

/// Legacy RTF to Markdown conversion (uses default config)
#[no_mangle]
pub unsafe extern "C" fn legacybridge_rtf_to_markdown(
    rtf_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    legacybridge_rtf_to_markdown_unified(rtf_content, output_buffer, output_length, ptr::null())
}

/// Legacy Markdown to RTF conversion (uses default config)
#[no_mangle]
pub unsafe extern "C" fn legacybridge_markdown_to_rtf(
    markdown_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    legacybridge_markdown_to_rtf_unified(markdown_content, output_buffer, output_length, ptr::null())
}

/// Secure RTF to Markdown conversion (uses high security config)
#[no_mangle]
pub unsafe extern "C" fn legacybridge_rtf_to_markdown_secure(
    rtf_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    let config = legacybridge_create_high_security_config();
    legacybridge_rtf_to_markdown_unified(rtf_content, output_buffer, output_length, &config)
}

/// Secure Markdown to RTF conversion (uses high security config)
#[no_mangle]
pub unsafe extern "C" fn legacybridge_markdown_to_rtf_secure(
    markdown_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    let config = legacybridge_create_high_security_config();
    legacybridge_markdown_to_rtf_unified(markdown_content, output_buffer, output_length, &config)
}