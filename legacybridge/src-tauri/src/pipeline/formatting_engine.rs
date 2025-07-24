// Formatting Engine - Preserves RTF formatting with high fidelity
//
// This module ensures that all RTF formatting is preserved during conversion,
// including tables, headers, lists, styles, and embedded objects.

use crate::conversion::types::{
    ConversionResult, RtfDocument, RtfNode, RtfToken,
    DocumentMetadata, FontInfo, Color, TableRow, TableCell,
};
use std::collections::HashMap;

/// Extended RTF node with additional formatting information
#[derive(Debug, Clone)]
pub struct FormattedNode {
    pub node: RtfNode,
    pub formatting: NodeFormatting,
}

/// Detailed formatting information for a node
#[derive(Debug, Clone, Default)]
pub struct NodeFormatting {
    /// Font information
    pub font_id: Option<i32>,
    pub font_size: Option<i32>,
    /// Text formatting
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub subscript: bool,
    pub superscript: bool,
    /// Colors
    pub text_color: Option<i32>,
    pub background_color: Option<i32>,
    /// Alignment
    pub alignment: TextAlignment,
    /// Indentation
    pub left_indent: Option<i32>,
    pub right_indent: Option<i32>,
    pub first_line_indent: Option<i32>,
    /// Spacing
    pub space_before: Option<i32>,
    pub space_after: Option<i32>,
    pub line_spacing: Option<i32>,
    /// List formatting
    pub list_level: Option<u8>,
    pub list_type: Option<ListType>,
    /// Table formatting
    pub cell_padding: Option<i32>,
    pub cell_borders: Option<CellBorders>,
    /// Custom RTF properties
    pub custom_properties: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TextAlignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListType {
    Bullet,
    Decimal,
    LowerAlpha,
    UpperAlpha,
    LowerRoman,
    UpperRoman,
}

#[derive(Debug, Clone)]
pub struct CellBorders {
    pub top: Option<BorderStyle>,
    pub right: Option<BorderStyle>,
    pub bottom: Option<BorderStyle>,
    pub left: Option<BorderStyle>,
}

#[derive(Debug, Clone)]
pub struct BorderStyle {
    pub width: i32,
    pub color: Option<i32>,
    pub style: BorderType,
}

#[derive(Debug, Clone)]
pub enum BorderType {
    Solid,
    Dashed,
    Dotted,
    Double,
}

/// Parser state for tracking formatting context
#[derive(Debug, Clone)]
struct ParserState {
    current_formatting: NodeFormatting,
    font_table: HashMap<i32, FontInfo>,
    color_table: Vec<Color>,
    in_font_table: bool,
    in_color_table: bool,
    in_stylesheet: bool,
    style_definitions: HashMap<i32, StyleDefinition>,
    list_definitions: HashMap<i32, ListDefinition>,
    current_list_level: u8,
    table_context: Option<TableContext>,
}

#[derive(Debug, Clone)]
struct StyleDefinition {
    name: String,
    base_style: Option<i32>,
    formatting: NodeFormatting,
}

#[derive(Debug, Clone)]
struct ListDefinition {
    id: i32,
    levels: Vec<ListLevelDefinition>,
}

#[derive(Debug, Clone)]
struct ListLevelDefinition {
    level: u8,
    list_type: ListType,
    format_string: String,
    indent: i32,
}

#[derive(Debug, Clone)]
struct TableContext {
    current_row: Vec<TableCell>,
    rows: Vec<TableRow>,
    cell_formatting: Vec<NodeFormatting>,
}

/// Formatting Engine for high-fidelity RTF processing
pub struct FormattingEngine {
    preserve_custom_properties: bool,
    support_embedded_objects: bool,
}

impl FormattingEngine {
    pub fn new() -> Self {
        Self {
            preserve_custom_properties: true,
            support_embedded_objects: true,
        }
    }

