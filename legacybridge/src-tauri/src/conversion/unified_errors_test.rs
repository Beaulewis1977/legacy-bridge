// Test module for unified error handling
//
// This demonstrates end-to-end error propagation across all layers

#[cfg(test)]
mod tests {
    use super::super::unified_errors::*;
    use super::super::types::ConversionError as OldError;
    use std::collections::HashMap;

    #[test]
    fn test_error_context_creation() {
        let mut context = ErrorContext::default();
        context.line = Some(42);
        context.column = Some(15);
        context.file_path = Some("test.rtf".to_string());
        context.context.insert("token".to_string(), "\\par".to_string());

        assert_eq!(context.line, Some(42));
        assert_eq!(context.column, Some(15));
        assert_eq!(context.file_path, Some("test.rtf".to_string()));
        assert_eq!(context.context.get("token"), Some(&"\\par".to_string()));
    }

    #[test]
    fn test_parse_error_creation() {
        let error = LegacyBridgeError::ParseError {
            message: "Unexpected token".to_string(),
            line: 10,
            column: 5,
            expected: Some("control word".to_string()),
            found: Some("plain text".to_string()),
            file_path: Some("document.rtf".to_string()),
        };

        assert_eq!(error.error_type(), "ParseError");
        assert_eq!(error.error_code(), -1001);
        assert!(!error.is_recoverable());
        
        let user_msg = error.user_message();
        assert_eq!(user_msg, "The document format is invalid or corrupted");
        
        let suggestions = error.suggestions();
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions[0].contains("valid RTF"));
    }

    #[test]
    fn test_conversion_error_with_suggestions() {
        let error = LegacyBridgeError::ConversionError {
            source_format: "RTF".to_string(),
            target_format: "Markdown".to_string(),
            details: "Complex nested tables not supported".to_string(),
            recoverable: true,
            suggestions: vec![
                "Simplify table structure".to_string(),
                "Use nested lists instead".to_string(),
            ],
        };

        assert_eq!(error.error_type(), "ConversionError");
        assert_eq!(error.error_code(), -1002);
        assert!(error.is_recoverable());
        
        let suggestions = error.suggestions();
        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0], "Simplify table structure");
    }

    #[test]
    fn test_io_error_creation() {
        let error = LegacyBridgeError::IOError {
            operation: "read".to_string(),
            path: "/path/to/file.rtf".to_string(),
            cause: "Permission denied".to_string(),
            error_code: Some(13),
        };

        assert_eq!(error.error_type(), "IOError");
        assert_eq!(error.error_code(), -1003);
        assert!(!error.is_recoverable());
        
        let display = format!("{}", error);
        assert!(display.contains("read"));
        assert!(display.contains("/path/to/file.rtf"));
        assert!(display.contains("Permission denied"));
    }

    #[test]
    fn test_validation_error_with_location() {
        let mut location = ErrorContext::default();
        location.line = Some(25);
        location.column = Some(10);

        let error = LegacyBridgeError::ValidationError {
            field: "font size".to_string(),
            expected: "positive integer".to_string(),
            received: "-12".to_string(),
            location: Some(location),
        };

        assert_eq!(error.error_type(), "ValidationError");
        assert!(error.is_recoverable());
        
        let display = format!("{}", error);
        assert!(display.contains("font size"));
        assert!(display.contains("positive integer"));
        assert!(display.contains("-12"));
    }

    #[test]
    fn test_resource_limit_error() {
        let error = LegacyBridgeError::ResourceLimitError {
            resource: "File size".to_string(),
            limit: "10MB".to_string(),
            actual: "25MB".to_string(),
            suggestion: "Split the file into smaller parts".to_string(),
        };

        assert_eq!(error.error_type(), "ResourceLimitError");
        assert!(!error.is_recoverable());
        
        let suggestions = error.suggestions();
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0], "Split the file into smaller parts");
    }

    #[test]
    fn test_error_json_serialization() {
        let error = LegacyBridgeError::ParseError {
            message: "Invalid RTF header".to_string(),
            line: 1,
            column: 1,
            expected: Some("{\\rtf".to_string()),
            found: Some("RTF".to_string()),
            file_path: Some("test.rtf".to_string()),
        };

        let json = error.to_json().unwrap();
        
        // Verify JSON contains all required fields
        assert!(json.contains("\"error_type\":\"ParseError\""));
        assert!(json.contains("\"error_code\":-1001"));
        assert!(json.contains("\"user_message\""));
        assert!(json.contains("\"suggestions\""));
        assert!(json.contains("\"recoverable\":false"));
        assert!(json.contains("\"timestamp\""));
        
        // Verify details are included
        assert!(json.contains("\"line\":1"));
        assert!(json.contains("\"column\":1"));
        assert!(json.contains("test.rtf"));
    }

    #[test]
    fn test_old_error_conversion() {
        // Test conversion from old error format
        let old_error = OldError::ParseError("Unexpected end of file".to_string());
        let new_error: LegacyBridgeError = old_error.into();

        assert_eq!(new_error.error_type(), "ParseError");
        
        if let LegacyBridgeError::ParseError { message, .. } = new_error {
            assert_eq!(message, "Unexpected end of file");
        } else {
            panic!("Wrong error type");
        }
    }

    #[test]
    fn test_thread_local_error_storage() {
        use super::super::unified_errors::{set_last_error, get_last_error_json, clear_last_error};

        // Clear any existing error
        clear_last_error();
        assert!(get_last_error_json().is_none());

        // Set an error
        let error = LegacyBridgeError::SystemError {
            component: "RTF Parser".to_string(),
            error_code: 500,
            description: "Internal parser error".to_string(),
            internal_message: Some("Stack overflow at depth 1000".to_string()),
        };
        
        set_last_error(error);
        
        // Retrieve error as JSON
        let json = get_last_error_json().expect("Should have error");
        assert!(json.contains("SystemError"));
        assert!(json.contains("RTF Parser"));
        assert!(json.contains("Internal parser error"));
        
        // Clear error
        clear_last_error();
        assert!(get_last_error_json().is_none());
    }

    #[test]
    fn test_error_severity_levels() {
        // Critical error
        let critical = LegacyBridgeError::SystemError {
            component: "Core".to_string(),
            error_code: 999,
            description: "Critical system failure".to_string(),
            internal_message: None,
        };
        assert!(!critical.is_recoverable());

        // High severity
        let high = LegacyBridgeError::ParseError {
            message: "Invalid document structure".to_string(),
            line: 1,
            column: 1,
            expected: None,
            found: None,
            file_path: None,
        };
        assert!(!high.is_recoverable());

        // Medium severity (recoverable)
        let medium = LegacyBridgeError::ConversionError {
            source_format: "RTF".to_string(),
            target_format: "MD".to_string(),
            details: "Unsupported feature".to_string(),
            recoverable: true,
            suggestions: vec!["Use alternative approach".to_string()],
        };
        assert!(medium.is_recoverable());

        // Low severity
        let low = LegacyBridgeError::NotImplementedError {
            feature: "Advanced tables".to_string(),
            workaround: Some("Use simple tables".to_string()),
            planned_version: Some("2.0".to_string()),
        };
        assert!(!low.is_recoverable()); // Not implemented features aren't recoverable
    }

    #[test]
    fn test_error_display_formatting() {
        let error = LegacyBridgeError::ParseError {
            message: "Unterminated group".to_string(),
            line: 42,
            column: 15,
            expected: Some("}".to_string()),
            found: Some("EOF".to_string()),
            file_path: Some("document.rtf".to_string()),
        };

        let display = format!("{}", error);
        
        // Should contain all relevant information
        assert!(display.contains("Parse error"));
        assert!(display.contains("Unterminated group"));
        assert!(display.contains("line 42"));
        assert!(display.contains("column 15"));
        assert!(display.contains("expected: }"));
        assert!(display.contains("found: EOF"));
        assert!(display.contains("document.rtf"));
    }

    // Integration test simulating FFI boundary
    #[test]
    fn test_ffi_error_propagation() {
        use crate::ffi_error_bridge::{create_parse_error, set_and_return_error};

        let error = create_parse_error(
            "Missing RTF header",
            1,
            1,
            Some("input.rtf")
        );

        let error_code = set_and_return_error(error);
        assert_eq!(error_code, -1001); // Parse error code

        // Check that error was stored
        let json = get_last_error_json().expect("Should have stored error");
        assert!(json.contains("Missing RTF header"));
        assert!(json.contains("input.rtf"));
    }
}