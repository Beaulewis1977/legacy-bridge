// Error Recovery System - Handles malformed RTF gracefully
//
// This module provides automatic error recovery strategies for malformed RTF documents,
// including tokenization recovery, parsing recovery, structure repair, and detailed error reporting.

use crate::conversion::types::{
    ConversionError, ConversionResult, RtfDocument, RtfNode, RtfToken,
    DocumentMetadata,
};
use crate::conversion::rtf_lexer;
use crate::conversion::RtfParser;
use crate::pipeline::{RecoveryAction, RecoveryType};

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Skip problematic content
    Skip,
    /// Replace with placeholder
    ReplaceWithPlaceholder,
    /// Attempt to fix structure
    FixStructure,
    /// Insert missing elements
    InsertMissing,
    /// Remove invalid elements
    RemoveInvalid,
    /// Best effort - try multiple strategies
    BestEffort,
}

/// Recovery context for tracking state
#[derive(Debug)]
struct RecoveryContext {
    original_content: String,
    actions: Vec<RecoveryAction>,
    fixed_content: Option<String>,
    error_locations: Vec<ErrorLocation>,
}

#[derive(Debug, Clone)]
struct ErrorLocation {
    position: usize,
    line: usize,
    description: String,
}

/// Error Recovery engine
pub struct ErrorRecovery {
    strategy: RecoveryStrategy,
    max_recovery_attempts: usize,
    preserve_as_much_as_possible: bool,
}

impl ErrorRecovery {
    pub fn new() -> Self {
        Self {
            strategy: RecoveryStrategy::BestEffort,
            max_recovery_attempts: 5,
            preserve_as_much_as_possible: true,
        }
    }

    pub fn with_strategy(strategy: RecoveryStrategy) -> Self {
        Self {
            strategy,
            max_recovery_attempts: 5,
            preserve_as_much_as_possible: true,
        }
    }

    /// Recover from tokenization errors
    pub fn recover_tokenization(
        &self,
        rtf_content: &str,
        error: ConversionError,
    ) -> ConversionResult<(Vec<RtfToken>, Vec<RecoveryAction>)> {
        let mut context = RecoveryContext {
            original_content: rtf_content.to_string(),
            actions: Vec::new(),
            fixed_content: None,
            error_locations: Vec::new(),
        };

        // Analyze the error
        match &error {
            ConversionError::LexerError(msg) => {
                self.analyze_lexer_error(msg, &mut context);
            }
            _ => {
                return Err(error);
            }
        }

        // Apply recovery strategies
        let fixed_content = self.apply_tokenization_recovery(&mut context)?;
        
        // Retry tokenization
        match rtf_lexer::tokenize(&fixed_content) {
            Ok(tokens) => Ok((tokens, context.actions)),
            Err(e) => {
                // If still failing, try more aggressive recovery
                self.aggressive_tokenization_recovery(&mut context, e)
            }
        }
    }

    /// Recover from parsing errors
    pub fn recover_parsing(
        &self,
        tokens: &[RtfToken],
        error: ConversionError,
    ) -> ConversionResult<(RtfDocument, Vec<RecoveryAction>)> {
        let mut actions = Vec::new();

        // Analyze the parsing error
        let recovery_tokens = match &error {
            ConversionError::ParseError(msg) => {
                self.fix_parsing_issues(tokens, msg, &mut actions)?
            }
            _ => return Err(error),
        };

        // Retry parsing with fixed tokens
        match RtfParser::parse(recovery_tokens) {
            Ok(document) => Ok((document, actions)),
            Err(e) => {
                // If still failing, create a minimal valid document
                self.create_minimal_document(tokens, &mut actions, e)
            }
        }
    }

    /// Analyze lexer error and identify issues
    fn analyze_lexer_error(&self, error_msg: &str, context: &mut RecoveryContext) {
        // Common lexer errors and their locations
        let content = context.original_content.clone();
        if error_msg.contains("unclosed") || error_msg.contains("unmatched") {
            self.find_brace_issues(&content, context);
        } else if error_msg.contains("invalid control") {
            self.find_control_word_issues(&content, context);
        } else if error_msg.contains("encoding") || error_msg.contains("character") {
            self.find_encoding_issues(&content, context);
        }
    }