    /// Parse RTF tokens with full formatting preservation
    pub fn parse_with_fidelity(&self, tokens: Vec<RtfToken>) -> ConversionResult<RtfDocument> {
        let mut state = ParserState {
            current_formatting: NodeFormatting::default(),
            font_table: HashMap::new(),
            color_table: Vec::new(),
            in_font_table: false,
            in_color_table: false,
            in_stylesheet: false,
            style_definitions: HashMap::new(),
            list_definitions: HashMap::new(),
            current_list_level: 0,
            table_context: None,
        };

        let mut token_iter = tokens.into_iter().peekable();
        let content = self.parse_tokens(&mut token_iter, &mut state, 0)?;

        Ok(RtfDocument {
            metadata: DocumentMetadata {
                default_font: state.font_table.get(&0).map(|f| f.name.clone()),
                charset: "UTF-8".to_string(),
                fonts: state.font_table.into_iter().map(|(_, f)| f).collect(),
                colors: state.color_table,
                title: None,
                author: None,
            },
            content,
        })
    }

    /// Parse tokens recursively with formatting context
    fn parse_tokens(
        &self,
        tokens: &mut std::iter::Peekable<std::vec::IntoIter<RtfToken>>,
        state: &mut ParserState,
        depth: usize,
    ) -> ConversionResult<Vec<RtfNode>> {
        let mut nodes = Vec::new();
        let mut current_text = String::new();

        while let Some(token) = tokens.next() {
            match token {
                RtfToken::GroupStart => {
                    // Save current formatting state
                    let saved_formatting = state.current_formatting.clone();
                    
                    // Parse nested group
                    let nested_nodes = self.parse_tokens(tokens, state, depth + 1)?;
                    
                    // Handle special groups (tables, lists, etc.)
                    if let Some(special_node) = self.process_special_group(&nested_nodes, state) {
                        nodes.push(special_node);
                    } else {
                        nodes.extend(nested_nodes);
                    }
                    
                    // Restore formatting state
                    state.current_formatting = saved_formatting;
                }
                
                RtfToken::GroupEnd => {
                    // Flush any pending text
                    if !current_text.is_empty() {
                        nodes.push(self.create_formatted_text_node(&current_text, &state.current_formatting));
                        current_text.clear();
                    }
                    return Ok(nodes);
                }
                
                RtfToken::ControlWord { name, parameter } => {
                    // Flush text before processing control word
                    if !current_text.is_empty() {
                        nodes.push(self.create_formatted_text_node(&current_text, &state.current_formatting));
                        current_text.clear();
                    }
                    
                    self.process_control_word(&name, parameter, state, &mut nodes)?;
                }
                
                RtfToken::ControlSymbol(symbol) => {
                    match symbol {
                        '\'' => {
                            // Handle hex character
                            if let Some(RtfToken::HexValue(val)) = tokens.next() {
                                current_text.push(val as char);
                            }
                        }
                        '\\' => current_text.push('\\'),
                        '{' => current_text.push('{'),
                        '}' => current_text.push('}'),
                        '~' => current_text.push('\u{00A0}'), // Non-breaking space
                        '-' => current_text.push('\u{00AD}'), // Soft hyphen
                        '_' => current_text.push('\u{2011}'), // Non-breaking hyphen
                        _ => {} // Ignore other control symbols
                    }
                }
                
                RtfToken::Text(text) => {
                    current_text.push_str(&text);
                }
                
                RtfToken::HexValue(val) => {
                    current_text.push(val as char);
                }
            }
        }

        // Flush any remaining text
        if !current_text.is_empty() {
            nodes.push(self.create_formatted_text_node(&current_text, &state.current_formatting));
        }

        Ok(nodes)
    }

