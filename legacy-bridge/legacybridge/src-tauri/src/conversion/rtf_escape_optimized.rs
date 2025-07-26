// Optimized RTF text escaping using Cow<str> for zero-copy operations
use std::borrow::Cow;

/// Escape special RTF characters using Cow<str> for zero-copy when possible
pub fn escape_rtf_text_optimized(text: &str) -> Cow<str> {
    // Fast path: check if escaping is needed at all
    let needs_escape = text.chars().any(|ch| matches!(ch, '\\' | '{' | '}' | '\n' | '\r') || ch as u32 > 127);
    
    if !needs_escape {
        // Zero-copy: return borrowed string when no escaping needed
        return Cow::Borrowed(text);
    }
    
    // Slow path: only allocate when necessary
    let mut result = String::with_capacity(text.len() + (text.len() / 4)); // Estimate 25% overhead
    
    for ch in text.chars() {
        match ch {
            '\\' => result.push_str("\\\\"),
            '{' => result.push_str("\\{"),
            '}' => result.push_str("\\}"),
            '\n' => result.push_str("\\par "),
            '\r' => {}, // Skip carriage returns
            c if c as u32 > 127 => {
                // Unicode characters - write! to String should never fail
                use std::fmt::Write;
                let _ = write!(&mut result, "\\u{}?", c as u32);
            }
            c => result.push(c),
        }
    }
    
    Cow::Owned(result)
}

/// Optimized RTF text escaping with pre-allocated buffer for reuse
pub struct RtfEscaper {
    buffer: String,
}

impl RtfEscaper {
    pub fn new() -> Self {
        Self {
            buffer: String::with_capacity(1024),
        }
    }
    
    /// Escape text reusing internal buffer to reduce allocations
    pub fn escape<'a>(&mut self, text: &'a str) -> Cow<'a, str> {
        // Fast path: check if escaping is needed at all
        let needs_escape = text.chars().any(|ch| matches!(ch, '\\' | '{' | '}' | '\n' | '\r') || ch as u32 > 127);
        
        if !needs_escape {
            // Zero-copy: return borrowed string when no escaping needed
            return Cow::Borrowed(text);
        }
        
        // Clear and reuse buffer
        self.buffer.clear();
        self.buffer.reserve(text.len() + (text.len() / 4)); // Estimate 25% overhead
        
        for ch in text.chars() {
            match ch {
                '\\' => self.buffer.push_str("\\\\"),
                '{' => self.buffer.push_str("\\{"),
                '}' => self.buffer.push_str("\\}"),
                '\n' => self.buffer.push_str("\\par "),
                '\r' => {}, // Skip carriage returns
                c if c as u32 > 127 => {
                    // Unicode characters - write! to String should never fail
                    use std::fmt::Write;
                    let _ = write!(&mut self.buffer, "\\u{}?", c as u32);
                }
                c => self.buffer.push(c),
            }
        }
        
        // Return owned string from buffer
        Cow::Owned(self.buffer.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_no_escape_needed() {
        let text = "Simple text without special characters";
        let escaped = escape_rtf_text_optimized(text);
        
        // Should return borrowed reference (zero-copy)
        assert!(matches!(escaped, Cow::Borrowed(_)));
        assert_eq!(&*escaped, text);
    }
    
    #[test]
    fn test_escape_special_chars() {
        let text = "Text with {braces} and \\backslash";
        let escaped = escape_rtf_text_optimized(text);
        
        // Should return owned string with escapes
        assert!(matches!(escaped, Cow::Owned(_)));
        assert_eq!(&*escaped, "Text with \\{braces\\} and \\\\backslash");
    }
    
    #[test]
    fn test_escape_newlines() {
        let text = "Line 1\nLine 2\rLine 3";
        let escaped = escape_rtf_text_optimized(text);
        
        assert!(matches!(escaped, Cow::Owned(_)));
        assert_eq!(&*escaped, "Line 1\\par Line 2Line 3");
    }
    
    #[test]
    fn test_escape_unicode() {
        let text = "Text with émojis 🎉";
        let escaped = escape_rtf_text_optimized(text);
        
        assert!(matches!(escaped, Cow::Owned(_)));
        assert!(escaped.contains("\\u233?"));
        assert!(escaped.contains("\\u127881?"));
    }
    
    #[test]
    fn test_escaper_reuse() {
        let mut escaper = RtfEscaper::new();
        
        // Test multiple uses
        let text1 = "Simple text";
        let escaped1 = escaper.escape(text1);
        assert!(matches!(escaped1, Cow::Borrowed(_)));
        
        let text2 = "Text with {special} chars";
        let escaped2 = escaper.escape(text2);
        assert!(matches!(escaped2, Cow::Owned(_)));
        assert_eq!(&*escaped2, "Text with \\{special\\} chars");
    }
}