# RTF Format Research Report for LegacyBridge

## Executive Summary

This report provides comprehensive research on the Rich Text Format (RTF) specification for implementing a lightweight parser in Rust, specifically focused on features commonly used in legacy VB6/VFP9 business systems. The report covers RTF structure, parsing algorithms, and conversion strategies to Markdown.

## 1. RTF Format Specification

### 1.1 Basic RTF Structure and Syntax

RTF is a document format developed by Microsoft that uses ASCII text with control codes to represent formatting. Key characteristics:

- **File Structure**: An entire RTF file is a single group enclosed in curly braces `{}`
- **Control Words**: Begin with a backslash `\` followed by alphabetic characters
- **Parameters**: Control words can have numeric parameters (e.g., `\fs24` for font size 24)
- **Groups**: Nested structures using `{}` to scope formatting properties
- **Plain Text**: Regular text appears between control sequences

Basic RTF document structure:
```rtf
{\rtf1\ansi\deff0 {\fonttbl{\f0 Times New Roman;}}
\f0\fs24 Hello, World!
}
```

### 1.2 Common Control Codes in Legacy Systems

VB6 and VFP9 systems commonly use these RTF control codes:

#### Document Level
- `\rtf1` - RTF version 1
- `\ansi` - ANSI character set
- `\deff0` - Default font (index 0)

#### Character Formatting
- `\b` - Bold (toggle)
- `\i` - Italic (toggle)
- `\ul` - Underline
- `\fs[N]` - Font size (N in half-points)
- `\f[N]` - Font number from font table

#### Paragraph Formatting
- `\par` - New paragraph
- `\line` - Line break
- `\ql` - Left align
- `\qc` - Center align
- `\qr` - Right align
- `\qj` - Justify (common in VFP9 reports)

#### Special Characters
- `\'XX` - Character by hex code (e.g., `\'a9` for ©)
- `\{` - Literal left brace
- `\}` - Literal right brace
- `\\` - Literal backslash
- `\~` - Non-breaking space
- `\-` - Optional hyphen

### 1.3 Table Formatting Codes

RTF tables are implemented as paragraph properties, not as a separate structure:

#### Core Table Control Words
```rtf
\trowd              - Start table row definition
\cellx[N]           - Cell boundary at position N (in twips)
\intbl              - Paragraph is inside table
\cell               - End of cell content
\row                - End of table row
```

#### Table Formatting
```rtf
\trgaph[N]          - Half the space between cells (N in twips)
\clbrdrt            - Cell border top
\clbrdrl            - Cell border left
\clbrdrb            - Cell border bottom
\clbrdrr            - Cell border right
\brdrs              - Single border style
```

Example of a 3-column table:
```rtf
{\rtf1\ansi\deff0 
\trowd\trgaph144
\cellx1440\cellx2880\cellx4320
Cell 1\intbl\cell
Cell 2\intbl\cell
Cell 3\intbl\cell
\row
}
```

### 1.4 Character Encoding and Special Characters

RTF uses several encoding mechanisms:

1. **7-bit ASCII Base**: Standard RTF files contain only 7-bit ASCII characters
2. **Code Page Escapes**: `\'XX` where XX is a two-digit hex value
3. **Unicode Escapes**: `\u[N]?` where N is a decimal Unicode value, followed by a fallback character
4. **ANSI Code Pages**: Specified by `\ansicpg[N]` (e.g., `\ansicpg1252` for Windows-1252)

### 1.5 Font and Style Definitions

Font table structure:
```rtf
{\fonttbl
{\f0\froman\fcharset0 Times New Roman;}
{\f1\fswiss\fcharset0 Arial;}
{\f2\fmodern\fcharset0 Courier New;}
}
```

Style attributes:
- `\froman` - Roman (serif) font family
- `\fswiss` - Swiss (sans-serif) font family
- `\fmodern` - Fixed-pitch font family
- `\fcharset0` - ANSI character set

### 1.6 Headers, Footers, and Document Properties

