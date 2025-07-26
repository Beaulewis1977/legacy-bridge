// RTF Generator - Converts RTF document structure to RTF format

use super::types::{ConversionResult, RtfDocument, RtfNode, TableCell, TableRow};

/// RTF Generator
pub struct RtfGenerator;

impl RtfGenerator {
    /// Generate RTF from a document structure
    pub fn generate(document: &RtfDocument) -> ConversionResult<String> {
        let mut output = String::new();
        
        // RTF header
        output.push_str("{\\rtf1\\ansi\\deff0");
        
        // Font table
        output.push_str("{\\fonttbl{\\f0\\froman\\fcharset0 Times New Roman;}{\\f1\\fswiss\\fcharset0 Arial;}}");
        
        // Color table (basic colors)
        output.push_str("{\\colortbl;\\red0\\green0\\blue0;\\red255\\green0\\blue0;\\red0\\green255\\blue0;\\red0\\green0\\blue255;}");
        
        // Document content
        for (i, node) in document.content.iter().enumerate() {
            if i > 0 {
                output.push_str("\\par ");
            }
            Self::generate_node(node, &mut output, 0)?;
        }
        
        output.push('}');
        Ok(output)
    }

    /// Generate RTF for a single node
    fn generate_node(node: &RtfNode, output: &mut String, depth: usize) -> ConversionResult<()> {
        match node {
            RtfNode::Text(text) => {
                output.push_str(&Self::escape_rtf_text(text));
            }
            RtfNode::Paragraph(nodes) => {
                for (i, node) in nodes.iter().enumerate() {
                    if i > 0 {
                        output.push(' ');
                    }
                    Self::generate_node(node, output, depth)?;
                }
            }
            RtfNode::Bold(nodes) => {
                output.push_str("{\\b ");
                for node in nodes {
                    Self::generate_node(node, output, depth + 1)?;
                }
                output.push('}');
            }
            RtfNode::Italic(nodes) => {
                output.push_str("{\\i ");
                for node in nodes {
                    Self::generate_node(node, output, depth + 1)?;
                }
                output.push('}');
            }
            RtfNode::Underline(nodes) => {
                output.push_str("{\\ul ");
                for node in nodes {
                    Self::generate_node(node, output, depth + 1)?;
                }
                output.push('}');
            }
            RtfNode::Heading { level, content } => {
                // RTF headings using font size
                let font_size = match level {
                    1 => 24,  // 24pt for H1
                    2 => 20,  // 20pt for H2
                    3 => 16,  // 16pt for H3
                    4 => 14,  // 14pt for H4
                    5 => 12,  // 12pt for H5
                    _ => 12,  // 12pt for H6 and beyond
                };
                
                output.push_str(&format!("{{\\b\\fs{} ", font_size * 2)); // RTF font size is in half-points
                for node in content {
                    Self::generate_node(node, output, depth + 1)?;
                }
                output.push('}');
            }
            RtfNode::ListItem { level, content } => {
                // RTF list formatting with indentation
                let indent = 720 * (*level as i32 + 1); // 720 twips = 0.5 inch
                output.push_str(&format!("{{\\pard\\li{}\\fi-360 ", indent));
                output.push_str("\\bullet\\tab ");
                
                for node in content {
                    Self::generate_node(node, output, depth + 1)?;
                }
                output.push_str("\\par}");
            }
            RtfNode::Table { rows } => {
                Self::generate_table(rows, output)?;
            }
            RtfNode::LineBreak => {
                output.push_str("\\line ");
            }
            RtfNode::PageBreak => {
                output.push_str("\\page ");
            }
        }
        Ok(())
    }

    /// Generate RTF table
    fn generate_table(rows: &[TableRow], output: &mut String) -> ConversionResult<()> {
        if rows.is_empty() {
            return Ok(());
        }

        for row in rows {
            // Table row definition
            output.push_str("{\\trowd\\trgaph108\\trleft-108");
            
            // Define cell widths (distribute evenly)
            let cell_count = row.cells.len();
            if cell_count > 0 {
                let cell_width = 9360 / cell_count; // Total width distributed evenly
                let mut cell_pos = 0;
                
                for _ in &row.cells {
                    cell_pos += cell_width;
                    output.push_str(&format!("\\cellx{}", cell_pos));
                }
            }
            
            // Table cells content
            for cell in &row.cells {
                output.push_str("{\\pard\\intbl ");
                Self::generate_cell(cell, output)?;
                output.push_str("\\cell}");
            }
            
            // End row
            output.push_str("\\row}");
        }

        Ok(())
    }

