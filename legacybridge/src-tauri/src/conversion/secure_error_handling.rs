// Secure Error Handling Module
//
// This module provides secure error handling patterns to prevent information
// disclosure, ensure graceful degradation, and protect against DoS attacks.

use std::collections::HashMap;
use std::fmt;
use serde::{Serialize, Deserialize};
use tracing::{error, warn, info};

/// Error codes for external interfaces (no internal details)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecureErrorCode {
    Success = 0,
    InvalidInput = 1001,
    ConversionFailed = 1002,
    ResourceLimit = 1003,
    Timeout = 1004,
    AccessDenied = 1005,
    NotSupported = 1006,
    InternalError = 1007,
}

/// Secure error response for external interfaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureError {
    /// Error code for programmatic handling
    pub code: SecureErrorCode,
    /// User-friendly message (no sensitive details)
    pub message: String,
    /// Optional error ID for support/debugging
    pub error_id: Option<String>,
}

/// Internal error details (never exposed externally)
#[derive(Debug)]
struct InternalErrorDetails {
    pub original_error: String,
    pub stack_trace: Option<String>,
    pub context: HashMap<String, String>,
}

/// Error sanitizer to remove sensitive information
pub struct ErrorSanitizer {
    /// Map internal errors to secure external errors
    error_mappings: HashMap<String, SecureErrorCode>,
    /// Generate unique error IDs for tracking
    generate_error_ids: bool,
}

impl Default for ErrorSanitizer {
    fn default() -> Self {
        let mut mappings = HashMap::new();
        
        // Map common internal error patterns to secure codes
        mappings.insert("file not found".to_string(), SecureErrorCode::InvalidInput);
        mappings.insert("permission denied".to_string(), SecureErrorCode::AccessDenied);
        mappings.insert("timeout".to_string(), SecureErrorCode::Timeout);
        mappings.insert("memory".to_string(), SecureErrorCode::ResourceLimit);
        mappings.insert("stack overflow".to_string(), SecureErrorCode::ResourceLimit);
        mappings.insert("recursion limit".to_string(), SecureErrorCode::ResourceLimit);
        mappings.insert("size limit".to_string(), SecureErrorCode::ResourceLimit);
        mappings.insert("invalid format".to_string(), SecureErrorCode::InvalidInput);
        mappings.insert("parse error".to_string(), SecureErrorCode::ConversionFailed);
        mappings.insert("not implemented".to_string(), SecureErrorCode::NotSupported);
        
        Self {
            error_mappings: mappings,
            generate_error_ids: true,
        }
    }
}

impl ErrorSanitizer {
    /// Sanitize an error for external consumption
    pub fn sanitize_error(&self, error: &dyn std::error::Error) -> SecureError {
        let error_str = error.to_string().to_lowercase();
        
        // Determine error code based on error content
        let code = self.determine_error_code(&error_str);
        
        // Generate user-friendly message
        let message = self.generate_user_message(code);
        
        // Generate error ID for tracking
        let error_id = if self.generate_error_ids {
            Some(self.generate_error_id())
        } else {
            None
        };
        
        // Log internal details for debugging (never exposed)
        self.log_internal_error(error, &error_id);
        
        SecureError {
            code,
            message,
            error_id,
        }
    }
    
    /// Determine appropriate error code from error string
    fn determine_error_code(&self, error_str: &str) -> SecureErrorCode {
        // Check error mappings
        for (pattern, code) in &self.error_mappings {
            if error_str.contains(pattern) {
                return *code;
            }
        }
        
        // Default to internal error
        SecureErrorCode::InternalError
    }
    
    /// Generate user-friendly error message
    fn generate_user_message(&self, code: SecureErrorCode) -> String {
        match code {
            SecureErrorCode::Success => "Operation completed successfully".to_string(),
            SecureErrorCode::InvalidInput => "The provided input is invalid or malformed".to_string(),
            SecureErrorCode::ConversionFailed => "Document conversion failed. Please check the input format".to_string(),
            SecureErrorCode::ResourceLimit => "Resource limit exceeded. The document may be too large or complex".to_string(),
            SecureErrorCode::Timeout => "Operation timed out. Please try with a smaller document".to_string(),
            SecureErrorCode::AccessDenied => "Access denied. Please check permissions".to_string(),
            SecureErrorCode::NotSupported => "This feature is not supported".to_string(),
            SecureErrorCode::InternalError => "An internal error occurred. Please contact support if the issue persists".to_string(),
        }
    }
    