```rtf
{\header \pard\qc\f0\fs20 Header Text\par}
{\footer \pard\qc\f0\fs20 Page {\field{\*\fldinst PAGE}}\par}
```

### 1.7 Lists and Indentation

Simple bulleted list:
```rtf
\pard\fi-360\li720 \'95\tab First item\par
\pard\fi-360\li720 \'95\tab Second item\par
```

- `\fi[N]` - First line indent (negative for hanging indent)
- `\li[N]` - Left indent
- `\'95` - Bullet character (•)

## 2. Parsing RTF Efficiently

### 2.1 Token-Based Parsing Approach

The recommended approach uses a two-stage parser:

1. **Lexical Analysis (Tokenization)**
   - Scan input character by character
   - Identify control words, parameters, groups, and text
   - Produce a token stream

2. **Syntactic Analysis (Parsing)**
   - Process token stream
   - Build document structure
   - Apply formatting rules

### 2.2 State Machine Design

RTF parsing requires a stack-based state machine due to nested groups:

```rust
#[derive(Clone)]
struct ParserState {
    font_index: usize,
    font_size: f32,
    bold: bool,
    italic: bool,
    underline: bool,
    alignment: Alignment,
    in_table: bool,
    // ... other properties
}

struct RtfParser {
    state_stack: Vec<ParserState>,
    current_state: ParserState,
    tokens: Vec<Token>,
    position: usize,
}
```

Key state transitions:
- `{` - Push current state onto stack
- `}` - Pop state from stack
- Control words - Modify current state
- Text - Apply current state formatting

### 2.3 Handling Nested Groups

Algorithm for group processing:

```rust
fn process_group(&mut self) -> Result<(), ParseError> {
    self.state_stack.push(self.current_state.clone());
    
    while let Some(token) = self.next_token() {
        match token {
            Token::GroupStart => self.process_group()?,
            Token::GroupEnd => {
                self.current_state = self.state_stack.pop()
                    .ok_or(ParseError::UnmatchedBrace)?;
                return Ok(());
            }
            Token::ControlWord(word, param) => {
                self.apply_control_word(&word, param)?;
            }
            Token::Text(text) => {
                self.emit_text(&text);
            }
        }
    }
    
    Err(ParseError::UnexpectedEof)
}
```

### 2.4 Error Recovery Strategies

1. **Ignore Unknown Control Words**: Essential for forward compatibility
2. **Brace Matching Recovery**: Track brace depth, attempt to resync
3. **Encoding Fallbacks**: Use replacement character for invalid encodings
4. **Table Structure Recovery**: Infer missing `\intbl` or `\cell` markers

## 3. RTF to Markdown Conversion Patterns

### 3.1 Mapping RTF Styles to Markdown

| RTF Style | Markdown Equivalent |
|-----------|-------------------|
| `\b` text `\b0` | **text** |
| `\i` text `\i0` | *text* |
| `\ul` text `\ul0` | No direct equivalent (use HTML `<u>`) |
| `\fs48` + bold | # Heading 1 |
| `\fs36` + bold | ## Heading 2 |
| `\par` | Double newline |
| `\line` | Single newline (two spaces + newline) |

### 3.2 Handling Tables

RTF to Markdown table conversion algorithm:

```rust
struct TableBuilder {
    rows: Vec<Vec<String>>,
    current_row: Vec<String>,
    current_cell: String,
}

impl TableBuilder {
    fn add_text(&mut self, text: &str) {
        self.current_cell.push_str(text);
    }
    
    fn end_cell(&mut self) {
        self.current_row.push(self.current_cell.clone());
        self.current_cell.clear();
    }
    
    fn end_row(&mut self) {
        self.rows.push(self.current_row.clone());
        self.current_row.clear();
    }
    
    fn to_markdown(&self) -> String {
        if self.rows.is_empty() {
            return String::new();
        }
        
        let mut result = String::new();
        
        // First row (header)
        result.push_str("| ");
        result.push_str(&self.rows[0].join(" | "));
        result.push_str(" |\n");
        
        // Separator
        result.push_str("|");
        for _ in &self.rows[0] {
            result.push_str(" --- |");
        }
        result.push('\n');
        
        // Data rows
        for row in &self.rows[1..] {
            result.push_str("| ");
            result.push_str(&row.join(" | "));
            result.push_str(" |\n");
        }
        
        result
    }
}
```

