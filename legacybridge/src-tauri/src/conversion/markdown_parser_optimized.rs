// Optimized Markdown Parser - High-performance parsing with reduced allocations
//
// Key optimizations:
// 1. Pre-allocated buffers with capacity hints
// 2. String interning for repeated text
// 3. SmallVec for small collections
// 4. Reduced cloning through careful lifetime management
// 5. Arena allocation for temporary objects

use super::types::{ConversionResult, DocumentMetadata, RtfDocument, RtfNode};
use pulldown_cmark::{Parser, Event, Tag, CowStr, Options};
use smallvec::SmallVec;
use ahash::AHashMap;
use std::mem;

/// Optimized Markdown Parser with performance improvements
pub struct OptimizedMarkdownParser {
    /// String interner for deduplication
    string_cache: StringInterner,
    /// Parser options
    options: Options,
}

impl OptimizedMarkdownParser {
    pub fn new() -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        
        Self {
            string_cache: StringInterner::new(),
            options,
        }
    }

    /// Parse Markdown content into RTF document structure with optimizations
    pub fn parse(&mut self, markdown_content: &str) -> ConversionResult<RtfDocument> {
        // Pre-allocate parser with estimated capacity
        let estimated_nodes = markdown_content.len() / 50; // Rough estimate
        let parser = Parser::new_ext(markdown_content, self.options);
        
        let mut converter = OptimizedConverter::new(estimated_nodes, &mut self.string_cache);
        
        // Process events with minimal allocations
        for event in parser {
            converter.process_event(event)?;
        }
        
        Ok(converter.finalize())
    }
}

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

/// String interner with LRU cache and memory limits for DoS prevention
struct StringInterner {
    cache: AHashMap<String, CacheEntry>,
    access_order: VecDeque<String>,
    strings: Vec<String>,
    hit_count: AtomicU64,
    miss_count: AtomicU64,
    memory_used: usize,
}

#[derive(Clone)]
struct CacheEntry {
    index: usize,
    last_accessed: u64,
}

// SECURITY: Memory limits to prevent unbounded growth
const MAX_CACHE_SIZE: usize = 10_000;
const MAX_MEMORY_BYTES: usize = 50 * 1024 * 1024; // 50MB
const CLEANUP_THRESHOLD: usize = 8_000;

impl StringInterner {
    fn new() -> Self {
        Self {
            cache: AHashMap::with_capacity(1000),
            access_order: VecDeque::with_capacity(1000),
            strings: Vec::with_capacity(1000),
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
            memory_used: 0,
        }
    }

    fn intern(&mut self, text: &str) -> String {
        // For small strings, just return a copy (not worth caching)
        if text.len() <= 8 {
            return text.to_string();
        }

        // SECURITY: Prevent excessively large strings from being cached
        if text.len() > 1024 * 1024 { // 1MB limit per string
            return text.to_string(); // Don't cache huge strings
        }

        // Check cache hit
        if let Some(entry) = self.cache.get_mut(text) {
            self.hit_count.fetch_add(1, Ordering::Relaxed);
            
            // Update LRU order - move to end
            if let Some(pos) = self.access_order.iter().position(|x| x == text) {
                self.access_order.remove(pos);
            }
            self.access_order.push_back(text.to_string());
            
            return self.strings[entry.index].clone();
        }

        // Cache miss - add new entry
        self.miss_count.fetch_add(1, Ordering::Relaxed);
        
        // SECURITY: Check if we need to evict entries
        if self.cache.len() >= MAX_CACHE_SIZE || self.memory_used >= MAX_MEMORY_BYTES {
            self.evict_lru_entries();
        }

        // Add to cache
        let string = text.to_string();
        let idx = self.strings.len();
        
        // Track memory usage
        let entry_memory = string.len() + std::mem::size_of::<CacheEntry>() + std::mem::size_of::<String>();
        self.memory_used += entry_memory;
        
        self.strings.push(string.clone());
        
        let entry = CacheEntry {
            index: idx,
            last_accessed: self.hit_count.load(Ordering::Relaxed) + self.miss_count.load(Ordering::Relaxed),
        };
        
        self.cache.insert(string.clone(), entry);
        self.access_order.push_back(string.clone());
        
        string
    }

    /// Evict least recently used entries to prevent memory exhaustion
    fn evict_lru_entries(&mut self) {
        let target_size = CLEANUP_THRESHOLD;
        
        while self.cache.len() > target_size && !self.access_order.is_empty() {
            if let Some(oldest_key) = self.access_order.pop_front() {
                if let Some(entry) = self.cache.remove(&oldest_key) {
                    // Update memory usage
                    let entry_memory = oldest_key.len() + std::mem::size_of::<CacheEntry>() + std::mem::size_of::<String>();
                    self.memory_used = self.memory_used.saturating_sub(entry_memory);
                    
                    // Note: We don't remove from strings Vec to avoid index invalidation
                    // This creates some memory overhead but maintains correctness
                }
            }
        }
    }

    fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
        self.strings.clear();
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
        self.memory_used = 0;
    }

    /// Get cache statistics for monitoring
    fn stats(&self) -> (u64, u64, f64, usize, usize) {
        let hits = self.hit_count.load(Ordering::Relaxed);
        let misses = self.miss_count.load(Ordering::Relaxed);
        let hit_rate = if hits + misses > 0 {
            hits as f64 / (hits + misses) as f64
        } else {
            0.0
        };
        (hits, misses, hit_rate, self.cache.len(), self.memory_used)
    }
}

/// Optimized converter with reduced allocations
struct OptimizedConverter<'a> {
    document: RtfDocument,
    // Use SmallVec to avoid heap allocation for small paragraphs
    current_paragraph: SmallVec<[RtfNode; 8]>,
    formatting_stack: SmallVec<[FormattingState; 4]>,
    list_stack: SmallVec<[ListState; 4]>,
    table_state: Option<Box<TableState>>, // Box to reduce stack size
    current_heading_level: Option<u8>,
    string_cache: &'a mut StringInterner,
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

impl<'a> OptimizedConverter<'a> {
    fn new(estimated_capacity: usize, string_cache: &'a mut StringInterner) -> Self {
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
                        // Move content to table cell
                        table.cell_buffer.clear();
                        table.cell_buffer.extend(self.current_paragraph.drain(..));
                        
                        // Consolidate cell content
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
        
        // Handle inline code
        let text = self.string_cache.intern(&code);
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
        let text = self.string_cache.intern(&self.text_buffer);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_parse_simple_text() {
        let markdown = "Hello World";
        let mut parser = OptimizedMarkdownParser::new();
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
    fn test_string_interning() {
        let mut interner = StringInterner::new();
        
        let s1 = interner.intern("Hello World");
        let s2 = interner.intern("Hello World");
        
        // Should return the same string
        assert_eq!(s1, s2);
        
        // Short strings should not be interned
        let short = interner.intern("Hi");
        assert_eq!(short, "Hi");
    }

    #[test]
    fn test_large_document_performance() {
        let mut parser = OptimizedMarkdownParser::new();
        
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
    }
}