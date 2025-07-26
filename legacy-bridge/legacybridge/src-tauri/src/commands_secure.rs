// Secure Tauri commands for RTF/Markdown conversion
// This module provides secure error handling for all command interfaces

use crate::conversion;
use crate::conversion::secure_error_handling::{
    SecureError, SecureErrorCode, ErrorSanitizer, SecureResultExt
};
use crate::pipeline::{PipelineConfig, convert_rtf_to_markdown_with_pipeline, convert_markdown_to_rtf_with_pipeline};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use base64::{Engine as _, engine::general_purpose};
use tracing::{error, warn, info};

/// Secure response structure for conversion operations
#[derive(Debug, Serialize, Deserialize)]
pub struct SecureConversionResponse {
    pub success: bool,
    pub result: Option<String>,
    pub error_code: Option<i32>,
    pub error_message: Option<String>,
    pub error_id: Option<String>,
}

impl From<SecureError> for SecureConversionResponse {
    fn from(error: SecureError) -> Self {
        SecureConversionResponse {
            success: false,
            result: None,
            error_code: Some(error.code as i32),
            error_message: Some(error.message),
            error_id: error.error_id,
        }
    }
}

/// Secure file operation response
#[derive(Debug, Serialize, Deserialize)]
pub struct SecureFileOperationResponse {
    pub success: bool,
    pub content: Option<String>,
    pub error_code: Option<i32>,
    pub error_message: Option<String>,
    pub error_id: Option<String>,
}

impl From<SecureError> for SecureFileOperationResponse {
    fn from(error: SecureError) -> Self {
        SecureFileOperationResponse {
            success: false,
            content: None,
            error_code: Some(error.code as i32),
            error_message: Some(error.message),
            error_id: error.error_id,
        }
    }
}

/// Convert RTF content to Markdown with secure error handling
#[tauri::command]
pub fn rtf_to_markdown_secure(rtf_content: String) -> SecureConversionResponse {
    // Validate input size
    if rtf_content.len() > 10 * 1024 * 1024 { // 10MB limit
        return SecureError {
            code: SecureErrorCode::ResourceLimit,
            message: "Document too large. Maximum size is 10MB".to_string(),
            error_id: None,
        }.into();
    }

    match conversion::rtf_to_markdown(&rtf_content) {
        Ok(markdown) => SecureConversionResponse {
            success: true,
            result: Some(markdown),
            error_code: None,
            error_message: None,
            error_id: None,
        },
        Err(internal_error) => {
            let error_id = generate_error_id();
            error!(
                error_id = %error_id,
                command = "rtf_to_markdown",
                "Conversion failed"
            );
            
            SecureError {
                code: SecureErrorCode::ConversionFailed,
                message: "Document conversion failed. Please check the input format".to_string(),
                error_id: Some(error_id),
            }.into()
        }
    }
}

/// Convert Markdown content to RTF with secure error handling
#[tauri::command]
pub fn markdown_to_rtf_secure(markdown_content: String) -> SecureConversionResponse {
    // Validate input size
    if markdown_content.len() > 10 * 1024 * 1024 { // 10MB limit
        return SecureError {
            code: SecureErrorCode::ResourceLimit,
            message: "Document too large. Maximum size is 10MB".to_string(),
            error_id: None,
        }.into();
    }

    match conversion::markdown_to_rtf(&markdown_content) {
        Ok(rtf) => SecureConversionResponse {
            success: true,
            result: Some(rtf),
            error_code: None,
            error_message: None,
            error_id: None,
        },
        Err(internal_error) => {
            let error_id = generate_error_id();
            error!(
                error_id = %error_id,
                command = "markdown_to_rtf",
                "Conversion failed"
            );
            
            SecureError {
                code: SecureErrorCode::ConversionFailed,
                message: "Document conversion failed. Please check the input format".to_string(),
                error_id: Some(error_id),
            }.into()
        }
    }
}

