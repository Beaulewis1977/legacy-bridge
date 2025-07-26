// Validation script for MD→RTF conversion
// Demonstrates and validates the conversion pipeline with real-world examples

use legacybridge::conversion::{markdown_to_rtf, MarkdownParser, RtfGenerator};
use legacybridge::pipeline::{convert_markdown_to_rtf_with_pipeline, PipelineConfig};
use std::fs;
use std::time::Instant;

fn main() {
    println!("=== MD→RTF Conversion Validation Suite ===\n");

    // Test cases with increasing complexity
    let test_cases = vec![
        (
            "simple",
            r#"# Simple Document

This is a basic paragraph with **bold** and *italic* text."#,
        ),
        (
            "lists",
            r#"# Document with Lists

## Unordered List
- First item
- Second item
  - Nested item
  - Another nested
- Third item

## Ordered List
1. Step one
2. Step two
   1. Sub-step A
   2. Sub-step B
3. Step three"#,
        ),
        (
            "tables",
            r#"# Document with Tables

## Simple Table

| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1-1 | Cell 1-2 | Cell 1-3 |
| Cell 2-1 | Cell 2-2 | Cell 2-3 |

## Complex Table

| Feature | Status | Notes |
|---------|--------|-------|
| **Bold** | ✓ | Fully supported |
| *Italic* | ✓ | Fully supported |
| `Code` | ~ | Rendered as plain text |
| ~~Strike~~ | ✗ | Not yet supported |"#,
        ),
        (
            "complex",
            r#"# Complex Technical Document

## Introduction

This document demonstrates the **full capabilities** of the MD→RTF conversion pipeline.

### Features Supported

1. **Text Formatting**
   - Bold text using `**bold**`
   - Italic text using `*italic*`
   - Combined ***bold and italic***
   
2. **Document Structure**
   - Multiple heading levels
   - Paragraphs with proper spacing
   - Line breaks and page breaks

### Code Examples

Here's some `inline code` within a paragraph.

```rust
fn example() {
    println!("Code blocks are converted to plain text");
}
```

### Advanced Features

#### Nested Lists with Formatting

1. First item with **bold text**
   - Nested bullet with *italic*
     1. Deep nesting with `code`
     2. Another nested item
   - Back to second level
2. Second main item

#### Unicode Support

- English: Hello World! 
- Spanish: ¡Hola Mundo! 
- Chinese: 你好世界
- Arabic: مرحبا بالعالم
- Emoji: 🚀 🎉 ✨

---

## Conclusion

This document has been successfully converted from Markdown to RTF format."#,
        ),
    ];

    // Test each case
    for (name, markdown) in test_cases {
        println!("Testing '{}' conversion...", name);
        
        // Method 1: Direct conversion
        let start = Instant::now();
        match markdown_to_rtf(markdown) {
            Ok(rtf) => {
                let duration = start.elapsed();
                println!("  ✓ Direct conversion: {:?}", duration);
                println!("    Input: {} bytes, Output: {} bytes", markdown.len(), rtf.len());
                
                // Validate RTF structure
                assert!(rtf.starts_with("{\\rtf1"), "RTF should start with proper header");
                assert!(rtf.ends_with('}'), "RTF should end with closing brace");
                
                // Save sample output
                let filename = format!("test_output_{}.rtf", name);
                if let Err(e) = fs::write(&filename, &rtf) {
                    eprintln!("    Failed to save {}: {}", filename, e);
                } else {
                    println!("    Saved to: {}", filename);
                }
            }
            Err(e) => {
                println!("  ✗ Direct conversion failed: {}", e);
            }
        }
        
        // Method 2: Pipeline conversion with configuration
        let config = PipelineConfig {
            strict_validation: true,
            auto_recovery: true,
            template: Some("professional".to_string()),
            preserve_formatting: true,
            legacy_mode: false,
        };
        
        let start = Instant::now();
        match convert_markdown_to_rtf_with_pipeline(markdown, Some(config)) {
            Ok((rtf, context)) => {
                let duration = start.elapsed();
                println!("  ✓ Pipeline conversion: {:?}", duration);
                println!("    Validations: {}, Recovery actions: {}", 
                    context.validation_results.len(),
                    context.recovery_actions.len()
                );
                
                // Save pipeline output
                let filename = format!("test_output_{}_pipeline.rtf", name);
                if let Err(e) = fs::write(&filename, &rtf) {
                    eprintln!("    Failed to save {}: {}", filename, e);
                } else {
                    println!("    Saved to: {}", filename);
                }
            }
            Err(e) => {
                println!("  ✗ Pipeline conversion failed: {}", e);
            }
        }
        
        println!();
    }
    
    // Performance test with large document
    println!("Performance Test: Large Document");
    let mut large_doc = String::new();
    for i in 0..100 {
        large_doc.push_str(&format!("# Section {}\n\n", i));
        large_doc.push_str("This is a paragraph with **bold** and *italic* formatting.\n\n");
        large_doc.push_str("- List item 1\n- List item 2\n- List item 3\n\n");
    }
    
    let start = Instant::now();
    match markdown_to_rtf(&large_doc) {
        Ok(rtf) => {
            let duration = start.elapsed();
            println!("  ✓ Large document ({} KB): {:?}", large_doc.len() / 1024, duration);
            println!("    Output size: {} KB", rtf.len() / 1024);
            
            let throughput = (large_doc.len() as f64 / 1024.0) / duration.as_secs_f64();
            println!("    Throughput: {:.2} KB/s", throughput);
        }
        Err(e) => {
            println!("  ✗ Large document conversion failed: {}", e);
        }
    }
    
    println!("\n=== Validation Complete ===");
    println!("Check the generated .rtf files with a word processor to verify formatting.");
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_validation_suite_runs() {
        // This ensures the validation suite compiles and basic conversions work
        let simple_md = "# Test\n\nSimple content.";
        let result = markdown_to_rtf(simple_md);
        assert!(result.is_ok(), "Basic conversion should succeed");
    }
}