### 3.3 Converting Lists

RTF lists to Markdown:

```rust
fn convert_list_item(indent: i32, marker: &str, text: &str) -> String {
    let spaces = " ".repeat((indent / 720) as usize * 2);
    
    match marker {
        "\'95" | "\'b7" => format!("{}* {}", spaces, text),
        "1." | "2." | "3." => format!("{}1. {}", spaces, text),
        _ => format!("{}* {}", spaces, text),
    }
}
```

### 3.4 Preserving Supported Formatting

Markdown supports limited formatting. Strategy for conversion:

1. **Preserve**: Bold, italic, links, code blocks
2. **Convert**: Headers based on font size + bold
3. **Approximate**: Tables, lists, blockquotes
4. **Discard**: Colors, exact fonts, precise spacing

### 3.5 Handling Unsupported Features

For features without Markdown equivalents:

1. **HTML Fallback**: Use inline HTML for underline, colors
2. **Comments**: Add HTML comments with lost formatting info
3. **Footnotes**: Convert to Markdown footnotes `[^1]`
4. **Images**: Extract and reference as Markdown images

## 4. Implementation Recommendations

### 4.1 Key RTF Control Codes to Support

Priority control codes for legacy business documents:

**Essential (Phase 1)**
- Document structure: `\rtf`, `\ansi`, `\deff`
- Basic formatting: `\b`, `\i`, `\ul`, `\fs`, `\f`
- Paragraphs: `\par`, `\line`, `\q[lcrj]`
- Tables: `\trowd`, `\cellx`, `\cell`, `\row`, `\intbl`
- Special chars: `\'XX`, escapes

**Important (Phase 2)**
- Lists: `\pn`, `\pnlvl`, bullet markers
- Indentation: `\fi`, `\li`, `\ri`
- Colors: `\cf`, `\cb`, `\colortbl`
- Page layout: `\paperw`, `\paperh`, `\margin`

**Nice-to-have (Phase 3)**
- Headers/footers: `\header`, `\footer`
- Fields: `\field`, `\fldinst`
- Images: `\pict`
- Advanced tables: borders, shading

### 4.2 Parsing Algorithm Recommendation

Use a hybrid approach:

