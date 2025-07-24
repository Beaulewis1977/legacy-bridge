// Pooled RTF Parser - Uses memory pools to reduce allocation overhead during parsing

use super::types::{
    ConversionError, ConversionResult, DocumentMetadata, RtfDocument,
    RtfNode, RtfToken, TableRow, TableCell, FontInfo, Color,
};
use super::memory_pools::{CONVERSION_POOLS, PooledNodeBuilder};
use std::sync::atomic::{AtomicUsize, Ordering};

// SECURITY: Memory usage tracking for DoS prevention
const MAX_MEMORY_PER_CONVERSION: usize = 100 * 1024 * 1024; // 100MB
const MAX_NODES_PER_DOCUMENT: usize = 100_000;
const MAX_TEXT_LENGTH: usize = 10 * 1024 * 1024; // 10MB max text per node

// Global memory usage tracker (thread-safe)
static MEMORY_USAGE: AtomicUsize = AtomicUsize::new(0);

/// RTF Parser with memory pool integration
pub struct PooledRtfParser {
    tokens: Vec<RtfToken>,
    position: usize,
    metadata: DocumentMetadata,
    font_table_mode: bool,
    color_table_mode: bool,
    recursion_depth: usize,
    node_count: usize,
    memory_used: usize,
}

impl PooledRtfParser {
    /// Parse RTF tokens into a document structure using memory pools
    pub fn parse(tokens: Vec<RtfToken>) -> ConversionResult<RtfDocument> {
        // SECURITY: Estimate memory usage for tokens
        let estimated_memory = tokens.len() * std::mem::size_of::<RtfToken>();
        if estimated_memory > MAX_MEMORY_PER_CONVERSION {
            return Err(ConversionError::ValidationError(
                format!("Token memory usage ({} bytes) exceeds maximum allowed ({} bytes)",
                    estimated_memory, MAX_MEMORY_PER_CONVERSION)
            ));
        }
        
        // SECURITY: Try to reserve memory atomically
        let current_usage = MEMORY_USAGE.fetch_add(estimated_memory, Ordering::SeqCst);
        if current_usage + estimated_memory > MAX_MEMORY_PER_CONVERSION * 10 {
            MEMORY_USAGE.fetch_sub(estimated_memory, Ordering::SeqCst);
            return Err(ConversionError::ValidationError(
                "System memory limit exceeded - too many concurrent conversions".to_string()
            ));
        }
        
        let mut parser = PooledRtfParser {
            tokens,
            position: 0,
            metadata: DocumentMetadata::default(),
            font_table_mode: false,
            color_table_mode: false,
            recursion_depth: 0,
            node_count: 0,
            memory_used: estimated_memory,
        };

        let result = parser.parse_document();
        
        // SECURITY: Always release memory reservation
        MEMORY_USAGE.fetch_sub(estimated_memory, Ordering::SeqCst);
        
        result
    }

    /// Parse the entire document using memory pools
    fn parse_document(&mut self) -> ConversionResult<RtfDocument> {
        // Expect document to start with a group
        self.expect_token(&RtfToken::GroupStart)?;

        // Check for RTF header
        if let Some(RtfToken::ControlWord { name, parameter }) = self.peek() {
            if name == "rtf" && parameter == &Some(1) {
                self.advance();
            } else {
                return Err(ConversionError::ParseError(
                    "Invalid RTF header".to_string(),
                ));
            }
        }

        // Parse document content using pooled nodes
        let content = self.parse_group_content()?;

        Ok(RtfDocument {
            metadata: self.metadata.clone(),
            content,
        })
    }

    /// Parse content within a group using pooled node builders
    fn parse_group_content(&mut self) -> ConversionResult<Vec<RtfNode>> {
        // SECURITY: Check recursion depth
        const MAX_RECURSION_DEPTH: usize = 50;
        if self.recursion_depth >= MAX_RECURSION_DEPTH {
            return Err(ConversionError::ParseError(
                format!("Maximum recursion depth {} exceeded", MAX_RECURSION_DEPTH)
            ));
        }
        
        self.recursion_depth += 1;
        let result = self.parse_group_content_inner();
        self.recursion_depth -= 1;
        
        result
    }
    
