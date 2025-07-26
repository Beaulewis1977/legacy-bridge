// RTF Parser - Parses RTF tokens into a document structure

use super::types::{
    ConversionError, ConversionResult, DocumentMetadata, RtfDocument,
    RtfNode, RtfToken,
};
use std::sync::atomic::{AtomicUsize, Ordering};

// SECURITY: Memory usage tracking for DoS prevention
const MAX_MEMORY_PER_CONVERSION: usize = 100 * 1024 * 1024; // 100MB
const MAX_NODES_PER_DOCUMENT: usize = 100_000; // Prevent excessive node creation
const MAX_TEXT_LENGTH: usize = 10 * 1024 * 1024; // 10MB max text per node

// Global memory usage tracker (thread-safe)
static MEMORY_USAGE: AtomicUsize = AtomicUsize::new(0);

/// RTF Parser
pub struct RtfParser {
    tokens: Vec<RtfToken>,
    position: usize,
    metadata: DocumentMetadata,
    font_table_mode: bool,
    color_table_mode: bool,
    recursion_depth: usize,
    node_count: usize, // SECURITY: Track node count to prevent memory exhaustion
    memory_used: usize, // SECURITY: Track memory usage for this parse
}

impl RtfParser {
    /// Parse RTF tokens into a document structure
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
        if current_usage + estimated_memory > MAX_MEMORY_PER_CONVERSION * 10 { // Allow 10 concurrent conversions
            MEMORY_USAGE.fetch_sub(estimated_memory, Ordering::SeqCst);
            return Err(ConversionError::ValidationError(
                "System memory limit exceeded - too many concurrent conversions".to_string()
            ));
        }
        
        let mut parser = RtfParser {
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

    /// Parse the entire document
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

        // Parse document content
        let content = self.parse_group_content()?;

        Ok(RtfDocument {
            metadata: self.metadata.clone(),
            content,
        })
    }

