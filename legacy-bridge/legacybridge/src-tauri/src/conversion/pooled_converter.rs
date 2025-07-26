// Pooled Converter - Main conversion functions using memory pools

use super::types::ConversionResult;
use super::rtf_lexer;
use super::rtf_lexer_pooled;
use super::rtf_parser_pooled::PooledRtfParser;
use super::markdown_generator_pooled::PooledMarkdownGenerator;
use super::markdown_parser::MarkdownParser;
use super::rtf_generator::RtfGenerator;
use super::secure_parser::SecureRtfParser;
use super::input_validation::InputValidator;
use super::memory_pools::CONVERSION_POOLS;

/// Convert RTF to Markdown using memory pools for reduced allocation overhead
pub fn rtf_to_markdown_pooled(rtf_content: &str) -> ConversionResult<String> {
    // SECURITY: Validate input size first (10MB limit)
    let validator = InputValidator::new();
    validator.validate_size(rtf_content, "RTF content")?;
    
    // Check if we should use the pipeline based on content characteristics
    if should_use_pipeline(rtf_content) {
        // Use pipeline for complex documents
        use crate::pipeline::{PipelineConfig, convert_rtf_to_markdown_with_pipeline};
        let config = PipelineConfig {
            strict_validation: true,
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
        // Use pooled lexer, parser and generator for simple documents
        let tokens = rtf_lexer_pooled::tokenize_pooled(rtf_content)?;
        let document = PooledRtfParser::parse(tokens)?;
        let markdown = PooledMarkdownGenerator::generate(&document)?;
        Ok(markdown)
    }
}

/// Secure RTF to Markdown conversion with memory pools
pub fn secure_rtf_to_markdown_pooled(rtf_content: &str) -> ConversionResult<String> {
    // Validate input first
    let validator = InputValidator::new();
    validator.pre_validate_rtf(rtf_content)?;
    
    // Use pooled lexer, parser and generator
    let tokens = rtf_lexer_pooled::tokenize_pooled(rtf_content)?;
    let document = PooledRtfParser::parse(tokens)?;
    let markdown = PooledMarkdownGenerator::generate(&document)?;
    Ok(markdown)
}

/// Convert Markdown to RTF (currently using standard allocators)
/// TODO: Implement pooled markdown parser and RTF generator
pub fn markdown_to_rtf_pooled(markdown_content: &str) -> ConversionResult<String> {
    // SECURITY: Validate input size first
    let validator = InputValidator::new();
    validator.validate_size(markdown_content, "Markdown content")?;
    
    // For now, use standard conversion
    // TODO: Implement PooledMarkdownParser and PooledRtfGenerator
    let document = MarkdownParser::parse(markdown_content)?;
    let rtf = RtfGenerator::generate(&document)?;
    Ok(rtf)
}

/// Get memory pool statistics
pub fn get_pool_stats() -> super::memory_pools::PoolStats {
    CONVERSION_POOLS.get_stats()
}

/// Warm up memory pools by pre-allocating objects
pub fn warm_up_pools() {
    // Pre-allocate some objects to warm up the pools
    let _strings: Vec<_> = (0..10)
        .map(|_| CONVERSION_POOLS.get_string_buffer(1024))
        .collect();
    
    let _buffers: Vec<_> = (0..5)
        .map(|_| CONVERSION_POOLS.get_byte_buffer(4096))
        .collect();
    
    let _nodes: Vec<_> = (0..5)
        .map(|_| CONVERSION_POOLS.get_node_vec())
        .collect();
    
    // Objects will be returned to pools when dropped
}

// Helper function copied from mod.rs
fn should_use_pipeline(rtf_content: &str) -> bool {
    rtf_content.contains("\\trowd") || // Tables
    rtf_content.contains("\\stylesheet") || // Style sheets
    rtf_content.len() > 50000 // Large documents
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pooled_rtf_to_markdown() {
        let rtf = r"{\rtf1\ansi\deff0 {\fonttbl{\f0 Times New Roman;}} Hello World\par}";
        let result = rtf_to_markdown_pooled(rtf);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Hello World"));
    }
    
    #[test]
    fn test_pool_warmup() {
        let initial_stats = get_pool_stats();
        warm_up_pools();
        let warmed_stats = get_pool_stats();
        
        // Some pools should have objects after warmup
        assert!(warmed_stats.total_pooled_objects() >= initial_stats.total_pooled_objects());
    }
    
    #[test]
    fn test_memory_efficiency() {
        // Get baseline stats
        let initial_stats = get_pool_stats();
        
        // Process multiple documents
        for i in 0..20 {
            let rtf = format!(
                r"{{\rtf1\ansi\deff0 {{\fonttbl{{\f0 Times;}}}} Document {} with some content\par}}",
                i
            );
            let _ = rtf_to_markdown_pooled(&rtf).unwrap();
        }
        
        // Check pool utilization
        let final_stats = get_pool_stats();
        println!("Initial pooled objects: {}", initial_stats.total_pooled_objects());
        println!("Final pooled objects: {}", final_stats.total_pooled_objects());
        println!("Estimated memory usage: {} bytes", final_stats.estimated_memory_usage());
        
        // Pools should be populated but not excessively
        assert!(final_stats.total_pooled_objects() > initial_stats.total_pooled_objects());
        assert!(final_stats.estimated_memory_usage() < 10 * 1024 * 1024); // Less than 10MB
    }
}