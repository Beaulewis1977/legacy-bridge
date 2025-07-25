// Pooled Markdown Generator - Uses memory pools to reduce allocation overhead

use super::types::{ConversionResult, RtfDocument, RtfNode, TableRow, TableCell};
use super::memory_pools::{CONVERSION_POOLS, PooledStringBuilder};

/// Markdown Generator with memory pool integration
pub struct PooledMarkdownGenerator;

impl PooledMarkdownGenerator {
    /// Generate Markdown from an RTF document using memory pools
    pub fn generate(document: &RtfDocument) -> ConversionResult<String> {
        let mut output = PooledStringBuilder::with_capacity(4096);
        let mut first_paragraph = true;

        for node in &document.content {
            if !first_paragraph {
                output.push_str("\n\n");
            }
            first_paragraph = false;

            Self::generate_node(node, &mut output)?;
        }

        let result = output.finish();
        Ok(result.trim().to_string())
    }

    /// Generate content for a single node using pooled string builder
    fn generate_node(node: &RtfNode, output: &mut PooledStringBuilder) -> ConversionResult<()> {
        match node {
            RtfNode::Text(text) => {
                output.push_str(&Self::escape_markdown(text));
            }
            RtfNode::Paragraph(nodes) => {
                for node in nodes {
                    Self::generate_node(node, output)?;
                }
            }
            RtfNode::Bold(nodes) => {
                output.push_str("**");
                for node in nodes {
                    Self::generate_node(node, output)?;
                }
                output.push_str("**");
            }
            RtfNode::Italic(nodes) => {
                output.push('*');
                for node in nodes {
                    Self::generate_node(node, output)?;
                }
                output.push('*');
            }
            RtfNode::Underline(nodes) => {
                // Markdown doesn't have native underline, use HTML
                output.push_str("<u>");
                for node in nodes {
                    Self::generate_node(node, output)?;
                }
                output.push_str("</u>");
            }
            RtfNode::Heading { level, content } => {
                for _ in 0..*level {
                    output.push('#');
                }
                output.push(' ');
                for node in content {
                    Self::generate_node(node, output)?;
                }
            }
            RtfNode::ListItem { level: _, content } => {
                output.push_str("- ");
                for node in content {
                    Self::generate_node(node, output)?;
                }
            }
            RtfNode::Table { rows } => {
                Self::generate_table(rows, output)?;
            }
            RtfNode::LineBreak => {
                output.push_str("  \n");
            }
            RtfNode::PageBreak => {
                output.push_str("\n---\n");
            }
        }

        Ok(())
    }

    /// Generate a markdown table using pooled string builder
    fn generate_table(rows: &[TableRow], output: &mut PooledStringBuilder) -> ConversionResult<()> {
        if rows.is_empty() {
            return Ok(());
        }

        // Generate table rows
        for (i, row) in rows.iter().enumerate() {
            output.push('|');
            for cell in &row.cells {
                output.push(' ');
                Self::generate_cell(cell, output)?;
                output.push_str(" |");
            }
            output.push('\n');

            // Add separator after header row
            if i == 0 {
                output.push('|');
                for _ in &row.cells {
                    output.push_str(" --- |");
                }
                output.push('\n');
            }
        }

        Ok(())
    }

    /// Generate table cell content using pooled string builder
    fn generate_cell(cell: &TableCell, output: &mut PooledStringBuilder) -> ConversionResult<()> {
        for node in &cell.content {
            Self::generate_node(node, output)?;
        }
        Ok(())
    }

    /// Escape special markdown characters using pooled string
    fn escape_markdown(text: &str) -> String {
        // For small strings, it's more efficient to use a pooled string
        if text.len() < 64 && !text.chars().any(|ch| matches!(ch, '*' | '_' | '[' | ']' | '(' | ')' | '#' | '!' | '`' | '\\')) {
            return text.to_string();
        }
        
        let mut result = CONVERSION_POOLS.get_string_buffer(text.len() * 2);
        
        for ch in text.chars() {
            match ch {
                '*' | '_' | '[' | ']' | '(' | ')' | '#' | '!' | '`' | '\\' => {
                    result.push('\\');
                    result.push(ch);
                }
                _ => result.push(ch),
            }
        }
        
        std::mem::take(&mut *result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::types::DocumentMetadata;

    #[test]
    fn test_pooled_simple_text() {
        let doc = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Paragraph(vec![RtfNode::Text(
                "Hello World".to_string(),
            )])],
        };

        let result = PooledMarkdownGenerator::generate(&doc).unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_pooled_bold_text() {
        let doc = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Paragraph(vec![
                RtfNode::Text("Normal ".to_string()),
                RtfNode::Bold(vec![RtfNode::Text("Bold".to_string())]),
                RtfNode::Text(" text".to_string()),
            ])],
        };

        let result = PooledMarkdownGenerator::generate(&doc).unwrap();
        assert_eq!(result, "Normal **Bold** text");
    }

    #[test]
    fn test_pooled_escape_markdown() {
        let doc = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Paragraph(vec![RtfNode::Text(
                "Text with *asterisks* and _underscores_".to_string(),
            )])],
        };

        let result = PooledMarkdownGenerator::generate(&doc).unwrap();
        assert_eq!(result, "Text with \\*asterisks\\* and \\_underscores\\_");
    }
    
    #[test]
    fn test_memory_pool_reuse() {
        // Get initial pool stats
        let initial_stats = CONVERSION_POOLS.get_stats();
        
        // Generate multiple documents
        for i in 0..10 {
            let doc = RtfDocument {
                metadata: DocumentMetadata::default(),
                content: vec![RtfNode::Paragraph(vec![
                    RtfNode::Text(format!("Document {}", i)),
                    RtfNode::Bold(vec![RtfNode::Text("Bold".to_string())]),
                ])],
            };
            
            let _ = PooledMarkdownGenerator::generate(&doc).unwrap();
        }
        
        // Check that pools have been populated
        let final_stats = CONVERSION_POOLS.get_stats();
        assert!(final_stats.total_pooled_objects() > initial_stats.total_pooled_objects());
    }
}