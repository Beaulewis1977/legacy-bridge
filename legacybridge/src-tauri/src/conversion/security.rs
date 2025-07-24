// Security configuration and constants for RTF conversion
//
// This module defines security limits and configurations to prevent
// various attack vectors including resource exhaustion, buffer overflows,
// and malicious content injection.

use std::collections::HashSet;

/// Security limits for RTF parsing and generation
pub struct SecurityLimits {
    /// Maximum file size in bytes (default: 10MB)
    pub max_file_size: usize,
    /// Maximum text chunk size in bytes (default: 1MB)
    pub max_text_size: usize,
    /// Maximum recursion/nesting depth (default: 50)
    pub max_nesting_depth: usize,
    /// Maximum numeric parameter value
    pub max_number_value: i32,
    /// Minimum numeric parameter value
    pub min_number_value: i32,
    /// Maximum control word length
    pub max_control_word_length: usize,
    /// Maximum number length
    pub max_number_length: usize,
    /// Maximum table rows
    pub max_table_rows: usize,
    /// Maximum table columns
    pub max_table_columns: usize,
    /// Maximum total table cells
    pub max_table_cells: usize,
    /// Parsing timeout in seconds
    pub parsing_timeout_secs: u64,
    /// Memory limit per conversion in bytes
    pub max_memory_usage: usize,
}

impl Default for SecurityLimits {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024,        // 10MB
            max_text_size: 1 * 1024 * 1024,         // 1MB
            max_nesting_depth: 50,
            max_number_value: 1_000_000,
            min_number_value: -1_000_000,
            max_control_word_length: 32,            // RTF spec limit
            max_number_length: 10,
            max_table_rows: 1000,
            max_table_columns: 100,
            max_table_cells: 10000,
            parsing_timeout_secs: 30,
            max_memory_usage: 100 * 1024 * 1024,    // 100MB
        }
    }
}

/// Control word security configuration
pub struct ControlWordSecurity {
    /// Allowed control words (if Some, only these are allowed)
    pub allowed_control_words: Option<HashSet<String>>,
    /// Explicitly forbidden control words
    pub forbidden_control_words: HashSet<String>,
}

impl Default for ControlWordSecurity {
    fn default() -> Self {
        let mut forbidden = HashSet::new();
        
        // Dangerous control words that could embed objects or execute code
        forbidden.insert("object".to_string());
        forbidden.insert("objdata".to_string());
        forbidden.insert("objemb".to_string());
        forbidden.insert("objlink".to_string());
        forbidden.insert("objautlink".to_string());
        forbidden.insert("objsub".to_string());
        forbidden.insert("objpub".to_string());
        forbidden.insert("objicemb".to_string());
        forbidden.insert("objhtml".to_string());
        forbidden.insert("objocx".to_string());
        forbidden.insert("datafield".to_string());
        forbidden.insert("datastore".to_string());
        forbidden.insert("result".to_string());
        forbidden.insert("xe".to_string());         // Index entries
        forbidden.insert("tc".to_string());         // Table of contents entries
        forbidden.insert("bkmkstart".to_string());  // Bookmarks
        forbidden.insert("bkmkend".to_string());
        forbidden.insert("field".to_string());      // Fields that could contain macros
        forbidden.insert("fldinst".to_string());
        forbidden.insert("fldrslt".to_string());
        
        // ActiveX and OLE related
        forbidden.insert("shppict".to_string());
        forbidden.insert("nonshppict".to_string());
        
        Self {
            allowed_control_words: None,
            forbidden_control_words: forbidden,
        }
    }
}