    /// Parse content within a group
    fn parse_group_content(&mut self) -> ConversionResult<Vec<RtfNode>> {
        // SECURITY: Check recursion depth to prevent stack overflow
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
    
    /// Inner group parsing logic
    fn parse_group_content_inner(&mut self) -> ConversionResult<Vec<RtfNode>> {
        let mut nodes = Vec::new();
        let mut current_paragraph = Vec::new();
        
        // SECURITY: Pre-allocate with reasonable capacity to prevent reallocation attacks
        nodes.reserve(std::cmp::min(1000, self.tokens.len() - self.position));

        while self.position < self.tokens.len() {
            match self.current_token() {
                Some(RtfToken::GroupEnd) => {
                    // End of current group
                    if !current_paragraph.is_empty() {
                        nodes.push(RtfNode::Paragraph(current_paragraph.clone()));
                        current_paragraph.clear();
                    }
                    self.advance();
                    break;
                }
                Some(RtfToken::GroupStart) => {
                    self.advance();
                    let group_content = self.parse_group_content()?;
                    current_paragraph.extend(group_content);
                }
                Some(RtfToken::ControlWord { name, parameter }) => {
                    let name = name.clone();
                    let parameter = *parameter;
                    self.advance();

                    match name.as_str() {
                        "par" => {
                            // End of paragraph
                            if !current_paragraph.is_empty() {
                                nodes.push(RtfNode::Paragraph(current_paragraph.clone()));
                                current_paragraph.clear();
                            }
                        }
                        "b" => {
                            // Bold
                            if parameter != Some(0) {
                                let bold_content = self.parse_formatted_content()?;
                                current_paragraph.push(RtfNode::Bold(bold_content));
                            }
                        }
                        "i" => {
                            // Italic
                            if parameter != Some(0) {
                                let italic_content = self.parse_formatted_content()?;
                                current_paragraph.push(RtfNode::Italic(italic_content));
                            }
                        }
                        "ul" | "ulnone" => {
                            // Underline
                            if name == "ul" {
                                let underline_content = self.parse_formatted_content()?;
                                current_paragraph.push(RtfNode::Underline(underline_content));
                            }
                        }
                        "line" => {
                            current_paragraph.push(RtfNode::LineBreak);
                        }
                        "page" => {
                            nodes.push(RtfNode::PageBreak);
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
                    // SECURITY: Check text length before cloning
                    if text.len() > MAX_TEXT_LENGTH {
                        return Err(ConversionError::ValidationError(
                            format!("Text node size ({} bytes) exceeds maximum allowed ({} bytes)",
                                text.len(), MAX_TEXT_LENGTH)
                        ));
                    }
                    
                    // SECURITY: Check node count
                    self.node_count += 1;
                    if self.node_count > MAX_NODES_PER_DOCUMENT {
                        return Err(ConversionError::ValidationError(
                            format!("Document complexity exceeds maximum allowed ({} nodes)",
                                MAX_NODES_PER_DOCUMENT)
                        ));
                    }
                    
                    // SECURITY: Track memory usage
                    let text_memory = text.len() + std::mem::size_of::<RtfNode>();
                    self.memory_used = self.memory_used.saturating_add(text_memory);
                    if self.memory_used > MAX_MEMORY_PER_CONVERSION {
                        return Err(ConversionError::ValidationError(
                            format!("Document memory usage ({} bytes) exceeds maximum allowed ({} bytes)",
                                self.memory_used, MAX_MEMORY_PER_CONVERSION)
                        ));
                    }
                    
                    current_paragraph.push(RtfNode::Text(text.clone()));
                    self.advance();
                }
                Some(RtfToken::HexValue(_)) => {
                    // Handle hex values (usually for special characters)
                    self.advance();
                }
                None => break,
            }
        }

        // Add any remaining paragraph
        if !current_paragraph.is_empty() {
            nodes.push(RtfNode::Paragraph(current_paragraph));
        }

        Ok(nodes)
    }

    /// Parse formatted content (for bold, italic, etc.)
    fn parse_formatted_content(&mut self) -> ConversionResult<Vec<RtfNode>> {
        let mut content = Vec::new();
        
        // SECURITY: Limit formatted content depth to prevent memory exhaustion
        const MAX_FORMATTED_NODES: usize = 1000;
        content.reserve(std::cmp::min(MAX_FORMATTED_NODES, 16));

        // Look for content until we hit a formatting boundary
        while let Some(token) = self.current_token() {
            match token {
                RtfToken::Text(text) => {
                    // SECURITY: Prevent excessive formatted content
                    if content.len() >= 1000 {
                        return Err(ConversionError::ValidationError(
                            "Formatted content too complex".to_string()
                        ));
                    }
                    
                    // SECURITY: Check text length
                    if text.len() > MAX_TEXT_LENGTH {
                        return Err(ConversionError::ValidationError(
                            format!("Text size in formatted content exceeds maximum allowed")
                        ));
                    }
                    
                    self.node_count += 1;
                    if self.node_count > MAX_NODES_PER_DOCUMENT {
                        return Err(ConversionError::ValidationError(
                            "Document too complex".to_string()
                        ));
                    }
                    
                    content.push(RtfNode::Text(text.clone()));
                    self.advance();
                }
                RtfToken::ControlWord { name, parameter } => {
                    // Check if this ends the formatting
                    if matches!(name.as_str(), "b" | "i" | "ul" | "ulnone") && *parameter == Some(0) {
                        self.advance();
                        break;
                    }
                    // Otherwise, let the parent handle it
                    break;
                }
                _ => break,
            }
        }

        Ok(content)
    }

    /// Parse font table
    fn parse_font_table(&mut self) -> ConversionResult<()> {
        // TODO: Implement full font table parsing
        self.font_table_mode = false;
        Ok(())
    }

    /// Parse color table
    fn parse_color_table(&mut self) -> ConversionResult<()> {
        // TODO: Implement full color table parsing
        self.color_table_mode = false;
        Ok(())
    }

    /// Parse info group (document metadata)
    fn parse_info_group(&mut self) -> ConversionResult<()> {
        // TODO: Implement info group parsing
        Ok(())
    }

    /// Get current token
    fn current_token(&self) -> Option<&RtfToken> {
        self.tokens.get(self.position)
    }

    /// Peek at current token without advancing
    fn peek(&self) -> Option<&RtfToken> {
        self.tokens.get(self.position)
    }

    /// Advance to next token
    fn advance(&mut self) {
        self.position += 1;
    }

    /// Expect a specific token
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
                "Unexpected end of input".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::rtf_lexer::tokenize;

    #[test]
    fn test_parse_simple_document() {
        let rtf = r"{\rtf1 Hello World\par}";
        let tokens = tokenize(rtf).expect("tokenization should succeed");
        let document = RtfParser::parse(tokens).expect("parsing should succeed");
        
        assert_eq!(document.content.len(), 1);
        match &document.content[0] {
            RtfNode::Paragraph(nodes) => {
                assert_eq!(nodes.len(), 1);
                match &nodes[0] {
                    RtfNode::Text(text) => assert_eq!(text, "Hello World"),
                    _ => panic!("Expected text node"),
                }
            }
            _ => panic!("Expected paragraph node"),
        }
    }

    #[test]
    fn test_parse_formatted_text() {
        let rtf = r"{\rtf1 Normal {\b Bold} {\i Italic}\par}";
        let tokens = tokenize(rtf).expect("tokenization should succeed");
        let document = RtfParser::parse(tokens).expect("parsing should succeed");
        
        assert_eq!(document.content.len(), 1);
    }
}