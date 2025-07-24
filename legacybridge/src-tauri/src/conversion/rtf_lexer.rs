// RTF Lexer - Tokenizes RTF input into a stream of tokens

use super::types::{ConversionError, ConversionResult, RtfToken};
use std::sync::atomic::{AtomicUsize, Ordering};

// SECURITY: Constants for safe parsing
const MAX_INPUT_SIZE: usize = 50 * 1024 * 1024; // 50MB max input
const MAX_TOKEN_COUNT: usize = 1_000_000; // Prevent token explosion
const MAX_CONTROL_WORD_LENGTH: usize = 32; // RTF spec limit

/// Tokenize RTF content into a vector of tokens
pub fn tokenize(input: &str) -> ConversionResult<Vec<RtfToken>> {
    // SECURITY: Validate input size first
    if input.len() > MAX_INPUT_SIZE {
        return Err(ConversionError::ValidationError(
            format!("Input size ({} bytes) exceeds maximum allowed ({} bytes)",
                input.len(), MAX_INPUT_SIZE)
        ));
    }
    
    let mut lexer = RtfLexer::new(input);
    lexer.tokenize()
}

/// RTF Lexer state
struct RtfLexer<'a> {
    input: &'a str,
    position: usize,
    current_char: Option<char>,
}

impl<'a> RtfLexer<'a> {
    /// Create a new lexer
    fn new(input: &'a str) -> Self {
        let mut lexer = RtfLexer {
            input,
            position: 0,
            current_char: None,
        };
        lexer.current_char = lexer.input.chars().next();
        lexer
    }

    /// Tokenize the entire input
    fn tokenize(&mut self) -> ConversionResult<Vec<RtfToken>> {
        let mut tokens = Vec::new();
        
        // SECURITY: Pre-allocate reasonable capacity
        tokens.reserve(std::cmp::min(self.input.len() / 4, 10000));

        while let Some(ch) = self.current_char {
            // SECURITY: Check token count to prevent memory exhaustion
            if tokens.len() >= MAX_TOKEN_COUNT {
                return Err(ConversionError::ValidationError(
                    format!("Token count exceeds maximum allowed ({})", MAX_TOKEN_COUNT)
                ));
            }
            
            match ch {
                '{' => {
                    tokens.push(RtfToken::GroupStart);
                    self.advance();
                }
                '}' => {
                    tokens.push(RtfToken::GroupEnd);
                    self.advance();
                }
                '\\' => {
                    self.advance();
                    tokens.push(self.read_control()?);
                }
                '\n' | '\r' => {
                    // Skip newlines
                    self.advance();
                }
                _ => {
                    tokens.push(self.read_text()?);
                }
            }
        }

        Ok(tokens)
    }

    /// Read a control word or symbol
    fn read_control(&mut self) -> ConversionResult<RtfToken> {
        match self.current_char {
            Some(ch) if ch.is_alphabetic() => self.read_control_word(),
            Some(ch) if ch == '\'' => self.read_hex_value(),
            Some(ch) => {
                // Control symbol
                let symbol = ch;
                self.advance();
                Ok(RtfToken::ControlSymbol(symbol))
            }
            None => Err(ConversionError::LexerError(
                "Unexpected end of input after backslash".to_string(),
            )),
        }
    }

