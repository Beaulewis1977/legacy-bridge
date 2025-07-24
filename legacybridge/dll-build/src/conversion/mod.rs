// Core conversion module that handles RTF to Markdown and vice versa

pub mod rtf_lexer;
pub mod rtf_parser;
pub mod markdown_generator;
pub mod markdown_parser;
pub mod rtf_generator;
pub mod types;

pub use types::{ConversionError, ConversionResult};
pub use rtf_parser::RtfParser;
pub use markdown_generator::MarkdownGenerator;
pub use markdown_parser::MarkdownParser;
pub use rtf_generator::RtfGenerator;

/// Convert RTF content to Markdown
pub fn rtf_to_markdown(rtf_content: &str) -> ConversionResult<String> {
    // Use simple conversion for all documents in DLL build
    let tokens = rtf_lexer::tokenize(rtf_content)?;
    let document = RtfParser::parse(tokens)?;
    let markdown = MarkdownGenerator::generate(&document)?;
    Ok(markdown)
}

/// Convert Markdown content to RTF
pub fn markdown_to_rtf(markdown_content: &str) -> ConversionResult<String> {
    // Use simple conversion for all documents in DLL build
    let document = MarkdownParser::parse(markdown_content)?;
    let rtf = RtfGenerator::generate(&document)?;
    Ok(rtf)
}