// Unified Error Handling Module
//
// This module provides a comprehensive error handling system that works consistently
// across TypeScript, Rust, and C FFI boundaries with proper error context preservation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Unified error structure for cross-language boundary communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Error location in source
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub offset: Option<usize>,
    /// File path if applicable
    pub file_path: Option<String>,
    /// Additional context
    pub context: HashMap<String, String>,
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self {
            line: None,
            column: None,
            offset: None,
            file_path: None,
            context: HashMap::new(),
        }
    }
}

/// Unified error type with rich context for all components
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum LegacyBridgeError {
    /// Parse errors with location information
    ParseError {
        message: String,
        line: u32,
        column: u32,
        expected: Option<String>,
        found: Option<String>,
        file_path: Option<String>,
    },
    /// Conversion errors between formats
    ConversionError {
        source_format: String,
        target_format: String,
        details: String,
        recoverable: bool,
        suggestions: Vec<String>,
    },
    /// IO errors with operation context
    IOError {
        operation: String,
        path: String,
        cause: String,
        error_code: Option<i32>,
    },
    /// Validation errors with field information
    ValidationError {
        field: String,
        expected: String,
        received: String,
        location: Option<ErrorContext>,
    },
    /// System errors from internal components
    SystemError {
        component: String,
        error_code: u32,
        description: String,
        internal_message: Option<String>,
    },
    /// Resource limit errors
    ResourceLimitError {
        resource: String,
        limit: String,
        actual: String,
        suggestion: String,
    },
    /// Feature not implemented
    NotImplementedError {
        feature: String,
        workaround: Option<String>,
        planned_version: Option<String>,
    },
}

impl fmt::Display for LegacyBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError { message, line, column, expected, found, file_path } => {
                write!(f, "Parse error: {}", message)?;
                if let Some(path) = file_path {
                    write!(f, " in {}", path)?;
                }
                write!(f, " at line {} column {}", line, column)?;
                if let Some(exp) = expected {
                    write!(f, " (expected: {})", exp)?;
                }
                if let Some(fnd) = found {
                    write!(f, " (found: {})", fnd)?;
                }
                Ok(())
            }
            Self::ConversionError { source_format, target_format, details, .. } => {
                write!(f, "Conversion error from {} to {}: {}", source_format, target_format, details)
            }
            Self::IOError { operation, path, cause, .. } => {
                write!(f, "IO error during {} on '{}': {}", operation, path, cause)
            }
            Self::ValidationError { field, expected, received, .. } => {
                write!(f, "Validation error for {}: expected {}, received {}", field, expected, received)
            }
            Self::SystemError { component, description, .. } => {
                write!(f, "System error in {}: {}", component, description)
            }
            Self::ResourceLimitError { resource, limit, actual, .. } => {
                write!(f, "Resource limit exceeded for {}: limit is {}, actual is {}", resource, limit, actual)
            }
            Self::NotImplementedError { feature, .. } => {
                write!(f, "Feature not implemented: {}", feature)
            }
        }
    }
}

impl std::error::Error for LegacyBridgeError {}

impl LegacyBridgeError {
    /// Get user-friendly error message (safe for display to end users)
    pub fn user_message(&self) -> String {
        match self {
            Self::ParseError { .. } => "The document format is invalid or corrupted".to_string(),
            Self::ConversionError { source_format, target_format, .. } => {
                format!("Failed to convert from {} to {}", source_format, target_format)
            }
            Self::IOError { operation, .. } => format!("Failed to {} file", operation),
            Self::ValidationError { field, .. } => format!("Invalid {}", field),
            Self::SystemError { .. } => "An internal error occurred".to_string(),
            Self::ResourceLimitError { resource, .. } => format!("{} limit exceeded", resource),
            Self::NotImplementedError { feature, .. } => format!("'{}' is not available yet", feature),
        }
    }

    /// Get detailed error information for developers
    pub fn developer_message(&self) -> String {
        self.to_string()
    }

    /// Get error code for FFI boundary
    pub fn error_code(&self) -> i32 {
        match self {
            Self::ParseError { .. } => -1001,
            Self::ConversionError { .. } => -1002,
            Self::IOError { .. } => -1003,
            Self::ValidationError { .. } => -1004,
            Self::SystemError { .. } => -1005,
            Self::ResourceLimitError { .. } => -1006,
            Self::NotImplementedError { .. } => -1007,
        }
    }

