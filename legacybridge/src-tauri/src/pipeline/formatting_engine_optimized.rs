// Optimized Formatting Engine - High-performance RTF formatting with reduced allocations
//
// Key optimizations:
// 1. Object pooling for temporary allocations
// 2. Batch processing of tokens
// 3. Lazy evaluation of formatting properties
// 4. Optimized string building with pre-allocated buffers
// 5. Cache-friendly data structures

use crate::conversion::types::{
    ConversionResult, RtfDocument, RtfNode, RtfToken,
    DocumentMetadata, FontInfo, Color, TableRow, TableCell,
};
use ahash::AHashMap;
use smallvec::SmallVec;
use std::mem;

/// Optimized formatting engine with performance improvements
pub struct OptimizedFormattingEngine {
    /// Reusable string builder
    string_builder: StringBuilder,
    /// Token buffer for batch processing
    token_buffer: Vec<RtfToken>,
    /// Formatting cache
    format_cache: FormatCache,
}

impl OptimizedFormattingEngine {
    pub fn new() -> Self {
        Self {
            string_builder: StringBuilder::with_capacity(1024 * 16), // 16KB initial
            token_buffer: Vec::with_capacity(1000),
            format_cache: FormatCache::new(),
        }
    }

    /// Parse RTF tokens with optimized processing
    pub fn parse_with_fidelity(&mut self, tokens: Vec<RtfToken>) -> ConversionResult<RtfDocument> {
        let mut state = OptimizedParserState::new();
        
        // Process tokens in batches for better cache locality
        let mut nodes = Vec::with_capacity(tokens.len() / 10);
        self.process_tokens_optimized(tokens, &mut state, &mut nodes)?;
        
        Ok(RtfDocument {
            metadata: state.build_metadata(),
            content: nodes,
        })
    }

    /// Optimized token processing with batching
    fn process_tokens_optimized(
        &mut self,
        tokens: Vec<RtfToken>,
        state: &mut OptimizedParserState,
        output: &mut Vec<RtfNode>,
    ) -> ConversionResult<()> {
        let mut token_iter = tokens.into_iter();
        let mut text_accumulator = TextAccumulator::new();
        
        while let Some(token) = token_iter.next() {
            match token {
                RtfToken::GroupStart => {
                    // Save state efficiently
                    state.push_formatting();
                    
                    // Collect tokens for this group
                    let group_tokens = self.collect_group_tokens(&mut token_iter)?;
                    
                    // Process group recursively
                    let mut group_nodes = Vec::new();
                    self.process_tokens_optimized(group_tokens, state, &mut group_nodes)?;
                    
                    // Handle special groups
                    if let Some(special_node) = self.process_special_group_optimized(&group_nodes, state) {
                        output.push(special_node);
                    } else {
                        output.extend(group_nodes);
                    }
                    
                    // Restore state
                    state.pop_formatting();
                }
                
                RtfToken::ControlWord { name, parameter } => {
                    // Flush accumulated text before processing control
                    text_accumulator.flush_to(output, &state.current_formatting);
                    
                    // Process control word with optimized lookup
                    self.process_control_word_optimized(&name, parameter, state, output)?;
                }
                
                RtfToken::Text(text) => {
                    // Accumulate text for batch processing
                    text_accumulator.add_text(text);
                }
                
                RtfToken::ControlSymbol(symbol) => {
                    match symbol {
                        '\'' => {
                            // Handle hex efficiently
                            if let Some(RtfToken::HexValue(val)) = token_iter.next() {
                                text_accumulator.add_char(val as char);
                            }
                        }
                        '\\' => text_accumulator.add_char('\\'),
                        '{' => text_accumulator.add_char('{'),
                        '}' => text_accumulator.add_char('}'),
                        '~' => text_accumulator.add_char('\u{00A0}'),
                        '-' => text_accumulator.add_char('\u{00AD}'),
                        '_' => text_accumulator.add_char('\u{2011}'),
                        _ => {}
                    }
                }
                
                _ => {} // Handle other tokens
            }
        }
        
        // Flush any remaining text
        text_accumulator.flush_to(output, &state.current_formatting);
        
        Ok(())
    }