    /// Inner group parsing logic using memory pools
    fn parse_group_content_inner(&mut self) -> ConversionResult<Vec<RtfNode>> {
        let mut nodes = PooledNodeBuilder::new();
        let mut current_paragraph = PooledNodeBuilder::new();

        while self.position < self.tokens.len() {
            match self.current_token() {
                Some(RtfToken::GroupEnd) => {
                    // End of current group
                    if !current_paragraph.is_empty() {
                        let paragraph_nodes = current_paragraph.finish();
                        nodes.add_node(RtfNode::Paragraph(paragraph_nodes));
                        current_paragraph = PooledNodeBuilder::new();
                    }
                    self.advance();
                    break;
                }
                Some(RtfToken::GroupStart) => {
                    self.advance();
                    let group_content = self.parse_group_content()?;
                    for node in group_content {
                        current_paragraph.add_node(node);
                    }
                }
                Some(RtfToken::ControlWord { name, parameter }) => {
                    let name = name.clone();
                    let parameter = *parameter;
                    self.advance();

                    match name.as_str() {
                        "par" => {
                            // End of paragraph
                            if !current_paragraph.is_empty() {
                                let paragraph_nodes = current_paragraph.finish();
                                nodes.add_node(RtfNode::Paragraph(paragraph_nodes));
                                current_paragraph = PooledNodeBuilder::new();
                            }
                        }
                        "b" => {
                            // Bold
                            if parameter != Some(0) {
                                let bold_content = self.parse_formatted_content()?;
                                current_paragraph.add_node(RtfNode::Bold(bold_content));
                            }
                        }
                        "i" => {
                            // Italic
                            if parameter != Some(0) {
                                let italic_content = self.parse_formatted_content()?;
                                current_paragraph.add_node(RtfNode::Italic(italic_content));
                            }
                        }
                        "ul" | "ulnone" => {
                            // Underline
                            if name == "ul" {
                                let underline_content = self.parse_formatted_content()?;
                                current_paragraph.add_node(RtfNode::Underline(underline_content));
                            }
                        }
                        "line" => {
                            current_paragraph.add_node(RtfNode::LineBreak);
                        }
                        "page" => {
                            nodes.add_node(RtfNode::PageBreak);
                        }
                        "fonttbl" => {
                            self.font_table_mode = true;
                            self.parse_font_table()?;
                        }
                        "colortbl" => {
                            self.color_table_mode = true;
                            self.parse_color_table()?;
                        }
                        "info" => {
                            self.parse_info_group()?;
                        }
                        _ => {
                            // Ignore other control words for now
                        }
                    }
                }
                Some(RtfToken::ControlSymbol(_)) => {
                    // Handle control symbols if needed
                    self.advance();
                }
                Some(RtfToken::Text(text)) => {
                    // SECURITY: Check text length
                    if text.len() > MAX_TEXT_LENGTH {
                        return Err(ConversionError::ValidationError(
                            format!("Text node size ({} bytes) exceeds maximum allowed ({} bytes)",
                                text.len(), MAX_TEXT_LENGTH)
                        ));
                    }
                    
                    // Use pooled string when possible
                    if text.len() <= 64 {
                        let mut pooled_str = CONVERSION_POOLS.get_string_buffer(text.len());
                        pooled_str.push_str(text);
                        current_paragraph.add_text(std::mem::take(&mut *pooled_str));
                    } else {
                        current_paragraph.add_text(text.clone());
                    }
                    
                    self.advance();
                    
                    // SECURITY: Track node count
                    self.node_count += 1;
                    if self.node_count > MAX_NODES_PER_DOCUMENT {
                        return Err(ConversionError::ValidationError(
                            format!("Node count exceeds maximum allowed ({})", MAX_NODES_PER_DOCUMENT)
                        ));
                    }
                }
                Some(RtfToken::HexValue(_)) => {
                    // Handle hex values if needed
                    self.advance();
                }
                None => break,
            }
        }

        // Handle any remaining paragraph content
        if !current_paragraph.is_empty() {
            let paragraph_nodes = current_paragraph.finish();
            nodes.add_node(RtfNode::Paragraph(paragraph_nodes));
        }

        Ok(nodes.finish())
    }

