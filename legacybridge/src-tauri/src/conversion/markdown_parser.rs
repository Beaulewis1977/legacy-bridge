// Markdown Parser - Parses Markdown content into an intermediate document structure

use super::types::{ConversionResult, DocumentMetadata, RtfDocument, RtfNode};
use pulldown_cmark::{Parser, Event, Tag, CowStr};

/// Markdown Parser
pub struct MarkdownParser;

impl MarkdownParser {
    /// Parse Markdown content into RTF document structure
    pub fn parse(markdown_content: &str) -> ConversionResult<RtfDocument> {
        let parser = Parser::new(markdown_content);
        let mut converter = MarkdownToRtfConverter::new();
        
        for event in parser {
            converter.process_event(event)?;
        }
        
        Ok(converter.finalize())
    }
}

/// Converter that processes Markdown events and builds RTF document structure
struct MarkdownToRtfConverter {
    document: RtfDocument,
    current_paragraph: Vec<RtfNode>,
    formatting_stack: Vec<FormattingState>,
    list_stack: Vec<ListState>,
    table_state: Option<TableState>,
    current_heading_level: Option<u8>,
}

#[derive(Debug, Clone)]
enum FormattingState {
    Bold,
    Italic,
    Underline,
    Code,
}

#[derive(Debug, Clone)]
struct ListState {
    level: u8,
    ordered: bool,
}

#[derive(Debug)]
struct TableState {
    current_row: Vec<RtfNode>,
    rows: Vec<super::types::TableRow>,
    in_header: bool,
}

impl MarkdownToRtfConverter {
    fn new() -> Self {
        Self {
            document: RtfDocument {
                metadata: DocumentMetadata::default(),
                content: Vec::new(),
            },
            current_paragraph: Vec::new(),
            formatting_stack: Vec::new(),
            list_stack: Vec::new(),
            table_state: None,
            current_heading_level: None,
        }
    }

    fn process_event(&mut self, event: Event) -> ConversionResult<()> {
        match event {
            Event::Start(tag) => self.handle_start_tag(tag)?,
            Event::End(tag) => self.handle_end_tag(tag)?,
            Event::Text(text) => self.handle_text(text)?,
            Event::Code(code) => self.handle_code(code)?,
            Event::Html(_) => {}, // Skip HTML for now
            Event::FootnoteReference(_) => {}, // Skip footnotes for now
            Event::SoftBreak => self.handle_soft_break()?,
            Event::HardBreak => self.handle_hard_break()?,
            Event::Rule => self.handle_rule()?,
            Event::TaskListMarker(_) => {}, // Skip task list markers for now
        }
        Ok(())
    }