/// Read a file with secure error handling
#[tauri::command]
pub fn read_file_secure(file_path: String) -> SecureFileOperationResponse {
    // Validate file extension
    let path = Path::new(&file_path);
    let extension = path.extension().and_then(|s| s.to_str());
    
    let allowed_extensions = ["rtf", "md", "markdown", "txt"];
    if !extension.map(|ext| allowed_extensions.contains(&ext)).unwrap_or(false) {
        return SecureError {
            code: SecureErrorCode::InvalidInput,
            message: "Unsupported file type".to_string(),
            error_id: None,
        }.into();
    }
    
    // Check file size before reading
    match fs::metadata(&path) {
        Ok(metadata) => {
            if metadata.len() > 10 * 1024 * 1024 { // 10MB limit
                return SecureError {
                    code: SecureErrorCode::ResourceLimit,
                    message: "File too large. Maximum size is 10MB".to_string(),
                    error_id: None,
                }.into();
            }
        }
        Err(_) => {
            return SecureError {
                code: SecureErrorCode::InvalidInput,
                message: "Cannot access file".to_string(),
                error_id: None,
            }.into();
        }
    }
    
    // Read file content
    match fs::read_to_string(&path) {
        Ok(content) => SecureFileOperationResponse {
            success: true,
            content: Some(content),
            error_code: None,
            error_message: None,
            error_id: None,
        },
        Err(io_error) => {
            let error_id = generate_error_id();
            error!(
                error_id = %error_id,
                command = "read_file",
                file_extension = ?extension,
                "File read failed"
            );
            
            // Determine appropriate error based on IO error kind
            let secure_error = match io_error.kind() {
                std::io::ErrorKind::NotFound => SecureError {
                    code: SecureErrorCode::InvalidInput,
                    message: "File not found".to_string(),
                    error_id: Some(error_id),
                },
                std::io::ErrorKind::PermissionDenied => SecureError {
                    code: SecureErrorCode::AccessDenied,
                    message: "Access denied".to_string(),
                    error_id: Some(error_id),
                },
                _ => SecureError {
                    code: SecureErrorCode::InternalError,
                    message: "Cannot read file".to_string(),
                    error_id: Some(error_id),
                },
            };
            
            secure_error.into()
        }
    }
}

/// Write file with secure error handling
#[tauri::command]
pub fn write_file_secure(file_path: String, content: String) -> SecureFileOperationResponse {
    // Validate file extension
    let path = Path::new(&file_path);
    let extension = path.extension().and_then(|s| s.to_str());
    
    let allowed_extensions = ["rtf", "md", "markdown", "txt"];
    if !extension.map(|ext| allowed_extensions.contains(&ext)).unwrap_or(false) {
        return SecureError {
            code: SecureErrorCode::InvalidInput,
            message: "Unsupported file type".to_string(),
            error_id: None,
        }.into();
    }
    
    // Validate content size
    if content.len() > 10 * 1024 * 1024 { // 10MB limit
        return SecureError {
            code: SecureErrorCode::ResourceLimit,
            message: "Content too large. Maximum size is 10MB".to_string(),
            error_id: None,
        }.into();
    }
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if let Err(_) = fs::create_dir_all(parent) {
            return SecureError {
                code: SecureErrorCode::InternalError,
                message: "Cannot create output directory".to_string(),
                error_id: None,
            }.into();
        }
    }
    
    // Write file
    match fs::write(&path, content) {
        Ok(_) => SecureFileOperationResponse {
            success: true,
            content: None,
            error_code: None,
            error_message: None,
            error_id: None,
        },
        Err(io_error) => {
            let error_id = generate_error_id();
            error!(
                error_id = %error_id,
                command = "write_file",
                file_extension = ?extension,
                "File write failed"
            );
            
            // Determine appropriate error based on IO error kind
            let secure_error = match io_error.kind() {
                std::io::ErrorKind::PermissionDenied => SecureError {
                    code: SecureErrorCode::AccessDenied,
                    message: "Access denied".to_string(),
                    error_id: Some(error_id),
                },
                std::io::ErrorKind::OutOfMemory => SecureError {
                    code: SecureErrorCode::ResourceLimit,
                    message: "Insufficient memory".to_string(),
                    error_id: Some(error_id),
                },
                _ => SecureError {
                    code: SecureErrorCode::InternalError,
                    message: "Cannot write file".to_string(),
                    error_id: Some(error_id),
                },
            };
            
            secure_error.into()
        }
    }
}

