// SIMD-Optimized Conversion Module
// High-performance RTF/Markdown conversion using SIMD instructions

use super::types::{ConversionError, ConversionResult, RtfDocument};
use super::rtf_lexer_simd::tokenize_simd;
use super::rtf_parser_optimized::OptimizedRtfParser;
use super::markdown_parser_optimized::OptimizedMarkdownParser;
use super::markdown_generator::MarkdownGenerator;
use super::rtf_generator::RtfGenerator;
use super::markdown_simd_utils::{SimdMarkdownScanner, SimdUtf8Validator, SimdWhitespaceOps};
use super::input_validation::InputValidator;

/// CPU feature detection for optimal path selection
struct SimdFeatures {
    has_sse2: bool,
    has_sse42: bool,
    has_avx2: bool,
}

impl SimdFeatures {
    fn detect() -> Self {
        Self {
            has_sse2: is_x86_feature_detected!("sse2"),
            has_sse42: is_x86_feature_detected!("sse4.2"),
            has_avx2: is_x86_feature_detected!("avx2"),
        }
    }
}

/// SIMD-optimized RTF to Markdown conversion
pub fn rtf_to_markdown_simd(rtf_content: &str) -> ConversionResult<String> {
    // Validate input
    let validator = InputValidator::new();
    validator.validate_size(rtf_content, "RTF content")?;
    
    // Check CPU features
    let features = SimdFeatures::detect();
    
    if !features.has_sse2 {
        // Fall back to non-SIMD implementation
        return super::rtf_to_markdown(rtf_content);
    }
    
    // Use SIMD-optimized tokenizer
    let tokens = tokenize_simd(rtf_content)?;
    
    // Parse with optimized parser
    let document = OptimizedRtfParser::parse(tokens)?;
    
    // Generate markdown
    let markdown = MarkdownGenerator::generate(&document)?;
    
    // Post-process with SIMD whitespace normalization
    let whitespace_ops = SimdWhitespaceOps::new();
    Ok(whitespace_ops.normalize_whitespace(&markdown))
}

/// SIMD-optimized Markdown to RTF conversion
pub fn markdown_to_rtf_simd(markdown_content: &str) -> ConversionResult<String> {
    // Validate input
    let validator = InputValidator::new();
    validator.validate_size(markdown_content, "Markdown content")?;
    
    // Check CPU features
    let features = SimdFeatures::detect();
    
    if !features.has_sse2 {
        // Fall back to non-SIMD implementation
        return super::markdown_to_rtf(markdown_content);
    }
    
    // Pre-process with SIMD UTF-8 validation
    let utf8_validator = SimdUtf8Validator::new();
    if !utf8_validator.is_valid_utf8(markdown_content.as_bytes()) {
        return Err(ConversionError::ValidationError("Invalid UTF-8 in input".to_string()));
    }
    
    // Parse with optimized parser
    let mut parser = OptimizedMarkdownParser::new();
    let document = parser.parse(markdown_content)?;
    
    // Generate RTF
    let rtf = RtfGenerator::generate(&document)?;
    
    Ok(rtf)
}

/// SIMD-accelerated string preprocessing for better parsing performance
pub fn preprocess_rtf_simd(rtf_content: &str) -> ConversionResult<String> {
    let whitespace_ops = SimdWhitespaceOps::new();
    let normalized = whitespace_ops.normalize_whitespace(rtf_content);
    Ok(normalized)
}

/// SIMD-accelerated markdown preprocessing
pub fn preprocess_markdown_simd(markdown_content: &str) -> ConversionResult<(String, Vec<usize>)> {
    let scanner = SimdMarkdownScanner::new();
    let special_chars = scanner.find_special_chars(markdown_content.as_bytes());
    
    let whitespace_ops = SimdWhitespaceOps::new();
    let normalized = whitespace_ops.normalize_whitespace(markdown_content);
    
    Ok((normalized, special_chars))
}

/// Benchmark utilities for SIMD optimizations
#[cfg(test)]
pub mod benchmarks {
    use super::*;
    use std::time::Instant;
    
