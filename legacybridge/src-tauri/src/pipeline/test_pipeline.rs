// Pipeline integration tests
#[cfg(test)]
mod tests {
    use crate::pipeline::{
        DocumentPipeline, PipelineConfig, convert_rtf_to_markdown_with_pipeline,
        ValidationLevel,
    };

    #[test]
    fn test_basic_pipeline_conversion() {
        let rtf = r"{\rtf1\ansi\deff0 {\fonttbl{\f0 Times New Roman;}} Hello Pipeline World\par}";
        
        let result = convert_rtf_to_markdown_with_pipeline(rtf, None);
        assert!(result.is_ok());
        
        let (markdown, context) = result.unwrap();
        assert!(markdown.contains("Hello Pipeline World"));
        assert!(context.validation_results.iter().all(|r| r.level != ValidationLevel::Error));
    }

    #[test]
    fn test_pipeline_with_validation() {
        let config = PipelineConfig {
            strict_validation: true,
            auto_recovery: true,
            template: None,
            preserve_formatting: true,
            legacy_mode: false,
        };
        
        // Valid RTF
        let valid_rtf = r"{\rtf1\ansi Hello World}";
        let result = convert_rtf_to_markdown_with_pipeline(valid_rtf, Some(config.clone()));
        assert!(result.is_ok());
        
        // Invalid RTF (missing header)
        let invalid_rtf = r"{Hello World}";
        let result = convert_rtf_to_markdown_with_pipeline(invalid_rtf, Some(config));
        // Should still work with auto-recovery
        assert!(result.is_ok());
    }

    #[test]
    fn test_pipeline_error_recovery() {
        let config = PipelineConfig {
            strict_validation: false,
            auto_recovery: true,
            template: None,
            preserve_formatting: true,
            legacy_mode: false,
        };
        
        // Malformed RTF with unmatched braces
        let malformed_rtf = r"{\rtf1 Hello {World";
        let result = convert_rtf_to_markdown_with_pipeline(malformed_rtf, Some(config));
        
        assert!(result.is_ok());
        let (_markdown, context) = result.unwrap();
        
        // Should have recovery actions
        assert!(!context.recovery_actions.is_empty());
        assert!(context.recovery_actions.iter().any(|a| a.applied));
    }

    #[test]
    fn test_pipeline_with_template() {
        let config = PipelineConfig {
            strict_validation: false,
            auto_recovery: true,
            template: Some("memo".to_string()),
            preserve_formatting: true,
            legacy_mode: false,
        };
        
        let rtf = r"{\rtf1\ansi\deff0 {\fonttbl{\f0 Arial;}} \f0 Memo content\par}";
        let result = convert_rtf_to_markdown_with_pipeline(rtf, Some(config));
        
        assert!(result.is_ok());
        let (markdown, _context) = result.unwrap();
        
        // Should have template applied
        assert!(markdown.contains("Memo content"));
    }

    #[test]
    fn test_pipeline_table_handling() {
        let rtf = r"{\rtf1\ansi\deff0
            {\trowd\cellx2000\cellx4000
            Cell 1\cell Cell 2\cell\row}
        }";
        
        let result = convert_rtf_to_markdown_with_pipeline(rtf, None);
        assert!(result.is_ok());
        
        let (markdown, _context) = result.unwrap();
        // Should contain table markers
        assert!(markdown.contains("|"));
    }
}