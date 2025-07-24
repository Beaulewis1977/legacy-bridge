// Secure implementation of Tauri commands with comprehensive input validation
//
// This file demonstrates how to properly secure all command handlers
// with input validation, path sanitization, and size limits.

use crate::conversion::{self, input_validation::InputValidator};
use crate::pipeline::{PipelineConfig, convert_rtf_to_markdown_with_pipeline, convert_markdown_to_rtf_with_pipeline};
use crate::commands::{
    ConversionResponse, VersionInfo, FileOperationResponse, 
    BatchConversionRequest, BatchConversionResponse,
    PipelineConfigRequest, PipelineConversionResponse,
    ValidationResultDto, RecoveryActionDto
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use base64::{Engine as _, engine::general_purpose};
use lazy_static::lazy_static;

lazy_static! {
    static ref INPUT_VALIDATOR: InputValidator = InputValidator::new();
}

/// Secure version of rtf_to_markdown with input validation
#[tauri::command]
pub fn secure_rtf_to_markdown(rtf_content: String) -> ConversionResponse {
    // Pre-validate input
    if let Err(e) = INPUT_VALIDATOR.pre_validate_rtf(&rtf_content) {
        return ConversionResponse {
            success: false,
            result: None,
            error: Some(format!("Validation error: {}", e)),
        };
    }
    
    // Use secure conversion
    match conversion::secure_rtf_to_markdown(&rtf_content) {
        Ok(markdown) => ConversionResponse {
            success: true,
            result: Some(markdown),
            error: None,
        },
        Err(e) => ConversionResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        },
    }
}

/// Secure version of markdown_to_rtf with input validation
#[tauri::command]
pub fn secure_markdown_to_rtf(markdown_content: String) -> ConversionResponse {
    // Pre-validate input
    if let Err(e) = INPUT_VALIDATOR.pre_validate_markdown(&markdown_content) {
        return ConversionResponse {
            success: false,
            result: None,
            error: Some(format!("Validation error: {}", e)),
        };
    }
    
    // Use secure conversion
    match conversion::secure_markdown_to_rtf(&markdown_content) {
        Ok(rtf) => ConversionResponse {
            success: true,
            result: Some(rtf),
            error: None,
        },
        Err(e) => ConversionResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        },
    }
}

/// Secure file reading with path sanitization and size limits
#[tauri::command]
pub fn secure_read_rtf_file(file_path: String, base_directory: Option<String>) -> FileOperationResponse {
    // Set base directory for path validation
    let base_dir = base_directory
        .as_ref()
        .map(Path::new)
        .or_else(|| std::env::current_dir().ok().as_deref());
    
    // Sanitize and validate file path
    let safe_path = match INPUT_VALIDATOR.sanitize_path(&file_path, base_dir) {
        Ok(p) => p,
        Err(e) => {
            return FileOperationResponse {
                success: false,
                path: Some(file_path),
                content: None,
                error: Some(format!("Invalid path: {}", e)),
            };
        }
    };
    
    // Validate file extension
    if safe_path.extension().and_then(|s| s.to_str()) != Some("rtf") {
        return FileOperationResponse {
            success: false,
            path: Some(file_path),
            content: None,
            error: Some("File must have .rtf extension".to_string()),
        };
    }
    
    // Check file exists
    if !safe_path.exists() {
        return FileOperationResponse {
            success: false,
            path: Some(file_path),
            content: None,
            error: Some("File not found".to_string()),
        };
    }
    
    // Check file size before reading
    match fs::metadata(&safe_path) {
        Ok(metadata) => {
            if metadata.len() > 10 * 1024 * 1024 { // 10MB limit
                return FileOperationResponse {
                    success: false,
                    path: Some(file_path),
                    content: None,
                    error: Some("File size exceeds 10MB limit".to_string()),
                };
            }
        }
        Err(e) => {
            return FileOperationResponse {
                success: false,
                path: Some(file_path),
                content: None,
                error: Some(format!("Failed to read file metadata: {}", e)),
            };
        }
    }
    
    // Read file content
    match fs::read_to_string(&safe_path) {
        Ok(rtf_content) => {
            // Validate RTF content
            if let Err(e) = INPUT_VALIDATOR.pre_validate_rtf(&rtf_content) {
                return FileOperationResponse {
                    success: false,
                    path: Some(file_path),
                    content: None,
                    error: Some(format!("Invalid RTF content: {}", e)),
                };
            }
            
            // Convert RTF to Markdown
            match conversion::secure_rtf_to_markdown(&rtf_content) {
                Ok(markdown) => FileOperationResponse {
                    success: true,
                    path: Some(file_path),
                    content: Some(markdown),
                    error: None,
                },
                Err(e) => FileOperationResponse {
                    success: false,
                    path: Some(file_path),
                    content: None,
                    error: Some(format!("Conversion error: {}", e)),
                },
            }
        }
        Err(e) => FileOperationResponse {
            success: false,
            path: Some(file_path),
            content: None,
            error: Some(format!("Failed to read file: {}", e)),
        },
    }
}

