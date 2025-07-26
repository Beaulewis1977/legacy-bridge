// Input validation module for secure RTF/Markdown conversion
//
// This module provides comprehensive input validation to prevent
// various attack vectors including path traversal, integer overflow,
// malicious control words, and resource exhaustion attacks.

use std::path::{Path, PathBuf, Component};
use crate::conversion::types::{ConversionError, ConversionResult};
use crate::conversion::security::SecurityLimits;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    // Regex for validating control words (alphanumeric only)
    static ref CONTROL_WORD_REGEX: Regex = Regex::new(r"^[a-zA-Z][a-zA-Z0-9]*$")
        .expect("Failed to compile control word regex");
    
    // Regex for detecting dangerous patterns
    static ref DANGEROUS_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"\\object").expect("Failed to compile object regex"),
        Regex::new(r"\\objdata").expect("Failed to compile objdata regex"),
        Regex::new(r"\\objemb").expect("Failed to compile objemb regex"),
        Regex::new(r"\\objlink").expect("Failed to compile objlink regex"),
        Regex::new(r"\\objautlink").expect("Failed to compile objautlink regex"),
        Regex::new(r"\\objsub").expect("Failed to compile objsub regex"),
        Regex::new(r"\\objpub").expect("Failed to compile objpub regex"),
        Regex::new(r"\\objicemb").expect("Failed to compile objicemb regex"),
        Regex::new(r"\\objhtml").expect("Failed to compile objhtml regex"),
        Regex::new(r"\\objocx").expect("Failed to compile objocx regex"),
        Regex::new(r"\\result").expect("Failed to compile result regex"),
        Regex::new(r"\\pict").expect("Failed to compile pict regex"),
        Regex::new(r"\\field").expect("Failed to compile field regex"),
        Regex::new(r"\\fldinst").expect("Failed to compile fldinst regex"),
        Regex::new(r"\\fldrslt").expect("Failed to compile fldrslt regex"),
        Regex::new(r"\\datafield").expect("Failed to compile datafield regex"),
        Regex::new(r"\\datastore").expect("Failed to compile datastore regex"),
        Regex::new(r"\\xe").expect("Failed to compile xe regex"),
        Regex::new(r"\\tc").expect("Failed to compile tc regex"),
        Regex::new(r"\\bkmkstart").expect("Failed to compile bkmkstart regex"),
        Regex::new(r"\\bkmkend").expect("Failed to compile bkmkend regex"),
        Regex::new(r"\\\*\\generator").expect("Failed to compile generator regex"),
    ];
    
    // Regex for script detection in Markdown
    static ref SCRIPT_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)<script[^>]*>").expect("Failed to compile script regex"),
        Regex::new(r"(?i)javascript:").expect("Failed to compile javascript regex"),
        Regex::new(r"(?i)vbscript:").expect("Failed to compile vbscript regex"),
        Regex::new(r"(?i)onload\s*=").expect("Failed to compile onload regex"),
        Regex::new(r"(?i)onerror\s*=").expect("Failed to compile onerror regex"),
        Regex::new(r"(?i)onclick\s*=").expect("Failed to compile onclick regex"),
        Regex::new(r"(?i)onmouseover\s*=").expect("Failed to compile onmouseover regex"),
    ];
}