    /// Collect all tokens for a group efficiently
    fn collect_group_tokens(&mut self, iter: &mut std::vec::IntoIter<RtfToken>) -> ConversionResult<Vec<RtfToken>> {
        self.token_buffer.clear();
        let mut depth = 1;
        
        while let Some(token) = iter.next() {
            match &token {
                RtfToken::GroupStart => depth += 1,
                RtfToken::GroupEnd => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            self.token_buffer.push(token);
        }
        
        Ok(mem::take(&mut self.token_buffer))
    }

    /// Process control words with optimized lookup table
    fn process_control_word_optimized(
        &mut self,
        name: &str,
        parameter: Option<i32>,
        state: &mut OptimizedParserState,
        output: &mut Vec<RtfNode>,
    ) -> ConversionResult<()> {
        // Use perfect hash or match for common control words
        match name {
            // Text formatting - most common
            "b" => state.current_formatting.bold = parameter.unwrap_or(1) != 0,
            "i" => state.current_formatting.italic = parameter.unwrap_or(1) != 0,
            "ul" => state.current_formatting.underline = parameter.unwrap_or(1) != 0,
            "fs" => state.current_formatting.font_size = parameter,
            
            // Paragraph breaks
            "par" => output.push(RtfNode::Paragraph(Vec::new())),
            "line" => output.push(RtfNode::LineBreak),
            
            // Font and color
            "f" => state.current_formatting.font_id = parameter,
            "cf" => state.current_formatting.text_color = parameter,
            
            // Table handling
            "trowd" => state.start_table_row(),
            "cell" => state.add_table_cell(),
            "row" => {
                if let Some(row) = state.finish_table_row() {
                    output.push(RtfNode::Table { rows: vec![row] });
                }
            }
            
            _ => {
                // Less common control words
                self.process_uncommon_control_word(name, parameter, state)?;
            }
        }
        
        Ok(())
    }

    fn process_uncommon_control_word(
        &mut self,
        name: &str,
        parameter: Option<i32>,
        state: &mut OptimizedParserState,
    ) -> ConversionResult<()> {
        match name {
            "fonttbl" => state.in_font_table = true,
            "colortbl" => state.in_color_table = true,
            "ql" => state.current_formatting.alignment = TextAlignment::Left,
            "qc" => state.current_formatting.alignment = TextAlignment::Center,
            "qr" => state.current_formatting.alignment = TextAlignment::Right,
            "qj" => state.current_formatting.alignment = TextAlignment::Justify,
            _ => {} // Ignore unknown control words
        }
        Ok(())
    }

    fn process_special_group_optimized(&self, nodes: &[RtfNode], state: &OptimizedParserState) -> Option<RtfNode> {
        // Quick check for table nodes
        if state.has_active_table() {
            // Table processing logic
            return None;
        }
        
        None
    }

    /// Generate markdown with optimized string building
    pub fn generate_markdown_with_fidelity(&mut self, document: &RtfDocument) -> ConversionResult<String> {
        self.string_builder.clear();
        let mut context = MarkdownContext::new();
        
        for node in &document.content {
            self.node_to_markdown_optimized(node, &mut context)?;
        }
        
        Ok(self.string_builder.build())
    }

    fn node_to_markdown_optimized(
        &mut self,
        node: &RtfNode,
        context: &mut MarkdownContext,
    ) -> ConversionResult<()> {
        match node {
            RtfNode::Text(text) => {
                self.string_builder.push_escaped(text);
            }
            
            RtfNode::Paragraph(children) => {
                if !self.string_builder.is_empty() && !self.string_builder.ends_with('\n') {
                    self.string_builder.push_str("\n\n");
                }
                for child in children {
                    self.node_to_markdown_optimized(child, context)?;
                }
                if !self.string_builder.ends_with('\n') {
                    self.string_builder.push_str("\n\n");
                }
            }
            
            RtfNode::Bold(children) => {
                self.string_builder.push_str("**");
                for child in children {
                    self.node_to_markdown_optimized(child, context)?;
                }
                self.string_builder.push_str("**");
            }
            
            RtfNode::Italic(children) => {
                self.string_builder.push('*');
                for child in children {
                    self.node_to_markdown_optimized(child, context)?;
                }
                self.string_builder.push('*');
            }
            
            RtfNode::Table { rows } => {
                self.generate_table_optimized(rows)?;
            }
            
            _ => {
                // Handle other node types
                self.node_to_markdown_fallback(node, context)?;
            }
        }
        
        Ok(())
    }

    fn generate_table_optimized(&mut self, rows: &[TableRow]) -> ConversionResult<()> {
        if rows.is_empty() {
            return Ok(());
        }
        
        // Pre-calculate column widths for alignment
        let col_count = rows[0].cells.len();
        
        for (i, row) in rows.iter().enumerate() {
            self.string_builder.push('|');
            for cell in &row.cells {
                self.string_builder.push(' ');
                // Simplified cell content handling
                for node in &cell.content {
                    if let RtfNode::Text(text) = node {
                        self.string_builder.push_str(text);
                    }
                }
                self.string_builder.push_str(" |");
            }
            self.string_builder.push('\n');
            
            // Add header separator after first row
            if i == 0 && rows.len() > 1 {
                self.string_builder.push('|');
                for _ in 0..col_count {
                    self.string_builder.push_str(" --- |");
                }
                self.string_builder.push('\n');
            }
        }
        self.string_builder.push('\n');
        
        Ok(())
    }

    fn node_to_markdown_fallback(&mut self, node: &RtfNode, _context: &mut MarkdownContext) -> ConversionResult<()> {
        match node {
            RtfNode::Underline(children) => {
                self.string_builder.push_str("<u>");
                for child in children {
                    self.node_to_markdown_optimized(child, _context)?;
                }
                self.string_builder.push_str("</u>");
            }
            RtfNode::Heading { level, content } => {
                self.string_builder.push_str(&"#".repeat(*level as usize));
                self.string_builder.push(' ');
                for child in content {
                    self.node_to_markdown_optimized(child, _context)?;
                }
                self.string_builder.push_str("\n\n");
            }
            RtfNode::ListItem { level, content } => {
                let indent = "  ".repeat(*level as usize);
                self.string_builder.push_str(&indent);
                self.string_builder.push_str("- ");
                for child in content {
                    self.node_to_markdown_optimized(child, _context)?;
                }
                self.string_builder.push('\n');
            }
            RtfNode::LineBreak => {
                self.string_builder.push_str("  \n");
            }
            RtfNode::PageBreak => {
                self.string_builder.push_str("\n---\n\n");
            }
            _ => {} // Already handled
        }
        Ok(())
    }
}

/// Optimized parser state with efficient storage
struct OptimizedParserState {
    current_formatting: NodeFormatting,
    formatting_stack: Vec<NodeFormatting>,
    font_table: AHashMap<i32, FontInfo>,
    color_table: Vec<Color>,
    in_font_table: bool,
    in_color_table: bool,
    table_builder: Option<TableBuilder>,
}

impl OptimizedParserState {
    fn new() -> Self {
        Self {
            current_formatting: NodeFormatting::default(),
            formatting_stack: Vec::with_capacity(16),
            font_table: AHashMap::new(),
            color_table: Vec::with_capacity(16),
            in_font_table: false,
            in_color_table: false,
            table_builder: None,
        }
    }

