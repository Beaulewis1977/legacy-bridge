// Secure RTF Generator with enhanced security controls
//
// This module provides a security-hardened RTF generator that prevents
// resource exhaustion and ensures safe output generation.

use super::types::{ConversionError, ConversionResult, RtfDocument, RtfNode, TableCell, TableRow};
use super::security::SecurityLimits;

/// Security-enhanced RTF Generator
pub struct SecureRtfGenerator {
    security_limits: SecurityLimits,
    output_size: usize,
    table_cells_count: usize,
}

impl SecureRtfGenerator {
    /// Generate RTF with security controls
    pub fn generate_with_security(
        document: &RtfDocument,
        security_limits: SecurityLimits,
    ) -> ConversionResult<String> {
        let mut generator = SecureRtfGenerator {
            security_limits,
            output_size: 0,
            table_cells_count: 0,
        };
        
        generator.generate_internal(document)
    }

    /// Generate with default security settings
    pub fn generate(document: &RtfDocument) -> ConversionResult<String> {
        Self::generate_with_security(document, SecurityLimits::default())
    }

    /// Check output size limit
    fn check_output_size(&mut self, additional: usize) -> ConversionResult<()> {
        self.output_size += additional;
        if self.output_size > self.security_limits.max_file_size {
            return Err(ConversionError::GenerationError(
                format!("Output size exceeds maximum allowed ({} bytes)", self.security_limits.max_file_size)
            ));
        }
        Ok(())
    }

    /// Generate RTF from document structure
    fn generate_internal(&mut self, document: &RtfDocument) -> ConversionResult<String> {
        let mut output = String::new();
        
        // RTF header
        let header = "{\\rtf1\\ansi\\deff0";
        self.check_output_size(header.len())?;
        output.push_str(header);
        
        // Font table
        let font_table = "{\\fonttbl{\\f0\\froman\\fcharset0 Times New Roman;}{\\f1\\fswiss\\fcharset0 Arial;}}";
        self.check_output_size(font_table.len())?;
        output.push_str(font_table);
        
        // Color table (basic colors)
        let color_table = "{\\colortbl;\\red0\\green0\\blue0;\\red255\\green0\\blue0;\\red0\\green255\\blue0;\\red0\\green0\\blue255;}";
        self.check_output_size(color_table.len())?;
        output.push_str(color_table);
        
        // Document content
        for (i, node) in document.content.iter().enumerate() {
            if i > 0 {
                let par = "\\par ";
                self.check_output_size(par.len())?;
                output.push_str(par);
            }
            self.generate_node(node, &mut output, 0)?;
        }
        
        self.check_output_size(1)?;
        output.push('}');
        
        Ok(output)
    }

    /// Generate RTF for a single node
    fn generate_node(&mut self, node: &RtfNode, output: &mut String, depth: usize) -> ConversionResult<()> {
        // Security: Check recursion depth
        if depth > self.security_limits.max_nesting_depth {
            return Err(ConversionError::GenerationError(
                format!("Maximum nesting depth {} exceeded", self.security_limits.max_nesting_depth)
            ));
        }

        match node {
            RtfNode::Text(text) => {
                let escaped = Self::escape_rtf_text(text);
                self.check_output_size(escaped.len())?;
                output.push_str(&escaped);
            }
            RtfNode::Paragraph(nodes) => {
                for (i, node) in nodes.iter().enumerate() {
                    if i > 0 {
                        self.check_output_size(1)?;
                        output.push(' ');
                    }
                    self.generate_node(node, output, depth)?;
                }
            }
            RtfNode::Bold(nodes) => {
                let start = "{\\b ";
                self.check_output_size(start.len())?;
                output.push_str(start);
                
                for node in nodes {
                    self.generate_node(node, output, depth + 1)?;
                }
                
                self.check_output_size(1)?;
                output.push('}');
            }
            RtfNode::Italic(nodes) => {
                let start = "{\\i ";
                self.check_output_size(start.len())?;
                output.push_str(start);
                
                for node in nodes {
                    self.generate_node(node, output, depth + 1)?;
                }
                
                self.check_output_size(1)?;
                output.push('}');
            }
            RtfNode::Underline(nodes) => {
                let start = "{\\ul ";
                self.check_output_size(start.len())?;
                output.push_str(start);
                
                for node in nodes {
                    self.generate_node(node, output, depth + 1)?;
                }
                
                self.check_output_size(1)?;
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
                
                let heading_start = format!("{{\\b\\fs{} ", font_size * 2); // RTF font size is in half-points
                self.check_output_size(heading_start.len())?;
                output.push_str(&heading_start);
                
                for node in content {
                    self.generate_node(node, output, depth + 1)?;
                }
                
                self.check_output_size(1)?;
                output.push('}');
            }
            RtfNode::ListItem { level, content } => {
                // RTF list formatting with indentation
                let indent = 720 * (*level as i32 + 1); // 720 twips = 0.5 inch
                let list_start = format!("{{\\pard\\li{}\\fi-360 ", indent);
                self.check_output_size(list_start.len())?;
                output.push_str(&list_start);
                
                let bullet = "\\bullet\\tab ";
                self.check_output_size(bullet.len())?;
                output.push_str(bullet);
                
                for node in content {
                    self.generate_node(node, output, depth + 1)?;
                }
                
                let end = "\\par}";
                self.check_output_size(end.len())?;
                output.push_str(end);
            }
            RtfNode::Table { rows } => {
                self.generate_table(rows, output)?;
            }
            RtfNode::LineBreak => {
                let line_break = "\\line ";
                self.check_output_size(line_break.len())?;
                output.push_str(line_break);
            }
            RtfNode::PageBreak => {
                let page_break = "\\page ";
                self.check_output_size(page_break.len())?;
                output.push_str(page_break);
            }
        }
        Ok(())
    }

