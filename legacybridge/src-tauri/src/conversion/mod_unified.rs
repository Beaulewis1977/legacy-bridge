// Core conversion module that handles RTF to Markdown and vice versa
// Using unified architecture to consolidate duplicate implementations

pub mod error;
pub mod rtf_lexer;
pub mod rtf_parser;
pub mod markdown_generator;
pub mod markdown_parser;
pub mod markdown_parser_optimized;
pub mod rtf_generator;
pub mod types;
pub mod security;
pub mod secure_parser;
pub mod secure_generator;
pub mod input_validation;
pub mod memory_pools;
pub mod markdown_generator_pooled;
pub mod rtf_parser_pooled;
pub mod pooled_converter;
pub mod pool_monitoring;

// Unified implementation modules
pub mod unified_config;
pub mod unified_parser;
pub mod unified_generator;

#[cfg(test)]
pub mod malicious_input_tests;

pub use types::{ConversionError, ConversionResult};
pub use rtf_parser::RtfParser;
pub use markdown_generator::MarkdownGenerator;
pub use markdown_parser::MarkdownParser;
pub use rtf_generator::RtfGenerator;
pub use secure_parser::SecureRtfParser;
pub use input_validation::InputValidator;

// Export unified components
pub use unified_config::{ConversionConfig, SecurityLevel, ConversionConfigBuilder};
pub use unified_parser::UnifiedParser;
pub use unified_generator::UnifiedGenerator;

/// Convert RTF content to Markdown
pub fn rtf_to_markdown(rtf_content: &str) -> ConversionResult<String> {
    rtf_to_markdown_with_config(rtf_content, ConversionConfig::default())
}

/// Convert RTF content to Markdown with custom configuration
pub fn rtf_to_markdown_with_config(rtf_content: &str, config: ConversionConfig) -> ConversionResult<String> {
    // Use unified parser and generator
    let mut parser = UnifiedParser::new(config.clone());
    let mut generator = UnifiedGenerator::new(config.clone());
    
    // Check if we should use the pipeline
    if config.use_pipeline && should_use_pipeline(rtf_content) {
        // Use pipeline for complex documents
        use crate::pipeline::{PipelineConfig, convert_rtf_to_markdown_with_pipeline};
        let pipeline_config = PipelineConfig {
            strict_validation: config.validation_enabled,
            auto_recovery: config.auto_recovery,
            template: None,
            preserve_formatting: config.preserve_formatting,
            legacy_mode: false,
        };
        match convert_rtf_to_markdown_with_pipeline(rtf_content, Some(pipeline_config)) {
            Ok((markdown, _context)) => Ok(markdown),
            Err(e) => Err(e),
        }
    } else {
        // Direct conversion using unified components
        let tokens = rtf_lexer::tokenize(rtf_content)?;
        let document = parser.parse_rtf(tokens)?;
        let markdown = generator.generate_markdown(&document)?;
        Ok(markdown)
    }
}

/// Determine if a document should use the pipeline
fn should_use_pipeline(rtf_content: &str) -> bool {
    // Use pipeline for documents that might benefit from advanced features
    rtf_content.contains("\\trowd") || // Tables
    rtf_content.contains("\\stylesheet") || // Style sheets
    rtf_content.len() > 50000 // Large documents
}

/// Convert Markdown content to RTF
pub fn markdown_to_rtf(markdown_content: &str) -> ConversionResult<String> {
    markdown_to_rtf_with_config(markdown_content, ConversionConfig::default())
}

