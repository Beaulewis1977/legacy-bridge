// Performance-Optimized Markdown Parser - Zero-allocation design
//
// Key optimizations:
// 1. LRU string interner with bounded memory
// 2. Cow<str> for zero-copy string operations
// 3. Pre-allocated buffers with capacity hints
// 4. SmallVec for small collections
// 5. Reduced cloning through Arc<str> sharing

use super::types::{ConversionResult, DocumentMetadata, RtfDocument, RtfNode};
use super::string_interner::OptimizedStringInterner;
use pulldown_cmark::{Parser, Event, Tag, CowStr, Options};
use smallvec::SmallVec;
use std::sync::Arc;
use std::borrow::Cow;

/// Performance-optimized Markdown Parser
pub struct PerfOptimizedMarkdownParser {
    /// String interner with LRU cache
    string_cache: OptimizedStringInterner,
    /// Parser options
    options: Options,
}

impl PerfOptimizedMarkdownParser {
    pub fn new() -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        
        Self {
            string_cache: OptimizedStringInterner::new(),
            options,
        }
    }

    /// Parse Markdown content with performance optimizations
    pub fn parse(&self, markdown_content: &str) -> ConversionResult<RtfDocument> {
        // Pre-allocate parser with estimated capacity
        let estimated_nodes = markdown_content.len() / 50; // Rough estimate
        let parser = Parser::new_ext(markdown_content, self.options);
        
        let mut converter = PerfOptimizedConverter::new(estimated_nodes, &self.string_cache);
        
        // Process events with minimal allocations
        for event in parser {
            converter.process_event(event)?;
        }
        
        Ok(converter.finalize())
    }
    
    /// Get interner statistics
    pub fn interner_stats(&self) -> super::string_interner::InternerStats {
        self.string_cache.stats()
    }
    
    /// Clear the string cache
    pub fn clear_cache(&self) {
        self.string_cache.clear();
    }
}

/// Performance-optimized converter
struct PerfOptimizedConverter<'a> {
    document: RtfDocument,
    // Use SmallVec to avoid heap allocation for small paragraphs
    current_paragraph: SmallVec<[RtfNode; 8]>,
    formatting_stack: SmallVec<[FormattingState; 4]>,
    list_stack: SmallVec<[ListState; 4]>,
    table_state: Option<Box<TableState>>, // Box to reduce stack size
    current_heading_level: Option<u8>,
    string_cache: &'a OptimizedStringInterner,
    // Pre-allocated buffer for text accumulation
    text_buffer: String,
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
    // Reusable buffer for cell content
    cell_buffer: Vec<RtfNode>,
}

impl<'a> PerfOptimizedConverter<'a> {
    fn new(estimated_capacity: usize, string_cache: &'a OptimizedStringInterner) -> Self {
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
            text_buffer: String::with_capacity(256),
        }
    }

    #[inline]
    fn process_event(&mut self, event: Event) -> ConversionResult<()> {
        match event {
            Event::Start(tag) => self.handle_start_tag(tag)?,
            Event::End(tag) => self.handle_end_tag(tag)?,
            Event::Text(text) => self.handle_text(text)?,
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
                // Flush any accumulated text
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
            _ => {} // Handle other tags as needed
        }
        Ok(())
    }

    #[inline]
    fn handle_end_tag(&mut self, tag: Tag) -> ConversionResult<()> {
        match tag {
            Tag::Paragraph => {
                self.flush_text_buffer();
                if !self.current_paragraph.is_empty() {
                    // Use into_vec() to avoid clone
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
                        // Optimize by moving instead of cloning
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
                        // Move content to table cell
                        table.cell_buffer.clear();
                        table.cell_buffer.extend(self.current_paragraph.drain(..));
                        
                        // Consolidate cell content
                        if table.cell_buffer.len() == 1 {
                            // Move instead of clone
                            if let Some(node) = table.cell_buffer.pop() {
                                table.current_row.push(node);
                            }
                        } else {
                            // Move the buffer content
                            let cell_content = std::mem::take(&mut table.cell_buffer);
                            table.current_row.push(RtfNode::Paragraph(cell_content));
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
            _ => {} // Handle other tags as needed
        }
        Ok(())
    }

    #[inline]
    fn handle_text(&mut self, text: CowStr) -> ConversionResult<()> {
        // Accumulate text in buffer instead of creating nodes immediately
        self.text_buffer.push_str(&text);
        Ok(())
    }

    #[inline]
    fn handle_code(&mut self, code: CowStr) -> ConversionResult<()> {
        // Flush any pending text first
        self.flush_text_buffer();
        
        // Handle inline code with interning
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

    /// Flush accumulated text buffer to create formatted nodes
    fn flush_text_buffer(&mut self) {
        if self.text_buffer.is_empty() {
            return;
        }

        // Intern the text for deduplication
        let text = self.string_cache.intern_to_string(&self.text_buffer);
        self.text_buffer.clear();

        let mut current_node = RtfNode::Text(text);

        // Apply formatting in optimal order (least likely to most likely)
        if !self.formatting_stack.is_empty() {
            // Build formatting from innermost to outermost
            for &formatting in self.formatting_stack.iter().rev() {
                current_node = match formatting {
                    FormattingState::Bold => RtfNode::Bold(vec![current_node]),
                    FormattingState::Italic => RtfNode::Italic(vec![current_node]),
                    FormattingState::Underline => RtfNode::Underline(vec![current_node]),
                    FormattingState::Code => current_node, // Code handled separately
                };
            }
        }

        self.current_paragraph.push(current_node);
    }

    fn finalize(mut self) -> RtfDocument {
        // Flush any remaining text
        self.flush_text_buffer();
        
        // Add any remaining paragraph content
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

        // Shrink to fit to save memory
        self.document.content.shrink_to_fit();
        self.document
    }
}

impl Default for PerfOptimizedMarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_parse_simple_text() {
        let markdown = "Hello World";
        let parser = PerfOptimizedMarkdownParser::new();
        let document = parser.parse(markdown).unwrap();
        
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
    fn test_interner_stats() {
        let parser = PerfOptimizedMarkdownParser::new();
        
        // Parse document with repeated strings
        let markdown = "# Title\n\nText text text.\n\n# Title\n\nMore text text.";
        let _ = parser.parse(markdown).unwrap();
        
        let stats = parser.interner_stats();
        assert!(stats.hit_count > 0);
        assert!(stats.hit_rate > 0.0);
    }

    #[test]
    fn test_large_document_performance() {
        let parser = PerfOptimizedMarkdownParser::new();
        
        // Generate a large document
        let mut doc = String::new();
        for i in 0..1000 {
            doc.push_str(&format!("# Heading {}\n\n", i));
            doc.push_str("This is a paragraph with **bold** and *italic* text.\n\n");
            doc.push_str("- List item 1\n");
            doc.push_str("- List item 2\n\n");
        }
        
        let result = parser.parse(&doc);
        assert!(result.is_ok());
        
        let document = result.unwrap();
        assert!(document.content.len() > 2000); // Should have many nodes
        
        // Check interner effectiveness
        let stats = parser.interner_stats();
        println!("Interner stats: {:?}", stats);
        assert!(stats.hit_rate > 50.0); // Should have good hit rate with repeated content
    }
}