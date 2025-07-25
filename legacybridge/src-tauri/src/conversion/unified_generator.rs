// Unified Generator - Consolidates secure and standard generation implementations
//
// This module provides a single generator implementation that replaces
// the duplicate secure_generator.rs and rtf_generator.rs with configurable
// security levels and performance optimizations.

use super::types::{ConversionError, ConversionResult, RtfDocument, RtfNode, TableCell, TableRow};
use super::unified_config::{ConversionConfig, SecurityLevel};
use super::markdown_generator::MarkdownGenerator;
use std::fmt::Write;

/// Unified generator for RTF and Markdown output
pub struct UnifiedGenerator {
    config: ConversionConfig,
    output_size: usize,
    recursion_depth: usize,
}

impl UnifiedGenerator {
    /// Create a new generator with the given configuration
    pub fn new(config: ConversionConfig) -> Self {
        Self {
            config,
            output_size: 0,
            recursion_depth: 0,
        }
    }
    
    /// Create a generator with default configuration
    pub fn default() -> Self {
        Self::new(ConversionConfig::default())
    }
    
    /// Generate RTF from a document structure
    pub fn generate_rtf(&mut self, document: &RtfDocument) -> ConversionResult<String> {
        self.output_size = 0;
        let mut output = String::new();
        
        // Pre-allocate if memory pool is configured
        if let Some(ref pool) = self.config.memory_pool {
            output.reserve(pool.initial_size);
        }
        
        // RTF header
        self.append_checked(&mut output, "{\\rtf1\\ansi\\deff0")?;
        
        // Font table
        self.append_checked(&mut output, 
            "{\\fonttbl{\\f0\\froman\\fcharset0 Times New Roman;}{\\f1\\fswiss\\fcharset0 Arial;}}")?;
        
        // Color table
        self.append_checked(&mut output,
            "{\\colortbl;\\red0\\green0\\blue0;\\red255\\green0\\blue0;\\red0\\green255\\blue0;\\red0\\green0\\blue255;}")?;
        
        // Document content
        for (i, node) in document.content.iter().enumerate() {
            if i > 0 {
                self.append_checked(&mut output, "\\par ")?;
            }
            self.generate_rtf_node(node, &mut output, 0)?;
        }
        
        self.append_checked(&mut output, "}")?;
        Ok(output)
    }
    
    /// Generate Markdown from a document structure
    pub fn generate_markdown(&mut self, document: &RtfDocument) -> ConversionResult<String> {
        // Use existing markdown generator for now
        // TODO: Integrate markdown generation logic here
        MarkdownGenerator::generate(document)
    }
    
    // Helper methods
    
    fn should_check_limits(&self) -> bool {
        self.config.security_level.enforce_size_limits()
    }
    
    fn append_checked(&mut self, output: &mut String, content: &str) -> ConversionResult<()> {
        if self.should_check_limits() {
            self.check_output_size(content.len())?;
        }
        output.push_str(content);
        Ok(())
    }
    
    fn check_output_size(&mut self, additional: usize) -> ConversionResult<()> {
        if let Some(limits) = self.config.security_level.limits() {
            self.output_size += additional;
            if self.output_size > limits.max_file_size {
                return Err(ConversionError::GenerationError(
                    format!("Output size {} exceeds maximum allowed {}", 
                        self.output_size, limits.max_file_size)
                ));
            }
        }
        Ok(())
    }
    
    fn check_recursion_depth(&self) -> ConversionResult<()> {
        if let Some(limits) = self.config.security_level.limits() {
            if self.recursion_depth >= limits.max_nesting_depth {
                return Err(ConversionError::GenerationError(
                    format!("Maximum recursion depth {} exceeded", limits.max_nesting_depth)
                ));
            }
        }
        Ok(())
    }
    
    // RTF generation methods
    
    fn generate_rtf_node(&mut self, node: &RtfNode, output: &mut String, depth: usize) -> ConversionResult<()> {
        // Check recursion depth if security is enabled
        if self.should_check_limits() {
            self.check_recursion_depth()?;
        }
        
        self.recursion_depth = depth;
        
        match node {
            RtfNode::Text(text) => {
                let escaped = Self::escape_rtf_text(text);
                self.append_checked(output, &escaped)?;
            }
            
            RtfNode::Paragraph(nodes) => {
                // Check node count limit if configured
                if self.should_check_limits() {
                    if let Some(limits) = self.config.security_level.limits() {
                        if nodes.len() > 10000 {
                            return Err(ConversionError::GenerationError(
                                "Paragraph contains too many nodes".to_string()
                            ));
                        }
                    }
                }
                
                for (i, node) in nodes.iter().enumerate() {
                    if i > 0 {
                        self.append_checked(output, " ")?;
                    }
                    self.generate_rtf_node(node, output, depth)?;
                }
            }
            
            RtfNode::Bold(nodes) => {
                self.append_checked(output, "{\\b ")?;
                for node in nodes {
                    self.generate_rtf_node(node, output, depth + 1)?;
                }
                self.append_checked(output, "}")?;
            }
            
            RtfNode::Italic(nodes) => {
                self.append_checked(output, "{\\i ")?;
                for node in nodes {
                    self.generate_rtf_node(node, output, depth + 1)?;
                }
                self.append_checked(output, "}")?;
            }
            
            RtfNode::Underline(nodes) => {
                self.append_checked(output, "{\\ul ")?;
                for node in nodes {
                    self.generate_rtf_node(node, output, depth + 1)?;
                }
                self.append_checked(output, "}")?;
            }
            
            RtfNode::Heading { level, content } => {
                let font_size = match level {
                    1 => 24,
                    2 => 20,
                    3 => 16,
                    4 => 14,
                    5 => 12,
                    _ => 12,
                };
                
                let heading_start = format!("{{\\b\\fs{} ", font_size * 2);
                self.append_checked(output, &heading_start)?;
                
                for node in content {
                    self.generate_rtf_node(node, output, depth + 1)?;
                }
                
                self.append_checked(output, "}")?;
            }
            
            RtfNode::ListItem { level, content } => {
                let indent = 720 * (*level as i32 + 1);
                let list_start = format!("{{\\pard\\li{}\\fi-360 ", indent);
                self.append_checked(output, &list_start)?;
                self.append_checked(output, "\\bullet\\tab ")?;
                
                for node in content {
                    self.generate_rtf_node(node, output, depth + 1)?;
                }
                
                self.append_checked(output, "\\par}")?;
            }
            
            RtfNode::Table { rows } => {
                self.generate_table(rows, output)?;
            }
            
            RtfNode::LineBreak => {
                self.append_checked(output, "\\line ")?;
            }
            
            RtfNode::PageBreak => {
                self.append_checked(output, "\\page ")?;
            }
        }
        
        Ok(())
    }
    
