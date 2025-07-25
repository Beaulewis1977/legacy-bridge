// Validation Layer - Document integrity checks
//
// This module provides comprehensive validation for RTF documents,
// including pre-conversion validation, post-conversion verification,
// structure integrity checks, and format compliance validation.

use crate::conversion::types::{RtfDocument, RtfNode, TableRow};
use crate::pipeline::{ValidationResult, ValidationLevel};
use regex::Regex;
use std::collections::HashSet;

/// Validation rules configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Check for proper RTF header
    pub check_header: bool,
    /// Check for balanced braces
    pub check_braces: bool,
    /// Check for valid control words
    pub check_control_words: bool,
    /// Check for proper encoding
    pub check_encoding: bool,
    /// Check for document structure
    pub check_structure: bool,
    /// Check for table integrity
    pub check_tables: bool,
    /// Check for font table
    pub require_font_table: bool,
    /// Check for maximum nesting depth
    pub max_nesting_depth: Option<usize>,
    /// Check for maximum file size
    pub max_file_size: Option<usize>,
    /// Allowed control words (if Some, only these are allowed)
    pub allowed_control_words: Option<HashSet<String>>,
    /// Forbidden control words
    pub forbidden_control_words: HashSet<String>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        let mut forbidden = HashSet::new();
        // Add potentially dangerous control words
        forbidden.insert("object".to_string());
        forbidden.insert("objdata".to_string());
        forbidden.insert("datafield".to_string());
        
        Self {
            check_header: true,
            check_braces: true,
            check_control_words: true,
            check_encoding: true,
            check_structure: true,
            check_tables: true,
            require_font_table: false,
            max_nesting_depth: Some(100),
            max_file_size: Some(50 * 1024 * 1024), // 50MB
            allowed_control_words: None,
            forbidden_control_words: forbidden,
        }
    }
}

/// Document validator
pub struct Validator {
    config: ValidationConfig,
    rtf_header_regex: Regex,
    control_word_regex: Regex,
}

impl Validator {
    pub fn new() -> Self {
        Self::with_config(ValidationConfig::default())
    }

    pub fn with_config(config: ValidationConfig) -> Self {
        // Use lazy_static or once_cell for compiled regexes to avoid unwrap
        // For now, we'll use expect() with descriptive messages since these are static patterns
        // that should never fail to compile
        Self {
            config,
            rtf_header_regex: Regex::new(r"^\{\\rtf1")
                .expect("Failed to compile RTF header regex - this is a bug"),
            control_word_regex: Regex::new(r"\\([a-zA-Z]+)(-?\d*)")
                .expect("Failed to compile control word regex - this is a bug"),
        }
    }

    /// Pre-validation: Check RTF content before parsing
    pub fn pre_validate(&self, rtf_content: &str) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        // Check file size
        if let Some(max_size) = self.config.max_file_size {
            if rtf_content.len() > max_size {
                results.push(ValidationResult {
                    level: ValidationLevel::Error,
                    code: "SIZE_EXCEEDED".to_string(),
                    message: format!(
                        "File size {} exceeds maximum allowed size of {} bytes",
                        rtf_content.len(),
                        max_size
                    ),
                    location: None,
                });
            }
        }

        // Check RTF header
        if self.config.check_header {
            if !self.rtf_header_regex.is_match(rtf_content) {
                results.push(ValidationResult {
                    level: ValidationLevel::Error,
                    code: "INVALID_HEADER".to_string(),
                    message: "Invalid or missing RTF header. Document must start with {\\rtf1".to_string(),
                    location: Some("Line 1".to_string()),
                });
            }
        }

        // Check brace balance
        if self.config.check_braces {
            let brace_result = self.check_brace_balance(rtf_content);
            results.extend(brace_result);
        }

        // Check encoding
        if self.config.check_encoding {
            let encoding_result = self.check_encoding(rtf_content);
            results.extend(encoding_result);
        }

        // Check control words
        if self.config.check_control_words {
            let control_word_result = self.check_control_words(rtf_content);
            results.extend(control_word_result);
        }

        // Check nesting depth
        if let Some(max_depth) = self.config.max_nesting_depth {
            let depth_result = self.check_nesting_depth(rtf_content, max_depth);
            results.extend(depth_result);
        }