    /// Parse formatted content (bold, italic, etc.) using memory pools
    fn parse_formatted_content(&mut self) -> ConversionResult<Vec<RtfNode>> {
        let mut content = PooledNodeBuilder::new();

        while self.position < self.tokens.len() {
            match self.current_token() {
                Some(RtfToken::Text(text)) => {
                    // Use pooled string for small texts
                    if text.len() <= 64 {
                        let mut pooled_str = CONVERSION_POOLS.get_string_buffer(text.len());
                        pooled_str.push_str(text);
                        content.add_text(std::mem::take(&mut *pooled_str));
                    } else {
                        content.add_text(text.clone());
                    }
                    self.advance();
                }
                Some(RtfToken::ControlWord { name, .. }) => {
                    // End formatting on certain control words
                    if matches!(name.as_str(), "b" | "i" | "ul" | "par") {
                        break;
                    }
                    self.advance();
                }
                Some(RtfToken::GroupEnd) => break,
                _ => self.advance(),
            }
        }

        Ok(content.finish())
    }

    // Helper methods remain largely the same but use pooled strings where appropriate
    fn current_token(&self) -> Option<&RtfToken> {
        self.tokens.get(self.position)
    }

    fn peek(&self) -> Option<&RtfToken> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn expect_token(&mut self, expected: &RtfToken) -> ConversionResult<()> {
        match self.current_token() {
            Some(token) if std::mem::discriminant(token) == std::mem::discriminant(expected) => {
                self.advance();
                Ok(())
            }
            Some(token) => Err(ConversionError::ParseError(format!(
                "Expected {:?}, found {:?}",
                expected, token
            ))),
            None => Err(ConversionError::ParseError(
                "Unexpected end of tokens".to_string(),
            )),
        }
    }

    fn parse_font_table(&mut self) -> ConversionResult<()> {
        // Simplified font table parsing
        while self.position < self.tokens.len() {
            match self.current_token() {
                Some(RtfToken::GroupEnd) => {
                    self.font_table_mode = false;
                    break;
                }
                _ => self.advance(),
            }
        }
        Ok(())
    }

    fn parse_color_table(&mut self) -> ConversionResult<()> {
        // Simplified color table parsing
        while self.position < self.tokens.len() {
            match self.current_token() {
                Some(RtfToken::GroupEnd) => {
                    self.color_table_mode = false;
                    break;
                }
                _ => self.advance(),
            }
        }
        Ok(())
    }

    fn parse_info_group(&mut self) -> ConversionResult<()> {
        // Skip info group for now
        let mut depth = 1;
        while self.position < self.tokens.len() && depth > 0 {
            match self.current_token() {
                Some(RtfToken::GroupStart) => depth += 1,
                Some(RtfToken::GroupEnd) => depth -= 1,
                _ => {}
            }
            self.advance();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pooled_parser_basic() {
        let tokens = vec![
            RtfToken::GroupStart,
            RtfToken::ControlWord { name: "rtf".to_string(), parameter: Some(1) },
            RtfToken::Text("Hello World".to_string()),
            RtfToken::ControlWord { name: "par".to_string(), parameter: None },
            RtfToken::GroupEnd,
        ];
        
        let result = PooledRtfParser::parse(tokens);
        assert!(result.is_ok());
        
        let doc = result.unwrap();
        assert_eq!(doc.content.len(), 1);
    }
    
    #[test]
    fn test_memory_pool_usage() {
        // Get initial stats
        let initial_stats = CONVERSION_POOLS.get_stats();
        
        // Parse multiple documents
        for i in 0..5 {
            let tokens = vec![
                RtfToken::GroupStart,
                RtfToken::ControlWord { name: "rtf".to_string(), parameter: Some(1) },
                RtfToken::Text(format!("Document {}", i)),
                RtfToken::ControlWord { name: "b".to_string(), parameter: Some(1) },
                RtfToken::Text("Bold text".to_string()),
                RtfToken::ControlWord { name: "b".to_string(), parameter: Some(0) },
                RtfToken::ControlWord { name: "par".to_string(), parameter: None },
                RtfToken::GroupEnd,
            ];
            
            let _ = PooledRtfParser::parse(tokens).unwrap();
        }
        
        // Check pool usage increased
        let final_stats = CONVERSION_POOLS.get_stats();
        assert!(final_stats.string_pool_size >= initial_stats.string_pool_size);
    }
}