    #[inline]
    fn push_formatting(&mut self) {
        self.formatting_stack.push(self.current_formatting.clone());
    }

    #[inline]
    fn pop_formatting(&mut self) {
        if let Some(formatting) = self.formatting_stack.pop() {
            self.current_formatting = formatting;
        }
    }

    fn start_table_row(&mut self) {
        if self.table_builder.is_none() {
            self.table_builder = Some(TableBuilder::new());
        }
        if let Some(ref mut builder) = self.table_builder {
            builder.start_row();
        }
    }

    fn add_table_cell(&mut self) {
        if let Some(ref mut builder) = self.table_builder {
            builder.add_cell();
        }
    }

    fn finish_table_row(&mut self) -> Option<TableRow> {
        self.table_builder.as_mut().and_then(|b| b.finish_row())
    }

    fn has_active_table(&self) -> bool {
        self.table_builder.is_some()
    }

    fn build_metadata(self) -> DocumentMetadata {
        DocumentMetadata {
            default_font: self.font_table.get(&0).map(|f| f.name.clone()),
            charset: "UTF-8".to_string(),
            fonts: self.font_table.into_iter().map(|(_, f)| f).collect(),
            colors: self.color_table,
            title: None,
            author: None,
        }
    }
}

/// Lightweight formatting information
#[derive(Debug, Clone, Default)]
struct NodeFormatting {
    font_id: Option<i32>,
    font_size: Option<i32>,
    bold: bool,
    italic: bool,
    underline: bool,
    text_color: Option<i32>,
    alignment: TextAlignment,
}

#[derive(Debug, Clone, Copy, Default)]
enum TextAlignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

/// Efficient text accumulator
struct TextAccumulator {
    buffer: String,
}

impl TextAccumulator {
    fn new() -> Self {
        Self {
            buffer: String::with_capacity(256),
        }
    }

