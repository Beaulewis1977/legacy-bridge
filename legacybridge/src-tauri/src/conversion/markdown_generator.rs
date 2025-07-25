// Markdown Generator - Converts RTF document structure to Markdown

use super::types::{ConversionResult, RtfDocument, RtfNode, TableCell, TableRow};

/// Markdown Generator
pub struct MarkdownGenerator;

impl MarkdownGenerator {
    /// Generate Markdown from an RTF document
    pub fn generate(document: &RtfDocument) -> ConversionResult<String> {
        let mut output = String::new();
        let mut first_paragraph = true;

        for node in &document.content {
            if !first_paragraph {
                output.push_str("\n\n");
            }
            first_paragraph = false;

            Self::generate_node(node, &mut output)?;
        }

        Ok(output.trim().to_string())
    }

    /// Generate Markdown for a single node
    fn generate_node(node: &RtfNode, output: &mut String) -> ConversionResult<()> {
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
                // Add appropriate number of # symbols
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

    /// Generate Markdown table
    fn generate_table(rows: &[TableRow], output: &mut String) -> ConversionResult<()> {
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

            // Add header separator after first row
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

    /// Generate content for a table cell
    fn generate_cell(cell: &TableCell, output: &mut String) -> ConversionResult<()> {
        for node in &cell.content {
            Self::generate_node(node, output)?;
        }
        Ok(())
    }

    /// Escape special Markdown characters
    fn escape_markdown(text: &str) -> String {
        text.chars()
            .map(|ch| match ch {
                '*' | '_' | '[' | ']' | '(' | ')' | '#' | '!' | '`' | '\\' => {
                    format!("\\{}", ch)
                }
                _ => ch.to_string(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_simple_text() {
        let document = RtfDocument {
            metadata: Default::default(),
            content: vec![RtfNode::Paragraph(vec![RtfNode::Text(
                "Hello World".to_string(),
            )])],
        };

        let markdown = MarkdownGenerator::generate(&document).unwrap();
        assert_eq!(markdown, "Hello World");
    }

    #[test]
    fn test_generate_bold_text() {
        let document = RtfDocument {
            metadata: Default::default(),
            content: vec![RtfNode::Paragraph(vec![
                RtfNode::Text("Normal ".to_string()),
                RtfNode::Bold(vec![RtfNode::Text("Bold".to_string())]),
                RtfNode::Text(" text".to_string()),
            ])],
        };

        let markdown = MarkdownGenerator::generate(&document).unwrap();
        assert_eq!(markdown, "Normal **Bold** text");
    }

    #[test]
    fn test_generate_italic_text() {
        let document = RtfDocument {
            metadata: Default::default(),
            content: vec![RtfNode::Paragraph(vec![
                RtfNode::Text("Normal ".to_string()),
                RtfNode::Italic(vec![RtfNode::Text("Italic".to_string())]),
                RtfNode::Text(" text".to_string()),
            ])],
        };

        let markdown = MarkdownGenerator::generate(&document).unwrap();
        assert_eq!(markdown, "Normal *Italic* text");
    }

    #[test]
    fn test_escape_special_characters() {
        let document = RtfDocument {
            metadata: Default::default(),
            content: vec![RtfNode::Paragraph(vec![RtfNode::Text(
                "Text with *asterisks* and _underscores_".to_string(),
            )])],
        };

        let markdown = MarkdownGenerator::generate(&document).unwrap();
        assert_eq!(
            markdown,
            "Text with \\*asterisks\\* and \\_underscores\\_"
        );
    }
}