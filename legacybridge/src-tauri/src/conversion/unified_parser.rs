// Unified Parser - Consolidates secure and standard parsing implementations
//
// This module provides a single parser implementation that replaces
// the duplicate secure_parser.rs and rtf_parser.rs with configurable
// security levels and performance optimizations.

use super::types::{
    ConversionError, ConversionResult, DocumentMetadata, RtfDocument,
    RtfNode, RtfToken, TableCell, TableRow,
};
use super::unified_config::{ConversionConfig, SecurityLevel};
use super::security::is_control_word_safe;
use pulldown_cmark::{Parser as MarkdownParser, Event, Tag, CowStr};
use std::time::{Duration, Instant};

/// Unified parser for RTF and Markdown content
pub struct UnifiedParser {
    config: ConversionConfig,
    start_time: Option<Instant>,
    nodes_processed: usize,
    recursion_depth: usize,
}

impl UnifiedParser {
    /// Create a new parser with the given configuration
    pub fn new(config: ConversionConfig) -> Self {
        let start_time = if config.security_level.enforce_timeout() {
            Some(Instant::now())
        } else {
            None
        };
        
        Self {
            config,
            start_time,
            nodes_processed: 0,
            recursion_depth: 0,
        }
    }
    
    /// Create a parser with default configuration
    pub fn default() -> Self {
        Self::new(ConversionConfig::default())
    }
    
    /// Parse RTF tokens into a document structure
    pub fn parse_rtf(&mut self, tokens: Vec<RtfToken>) -> ConversionResult<RtfDocument> {
        // Initialize parser state
        let mut parser_state = RtfParserState {
            tokens,
            position: 0,
            metadata: DocumentMetadata::default(),
            font_table_mode: false,
            color_table_mode: false,
        };
        
        // Parse the document
        self.parse_rtf_document(&mut parser_state)
    }
    
    /// Parse Markdown content into a document structure
    pub fn parse_markdown(&mut self, content: &str) -> ConversionResult<RtfDocument> {
        // Validate input if required
        if self.config.should_validate() {
            self.validate_input_size(content, "Markdown")?;
        }
        
        let parser = MarkdownParser::new(content);
        let mut converter = MarkdownConverter::new();
        
        for event in parser {
            if self.should_check_limits() {
                self.increment_and_check_limits()?;
            }
            converter.process_event(event)?;
        }
        
        Ok(converter.finalize())
    }
    
    // Security and limit checking methods
    
    fn should_check_limits(&self) -> bool {
        !matches!(self.config.security_level, SecurityLevel::Standard)
    }
    
    fn validate_input_size(&self, input: &str, context: &str) -> ConversionResult<()> {
        if let Some(limits) = self.config.security_level.limits() {
            if input.len() > limits.max_file_size {
                return Err(ConversionError::ValidationError(
                    format!("{} size exceeds maximum allowed: {} > {}", 
                        context, input.len(), limits.max_file_size)
                ));
            }
        }
        Ok(())
    }
    
    fn check_timeout(&self) -> ConversionResult<()> {
        if let (Some(start_time), Some(timeout)) = (self.start_time, self.config.timeout_duration()) {
            if start_time.elapsed() > timeout {
                return Err(ConversionError::ParseError(
                    format!("Parsing timeout exceeded: {:?}", timeout)
                ));
            }
        }
        Ok(())
    }
    
    fn increment_and_check_limits(&mut self) -> ConversionResult<()> {
        self.nodes_processed += 1;
        
        // Check periodically to reduce overhead
        if self.nodes_processed % 100 == 0 {
            self.check_timeout()?;
        }
        
        Ok(())
    }
    
    fn check_recursion_depth(&self) -> ConversionResult<()> {
        if let Some(limits) = self.config.security_level.limits() {
            if self.recursion_depth >= limits.max_nesting_depth {
                return Err(ConversionError::ParseError(
                    format!("Maximum nesting depth {} exceeded", limits.max_nesting_depth)
                ));
            }
        }
        Ok(())
    }
    
    // RTF parsing methods
    
    fn parse_rtf_document(&mut self, state: &mut RtfParserState) -> ConversionResult<RtfDocument> {
        // Expect document to start with a group
        state.expect_token(&RtfToken::GroupStart)?;
        
        // Check for RTF header
        if let Some(RtfToken::ControlWord { name, parameter }) = state.peek() {
            if name == "rtf" && parameter == &Some(1) {
                state.advance();
            } else {
                return Err(ConversionError::ParseError(
                    "Invalid RTF header".to_string()
                ));
            }
        }
        
        // Parse document content
        let content = self.parse_rtf_group_content(state)?;
        
        Ok(RtfDocument {
            metadata: state.metadata.clone(),
            content,
        })
    }
    
