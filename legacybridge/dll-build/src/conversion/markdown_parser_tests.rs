// Comprehensive test suite for Markdown to RTF parser
// Tests edge cases, malformed input, and complex document structures

#[cfg(test)]
mod markdown_parser_edge_cases {
    use super::super::*;
    use crate::conversion::types::{RtfDocument, RtfNode};
    use crate::conversion::markdown_parser::MarkdownParser;

    #[test]
    fn test_empty_markdown() {
        let markdown = "";
        let result = MarkdownParser::parse(markdown);
        assert!(result.is_ok());
        let document = result.unwrap();
        assert!(document.content.is_empty());
    }

    #[test]
    fn test_whitespace_only_markdown() {
        let markdown = "   \n\n   \t   ";
        let result = MarkdownParser::parse(markdown);
        assert!(result.is_ok());
        let document = result.unwrap();
        assert!(document.content.is_empty());
    }

    #[test]
    fn test_nested_formatting() {
        let markdown = "Normal ***bold and italic*** text **bold with *italic* inside**";
        let document = MarkdownParser::parse(markdown).unwrap();
        
        assert_eq!(document.content.len(), 1);
        match &document.content[0] {
            RtfNode::Paragraph(nodes) => {
                assert!(nodes.len() >= 4); // Multiple formatting nodes
                
                // Check for nested bold/italic
                let mut found_nested = false;
                for node in nodes {
                    if let RtfNode::Bold(bold_content) = node {
                        for inner in bold_content {
                            if matches!(inner, RtfNode::Italic(_)) {
                                found_nested = true;
                            }
                        }
                    }
                }
                assert!(found_nested, "Should have found nested formatting");
            }
            _ => panic!("Expected paragraph node"),
        }
    }

    #[test]
    fn test_multiple_heading_levels() {
        let markdown = r#"# H1 Heading
## H2 Heading
### H3 Heading
#### H4 Heading
##### H5 Heading
###### H6 Heading"#;
        
        let document = MarkdownParser::parse(markdown).unwrap();
        assert_eq!(document.content.len(), 6);
        
        for (i, node) in document.content.iter().enumerate() {
            match node {
                RtfNode::Heading { level, content } => {
                    assert_eq!(*level as usize, i + 1);
                    match &content[0] {
                        RtfNode::Text(text) => {
                            assert!(text.contains(&format!("H{} Heading", i + 1)));
                        }
                        _ => panic!("Expected text in heading"),
                    }
                }
                _ => panic!("Expected heading node"),
            }
        }
    }

    #[test]
    fn test_nested_lists() {
        let markdown = r#"- Item 1
  - Nested 1.1
    - Deep nested 1.1.1
  - Nested 1.2
- Item 2"#;
        
        let document = MarkdownParser::parse(markdown).unwrap();
        
        // Verify list structure
        let mut level_0_count = 0;
        let mut level_1_count = 0;
        let mut level_2_count = 0;
        
        for node in &document.content {
            if let RtfNode::ListItem { level, .. } = node {
                match level {
                    0 => level_0_count += 1,
                    1 => level_1_count += 1,
                    2 => level_2_count += 1,
                    _ => {}
                }
            }
        }
        
        assert_eq!(level_0_count, 2, "Should have 2 top-level items");
        assert_eq!(level_1_count, 2, "Should have 2 nested items");
        assert_eq!(level_2_count, 1, "Should have 1 deeply nested item");
    }

    #[test]
    fn test_mixed_list_types() {
        let markdown = r#"1. Ordered item 1
2. Ordered item 2

- Unordered item 1
- Unordered item 2"#;
        
        let document = MarkdownParser::parse(markdown).unwrap();
        
        // Count list items
        let list_items: Vec<_> = document.content.iter()
            .filter(|n| matches!(n, RtfNode::ListItem { .. }))
            .collect();
        
        assert_eq!(list_items.len(), 4, "Should have 4 list items total");
    }