/// Convert Markdown content to RTF with custom configuration
pub fn markdown_to_rtf_with_config(markdown_content: &str, config: ConversionConfig) -> ConversionResult<String> {
    // Use unified parser and generator
    let mut parser = UnifiedParser::new(config.clone());
    let mut generator = UnifiedGenerator::new(config.clone());
    
    // Check if we should use the pipeline
    if config.use_pipeline && should_use_pipeline_md(markdown_content) {
        // Use pipeline for complex documents
        use crate::pipeline::{PipelineConfig, convert_markdown_to_rtf_with_pipeline};
        let pipeline_config = PipelineConfig {
            strict_validation: config.validation_enabled,
            auto_recovery: config.auto_recovery,
            template: None,
            preserve_formatting: config.preserve_formatting,
            legacy_mode: false,
        };
        match convert_markdown_to_rtf_with_pipeline(markdown_content, Some(pipeline_config)) {
            Ok((rtf, _context)) => Ok(rtf),
            Err(e) => Err(e),
        }
    } else {
        // Direct conversion using unified components
        let document = parser.parse_markdown(markdown_content)?;
        let rtf = generator.generate_rtf(&document)?;
        Ok(rtf)
    }
}

/// Determine if a markdown document should use the pipeline
fn should_use_pipeline_md(markdown_content: &str) -> bool {
    // Use pipeline for documents that might benefit from advanced features
    markdown_content.contains("|") || // Tables
    markdown_content.contains("```") || // Code blocks  
    markdown_content.contains("[^") || // Footnotes
    markdown_content.len() > 50000 // Large documents
}

/// Secure RTF to Markdown conversion with input validation
/// DEPRECATED: Use rtf_to_markdown_with_config with high_security() config instead
#[deprecated(since = "0.2.0", note = "Use rtf_to_markdown_with_config with ConversionConfig::high_security()")]
pub fn secure_rtf_to_markdown(rtf_content: &str) -> ConversionResult<String> {
    rtf_to_markdown_with_config(rtf_content, ConversionConfig::high_security())
}

/// Secure Markdown to RTF conversion with input validation
/// DEPRECATED: Use markdown_to_rtf_with_config with high_security() config instead
#[deprecated(since = "0.2.0", note = "Use markdown_to_rtf_with_config with ConversionConfig::high_security()")]
pub fn secure_markdown_to_rtf(markdown_content: &str) -> ConversionResult<String> {
    markdown_to_rtf_with_config(markdown_content, ConversionConfig::high_security())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_rtf_to_markdown() {
        let rtf = r"{\rtf1\ansi\deff0 {\fonttbl{\f0 Times New Roman;}} Hello World\par}";
        let result = rtf_to_markdown(rtf);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_secure_rtf_to_markdown() {
        let rtf = r"{\rtf1 Hello World\par}";
        let result = rtf_to_markdown_with_config(rtf, ConversionConfig::high_security());
        assert!(result.is_ok());
        let markdown = result.expect("conversion should succeed");
        assert!(markdown.contains("Hello World"));
    }
    
    #[test]
    fn test_secure_blocks_dangerous_rtf() {
        let dangerous_rtf = r"{\rtf1 \object\objdata Malicious}";
        let result = rtf_to_markdown_with_config(dangerous_rtf, ConversionConfig::high_security());
        assert!(result.is_err());
    }
    
    #[test]
    fn test_configurable_security_levels() {
        let rtf = r"{\rtf1 Hello World\par}";
        
        // Test different security levels
        let high_sec_result = rtf_to_markdown_with_config(rtf, ConversionConfig::high_security());
        assert!(high_sec_result.is_ok());
        
        let balanced_result = rtf_to_markdown_with_config(rtf, ConversionConfig::balanced());
        assert!(balanced_result.is_ok());
        
        let high_perf_result = rtf_to_markdown_with_config(rtf, ConversionConfig::high_performance());
        assert!(high_perf_result.is_ok());
    }
    
    #[test]
    fn test_config_builder() {
        let rtf = r"{\rtf1 Test\par}";
        let config = ConversionConfigBuilder::new()
            .security_level(SecurityLevel::Enhanced { 
                limits: security::SecurityLimits::default() 
            })
            .validation(true)
            .logging(false)
            .build();
            
        let result = rtf_to_markdown_with_config(rtf, config);
        assert!(result.is_ok());
    }
}