    fn parse_rtf_group_content(&mut self, state: &mut RtfParserState) -> ConversionResult<Vec<RtfNode>> {
        // Check recursion depth if security is enabled
        if self.should_check_limits() {
            self.check_recursion_depth()?;
        }
        
        self.recursion_depth += 1;
        let result = self.parse_rtf_group_content_inner(state);
        self.recursion_depth -= 1;
        
        result
    }
    
    fn parse_rtf_group_content_inner(&mut self, state: &mut RtfParserState) -> ConversionResult<Vec<RtfNode>> {
        let mut nodes = Vec::new();
        let mut current_paragraph = Vec::new();
        
        while state.position < state.tokens.len() {
            if self.should_check_limits() {
                self.increment_and_check_limits()?;
            }
            
            match state.current_token() {
                Some(RtfToken::GroupEnd) => {
                    if !current_paragraph.is_empty() {
                        nodes.push(RtfNode::Paragraph(current_paragraph));
                        current_paragraph = Vec::new();
                    }
                    state.advance();
                    break;
                }
                
                Some(RtfToken::GroupStart) => {
                    state.advance();
                    let group_content = self.parse_rtf_group_content(state)?;
                    current_paragraph.extend(group_content);
                }
                
                Some(RtfToken::ControlWord { name, parameter }) => {
                    let name = name.clone();
                    let parameter = *parameter;
                    
                    // Check control word safety if configured
                    if let Some(control_words) = self.config.security_level.control_words() {
                        if !is_control_word_safe(&name, control_words) {
                            if self.config.auto_recovery {
                                // Skip forbidden control word in recovery mode
                                state.advance();
                                continue;
                            } else {
                                return Err(ConversionError::ParseError(
                                    format!("Forbidden control word: \\{}", name)
                                ));
                            }
                        }
                    }
                    
                    state.advance();
                    self.handle_control_word(state, &name, parameter, &mut nodes, &mut current_paragraph)?;
                }
                
                Some(RtfToken::Text(text)) => {
                    current_paragraph.push(RtfNode::Text(text.clone()));
                    state.advance();
                }
                
                Some(_) => {
                    state.advance();
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
    
    fn handle_control_word(
        &mut self,
        state: &mut RtfParserState,
        name: &str,
        parameter: Option<i32>,
        nodes: &mut Vec<RtfNode>,
        current_paragraph: &mut Vec<RtfNode>,
    ) -> ConversionResult<()> {
        match name {
            "par" => {
                if !current_paragraph.is_empty() {
                    nodes.push(RtfNode::Paragraph(current_paragraph.clone()));
                    current_paragraph.clear();
                }
            }
            "b" if parameter != Some(0) => {
                let content = self.parse_formatted_content(state)?;
                current_paragraph.push(RtfNode::Bold(content));
            }
            "i" if parameter != Some(0) => {
                let content = self.parse_formatted_content(state)?;
                current_paragraph.push(RtfNode::Italic(content));
            }
            "ul" => {
                let content = self.parse_formatted_content(state)?;
                current_paragraph.push(RtfNode::Underline(content));
            }
            "line" => current_paragraph.push(RtfNode::LineBreak),
            "page" => nodes.push(RtfNode::PageBreak),
            _ => {
                // Handle other control words based on configuration
                if self.config.logging_enabled {
                    eprintln!("Unhandled control word: \\{}", name);
                }
            }
        }
        Ok(())
    }
    
    fn parse_formatted_content(&mut self, state: &mut RtfParserState) -> ConversionResult<Vec<RtfNode>> {
        let mut content = Vec::new();
        
        while let Some(token) = state.current_token() {
            if self.should_check_limits() {
                self.increment_and_check_limits()?;
            }
            
            match token {
                RtfToken::Text(text) => {
                    content.push(RtfNode::Text(text.clone()));
                    state.advance();
                }
                RtfToken::ControlWord { name, parameter } => {
                    if matches!(name.as_str(), "b" | "i" | "ul" | "ulnone") && *parameter == Some(0) {
                        state.advance();
                        break;
                    }
                    break;
                }
                _ => break,
            }
        }
        
        Ok(content)
    }
}

// Helper structures

struct RtfParserState {
    tokens: Vec<RtfToken>,
    position: usize,
    metadata: DocumentMetadata,
    font_table_mode: bool,
    color_table_mode: bool,
}

impl RtfParserState {
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
                "Unexpected end of input".to_string()
            )),
        }
    }
}