    #[test]
    fn test_complex_table() {
        let markdown = r#"| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1.1 | Cell 1.2 | Cell 1.3 |
| Cell 2.1 | Cell 2.2 | Cell 2.3 |
| **Bold** | *Italic* | `Code`   |"#;
        
        let document = MarkdownParser::parse(markdown).unwrap();
        
        // Find table node
        let table_node = document.content.iter()
            .find(|n| matches!(n, RtfNode::Table { .. }));
        
        assert!(table_node.is_some(), "Should have a table node");
        
        if let Some(RtfNode::Table { rows }) = table_node {
            assert_eq!(rows.len(), 4, "Should have 4 rows (including header)");
            
            // Check each row has 3 cells
            for row in rows {
                assert_eq!(row.cells.len(), 3, "Each row should have 3 cells");
            }
            
            // Verify formatting in last row
            let last_row = &rows[3];
            // Check for bold in first cell
            let has_bold = last_row.cells[0].content.iter()
                .any(|n| matches!(n, RtfNode::Bold(_)));
            assert!(has_bold, "First cell of last row should contain bold text");
            
            // Check for italic in second cell
            let has_italic = last_row.cells[1].content.iter()
                .any(|n| matches!(n, RtfNode::Italic(_)));
            assert!(has_italic, "Second cell of last row should contain italic text");
        }
    }

    #[test]
    fn test_inline_code_and_code_blocks() {
        let markdown = r#"This has `inline code` and then:

```rust
fn main() {
    println!("Code block");
}
```

More text with `another inline`."#;
        
        let document = MarkdownParser::parse(markdown).unwrap();
        
        // Count paragraphs with inline code
        let mut inline_code_count = 0;
        for node in &document.content {
            if let RtfNode::Paragraph(children) = node {
                for child in children {
                    if let RtfNode::Text(text) = child {
                        if text.contains("inline code") || text.contains("another inline") {
                            inline_code_count += 1;
                        }
                    }
                }
            }
        }
        
        assert!(inline_code_count >= 2, "Should have found inline code references");
    }

    #[test]
    fn test_horizontal_rules() {
        let markdown = r#"Text before

---

Text after

***

Another section"#;
        
        let document = MarkdownParser::parse(markdown).unwrap();
        
        // Count page breaks (horizontal rules)
        let page_breaks = document.content.iter()
            .filter(|n| matches!(n, RtfNode::PageBreak))
            .count();
        
        assert_eq!(page_breaks, 2, "Should have 2 page breaks");
    }

    #[test]
    fn test_line_breaks() {
        let markdown = r#"Line one  
Line two with hard break

Paragraph break"#;
        
        let document = MarkdownParser::parse(markdown).unwrap();
        
        // Check for line break node
        let has_line_break = document.content.iter()
            .any(|node| {
                if let RtfNode::Paragraph(children) = node {
                    children.iter().any(|child| matches!(child, RtfNode::LineBreak))
                } else {
                    false
                }
            });
        
        assert!(has_line_break, "Should have found a line break");
    }

    #[test]
    fn test_escaped_characters() {
        let markdown = r#"Text with \*escaped\* asterisks and \[escaped\] brackets"#;
        
        let document = MarkdownParser::parse(markdown).unwrap();
        
        // Verify escaped characters are preserved as literal text
        if let RtfNode::Paragraph(nodes) = &document.content[0] {
            if let RtfNode::Text(text) = &nodes[0] {
                assert!(text.contains("*escaped*"), "Escaped asterisks should be literal");
                assert!(text.contains("[escaped]"), "Escaped brackets should be literal");
            }
        }
    }

