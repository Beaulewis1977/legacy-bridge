// SIMD-Optimized Markdown Parser
// High-performance Markdown parsing with SIMD string operations

use super::types::{ConversionResult, DocumentMetadata, RtfDocument, RtfNode};
use super::markdown_simd_utils::{SimdMarkdownScanner, SimdUtf8Validator, SimdWhitespaceOps};
use super::string_interner::OptimizedStringInterner;
use pulldown_cmark::{Parser, Event, Tag, CowStr, Options};
use smallvec::SmallVec;
use std::arch::x86_64::*;

/// SIMD-optimized Markdown parser
pub struct SimdMarkdownParser {
    /// String interner for deduplication
    string_cache: OptimizedStringInterner,
    /// Parser options
    options: Options,
    /// SIMD utilities
    scanner: SimdMarkdownScanner,
    whitespace_ops: SimdWhitespaceOps,
    utf8_validator: SimdUtf8Validator,
}

impl SimdMarkdownParser {
    pub fn new() -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        
        Self {
            string_cache: OptimizedStringInterner::new(),
            options,
            scanner: SimdMarkdownScanner::new(),
            whitespace_ops: SimdWhitespaceOps::new(),
            utf8_validator: SimdUtf8Validator::new(),
        }
    }
    
    /// Parse Markdown with SIMD preprocessing
    pub fn parse(&mut self, markdown_content: &str) -> ConversionResult<RtfDocument> {
        // SIMD UTF-8 validation
        if !self.utf8_validator.is_valid_utf8(markdown_content.as_bytes()) {
            return Err(super::types::ConversionError::ValidationError(
                "Invalid UTF-8 in markdown content".to_string()
            ));
        }
        
        // SIMD preprocessing: find special characters for optimization hints
        let special_chars = self.scanner.find_special_chars(markdown_content.as_bytes());
        let line_count = self.scanner.count_lines(markdown_content.as_bytes());
        
        // Estimate capacity based on document characteristics
        let estimated_nodes = special_chars.len() + line_count * 2;
        
        // Parse with pulldown-cmark
        let parser = Parser::new_ext(markdown_content, self.options);
        
        let mut converter = SimdConverter::new(
            estimated_nodes,
            &mut self.string_cache,
            &self.whitespace_ops,
            special_chars,
        );
        
        // Process events
        for event in parser {
            converter.process_event(event)?;
        }
        
        Ok(converter.finalize())
    }
}

/// SIMD-enhanced converter
struct SimdConverter<'a> {
    document: RtfDocument,
    current_paragraph: SmallVec<[RtfNode; 8]>,
    formatting_stack: SmallVec<[FormattingState; 4]>,
    list_stack: SmallVec<[ListState; 4]>,
    table_state: Option<Box<TableState>>,
    current_heading_level: Option<u8>,
    string_cache: &'a mut OptimizedStringInterner,
    whitespace_ops: &'a SimdWhitespaceOps,
    text_buffer: String,
    special_char_positions: Vec<usize>,
    next_special_idx: usize,
}

#[derive(Debug, Clone, Copy)]
enum FormattingState {
    Bold,
    Italic,
    Underline,
    Code,
}

#[derive(Debug, Clone, Copy)]
struct ListState {
    level: u8,
    ordered: bool,
}

#[derive(Debug)]
struct TableState {
    current_row: Vec<RtfNode>,
    rows: Vec<super::types::TableRow>,
    in_header: bool,
    cell_buffer: Vec<RtfNode>,
}

impl<'a> SimdConverter<'a> {
    fn new(
        estimated_capacity: usize,
        string_cache: &'a mut OptimizedStringInterner,
        whitespace_ops: &'a SimdWhitespaceOps,
        special_char_positions: Vec<usize>,
    ) -> Self {
        Self {
            document: RtfDocument {
                metadata: DocumentMetadata::default(),
                content: Vec::with_capacity(estimated_capacity),
            },
            current_paragraph: SmallVec::new(),
            formatting_stack: SmallVec::new(),
            list_stack: SmallVec::new(),
            table_state: None,
            current_heading_level: None,
            string_cache,
            whitespace_ops,
            text_buffer: String::with_capacity(256),
            special_char_positions,
            next_special_idx: 0,
        }
    }
    
    #[inline]
    fn process_event(&mut self, event: Event) -> ConversionResult<()> {
        match event {
            Event::Start(tag) => self.handle_start_tag(tag)?,
            Event::End(tag) => self.handle_end_tag(tag)?,
            Event::Text(text) => self.handle_text_simd(text)?,
            Event::Code(code) => self.handle_code(code)?,
            Event::Html(_) => {}, // Skip HTML
            Event::FootnoteReference(_) => {}, // Skip footnotes
            Event::SoftBreak => self.handle_soft_break()?,
            Event::HardBreak => self.handle_hard_break()?,
            Event::Rule => self.handle_rule()?,
            Event::TaskListMarker(_) => {}, // Skip task list markers
        }
        Ok(())
    }
    
