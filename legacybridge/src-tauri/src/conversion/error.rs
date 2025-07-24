// Comprehensive error handling for the conversion module
// This module provides robust error types to replace all unwrap() calls

use thiserror::Error;
use std::fmt;

/// Main conversion error type with comprehensive error variants
#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parser error: {0}")]
    Parser(String),
    
    #[error("Lexer error: {0}")]
    Lexer(String),
    
    #[error("Generator error: {0}")]
    Generator(String),
    
    #[error("Memory limit exceeded: current {current} bytes > limit {limit} bytes")]
    MemoryLimit { current: usize, limit: usize },
    
    #[error("Invalid input format: {0}")]
    InvalidFormat(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    
    #[error("UTF-8 string conversion error: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    
    #[error("Parse int error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    
    #[error("Timeout exceeded: operation took longer than {0} seconds")]
    Timeout(u64),
    
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
    
    #[error("Invalid control word: {0}")]
    InvalidControlWord(String),
    
    #[error("Unexpected end of input")]
    UnexpectedEof,
    
    #[error("Group nesting error: {0}")]
    GroupNesting(String),
    
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),
    
    #[error("Security violation: {0}")]
    Security(String),
    
    #[error("Invalid hex value: {0}")]
    InvalidHex(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("FFI error: {0}")]
    Ffi(String),
}

/// System-level errors that can occur during processing
#[derive(Error, Debug)]
pub enum SystemError {
    #[error("Insufficient memory: required {required} bytes, available {available} bytes")]
    InsufficientMemory { required: usize, available: usize },
    
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Thread pool error: {0}")]
    ThreadPool(String),
}

/// Validation errors for input validation
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Size limit exceeded: {actual} > {max}")]
    SizeLimit { actual: usize, max: usize },
    
    #[error("Format mismatch: expected {expected}, got {actual}")]
    FormatMismatch { expected: String, actual: String },
    
    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String),
    
    #[error("Malformed structure: {0}")]
    MalformedStructure(String),
}

/// Processing errors that occur during conversion
#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Parsing failed: {0}")]
    ParsingFailed(String),
    
    #[error("Conversion error: {0}")]
    ConversionError(String),
    
    #[error("Timeout exceeded after {0} seconds")]
    TimeoutExceeded(u64),
    
    #[error("Resource limit reached: {0}")]
    ResourceLimit(String),
}

/// Result type aliases for convenience
pub type ConversionResult<T> = Result<T, ConversionError>;
pub type SystemResult<T> = Result<T, SystemError>;
pub type ValidationResult<T> = Result<T, ValidationError>;
pub type ProcessingResult<T> = Result<T, ProcessingError>;

/// Extension trait for adding context to errors
pub trait ErrorContext<T> {
    fn context(self, msg: &str) -> Result<T, ConversionError>;
    fn with_context<F>(self, f: F) -> Result<T, ConversionError>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<ConversionError>,
{
    fn context(self, msg: &str) -> Result<T, ConversionError> {
        self.map_err(|e| {
            let base_error = e.into();
            ConversionError::Parser(format!("{}: {}", msg, base_error))
        })
    }
    
    fn with_context<F>(self, f: F) -> Result<T, ConversionError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            ConversionError::Parser(format!("{}: {}", f(), base_error))
        })
    }
}

/// Helper functions for common error patterns
impl ConversionError {
    /// Create a parser error with context
    pub fn parser<S: Into<String>>(msg: S) -> Self {
        ConversionError::Parser(msg.into())
    }
    
    /// Create a lexer error with context
    pub fn lexer<S: Into<String>>(msg: S) -> Self {
        ConversionError::Lexer(msg.into())
    }
    
    /// Create a generator error with context
    pub fn generator<S: Into<String>>(msg: S) -> Self {
        ConversionError::Generator(msg.into())
    }
    
    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        ConversionError::Validation(msg.into())
    }
    
    /// Create an invalid format error
    pub fn invalid_format<S: Into<String>>(msg: S) -> Self {
        ConversionError::InvalidFormat(msg.into())
    }
}

/// Conversion from legacy error types to new error types
impl From<ConversionError> for crate::conversion::types::ConversionError {
    fn from(err: ConversionError) -> Self {
        match err {
            ConversionError::Parser(msg) => crate::conversion::types::ConversionError::ParseError(msg),
            ConversionError::Lexer(msg) => crate::conversion::types::ConversionError::LexerError(msg),
            ConversionError::Generator(msg) => crate::conversion::types::ConversionError::GenerationError(msg),
            ConversionError::InvalidFormat(msg) => crate::conversion::types::ConversionError::InvalidFormat(msg),
            ConversionError::Validation(msg) => crate::conversion::types::ConversionError::ValidationError(msg),
            ConversionError::NotImplemented(msg) => crate::conversion::types::ConversionError::NotImplemented(msg),
            ConversionError::Io(e) => crate::conversion::types::ConversionError::IoError(e.to_string()),
            _ => crate::conversion::types::ConversionError::ParseError(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_context() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found"
        ));
        
        let with_context = result.context("Failed to open configuration");
        assert!(with_context.is_err());
        
        let err = with_context.unwrap_err();
        assert!(err.to_string().contains("Failed to open configuration"));
    }
    
    #[test]
    fn test_memory_limit_error() {
        let err = ConversionError::MemoryLimit {
            current: 1024 * 1024,
            limit: 512 * 1024,
        };
        
        assert!(err.to_string().contains("Memory limit exceeded"));
        assert!(err.to_string().contains("1048576"));
        assert!(err.to_string().contains("524288"));
    }
    
    #[test]
    fn test_validation_errors() {
        let err = ValidationError::SizeLimit {
            actual: 1000,
            max: 500,
        };
        
        assert!(err.to_string().contains("Size limit exceeded"));
        assert!(err.to_string().contains("1000 > 500"));
    }
}