    fn generate_table(&mut self, rows: &[TableRow], output: &mut String) -> ConversionResult<()> {
        if rows.is_empty() {
            return Ok(());
        }
        
        // Check table limits if security is enabled
        if self.should_check_limits() {
            if let Some(limits) = self.config.security_level.limits() {
                if rows.len() > limits.max_table_rows {
                    return Err(ConversionError::GenerationError(
                        format!("Table has {} rows, exceeds maximum of {}", 
                            rows.len(), limits.max_table_rows)
                    ));
                }
                
                let total_cells: usize = rows.iter().map(|r| r.cells.len()).sum();
                if total_cells > limits.max_table_cells {
                    return Err(ConversionError::GenerationError(
                        format!("Table has {} cells, exceeds maximum of {}", 
                            total_cells, limits.max_table_cells)
                    ));
                }
            }
        }
        
        for row in rows {
            // Table row definition
            self.append_checked(output, "{\\trowd\\trgaph108\\trleft-108")?;
            
            // Define cell widths
            let cell_count = row.cells.len();
            if cell_count > 0 {
                // Check column limit
                if self.should_check_limits() {
                    if let Some(limits) = self.config.security_level.limits() {
                        if cell_count > limits.max_table_columns {
                            return Err(ConversionError::GenerationError(
                                format!("Row has {} columns, exceeds maximum of {}", 
                                    cell_count, limits.max_table_columns)
                            ));
                        }
                    }
                }
                
                let cell_width = 9360 / cell_count;
                let mut cell_pos = 0;
                
                for _ in &row.cells {
                    cell_pos += cell_width;
                    let cell_def = format!("\\cellx{}", cell_pos);
                    self.append_checked(output, &cell_def)?;
                }
            }
            
            // Table cells content
            for cell in &row.cells {
                self.append_checked(output, "{\\pard\\intbl ")?;
                self.generate_cell(cell, output)?;
                self.append_checked(output, "\\cell}")?;
            }
            
            self.append_checked(output, "\\row}")?;
        }
        
        Ok(())
    }
    
    fn generate_cell(&mut self, cell: &TableCell, output: &mut String) -> ConversionResult<()> {
        for node in &cell.content {
            self.generate_rtf_node(node, output, self.recursion_depth + 1)?;
        }
        Ok(())
    }
    
    /// Escape special RTF characters in text
    fn escape_rtf_text(text: &str) -> String {
        let mut escaped = String::with_capacity(text.len() + 10);
        
        for ch in text.chars() {
            match ch {
                '\\' => escaped.push_str("\\\\"),
                '{' => escaped.push_str("\\{"),
                '}' => escaped.push_str("\\}"),
                '\n' => escaped.push_str("\\par "),
                '\r' => {}, // Skip carriage returns
                _ if ch as u32 > 127 => {
                    // Unicode character - write! to String should never fail
                    use std::fmt::Write;
                    let _ = write!(escaped, "\\u{}?", ch as u32);
                }
                _ => escaped.push(ch),
            }
        }
        
        escaped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::types::DocumentMetadata;
    
    #[test]
    fn test_escape_rtf_text() {
        assert_eq!(UnifiedGenerator::escape_rtf_text("Hello"), "Hello");
        assert_eq!(UnifiedGenerator::escape_rtf_text("Hello\\World"), "Hello\\\\World");
        assert_eq!(UnifiedGenerator::escape_rtf_text("{text}"), "\\{text\\}");
        assert_eq!(UnifiedGenerator::escape_rtf_text("Line1\nLine2"), "Line1\\par Line2");
    }
    
    #[test]
    fn test_generate_simple_rtf() {
        let mut generator = UnifiedGenerator::default();
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![
                RtfNode::Paragraph(vec![RtfNode::Text("Hello World".to_string())]),
            ],
        };
        
        let result = generator.generate_rtf(&document);
        assert!(result.is_ok());
        let rtf = result.unwrap();
        assert!(rtf.contains("Hello World"));
        assert!(rtf.starts_with("{\\rtf1"));
        assert!(rtf.ends_with("}"));
    }
    
    #[test]
    fn test_security_limits() {
        let config = ConversionConfig::high_security();
        let mut generator = UnifiedGenerator::new(config);
        
        // Create a document that exceeds limits
        let mut large_content = Vec::new();
        for _ in 0..20000 {
            large_content.push(RtfNode::Text("X".repeat(1000)));
        }
        
        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![RtfNode::Paragraph(large_content)],
        };
        
        let result = generator.generate_rtf(&document);
        assert!(result.is_err());
    }
}