impl ControlWordSecurity {
    /// Get a whitelist configuration (more restrictive)
    pub fn whitelist() -> Self {
        let mut allowed = HashSet::new();
        
        // Basic document structure
        allowed.insert("rtf".to_string());
        allowed.insert("ansi".to_string());
        allowed.insert("deff".to_string());
        allowed.insert("deflang".to_string());
        
        // Font table
        allowed.insert("fonttbl".to_string());
        allowed.insert("f".to_string());
        allowed.insert("froman".to_string());
        allowed.insert("fswiss".to_string());
        allowed.insert("fmodern".to_string());
        allowed.insert("fscript".to_string());
        allowed.insert("fdecor".to_string());
        allowed.insert("ftech".to_string());
        allowed.insert("fbidi".to_string());
        allowed.insert("fcharset".to_string());
        allowed.insert("fprq".to_string());
        
        // Color table
        allowed.insert("colortbl".to_string());
        allowed.insert("red".to_string());
        allowed.insert("green".to_string());
        allowed.insert("blue".to_string());
        
        // Paragraph formatting
        allowed.insert("par".to_string());
        allowed.insert("pard".to_string());
        allowed.insert("ql".to_string());
        allowed.insert("qr".to_string());
        allowed.insert("qc".to_string());
        allowed.insert("qj".to_string());
        allowed.insert("li".to_string());
        allowed.insert("ri".to_string());
        allowed.insert("fi".to_string());
        allowed.insert("sa".to_string());
        allowed.insert("sb".to_string());
        allowed.insert("sl".to_string());
        allowed.insert("slmult".to_string());
        
        // Character formatting
        allowed.insert("b".to_string());
        allowed.insert("i".to_string());
        allowed.insert("ul".to_string());
        allowed.insert("ulnone".to_string());
        allowed.insert("strike".to_string());
        allowed.insert("plain".to_string());
        allowed.insert("fs".to_string());
        allowed.insert("cf".to_string());
        allowed.insert("cb".to_string());
        allowed.insert("highlight".to_string());
        
        // Special characters
        allowed.insert("line".to_string());
        allowed.insert("page".to_string());
        allowed.insert("bullet".to_string());
        allowed.insert("tab".to_string());
        allowed.insert("emdash".to_string());
        allowed.insert("endash".to_string());
        allowed.insert("lquote".to_string());
        allowed.insert("rquote".to_string());
        allowed.insert("ldblquote".to_string());
        allowed.insert("rdblquote".to_string());
        
        // Tables
        allowed.insert("trowd".to_string());
        allowed.insert("trgaph".to_string());
        allowed.insert("trleft".to_string());
        allowed.insert("cellx".to_string());
        allowed.insert("intbl".to_string());
        allowed.insert("cell".to_string());
        allowed.insert("row".to_string());
        
        // Document info (safe subset)
        allowed.insert("info".to_string());
        allowed.insert("title".to_string());
        allowed.insert("author".to_string());
        allowed.insert("creatim".to_string());
        allowed.insert("revtim".to_string());
        
        // Page setup
        allowed.insert("paperw".to_string());
        allowed.insert("paperh".to_string());
        allowed.insert("margl".to_string());
        allowed.insert("margr".to_string());
        allowed.insert("margt".to_string());
        allowed.insert("margb".to_string());
        
        Self {
            allowed_control_words: Some(allowed),
            forbidden_control_words: HashSet::new(),
        }
    }
}

/// Check if a control word is safe to process
pub fn is_control_word_safe(word: &str, config: &ControlWordSecurity) -> bool {
    // First check if it's explicitly forbidden
    if config.forbidden_control_words.contains(word) {
        return false;
    }
    
    // If we have a whitelist, check if it's allowed
    if let Some(ref allowed) = config.allowed_control_words {
        return allowed.contains(word);
    }
    
    // No whitelist means allow by default (unless forbidden)
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limits() {
        let limits = SecurityLimits::default();
        assert_eq!(limits.max_file_size, 10 * 1024 * 1024);
        assert_eq!(limits.max_nesting_depth, 50);
    }

    #[test]
    fn test_forbidden_control_words() {
        let config = ControlWordSecurity::default();
        assert!(config.forbidden_control_words.contains("object"));
        assert!(config.forbidden_control_words.contains("objdata"));
        assert!(!config.forbidden_control_words.contains("rtf"));
    }

    #[test]
    fn test_control_word_safety() {
        let config = ControlWordSecurity::default();
        assert!(!is_control_word_safe("object", &config));
        assert!(is_control_word_safe("rtf", &config));
        assert!(is_control_word_safe("par", &config));
        
        let whitelist_config = ControlWordSecurity::whitelist();
        assert!(is_control_word_safe("rtf", &whitelist_config));
        assert!(is_control_word_safe("par", &whitelist_config));
        assert!(!is_control_word_safe("unknown", &whitelist_config));
    }
}