/// Batch conversion response with secure error handling
#[derive(Debug, Serialize, Deserialize)]
pub struct SecureBatchConversionResponse {
    pub success: bool,
    pub converted_count: usize,
    pub failed_count: usize,
    pub error_ids: Vec<String>,
}

/// Batch convert files with secure error handling
#[tauri::command]
pub fn batch_convert_secure(
    input_paths: Vec<String>,
    output_directory: String,
    is_rtf_to_md: bool,
) -> SecureBatchConversionResponse {
    let mut converted_count = 0;
    let mut failed_count = 0;
    let mut error_ids = Vec::new();
    
    // Validate output directory
    if let Err(_) = fs::create_dir_all(&output_directory) {
        return SecureBatchConversionResponse {
            success: false,
            converted_count: 0,
            failed_count: input_paths.len(),
            error_ids: vec![generate_error_id()],
        };
    }
    
    for input_path in input_paths {
        let path = Path::new(&input_path);
        
        // Read file
        match fs::read_to_string(&path) {
            Ok(content) => {
                // Check size
                if content.len() > 10 * 1024 * 1024 {
                    failed_count += 1;
                    error_ids.push(generate_error_id());
                    continue;
                }
                
                // Convert
                let result = if is_rtf_to_md {
                    conversion::rtf_to_markdown(&content)
                } else {
                    conversion::markdown_to_rtf(&content)
                };
                
                match result {
                    Ok(converted) => {
                        // Generate output filename
                        let file_stem = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("output");
                        let extension = if is_rtf_to_md { "md" } else { "rtf" };
                        let output_filename = format!("{}.{}", file_stem, extension);
                        let output_path = PathBuf::from(&output_directory)
                            .join(&output_filename);
                        
                        // Write output
                        if fs::write(&output_path, converted).is_ok() {
                            converted_count += 1;
                        } else {
                            failed_count += 1;
                            error_ids.push(generate_error_id());
                        }
                    }
                    Err(_) => {
                        failed_count += 1;
                        error_ids.push(generate_error_id());
                    }
                }
            }
            Err(_) => {
                failed_count += 1;
                error_ids.push(generate_error_id());
            }
        }
    }
    
    SecureBatchConversionResponse {
        success: failed_count == 0,
        converted_count,
        failed_count,
        error_ids,
    }
}

/// Test connection with secure response
#[tauri::command]
pub fn test_connection_secure() -> String {
    "Connection successful".to_string()
}

/// Get version information securely
#[tauri::command]
pub fn get_version_info_secure() -> serde_json::Value {
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "api_version": "1.0"
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_rtf_conversion() {
        let rtf = r"{\rtf1 Hello World\par}";
        let response = rtf_to_markdown_secure(rtf.to_string());
        
        assert!(response.success);
        assert!(response.result.is_some());
        assert!(response.error_code.is_none());
    }

    #[test]
    fn test_large_input_rejection() {
        let large_content = "x".repeat(11 * 1024 * 1024); // 11MB
        let response = rtf_to_markdown_secure(large_content);
        
        assert!(!response.success);
        assert_eq!(response.error_code, Some(SecureErrorCode::ResourceLimit as i32));
        assert!(response.error_message.is_some());
        assert!(!response.error_message.unwrap().contains("11"));
    }

    #[test]
    fn test_error_id_generation() {
        let id1 = generate_error_id();
        let id2 = generate_error_id();
        
        assert!(id1.starts_with("ERR-"));
        assert!(id2.starts_with("ERR-"));
        assert_ne!(id1, id2);
    }
}