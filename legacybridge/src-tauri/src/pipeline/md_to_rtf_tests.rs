// Comprehensive MD→RTF pipeline integration tests
// Tests the complete conversion pipeline with various configurations

#[cfg(test)]
mod md_to_rtf_pipeline_tests {
    use super::super::*;
    use crate::pipeline::{
        convert_markdown_to_rtf_with_pipeline, PipelineConfig, ValidationLevel,
    };

    #[test]
    fn test_basic_md_to_rtf_conversion() {
        let markdown = r#"# Test Document

This is a **bold** paragraph with *italic* text.

## Lists

- Item 1
- Item 2
  - Nested item

## Table

| Column 1 | Column 2 |
|----------|----------|
| Cell 1   | Cell 2   |"#;

        let (rtf, context) = convert_markdown_to_rtf_with_pipeline(markdown, None).unwrap();
        
        // Verify RTF structure
        assert!(rtf.starts_with("{\\rtf1\\ansi"));
        assert!(rtf.ends_with('}'));
        
        // Verify content preservation
        assert!(rtf.contains("Test Document"));
        assert!(rtf.contains("{\\b bold}"));
        assert!(rtf.contains("{\\i italic}"));
        assert!(rtf.contains("\\bullet"));
        assert!(rtf.contains("\\trowd")); // Table
        
        // Verify no errors
        assert!(context.validation_results.iter().all(|r| r.level != ValidationLevel::Error));
    }

    #[test]
    fn test_md_to_rtf_with_strict_validation() {
        let config = PipelineConfig {
            strict_validation: true,
            auto_recovery: false,
            template: None,
            preserve_formatting: true,
            legacy_mode: false,
        };

        let markdown = "# Valid Document\n\nThis is valid markdown.";
        let result = convert_markdown_to_rtf_with_pipeline(markdown, Some(config.clone()));
        assert!(result.is_ok(), "Valid markdown should pass strict validation");

        // Test empty content
        let empty_result = convert_markdown_to_rtf_with_pipeline("", Some(config));
        assert!(empty_result.is_err(), "Empty content should fail strict validation");
    }