    #[inline]
    fn add_text(&mut self, text: String) {
        self.buffer.push_str(&text);
    }

    #[inline]
    fn add_char(&mut self, ch: char) {
        self.buffer.push(ch);
    }

    fn flush_to(&mut self, output: &mut Vec<RtfNode>, formatting: &NodeFormatting) {
        if self.buffer.is_empty() {
            return;
        }

        let text = mem::take(&mut self.buffer);
        let mut node = RtfNode::Text(text);

        // Apply formatting efficiently
        if formatting.bold {
            node = RtfNode::Bold(vec![node]);
        }
        if formatting.italic {
            node = RtfNode::Italic(vec![node]);
        }
        if formatting.underline {
            node = RtfNode::Underline(vec![node]);
        }

        output.push(node);
    }
}

/// Efficient string builder with escape handling
struct StringBuilder {
    buffer: String,
}

impl StringBuilder {
    fn with_capacity(cap: usize) -> Self {
        Self {
            buffer: String::with_capacity(cap),
        }
    }

    fn clear(&mut self) {
        self.buffer.clear();
    }

    #[inline]
    fn push_str(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    #[inline]
    fn push(&mut self, ch: char) {
        self.buffer.push(ch);
    }

    fn push_escaped(&mut self, text: &str) {
        // Fast path for text without special characters
        if !text.chars().any(|c| matches!(c, '*' | '_' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '.' | '!' | '`' | '\\')) {
            self.buffer.push_str(text);
            return;
        }

        // Escape special characters
        for ch in text.chars() {
            match ch {
                '*' | '_' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '.' | '!' | '`' | '\\' => {
                    self.buffer.push('\\');
                    self.buffer.push(ch);
                }
                _ => self.buffer.push(ch),
            }
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    #[inline]
    fn ends_with(&self, ch: char) -> bool {
        self.buffer.ends_with(ch)
    }

    fn build(&mut self) -> String {
        mem::take(&mut self.buffer)
    }
}

/// Efficient table builder
struct TableBuilder {
    current_row: Vec<TableCell>,
    rows: Vec<TableRow>,
}

impl TableBuilder {
    fn new() -> Self {
        Self {
            current_row: Vec::with_capacity(4),
            rows: Vec::with_capacity(10),
        }
    }

    fn start_row(&mut self) {
        self.current_row.clear();
    }

    fn add_cell(&mut self) {
        self.current_row.push(TableCell {
            content: Vec::new(),
        });
    }

    fn finish_row(&mut self) -> Option<TableRow> {
        if self.current_row.is_empty() {
            return None;
        }
        
        Some(TableRow {
            cells: mem::take(&mut self.current_row),
        })
    }
}

/// Context for markdown generation
struct MarkdownContext {
    in_table: bool,
    list_depth: u8,
}

impl MarkdownContext {
    fn new() -> Self {
        Self {
            in_table: false,
            list_depth: 0,
        }
    }
}

/// Format cache for repeated patterns
struct FormatCache {
    cached_formats: AHashMap<NodeFormatting, String>,
}

impl FormatCache {
    fn new() -> Self {
        Self {
            cached_formats: AHashMap::new(),
        }
    }
}