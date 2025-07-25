// Core conversion module that handles RTF to Markdown and vice versa

pub mod rtf_lexer;
pub mod rtf_parser;
pub mod markdown_generator;
pub mod types;

pub use types::{ConversionError, ConversionResult};
pub use rtf_parser::RtfParser;
pub use markdown_generator::MarkdownGenerator;

/// Convert RTF content to Markdown
pub fn rtf_to_markdown(rtf_content: &str) -> ConversionResult<String> {
    // Check if we should use the pipeline based on content characteristics
    if should_use_pipeline(rtf_content) {
        // Use pipeline for complex documents
        use crate::pipeline::{PipelineConfig, convert_rtf_to_markdown_with_pipeline};
        let config = PipelineConfig {
            strict_validation: false, // Less strict for backward compatibility
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
        // Use simple conversion for basic documents
        let tokens = rtf_lexer::tokenize(rtf_content)?;
        let document = RtfParser::parse(tokens)?;
        let markdown = MarkdownGenerator::generate(&document)?;
        Ok(markdown)
    }
}

/// Determine if a document should use the pipeline
fn should_use_pipeline(rtf_content: &str) -> bool {
    // Use pipeline for documents that might benefit from advanced features
    rtf_content.contains("\\trowd") || // Tables
    rtf_content.contains("\\object") || // Embedded objects
    rtf_content.contains("\\stylesheet") || // Style sheets
    rtf_content.len() > 50000 // Large documents
}

/// Convert Markdown content to RTF
pub fn markdown_to_rtf(_markdown_content: &str) -> ConversionResult<String> {
    // TODO: Implement markdown to RTF conversion
    Err(ConversionError::NotImplemented(
        "Markdown to RTF conversion is not yet implemented".to_string()
    ))
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
}