        results
    }

    /// Post-validation: Check parsed document structure
    pub fn post_validate(&self, document: &RtfDocument) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        // Check document structure
        if self.config.check_structure {
            let structure_result = self.check_document_structure(document);
            results.extend(structure_result);
        }

        // Check tables
        if self.config.check_tables {
            let table_result = self.check_table_integrity(document);
            results.extend(table_result);
        }

        // Check font table
        if self.config.require_font_table && document.metadata.fonts.is_empty() {
            results.push(ValidationResult {
                level: ValidationLevel::Warning,
                code: "MISSING_FONT_TABLE".to_string(),
                message: "Document is missing font table definition".to_string(),
                location: Some("Document metadata".to_string()),
            });
        }

        results
    }

    /// Check brace balance
    fn check_brace_balance(&self, content: &str) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        let mut depth = 0;
        let mut max_depth = 0;
        let mut line_number = 1;
        let mut escaped = false;

        for (i, ch) in content.char_indices() {
            if ch == '\n' {
                line_number += 1;
            }

            if escaped {
                escaped = false;
                continue;
            }

            match ch {
                '\\' => escaped = true,
                '{' => {
                    depth += 1;
                    max_depth = max_depth.max(depth);
                }
                '}' => {
                    depth -= 1;
                    if depth < 0 {
                        results.push(ValidationResult {
                            level: ValidationLevel::Error,
                            code: "UNMATCHED_BRACE".to_string(),
                            message: format!("Unmatched closing brace at position {}", i),
                            location: Some(format!("Line {}", line_number)),
                        });
                        depth = 0; // Reset to continue checking
                    }
                }
                _ => {}
            }
        }

        if depth > 0 {
            results.push(ValidationResult {
                level: ValidationLevel::Error,
                code: "UNCLOSED_BRACE".to_string(),
                message: format!("{} unclosed opening brace(s)", depth),
                location: None,
            });
        }

        results
    }

    /// Check encoding issues
    fn check_encoding(&self, content: &str) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        // Check for null bytes
        if content.contains('\0') {
            results.push(ValidationResult {
                level: ValidationLevel::Error,
                code: "NULL_BYTE".to_string(),
                message: "Document contains null bytes which may indicate corruption".to_string(),
                location: None,
            });
        }

        // Check for invalid UTF-8 sequences (if applicable)
        // RTF is typically ASCII/ANSI, but we check for common issues
        let mut line_number = 1;
        for (i, ch) in content.char_indices() {
            if ch == '\n' {
                line_number += 1;
            }
            
            // Check for control characters (except common ones)
            if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
                results.push(ValidationResult {
                    level: ValidationLevel::Warning,
                    code: "CONTROL_CHARACTER".to_string(),
                    message: format!("Unexpected control character at position {}", i),
                    location: Some(format!("Line {}", line_number)),
                });
            }
        }

        results
    }

    /// Check control words
    fn check_control_words(&self, content: &str) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        for capture in self.control_word_regex.captures_iter(content) {
            let control_word = &capture[1];
            
            // Check forbidden control words
            if self.config.forbidden_control_words.contains(control_word) {
                results.push(ValidationResult {
                    level: ValidationLevel::Error,
                    code: "FORBIDDEN_CONTROL_WORD".to_string(),
                    message: format!("Forbidden control word '\\{}' detected", control_word),
                    location: None,
                });
            }
            
            // Check allowed control words
            if let Some(ref allowed) = self.config.allowed_control_words {
                if !allowed.contains(control_word) {
                    results.push(ValidationResult {
                        level: ValidationLevel::Warning,
                        code: "UNKNOWN_CONTROL_WORD".to_string(),
                        message: format!("Unknown or unsupported control word '\\{}'", control_word),
                        location: None,
                    });
                }
            }
        }

        results
    }

    /// Check nesting depth
    fn check_nesting_depth(&self, content: &str, max_depth: usize) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        let mut depth: usize = 0;
        let mut max_found = 0;
        let mut escaped = false;

        for ch in content.chars() {
            if escaped {
                escaped = false;
                continue;
            }

            match ch {
                '\\' => escaped = true,
                '{' => {
                    depth += 1;
                    max_found = max_found.max(depth);
                }
                '}' => {
                    depth = depth.saturating_sub(1);
                }
                _ => {}
            }
        }

        if max_found > max_depth {
            results.push(ValidationResult {
                level: ValidationLevel::Error,
                code: "MAX_NESTING_EXCEEDED".to_string(),
                message: format!(
                    "Maximum nesting depth {} exceeded (found {})",
                    max_depth, max_found
                ),
                location: None,
            });
        }

        results
    }

    /// Check document structure integrity
    fn check_document_structure(&self, document: &RtfDocument) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        // Check for empty document
        if document.content.is_empty() {
            results.push(ValidationResult {
                level: ValidationLevel::Warning,
                code: "EMPTY_DOCUMENT".to_string(),
                message: "Document has no content".to_string(),
                location: Some("Document root".to_string()),
            });
        }

        // Validate node structure
        for (index, node) in document.content.iter().enumerate() {
            self.validate_node(node, &mut results, &format!("Node {}", index));
        }

        results
    }

    /// Validate individual node
    fn validate_node(&self, node: &RtfNode, results: &mut Vec<ValidationResult>, location: &str) {
        match node {
            RtfNode::Heading { level, content } => {
                if *level == 0 || *level > 6 {
                    results.push(ValidationResult {
                        level: ValidationLevel::Warning,
                        code: "INVALID_HEADING_LEVEL".to_string(),
                        message: format!("Invalid heading level: {}", level),
                        location: Some(location.to_string()),
                    });
                }
                for child in content {
                    self.validate_node(child, results, &format!("{} > content", location));
                }
            }
            
            RtfNode::Paragraph(children) |
            RtfNode::Bold(children) |
            RtfNode::Italic(children) |
            RtfNode::Underline(children) => {
                for (i, child) in children.iter().enumerate() {
                    self.validate_node(child, results, &format!("{} > child {}", location, i));
                }
            }
            
            RtfNode::ListItem { content, .. } => {
                for (i, child) in content.iter().enumerate() {
                    self.validate_node(child, results, &format!("{} > item {}", location, i));
                }
            }
            
            RtfNode::Table { rows } => {
                self.validate_table_structure(rows, results, location);
            }
            
            _ => {} // Text, LineBreak, PageBreak are always valid
        }
    }

    /// Check table integrity
    fn check_table_integrity(&self, document: &RtfDocument) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        let mut table_count = 0;

        fn find_tables<'a>(nodes: &'a [RtfNode], table_count: &mut usize) -> Vec<&'a RtfNode> {
            let mut tables = Vec::new();
            for node in nodes {
                match node {
                    RtfNode::Table { .. } => {
                        *table_count += 1;
                        tables.push(node);
                    }
                    RtfNode::Paragraph(children) |
                    RtfNode::Bold(children) |
                    RtfNode::Italic(children) |
                    RtfNode::Underline(children) |
                    RtfNode::Heading { content: children, .. } |
                    RtfNode::ListItem { content: children, .. } => {
                        tables.extend(find_tables(children, table_count));
                    }
                    _ => {}
                }
            }
            tables
        }

        let tables = find_tables(&document.content, &mut table_count);

        for (index, table_node) in tables.iter().enumerate() {
            if let RtfNode::Table { rows } = table_node {
                self.validate_table_structure(rows, &mut results, &format!("Table {}", index + 1));
            }
        }

        results
    }

    /// Validate table structure
    fn validate_table_structure(
        &self,
        rows: &[TableRow],
        results: &mut Vec<ValidationResult>,
        location: &str,
    ) {
        if rows.is_empty() {
            results.push(ValidationResult {
                level: ValidationLevel::Warning,
                code: "EMPTY_TABLE".to_string(),
                message: "Table has no rows".to_string(),
                location: Some(location.to_string()),
            });
            return;
        }

        // Check for consistent column count
        let first_row_cols = rows[0].cells.len();
        let mut inconsistent_rows = Vec::new();

        for (i, row) in rows.iter().enumerate() {
            if row.cells.is_empty() {
                results.push(ValidationResult {
                    level: ValidationLevel::Warning,
                    code: "EMPTY_ROW".to_string(),
                    message: format!("Row {} has no cells", i + 1),
                    location: Some(format!("{} > Row {}", location, i + 1)),
                });
            }

            if row.cells.len() != first_row_cols {
                inconsistent_rows.push(i + 1);
            }
        }

        if !inconsistent_rows.is_empty() {
            results.push(ValidationResult {
                level: ValidationLevel::Warning,
                code: "INCONSISTENT_COLUMNS".to_string(),
                message: format!(
                    "Table has inconsistent column count. First row has {} columns, but rows {:?} have different counts",
                    first_row_cols, inconsistent_rows
                ),
                location: Some(location.to_string()),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brace_validation() {
        let validator = Validator::new();
        
        // Valid RTF
        let valid_rtf = r"{\rtf1 Hello world}";
        let results = validator.pre_validate(valid_rtf);
        assert!(results.iter().all(|r| r.level != ValidationLevel::Error));
        
        // Unmatched closing brace
        let invalid_rtf = r"{\rtf1 Hello world}}";
        let results = validator.pre_validate(invalid_rtf);
        assert!(results.iter().any(|r| r.code == "UNMATCHED_BRACE"));
        
        // Unclosed opening brace
        let invalid_rtf2 = r"{\rtf1 Hello {world}";
        let results = validator.pre_validate(invalid_rtf2);
        assert!(results.iter().any(|r| r.code == "UNCLOSED_BRACE"));
    }

    #[test]
    fn test_header_validation() {
        let validator = Validator::new();
        
        // Missing header
        let no_header = r"{Hello world}";
        let results = validator.pre_validate(no_header);
        assert!(results.iter().any(|r| r.code == "INVALID_HEADER"));
    }
}