    /// Get error type as string for categorization
    pub fn error_type(&self) -> &'static str {
        match self {
            Self::ParseError { .. } => "ParseError",
            Self::ConversionError { .. } => "ConversionError",
            Self::IOError { .. } => "IOError",
            Self::ValidationError { .. } => "ValidationError",
            Self::SystemError { .. } => "SystemError",
            Self::ResourceLimitError { .. } => "ResourceLimitError",
            Self::NotImplementedError { .. } => "NotImplementedError",
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::ConversionError { recoverable, .. } => *recoverable,
            Self::ParseError { .. } => false,
            Self::IOError { .. } => false,
            Self::ValidationError { .. } => true,
            Self::SystemError { .. } => false,
            Self::ResourceLimitError { .. } => false,
            Self::NotImplementedError { .. } => false,
        }
    }

    /// Get suggestions for fixing the error
    pub fn suggestions(&self) -> Vec<String> {
        match self {
            Self::ConversionError { suggestions, .. } => suggestions.clone(),
            Self::ParseError { .. } => vec![
                "Check if the file is a valid RTF document".to_string(),
                "Ensure the file is not corrupted".to_string(),
            ],
            Self::IOError { .. } => vec![
                "Check file permissions".to_string(),
                "Ensure the file path is correct".to_string(),
                "Verify disk space is available".to_string(),
            ],
            Self::ValidationError { expected, .. } => vec![
                format!("Ensure the value matches the expected format: {}", expected),
            ],
            Self::ResourceLimitError { suggestion, .. } => vec![suggestion.clone()],
            Self::NotImplementedError { workaround, .. } => {
                workaround.as_ref().map(|w| vec![w.clone()]).unwrap_or_default()
            }
            _ => vec![],
        }
    }

    /// Convert to JSON for FFI boundary
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct ErrorResponse {
            error_type: String,
            error_code: i32,
            message: String,
            user_message: String,
            details: serde_json::Value,
            suggestions: Vec<String>,
            recoverable: bool,
            timestamp: String,
        }

        let response = ErrorResponse {
            error_type: self.error_type().to_string(),
            error_code: self.error_code(),
            message: self.developer_message(),
            user_message: self.user_message(),
            details: serde_json::to_value(self)?,
            suggestions: self.suggestions(),
            recoverable: self.is_recoverable(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        serde_json::to_string(&response)
    }
}

/// Convert old ConversionError to new LegacyBridgeError
impl From<super::types::ConversionError> for LegacyBridgeError {
    fn from(err: super::types::ConversionError) -> Self {
        match err {
            super::types::ConversionError::LexerError(msg) => Self::ParseError {
                message: msg,
                line: 0,
                column: 0,
                expected: None,
                found: None,
                file_path: None,
            },
            super::types::ConversionError::ParseError(msg) => Self::ParseError {
                message: msg,
                line: 0,
                column: 0,
                expected: None,
                found: None,
                file_path: None,
            },
            super::types::ConversionError::GenerationError(msg) => Self::ConversionError {
                source_format: "unknown".to_string(),
                target_format: "unknown".to_string(),
                details: msg,
                recoverable: false,
                suggestions: vec![],
            },
            super::types::ConversionError::NotImplemented(msg) => Self::NotImplementedError {
                feature: msg,
                workaround: None,
                planned_version: None,
            },
            super::types::ConversionError::IoError(msg) => Self::IOError {
                operation: "unknown".to_string(),
                path: "unknown".to_string(),
                cause: msg,
                error_code: None,
            },
            super::types::ConversionError::InvalidFormat(msg) => Self::ValidationError {
                field: "format".to_string(),
                expected: "valid format".to_string(),
                received: msg,
                location: None,
            },
            super::types::ConversionError::ValidationError(msg) => Self::ValidationError {
                field: "unknown".to_string(),
                expected: "valid value".to_string(),
                received: msg,
                location: None,
            },
        }
    }
}

/// Result type using unified error
pub type UnifiedResult<T> = Result<T, LegacyBridgeError>;

/// Thread-local storage for last error (for FFI)
thread_local! {
    static LAST_ERROR: std::cell::RefCell<Option<LegacyBridgeError>> = std::cell::RefCell::new(None);
}

/// Set the last error for FFI retrieval
pub fn set_last_error(error: LegacyBridgeError) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(error);
    });
}

/// Get the last error as JSON for FFI
pub fn get_last_error_json() -> Option<String> {
    LAST_ERROR.with(|e| {
        e.borrow().as_ref().and_then(|err| err.to_json().ok())
    })
}

/// Clear the last error
pub fn clear_last_error() {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = None;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_serialization() {
        let error = LegacyBridgeError::ParseError {
            message: "Unexpected token".to_string(),
            line: 42,
            column: 15,
            expected: Some("'}}'".to_string()),
            found: Some("EOF".to_string()),
            file_path: Some("test.rtf".to_string()),
        };

        let json = error.to_json().unwrap();
        assert!(json.contains("ParseError"));
        assert!(json.contains("line"));
        assert!(json.contains("column"));
    }

    #[test]
    fn test_error_suggestions() {
        let error = LegacyBridgeError::ConversionError {
            source_format: "RTF".to_string(),
            target_format: "Markdown".to_string(),
            details: "Invalid table structure".to_string(),
            recoverable: true,
            suggestions: vec![
                "Try simplifying the table structure".to_string(),
                "Remove nested tables".to_string(),
            ],
        };

        let suggestions = error.suggestions();
        assert_eq!(suggestions.len(), 2);
        assert!(error.is_recoverable());
    }

    #[test]
    fn test_user_messages() {
        let error = LegacyBridgeError::SystemError {
            component: "RTF Parser".to_string(),
            error_code: 500,
            description: "Stack overflow in recursive descent parser".to_string(),
            internal_message: Some("Max recursion depth exceeded at node 1000".to_string()),
        };

        let user_msg = error.user_message();
        assert_eq!(user_msg, "An internal error occurred");
        
        let dev_msg = error.developer_message();
        assert!(dev_msg.contains("RTF Parser"));
        assert!(dev_msg.contains("Stack overflow"));
    }
}