    fn handle_start_tag(&mut self, tag: Tag) -> ConversionResult<()> {
        match tag {
            Tag::Paragraph => {
                // Start new paragraph - current_paragraph will collect content
            }
            Tag::Heading(level, _, _) => {
                self.current_heading_level = Some(level as u8);
            }
            Tag::BlockQuote => {
                // TODO: Implement blockquotes
            }
            Tag::CodeBlock(_) => {
                // TODO: Implement code blocks
            }
            Tag::List(first_item_number) => {
                let ordered = first_item_number.is_some();
                let level = self.list_stack.len() as u8;
                self.list_stack.push(ListState { level, ordered });
            }
            Tag::Item => {
                // List item will be handled when we get text
            }
            Tag::FootnoteDefinition(_) => {
                // Skip footnotes for now
            }
            Tag::Table(_) => {
                self.table_state = Some(TableState {
                    current_row: Vec::new(),
                    rows: Vec::new(),
                    in_header: true,
                });
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
            Tag::TableCell => {
                // Cell content will be collected in current_paragraph
            }
            Tag::Emphasis => {
                self.formatting_stack.push(FormattingState::Italic);
            }
            Tag::Strong => {
                self.formatting_stack.push(FormattingState::Bold);
            }
            Tag::Strikethrough => {
                // RTF doesn't have native strikethrough, skip for now
            }
            Tag::Link { .. } => {
                // TODO: Implement links
            }
            Tag::Image { .. } => {
                // TODO: Implement images
            }
        }
        Ok(())
    }

    fn handle_end_tag(&mut self, tag: Tag) -> ConversionResult<()> {
        match tag {
            Tag::Paragraph => {
                if !self.current_paragraph.is_empty() {
                    let paragraph_content = std::mem::take(&mut self.current_paragraph);
                    
                    if let Some(level) = self.current_heading_level.take() {
                        // This was a heading
                        self.document.content.push(RtfNode::Heading {
                            level,
                            content: paragraph_content,
                        });
                    } else if !self.list_stack.is_empty() {
                        // This is a list item
                        let list_state = self.list_stack.last()
                            .expect("List stack should not be empty after check");
                        self.document.content.push(RtfNode::ListItem {
                            level: list_state.level,
                            content: paragraph_content,
                        });
                    } else {
                        // Regular paragraph
                        self.document.content.push(RtfNode::Paragraph(paragraph_content));
                    }
                }
            }
            Tag::Heading(..) => {
                // Handled in paragraph end
            }
            Tag::BlockQuote => {
                // TODO: Implement blockquotes
            }
            Tag::CodeBlock(_) => {
                // TODO: Implement code blocks
            }
            Tag::List(_) => {
                self.list_stack.pop();
            }
            Tag::Item => {
                // Handled in paragraph end
            }
            Tag::FootnoteDefinition(_) => {
                // Skip footnotes
            }
            Tag::Table(_) => {
                if let Some(table_state) = self.table_state.take() {
                    if !table_state.rows.is_empty() {
                        self.document.content.push(RtfNode::Table {
                            rows: table_state.rows,
                        });
                    }
                }
            }
            Tag::TableHead => {
                if let Some(ref mut table) = self.table_state {
                    table.in_header = false;
                }
            }
            Tag::TableRow => {
                if let Some(ref mut table) = self.table_state {
                    if !table.current_row.is_empty() {
                        let cells: Vec<super::types::TableCell> = table.current_row
                            .iter()
                            .map(|content| super::types::TableCell {
                                content: vec![content.clone()],
                            })
                            .collect();
                        
                        table.rows.push(super::types::TableRow { cells });
                        table.current_row.clear();
                    }
                }
            }
            Tag::TableCell => {
                if let Some(ref mut table) = self.table_state {
                    if !self.current_paragraph.is_empty() {
                        let cell_content = std::mem::take(&mut self.current_paragraph);
                        for node in cell_content {
                            table.current_row.push(node);
                        }
                    }
                }
            }
            Tag::Emphasis => {
                self.formatting_stack.retain(|f| !matches!(f, FormattingState::Italic));
            }
            Tag::Strong => {
                self.formatting_stack.retain(|f| !matches!(f, FormattingState::Bold));
            }
            Tag::Strikethrough => {
                // Skip strikethrough
            }
            Tag::Link { .. } => {
                // TODO: Implement links
            }
            Tag::Image { .. } => {
                // TODO: Implement images
            }
        }
        Ok(())
    }

    fn handle_text(&mut self, text: CowStr) -> ConversionResult<()> {
        let text_content = text.to_string();
        let mut current_node = RtfNode::Text(text_content);

        // Apply formatting stack in reverse order (innermost first)
        for formatting in self.formatting_stack.iter().rev() {
            current_node = match formatting {
                FormattingState::Bold => RtfNode::Bold(vec![current_node]),
                FormattingState::Italic => RtfNode::Italic(vec![current_node]),
                FormattingState::Underline => RtfNode::Underline(vec![current_node]),
                FormattingState::Code => {
                    // For now, treat code as plain text
                    current_node
                }
            };
        }

        self.current_paragraph.push(current_node);
        Ok(())
    }

    fn handle_code(&mut self, code: CowStr) -> ConversionResult<()> {
        // Handle inline code - for now just treat as plain text
        let text_node = RtfNode::Text(code.to_string());
        self.current_paragraph.push(text_node);
        Ok(())
    }

    fn handle_soft_break(&mut self) -> ConversionResult<()> {
        // Soft break - just add a space
        self.current_paragraph.push(RtfNode::Text(" ".to_string()));
        Ok(())
    }

    fn handle_hard_break(&mut self) -> ConversionResult<()> {
        // Hard break - add a line break
        self.current_paragraph.push(RtfNode::LineBreak);
        Ok(())
    }

    fn handle_rule(&mut self) -> ConversionResult<()> {
        // Horizontal rule - treat as page break for RTF
        self.document.content.push(RtfNode::PageBreak);
        Ok(())
    }

    fn finalize(mut self) -> RtfDocument {
        // Add any remaining paragraph content
        if !self.current_paragraph.is_empty() {
            let paragraph_content = std::mem::take(&mut self.current_paragraph);
            
            if let Some(level) = self.current_heading_level.take() {
                self.document.content.push(RtfNode::Heading {
                    level,
                    content: paragraph_content,
                });
            } else if !self.list_stack.is_empty() {
                let list_state = self.list_stack.last()
                    .expect("List stack should not be empty after check");
                self.document.content.push(RtfNode::ListItem {
                    level: list_state.level,
                    content: paragraph_content,
                });
            } else {
                self.document.content.push(RtfNode::Paragraph(paragraph_content));
            }
        }

        self.document
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_text() {
        let markdown = "Hello World";
        let document = MarkdownParser::parse(markdown).unwrap();
        
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
        let markdown = "Normal **bold** and *italic* text";
        let document = MarkdownParser::parse(markdown).unwrap();
        
        assert_eq!(document.content.len(), 1);
        match &document.content[0] {
            RtfNode::Paragraph(nodes) => {
                assert_eq!(nodes.len(), 5); // "Normal ", bold, " and ", italic, " text"
                
                match &nodes[1] {
                    RtfNode::Bold(bold_nodes) => {
                        assert_eq!(bold_nodes.len(), 1);
                        match &bold_nodes[0] {
                            RtfNode::Text(text) => assert_eq!(text, "bold"),
                            _ => panic!("Expected text in bold"),
                        }
                    }
                    _ => panic!("Expected bold node"),
                }
                
                match &nodes[3] {
                    RtfNode::Italic(italic_nodes) => {
                        assert_eq!(italic_nodes.len(), 1);
                        match &italic_nodes[0] {
                            RtfNode::Text(text) => assert_eq!(text, "italic"),
                            _ => panic!("Expected text in italic"),
                        }
                    }
                    _ => panic!("Expected italic node"),
                }
            }
            _ => panic!("Expected paragraph node"),
        }
    }

    #[test]
    fn test_parse_heading() {
        let markdown = "# Main Heading";
        let document = MarkdownParser::parse(markdown).unwrap();
        
        assert_eq!(document.content.len(), 1);
        match &document.content[0] {
            RtfNode::Heading { level, content } => {
                assert_eq!(*level, 1);
                assert_eq!(content.len(), 1);
                match &content[0] {
                    RtfNode::Text(text) => assert_eq!(text, "Main Heading"),
                    _ => panic!("Expected text in heading"),
                }
            }
            _ => panic!("Expected heading node"),
        }
    }

    #[test]
    fn test_parse_list() {
        let markdown = "- Item 1\n- Item 2";
        let document = MarkdownParser::parse(markdown).unwrap();
        
        assert_eq!(document.content.len(), 2);
        for (i, node) in document.content.iter().enumerate() {
            match node {
                RtfNode::ListItem { level, content } => {
                    assert_eq!(*level, 0);
                    assert_eq!(content.len(), 1);
                    match &content[0] {
                        RtfNode::Text(text) => {
                            assert_eq!(text, &format!("Item {}", i + 1));
                        }
                        _ => panic!("Expected text in list item"),
                    }
                }
                _ => panic!("Expected list item node"),
            }
        }
    }
}