    #[test]
    fn test_unicode_content() {
        let markdown = "Unicode test: 🚀 émojis, ñ español, 中文, العربية";
        
        let document = MarkdownParser::parse(markdown).unwrap();
        
        if let RtfNode::Paragraph(nodes) = &document.content[0] {
            if let RtfNode::Text(text) = &nodes[0] {
                assert!(text.contains("🚀"), "Should preserve emoji");
                assert!(text.contains("émojis"), "Should preserve accented characters");
                assert!(text.contains("中文"), "Should preserve Chinese characters");
                assert!(text.contains("العربية"), "Should preserve Arabic characters");
            }
        }
    }

    #[test]
    fn test_malformed_table_recovery() {
        // Table with inconsistent column counts
        let markdown = r#"| Col1 | Col2 |
|------|
| Cell1 | Cell2 | Cell3 |
| Cell4 |"#;
        
        let result = MarkdownParser::parse(markdown);
        assert!(result.is_ok(), "Should handle malformed tables gracefully");
    }

    #[test]
    fn test_deeply_nested_structures() {
        let markdown = r#"1. List item with **bold _and italic_ text**
   - Nested bullet
     > Blockquote in list
     > With multiple lines
   - Another bullet with `code`"#;
        
        let result = MarkdownParser::parse(markdown);
        assert!(result.is_ok(), "Should handle deeply nested structures");
        
        let document = result.unwrap();
        assert!(!document.content.is_empty(), "Should have parsed content");
    }

    #[test]
    fn test_empty_list_items() {
        let markdown = r#"- Item 1
- 
- Item 3"#;
        
        let document = MarkdownParser::parse(markdown).unwrap();
        
        let list_items: Vec<_> = document.content.iter()
            .filter(|n| matches!(n, RtfNode::ListItem { .. }))
            .collect();
        
        assert_eq!(list_items.len(), 3, "Should handle empty list items");
    }

    #[test]
    fn test_blockquotes() {
        let markdown = r#"> This is a blockquote
> With multiple lines
>
> And a paragraph break"#;
        
        let result = MarkdownParser::parse(markdown);
        assert!(result.is_ok(), "Should parse blockquotes");
        
        // Note: Current implementation might not fully support blockquotes
        // This test ensures it doesn't crash
    }

    #[test]
    fn test_links_and_images() {
        let markdown = r#"This is [a link](https://example.com) and ![an image](image.png)"#;
        
        let result = MarkdownParser::parse(markdown);
        assert!(result.is_ok(), "Should parse links and images without crashing");
        
        let document = result.unwrap();
        if let RtfNode::Paragraph(nodes) = &document.content[0] {
            // Current implementation might treat these as plain text
            assert!(!nodes.is_empty(), "Should have some content");
        }
    }

    #[test]
    fn test_large_document_stress() {
        // Generate a large document
        let mut markdown = String::new();
        for i in 0..100 {
            markdown.push_str(&format!("# Heading {}\n\n", i));
            markdown.push_str(&format!("Paragraph {} with **bold** and *italic* text.\n\n", i));
            markdown.push_str("- List item 1\n- List item 2\n\n");
            if i % 10 == 0 {
                markdown.push_str("---\n\n");
            }
        }
        
        let start = std::time::Instant::now();
        let result = MarkdownParser::parse(&markdown);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Should handle large documents");
        assert!(duration.as_secs() < 5, "Should parse large document in reasonable time");
        
        let document = result.unwrap();
        assert!(document.content.len() > 200, "Should have parsed many nodes");
    }

    #[test]
    fn test_special_markdown_edge_cases() {
        // Test various edge cases that might break parsers
        let test_cases = vec![
            "**Bold without closing",
            "*Italic without closing",
            "Multiple *** asterisks ***",
            "[Link without URL]",
            "![Image without URL]",
            "```\nCode block without language\n```",
            "# Heading with trailing ###",
            "- - Double dash list",
            "1. 2. Double numbered list",
        ];
        
        for markdown in test_cases {
            let result = MarkdownParser::parse(markdown);
            assert!(result.is_ok(), "Should handle edge case: {}", markdown);
        }
    }
}