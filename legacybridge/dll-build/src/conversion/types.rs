// Common types and structures used throughout the conversion module

use serde::{Deserialize, Serialize};
use std::fmt;

/// Result type for conversion operations
pub type ConversionResult<T> = Result<T, ConversionError>;

/// Errors that can occur during conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversionError {
    /// Error during lexical analysis
    LexerError(String),
    /// Error during parsing
    ParseError(String),
    /// Error during generation
    GenerationError(String),
    /// Feature not yet implemented
    NotImplemented(String),
    /// IO error
    IoError(String),
    /// Invalid input format
    InvalidFormat(String),
    /// Validation error
    ValidationError(String),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionError::LexerError(msg) => write!(f, "Lexer error: {}", msg),
            ConversionError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConversionError::GenerationError(msg) => write!(f, "Generation error: {}", msg),
            ConversionError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            ConversionError::IoError(msg) => write!(f, "IO error: {}", msg),
            ConversionError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ConversionError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ConversionError {}

/// RTF Token types
#[derive(Debug, Clone, PartialEq)]
pub enum RtfToken {
    /// Control word with optional parameter
    ControlWord {
        name: String,
        parameter: Option<i32>,
    },
    /// Control symbol (single character commands)
    ControlSymbol(char),
    /// Plain text content
    Text(String),
    /// Group start {
    GroupStart,
    /// Group end }
    GroupEnd,
    /// Hexadecimal value
    HexValue(u8),
}

/// RTF Document structure
#[derive(Debug, Clone)]
pub struct RtfDocument {
    /// Document metadata
    pub metadata: DocumentMetadata,
    /// Document content as a tree of nodes
    pub content: Vec<RtfNode>,
}

/// Document metadata
#[derive(Debug, Clone, Default)]
pub struct DocumentMetadata {
    /// Default font
    pub default_font: Option<String>,
    /// Character set
    pub charset: String,
    /// Font table
    pub fonts: Vec<FontInfo>,
    /// Color table
    pub colors: Vec<Color>,
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
}

/// Font information
#[derive(Debug, Clone)]
pub struct FontInfo {
    pub id: i32,
    pub name: String,
    pub family: FontFamily,
}

/// Font families
#[derive(Debug, Clone)]
pub enum FontFamily {
    Roman,
    Swiss,
    Modern,
    Script,
    Decorative,
    Technical,
    Bidirectional,
}

/// RGB Color
#[derive(Debug, Clone)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

/// RTF document node
#[derive(Debug, Clone)]
pub enum RtfNode {
    /// Plain text
    Text(String),
    /// Paragraph
    Paragraph(Vec<RtfNode>),
    /// Bold text
    Bold(Vec<RtfNode>),
    /// Italic text
    Italic(Vec<RtfNode>),
    /// Underlined text
    Underline(Vec<RtfNode>),
    /// Heading
    Heading {
        level: u8,
        content: Vec<RtfNode>,
    },
    /// List item
    ListItem {
        level: u8,
        content: Vec<RtfNode>,
    },
    /// Table
    Table {
        rows: Vec<TableRow>,
    },
    /// Line break
    LineBreak,
    /// Page break
    PageBreak,
}

/// Table row
#[derive(Debug, Clone)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

/// Table cell
#[derive(Debug, Clone)]
pub struct TableCell {
    pub content: Vec<RtfNode>,
}