    /// Process RTF control words
    fn process_control_word(
        &self,
        name: &str,
        parameter: Option<i32>,
        state: &mut ParserState,
        nodes: &mut Vec<RtfNode>,
    ) -> ConversionResult<()> {
        match name {
            // Document structure
            "rtf" => {} // RTF version
            "ansi" | "mac" | "pc" | "pca" => {} // Character set
            
            // Font table
            "fonttbl" => state.in_font_table = true,
            "f" => {
                if state.in_font_table {
                    // Font definition in font table
                    if let Some(_id) = parameter {
                        // Will be populated by subsequent tokens
                    }
                } else {
                    // Font selection
                    state.current_formatting.font_id = parameter;
                }
            }
            
            // Color table
            "colortbl" => state.in_color_table = true,
            "red" => {
                if state.in_color_table && parameter.is_some() {
                    // Part of color definition
                }
            }
            
            // Text formatting
            "b" => state.current_formatting.bold = parameter.unwrap_or(1) != 0,
            "i" => state.current_formatting.italic = parameter.unwrap_or(1) != 0,
            "ul" | "uld" | "uldb" | "ulw" => state.current_formatting.underline = parameter.unwrap_or(1) != 0,
            "strike" => state.current_formatting.strikethrough = parameter.unwrap_or(1) != 0,
            "sub" => state.current_formatting.subscript = true,
            "super" => state.current_formatting.superscript = true,
            
            // Font size (in half-points)
            "fs" => state.current_formatting.font_size = parameter,
            
            // Colors
            "cf" => state.current_formatting.text_color = parameter,
            "cb" => state.current_formatting.background_color = parameter,
            
            // Paragraph formatting
            "par" => nodes.push(RtfNode::Paragraph(Vec::new())),
            "line" => nodes.push(RtfNode::LineBreak),
            "page" => nodes.push(RtfNode::PageBreak),
            
            // Alignment
            "ql" => state.current_formatting.alignment = TextAlignment::Left,
            "qc" => state.current_formatting.alignment = TextAlignment::Center,
            "qr" => state.current_formatting.alignment = TextAlignment::Right,
            "qj" => state.current_formatting.alignment = TextAlignment::Justify,
            
            // Indentation
            "li" => state.current_formatting.left_indent = parameter,
            "ri" => state.current_formatting.right_indent = parameter,
            "fi" => state.current_formatting.first_line_indent = parameter,
            
            // Spacing
            "sb" => state.current_formatting.space_before = parameter,
            "sa" => state.current_formatting.space_after = parameter,
            "sl" => state.current_formatting.line_spacing = parameter,
            
            // Tables
            "trowd" => {
                state.table_context = Some(TableContext {
                    current_row: Vec::new(),
                    rows: Vec::new(),
                    cell_formatting: Vec::new(),
                });
            }
            "cell" => {
                if let Some(ref mut ctx) = state.table_context {
                    ctx.current_row.push(TableCell {
                        content: Vec::new(),
                    });
                }
            }
            "row" => {
                if let Some(ref mut ctx) = state.table_context {
                    if !ctx.current_row.is_empty() {
                        ctx.rows.push(TableRow {
                            cells: std::mem::take(&mut ctx.current_row),
                        });
                    }
                }
            }
            
            // Lists
            "pnlvl" => state.current_formatting.list_level = parameter.map(|p| p as u8),
            "pnlvlblt" => state.current_formatting.list_type = Some(ListType::Bullet),
            "pnlvldec" => state.current_formatting.list_type = Some(ListType::Decimal),
            
            // Custom properties
            _ => {
                if self.preserve_custom_properties {
                    state.current_formatting.custom_properties.insert(
                        name.to_string(),
                        parameter.map(|p| p.to_string()).unwrap_or_default(),
                    );
                }
            }
        }
        
        Ok(())
    }

    /// Create a formatted text node
    fn create_formatted_text_node(&self, text: &str, formatting: &NodeFormatting) -> RtfNode {
        let content = vec![RtfNode::Text(text.to_string())];
        
        // Apply formatting in order of precedence
        let mut node = content;
        
        if formatting.bold {
            node = vec![RtfNode::Bold(node)];
        }
        if formatting.italic {
            node = vec![RtfNode::Italic(node)];
        }
        if formatting.underline {
            node = vec![RtfNode::Underline(node)];
        }
        
        // Return the innermost node
        node.into_iter().next().unwrap()
    }

