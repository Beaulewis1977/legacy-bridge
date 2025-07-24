// Secure RTF Parser with enhanced security controls
//
// This module provides a security-hardened RTF parser that prevents
// various attack vectors including stack overflow, resource exhaustion,
// and malicious content injection.

use super::types::{
    ConversionError, ConversionResult, DocumentMetadata, RtfDocument,
    RtfNode, RtfToken,
};
use super::security::{SecurityLimits, ControlWordSecurity, is_control_word_safe};
use std::time::{Duration, Instant};

/// Security-enhanced RTF Parser
pub struct SecureRtfParser {
    tokens: Vec<RtfToken>,
    position: usize,
    metadata: DocumentMetadata,
    font_table_mode: bool,
    color_table_mode: bool,
    recursion_depth: usize,
    security_limits: SecurityLimits,
    control_word_security: ControlWordSecurity,
    start_time: Instant,
    nodes_processed: usize,
}

impl SecureRtfParser {
    /// Parse RTF tokens with security controls
    pub fn parse_with_security(
        tokens: Vec<RtfToken>,
        security_limits: SecurityLimits,
        control_word_security: ControlWordSecurity,
    ) -> ConversionResult<RtfDocument> {
        let mut parser = SecureRtfParser {
            tokens,
            position: 0,
            metadata: DocumentMetadata::default(),
            font_table_mode: false,
            color_table_mode: false,
            recursion_depth: 0,
            security_limits,
            control_word_security,
            start_time: Instant::now(),
            nodes_processed: 0,
        };

        parser.parse_document()
    }

    /// Parse with default security settings
    pub fn parse(tokens: Vec<RtfToken>) -> ConversionResult<RtfDocument> {
        Self::parse_with_security(
            tokens,
            SecurityLimits::default(),
            ControlWordSecurity::default(),
        )
    }

    /// Check if parsing timeout has been exceeded
    fn check_timeout(&self) -> ConversionResult<()> {
        let elapsed = self.start_time.elapsed();
        if elapsed > Duration::from_secs(self.security_limits.parsing_timeout_secs) {
            return Err(ConversionError::ParseError(
                format!("Parsing timeout exceeded ({} seconds)", self.security_limits.parsing_timeout_secs)
            ));
        }
        Ok(())
    }

    /// Increment and check node processing limit
    fn increment_nodes(&mut self) -> ConversionResult<()> {
        self.nodes_processed += 1;
        
        // Check every 100 nodes to avoid excessive overhead
        if self.nodes_processed % 100 == 0 {
            self.check_timeout()?;
        }
        
        Ok(())
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

    /// Parse content within a group with security checks
    fn parse_group_content(&mut self) -> ConversionResult<Vec<RtfNode>> {
        // Security: Check recursion depth
        if self.recursion_depth >= self.security_limits.max_nesting_depth {
            return Err(ConversionError::ParseError(
                format!("Maximum nesting depth {} exceeded", self.security_limits.max_nesting_depth)
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

        while self.position < self.tokens.len() {
            self.increment_nodes()?;
            
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
                    
                    // Security: Check if control word is allowed
                    if !is_control_word_safe(&name, &self.control_word_security) {
                        return Err(ConversionError::ParseError(
                            format!("Forbidden control word: \\{}", name)
                        ));
                    }
                    
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
                            // Ignore other control words (that passed security check)
                        }
                    }
                }
                Some(RtfToken::ControlSymbol(_)) => {
                    // Handle control symbols if needed
                    self.advance();
                }
                Some(RtfToken::Text(text)) => {
                    // Security: Text size is already limited by the lexer
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

    /// Parse formatted content with security limits
    fn parse_formatted_content(&mut self) -> ConversionResult<Vec<RtfNode>> {
        let mut content = Vec::new();

        // Look for content until we hit a formatting boundary
        while let Some(token) = self.current_token() {
            self.increment_nodes()?;
            
            match token {
                RtfToken::Text(text) => {
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

    /// Parse font table (stub - implement with security checks)
    fn parse_font_table(&mut self) -> ConversionResult<()> {
        // TODO: Implement full font table parsing with security limits
        self.font_table_mode = false;
        Ok(())
    }

    /// Parse color table (stub - implement with security checks)
    fn parse_color_table(&mut self) -> ConversionResult<()> {
        // TODO: Implement full color table parsing with security limits
        self.color_table_mode = false;
        Ok(())
    }

    /// Parse info group (stub - implement with security checks)
    fn parse_info_group(&mut self) -> ConversionResult<()> {
        // TODO: Implement info group parsing with security limits
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
    fn test_parse_with_security() {
        let rtf = r"{\rtf1 Hello World\par}";
        let tokens = tokenize(rtf).unwrap();
        let document = SecureRtfParser::parse(tokens).unwrap();
        
        assert_eq!(document.content.len(), 1);
    }

    #[test]
    fn test_forbidden_control_word() {
        let rtf = r"{\rtf1 {\object\objdata} Hello\par}";
        let tokens = tokenize(rtf).unwrap();
        let result = SecureRtfParser::parse(tokens);
        
        assert!(result.is_err());
        match result {
            Err(ConversionError::ParseError(msg)) => {
                assert!(msg.contains("Forbidden control word"));
            }
            _ => panic!("Expected parse error for forbidden control word"),
        }
    }

    #[test]
    fn test_max_nesting_depth() {
        // Create deeply nested RTF
        let mut rtf = String::from(r"{\rtf1 ");
        for _ in 0..60 {
            rtf.push('{');
        }
        rtf.push_str("Hello");
        for _ in 0..60 {
            rtf.push('}');
        }
        rtf.push('}');
        
        let tokens = tokenize(&rtf).unwrap();
        let result = SecureRtfParser::parse(tokens);
        
        assert!(result.is_err());
        match result {
            Err(ConversionError::ParseError(msg)) => {
                assert!(msg.contains("Maximum nesting depth"));
            }
            _ => panic!("Expected parse error for excessive nesting"),
        }
    }
}