    /// Generate unique error ID for tracking
    fn generate_error_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        use rand::Rng;
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let random: u32 = rand::thread_rng().gen();
        
        format!("ERR-{}-{:08X}", timestamp, random)
    }
    
    /// Log internal error details (never exposed to users)
    fn log_internal_error(&self, error: &dyn std::error::Error, error_id: &Option<String>) {
        let error_id_str = error_id.as_ref().map(|id| id.as_str()).unwrap_or("NO-ID");
        
        error!(
            error_id = error_id_str,
            error_type = std::any::type_name_of_val(error),
            error_message = %error,
            "Internal error occurred"
        );
        
        // Log error chain if available
        let mut source = error.source();
        let mut depth = 1;
        while let Some(err) = source {
            error!(
                error_id = error_id_str,
                error_depth = depth,
                error_cause = %err,
                "Error cause"
            );
            source = err.source();
            depth += 1;
        }
    }
}

/// Secure error wrapper for FFI boundaries
pub struct FfiErrorHandler;

impl FfiErrorHandler {
    /// Convert internal error to FFI-safe error code
    pub fn to_ffi_error(error: &dyn std::error::Error) -> i32 {
        let sanitizer = ErrorSanitizer::default();
        let secure_error = sanitizer.sanitize_error(error);
        
        match secure_error.code {
            SecureErrorCode::Success => 0,
            SecureErrorCode::InvalidInput => -1,
            SecureErrorCode::ConversionFailed => -2,
            SecureErrorCode::ResourceLimit => -3,
            SecureErrorCode::Timeout => -4,
            SecureErrorCode::AccessDenied => -5,
            SecureErrorCode::NotSupported => -6,
            SecureErrorCode::InternalError => -99,
        }
    }
    
    /// Get last error message for FFI (sanitized)
    pub fn get_last_error_message() -> &'static str {
        // Return generic message - no internal details
        "Operation failed. Check error code for details"
    }
}

/// Panic handler to prevent crashes
pub struct PanicHandler;

impl PanicHandler {
    /// Install custom panic handler
    pub fn install() {
        std::panic::set_hook(Box::new(|panic_info| {
            // Log panic information securely
            let location = panic_info.location()
                .map(|loc| format!("{}:{}", loc.file(), loc.line()))
                .unwrap_or_else(|| "unknown location".to_string());
            
            let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };
            
            error!(
                panic_location = location,
                panic_message = message,
                "Application panic occurred"
            );
            
            // Don't expose panic details to users
            eprintln!("An internal error occurred. Please contact support.");
        }));
    }
}

/// Result type alias for secure error handling
pub type SecureResult<T> = Result<T, SecureError>;

/// Extension trait for Result types
pub trait SecureResultExt<T> {
    /// Convert to secure result, sanitizing errors
    fn secure(self) -> SecureResult<T>;
}

impl<T, E: std::error::Error> SecureResultExt<T> for Result<T, E> {
    fn secure(self) -> SecureResult<T> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => {
                let sanitizer = ErrorSanitizer::default();
                Err(sanitizer.sanitize_error(&error))
            }
        }
    }
}

/// Secure logging macros that don't expose sensitive data
#[macro_export]
macro_rules! secure_error {
    ($msg:expr) => {
        error!(message = $msg, "Error occurred");
    };
    ($msg:expr, $($key:ident = $value:expr),+) => {
        error!(message = $msg, $($key = $value),+, "Error occurred");
    };
}

#[macro_export]
macro_rules! secure_warn {
    ($msg:expr) => {
        warn!(message = $msg, "Warning");
    };
    ($msg:expr, $($key:ident = $value:expr),+) => {
        warn!(message = $msg, $($key = $value),+, "Warning");
    };
}

