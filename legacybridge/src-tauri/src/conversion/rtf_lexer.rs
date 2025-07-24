// RTF Lexer - Tokenizes RTF input into a stream of tokens

use super::types::{ConversionError, ConversionResult, RtfToken};

/// Tokenize RTF content into a vector of tokens
pub fn tokenize(input: &str) -> ConversionResult<Vec<RtfToken>> {
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

        while let Some(ch) = self.current_char {
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

        // Read alphabetic characters
        while let Some(ch) = self.current_char {
            if ch.is_alphabetic() {
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

        // Handle negative sign
        if self.current_char == Some('-') {
            number.push('-');
            self.advance();
        }

        // Read digits with length limit to prevent overflow
        const MAX_DIGITS: usize = 10; // Enough for i32 range
        while let Some(ch) = self.current_char {
            if ch.is_numeric() {
                if number.len() >= MAX_DIGITS + 1 { // +1 for possible negative sign
                    return Err(ConversionError::LexerError(
                        format!("Number too large: {}", number)
                    ));
                }
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if number.is_empty() || number == "-" {
            return Ok(None);
        }

        // SECURITY: Parse with bounds checking
        match number.parse::<i32>() {
            Ok(n) => {
                // Additional bounds check for security (-1M to +1M)
                if n < -1_000_000 || n > 1_000_000 {
                    return Err(ConversionError::LexerError(
                        format!("Number {} outside allowed range [-1000000, 1000000]", n)
                    ));
                }
                Ok(Some(n))
            }
            Err(_) => Err(ConversionError::LexerError(
                format!("Invalid number: {}", number)
            ))
        }
    }

    /// Read a hexadecimal value
    fn read_hex_value(&mut self) -> ConversionResult<RtfToken> {
        self.advance(); // Skip the apostrophe

        let mut hex = String::new();
        
        // Read exactly 2 hex digits
        for _ in 0..2 {
            match self.current_char {
                Some(ch) if ch.is_ascii_hexdigit() => {
                    hex.push(ch);
                    self.advance();
                }
                _ => {
                    return Err(ConversionError::LexerError(
                        "Expected hex digit after \\'".to_string(),
                    ));
                }
            }
        }

        let value = u8::from_str_radix(&hex, 16)
            .map_err(|_| ConversionError::LexerError(format!("Invalid hex value: {}", hex)))?;

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
        self.position += 1;
        self.current_char = self.input.chars().nth(self.position);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_text() {
        let tokens = tokenize("Hello World").unwrap();
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            RtfToken::Text(text) => assert_eq!(text, "Hello World"),
            _ => panic!("Expected text token"),
        }
    }

    #[test]
    fn test_tokenize_groups() {
        let tokens = tokenize("{Hello}").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], RtfToken::GroupStart));
        assert!(matches!(tokens[1], RtfToken::Text(_)));
        assert!(matches!(tokens[2], RtfToken::GroupEnd));
    }

    #[test]
    fn test_tokenize_control_word() {
        let tokens = tokenize(r"\rtf1").unwrap();
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
        let tokens = tokenize(r"\*").unwrap();
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            RtfToken::ControlSymbol(ch) => assert_eq!(*ch, '*'),
            _ => panic!("Expected control symbol"),
        }
    }

    #[test]
    fn test_tokenize_hex_value() {
        let tokens = tokenize(r"\'4a").unwrap();
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            RtfToken::HexValue(value) => assert_eq!(*value, 0x4a),
            _ => panic!("Expected hex value"),
        }
    }
}