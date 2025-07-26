// Tauri commands for RTF/Markdown conversion

use crate::conversion;
use crate::pipeline::{PipelineConfig, convert_rtf_to_markdown_with_pipeline, convert_markdown_to_rtf_with_pipeline};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use base64::{Engine as _, engine::general_purpose};

/// Response structure for conversion operations
#[derive(Debug, Serialize, Deserialize)]
pub struct ConversionResponse {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
}

/// Version information
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub app_version: String,
    pub rtf_parser_version: String,
    pub supported_features: Vec<String>,
}

/// File operation response
#[derive(Debug, Serialize, Deserialize)]
pub struct FileOperationResponse {
    pub success: bool,
    pub path: Option<String>,
    pub content: Option<String>,
    pub error: Option<String>,
}

/// Batch conversion request
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchConversionRequest {
    pub input_paths: Vec<String>,
    pub output_directory: String,
}

/// Batch conversion response
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchConversionResponse {
    pub success: bool,
    pub converted_files: Vec<String>,
    pub failed_files: Vec<(String, String)>, // (path, error)
    pub error: Option<String>,
}

/// Convert RTF content to Markdown
#[tauri::command]
pub fn rtf_to_markdown(rtf_content: String) -> ConversionResponse {
    // SECURITY: Use secure conversion by default
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

/// Convert Markdown content to RTF
#[tauri::command]
pub fn markdown_to_rtf(markdown_content: String) -> ConversionResponse {
    // SECURITY: Use secure conversion by default
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

/// Test connection between frontend and backend
#[tauri::command]
pub fn test_connection() -> String {
    "Connection successful! Backend is ready.".to_string()
}

/// Get version information
#[tauri::command]
pub fn get_version_info() -> VersionInfo {
    VersionInfo {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        rtf_parser_version: "0.1.0".to_string(),
        supported_features: vec![
            "RTF to Markdown".to_string(),
            "Basic text formatting (bold, italic, underline)".to_string(),
            "Paragraphs".to_string(),
            "Line breaks".to_string(),
            "Page breaks".to_string(),
            "File reading and writing".to_string(),
            "Batch conversion".to_string(),
        ],
    }
}

/// Read an RTF file from disk and convert it to Markdown
#[tauri::command]
pub fn read_rtf_file(file_path: String) -> FileOperationResponse {
    let path = Path::new(&file_path);
    
    // Validate file extension
    if path.extension().and_then(|s| s.to_str()) != Some("rtf") {
        return FileOperationResponse {
            success: false,
            path: Some(file_path),
            content: None,
            error: Some("File must have .rtf extension".to_string()),
        };
    }
    
    // SECURITY: Check file size before reading
    match fs::metadata(&path) {
        Ok(metadata) => {
            // Check file size (10MB limit)
            if metadata.len() > 10 * 1024 * 1024 {
                return FileOperationResponse {
                    success: false,
                    path: Some(file_path),
                    content: None,
                    error: Some("File size exceeds 10MB limit".to_string()),
                };
            }
            
            // Read file content
            match fs::read_to_string(&path) {
                Ok(rtf_content) => {
                    // SECURITY: Use secure conversion
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
        Err(e) => FileOperationResponse {
            success: false,
            path: Some(file_path),
            content: None,
            error: Some(format!("Failed to check file size: {}", e)),
        },
    }
}

/// Write Markdown content to a file
#[tauri::command]
pub fn write_markdown_file(file_path: String, content: String) -> FileOperationResponse {
    // SECURITY: Validate content size (10MB limit)
    if content.len() > 10 * 1024 * 1024 {
        return FileOperationResponse {
            success: false,
            path: Some(file_path),
            content: None,
            error: Some("Content size exceeds 10MB limit".to_string()),
        };
    }
    
    let path = Path::new(&file_path);
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
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
    match fs::write(&path, content) {
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

/// Read file as base64 (useful for binary files or frontend transfer)
#[tauri::command]
pub fn read_file_base64(file_path: String) -> FileOperationResponse {
    // SECURITY: Check file size before reading
    let path = Path::new(&file_path);
    match fs::metadata(&path) {
        Ok(metadata) => {
            // Check file size (10MB limit)
            if metadata.len() > 10 * 1024 * 1024 {
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
                error: Some(format!("Failed to check file size: {}", e)),
            };
        }
    }
    
    match fs::read(&file_path) {
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

/// Write base64 content to file
#[tauri::command]
pub fn write_file_base64(file_path: String, base64_content: String) -> FileOperationResponse {
    // SECURITY: Validate base64 content size (10MB limit after decoding)
    // Base64 increases size by ~33%, so check if encoded size is reasonable
    if base64_content.len() > 14 * 1024 * 1024 {
        return FileOperationResponse {
            success: false,
            path: Some(file_path),
            content: None,
            error: Some("Base64 content too large".to_string()),
        };
    }
    
    match general_purpose::STANDARD.decode(&base64_content) {
        Ok(bytes) => {
            match fs::write(&file_path, bytes) {
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
        Err(e) => FileOperationResponse {
            success: false,
            path: Some(file_path),
            content: None,
            error: Some(format!("Invalid base64 content: {}", e)),
        },
    }
}

/// Batch convert RTF files to Markdown
#[tauri::command]
pub fn batch_convert_rtf_to_markdown(request: BatchConversionRequest) -> BatchConversionResponse {
    let mut converted_files = Vec::new();
    let mut failed_files = Vec::new();
    
    // Ensure output directory exists
    if let Err(e) = fs::create_dir_all(&request.output_directory) {
        return BatchConversionResponse {
            success: false,
            converted_files,
            failed_files,
            error: Some(format!("Failed to create output directory: {}", e)),
        };
    }
    
    for input_path in request.input_paths {
        let path = Path::new(&input_path);
        
        // Read RTF file
        match fs::read_to_string(&path) {
            Ok(rtf_content) => {
                // SECURITY: Use secure conversion
                match conversion::secure_rtf_to_markdown(&rtf_content) {
                    Ok(markdown) => {
                        // Generate output filename
                        let file_stem = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("output");
                        let output_filename = format!("{}.md", file_stem);
                        let output_path = PathBuf::from(&request.output_directory)
                            .join(&output_filename);
                        
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

/// Pipeline configuration for advanced conversion
#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineConfigRequest {
    pub strict_validation: Option<bool>,
    pub auto_recovery: Option<bool>,
    pub template: Option<String>,
    pub preserve_formatting: Option<bool>,
    pub legacy_mode: Option<bool>,
}

/// Pipeline conversion response with detailed context
#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineConversionResponse {
    pub success: bool,
    pub markdown: Option<String>,
    pub validation_results: Vec<ValidationResultDto>,
    pub recovery_actions: Vec<RecoveryActionDto>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResultDto {
    pub level: String,
    pub code: String,
    pub message: String,
    pub location: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecoveryActionDto {
    pub action_type: String,
    pub description: String,
    pub applied: bool,
}

/// Convert RTF to Markdown using the advanced pipeline
#[tauri::command]
pub fn rtf_to_markdown_pipeline(
    rtf_content: String,
    config: Option<PipelineConfigRequest>,
) -> PipelineConversionResponse {
    // Build pipeline configuration
    let pipeline_config = if let Some(cfg) = config {
        PipelineConfig {
            strict_validation: cfg.strict_validation.unwrap_or(true),
            auto_recovery: cfg.auto_recovery.unwrap_or(true),
            template: cfg.template,
            preserve_formatting: cfg.preserve_formatting.unwrap_or(true),
            legacy_mode: cfg.legacy_mode.unwrap_or(false),
        }
    } else {
        PipelineConfig::default()
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

/// Read RTF file and convert using pipeline
#[tauri::command]
pub fn read_rtf_file_pipeline(
    file_path: String,
    config: Option<PipelineConfigRequest>,
) -> PipelineConversionResponse {
    let path = Path::new(&file_path);
    
    // Validate file extension
    if path.extension().and_then(|s| s.to_str()) != Some("rtf") {
        return PipelineConversionResponse {
            success: false,
            markdown: None,
            validation_results: Vec::new(),
            recovery_actions: Vec::new(),
            error: Some("File must have .rtf extension".to_string()),
        };
    }
    
    // SECURITY: Check file size before reading
    match fs::metadata(&path) {
        Ok(metadata) => {
            // Check file size (10MB limit)
            if metadata.len() > 10 * 1024 * 1024 {
                return PipelineConversionResponse {
                    success: false,
                    markdown: None,
                    validation_results: Vec::new(),
                    recovery_actions: Vec::new(),
                    error: Some("File size exceeds 10MB limit".to_string()),
                };
            }
            
            // Read file content
            match fs::read_to_string(&path) {
                Ok(rtf_content) => {
                    // Convert using pipeline
                    rtf_to_markdown_pipeline(rtf_content, config)
                }
                Err(e) => PipelineConversionResponse {
                    success: false,
                    markdown: None,
                    validation_results: Vec::new(),
                    recovery_actions: Vec::new(),
                    error: Some(format!("Failed to read file: {}", e)),
                },
            }
        }
        Err(e) => PipelineConversionResponse {
            success: false,
            markdown: None,
            validation_results: Vec::new(),
            recovery_actions: Vec::new(),
            error: Some(format!("Failed to read file: {}", e)),
        },
    }
}

/// Convert Markdown to RTF using the advanced pipeline
#[tauri::command]
pub fn markdown_to_rtf_pipeline(
    markdown_content: String,
    config: Option<PipelineConfigRequest>,
) -> PipelineConversionResponse {
    // Build pipeline configuration
    let pipeline_config = if let Some(cfg) = config {
        PipelineConfig {
            strict_validation: cfg.strict_validation.unwrap_or(true),
            auto_recovery: cfg.auto_recovery.unwrap_or(true),
            template: cfg.template,
            preserve_formatting: cfg.preserve_formatting.unwrap_or(true),
            legacy_mode: cfg.legacy_mode.unwrap_or(false),
        }
    } else {
        PipelineConfig::default()
    };

    // Run conversion through pipeline
    match convert_markdown_to_rtf_with_pipeline(&markdown_content, Some(pipeline_config)) {
        Ok((rtf, context)) => {
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
                markdown: Some(rtf), // Using markdown field for RTF output for compatibility
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

/// Read markdown file and convert to RTF using pipeline
#[tauri::command]
pub fn read_markdown_file_pipeline(
    file_path: String,
    config: Option<PipelineConfigRequest>,
) -> PipelineConversionResponse {
    let path = Path::new(&file_path);
    
    // Validate file extension
    let extension = path.extension().and_then(|s| s.to_str());
    if !matches!(extension, Some("md") | Some("markdown")) {
        return PipelineConversionResponse {
            success: false,
            markdown: None,
            validation_results: Vec::new(),
            recovery_actions: Vec::new(),
            error: Some("File must have .md or .markdown extension".to_string()),
        };
    }
    
    // Read file content
    match fs::read_to_string(&path) {
        Ok(markdown_content) => {
            // Convert using pipeline
            markdown_to_rtf_pipeline(markdown_content, config)
        }
        Err(e) => PipelineConversionResponse {
            success: false,
            markdown: None,
            validation_results: Vec::new(),
            recovery_actions: Vec::new(),
            error: Some(format!("Failed to read file: {}", e)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtf_to_markdown_command() {
        let rtf = r"{\rtf1 Hello World\par}";
        let response = rtf_to_markdown(rtf.to_string());
        
        assert!(response.success);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_markdown_to_rtf_command() {
        let markdown = "# Hello World";
        let response = markdown_to_rtf(markdown.to_string());
        
        // Now implemented, should succeed
        assert!(response.success);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        
        // Verify RTF structure
        let rtf = response.result.unwrap();
        assert!(rtf.contains("{\\rtf1\\ansi"));
        assert!(rtf.contains("Hello World"));
    }

    #[test]
    fn test_markdown_to_rtf_pipeline_command() {
        let markdown = "**Bold** and *italic* text";
        let response = markdown_to_rtf_pipeline(markdown.to_string(), None);
        
        assert!(response.success);
        assert!(response.markdown.is_some()); // RTF stored in markdown field for compatibility
        assert!(response.error.is_none());
        
        // Verify RTF structure
        let rtf = response.markdown.unwrap();
        assert!(rtf.contains("{\\rtf1\\ansi"));
        assert!(rtf.contains("Bold"));
        assert!(rtf.contains("italic"));
    }

    #[test]
    fn test_connection_command() {
        let result = test_connection();
        assert_eq!(result, "Connection successful! Backend is ready.");
    }

    #[test]
    fn test_version_info() {
        let info = get_version_info();
        assert!(!info.app_version.is_empty());
        assert!(!info.rtf_parser_version.is_empty());
        assert!(!info.supported_features.is_empty());
    }
}