    #[inline]
    fn handle_start_tag(&mut self, tag: Tag) -> ConversionResult<()> {
        match tag {
            Tag::Paragraph => {
                self.flush_text_buffer();
            }
            Tag::Heading { level, .. } => {
                self.current_heading_level = Some(level as u8);
            }
            Tag::List(first_item_number) => {
                let ordered = first_item_number.is_some();
                let level = self.list_stack.len() as u8;
                self.list_stack.push(ListState { level, ordered });
            }
            Tag::Table(_) => {
                self.table_state = Some(Box::new(TableState {
                    current_row: Vec::with_capacity(4),
                    rows: Vec::with_capacity(10),
                    in_header: true,
                    cell_buffer: Vec::with_capacity(4),
                }));
            }
            Tag::TableHead => {
                if let Some(ref mut table) = self.table_state {
                    table.in_header = true;
                }
            }
            Tag::TableRow => {
                if let Some(ref mut table) = self.table_state {
                    table.current_row.clear();
                }
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
    
    #[inline]
    fn handle_end_tag(&mut self, tag: Tag) -> ConversionResult<()> {
        match tag {
            Tag::Paragraph => {
                self.flush_text_buffer();
                if !self.current_paragraph.is_empty() {
                    let paragraph_content: Vec<RtfNode> = self.current_paragraph.drain(..).collect();
                    
                    if let Some(level) = self.current_heading_level.take() {
                        self.document.content.push(RtfNode::Heading {
                            level,
                            content: paragraph_content,
                        });
                    } else if !self.list_stack.is_empty() {
                        let list_state = self.list_stack.last().unwrap();
                        self.document.content.push(RtfNode::ListItem {
                            level: list_state.level,
                            content: paragraph_content,
                        });
                    } else {
                        self.document.content.push(RtfNode::Paragraph(paragraph_content));
                    }
                }
            }
            Tag::List(_) => {
                self.list_stack.pop();
            }
            Tag::Table(_) => {
                if let Some(mut table_state) = self.table_state.take() {
                    if !table_state.rows.is_empty() {
                        self.document.content.push(RtfNode::Table {
                            rows: table_state.rows,
                        });
                    }
                }
            }
            Tag::TableRow => {
                if let Some(ref mut table) = self.table_state {
                    if !table.current_row.is_empty() {
                        let cells: Vec<super::types::TableCell> = table.current_row
                            .drain(..)
                            .map(|content| super::types::TableCell {
                                content: vec![content],
                            })
                            .collect();
                        
                        table.rows.push(super::types::TableRow { cells });
                    }
                }
            }
            Tag::TableCell => {
                self.flush_text_buffer();
                if let Some(ref mut table) = self.table_state {
                    if !self.current_paragraph.is_empty() {
                        table.cell_buffer.clear();
                        table.cell_buffer.extend(self.current_paragraph.drain(..));
                        
                        if table.cell_buffer.len() == 1 {
                            table.current_row.push(table.cell_buffer[0].clone());
                        } else {
                            table.current_row.push(RtfNode::Paragraph(table.cell_buffer.clone()));
                        }
                    }
                }
            }
            Tag::Emphasis => {
                self.formatting_stack.retain(|&f| !matches!(f, FormattingState::Italic));
            }
            Tag::Strong => {
                self.formatting_stack.retain(|&f| !matches!(f, FormattingState::Bold));
            }
            _ => {}
        }
        Ok(())
    }
    
    /// SIMD-optimized text handling
    #[inline]
    fn handle_text_simd(&mut self, text: CowStr) -> ConversionResult<()> {
        // Use SIMD to normalize whitespace if needed
        let normalized = if text.contains('\n') || text.contains('\t') || text.contains("  ") {
            self.whitespace_ops.normalize_whitespace(&text)
        } else {
            text.into_owned()
        };
        
        self.text_buffer.push_str(&normalized);
        Ok(())
    }
    
    #[inline]
    fn handle_code(&mut self, code: CowStr) -> ConversionResult<()> {
        self.flush_text_buffer();
        let text = self.string_cache.intern_to_string(&code);
        self.current_paragraph.push(RtfNode::Text(text));
        Ok(())
    }
    
    #[inline]
    fn handle_soft_break(&mut self) -> ConversionResult<()> {
        self.text_buffer.push(' ');
        Ok(())
    }
    
    #[inline]
    fn handle_hard_break(&mut self) -> ConversionResult<()> {
        self.flush_text_buffer();
        self.current_paragraph.push(RtfNode::LineBreak);
        Ok(())
    }
    
    #[inline]
    fn handle_rule(&mut self) -> ConversionResult<()> {
        self.flush_text_buffer();
        self.document.content.push(RtfNode::PageBreak);
        Ok(())
    }
    
    /// Flush accumulated text buffer with SIMD-optimized interning
    fn flush_text_buffer(&mut self) {
        if self.text_buffer.is_empty() {
            return;
        }
        
        let text = self.string_cache.intern_to_string(&self.text_buffer);
        self.text_buffer.clear();
        
        let mut current_node = RtfNode::Text(text);
        
        if !self.formatting_stack.is_empty() {
            for &formatting in self.formatting_stack.iter().rev() {
                current_node = match formatting {
                    FormattingState::Bold => RtfNode::Bold(vec![current_node]),
                    FormattingState::Italic => RtfNode::Italic(vec![current_node]),
                    FormattingState::Underline => RtfNode::Underline(vec![current_node]),
                    FormattingState::Code => current_node,
                };
            }
        }
        
        self.current_paragraph.push(current_node);
    }
    
    fn finalize(mut self) -> RtfDocument {
        self.flush_text_buffer();
        
        if !self.current_paragraph.is_empty() {
            let paragraph_content: Vec<RtfNode> = self.current_paragraph.into_vec();
            
            if let Some(level) = self.current_heading_level.take() {
                self.document.content.push(RtfNode::Heading {
                    level,
                    content: paragraph_content,
                });
            } else if !self.list_stack.is_empty() {
                let list_state = self.list_stack.last().unwrap();
                self.document.content.push(RtfNode::ListItem {
                    level: list_state.level,
                    content: paragraph_content,
                });
            } else {
                self.document.content.push(RtfNode::Paragraph(paragraph_content));
            }
        }
        
        self.document.content.shrink_to_fit();
        self.document
    }
}

/// SIMD-accelerated text processing utilities
pub mod simd_text_utils {
    use std::arch::x86_64::*;
    
    /// Fast character counting using SIMD
    #[target_feature(enable = "avx2")]
    pub unsafe fn count_chars_avx2(text: &[u8], target: u8) -> usize {
        let mut count = 0;
        let mut pos = 0;
        let len = text.len();
        let target_vec = _mm256_set1_epi8(target as i8);
        
        while pos + 32 <= len {
            let chunk = _mm256_loadu_si256(text[pos..].as_ptr() as *const __m256i);
            let cmp = _mm256_cmpeq_epi8(chunk, target_vec);
            let mask = _mm256_movemask_epi8(cmp);
            count += mask.count_ones() as usize;
            pos += 32;
        }
        
        // Handle remainder
        count += text[pos..].iter().filter(|&&b| b == target).count();
        count
    }
    
    /// Fast newline normalization using SIMD
    #[target_feature(enable = "avx2")]
    pub unsafe fn normalize_newlines_avx2(text: &mut Vec<u8>) {
        let mut write_pos = 0;
        let mut read_pos = 0;
        let len = text.len();
        
        let cr_vec = _mm256_set1_epi8(b'\r' as i8);
        let lf_vec = _mm256_set1_epi8(b'\n' as i8);
        
        while read_pos + 32 <= len {
            let chunk = _mm256_loadu_si256(text[read_pos..].as_ptr() as *const __m256i);
            
            // Check for CR
            let cr_mask = _mm256_cmpeq_epi8(chunk, cr_vec);
            let cr_bits = _mm256_movemask_epi8(cr_mask);
            
            if cr_bits == 0 {
                // No CRs, copy as-is
                if write_pos != read_pos {
                    text.copy_within(read_pos..read_pos + 32, write_pos);
                }
                write_pos += 32;
                read_pos += 32;
            } else {
                // Process byte by byte when CRs found
                for i in 0..32 {
                    if read_pos + i >= len {
                        break;
                    }
                    
                    let byte = text[read_pos + i];
                    if byte == b'\r' {
                        // Skip CR, LF will be kept
                        continue;
                    }
                    
                    text[write_pos] = byte;
                    write_pos += 1;
                }
                read_pos += 32;
            }
        }
        
        // Handle remainder
        while read_pos < len {
            let byte = text[read_pos];
            if byte != b'\r' {
                text[write_pos] = byte;
                write_pos += 1;
            }
            read_pos += 1;
        }
        
        text.truncate(write_pos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simd_markdown_parse_simple() {
        let mut parser = SimdMarkdownParser::new();
        let document = parser.parse("# Hello World\n\nThis is a test.").unwrap();
        
        assert_eq!(document.content.len(), 2);
        
        // Check heading
        match &document.content[0] {
            RtfNode::Heading { level, content } => {
                assert_eq!(*level, 1);
                assert_eq!(content.len(), 1);
            }
            _ => panic!("Expected heading"),
        }
    }
    
    #[test]
    fn test_simd_markdown_special_chars() {
        let mut parser = SimdMarkdownParser::new();
        let document = parser.parse("**bold** and *italic* and `code`").unwrap();
        
        assert_eq!(document.content.len(), 1);
        match &document.content[0] {
            RtfNode::Paragraph(nodes) => {
                // Should have bold, text, italic, text, and code nodes
                assert!(nodes.len() >= 5);
            }
            _ => panic!("Expected paragraph"),
        }
    }
    
    #[test]
    fn test_simd_text_utils() {
        unsafe {
            // Test character counting
            let text = b"Hello World! Hello again!";
            let count = simd_text_utils::count_chars_avx2(text, b'l');
            assert_eq!(count, 5);
            
            // Test newline normalization
            let mut text = b"Hello\r\nWorld\rTest\n".to_vec();
            simd_text_utils::normalize_newlines_avx2(&mut text);
            assert_eq!(&text, b"Hello\nWorldTest\n");
        }
    }
}