    /// Generate content for a table cell
    fn generate_cell(cell: &TableCell, output: &mut String) -> ConversionResult<()> {
        for (i, node) in cell.content.iter().enumerate() {
            if i > 0 {
                output.push(' ');
            }
            Self::generate_node(node, output, 0)?;
        }
        Ok(())
    }

    /// Escape special RTF characters
    fn escape_rtf_text(text: &str) -> String {
        text.chars()
            .map(|ch| match ch {
                '\\' => "\\\\".to_string(),
                '{' => "\\{".to_string(),
                '}' => "\\}".to_string(),
                '\n' => "\\par ".to_string(),
                '\r' => String::new(), // Skip carriage returns
                c if c as u32 > 127 => {
                    // Unicode characters
                    format!("\\u{}?", c as u32)
                }
                c => c.to_string(),
            })
            .collect()
    }

    /// Generate RTF with template support
    pub fn generate_with_template(
        document: &RtfDocument, 
        template_name: Option<&str>
    ) -> ConversionResult<String> {
        match template_name {
            Some("minimal") => Self::generate_minimal(document),
            Some("professional") => Self::generate_professional(document),
            Some("academic") => Self::generate_academic(document),
            _ => Self::generate(document),
        }
    }

    /// Generate minimal RTF (basic formatting only)
    fn generate_minimal(document: &RtfDocument) -> ConversionResult<String> {
        let mut output = String::new();
        
        // Minimal RTF header
        output.push_str("{\\rtf1\\ansi\\deff0{\\fonttbl{\\f0 Times New Roman;}}");
        
        // Simple content without complex formatting
        for (i, node) in document.content.iter().enumerate() {
            if i > 0 {
                output.push_str("\\par ");
            }
            Self::generate_node_minimal(node, &mut output)?;
        }
        
        output.push('}');
        Ok(output)
    }

    /// Generate professional RTF (enhanced formatting)
    fn generate_professional(document: &RtfDocument) -> ConversionResult<String> {
        let mut output = String::new();
        
        // Professional RTF header with extended formatting
        output.push_str("{\\rtf1\\ansi\\deff0");
        output.push_str("{\\fonttbl{\\f0\\froman Times New Roman;}{\\f1\\fswiss Arial;}{\\f2\\fmodern Courier New;}}");
        output.push_str("{\\colortbl;\\red0\\green0\\blue0;\\red51\\green51\\blue153;\\red153\\green51\\blue51;}");
        output.push_str("\\margl1440\\margr1440\\margt1440\\margb1440"); // 1 inch margins
        
        // Professional styling
        for (i, node) in document.content.iter().enumerate() {
            if i > 0 {
                output.push_str("\\par ");
            }
            Self::generate_node(node, &mut output, 0)?;
        }
        
        output.push('}');
        Ok(output)
    }

    /// Generate academic RTF (citation-ready formatting)
    fn generate_academic(document: &RtfDocument) -> ConversionResult<String> {
        let mut output = String::new();
        
        // Academic RTF header
        output.push_str("{\\rtf1\\ansi\\deff0");
        output.push_str("{\\fonttbl{\\f0\\froman Times New Roman;}{\\f1\\fswiss Arial;}}");
        output.push_str("\\margl1800\\margr1800\\margt1440\\margb1440"); // Wider left margin for binding
        output.push_str("\\sa240\\sl276\\slmult1"); // Paragraph spacing and line height
        
        // Academic content with enhanced formatting
        for (i, node) in document.content.iter().enumerate() {
            if i > 0 {
                output.push_str("\\par ");
            }
            Self::generate_node(node, &mut output, 0)?;
        }
        
        output.push('}');
        Ok(output)
    }

