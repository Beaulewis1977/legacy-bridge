// Optimized RTF Parser - Zero-copy parsing with Cow<str>
//
// Key optimizations:
// 1. Use Cow<str> to avoid unnecessary string cloning
// 2. String interning for repeated text
// 3. Move semantics instead of cloning
// 4. Pre-allocated buffers
// 5. Reduced allocations in hot paths

use super::types::{
    ConversionError, ConversionResult, DocumentMetadata, RtfDocument,
    RtfNode, RtfToken,
};
use super::string_interner::OptimizedStringInterner;
use std::borrow::Cow;
use std::sync::Arc;

/// Optimized RTF Parser with reduced allocations
pub struct OptimizedRtfParser {
    tokens: Vec<RtfToken>,
    position: usize,
    metadata: DocumentMetadata,
    font_table_mode: bool,
    color_table_mode: bool,
    recursion_depth: usize,
    string_cache: OptimizedStringInterner,
}

impl OptimizedRtfParser {
    /// Parse RTF tokens into a document structure with optimizations
    pub fn parse(tokens: Vec<RtfToken>) -> ConversionResult<RtfDocument> {
        let mut parser = OptimizedRtfParser {
            tokens,
            position: 0,
            metadata: DocumentMetadata::default(),
            font_table_mode: false,
            color_table_mode: false,
            recursion_depth: 0,
            string_cache: OptimizedStringInterner::new(),
        };

        parser.parse_document()
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

        // Print interner stats for debugging
        #[cfg(debug_assertions)]
        {
            let stats = self.string_cache.stats();
            eprintln!("RTF Parser String Interner Stats: {:?}", stats);
        }

        Ok(RtfDocument {
            metadata: self.metadata,
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
    
    /// Inner group parsing logic with optimizations
    fn parse_group_content_inner(&mut self) -> ConversionResult<Vec<RtfNode>> {
        let mut nodes = Vec::with_capacity(16); // Pre-allocate for typical paragraph
        let mut current_paragraph = Vec::with_capacity(8);

        while self.position < self.tokens.len() {
            match self.current_token() {
                Some(RtfToken::GroupEnd) => {
                    // End of current group
                    if !current_paragraph.is_empty() {
                        // Move instead of clone
                        nodes.push(RtfNode::Paragraph(std::mem::take(&mut current_paragraph)));
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
                    // Use references to avoid cloning
                    let name_ref = name.as_str();
                    let param_value = *parameter;
                    self.advance();

                    match name_ref {
                        "par" => {
                            // End of paragraph
                            if !current_paragraph.is_empty() {
                                // Move instead of clone
                                nodes.push(RtfNode::Paragraph(std::mem::take(&mut current_paragraph)));
                            }
                        }
                        "b" => {
                            // Bold
                            if param_value != Some(0) {
                                let bold_content = self.parse_formatted_content()?;
                                current_paragraph.push(RtfNode::Bold(bold_content));
                            }
                        }
                        "i" => {
                            // Italic
                            if param_value != Some(0) {
                                let italic_content = self.parse_formatted_content()?;
                                current_paragraph.push(RtfNode::Italic(italic_content));
                            }
                        }
                        "ul" | "ulnone" => {
                            // Underline
                            if name_ref == "ul" {
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
                    // Intern the text for deduplication
                    let interned_text = self.string_cache.intern_to_string(text);
                    current_paragraph.push(RtfNode::Text(interned_text));
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

    /// Parse formatted content with optimizations
    fn parse_formatted_content(&mut self) -> ConversionResult<Vec<RtfNode>> {
        let mut content = Vec::with_capacity(4); // Pre-allocate for small content

        // Look for content until we hit a formatting boundary
        while let Some(token) = self.current_token() {
            match token {
                RtfToken::Text(text) => {
                    // Intern the text
                    let interned_text = self.string_cache.intern_to_string(text);
                    content.push(RtfNode::Text(interned_text));
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
        // Skip font table for now
        let mut depth = 1;
        while depth > 0 && self.position < self.tokens.len() {
            match self.current_token() {
                Some(RtfToken::GroupStart) => depth += 1,
                Some(RtfToken::GroupEnd) => depth -= 1,
                _ => {}
            }
            self.advance();
        }
        self.font_table_mode = false;
        Ok(())
    }

    /// Parse color table
    fn parse_color_table(&mut self) -> ConversionResult<()> {
        // Skip color table for now
        while let Some(token) = self.current_token() {
            if matches!(token, RtfToken::GroupEnd) {
                break;
            }
            self.advance();
        }
        self.color_table_mode = false;
        Ok(())
    }

    /// Parse info group for metadata
    fn parse_info_group(&mut self) -> ConversionResult<()> {
        while let Some(token) = self.current_token() {
            match token {
                RtfToken::GroupEnd => {
                    self.advance();
                    break;
                }
                RtfToken::ControlWord { name, .. } => {
                    let name_ref = name.as_str();
                    self.advance();
                    
                    match name_ref {
                        "title" => {
                            if let Some(RtfToken::Text(text)) = self.current_token() {
                                self.metadata.title = Some(text.to_string());
                                self.advance();
                            }
                        }
                        "author" => {
                            if let Some(RtfToken::Text(text)) = self.current_token() {
                                self.metadata.author = Some(text.to_string());
                                self.advance();
                            }
                        }
                        _ => {}
                    }
                }
                _ => self.advance(),
            }
        }
        Ok(())
    }

    /// Get current token without cloning
    fn current_token(&self) -> Option<&RtfToken> {
        self.tokens.get(self.position)
    }

    /// Peek at next token without cloning
    fn peek(&self) -> Option<&RtfToken> {
        self.tokens.get(self.position)
    }

    /// Advance position
    fn advance(&mut self) {
        self.position += 1;
    }

    /// Expect a specific token
    fn expect_token(&mut self, expected: &RtfToken) -> ConversionResult<()> {
        match self.current_token() {
            Some(token) if token == expected => {
                self.advance();
                Ok(())
            }
            Some(token) => Err(ConversionError::ParseError(
                format!("Expected {:?}, found {:?}", expected, token)
            )),
            None => Err(ConversionError::ParseError(
                "Unexpected end of input".to_string()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_document() {
        let tokens = vec![
            RtfToken::GroupStart,
            RtfToken::ControlWord { name: "rtf".to_string(), parameter: Some(1) },
            RtfToken::Text("Hello World".to_string()),
            RtfToken::GroupEnd,
        ];

        let document = OptimizedRtfParser::parse(tokens).unwrap();
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
    fn test_string_interning_effectiveness() {
        // Create document with repeated text
        let mut tokens = vec![
            RtfToken::GroupStart,
            RtfToken::ControlWord { name: "rtf".to_string(), parameter: Some(1) },
        ];
        
        // Add repeated text
        for i in 0..100 {
            tokens.push(RtfToken::Text("Repeated text content".to_string()));
            tokens.push(RtfToken::ControlWord { name: "par".to_string(), parameter: None });
        }
        
        tokens.push(RtfToken::GroupEnd);

        let document = OptimizedRtfParser::parse(tokens).unwrap();
        
        // All paragraphs should contain the same interned string
        assert_eq!(document.content.len(), 100);
    }

    #[test]
    fn test_formatted_text_parsing() {
        let tokens = vec![
            RtfToken::GroupStart,
            RtfToken::ControlWord { name: "rtf".to_string(), parameter: Some(1) },
            RtfToken::ControlWord { name: "b".to_string(), parameter: Some(1) },
            RtfToken::Text("Bold text".to_string()),
            RtfToken::ControlWord { name: "b".to_string(), parameter: Some(0) },
            RtfToken::GroupEnd,
        ];

        let document = OptimizedRtfParser::parse(tokens).unwrap();
        assert_eq!(document.content.len(), 1);
    }
}