/// Error recovery strategies
pub enum RecoveryStrategy {
    /// Return default value
    Default,
    /// Retry with backoff
    Retry { max_attempts: u32, backoff_ms: u64 },
    /// Fallback to alternative implementation
    Fallback,
    /// Graceful degradation
    Degrade,
}

/// Error recovery handler
pub struct ErrorRecoveryHandler;

impl ErrorRecoveryHandler {
    /// Handle error with recovery strategy
    pub fn handle_with_recovery<T, F, E>(
        operation: F,
        strategy: RecoveryStrategy,
        default: T,
    ) -> Result<T, SecureError>
    where
        F: Fn() -> Result<T, E>,
        E: std::error::Error,
    {
        match strategy {
            RecoveryStrategy::Default => {
                match operation() {
                    Ok(value) => Ok(value),
                    Err(error) => {
                        warn!("Operation failed, using default value");
                        let sanitizer = ErrorSanitizer::default();
                        let _ = sanitizer.sanitize_error(&error);
                        Ok(default)
                    }
                }
            }
            RecoveryStrategy::Retry { max_attempts, backoff_ms } => {
                let mut attempts = 0;
                let mut last_error = None;
                
                while attempts < max_attempts {
                    match operation() {
                        Ok(value) => return Ok(value),
                        Err(error) => {
                            last_error = Some(error);
                            attempts += 1;
                            if attempts < max_attempts {
                                std::thread::sleep(std::time::Duration::from_millis(
                                    backoff_ms * attempts as u64
                                ));
                            }
                        }
                    }
                }
                
                let sanitizer = ErrorSanitizer::default();
                if let Some(error) = last_error {
                    Err(sanitizer.sanitize_error(&error))
                } else {
                    Err(SecureError {
                        code: SecureErrorCode::InternalError,
                        message: "Operation failed after retries".to_string(),
                        error_id: None,
                    })
                }
            }
            RecoveryStrategy::Fallback => {
                match operation() {
                    Ok(value) => Ok(value),
                    Err(error) => {
                        warn!("Primary operation failed, attempting fallback");
                        let sanitizer = ErrorSanitizer::default();
                        let _ = sanitizer.sanitize_error(&error);
                        Ok(default)
                    }
                }
            }
            RecoveryStrategy::Degrade => {
                match operation() {
                    Ok(value) => Ok(value),
                    Err(error) => {
                        warn!("Operation failed, degrading gracefully");
                        let sanitizer = ErrorSanitizer::default();
                        let _ = sanitizer.sanitize_error(&error);
                        Ok(default)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Debug)]
    struct TestError(String);
    
    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    
    impl std::error::Error for TestError {}
    
    #[test]
    fn test_error_sanitization() {
        let sanitizer = ErrorSanitizer::default();
        
        // Test file path sanitization
        let error = TestError("Failed to open /etc/passwd: permission denied".to_string());
        let secure_error = sanitizer.sanitize_error(&error);
        assert_eq!(secure_error.code, SecureErrorCode::AccessDenied);
        assert!(!secure_error.message.contains("/etc/passwd"));
        
        // Test memory error
        let error = TestError("Out of memory at address 0x12345678".to_string());
        let secure_error = sanitizer.sanitize_error(&error);
        assert_eq!(secure_error.code, SecureErrorCode::ResourceLimit);
        assert!(!secure_error.message.contains("0x12345678"));
    }
    
    #[test]
    fn test_error_id_generation() {
        let sanitizer = ErrorSanitizer::default();
        let error = TestError("Test error".to_string());
        let secure_error = sanitizer.sanitize_error(&error);
        
        assert!(secure_error.error_id.is_some());
        let error_id = secure_error.error_id.unwrap();
        assert!(error_id.starts_with("ERR-"));
    }
    
    #[test]
    fn test_ffi_error_codes() {
        let error = TestError("Invalid input format".to_string());
        let code = FfiErrorHandler::to_ffi_error(&error);
        assert_eq!(code, -1); // InvalidInput
        
        let error = TestError("Timeout occurred".to_string());
        let code = FfiErrorHandler::to_ffi_error(&error);
        assert_eq!(code, -4); // Timeout
    }
}