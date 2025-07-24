// Pooled RTF Lexer - Tokenizes RTF content using memory pools

use super::types::{ConversionError, ConversionResult, RtfToken};
use super::memory_pools::{CONVERSION_POOLS, PooledStringBuilder};

/// Tokenize RTF content into tokens using memory pools
pub fn tokenize_pooled(input: &str) -> ConversionResult<Vec<RtfToken>> {
    let mut lexer = PooledRtfLexer::new(input);
    lexer.tokenize()
}

struct PooledRtfLexer<'a> {
    input: &'a str,
    position: usize,
    // Pre-allocate token vector from pool
    tokens: crate::conversion::memory_pools::PooledObject<Vec<RtfToken>>,
}

impl<'a> PooledRtfLexer<'a> {
    fn new(input: &'a str) -> Self {
        let mut tokens = CONVERSION_POOLS.get_token_buffer();
        tokens.clear();
        tokens.reserve(input.len() / 10); // Estimate ~10 chars per token
        
        Self {
            input,
            position: 0,
            tokens,
        }
    }

    fn tokenize(mut self) -> ConversionResult<Vec<RtfToken>> {
        while self.position < self.input.len() {
            match self.current_char() {
                Some('{') => {
                    self.tokens.push(RtfToken::GroupStart);
                    self.advance();
                }
                Some('}') => {
                    self.tokens.push(RtfToken::GroupEnd);
                    self.advance();
                }
                Some('\\') => {
                    self.advance();
                    self.parse_control()?;
                }
                Some(_) => {
                    self.parse_text()?;
                }
                None => break,
            }
        }

        // Extract tokens from pooled object
        Ok(std::mem::take(&mut *self.tokens))
    }

    fn parse_control(&mut self) -> ConversionResult<()> {
        match self.current_char() {
            Some(ch) if ch.is_alphabetic() => self.parse_control_word(),
            Some(ch) if ch == '\'' => self.parse_hex_value(),
            Some(ch) => {
                // Control symbol
                self.tokens.push(RtfToken::ControlSymbol(ch));
                self.advance();
                Ok(())
            }
            None => Err(ConversionError::LexerError(
                "Unexpected end after backslash".to_string(),
            )),
        }
    }

    fn parse_control_word(&mut self) -> ConversionResult<()> {
        // Use pooled string for control word name
        let mut name = CONVERSION_POOLS.get_string_buffer(16);
        name.clear();
        
        // Read alphabetic characters
        while let Some(ch) = self.current_char() {
            if ch.is_alphabetic() {
                name.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Parse optional numeric parameter
        let parameter = self.parse_parameter();

        // For common control words, use interned strings to save allocations
        let name_str = match name.as_str() {
            // Common control words - intern these
            "par" | "b" | "i" | "ul" | "f" | "fs" | "cf" | "rtf" | "ansi" | "deff" |
            "fonttbl" | "colortbl" | "pard" | "plain" | "line" | "tab" => {
                name.as_str().to_string() // These will be optimized by compiler
            }
            // Less common - take ownership of pooled string
            _ => std::mem::take(&mut *name),
        };

        self.tokens.push(RtfToken::ControlWord {
            name: name_str,
            parameter,
        });

        Ok(())
    }

    fn parse_parameter(&mut self) -> Option<i32> {
        let mut has_sign = false;
        let mut is_negative = false;

        // Check for sign
        if let Some('-') = self.current_char() {
            has_sign = true;
            is_negative = true;
            self.advance();
        }

        // Use small string from pool for number parsing
        let mut num_str = CONVERSION_POOLS.get_string_buffer(8);
        num_str.clear();
        
        let mut has_digits = false;

        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                has_digits = true;
                self.advance();
            } else {
                break;
            }
        }

        if !has_digits && has_sign {
            // Just a sign, not a parameter
            return None;
        }

        if has_digits {
            num_str.parse::<i32>().ok().map(|n| if is_negative { -n } else { n })
        } else {
            None
        }
    }

    fn parse_hex_value(&mut self) -> ConversionResult<()> {
        self.advance(); // Skip '
        
        let hex1 = self.current_char()
            .ok_or_else(|| ConversionError::LexerError("Expected hex digit".to_string()))?;
        self.advance();
        
        let hex2 = self.current_char()
            .ok_or_else(|| ConversionError::LexerError("Expected hex digit".to_string()))?;
        self.advance();

        let value = u8::from_str_radix(&format!("{}{}", hex1, hex2), 16)
            .map_err(|_| ConversionError::LexerError("Invalid hex value".to_string()))?;

        self.tokens.push(RtfToken::HexValue(value));
        Ok(())
    }

    fn parse_text(&mut self) -> ConversionResult<()> {
        // Use pooled string builder for text accumulation
        let mut text = PooledStringBuilder::new();
        
        while let Some(ch) = self.current_char() {
            match ch {
                '{' | '}' | '\\' => break,
                '\r' | '\n' => {
                    // Skip newlines
                    self.advance();
                }
                _ => {
                    text.push(ch);
                    self.advance();
                }
            }
        }

        if !text.is_empty() {
            self.tokens.push(RtfToken::Text(text.finish()));
        }

        Ok(())
    }

    fn current_char(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    fn advance(&mut self) {
        if self.position < self.input.len() {
            self.position += self.current_char().map(|c| c.len_utf8()).unwrap_or(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pooled_tokenize_simple() {
        let input = r"{\rtf1 Hello World\par}";
        let tokens = tokenize_pooled(input).unwrap();
        
        assert_eq!(tokens.len(), 7);
        assert!(matches!(tokens[0], RtfToken::GroupStart));
        assert!(matches!(tokens[1], RtfToken::ControlWord { ref name, .. } if name == "rtf"));
    }

    #[test]
    fn test_pooled_tokenize_with_formatting() {
        let input = r"{\b Bold} {\i Italic}";
        let tokens = tokenize_pooled(input).unwrap();
        
        assert!(tokens.iter().any(|t| matches!(t, RtfToken::ControlWord { name, .. } if name == "b")));
        assert!(tokens.iter().any(|t| matches!(t, RtfToken::ControlWord { name, .. } if name == "i")));
    }
    
    #[test]
    fn test_pool_efficiency() {
        // Get initial pool stats
        let initial_stats = CONVERSION_POOLS.get_stats();
        
        // Tokenize multiple documents
        for i in 0..10 {
            let input = format!(r"{{\rtf1 Document {} with \b bold and \i italic text\par}}", i);
            let _ = tokenize_pooled(&input).unwrap();
        }
        
        // Check pool usage
        let final_stats = CONVERSION_POOLS.get_stats();
        assert!(final_stats.token_buffer_pool_size >= initial_stats.token_buffer_pool_size);
        assert!(final_stats.string_pool_size >= initial_stats.string_pool_size);
    }
}