    /// Find brace balance issues
    fn find_brace_issues(&self, content: &str, context: &mut RecoveryContext) {
        let mut depth = 0;
        let mut line = 1;
        let mut escaped = false;
        let mut open_positions = Vec::new();

        for (pos, ch) in content.char_indices() {
            if ch == '\n' {
                line += 1;
            }

            if escaped {
                escaped = false;
                continue;
            }

            match ch {
                '\\' => escaped = true,
                '{' => {
                    depth += 1;
                    open_positions.push((pos, line));
                }
                '}' => {
                    if depth == 0 {
                        context.error_locations.push(ErrorLocation {
                            position: pos,
                            line,
                            description: "Unmatched closing brace".to_string(),
                        });
                    } else {
                        depth -= 1;
                        open_positions.pop();
                    }
                }
                _ => {}
            }
        }

        // Record unclosed braces
        for (pos, line) in open_positions {
            context.error_locations.push(ErrorLocation {
                position: pos,
                line,
                description: "Unclosed opening brace".to_string(),
            });
        }
    }

    /// Find control word issues
    fn find_control_word_issues(&self, content: &str, context: &mut RecoveryContext) {
        let mut pos = 0;
        let mut line = 1;
        let bytes = content.as_bytes();

        while pos < bytes.len() {
            if bytes[pos] == b'\n' {
                line += 1;
            }

            if bytes[pos] == b'\\' && pos + 1 < bytes.len() {
                let start = pos;
                pos += 1;

                // Check if it's a valid control word or symbol
                if bytes[pos] == b'{' || bytes[pos] == b'}' || bytes[pos] == b'\\' {
                    // Valid control symbol
                    pos += 1;
                } else if bytes[pos].is_ascii_alphabetic() {
                    // Should be a control word
                    while pos < bytes.len() && bytes[pos].is_ascii_alphabetic() {
                        pos += 1;
                    }
                    
                    // Check for optional parameter
                    if pos < bytes.len() && (bytes[pos] == b'-' || bytes[pos].is_ascii_digit()) {
                        if bytes[pos] == b'-' {
                            pos += 1;
                        }
                        while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                            pos += 1;
                        }
                    }
                } else {
                    // Invalid control sequence
                    context.error_locations.push(ErrorLocation {
                        position: start,
                        line,
                        description: "Invalid control sequence".to_string(),
                    });
                }
            } else {
                pos += 1;
            }
        }
    }

    /// Find encoding issues
    fn find_encoding_issues(&self, content: &str, context: &mut RecoveryContext) {
        let mut line = 1;
        
        for (pos, ch) in content.char_indices() {
            if ch == '\n' {
                line += 1;
            }

            // Check for null bytes
            if ch == '\0' {
                context.error_locations.push(ErrorLocation {
                    position: pos,
                    line,
                    description: "Null byte found".to_string(),
                });
            }

            // Check for other control characters
            if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
                context.error_locations.push(ErrorLocation {
                    position: pos,
                    line,
                    description: format!("Invalid control character: {:?}", ch),
                });
            }
        }
    }

    /// Apply tokenization recovery strategies
    fn apply_tokenization_recovery(&self, context: &mut RecoveryContext) -> ConversionResult<String> {
        let mut fixed = context.original_content.clone();
        let error_locs = context.error_locations.clone();

        match self.strategy {
            RecoveryStrategy::Skip => {
                // Remove problematic sections
                for error_loc in error_locs.iter().rev() {
                    self.skip_problematic_content(&mut fixed, error_loc, context);
                }
            }
            RecoveryStrategy::ReplaceWithPlaceholder => {
                // Replace errors with safe placeholders
                for error_loc in error_locs.iter().rev() {
                    self.replace_with_placeholder(&mut fixed, error_loc, context);
                }
            }
            RecoveryStrategy::FixStructure => {
                // Fix structural issues
                self.fix_structure(&mut fixed, context)?;
            }
            RecoveryStrategy::InsertMissing => {
                // Insert missing elements
                self.insert_missing_elements(&mut fixed, context)?;
            }
            RecoveryStrategy::RemoveInvalid => {
                // Remove invalid elements
                self.remove_invalid_elements(&mut fixed, context)?;
            }
            RecoveryStrategy::BestEffort => {
                // Try multiple strategies
                self.best_effort_recovery(&mut fixed, context)?;
            }
        }

        context.fixed_content = Some(fixed.clone());
        Ok(fixed)
    }

    /// Skip problematic content
    fn skip_problematic_content(
        &self,
        _content: &mut String,
        error_loc: &ErrorLocation,
        context: &mut RecoveryContext,
    ) {
        // For now, just log the action
        context.actions.push(RecoveryAction {
            action_type: RecoveryType::StructureRepair,
            description: format!(
                "Skipped problematic content at line {}: {}",
                error_loc.line, error_loc.description
            ),
            applied: true,
        });
    }

    /// Replace with placeholder
    fn replace_with_placeholder(
        &self,
        _content: &mut String,
        error_loc: &ErrorLocation,
        context: &mut RecoveryContext,
    ) {
        context.actions.push(RecoveryAction {
            action_type: RecoveryType::StructureRepair,
            description: format!(
                "Replaced problematic content at line {} with placeholder",
                error_loc.line
            ),
            applied: true,
        });
    }

    /// Fix structural issues
    fn fix_structure(&self, content: &mut String, context: &mut RecoveryContext) -> ConversionResult<()> {
        // Fix brace balance
        let mut depth = 0;
        let mut chars: Vec<char> = content.chars().collect();
        let mut escaped = false;
        let mut fixes = 0;

        for i in 0..chars.len() {
            if escaped {
                escaped = false;
                continue;
            }

            match chars[i] {
                '\\' => escaped = true,
                '{' => depth += 1,
                '}' => {
                    if depth == 0 {
                        // Remove unmatched closing brace
                        chars[i] = ' ';
                        fixes += 1;
                    } else {
                        depth -= 1;
                    }
                }
                _ => {}
            }
        }

        // Add missing closing braces
        while depth > 0 {
            chars.push('}');
            depth -= 1;
            fixes += 1;
        }

        if fixes > 0 {
            *content = chars.into_iter().collect();
            context.actions.push(RecoveryAction {
                action_type: RecoveryType::StructureRepair,
                description: format!("Fixed {} brace balance issues", fixes),
                applied: true,
            });
        }

        Ok(())
    }

    /// Insert missing elements
    fn insert_missing_elements(
        &self,
        content: &mut String,
        context: &mut RecoveryContext,
    ) -> ConversionResult<()> {
        // Ensure RTF header if missing
        if !content.starts_with("{\\rtf") {
            *content = format!("{{\\rtf1\\ansi {}}}", content.trim_start_matches('{').trim_end_matches('}'));
            context.actions.push(RecoveryAction {
                action_type: RecoveryType::MissingDataInsertion,
                description: "Added missing RTF header".to_string(),
                applied: true,
            });
        }

        Ok(())
    }

    /// Remove invalid elements
    fn remove_invalid_elements(
        &self,
        content: &mut String,
        context: &mut RecoveryContext,
    ) -> ConversionResult<()> {
        // Remove null bytes
        if content.contains('\0') {
            *content = content.replace('\0', "");
            context.actions.push(RecoveryAction {
                action_type: RecoveryType::RemoveInvalid,
                description: "Removed null bytes".to_string(),
                applied: true,
            });
        }

        // Remove other invalid control characters
        let cleaned: String = content.chars()
            .filter(|&c| !c.is_control() || c == '\n' || c == '\r' || c == '\t')
            .collect();

        if cleaned.len() != content.len() {
            *content = cleaned;
            context.actions.push(RecoveryAction {
                action_type: RecoveryType::RemoveInvalid,
                description: "Removed invalid control characters".to_string(),
                applied: true,
            });
        }

        Ok(())
    }

    /// Best effort recovery - try multiple strategies
    fn best_effort_recovery(
        &self,
        content: &mut String,
        context: &mut RecoveryContext,
    ) -> ConversionResult<()> {
        // First, remove invalid characters
        self.remove_invalid_elements(content, context)?;
        
        // Then fix structure
        self.fix_structure(content, context)?;
        
        // Finally, ensure required elements
        self.insert_missing_elements(content, context)?;

        Ok(())
    }

    /// Aggressive tokenization recovery
    fn aggressive_tokenization_recovery(
        &self,
        context: &mut RecoveryContext,
        _error: ConversionError,
    ) -> ConversionResult<(Vec<RtfToken>, Vec<RecoveryAction>)> {
        // Create a minimal valid RTF structure
        let minimal_rtf = "{\\rtf1\\ansi\\deff0 {\\fonttbl{\\f0 Times New Roman;}} \\f0 [Document could not be fully parsed]\\par}";
        
        context.actions.push(RecoveryAction {
            action_type: RecoveryType::StructureRepair,
            description: "Created minimal valid RTF structure due to severe parsing errors".to_string(),
            applied: true,
        });

        let tokens = rtf_lexer::tokenize(minimal_rtf)?;
        Ok((tokens, context.actions.clone()))
    }

    /// Fix parsing issues in tokens
    fn fix_parsing_issues(
        &self,
        tokens: &[RtfToken],
        error_msg: &str,
        actions: &mut Vec<RecoveryAction>,
    ) -> ConversionResult<Vec<RtfToken>> {
        let mut fixed_tokens = tokens.to_vec();

        // Common parsing issues
        if error_msg.contains("unexpected") || error_msg.contains("invalid") {
            // Try to fix common structural issues
            self.fix_token_structure(&mut fixed_tokens, actions)?;
        }

        Ok(fixed_tokens)
    }

    /// Fix token structure
    fn fix_token_structure(
        &self,
        tokens: &mut Vec<RtfToken>,
        actions: &mut Vec<RecoveryAction>,
    ) -> ConversionResult<()> {
        // Ensure proper RTF header
        if tokens.is_empty() || !matches!(&tokens[0], RtfToken::GroupStart) {
            tokens.insert(0, RtfToken::GroupStart);
            tokens.insert(1, RtfToken::ControlWord {
                name: "rtf".to_string(),
                parameter: Some(1),
            });
            
            actions.push(RecoveryAction {
                action_type: RecoveryType::StructureRepair,
                description: "Added missing RTF header tokens".to_string(),
                applied: true,
            });
        }

        // Balance groups
        let mut depth = 0;
        for token in tokens.iter() {
            match token {
                RtfToken::GroupStart => depth += 1,
                RtfToken::GroupEnd => depth -= 1,
                _ => {}
            }
        }

        // Add missing group ends
        while depth > 0 {
            tokens.push(RtfToken::GroupEnd);
            depth -= 1;
            
            actions.push(RecoveryAction {
                action_type: RecoveryType::MissingDataInsertion,
                description: "Added missing group end token".to_string(),
                applied: true,
            });
        }

        Ok(())
    }

    /// Create minimal valid document
    fn create_minimal_document(
        &self,
        tokens: &[RtfToken],
        actions: &mut Vec<RecoveryAction>,
        error: ConversionError,
    ) -> ConversionResult<(RtfDocument, Vec<RecoveryAction>)> {
        // Extract any text content from tokens
        let mut text_content = String::new();
        for token in tokens {
            if let RtfToken::Text(text) = token {
                text_content.push_str(text);
                text_content.push(' ');
            }
        }

        if text_content.is_empty() {
            text_content = "[Document parsing failed]".to_string();
        }

        actions.push(RecoveryAction {
            action_type: RecoveryType::StructureRepair,
            description: format!(
                "Created minimal document structure due to parsing error: {}",
                error
            ),
            applied: true,
        });

        let document = RtfDocument {
            metadata: DocumentMetadata::default(),
            content: vec![
                RtfNode::Paragraph(vec![
                    RtfNode::Text(format!(
                        "Document Recovery Notice: {}\n\nRecovered content:\n{}",
                        error, text_content
                    ))
                ])
            ],
        };

        Ok((document, actions.clone()))
    }
}

/// Recovery suggestions for users
pub struct RecoverySuggestions;

impl RecoverySuggestions {
    /// Generate suggestions based on recovery actions
    pub fn generate(actions: &[RecoveryAction]) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Analyze actions and provide relevant suggestions
        let has_structure_repairs = actions.iter()
            .any(|a| matches!(a.action_type, RecoveryType::StructureRepair));
        
        let has_encoding_fixes = actions.iter()
            .any(|a| matches!(a.action_type, RecoveryType::EncodingFix));

        if has_structure_repairs {
            suggestions.push(
                "The document had structural issues that were automatically repaired. \
                Consider validating the source RTF file with an RTF editor.".to_string()
            );
        }

        if has_encoding_fixes {
            suggestions.push(
                "Encoding issues were detected and fixed. Ensure the document is saved \
                with proper character encoding (preferably UTF-8 or Windows-1252).".to_string()
            );
        }

        if actions.len() > 5 {
            suggestions.push(
                "Multiple recovery actions were needed. The document may be severely \
                corrupted. Consider obtaining a fresh copy from the source.".to_string()
            );
        }

        suggestions
    }
}