/// Input validator with configurable security limits
pub struct InputValidator {
    limits: SecurityLimits,
    allow_absolute_paths: bool,
    allowed_extensions: Vec<String>,
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl InputValidator {
    /// Create a new input validator with default security limits
    pub fn new() -> Self {
        Self {
            limits: SecurityLimits::default(),
            allow_absolute_paths: false,
            allowed_extensions: vec![
                "rtf".to_string(),
                "md".to_string(),
                "markdown".to_string(),
                "txt".to_string(),
            ],
        }
    }
    
    /// Create a validator with custom security limits
    pub fn with_limits(limits: SecurityLimits) -> Self {
        Self {
            limits,
            allow_absolute_paths: false,
            allowed_extensions: vec![
                "rtf".to_string(),
                "md".to_string(),
                "markdown".to_string(),
                "txt".to_string(),
            ],
        }
    }

    /// Validate input size against configured limits
    pub fn validate_size(&self, input: &str, context: &str) -> ConversionResult<()> {
        if input.is_empty() {
            return Err(ConversionError::ValidationError(
                format!("{}: Input is empty", context)
            ));
        }
        
        if input.len() > self.limits.max_file_size {
            return Err(ConversionError::ValidationError(
                format!("{}: Input size {} exceeds maximum allowed size of {} bytes", 
                    context, input.len(), self.limits.max_file_size)
            ));
        }
        
        Ok(())
    }

    /// Validate and sanitize file paths to prevent directory traversal
    pub fn sanitize_path(&self, user_path: &str, allowed_dir: Option<&Path>) -> ConversionResult<PathBuf> {
        // Check for null bytes
        if user_path.contains('\0') {
            return Err(ConversionError::ValidationError(
                "Path contains null bytes".to_string()
            ));
        }
        
        // Check path length
        if user_path.len() > 4096 { // Common filesystem limit
            return Err(ConversionError::ValidationError(
                "Path exceeds maximum length".to_string()
            ));
        }
        
        let path = Path::new(user_path);
        
        // Check each component for safety
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    return Err(ConversionError::ValidationError(
                        "Path contains parent directory references (..)".to_string()
                    ));
                }
                Component::RootDir if !self.allow_absolute_paths => {
                    return Err(ConversionError::ValidationError(
                        "Absolute paths are not allowed".to_string()
                    ));
                }
                Component::Normal(os_str) => {
                    if let Some(s) = os_str.to_str() {
                        // Check for dangerous characters in filename
                        if s.contains('\0') || s.contains('/') || s.contains('\\') {
                            return Err(ConversionError::ValidationError(
                                format!("Path component contains invalid characters: {}", s)
                            ));
                        }
                    } else {
                        return Err(ConversionError::ValidationError(
                            "Path contains invalid UTF-8".to_string()
                        ));
                    }
                }
                _ => {}
            }
        }
        
        // Validate file extension
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                if !self.allowed_extensions.iter().any(|e| e.eq_ignore_ascii_case(ext_str)) {
                    return Err(ConversionError::ValidationError(
                        format!("File extension '{}' is not allowed. Allowed extensions: {:?}", 
                            ext_str, self.allowed_extensions)
                    ));
                }
            }
        }
        
        // If allowed directory is specified, ensure path is within it
        if let Some(allowed) = allowed_dir {
            let full_path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                allowed.join(path)
            };
            
            // Canonicalize to resolve symlinks and relative paths
            let canonical = full_path.canonicalize()
                .map_err(|e| ConversionError::ValidationError(
                    format!("Invalid path: {}", e)
                ))?;
            
            // Ensure the canonical path is within the allowed directory
            if !canonical.starts_with(allowed) {
                return Err(ConversionError::ValidationError(
                    "Path escapes allowed directory".to_string()
                ));
            }
            
            Ok(canonical)
        } else {
            Ok(path.to_path_buf())
        }
    }

    /// Validate numeric parameters with bounds checking
    pub fn validate_number(&self, value: i32, context: &str) -> ConversionResult<i32> {
        if value < self.limits.min_number_value || value > self.limits.max_number_value {
            return Err(ConversionError::ValidationError(
                format!("{}: Numeric value {} is outside allowed range [{}, {}]",
                    context, value, self.limits.min_number_value, self.limits.max_number_value)
            ));
        }
        Ok(value)
    }
    
    /// Validate numeric string before parsing to prevent overflow
    pub fn validate_number_string(&self, number_str: &str, context: &str) -> ConversionResult<()> {
        // Check length to prevent parsing extremely large numbers
        if number_str.len() > self.limits.max_number_length {
            return Err(ConversionError::ValidationError(
                format!("{}: Number string '{}' exceeds maximum length of {} digits",
                    context, number_str, self.limits.max_number_length)
            ));
        }
        
        // Check for valid numeric format
        let is_valid = if number_str.starts_with('-') {
            number_str[1..].chars().all(|c| c.is_ascii_digit())
        } else {
            number_str.chars().all(|c| c.is_ascii_digit())
        };
        
        if !is_valid {
            return Err(ConversionError::ValidationError(
                format!("{}: Invalid number format: {}", context, number_str)
            ));
        }
        
        Ok(())
    }

    /// Validate control word format and safety
    pub fn validate_control_word(&self, word: &str) -> ConversionResult<()> {
        // Check length
        if word.is_empty() {
            return Err(ConversionError::ValidationError(
                "Control word is empty".to_string()
            ));
        }
        
        if word.len() > self.limits.max_control_word_length {
            return Err(ConversionError::ValidationError(
                format!("Control word '{}' exceeds maximum length of {} characters",
                    word, self.limits.max_control_word_length)
            ));
        }
        
        // Check format (must be alphanumeric, starting with letter)
        if !CONTROL_WORD_REGEX.is_match(word) {
            return Err(ConversionError::ValidationError(
                format!("Control word '{}' contains invalid characters", word)
            ));
        }
        
        Ok(())
    }

    /// Pre-validate RTF content for security issues
    pub fn pre_validate_rtf(&self, content: &str) -> ConversionResult<()> {
        // Size validation
        self.validate_size(content, "RTF content")?;
        
        // Basic structure validation
        let trimmed = content.trim();
        if !trimmed.starts_with(r"{\rtf") {
            return Err(ConversionError::ValidationError(
                "Invalid RTF: Document must start with {\\rtf".to_string()
            ));
        }
        
        if !trimmed.ends_with('}') {
            return Err(ConversionError::ValidationError(
                "Invalid RTF: Document must end with }".to_string()
            ));
        }
        
        // Check brace balance (basic check)
        let open_braces = content.matches('{').count();
        let close_braces = content.matches('}').count();
        if open_braces != close_braces {
            return Err(ConversionError::ValidationError(
                format!("Unbalanced braces: {} open, {} close", open_braces, close_braces)
            ));
        }
        
        // Check for dangerous patterns
        for pattern in DANGEROUS_PATTERNS.iter() {
            if pattern.is_match(content) {
                return Err(ConversionError::ValidationError(
                    format!("RTF contains forbidden pattern: {}", pattern.as_str())
                ));
            }
        }
        
        // Check for excessive nesting depth (simple approximation)
        let mut max_depth = 0;
        let mut current_depth = 0;
        let mut escaped = false;
        
        for ch in content.chars() {
            if escaped {
                escaped = false;
                continue;
            }
            
            match ch {
                '\\' => escaped = true,
                '{' => {
                    current_depth += 1;
                    max_depth = max_depth.max(current_depth);
                }
                '}' => {
                    current_depth = current_depth.saturating_sub(1);
                }
                _ => {}
            }
        }
        
        if max_depth > self.limits.max_nesting_depth {
            return Err(ConversionError::ValidationError(
                format!("RTF nesting depth {} exceeds maximum allowed depth of {}",
                    max_depth, self.limits.max_nesting_depth)
            ));
        }
        
        Ok(())
    }

    /// Pre-validate Markdown content for security issues
    pub fn pre_validate_markdown(&self, content: &str) -> ConversionResult<()> {
        // Size validation
        self.validate_size(content, "Markdown content")?;
        
        // Check for script injection attempts
        for pattern in SCRIPT_PATTERNS.iter() {
            if pattern.is_match(content) {
                return Err(ConversionError::ValidationError(
                    format!("Markdown contains potentially malicious pattern: {}", pattern.as_str())
                ));
            }
        }
        
        // Check for data URLs that could embed malicious content
        if content.contains("data:") && content.contains("base64") {
            return Err(ConversionError::ValidationError(
                "Markdown contains data URLs which are not allowed".to_string()
            ));
        }
        
        // Check for file:// URLs
        if content.contains("file://") {
            return Err(ConversionError::ValidationError(
                "Markdown contains file:// URLs which are not allowed".to_string()
            ));
        }
        
        Ok(())
    }

    /// Validate table dimensions
    pub fn validate_table_dimensions(&self, rows: usize, cols: usize) -> ConversionResult<()> {
        if rows > self.limits.max_table_rows {
            return Err(ConversionError::ValidationError(
                format!("Table has {} rows, exceeds maximum of {}", 
                    rows, self.limits.max_table_rows)
            ));
        }
        
        if cols > self.limits.max_table_columns {
            return Err(ConversionError::ValidationError(
                format!("Table has {} columns, exceeds maximum of {}", 
                    cols, self.limits.max_table_columns)
            ));
        }
        
        let total_cells = rows * cols;
        if total_cells > self.limits.max_table_cells {
            return Err(ConversionError::ValidationError(
                format!("Table has {} cells, exceeds maximum of {}", 
                    total_cells, self.limits.max_table_cells)
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_validation() {
        let validator = InputValidator::new();
        
        // Valid size
        assert!(validator.validate_size("Hello, World!", "test").is_ok());
        
        // Empty input
        assert!(validator.validate_size("", "test").is_err());
        
        // Exceeds limit
        let large_input = "x".repeat(11 * 1024 * 1024); // 11MB
        assert!(validator.validate_size(&large_input, "test").is_err());
    }

    #[test]
    fn test_path_sanitization() {
        let validator = InputValidator::new();
        
        // Valid paths
        assert!(validator.sanitize_path("document.rtf", None).is_ok());
        assert!(validator.sanitize_path("subfolder/file.rtf", None).is_ok());
        assert!(validator.sanitize_path("path/to/file.md", None).is_ok());
        
        // Invalid paths - directory traversal
        assert!(validator.sanitize_path("../escape.rtf", None).is_err());
        assert!(validator.sanitize_path("../../etc/passwd", None).is_err());
        assert!(validator.sanitize_path("..\\..\\windows\\system32", None).is_err());
        
        // Invalid paths - absolute paths (when not allowed)
        assert!(validator.sanitize_path("/etc/passwd", None).is_err());
        assert!(validator.sanitize_path("C:\\Windows\\System32", None).is_err());
        
        // Invalid paths - null bytes
        assert!(validator.sanitize_path("file\0name.rtf", None).is_err());
        
        // Invalid extension
        assert!(validator.sanitize_path("script.exe", None).is_err());
        assert!(validator.sanitize_path("payload.dll", None).is_err());
    }

    #[test]
    fn test_number_validation() {
        let validator = InputValidator::new();
        
        // Valid numbers
        assert_eq!(validator.validate_number(0, "test").expect("Should validate 0"), 0);
        assert_eq!(validator.validate_number(100, "test").expect("Should validate 100"), 100);
        assert_eq!(validator.validate_number(-100, "test").expect("Should validate -100"), -100);
        assert_eq!(validator.validate_number(999999, "test").expect("Should validate 999999"), 999999);
        
        // Out of bounds
        assert!(validator.validate_number(2_000_000, "test").is_err());
        assert!(validator.validate_number(-2_000_000, "test").is_err());
        assert!(validator.validate_number(i32::MAX, "test").is_err());
        assert!(validator.validate_number(i32::MIN, "test").is_err());
    }

    #[test]
    fn test_number_string_validation() {
        let validator = InputValidator::new();
        
        // Valid number strings
        assert!(validator.validate_number_string("123", "test").is_ok());
        assert!(validator.validate_number_string("-456", "test").is_ok());
        assert!(validator.validate_number_string("0", "test").is_ok());
        
        // Invalid formats
        assert!(validator.validate_number_string("12.34", "test").is_err());
        assert!(validator.validate_number_string("abc", "test").is_err());
        assert!(validator.validate_number_string("1e10", "test").is_err());
        assert!(validator.validate_number_string("", "test").is_err());
        
        // Too long
        assert!(validator.validate_number_string("12345678901", "test").is_err());
    }

    #[test]
    fn test_control_word_validation() {
        let validator = InputValidator::new();
        
        // Valid control words
        assert!(validator.validate_control_word("rtf").is_ok());
        assert!(validator.validate_control_word("par").is_ok());
        assert!(validator.validate_control_word("b").is_ok());
        assert!(validator.validate_control_word("f0").is_ok());
        
        // Invalid control words
        assert!(validator.validate_control_word("").is_err());
        assert!(validator.validate_control_word("rtf-word").is_err());
        assert!(validator.validate_control_word("123start").is_err());
        assert!(validator.validate_control_word("a".repeat(40).as_str()).is_err());
    }

    #[test]
    fn test_dangerous_rtf_patterns() {
        let validator = InputValidator::new();
        
        // Safe RTF
        assert!(validator.pre_validate_rtf(r"{\rtf1 Hello World}").is_ok());
        assert!(validator.pre_validate_rtf(r"{\rtf1\ansi\deff0 {\fonttbl{\f0 Times;}} Text}").is_ok());
        
        // Dangerous patterns
        assert!(validator.pre_validate_rtf(r"{\rtf1 \object\objdata}").is_err());
        assert!(validator.pre_validate_rtf(r"{\rtf1 \field\fldinst}").is_err());
        assert!(validator.pre_validate_rtf(r"{\rtf1 \pict\picw100}").is_err());
        assert!(validator.pre_validate_rtf(r"{\rtf1 \result malicious}").is_err());
        
        // Invalid structure
        assert!(validator.pre_validate_rtf("Not RTF").is_err());
        assert!(validator.pre_validate_rtf(r"{\rtf1 No closing brace").is_err());
        assert!(validator.pre_validate_rtf(r"No opening brace}").is_err());
        
        // Excessive nesting
        let mut deep_rtf = String::from(r"{\rtf1 ");
        for _ in 0..60 {
            deep_rtf.push('{');
        }
        deep_rtf.push_str("text");
        for _ in 0..60 {
            deep_rtf.push('}');
        }
        deep_rtf.push('}');
        assert!(validator.pre_validate_rtf(&deep_rtf).is_err());
    }

    #[test]
    fn test_markdown_script_detection() {
        let validator = InputValidator::new();
        
        // Safe Markdown
        assert!(validator.pre_validate_markdown("# Hello\n\nThis is **bold** text").is_ok());
        assert!(validator.pre_validate_markdown("[Link](https://example.com)").is_ok());
        
        // Script injection attempts
        assert!(validator.pre_validate_markdown("<script>alert('XSS')</script>").is_err());
        assert!(validator.pre_validate_markdown("[Click](javascript:alert('XSS'))").is_err());
        assert!(validator.pre_validate_markdown("<img onload='evil()'>").is_err());
        assert!(validator.pre_validate_markdown("<div onclick='hack()'>").is_err());
        
        // Data URLs
        assert!(validator.pre_validate_markdown("![](data:text/html;base64,PHNjcmlwdD4=)").is_err());
        
        // File URLs
        assert!(validator.pre_validate_markdown("[Local](file:///etc/passwd)").is_err());
    }

    #[test]
    fn test_table_dimension_validation() {
        let validator = InputValidator::new();
        
        // Valid tables
        assert!(validator.validate_table_dimensions(10, 5).is_ok());
        assert!(validator.validate_table_dimensions(100, 10).is_ok());
        
        // Too many rows
        assert!(validator.validate_table_dimensions(2000, 5).is_err());
        
        // Too many columns  
        assert!(validator.validate_table_dimensions(10, 200).is_err());
        
        // Too many total cells
        assert!(validator.validate_table_dimensions(500, 500).is_err());
    }
}