1. **Fast Lexer**: Character-by-character scanning with minimal allocation
2. **Streaming Parser**: Process tokens as they arrive (don't buffer entire document)
3. **Lazy State Management**: Only track state changes that affect output
4. **Table Buffering**: Buffer table rows for proper Markdown conversion

### 4.3 Data Structures for RTF Representation

```rust
// Core document structure
pub struct RtfDocument {
    pub metadata: DocumentMetadata,
    pub font_table: Vec<FontDefinition>,
    pub color_table: Vec<Color>,
    pub content: Vec<Block>,
}

// Block-level content
pub enum Block {
    Paragraph(Paragraph),
    Table(Table),
    List(List),
}

// Inline content
pub struct Paragraph {
    pub alignment: Alignment,
    pub indent: Indent,
    pub spans: Vec<Span>,
}

pub struct Span {
    pub text: String,
    pub formatting: TextFormat,
}

pub struct TextFormat {
    pub font_index: Option<usize>,
    pub font_size: Option<f32>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color_index: Option<usize>,
}

// Table structure
pub struct Table {
    pub rows: Vec<TableRow>,
}

pub struct TableRow {
    pub cells: Vec<TableCell>,
    pub height: Option<i32>,
}

pub struct TableCell {
    pub width: i32,
    pub content: Vec<Block>,
    pub borders: CellBorders,
}
```

### 4.4 Conversion Strategy: RTF → Markdown

Recommended conversion pipeline:

1. **Parse RTF** → Document structure
2. **Normalize** → Simplify nested formatting
3. **Analyze** → Detect headers, lists, tables
4. **Transform** → Convert to Markdown AST
5. **Render** → Generate Markdown text

### 4.5 Code Examples in Rust

#### Lexer Example

```rust
use std::str::Chars;
use std::iter::Peekable;

pub enum Token {
    GroupStart,
    GroupEnd,
    ControlWord(String, Option<i32>),
    Text(String),
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
        }
    }
    
    pub fn next_token(&mut self) -> Option<Token> {
        match self.input.next()? {
            '{' => Some(Token::GroupStart),
            '}' => Some(Token::GroupEnd),
            '\\' => self.read_control_sequence(),
            ch => self.read_text(ch),
        }
    }
    
    fn read_control_sequence(&mut self) -> Option<Token> {
        match self.input.peek()? {
            &'\\' | &'{' | &'}' => {
                // Escaped special character
                let ch = self.input.next()?;
                Some(Token::Text(ch.to_string()))
            }
            &'\'' => {
                // Hex character escape
                self.input.next(); // consume '
                let hex = self.read_hex(2)?;
                Some(Token::Text(self.decode_hex_char(hex)))
            }
            _ => {
                // Control word
                let word = self.read_control_word();
                let param = self.read_parameter();
                Some(Token::ControlWord(word, param))
            }
        }
    }
    
    fn read_text(&mut self, first: char) -> Option<Token> {
        let mut text = String::new();
        text.push(first);
        
        while let Some(&ch) = self.input.peek() {
            if ch == '{' || ch == '}' || ch == '\\' {
                break;
            }
            text.push(ch);
            self.input.next();
        }
        
        Some(Token::Text(text))
    }
    
    fn read_control_word(&mut self) -> String {
        let mut word = String::new();
        
        while let Some(&ch) = self.input.peek() {
            if ch.is_alphabetic() {
                word.push(ch);
                self.input.next();
            } else {
                break;
            }
        }
        
        word
    }
    
    fn read_parameter(&mut self) -> Option<i32> {
        let mut negative = false;
        let mut digits = String::new();
        
        if self.input.peek() == Some(&'-') {
            negative = true;
            self.input.next();
        }
        
        while let Some(&ch) = self.input.peek() {
            if ch.is_numeric() {
                digits.push(ch);
                self.input.next();
            } else {
                break;
            }
        }
        
        // Consume trailing space if present
        if self.input.peek() == Some(&' ') {
            self.input.next();
        }
        
        if digits.is_empty() {
            None
        } else {
            let value = digits.parse::<i32>().ok()?;
            Some(if negative { -value } else { value })
        }
    }
    
    fn read_hex(&mut self, count: usize) -> Option<String> {
        let mut hex = String::new();
        
        for _ in 0..count {
            if let Some(ch) = self.input.next() {
                if ch.is_ascii_hexdigit() {
                    hex.push(ch);
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        
        Some(hex)
    }
    
    fn decode_hex_char(&self, hex: String) -> String {
        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
            // For now, assume Windows-1252 encoding
            // In production, use the document's code page
            self.decode_windows_1252(byte)
        } else {
            String::from("?")
        }
    }
    
    fn decode_windows_1252(&self, byte: u8) -> String {
        // Simplified - in production, use proper encoding tables
        match byte {
            0x80..=0x9F => {
                // Windows-1252 specific mappings
                match byte {
                    0x80 => "€",
                    0x82 => "‚",
                    0x83 => "ƒ",
                    0x84 => "„",
                    0x85 => "…",
                    0x86 => "†",
                    0x87 => "‡",
                    0x88 => "ˆ",
                    0x89 => "‰",
                    0x8A => "Š",
                    0x8B => "‹",
                    0x8C => "Œ",
                    0x8E => "Ž",
                    0x91 => "'",
                    0x92 => "'",
                    0x93 => """,
                    0x94 => """,
                    0x95 => "•",
                    0x96 => "–",
                    0x97 => "—",
                    0x98 => "˜",
                    0x99 => "™",
                    0x9A => "š",
                    0x9B => "›",
                    0x9C => "œ",
                    0x9E => "ž",
                    0x9F => "Ÿ",
                    _ => "?",
                }.to_string()
            }
            _ => {
                // Standard ASCII or Latin-1
                (byte as char).to_string()
            }
        }
    }
}
```

#### Parser Example

```rust
use crate::lexer::{Token, Lexer};

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    state_stack: Vec<ParserState>,
    current_state: ParserState,
    document: RtfDocument,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        
        while let Some(token) = lexer.next_token() {
            tokens.push(token);
        }
        
        Parser {
            tokens,
            position: 0,
            state_stack: Vec::new(),
            current_state: ParserState::default(),
            document: RtfDocument::default(),
        }
    }
    
    pub fn parse(mut self) -> Result<RtfDocument, ParseError> {
        // Expect document to start with {\rtf
        self.expect_token(Token::GroupStart)?;
        self.expect_control_word("rtf", Some(1))?;
        
        // Parse document content
        self.parse_document_content()?;
        
        // Expect closing brace
        self.expect_token(Token::GroupEnd)?;
        
        Ok(self.document)
    }
    
    fn parse_document_content(&mut self) -> Result<(), ParseError> {
        while let Some(token) = self.peek_token() {
            match token {
                Token::GroupStart => {
                    self.next_token();
                    self.parse_group()?;
                }
                Token::GroupEnd => {
                    // End of current group
                    break;
                }
                Token::ControlWord(word, param) => {
                    self.next_token();
                    self.handle_control_word(word, param)?;
                }
                Token::Text(text) => {
                    self.next_token();
                    self.handle_text(text)?;
                }
            }
        }
        Ok(())
    }
    
    fn parse_group(&mut self) -> Result<(), ParseError> {
        // Save current state
        self.state_stack.push(self.current_state.clone());
        
        // Check for special destination groups
        if let Some(Token::ControlWord(word, _)) = self.peek_token() {
            match word.as_str() {
                "fonttbl" => {
                    self.next_token();
                    self.parse_font_table()?;
                }
                "colortbl" => {
                    self.next_token();
                    self.parse_color_table()?;
                }
                "*" => {
                    // Ignorable destination
                    self.skip_group()?;
                }
                _ => {
                    // Regular group
                    self.parse_document_content()?;
                }
            }
        } else {
            // Regular group
            self.parse_document_content()?;
        }
        
        // Restore state
        self.current_state = self.state_stack.pop()
            .ok_or(ParseError::UnmatchedBrace)?;
        
        // Expect closing brace
        self.expect_token(Token::GroupEnd)?;
        
        Ok(())
    }
    
    fn handle_control_word(&mut self, word: &str, param: &Option<i32>) 
        -> Result<(), ParseError> {
        match word {
            // Character formatting
            "b" => self.current_state.bold = param.unwrap_or(1) != 0,
            "i" => self.current_state.italic = param.unwrap_or(1) != 0,
            "ul" => self.current_state.underline = param.unwrap_or(1) != 0,
            "fs" => {
                if let Some(size) = param {
                    self.current_state.font_size = *size as f32 / 2.0;
                }
            }
            "f" => {
                if let Some(index) = param {
                    self.current_state.font_index = *index as usize;
                }
            }
            
            // Paragraph formatting
            "par" => self.handle_paragraph()?,
            "line" => self.handle_line_break()?,
            "ql" => self.current_state.alignment = Alignment::Left,
            "qc" => self.current_state.alignment = Alignment::Center,
            "qr" => self.current_state.alignment = Alignment::Right,
            "qj" => self.current_state.alignment = Alignment::Justify,
            
            // Table commands
            "trowd" => self.start_table_row()?,
            "cellx" => {
                if let Some(pos) = param {
                    self.add_cell_boundary(*pos)?;
                }
            }
            "intbl" => self.current_state.in_table = true,
            "cell" => self.end_table_cell()?,
            "row" => self.end_table_row()?,
            
            // Document properties
            "ansi" => self.document.metadata.charset = Charset::Ansi,
            "ansicpg" => {
                if let Some(cp) = param {
                    self.document.metadata.code_page = *cp;
                }
            }
            "deff" => {
                if let Some(font) = param {
                    self.document.metadata.default_font = *font as usize;
                }
            }
            
            // Unknown control word - ignore per spec
            _ => {}
        }
        
        Ok(())
    }
    
    fn handle_text(&mut self, text: &str) -> Result<(), ParseError> {
        if self.current_state.in_table {
            self.append_to_table_cell(text)?;
        } else {
            self.append_to_paragraph(text)?;
        }
        Ok(())
    }
}
```

#### Markdown Converter Example

```rust
pub struct MarkdownConverter {
    document: RtfDocument,
    output: String,
}

impl MarkdownConverter {
    pub fn new(document: RtfDocument) -> Self {
        MarkdownConverter {
            document,
            output: String::new(),
        }
    }
    
    pub fn convert(mut self) -> String {
        for block in &self.document.content {
            match block {
                Block::Paragraph(para) => self.convert_paragraph(para),
                Block::Table(table) => self.convert_table(table),
                Block::List(list) => self.convert_list(list),
            }
        }
        
        self.output
    }
    
    fn convert_paragraph(&mut self, para: &Paragraph) {
        let mut para_text = String::new();
        
        for span in &para.spans {
            let formatted = self.format_span(span);
            para_text.push_str(&formatted);
        }
        
        // Check if this might be a header
        if self.is_header(para) {
            let level = self.header_level(para);
            self.output.push_str(&"#".repeat(level));
            self.output.push(' ');
            self.output.push_str(&para_text);
        } else {
            // Apply alignment if not default
            match para.alignment {
                Alignment::Center => {
                    self.output.push_str("<div align=\"center\">\n");
                    self.output.push_str(&para_text);
                    self.output.push_str("\n</div>");
                }
                Alignment::Right => {
                    self.output.push_str("<div align=\"right\">\n");
                    self.output.push_str(&para_text);
                    self.output.push_str("\n</div>");
                }
                _ => {
                    self.output.push_str(&para_text);
                }
            }
        }
        
        self.output.push_str("\n\n");
    }
    
    fn format_span(&self, span: &Span) -> String {
        let mut result = span.text.clone();
        
        // Apply formatting
        if span.formatting.bold && span.formatting.italic {
            result = format!("***{}***", result);
        } else if span.formatting.bold {
            result = format!("**{}**", result);
        } else if span.formatting.italic {
            result = format!("*{}*", result);
        }
        
        if span.formatting.underline {
            result = format!("<u>{}</u>", result);
        }
        
        result
    }
    
    fn convert_table(&mut self, table: &Table) {
        if table.rows.is_empty() {
            return;
        }
        
        // Assume first row is header
        let header = &table.rows[0];
        
        // Header row
        self.output.push_str("| ");
        for cell in &header.cells {
            let content = self.cell_content(cell);
            self.output.push_str(&content);
            self.output.push_str(" | ");
        }
        self.output.push_str("\n");
        
        // Separator
        self.output.push_str("|");
        for _ in &header.cells {
            self.output.push_str(" --- |");
        }
        self.output.push_str("\n");
        
        // Data rows
        for row in &table.rows[1..] {
            self.output.push_str("| ");
            for cell in &row.cells {
                let content = self.cell_content(cell);
                self.output.push_str(&content);
                self.output.push_str(" | ");
            }
            self.output.push_str("\n");
        }
        
        self.output.push('\n');
    }
    
    fn cell_content(&self, cell: &TableCell) -> String {
        let mut content = String::new();
        
        for block in &cell.content {
            if let Block::Paragraph(para) = block {
                for span in &para.spans {
                    content.push_str(&self.format_span(span));
                }
            }
        }
        
        // Remove trailing newlines and clean up
        content.trim().to_string()
    }
    
    fn is_header(&self, para: &Paragraph) -> bool {
        // Simple heuristic: large font + bold = header
        para.spans.iter().any(|span| {
            span.formatting.bold && 
            span.formatting.font_size.unwrap_or(12.0) > 14.0
        })
    }
    
    fn header_level(&self, para: &Paragraph) -> usize {
        // Map font size to header level
        let avg_size = para.spans.iter()
            .filter_map(|s| s.formatting.font_size)
            .sum::<f32>() / para.spans.len() as f32;
        
        match avg_size {
            s if s >= 24.0 => 1,
            s if s >= 18.0 => 2,
            s if s >= 16.0 => 3,
            _ => 4,
        }
    }
}
```

## 5. Testing and Validation Strategy

### 5.1 Test Documents

Create test RTF documents covering:

1. **Basic Formatting**: Bold, italic, underline combinations
2. **Tables**: Simple and complex table structures
3. **Lists**: Bulleted and numbered lists with nesting
4. **Special Characters**: Unicode, symbols, escapes
5. **Legacy Patterns**: Common VB6/VFP9 report formats

### 5.2 Edge Cases

- Malformed RTF (missing braces, incomplete commands)
- Deeply nested groups
- Large documents (streaming performance)
- Mixed encodings
- Invalid control sequences

### 5.3 Validation Approach

1. **Round-trip Testing**: RTF → Parse → Serialize → RTF
2. **Visual Comparison**: Render both RTF and Markdown
3. **Compatibility Testing**: Test with actual VB6/VFP9 generated files
4. **Performance Benchmarks**: Parse speed and memory usage

## 6. Performance Considerations

### 6.1 Optimization Strategies

1. **Streaming Parser**: Don't load entire document into memory
2. **String Building**: Use `String::with_capacity()` for known sizes
3. **Lazy Evaluation**: Only process formatting that affects output
4. **Table Buffering**: Minimal buffering for table structure

### 6.2 Memory Management

```rust
// Use Cow for efficient string handling
use std::borrow::Cow;

pub struct TextSpan<'a> {
    text: Cow<'a, str>,
    format: TextFormat,
}

// Pool allocators for frequently created objects
use typed_arena::Arena;

pub struct ParserArena<'a> {
    spans: Arena<Span>,
    paragraphs: Arena<Paragraph>,
}
```

## 7. Integration with LegacyBridge

### 7.1 API Design

```rust
// High-level API
pub fn rtf_to_markdown(input: &str) -> Result<String, ConversionError> {
    let document = parse_rtf(input)?;
    let markdown = convert_to_markdown(document);
    Ok(markdown)
}

// Streaming API for large documents
pub struct RtfStreamConverter<R: Read, W: Write> {
    reader: R,
    writer: W,
    buffer_size: usize,
}

impl<R: Read, W: Write> RtfStreamConverter<R, W> {
    pub fn convert(&mut self) -> Result<(), ConversionError> {
        // Implementation
    }
}
```

### 7.2 Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Invalid RTF structure: {0}")]
    InvalidStructure(String),
    
    #[error("Unsupported encoding: {0}")]
    UnsupportedEncoding(i32),
    
    #[error("Table structure error: {0}")]
    TableError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

## 8. Conclusion

This research provides a comprehensive foundation for implementing an RTF parser in Rust optimized for legacy business documents. The key recommendations are:

1. **Use a two-stage parser** with separate lexer and parser phases
2. **Implement a stack-based state machine** for handling nested groups
3. **Focus on essential control codes** used in business documents
4. **Provide graceful degradation** for unsupported features
5. **Optimize for streaming** to handle large documents efficiently

The provided code examples demonstrate practical implementation patterns that can be extended and refined based on specific requirements of the LegacyBridge project.

## References

1. RTF Specification 1.9.1 (Microsoft)
2. "rtf-parser" Rust crate documentation
3. VB6 RichTextBox control documentation
4. VFP9 RTF control usage patterns
5. CommonMark specification for Markdown output