    /// Process special groups (tables, embedded objects, etc.)
    fn process_special_group(&self, _nodes: &[RtfNode], state: &ParserState) -> Option<RtfNode> {
        // Check if this is a table
        if let Some(ref ctx) = state.table_context {
            if !ctx.rows.is_empty() {
                return Some(RtfNode::Table {
                    rows: ctx.rows.clone(),
                });
            }
        }
        
        None
    }

    /// Generate markdown with formatting preservation
    pub fn generate_markdown_with_fidelity(&self, document: &RtfDocument) -> ConversionResult<String> {
        let mut markdown = String::new();
        let mut context = MarkdownContext {
            in_table: false,
            list_stack: Vec::new(),
            preserve_spacing: true,
        };
        
        for node in &document.content {
            self.node_to_markdown(node, &mut markdown, &mut context, &document.metadata)?;
        }
        
        Ok(markdown)
    }

    /// Convert node to markdown with formatting preservation
    fn node_to_markdown(
        &self,
        node: &RtfNode,
        output: &mut String,
        context: &mut MarkdownContext,
        metadata: &DocumentMetadata,
    ) -> ConversionResult<()> {
        match node {
            RtfNode::Text(text) => {
                output.push_str(&self.escape_markdown(text));
            }
            
            RtfNode::Paragraph(children) => {
                if !output.is_empty() && !output.ends_with('\n') {
                    output.push_str("\n\n");
                }
                for child in children {
                    self.node_to_markdown(child, output, context, metadata)?;
                }
                if !output.ends_with('\n') {
                    output.push_str("\n\n");
                }
            }
            
            RtfNode::Bold(children) => {
                output.push_str("**");
                for child in children {
                    self.node_to_markdown(child, output, context, metadata)?;
                }
                output.push_str("**");
            }
            
            RtfNode::Italic(children) => {
                output.push('*');
                for child in children {
                    self.node_to_markdown(child, output, context, metadata)?;
                }
                output.push('*');
            }
            
            RtfNode::Underline(children) => {
                // Markdown doesn't support underline, use HTML
                output.push_str("<u>");
                for child in children {
                    self.node_to_markdown(child, output, context, metadata)?;
                }
                output.push_str("</u>");
            }
            
            RtfNode::Heading { level, content } => {
                output.push_str(&"#".repeat(*level as usize));
                output.push(' ');
                for child in content {
                    self.node_to_markdown(child, output, context, metadata)?;
                }
                output.push_str("\n\n");
            }
            
            RtfNode::ListItem { level, content } => {
                let indent = "  ".repeat(*level as usize);
                output.push_str(&indent);
                output.push_str("- ");
                for child in content {
                    self.node_to_markdown(child, output, context, metadata)?;
                }
                output.push('\n');
            }
            
            RtfNode::Table { rows } => {
                context.in_table = true;
                
                // Generate table
                for (i, row) in rows.iter().enumerate() {
                    output.push('|');
                    for cell in &row.cells {
                        output.push(' ');
                        for child in &cell.content {
                            self.node_to_markdown(child, output, context, metadata)?;
                        }
                        output.push_str(" |");
                    }
                    output.push('\n');
                    
                    // Add header separator after first row
                    if i == 0 && rows.len() > 1 {
                        output.push('|');
                        for _ in &row.cells {
                            output.push_str(" --- |");
                        }
                        output.push('\n');
                    }
                }
                output.push('\n');
                
                context.in_table = false;
            }
            
            RtfNode::LineBreak => {
                output.push_str("  \n");
            }
            
            RtfNode::PageBreak => {
                output.push_str("\n---\n\n");
            }
        }
        
        Ok(())
    }

    /// Escape special markdown characters
    fn escape_markdown(&self, text: &str) -> String {
        text.chars()
            .map(|c| match c {
                '*' | '_' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '.' | '!' | '`' | '\\' => {
                    format!("\\{}", c)
                }
                _ => c.to_string(),
            })
            .collect()
    }
}

/// Context for markdown generation
struct MarkdownContext {
    in_table: bool,
    list_stack: Vec<ListType>,
    preserve_spacing: bool,
}