    /// Read a control word
    fn read_control_word(&mut self) -> ConversionResult<RtfToken> {
        let mut name = String::new();
        name.reserve(16); // Common control word size

        // Read alphabetic characters
        while let Some(ch) = self.current_char {
            if ch.is_alphabetic() {
                // SECURITY: Limit control word length
                if name.len() >= MAX_CONTROL_WORD_LENGTH {
                    return Err(ConversionError::LexerError(
                        format!("Control word too long (max {} characters)", MAX_CONTROL_WORD_LENGTH)
                    ));
                }
                name.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Read optional numeric parameter
        let parameter = if let Some(ch) = self.current_char {
            if ch == '-' || ch.is_numeric() {
                self.read_number()?
            } else {
                None
            }
        } else {
            None
        };

        // Skip optional space after control word
        if self.current_char == Some(' ') {
            self.advance();
        }

        Ok(RtfToken::ControlWord { name, parameter })
    }

    /// Read a numeric parameter
    fn read_number(&mut self) -> ConversionResult<Option<i32>> {
        let mut number = String::new();
        number.reserve(11); // Max i32 is 10 digits + sign

        // Handle negative sign
        let is_negative = if self.current_char == Some('-') {
            number.push('-');
            self.advance();
            true
        } else {
            false
        };

        // SECURITY: Use checked arithmetic throughout
        let mut value: i32 = 0;
        let mut digit_count = 0;
        
        while let Some(ch) = self.current_char {
            if let Some(digit) = ch.to_digit(10) {
                digit_count += 1;
                
                // SECURITY: Prevent excessive digits
                if digit_count > 10 {
                    return Err(ConversionError::LexerError(
                        "Number has too many digits".to_string()
                    ));
                }
                
                // SECURITY: Use checked multiplication and addition
                match value.checked_mul(10).and_then(|v| {
                    if is_negative {
                        v.checked_sub(digit as i32)
                    } else {
                        v.checked_add(digit as i32)
                    }
                }) {
                    Some(new_value) => {
                        // SECURITY: Additional range check
                        if new_value < -1_000_000 || new_value > 1_000_000 {
                            return Err(ConversionError::LexerError(
                                format!("Number outside allowed range [-1000000, 1000000]")
                            ));
                        }
                        value = new_value;
                    }
                    None => {
                        return Err(ConversionError::LexerError(
                            "Integer overflow in numeric parameter".to_string()
                        ));
                    }
                }
                
                self.advance();
            } else {
                break;
            }
        }

        if digit_count == 0 {
            return Ok(None);
        }

        Ok(Some(value))
    }

    /// Read a hexadecimal value
    fn read_hex_value(&mut self) -> ConversionResult<RtfToken> {
        self.advance(); // Skip the apostrophe

        // SECURITY: Read hex digits with checked arithmetic
        let mut value: u8 = 0;
        
        // Read exactly 2 hex digits
        for i in 0..2 {
            match self.current_char {
                Some(ch) if ch.is_ascii_hexdigit() => {
                    let digit_value = ch.to_digit(16)
                        .ok_or_else(|| ConversionError::LexerError(
                            format!("Invalid hex digit: {}", ch)
                        ))? as u8;
                    
                    // SECURITY: Use checked arithmetic
                    match value.checked_mul(16).and_then(|v| v.checked_add(digit_value)) {
                        Some(new_value) => value = new_value,
                        None => {
                            return Err(ConversionError::LexerError(
                                "Hex value overflow".to_string()
                            ));
                        }
                    }
                    
                    self.advance();
                }
                _ => {
                    return Err(ConversionError::LexerError(
                        format!("Expected hex digit after \\', got {:?}", self.current_char)
                    ));
                }
            }
        }

        Ok(RtfToken::HexValue(value))
    }

    /// Read plain text
    fn read_text(&mut self) -> ConversionResult<RtfToken> {
        const MAX_TEXT_SIZE: usize = 1_000_000; // 1MB limit
        let mut text = String::new();

        while let Some(ch) = self.current_char {
            // Security: Prevent unbounded string growth
            if text.len() >= MAX_TEXT_SIZE {
                return Err(ConversionError::LexerError(
                    "Text size exceeds maximum allowed (1MB)".to_string()
                ));
            }
            
            match ch {
                '{' | '}' | '\\' => break,
                '\n' | '\r' => {
                    // Include space for line breaks in text
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    self.advance();
                }
                _ => {
                    text.push(ch);
                    self.advance();
                }
            }
        }

        Ok(RtfToken::Text(text))
    }

    /// Advance to the next character
    fn advance(&mut self) {
        // SECURITY: Use checked addition to prevent overflow
        match self.position.checked_add(1) {
            Some(new_pos) if new_pos <= self.input.len() => {
                self.position = new_pos;
                self.current_char = self.input.chars().nth(self.position);
            }
            _ => {
                self.position = self.input.len();
                self.current_char = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_text() {
        let tokens = tokenize("Hello World").expect("tokenization should succeed");
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            RtfToken::Text(text) => assert_eq!(text, "Hello World"),
            _ => panic!("Expected text token"),
        }
    }

    #[test]
    fn test_tokenize_groups() {
        let tokens = tokenize("{Hello}").expect("tokenization should succeed");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], RtfToken::GroupStart));
        assert!(matches!(tokens[1], RtfToken::Text(_)));
        assert!(matches!(tokens[2], RtfToken::GroupEnd));
    }

    #[test]
    fn test_tokenize_control_word() {
        let tokens = tokenize(r"\rtf1").expect("tokenization should succeed");
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            RtfToken::ControlWord { name, parameter } => {
                assert_eq!(name, "rtf");
                assert_eq!(*parameter, Some(1));
            }
            _ => panic!("Expected control word"),
        }
    }

    #[test]
    fn test_tokenize_control_symbol() {
        let tokens = tokenize(r"\*").expect("tokenization should succeed");
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            RtfToken::ControlSymbol(ch) => assert_eq!(*ch, '*'),
            _ => panic!("Expected control symbol"),
        }
    }

    #[test]
    fn test_tokenize_hex_value() {
        let tokens = tokenize(r"\'4a").expect("tokenization should succeed");
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            RtfToken::HexValue(value) => assert_eq!(*value, 0x4a),
            _ => panic!("Expected hex value"),
        }
    }
}