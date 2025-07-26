// Updated conversion module with SIMD support
// Core conversion module that handles RTF to Markdown and vice versa

pub mod error;
pub mod rtf_lexer;
pub mod rtf_lexer_simd;
pub mod rtf_parser;
pub mod rtf_parser_optimized;
pub mod markdown_generator;
pub mod markdown_parser;
pub mod markdown_parser_optimized;
pub mod markdown_parser_simd;
pub mod markdown_simd_utils;
pub mod rtf_generator;
pub mod types;
pub mod security;
pub mod secure_parser;
pub mod secure_generator;
pub mod input_validation;
pub mod memory_pools;
pub mod markdown_generator_pooled;
pub mod rtf_parser_pooled;
pub mod string_interner;
pub mod simd_conversion;

#[cfg(all(test, not(target_env = "msvc")))]
pub mod simd_benchmarks;

#[cfg(test)]
pub mod malicious_input_tests;

pub use types::{ConversionError, ConversionResult};
pub use rtf_parser::RtfParser;
pub use markdown_generator::MarkdownGenerator;
pub use markdown_parser::MarkdownParser;
pub use rtf_generator::RtfGenerator;
pub use secure_parser::SecureRtfParser;
pub use input_validation::InputValidator;

/// Convert RTF content to Markdown with automatic SIMD optimization
pub fn rtf_to_markdown(rtf_content: &str) -> ConversionResult<String> {
    // Check if SIMD is available
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("sse2") {
            return simd_conversion::rtf_to_markdown_simd(rtf_content);
        }
    }
    
    // Fallback to non-SIMD version
    rtf_to_markdown_scalar(rtf_content)
}

/// Scalar (non-SIMD) RTF to Markdown conversion
fn rtf_to_markdown_scalar(rtf_content: &str) -> ConversionResult<String> {
    // SECURITY: Validate input size first (10MB limit)
    let validator = InputValidator::new();
    validator.validate_size(rtf_content, "RTF content")?;
    
    // Check if we should use the pipeline based on content characteristics
    if should_use_pipeline(rtf_content) {
        // Use pipeline for complex documents
        use crate::pipeline::{PipelineConfig, convert_rtf_to_markdown_with_pipeline};
        let config = PipelineConfig {
            strict_validation: true, // SECURITY: Enable strict validation
            auto_recovery: true,
            template: None,
            preserve_formatting: true,
            legacy_mode: false,
        };
        match convert_rtf_to_markdown_with_pipeline(rtf_content, Some(config)) {
            Ok((markdown, _context)) => Ok(markdown),
            Err(e) => Err(e),
        }
    } else {
        // SECURITY: Use SecureRtfParser instead of standard parser
        let tokens = rtf_lexer::tokenize(rtf_content)?;
        let document = SecureRtfParser::parse(tokens)?;
        let markdown = MarkdownGenerator::generate(&document)?;
        Ok(markdown)
    }
}

/// Determine if a document should use the pipeline
fn should_use_pipeline(rtf_content: &str) -> bool {
    // Use pipeline for documents that might benefit from advanced features
    rtf_content.contains("\\trowd") || // Tables
    // SECURITY: Removed \object check - dangerous control word
    rtf_content.contains("\\stylesheet") || // Style sheets
    rtf_content.len() > 50000 // Large documents
}

/// Convert Markdown content to RTF with automatic SIMD optimization
pub fn markdown_to_rtf(markdown_content: &str) -> ConversionResult<String> {
    // Check if SIMD is available
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("sse2") {
            return simd_conversion::markdown_to_rtf_simd(markdown_content);
        }
    }
    
    // Fallback to non-SIMD version
    markdown_to_rtf_scalar(markdown_content)
}

/// Scalar (non-SIMD) Markdown to RTF conversion
fn markdown_to_rtf_scalar(markdown_content: &str) -> ConversionResult<String> {
    // SECURITY: Validate input size first (10MB limit)
    let validator = InputValidator::new();
    validator.validate_size(markdown_content, "Markdown content")?;
    
    // Check if we should use the pipeline based on content characteristics
    if should_use_pipeline_md(markdown_content) {
        // Use pipeline for complex documents
        use crate::pipeline::{PipelineConfig, convert_markdown_to_rtf_with_pipeline};
        let config = PipelineConfig {
            strict_validation: true, // SECURITY: Enable strict validation
            auto_recovery: true,
            template: None,
            preserve_formatting: true,
            legacy_mode: false,
        };
        match convert_markdown_to_rtf_with_pipeline(markdown_content, Some(config)) {
            Ok((rtf, _context)) => Ok(rtf),
            Err(e) => Err(e),
        }
    } else {
        // Use simple conversion for basic documents
        let document = MarkdownParser::parse(markdown_content)?;
        let rtf = RtfGenerator::generate(&document)?;
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
pub fn secure_rtf_to_markdown(rtf_content: &str) -> ConversionResult<String> {
    // Validate input first
    let validator = InputValidator::new();
    validator.pre_validate_rtf(rtf_content)?;
    
    // Use secure parser with SIMD if available
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("sse2") {
            let tokens = rtf_lexer_simd::tokenize_simd(rtf_content)?;
            let document = SecureRtfParser::parse(tokens)?;
            let markdown = MarkdownGenerator::generate(&document)?;
            return Ok(markdown);
        }
    }
    
    // Fallback to non-SIMD
    let tokens = rtf_lexer::tokenize(rtf_content)?;
    let document = SecureRtfParser::parse(tokens)?;
    let markdown = MarkdownGenerator::generate(&document)?;
    Ok(markdown)
}

/// Secure Markdown to RTF conversion with input validation
pub fn secure_markdown_to_rtf(markdown_content: &str) -> ConversionResult<String> {
    // Validate input first
    let validator = InputValidator::new();
    validator.pre_validate_markdown(markdown_content)?;
    
    // Use SIMD parser if available
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("sse2") {
            let mut parser = markdown_parser_simd::SimdMarkdownParser::new();
            let document = parser.parse(markdown_content)?;
            let rtf = RtfGenerator::generate(&document)?;
            return Ok(rtf);
        }
    }
    
    // Fallback to standard parser
    let document = MarkdownParser::parse(markdown_content)?;
    let rtf = RtfGenerator::generate(&document)?;
    Ok(rtf)
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
        let result = secure_rtf_to_markdown(rtf);
        assert!(result.is_ok());
        let markdown = result.expect("conversion should succeed");
        assert!(markdown.contains("Hello World"));
    }
    
    #[test]
    fn test_secure_blocks_dangerous_rtf() {
        let dangerous_rtf = r"{\rtf1 \object\objdata Malicious}";
        let result = secure_rtf_to_markdown(dangerous_rtf);
        assert!(result.is_err());
    }
    
    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn test_simd_conversion() {
        if is_x86_feature_detected!("sse2") {
            let rtf = r"{\rtf1 SIMD Test\par}";
            let result = rtf_to_markdown(rtf);
            assert!(result.is_ok());
            
            let markdown = "# SIMD Test\n\nThis tests SIMD conversion.";
            let result = markdown_to_rtf(markdown);
            assert!(result.is_ok());
        }
    }
}