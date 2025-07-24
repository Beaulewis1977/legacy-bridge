// RTF Parser - Parses RTF tokens into a document structure

use super::types::{
    ConversionError, ConversionResult, DocumentMetadata, RtfDocument,
    RtfNode, RtfToken,
};

/// RTF Parser
pub struct RtfParser {
    tokens: Vec<RtfToken>,
    position: usize,
    metadata: DocumentMetadata,
    font_table_mode: bool,
    color_table_mode: bool,
}

impl RtfParser {
    /// Parse RTF tokens into a document structure
    pub fn parse(tokens: Vec<RtfToken>) -> ConversionResult<RtfDocument> {
        let mut parser = RtfParser {
            tokens,
            position: 0,
            metadata: DocumentMetadata::default(),
            font_table_mode: false,
            color_table_mode: false,
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

        Ok(RtfDocument {
            metadata: self.metadata.clone(),
            content,
        })
    }

    /// Parse content within a group
    fn parse_group_content(&mut self) -> ConversionResult<Vec<RtfNode>> {
        let mut nodes = Vec::new();
        let mut current_paragraph = Vec::new();

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

        // Look for content until we hit a formatting boundary
        while let Some(token) = self.current_token() {
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
        let tokens = tokenize(rtf).unwrap();
        let document = RtfParser::parse(tokens).unwrap();
        
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
        let tokens = tokenize(rtf).unwrap();
        let document = RtfParser::parse(tokens).unwrap();
        
        assert_eq!(document.content.len(), 1);
    }
}