    pub struct SimdBenchmark {
        name: String,
        simd_time: f64,
        scalar_time: f64,
        improvement: f64,
    }
    
    impl SimdBenchmark {
        pub fn run_rtf_benchmark(rtf_content: &str) -> Self {
            // Warm up
            let _ = super::super::rtf_to_markdown(rtf_content);
            let _ = rtf_to_markdown_simd(rtf_content);
            
            // Benchmark scalar version
            let scalar_start = Instant::now();
            for _ in 0..10 {
                let _ = super::super::rtf_to_markdown(rtf_content);
            }
            let scalar_time = scalar_start.elapsed().as_secs_f64() / 10.0;
            
            // Benchmark SIMD version
            let simd_start = Instant::now();
            for _ in 0..10 {
                let _ = rtf_to_markdown_simd(rtf_content);
            }
            let simd_time = simd_start.elapsed().as_secs_f64() / 10.0;
            
            let improvement = ((scalar_time - simd_time) / scalar_time) * 100.0;
            
            Self {
                name: "RTF to Markdown".to_string(),
                simd_time,
                scalar_time,
                improvement,
            }
        }
        
        pub fn run_markdown_benchmark(markdown_content: &str) -> Self {
            // Warm up
            let _ = super::super::markdown_to_rtf(markdown_content);
            let _ = markdown_to_rtf_simd(markdown_content);
            
            // Benchmark scalar version
            let scalar_start = Instant::now();
            for _ in 0..10 {
                let _ = super::super::markdown_to_rtf(markdown_content);
            }
            let scalar_time = scalar_start.elapsed().as_secs_f64() / 10.0;
            
            // Benchmark SIMD version
            let simd_start = Instant::now();
            for _ in 0..10 {
                let _ = markdown_to_rtf_simd(markdown_content);
            }
            let simd_time = simd_start.elapsed().as_secs_f64() / 10.0;
            
            let improvement = ((scalar_time - simd_time) / scalar_time) * 100.0;
            
            Self {
                name: "Markdown to RTF".to_string(),
                simd_time,
                scalar_time,
                improvement,
            }
        }
        
        pub fn report(&self) {
            println!("Benchmark: {}", self.name);
            println!("  Scalar time: {:.6}s", self.scalar_time);
            println!("  SIMD time: {:.6}s", self.simd_time);
            println!("  Improvement: {:.1}%", self.improvement);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simd_rtf_to_markdown() {
        let rtf = r"{\rtf1\ansi\deff0 {\fonttbl{\f0 Times New Roman;}} Hello World\par}";
        let result = rtf_to_markdown_simd(rtf);
        assert!(result.is_ok());
        let markdown = result.unwrap();
        assert!(markdown.contains("Hello World"));
    }
    
    #[test]
    fn test_simd_markdown_to_rtf() {
        let markdown = "# Hello World\n\nThis is a **test** document.";
        let result = markdown_to_rtf_simd(markdown);
        assert!(result.is_ok());
        let rtf = result.unwrap();
        assert!(rtf.contains("rtf1"));
        assert!(rtf.contains("Hello World"));
    }
    
    #[test]
    fn test_simd_preprocessing() {
        let rtf = "Hello   \t\n\r   World";
        let result = preprocess_rtf_simd(rtf);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello World");
        
        let markdown = "# Test\n\n**Bold** and *italic*";
        let result = preprocess_markdown_simd(markdown);
        assert!(result.is_ok());
        let (normalized, special_chars) = result.unwrap();
        assert!(!special_chars.is_empty());
    }
    
    #[test]
    fn test_simd_performance_improvement() {
        // Generate test document
        let mut rtf = String::from(r"{\rtf1\ansi\deff0 ");
        for i in 0..100 {
            rtf.push_str(&format!(r"\par Paragraph {} with \b bold\b0 and \i italic\i0 text. ", i));
        }
        rtf.push('}');
        
        let benchmark = benchmarks::SimdBenchmark::run_rtf_benchmark(&rtf);
        benchmark.report();
        
        // We expect at least 20% improvement with SIMD
        assert!(benchmark.improvement >= 20.0, 
                "SIMD improvement was only {:.1}%", benchmark.improvement);
    }
}