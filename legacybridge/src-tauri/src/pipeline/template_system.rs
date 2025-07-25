// Template System - Enterprise document templates
//
// This module provides a template system for applying enterprise-specific
// formatting and structure to converted documents.

use crate::conversion::types::{
    ConversionError, ConversionResult, RtfDocument, RtfNode, DocumentMetadata,
    FontInfo, FontFamily, Color,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTemplate {
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Template version
    pub version: String,
    /// Template type (memo, report, letter, etc.)
    pub template_type: TemplateType,
    /// Header configuration
    pub header: Option<HeaderConfig>,
    /// Footer configuration
    pub footer: Option<FooterConfig>,
    /// Style definitions
    pub styles: HashMap<String, StyleDefinition>,
    /// Document metadata overrides
    pub metadata_overrides: Option<MetadataOverrides>,
    /// Content transformations
    pub transformations: Vec<ContentTransformation>,
    /// Legacy system compatibility settings
    pub legacy_settings: Option<LegacySettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TemplateType {
    Memo,
    Report,
    Letter,
    Invoice,
    Contract,
    Manual,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderConfig {
    /// Header content template
    pub content: String,
    /// Variables to replace in content
    pub variables: HashMap<String, String>,
    /// Alignment
    pub alignment: Alignment,
    /// Include page numbers
    pub include_page_numbers: bool,
    /// Include date
    pub include_date: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FooterConfig {
    /// Footer content template
    pub content: String,
    /// Variables to replace in content
    pub variables: HashMap<String, String>,
    /// Alignment
    pub alignment: Alignment,
    /// Include page numbers
    pub include_page_numbers: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleDefinition {
    /// Style name
    pub name: String,
    /// Font settings
    pub font: Option<FontSettings>,
    /// Paragraph settings
    pub paragraph: Option<ParagraphSettings>,
    /// List settings
    pub list: Option<ListSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSettings {
    pub family: String,
    pub size: i32,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphSettings {
    pub alignment: Alignment,
    pub line_spacing: f32,
    pub space_before: i32,
    pub space_after: i32,
    pub first_line_indent: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSettings {
    pub list_type: String,
    pub indent_per_level: i32,
    pub numbering_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataOverrides {
    pub default_font: Option<String>,
    pub author: Option<String>,
    pub company: Option<String>,
    pub department: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTransformation {
    /// Transformation type
    pub transform_type: TransformationType,
    /// Target elements (heading, paragraph, list, etc.)
    pub target: TransformTarget,
    /// Transformation parameters
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformationType {
    /// Add prefix or suffix
    AddWrapper,
    /// Replace content matching pattern
    ReplacePattern,
    /// Apply style
    ApplyStyle,
    /// Insert element
    InsertElement,
    /// Restructure content
    Restructure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformTarget {
    AllHeadings,
    HeadingLevel(u8),
    AllParagraphs,
    FirstParagraph,
    AllLists,
    Tables,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacySettings {
    /// VB6 compatibility mode
    pub vb6_compatible: bool,
    /// VFP9 compatibility mode
    pub vfp9_compatible: bool,
    /// Use legacy encoding
    pub use_legacy_encoding: bool,
    /// Legacy date format
    pub date_format: Option<String>,
    /// Legacy number format
    pub number_format: Option<String>,
}

/// Template system implementation
pub struct TemplateSystem {
    templates: HashMap<String, DocumentTemplate>,
    template_dir: Option<String>,
}

impl TemplateSystem {
    pub fn new() -> Self {
        let mut system = Self {
            templates: HashMap::new(),
            template_dir: None,
        };
        
        // Load built-in templates
        system.load_builtin_templates();
        
        system
    }

    /// Load built-in enterprise templates
    fn load_builtin_templates(&mut self) {
        // Memo template
        self.templates.insert(
            "memo".to_string(),
            DocumentTemplate {
                name: "Enterprise Memo".to_string(),
                description: "Standard enterprise memo template".to_string(),
                version: "1.0.0".to_string(),
                template_type: TemplateType::Memo,
                header: Some(HeaderConfig {
                    content: "{{company}} - Internal Memorandum".to_string(),
                    variables: HashMap::from([
                        ("company".to_string(), "ACME Corporation".to_string()),
                    ]),
                    alignment: Alignment::Center,
                    include_page_numbers: true,
                    include_date: true,
                }),
                footer: Some(FooterConfig {
                    content: "Confidential - Page {{page}}".to_string(),
                    variables: HashMap::new(),
                    alignment: Alignment::Center,
                    include_page_numbers: true,
                }),
                styles: self.create_memo_styles(),
                metadata_overrides: Some(MetadataOverrides {
                    default_font: Some("Arial".to_string()),
                    author: None,
                    company: Some("ACME Corporation".to_string()),
                    department: None,
                }),
                transformations: vec![
                    ContentTransformation {
                        transform_type: TransformationType::ApplyStyle,
                        target: TransformTarget::HeadingLevel(1),
                        parameters: HashMap::from([
                            ("style".to_string(), "memo-heading".to_string()),
                        ]),
                    },
                ],
                legacy_settings: Some(LegacySettings {
                    vb6_compatible: true,
                    vfp9_compatible: true,
                    use_legacy_encoding: false,
                    date_format: Some("MM/DD/YYYY".to_string()),
                    number_format: Some("#,##0.00".to_string()),
                }),
            },
        );

        // Report template
        self.templates.insert(
            "report".to_string(),
            DocumentTemplate {
                name: "Enterprise Report".to_string(),
                description: "Standard enterprise report template".to_string(),
                version: "1.0.0".to_string(),
                template_type: TemplateType::Report,
                header: Some(HeaderConfig {
                    content: "{{title}} - {{date}}".to_string(),
                    variables: HashMap::new(),
                    alignment: Alignment::Right,
                    include_page_numbers: true,
                    include_date: true,
                }),
                footer: Some(FooterConfig {
                    content: "{{company}} - Proprietary Information".to_string(),
                    variables: HashMap::from([
                        ("company".to_string(), "ACME Corporation".to_string()),
                    ]),
                    alignment: Alignment::Center,
                    include_page_numbers: true,
                }),
                styles: self.create_report_styles(),
                metadata_overrides: Some(MetadataOverrides {
                    default_font: Some("Times New Roman".to_string()),
                    author: None,
                    company: Some("ACME Corporation".to_string()),
                    department: None,
                }),
                transformations: vec![
                    ContentTransformation {
                        transform_type: TransformationType::AddWrapper,
                        target: TransformTarget::FirstParagraph,
                        parameters: HashMap::from([
                            ("prefix".to_string(), "Executive Summary: ".to_string()),
                        ]),
                    },
                ],
                legacy_settings: None,
            },
        );
    }

    /// Create memo styles
    fn create_memo_styles(&self) -> HashMap<String, StyleDefinition> {
        let mut styles = HashMap::new();
        
        styles.insert(
            "memo-heading".to_string(),
            StyleDefinition {
                name: "Memo Heading".to_string(),
                font: Some(FontSettings {
                    family: "Arial".to_string(),
                    size: 14,
                    bold: true,
                    italic: false,
                    underline: false,
                    color: Some("#000080".to_string()),
                }),
                paragraph: Some(ParagraphSettings {
                    alignment: Alignment::Center,
                    line_spacing: 1.5,
                    space_before: 12,
                    space_after: 6,
                    first_line_indent: 0,
                }),
                list: None,
            },
        );
        
        styles
    }

    /// Create report styles
    fn create_report_styles(&self) -> HashMap<String, StyleDefinition> {
        let mut styles = HashMap::new();
        
        styles.insert(
            "report-title".to_string(),
            StyleDefinition {
                name: "Report Title".to_string(),
                font: Some(FontSettings {
                    family: "Times New Roman".to_string(),
                    size: 18,
                    bold: true,
                    italic: false,
                    underline: false,
                    color: None,
                }),
                paragraph: Some(ParagraphSettings {
                    alignment: Alignment::Center,
                    line_spacing: 2.0,
                    space_before: 24,
                    space_after: 12,
                    first_line_indent: 0,
                }),
                list: None,
            },
        );
        
        styles
    }

    /// Apply template to document
    pub fn apply_template(
        &self,
        document: &mut RtfDocument,
        template_name: &str,
    ) -> ConversionResult<()> {
        let template = self.templates.get(template_name)
            .ok_or_else(|| ConversionError::GenerationError(
                format!("Template '{}' not found", template_name)
            ))?;

        // Apply metadata overrides
        if let Some(ref overrides) = template.metadata_overrides {
            self.apply_metadata_overrides(&mut document.metadata, overrides);
        }

        // Apply content transformations
        for transformation in &template.transformations {
            self.apply_transformation(&mut document.content, transformation)?;
        }

        // Add header/footer if configured
        if template.header.is_some() || template.footer.is_some() {
            self.add_header_footer(document, template)?;
        }

        Ok(())
    }

    /// Apply metadata overrides
    fn apply_metadata_overrides(
        &self,
        metadata: &mut DocumentMetadata,
        overrides: &MetadataOverrides,
    ) {
        if let Some(ref font) = overrides.default_font {
            metadata.default_font = Some(font.clone());
            
            // Ensure font is in font table
            if !metadata.fonts.iter().any(|f| &f.name == font) {
                metadata.fonts.push(FontInfo {
                    id: metadata.fonts.len() as i32,
                    name: font.clone(),
                    family: FontFamily::Swiss,
                });
            }
        }

        if let Some(ref author) = overrides.author {
            metadata.author = Some(author.clone());
        }
    }

    /// Apply content transformation
    fn apply_transformation(
        &self,
        content: &mut Vec<RtfNode>,
        transformation: &ContentTransformation,
    ) -> ConversionResult<()> {
        match transformation.transform_type {
            TransformationType::ApplyStyle => {
                self.apply_style_transformation(content, transformation)?;
            }
            TransformationType::AddWrapper => {
                self.apply_wrapper_transformation(content, transformation)?;
            }
            TransformationType::ReplacePattern => {
                self.apply_replace_transformation(content, transformation)?;
            }
            TransformationType::InsertElement => {
                self.apply_insert_transformation(content, transformation)?;
            }
            TransformationType::Restructure => {
                self.apply_restructure_transformation(content, transformation)?;
            }
        }
        
        Ok(())
    }

    /// Apply style transformation
    fn apply_style_transformation(
        &self,
        content: &mut Vec<RtfNode>,
        transformation: &ContentTransformation,
    ) -> ConversionResult<()> {
        let style_name = transformation.parameters.get("style")
            .ok_or_else(|| ConversionError::GenerationError(
                "Style transformation requires 'style' parameter".to_string()
            ))?;

        // Apply to matching nodes
        self.transform_nodes_recursive(content, &transformation.target, |node| {
            // In a real implementation, we would apply the style here
            // For now, we'll just mark it as transformed
            match node {
                RtfNode::Heading { .. } => {
                    // Apply heading style
                }
                RtfNode::Paragraph(_) => {
                    // Apply paragraph style
                }
                _ => {}
            }
        });

        Ok(())
    }

    /// Apply wrapper transformation
    fn apply_wrapper_transformation(
        &self,
        content: &mut Vec<RtfNode>,
        transformation: &ContentTransformation,
    ) -> ConversionResult<()> {
        let prefix = transformation.parameters.get("prefix").cloned().unwrap_or_default();
        let suffix = transformation.parameters.get("suffix").cloned().unwrap_or_default();

        match transformation.target {
            TransformTarget::FirstParagraph => {
                if let Some(first_para) = content.iter_mut().find(|n| matches!(n, RtfNode::Paragraph(_))) {
                    if let RtfNode::Paragraph(children) = first_para {
                        if !prefix.is_empty() {
                            children.insert(0, RtfNode::Text(prefix));
                        }
                        if !suffix.is_empty() {
                            children.push(RtfNode::Text(suffix));
                        }
                    }
                }
            }
            _ => {
                // Handle other targets
            }
        }

        Ok(())
    }

    /// Apply replace transformation
    fn apply_replace_transformation(
        &self,
        content: &mut Vec<RtfNode>,
        transformation: &ContentTransformation,
    ) -> ConversionResult<()> {
        let pattern = transformation.parameters.get("pattern")
            .ok_or_else(|| ConversionError::GenerationError(
                "Replace transformation requires 'pattern' parameter".to_string()
            ))?;
        let replacement = transformation.parameters.get("replacement").cloned().unwrap_or_default();

        self.replace_text_recursive(content, pattern, &replacement);
        
        Ok(())
    }

    /// Apply insert transformation
    fn apply_insert_transformation(
        &self,
        content: &mut Vec<RtfNode>,
        transformation: &ContentTransformation,
    ) -> ConversionResult<()> {
        let element_type = transformation.parameters.get("element")
            .ok_or_else(|| ConversionError::GenerationError(
                "Insert transformation requires 'element' parameter".to_string()
            ))?;
        let position = transformation.parameters.get("position").cloned().unwrap_or_else(|| "start".to_string());

        let new_element = match element_type.as_str() {
            "page_break" => RtfNode::PageBreak,
            "line_break" => RtfNode::LineBreak,
            _ => RtfNode::Text(format!("[{}]", element_type)),
        };

        match position.as_str() {
            "start" => content.insert(0, new_element),
            "end" => content.push(new_element),
            _ => {} // Handle other positions
        }

        Ok(())
    }

    /// Apply restructure transformation
    fn apply_restructure_transformation(
        &self,
        _content: &mut Vec<RtfNode>,
        _transformation: &ContentTransformation,
    ) -> ConversionResult<()> {
        // Complex restructuring logic would go here
        Ok(())
    }

    /// Transform nodes recursively
    fn transform_nodes_recursive<F>(
        &self,
        nodes: &mut Vec<RtfNode>,
        target: &TransformTarget,
        mut transform_fn: F,
    ) where
        F: FnMut(&mut RtfNode),
    {
        for node in nodes.iter_mut() {
            let should_transform = match (target, node) {
                (TransformTarget::AllHeadings, RtfNode::Heading { .. }) => true,
                (TransformTarget::HeadingLevel(level), RtfNode::Heading { level: h_level, .. }) => h_level == level,
                (TransformTarget::AllParagraphs, RtfNode::Paragraph(_)) => true,
                (TransformTarget::AllLists, RtfNode::ListItem { .. }) => true,
                (TransformTarget::Tables, RtfNode::Table { .. }) => true,
                _ => false,
            };

            if should_transform {
                transform_fn(node);
            }

            // Recurse into children
            match node {
                RtfNode::Paragraph(children) |
                RtfNode::Bold(children) |
                RtfNode::Italic(children) |
                RtfNode::Underline(children) |
                RtfNode::Heading { content: children, .. } |
                RtfNode::ListItem { content: children, .. } => {
                    self.transform_nodes_recursive(children, target, &mut transform_fn);
                }
                _ => {}
            }
        }
    }

    /// Replace text recursively
    fn replace_text_recursive(&self, nodes: &mut Vec<RtfNode>, pattern: &str, replacement: &str) {
        for node in nodes.iter_mut() {
            match node {
                RtfNode::Text(text) => {
                    *text = text.replace(pattern, replacement);
                }
                RtfNode::Paragraph(children) |
                RtfNode::Bold(children) |
                RtfNode::Italic(children) |
                RtfNode::Underline(children) |
                RtfNode::Heading { content: children, .. } |
                RtfNode::ListItem { content: children, .. } => {
                    self.replace_text_recursive(children, pattern, replacement);
                }
                RtfNode::Table { rows } => {
                    for row in rows {
                        for cell in &mut row.cells {
                            self.replace_text_recursive(&mut cell.content, pattern, replacement);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Add header and footer to document
    fn add_header_footer(
        &self,
        document: &mut RtfDocument,
        template: &DocumentTemplate,
    ) -> ConversionResult<()> {
        // In a real implementation, this would add proper RTF header/footer
        // For now, we'll add them as content at the beginning and end
        
        if let Some(ref header) = template.header {
            let header_text = self.process_template_variables(&header.content, &header.variables);
            document.content.insert(0, RtfNode::Paragraph(vec![
                RtfNode::Text(header_text),
                RtfNode::LineBreak,
            ]));
        }

        if let Some(ref footer) = template.footer {
            let footer_text = self.process_template_variables(&footer.content, &footer.variables);
            document.content.push(RtfNode::Paragraph(vec![
                RtfNode::LineBreak,
                RtfNode::Text(footer_text),
            ]));
        }

        Ok(())
    }

    /// Process template variables
    fn process_template_variables(&self, template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        
        for (key, value) in variables {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        
        // Handle built-in variables
        result = result.replace("{{date}}", &chrono::Local::now().format("%Y-%m-%d").to_string());
        result = result.replace("{{time}}", &chrono::Local::now().format("%H:%M:%S").to_string());
        
        result
    }

    /// Load custom template from file
    pub fn load_template_from_file(&mut self, path: &Path) -> ConversionResult<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConversionError::IoError(e.to_string()))?;
        
        let template: DocumentTemplate = serde_json::from_str(&content)
            .map_err(|e| ConversionError::ParseError(format!("Invalid template format: {}", e)))?;
        
        self.templates.insert(template.name.clone(), template);
        
        Ok(())
    }

    /// List available templates
    pub fn list_templates(&self) -> Vec<&str> {
        self.templates.keys().map(|k| k.as_str()).collect()
    }

    /// Get template by name
    pub fn get_template(&self, name: &str) -> Option<&DocumentTemplate> {
        self.templates.get(name)
    }
}