struct MarkdownConverter {
    document: RtfDocument,
    current_paragraph: Vec<RtfNode>,
    formatting_stack: Vec<FormattingState>,
    list_stack: Vec<ListState>,
    current_heading_level: Option<u8>,
}

#[derive(Clone)]
enum FormattingState {
    Bold,
    Italic,
    Code,
}

#[derive(Clone)]
struct ListState {
    level: u8,
    ordered: bool,
}

impl MarkdownConverter {
    fn new() -> Self {
        Self {
            document: RtfDocument {
                metadata: DocumentMetadata::default(),
                content: Vec::new(),
            },
            current_paragraph: Vec::new(),
            formatting_stack: Vec::new(),
            list_stack: Vec::new(),
            current_heading_level: None,
        }
    }
    
    fn process_event(&mut self, event: Event) -> ConversionResult<()> {
        match event {
            Event::Start(tag) => self.handle_start_tag(tag),
            Event::End(tag) => self.handle_end_tag(tag),
            Event::Text(text) => self.handle_text(text),
            Event::Code(code) => self.handle_code(code),
            Event::SoftBreak => self.handle_soft_break(),
            Event::HardBreak => self.handle_hard_break(),
            Event::Rule => self.handle_rule(),
            _ => Ok(()),
        }
    }
    
    fn handle_start_tag(&mut self, tag: Tag) -> ConversionResult<()> {
        match tag {
            Tag::Heading(level, _, _) => {
                self.current_heading_level = Some(level as u8);
            }
            Tag::List(first_item) => {
                let ordered = first_item.is_some();
                let level = self.list_stack.len() as u8;
                self.list_stack.push(ListState { level, ordered });
            }
            Tag::Emphasis => {
                self.formatting_stack.push(FormattingState::Italic);
            }
            Tag::Strong => {
                self.formatting_stack.push(FormattingState::Bold);
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_end_tag(&mut self, tag: Tag) -> ConversionResult<()> {
        match tag {
            Tag::Paragraph => {
                if !self.current_paragraph.is_empty() {
                    let content = std::mem::take(&mut self.current_paragraph);
                    
                    if let Some(level) = self.current_heading_level.take() {
                        self.document.content.push(RtfNode::Heading { level, content });
                    } else if !self.list_stack.is_empty() {
                        let list_state = self.list_stack.last()
                            .expect("List stack should not be empty after check");
                        self.document.content.push(RtfNode::ListItem {
                            level: list_state.level,
                            content,
                        });
                    } else {
                        self.document.content.push(RtfNode::Paragraph(content));
                    }
                }
            }
            Tag::List(_) => {
                self.list_stack.pop();
            }
            Tag::Emphasis => {
                self.formatting_stack.retain(|f| !matches!(f, FormattingState::Italic));
            }
            Tag::Strong => {
                self.formatting_stack.retain(|f| !matches!(f, FormattingState::Bold));
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_text(&mut self, text: CowStr) -> ConversionResult<()> {
        let mut node = RtfNode::Text(text.to_string());
        
        // Apply formatting
        for formatting in self.formatting_stack.iter().rev() {
            node = match formatting {
                FormattingState::Bold => RtfNode::Bold(vec![node]),
                FormattingState::Italic => RtfNode::Italic(vec![node]),
                FormattingState::Code => node, // Keep as text for now
            };
        }
        
        self.current_paragraph.push(node);
        Ok(())
    }
    
    fn handle_code(&mut self, code: CowStr) -> ConversionResult<()> {
        self.current_paragraph.push(RtfNode::Text(code.to_string()));
        Ok(())
    }
    
    fn handle_soft_break(&mut self) -> ConversionResult<()> {
        self.current_paragraph.push(RtfNode::Text(" ".to_string()));
        Ok(())
    }
    
    fn handle_hard_break(&mut self) -> ConversionResult<()> {
        self.current_paragraph.push(RtfNode::LineBreak);
        Ok(())
    }
    
    fn handle_rule(&mut self) -> ConversionResult<()> {
        self.document.content.push(RtfNode::PageBreak);
        Ok(())
    }
    
    fn finalize(mut self) -> RtfDocument {
        if !self.current_paragraph.is_empty() {
            let content = std::mem::take(&mut self.current_paragraph);
            self.document.content.push(RtfNode::Paragraph(content));
        }
        self.document
    }
}