    /// Generate minimal node (simplified formatting)
    fn generate_node_minimal(node: &RtfNode, output: &mut String) -> ConversionResult<()> {
        match node {
            RtfNode::Text(text) => {
                output.push_str(&Self::escape_rtf_text(text));
            }
            RtfNode::Paragraph(nodes) => {
                for node in nodes {
                    Self::generate_node_minimal(node, output)?;
                }
            }
            RtfNode::Bold(nodes) => {
                output.push_str("{\\b ");
                for node in nodes {
                    Self::generate_node_minimal(node, output)?;
                }
                output.push('}');
            }
            RtfNode::Italic(nodes) => {
                output.push_str("{\\i ");
                for node in nodes {
                    Self::generate_node_minimal(node, output)?;
                }
                output.push('}');
            }
            RtfNode::Heading { content, .. } => {
                // Simple bold headings in minimal mode
                output.push_str("{\\b ");
                for node in content {
                    Self::generate_node_minimal(node, output)?;
                }
                output.push('}');
            }
            RtfNode::ListItem { content, .. } => {
                output.push_str("- ");
                for node in content {
                    Self::generate_node_minimal(node, output)?;
                }
            }
            RtfNode::LineBreak => {
                output.push_str("\\line ");
            }
            RtfNode::PageBreak => {
                output.push_str("\\page ");
            }
            _ => {
                // Skip complex formatting in minimal mode
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::types::DocumentMetadata;

    #[test]
    fn test_generate_simple_text() {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Paragraph(vec![RtfNode::Text(
                "Hello World".to_string(),
            )])],
        };

        let rtf = RtfGenerator::generate(&document).unwrap();
        assert!(rtf.contains("Hello World"));
        assert!(rtf.starts_with("{\\rtf1\\ansi"));
        assert!(rtf.ends_with('}'));
    }

    #[test]
    fn test_generate_bold_text() {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Paragraph(vec![
                RtfNode::Text("Normal ".to_string()),
                RtfNode::Bold(vec![RtfNode::Text("Bold".to_string())]),
                RtfNode::Text(" text".to_string()),
            ])],
        };

        let rtf = RtfGenerator::generate(&document).unwrap();
        assert!(rtf.contains("Normal"));
        assert!(rtf.contains("{\\b Bold}"));
        assert!(rtf.contains("text"));
    }

    #[test]
    fn test_generate_heading() {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Heading {
                level: 1,
                content: vec![RtfNode::Text("Main Heading".to_string())],
            }],
        };

        let rtf = RtfGenerator::generate(&document).unwrap();
        assert!(rtf.contains("{\\b\\fs48 Main Heading}"));
    }

    #[test]
    fn test_generate_list() {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![
                RtfNode::ListItem {
                    level: 0,
                    content: vec![RtfNode::Text("Item 1".to_string())],
                },
                RtfNode::ListItem {
                    level: 0,
                    content: vec![RtfNode::Text("Item 2".to_string())],
                },
            ],
        };

        let rtf = RtfGenerator::generate(&document).unwrap();
        assert!(rtf.contains("\\bullet\\tab Item 1"));
        assert!(rtf.contains("\\bullet\\tab Item 2"));
    }

    #[test]
    fn test_escape_special_characters() {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Paragraph(vec![RtfNode::Text(
                "Text with {braces} and \\backslash".to_string(),
            )])],
        };

        let rtf = RtfGenerator::generate(&document).unwrap();
        assert!(rtf.contains("\\{braces\\}"));
        assert!(rtf.contains("\\\\backslash"));
    }

    #[test]
    fn test_generate_with_template() {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Paragraph(vec![RtfNode::Text(
                "Test content".to_string(),
            )])],
        };

        let rtf_minimal = RtfGenerator::generate_with_template(&document, Some("minimal")).unwrap();
        let rtf_professional = RtfGenerator::generate_with_template(&document, Some("professional")).unwrap();
        let rtf_academic = RtfGenerator::generate_with_template(&document, Some("academic")).unwrap();

        assert!(rtf_minimal.contains("Test content"));
        assert!(rtf_professional.contains("Test content"));
        assert!(rtf_academic.contains("Test content"));
        
        // Professional should have margins
        assert!(rtf_professional.contains("\\margl1440"));
        
        // Academic should have wider left margin
        assert!(rtf_academic.contains("\\margl1800"));
    }
}