    #[test]
    fn test_md_to_rtf_with_templates() {
        let templates = vec!["minimal", "professional", "academic"];
        let markdown = "# Document\n\nContent with **formatting**.";

        for template in templates {
            let config = PipelineConfig {
                strict_validation: false,
                auto_recovery: true,
                template: Some(template.to_string()),
                preserve_formatting: true,
                legacy_mode: false,
            };

            let (rtf, _) = convert_markdown_to_rtf_with_pipeline(markdown, Some(config)).unwrap();
            
            match template {
                "minimal" => {
                    assert!(!rtf.contains("\\colortbl"), "Minimal template should not have color table");
                }
                "professional" => {
                    assert!(rtf.contains("\\margl1440"), "Professional should have margins");
                }
                "academic" => {
                    assert!(rtf.contains("\\margl1800"), "Academic should have wider left margin");
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_complex_markdown_conversion() {
        let markdown = r#"# Main Title

## Introduction

This document demonstrates **various** *formatting* options including:

1. Numbered lists
2. With multiple items
   a. And sub-items
   b. Like this

### Code Examples

Here's some `inline code` and a code block:

```rust
fn main() {
    println!("Hello, RTF!");
}
```

### Tables

| Feature | Supported | Notes |
|---------|-----------|-------|
| **Bold** | ✓ | Full support |
| *Italic* | ✓ | Full support |
| `Code` | Partial | As plain text |

---

## Conclusion

This document has been converted from Markdown to RTF."#;

        let (rtf, context) = convert_markdown_to_rtf_with_pipeline(markdown, None).unwrap();
        
        // Verify complex structures
        assert!(rtf.contains("Main Title"));
        assert!(rtf.contains("Introduction"));
        assert!(rtf.contains("Code Examples"));
        assert!(rtf.contains("\\trowd"), "Should contain table");
        assert!(rtf.contains("\\page"), "Should contain page break from ---");
        
        // Check for successful conversion
        assert!(context.rtf.is_some());
        assert!(context.document.is_some());
    }

    #[test]
    fn test_unicode_handling_in_pipeline() {
        let markdown = r#"# Unicode Test 🚀

## International Content

- English: Hello World
- Spanish: ¡Hola Mundo!
- Chinese: 你好世界
- Arabic: مرحبا بالعالم
- Emoji: 😀 🎉 🚀"#;

        let (rtf, _) = convert_markdown_to_rtf_with_pipeline(markdown, None).unwrap();
        
        // RTF should handle Unicode (even if escaped)
        assert!(rtf.contains("Unicode Test"));
        assert!(rtf.contains("International Content"));
        // Unicode characters should be encoded or preserved
        assert!(rtf.len() > markdown.len(), "RTF should be longer due to encoding");
    }

    #[test]
    fn test_error_recovery_mode() {
        let config = PipelineConfig {
            strict_validation: false,
            auto_recovery: true,
            template: None,
            preserve_formatting: true,
            legacy_mode: false,
        };

        // Markdown that might cause issues
        let markdown = r#"# Heading

**Unclosed bold

*Unclosed italic

[Link without URL]

![Image without URL]"#;

        let result = convert_markdown_to_rtf_with_pipeline(markdown, Some(config));
        assert!(result.is_ok(), "Auto-recovery should handle malformed markdown");
        
        let (rtf, context) = result.unwrap();
        assert!(!context.recovery_actions.is_empty(), "Should have recovery actions");
        assert!(rtf.contains("Heading"), "Should preserve valid content");
    }

    #[test]
    fn test_legacy_mode() {
        let config = PipelineConfig {
            strict_validation: false,
            auto_recovery: true,
            template: None,
            preserve_formatting: true,
            legacy_mode: true,
        };

        let markdown = "# Simple Document\n\nBasic content.";
        let (rtf, _) = convert_markdown_to_rtf_with_pipeline(markdown, Some(config)).unwrap();
        
        // Legacy mode should produce compatible RTF
        assert!(rtf.starts_with("{\\rtf1"));
        assert!(rtf.contains("Simple Document"));
    }

    #[test]
    fn test_preserve_formatting_flag() {
        let markdown = "# Document\n\n**Bold** and *italic* text.";
        
        // Test with formatting preserved
        let config_preserve = PipelineConfig {
            strict_validation: false,
            auto_recovery: true,
            template: None,
            preserve_formatting: true,
            legacy_mode: false,
        };
        
        let (rtf_preserve, _) = convert_markdown_to_rtf_with_pipeline(markdown, Some(config_preserve)).unwrap();
        assert!(rtf_preserve.contains("{\\b "), "Should preserve bold formatting");
        assert!(rtf_preserve.contains("{\\i "), "Should preserve italic formatting");
        
        // Test without formatting preserved (using minimal template)
        let config_minimal = PipelineConfig {
            strict_validation: false,
            auto_recovery: true,
            template: Some("minimal".to_string()),
            preserve_formatting: false,
            legacy_mode: false,
        };
        
        let (rtf_minimal, _) = convert_markdown_to_rtf_with_pipeline(markdown, Some(config_minimal)).unwrap();
        // Minimal template still preserves basic formatting
        assert!(rtf_minimal.contains("{\\b "), "Minimal still has bold");
    }

    #[test]
    fn test_nested_list_conversion() {
        let markdown = r#"# Lists

1. First item
   - Nested bullet
   - Another bullet
2. Second item
   1. Nested number
   2. Another number
      - Deep nesting"#;

        let (rtf, _) = convert_markdown_to_rtf_with_pipeline(markdown, None).unwrap();
        
        // Verify list structure is preserved
        assert!(rtf.contains("\\bullet"), "Should have bullet points");
        assert!(rtf.contains("First item"));
        assert!(rtf.contains("Nested bullet"));
        assert!(rtf.contains("Deep nesting"));
    }

    #[test]
    fn test_performance_large_markdown() {
        // Generate a large markdown document
        let mut markdown = String::new();
        for i in 0..100 {
            markdown.push_str(&format!("# Section {}\n\n", i));
            markdown.push_str("This is a paragraph with **bold** and *italic* text.\n\n");
            markdown.push_str("- List item 1\n- List item 2\n- List item 3\n\n");
            
            if i % 10 == 0 {
                markdown.push_str("| Col1 | Col2 | Col3 |\n");
                markdown.push_str("|------|------|------|\n");
                markdown.push_str("| A    | B    | C    |\n\n");
            }
        }

        let start = std::time::Instant::now();
        let result = convert_markdown_to_rtf_with_pipeline(&markdown, None);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Should handle large documents");
        assert!(duration.as_secs() < 5, "Should complete in reasonable time");
        
        let (rtf, _) = result.unwrap();
        assert!(rtf.len() > markdown.len(), "RTF should be larger than markdown");
    }

    #[test]
    fn test_validation_messages() {
        let config = PipelineConfig {
            strict_validation: true,
            auto_recovery: false,
            template: None,
            preserve_formatting: true,
            legacy_mode: false,
        };

        // Test various validation scenarios
        let test_cases = vec![
            ("", "Empty content"),
            ("   \n\n   ", "Whitespace only"),
        ];

        for (markdown, description) in test_cases {
            let result = convert_markdown_to_rtf_with_pipeline(markdown, Some(config.clone()));
            assert!(result.is_err(), "{} should fail validation", description);
        }
    }

    #[test]
    fn test_round_trip_stability() {
        // Note: This tests that MD→RTF produces valid RTF, not full round-trip
        let markdown = r#"# Test Document

## Formatting

This has **bold**, *italic*, and ***both***.

## Lists

1. One
2. Two
   - Two A
   - Two B

## Table

| Header 1 | Header 2 |
|----------|----------|
| Data 1   | Data 2   |"#;

        let (rtf, context) = convert_markdown_to_rtf_with_pipeline(markdown, None).unwrap();
        
        // Verify document structure was parsed
        assert!(context.document.is_some());
        let doc = context.document.unwrap();
        
        // Count different node types
        let heading_count = doc.content.iter()
            .filter(|n| matches!(n, crate::conversion::types::RtfNode::Heading { .. }))
            .count();
        assert_eq!(heading_count, 3, "Should have 3 headings");
        
        // Verify RTF is well-formed
        let open_braces = rtf.matches('{').count();
        let close_braces = rtf.matches('}').count();
        assert_eq!(open_braces, close_braces, "RTF should have balanced braces");
    }
}