    /// Generate RTF table with security limits
    fn generate_table(&mut self, rows: &[TableRow], output: &mut String) -> ConversionResult<()> {
        if rows.is_empty() {
            return Ok(());
        }

        // Security: Check table size limits
        if rows.len() > self.security_limits.max_table_rows {
            return Err(ConversionError::GenerationError(
                format!("Table has {} rows, exceeds maximum of {}", 
                    rows.len(), self.security_limits.max_table_rows)
            ));
        }

        for row in rows {
            // Security: Check column limit
            if row.cells.len() > self.security_limits.max_table_columns {
                return Err(ConversionError::GenerationError(
                    format!("Table row has {} columns, exceeds maximum of {}", 
                        row.cells.len(), self.security_limits.max_table_columns)
                ));
            }

            // Security: Track total cells
            self.table_cells_count += row.cells.len();
            if self.table_cells_count > self.security_limits.max_table_cells {
                return Err(ConversionError::GenerationError(
                    format!("Total table cells {} exceeds maximum of {}", 
                        self.table_cells_count, self.security_limits.max_table_cells)
                ));
            }

            // Table row definition
            let row_start = "{\\trowd\\trgaph108\\trleft-108";
            self.check_output_size(row_start.len())?;
            output.push_str(row_start);
            
            // Define cell widths (distribute evenly)
            let cell_count = row.cells.len();
            if cell_count > 0 {
                let cell_width = 9360 / cell_count; // Total width distributed evenly
                let mut cell_pos = 0;
                
                for _ in &row.cells {
                    cell_pos += cell_width;
                    let cell_def = format!("\\cellx{}", cell_pos);
                    self.check_output_size(cell_def.len())?;
                    output.push_str(&cell_def);
                }
            }
            
            // Table cells content
            for cell in &row.cells {
                let cell_start = "{\\pard\\intbl ";
                self.check_output_size(cell_start.len())?;
                output.push_str(cell_start);
                
                self.generate_cell(cell, output)?;
                
                let cell_end = "\\cell}";
                self.check_output_size(cell_end.len())?;
                output.push_str(cell_end);
            }
            
            // End row
            let row_end = "\\row}";
            self.check_output_size(row_end.len())?;
            output.push_str(row_end);
        }

        Ok(())
    }

    /// Generate content for a table cell
    fn generate_cell(&mut self, cell: &TableCell, output: &mut String) -> ConversionResult<()> {
        for (i, node) in cell.content.iter().enumerate() {
            if i > 0 {
                self.check_output_size(1)?;
                output.push(' ');
            }
            self.generate_node(node, output, 0)?;
        }
        Ok(())
    }

    /// Escape special RTF characters with proper Unicode handling
    fn escape_rtf_text(text: &str) -> String {
        text.chars()
            .map(|ch| match ch {
                '\\' => "\\\\".to_string(),
                '{' => "\\{".to_string(),
                '}' => "\\}".to_string(),
                '\n' => "\\par ".to_string(),
                '\r' => String::new(), // Skip carriage returns
                c if c as u32 > 127 => {
                    // Security: Proper Unicode handling
                    let code_point = c as u32;
                    if code_point <= 0xFFFF {
                        format!("\\u{}?", code_point as i16)
                    } else {
                        // For characters outside BMP, use replacement character
                        "\\u63?"
                    }
                }
                c => c.to_string(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::types::DocumentMetadata;

    #[test]
    fn test_generate_with_security() {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Paragraph(vec![RtfNode::Text(
                "Hello World".to_string(),
            )])],
        };

        let rtf = SecureRtfGenerator::generate(&document).unwrap();
        assert!(rtf.contains("Hello World"));
        assert!(rtf.starts_with("{\\rtf1\\ansi"));
        assert!(rtf.ends_with('}'));
    }

    #[test]
    fn test_output_size_limit() {
        let mut limits = SecurityLimits::default();
        limits.max_file_size = 100; // Very small limit
        
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Paragraph(vec![RtfNode::Text(
                "A".repeat(1000), // Large text
            )])],
        };

        let result = SecureRtfGenerator::generate_with_security(&document, limits);
        assert!(result.is_err());
        match result {
            Err(ConversionError::GenerationError(msg)) => {
                assert!(msg.contains("Output size exceeds maximum"));
            }
            _ => panic!("Expected generation error for output size"),
        }
    }

    #[test]
    fn test_table_limits() {
        let mut limits = SecurityLimits::default();
        limits.max_table_rows = 2;
        
        // Create table with too many rows
        let mut rows = Vec::new();
        for _ in 0..5 {
            rows.push(TableRow {
                cells: vec![TableCell {
                    content: vec![RtfNode::Text("Cell".to_string())],
                }],
            });
        }
        
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Table { rows }],
        };

        let result = SecureRtfGenerator::generate_with_security(&document, limits);
        assert!(result.is_err());
        match result {
            Err(ConversionError::GenerationError(msg)) => {
                assert!(msg.contains("exceeds maximum"));
            }
            _ => panic!("Expected generation error for table size"),
        }
    }

    #[test]
    fn test_unicode_escape() {
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Paragraph(vec![RtfNode::Text(
                "Unicode: \u{1F600} test".to_string(), // Emoji
            )])],
        };

        let rtf = SecureRtfGenerator::generate(&document).unwrap();
        assert!(rtf.contains("\\u")); // Should contain Unicode escape
    }
}