/// Secure file writing with path sanitization
#[tauri::command]
pub fn secure_write_markdown_file(
    file_path: String, 
    content: String,
    base_directory: Option<String>
) -> FileOperationResponse {
    // Validate content first
    if let Err(e) = INPUT_VALIDATOR.validate_size(&content, "File content") {
        return FileOperationResponse {
            success: false,
            path: Some(file_path),
            content: None,
            error: Some(e.to_string()),
        };
    }
    
    // Set base directory for path validation
    let base_dir = base_directory
        .as_ref()
        .map(Path::new)
        .or_else(|| std::env::current_dir().ok().as_deref());
    
    // Sanitize and validate file path
    let safe_path = match INPUT_VALIDATOR.sanitize_path(&file_path, base_dir) {
        Ok(p) => p,
        Err(e) => {
            return FileOperationResponse {
                success: false,
                path: Some(file_path),
                content: None,
                error: Some(format!("Invalid path: {}", e)),
            };
        }
    };
    
    // Ensure parent directory exists
    if let Some(parent) = safe_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return FileOperationResponse {
                success: false,
                path: Some(file_path),
                content: None,
                error: Some(format!("Failed to create directory: {}", e)),
            };
        }
    }
    
    // Write file
    match fs::write(&safe_path, content) {
        Ok(_) => FileOperationResponse {
            success: true,
            path: Some(file_path),
            content: None,
            error: None,
        },
        Err(e) => FileOperationResponse {
            success: false,
            path: Some(file_path),
            content: None,
            error: Some(format!("Failed to write file: {}", e)),
        },
    }
}

/// Secure base64 file operations
#[tauri::command]
pub fn secure_read_file_base64(
    file_path: String,
    base_directory: Option<String>
) -> FileOperationResponse {
    // Set base directory for path validation
    let base_dir = base_directory
        .as_ref()
        .map(Path::new)
        .or_else(|| std::env::current_dir().ok().as_deref());
    
    // Sanitize and validate file path
    let safe_path = match INPUT_VALIDATOR.sanitize_path(&file_path, base_dir) {
        Ok(p) => p,
        Err(e) => {
            return FileOperationResponse {
                success: false,
                path: Some(file_path),
                content: None,
                error: Some(format!("Invalid path: {}", e)),
            };
        }
    };
    
    // Check file size before reading
    match fs::metadata(&safe_path) {
        Ok(metadata) => {
            if metadata.len() > 50 * 1024 * 1024 { // 50MB limit for base64
                return FileOperationResponse {
                    success: false,
                    path: Some(file_path),
                    content: None,
                    error: Some("File size exceeds 50MB limit".to_string()),
                };
            }
        }
        Err(e) => {
            return FileOperationResponse {
                success: false,
                path: Some(file_path),
                content: None,
                error: Some(format!("Failed to read file metadata: {}", e)),
            };
        }
    }
    
    match fs::read(&safe_path) {
        Ok(bytes) => {
            let base64_content = general_purpose::STANDARD.encode(&bytes);
            FileOperationResponse {
                success: true,
                path: Some(file_path),
                content: Some(base64_content),
                error: None,
            }
        }
        Err(e) => FileOperationResponse {
            success: false,
            path: Some(file_path),
            content: None,
            error: Some(format!("Failed to read file: {}", e)),
        },
    }
}

/// Secure batch conversion with rate limiting
#[tauri::command]
pub fn secure_batch_convert_rtf_to_markdown(
    request: BatchConversionRequest,
    base_directory: Option<String>
) -> BatchConversionResponse {
    let mut converted_files = Vec::new();
    let mut failed_files = Vec::new();
    
    // Limit batch size to prevent DoS
    const MAX_BATCH_SIZE: usize = 100;
    if request.input_paths.len() > MAX_BATCH_SIZE {
        return BatchConversionResponse {
            success: false,
            converted_files,
            failed_files,
            error: Some(format!("Batch size exceeds maximum of {} files", MAX_BATCH_SIZE)),
        };
    }
    
    // Set base directory for path validation
    let base_dir = base_directory
        .as_ref()
        .map(Path::new)
        .or_else(|| std::env::current_dir().ok().as_deref());
    
    // Validate output directory
    let output_dir = match INPUT_VALIDATOR.sanitize_path(&request.output_directory, base_dir) {
        Ok(p) => p,
        Err(e) => {
            return BatchConversionResponse {
                success: false,
                converted_files,
                failed_files,
                error: Some(format!("Invalid output directory: {}", e)),
            };
        }
    };
    
    // Ensure output directory exists
    if let Err(e) = fs::create_dir_all(&output_dir) {
        return BatchConversionResponse {
            success: false,
            converted_files,
            failed_files,
            error: Some(format!("Failed to create output directory: {}", e)),
        };
    }
    
    for input_path in request.input_paths {
        // Validate input path
        let safe_input_path = match INPUT_VALIDATOR.sanitize_path(&input_path, base_dir) {
            Ok(p) => p,
            Err(e) => {
                failed_files.push((input_path, format!("Invalid path: {}", e)));
                continue;
            }
        };
        
        // Read and validate RTF file
        match fs::read_to_string(&safe_input_path) {
            Ok(rtf_content) => {
                // Validate content
                if let Err(e) = INPUT_VALIDATOR.pre_validate_rtf(&rtf_content) {
                    failed_files.push((
                        input_path.clone(),
                        format!("Invalid RTF: {}", e)
                    ));
                    continue;
                }
                
                // Convert to Markdown
                match conversion::secure_rtf_to_markdown(&rtf_content) {
                    Ok(markdown) => {
                        // Generate output filename
                        let file_stem = safe_input_path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("output");
                        let output_filename = format!("{}.md", file_stem);
                        let output_path = output_dir.join(&output_filename);
                        
                        // Write Markdown file
                        match fs::write(&output_path, markdown) {
                            Ok(_) => {
                                converted_files.push(output_path.to_string_lossy().to_string());
                            }
                            Err(e) => {
                                failed_files.push((
                                    input_path.clone(),
                                    format!("Failed to write output: {}", e)
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        failed_files.push((
                            input_path.clone(),
                            format!("Conversion error: {}", e)
                        ));
                    }
                }
            }
            Err(e) => {
                failed_files.push((
                    input_path.clone(),
                    format!("Failed to read file: {}", e)
                ));
            }
        }
    }
    
    BatchConversionResponse {
        success: failed_files.is_empty(),
        converted_files,
        failed_files,
        error: None,
    }
}

/// Secure pipeline conversion with validation
#[tauri::command]
pub fn secure_rtf_to_markdown_pipeline(
    rtf_content: String,
    config: Option<PipelineConfigRequest>,
) -> PipelineConversionResponse {
    // Pre-validate input
    if let Err(e) = INPUT_VALIDATOR.pre_validate_rtf(&rtf_content) {
        return PipelineConversionResponse {
            success: false,
            markdown: None,
            validation_results: vec![ValidationResultDto {
                level: "Error".to_string(),
                code: "VALIDATION_FAILED".to_string(),
                message: e.to_string(),
                location: None,
            }],
            recovery_actions: Vec::new(),
            error: Some(format!("Validation error: {}", e)),
        };
    }
    
    // Build pipeline configuration with secure defaults
    let pipeline_config = if let Some(cfg) = config {
        PipelineConfig {
            strict_validation: cfg.strict_validation.unwrap_or(true),
            auto_recovery: cfg.auto_recovery.unwrap_or(true),
            template: cfg.template,
            preserve_formatting: cfg.preserve_formatting.unwrap_or(true),
            legacy_mode: cfg.legacy_mode.unwrap_or(false),
        }
    } else {
        PipelineConfig {
            strict_validation: true,
            auto_recovery: true,
            template: None,
            preserve_formatting: true,
            legacy_mode: false,
        }
    };

    // Run conversion through pipeline
    match convert_rtf_to_markdown_with_pipeline(&rtf_content, Some(pipeline_config)) {
        Ok((markdown, context)) => {
            // Convert validation results to DTOs
            let validation_results = context.validation_results.iter()
                .map(|v| ValidationResultDto {
                    level: format!("{:?}", v.level),
                    code: v.code.clone(),
                    message: v.message.clone(),
                    location: v.location.clone(),
                })
                .collect();

            // Convert recovery actions to DTOs
            let recovery_actions = context.recovery_actions.iter()
                .map(|a| RecoveryActionDto {
                    action_type: format!("{:?}", a.action_type),
                    description: a.description.clone(),
                    applied: a.applied,
                })
                .collect();

            PipelineConversionResponse {
                success: true,
                markdown: Some(markdown),
                validation_results,
                recovery_actions,
                error: None,
            }
        }
        Err(e) => PipelineConversionResponse {
            success: false,
            markdown: None,
            validation_results: Vec::new(),
            recovery_actions: Vec::new(),
            error: Some(e.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_rtf_conversion() {
        let rtf = r"{\rtf1 Hello World\par}";
        let response = secure_rtf_to_markdown(rtf.to_string());
        assert!(response.success);
        assert!(response.result.is_some());
    }

    #[test]
    fn test_blocks_malicious_rtf() {
        let malicious = r"{\rtf1 \object\objdata Evil}";
        let response = secure_rtf_to_markdown(malicious.to_string());
        assert!(!response.success);
        assert!(response.error.unwrap().contains("forbidden"));
    }

    #[test]
    fn test_size_limit_enforcement() {
        let huge_rtf = format!(r"{{\rtf1 {}}}", "A".repeat(11 * 1024 * 1024));
        let response = secure_rtf_to_markdown(huge_rtf);
        assert!(!response.success);
        assert!(response.error.unwrap().contains("size"));
    }

    #[test]
    fn test_path_sanitization() {
        let response = secure_read_rtf_file(
            "../../../etc/passwd".to_string(),
            Some("/safe/dir".to_string())
        );
        assert!(!response.success);
        assert!(response.error.unwrap().contains("path"));
    }
}