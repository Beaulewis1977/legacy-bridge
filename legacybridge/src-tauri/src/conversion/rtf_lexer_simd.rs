// SIMD-Optimized RTF Lexer
// High-performance RTF tokenization using SIMD instructions

use super::types::{ConversionError, ConversionResult, RtfToken};
use std::arch::x86_64::*;

// SECURITY: Constants for safe parsing
const MAX_INPUT_SIZE: usize = 50 * 1024 * 1024; // 50MB max input
const MAX_TOKEN_COUNT: usize = 1_000_000; // Prevent token explosion
const MAX_CONTROL_WORD_LENGTH: usize = 32; // RTF spec limit

/// CPU feature detection for runtime SIMD selection
pub struct CpuFeatures {
    pub has_sse2: bool,
    pub has_sse42: bool,
    pub has_avx2: bool,
}

impl CpuFeatures {
    pub fn detect() -> Self {
        Self {
            has_sse2: is_x86_feature_detected!("sse2"),
            has_sse42: is_x86_feature_detected!("sse4.2"),
            has_avx2: is_x86_feature_detected!("avx2"),
        }
    }
}

/// SIMD-optimized RTF tokenizer
pub fn tokenize_simd(input: &str) -> ConversionResult<Vec<RtfToken>> {
    // SECURITY: Validate input size first
    if input.len() > MAX_INPUT_SIZE {
        return Err(ConversionError::ValidationError(
            format!("Input size ({} bytes) exceeds maximum allowed ({} bytes)",
                input.len(), MAX_INPUT_SIZE)
        ));
    }
    
    let cpu = CpuFeatures::detect();
    
    if cpu.has_avx2 {
        unsafe { tokenize_avx2(input) }
    } else if cpu.has_sse42 {
        unsafe { tokenize_sse42(input) }
    } else {
        // Fallback to scalar implementation
        super::rtf_lexer::tokenize(input)
    }
}

/// AVX2-optimized tokenizer (32-byte vectors)
#[target_feature(enable = "avx2")]
unsafe fn tokenize_avx2(input: &str) -> ConversionResult<Vec<RtfToken>> {
    let mut lexer = SimdRtfLexerAvx2::new(input);
    lexer.tokenize()
}

/// SSE4.2-optimized tokenizer (16-byte vectors)
#[target_feature(enable = "sse4.2")]
unsafe fn tokenize_sse42(input: &str) -> ConversionResult<Vec<RtfToken>> {
    let mut lexer = SimdRtfLexerSse42::new(input);
    lexer.tokenize()
}

/// AVX2 RTF Lexer implementation
struct SimdRtfLexerAvx2 {
    input: Vec<u8>,
    position: usize,
    tokens: Vec<RtfToken>,
}

impl SimdRtfLexerAvx2 {
    fn new(input: &str) -> Self {
        Self {
            input: input.as_bytes().to_vec(),
            position: 0,
            tokens: Vec::with_capacity(input.len() / 8), // Estimate
        }
    }
    
    #[target_feature(enable = "avx2")]
    unsafe fn tokenize(&mut self) -> ConversionResult<Vec<RtfToken>> {
        while self.position < self.input.len() {
            // SECURITY: Check token count
            if self.tokens.len() >= MAX_TOKEN_COUNT {
                return Err(ConversionError::ValidationError(
                    format!("Token count exceeds maximum allowed ({})", MAX_TOKEN_COUNT)
                ));
            }
            
            // Find next control character using SIMD
            if let Some((pos, ch)) = self.find_next_control_avx2() {
                // Process text before control character
                if pos > self.position {
                    let text = self.extract_text(self.position, pos)?;
                    if !text.is_empty() {
                        self.tokens.push(RtfToken::Text(text));
                    }
                }
                
                self.position = pos + 1;
                
                // Process control character
                match ch {
                    b'{' => self.tokens.push(RtfToken::GroupStart),
                    b'}' => self.tokens.push(RtfToken::GroupEnd),
                    b'\\' => {
                        let token = self.read_control()?;
                        self.tokens.push(token);
                    }
                    _ => unreachable!(),
                }
            } else {
                // Process remaining text
                if self.position < self.input.len() {
                    let text = self.extract_text(self.position, self.input.len())?;
                    if !text.is_empty() {
                        self.tokens.push(RtfToken::Text(text));
                    }
                    break;
                }
            }
        }
        
        Ok(std::mem::take(&mut self.tokens))
    }
    
    #[target_feature(enable = "avx2")]
    unsafe fn find_next_control_avx2(&self) -> Option<(usize, u8)> {
        let mut pos = self.position;
        let len = self.input.len();
        
        // SIMD vectors for control characters
        let backslash_vec = _mm256_set1_epi8(b'\\' as i8);
        let open_brace_vec = _mm256_set1_epi8(b'{' as i8);
        let close_brace_vec = _mm256_set1_epi8(b'}' as i8);
        
        // Process 32 bytes at a time
        while pos + 32 <= len {
            let chunk = _mm256_loadu_si256(self.input[pos..].as_ptr() as *const __m256i);
            
            // Check for all control characters in parallel
            let backslash_mask = _mm256_cmpeq_epi8(chunk, backslash_vec);
            let open_mask = _mm256_cmpeq_epi8(chunk, open_brace_vec);
            let close_mask = _mm256_cmpeq_epi8(chunk, close_brace_vec);
            
            // Combine masks
            let combined = _mm256_or_si256(_mm256_or_si256(backslash_mask, open_mask), close_mask);
            let mask = _mm256_movemask_epi8(combined);
            
            if mask != 0 {
                // Found at least one control character
                let offset = mask.trailing_zeros() as usize;
                return Some((pos + offset, self.input[pos + offset]));
            }
            
            pos += 32;
        }
        
        // Process 16 bytes at a time for remainder
        if pos + 16 <= len {
            let chunk = _mm_loadu_si128(self.input[pos..].as_ptr() as *const __m128i);
            
            let backslash_mask = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'\\' as i8));
            let open_mask = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'{' as i8));
            let close_mask = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'}' as i8));
            
            let combined = _mm_or_si128(_mm_or_si128(backslash_mask, open_mask), close_mask);
            let mask = _mm_movemask_epi8(combined);
            
            if mask != 0 {
                let offset = mask.trailing_zeros() as usize;
                return Some((pos + offset, self.input[pos + offset]));
            }
            
            pos += 16;
        }
        
        // Scalar fallback for remaining bytes
        for i in pos..len {
            match self.input[i] {
                b'\\' | b'{' | b'}' => return Some((i, self.input[i])),
                _ => continue,
            }
        }
        
        None
    }
    
    fn extract_text(&self, start: usize, end: usize) -> ConversionResult<String> {
        if start >= end {
            return Ok(String::new());
        }
        
        let slice = &self.input[start..end];
        
        // Remove newlines and normalize whitespace
        let mut result = String::with_capacity(slice.len());
        let mut prev_space = false;
        
        for &byte in slice {
            match byte {
                b'\n' | b'\r' => {
                    if !prev_space && !result.is_empty() {
                        result.push(' ');
                        prev_space = true;
                    }
                }
                b' ' | b'\t' => {
                    if !prev_space {
                        result.push(' ');
                        prev_space = true;
                    }
                }
                _ => {
                    result.push(byte as char);
                    prev_space = false;
                }
            }
        }
        
        Ok(result)
    }
    
    fn read_control(&mut self) -> ConversionResult<RtfToken> {
        if self.position >= self.input.len() {
            return Err(ConversionError::LexerError(
                "Unexpected end of input after backslash".to_string(),
            ));
        }
        
        let ch = self.input[self.position];
        
        if ch.is_ascii_alphabetic() {
            self.read_control_word()
        } else if ch == b'\'' {
            self.read_hex_value()
        } else {
            // Control symbol
            self.position += 1;
            Ok(RtfToken::ControlSymbol(ch as char))
        }
    }
    
    fn read_control_word(&mut self) -> ConversionResult<RtfToken> {
        let start = self.position;
        
        // Find end of alphabetic sequence
        while self.position < self.input.len() && self.input[self.position].is_ascii_alphabetic() {
            if self.position - start >= MAX_CONTROL_WORD_LENGTH {
                return Err(ConversionError::LexerError(
                    format!("Control word too long (max {} characters)", MAX_CONTROL_WORD_LENGTH)
                ));
            }
            self.position += 1;
        }
        
        let name = String::from_utf8_lossy(&self.input[start..self.position]).into_owned();
        
        // Read optional numeric parameter
        let parameter = if self.position < self.input.len() {
            let ch = self.input[self.position];
            if ch == b'-' || ch.is_ascii_digit() {
                self.read_number()?
            } else {
                None
            }
        } else {
            None
        };
        
        // Skip optional space
        if self.position < self.input.len() && self.input[self.position] == b' ' {
            self.position += 1;
        }
        
        Ok(RtfToken::ControlWord { name, parameter })
    }
    
    fn read_number(&mut self) -> ConversionResult<Option<i32>> {
        let is_negative = if self.position < self.input.len() && self.input[self.position] == b'-' {
            self.position += 1;
            true
        } else {
            false
        };
        
        let start = self.position;
        while self.position < self.input.len() && self.input[self.position].is_ascii_digit() {
            if self.position - start > 10 {
                return Err(ConversionError::LexerError(
                    "Number has too many digits".to_string()
                ));
            }
            self.position += 1;
        }
        
        if self.position == start {
            return Ok(None);
        }
        
        let num_str = std::str::from_utf8(&self.input[start..self.position])
            .map_err(|_| ConversionError::LexerError("Invalid UTF-8 in number".to_string()))?;
        
        let value: i32 = num_str.parse()
            .map_err(|_| ConversionError::LexerError("Invalid number".to_string()))?;
        
        Ok(Some(if is_negative { -value } else { value }))
    }
    
    fn read_hex_value(&mut self) -> ConversionResult<RtfToken> {
        self.position += 1; // Skip apostrophe
        
        if self.position + 2 > self.input.len() {
            return Err(ConversionError::LexerError(
                "Expected two hex digits after \\'".to_string()
            ));
        }
        
        let hex_str = std::str::from_utf8(&self.input[self.position..self.position + 2])
            .map_err(|_| ConversionError::LexerError("Invalid UTF-8 in hex value".to_string()))?;
        
        let value = u8::from_str_radix(hex_str, 16)
            .map_err(|_| ConversionError::LexerError(format!("Invalid hex value: {}", hex_str)))?;
        
        self.position += 2;
        Ok(RtfToken::HexValue(value))
    }
}

/// SSE4.2 RTF Lexer implementation
struct SimdRtfLexerSse42 {
    input: Vec<u8>,
    position: usize,
    tokens: Vec<RtfToken>,
}

impl SimdRtfLexerSse42 {
    fn new(input: &str) -> Self {
        Self {
            input: input.as_bytes().to_vec(),
            position: 0,
            tokens: Vec::with_capacity(input.len() / 8),
        }
    }
    
    #[target_feature(enable = "sse4.2")]
    unsafe fn tokenize(&mut self) -> ConversionResult<Vec<RtfToken>> {
        // Similar to AVX2 but using SSE instructions
        while self.position < self.input.len() {
            if self.tokens.len() >= MAX_TOKEN_COUNT {
                return Err(ConversionError::ValidationError(
                    format!("Token count exceeds maximum allowed ({})", MAX_TOKEN_COUNT)
                ));
            }
            
            if let Some((pos, ch)) = self.find_next_control_sse42() {
                if pos > self.position {
                    let text = self.extract_text(self.position, pos)?;
                    if !text.is_empty() {
                        self.tokens.push(RtfToken::Text(text));
                    }
                }
                
                self.position = pos + 1;
                
                match ch {
                    b'{' => self.tokens.push(RtfToken::GroupStart),
                    b'}' => self.tokens.push(RtfToken::GroupEnd),
                    b'\\' => {
                        let token = self.read_control()?;
                        self.tokens.push(token);
                    }
                    _ => unreachable!(),
                }
            } else {
                if self.position < self.input.len() {
                    let text = self.extract_text(self.position, self.input.len())?;
                    if !text.is_empty() {
                        self.tokens.push(RtfToken::Text(text));
                    }
                    break;
                }
            }
        }
        
        Ok(std::mem::take(&mut self.tokens))
    }
    
    #[target_feature(enable = "sse4.2")]
    unsafe fn find_next_control_sse42(&self) -> Option<(usize, u8)> {
        let mut pos = self.position;
        let len = self.input.len();
        
        // SIMD vectors for control characters
        let backslash_vec = _mm_set1_epi8(b'\\' as i8);
        let open_brace_vec = _mm_set1_epi8(b'{' as i8);
        let close_brace_vec = _mm_set1_epi8(b'}' as i8);
        
        // Process 16 bytes at a time
        while pos + 16 <= len {
            let chunk = _mm_loadu_si128(self.input[pos..].as_ptr() as *const __m128i);
            
            let backslash_mask = _mm_cmpeq_epi8(chunk, backslash_vec);
            let open_mask = _mm_cmpeq_epi8(chunk, open_brace_vec);
            let close_mask = _mm_cmpeq_epi8(chunk, close_brace_vec);
            
            let combined = _mm_or_si128(_mm_or_si128(backslash_mask, open_mask), close_mask);
            let mask = _mm_movemask_epi8(combined);
            
            if mask != 0 {
                let offset = mask.trailing_zeros() as usize;
                return Some((pos + offset, self.input[pos + offset]));
            }
            
            pos += 16;
        }
        
        // Scalar fallback for remaining bytes
        for i in pos..len {
            match self.input[i] {
                b'\\' | b'{' | b'}' => return Some((i, self.input[i])),
                _ => continue,
            }
        }
        
        None
    }
    
    // Reuse the same helper methods from AVX2 implementation
    fn extract_text(&self, start: usize, end: usize) -> ConversionResult<String> {
        if start >= end {
            return Ok(String::new());
        }
        
        let slice = &self.input[start..end];
        let mut result = String::with_capacity(slice.len());
        let mut prev_space = false;
        
        for &byte in slice {
            match byte {
                b'\n' | b'\r' => {
                    if !prev_space && !result.is_empty() {
                        result.push(' ');
                        prev_space = true;
                    }
                }
                b' ' | b'\t' => {
                    if !prev_space {
                        result.push(' ');
                        prev_space = true;
                    }
                }
                _ => {
                    result.push(byte as char);
                    prev_space = false;
                }
            }
        }
        
        Ok(result)
    }
    
    fn read_control(&mut self) -> ConversionResult<RtfToken> {
        if self.position >= self.input.len() {
            return Err(ConversionError::LexerError(
                "Unexpected end of input after backslash".to_string(),
            ));
        }
        
        let ch = self.input[self.position];
        
        if ch.is_ascii_alphabetic() {
            self.read_control_word()
        } else if ch == b'\'' {
            self.read_hex_value()
        } else {
            self.position += 1;
            Ok(RtfToken::ControlSymbol(ch as char))
        }
    }
    
    fn read_control_word(&mut self) -> ConversionResult<RtfToken> {
        let start = self.position;
        
        while self.position < self.input.len() && self.input[self.position].is_ascii_alphabetic() {
            if self.position - start >= MAX_CONTROL_WORD_LENGTH {
                return Err(ConversionError::LexerError(
                    format!("Control word too long (max {} characters)", MAX_CONTROL_WORD_LENGTH)
                ));
            }
            self.position += 1;
        }
        
        let name = String::from_utf8_lossy(&self.input[start..self.position]).into_owned();
        
        let parameter = if self.position < self.input.len() {
            let ch = self.input[self.position];
            if ch == b'-' || ch.is_ascii_digit() {
                self.read_number()?
            } else {
                None
            }
        } else {
            None
        };
        
        if self.position < self.input.len() && self.input[self.position] == b' ' {
            self.position += 1;
        }
        
        Ok(RtfToken::ControlWord { name, parameter })
    }
    
    fn read_number(&mut self) -> ConversionResult<Option<i32>> {
        let is_negative = if self.position < self.input.len() && self.input[self.position] == b'-' {
            self.position += 1;
            true
        } else {
            false
        };
        
        let start = self.position;
        while self.position < self.input.len() && self.input[self.position].is_ascii_digit() {
            if self.position - start > 10 {
                return Err(ConversionError::LexerError(
                    "Number has too many digits".to_string()
                ));
            }
            self.position += 1;
        }
        
        if self.position == start {
            return Ok(None);
        }
        
        let num_str = std::str::from_utf8(&self.input[start..self.position])
            .map_err(|_| ConversionError::LexerError("Invalid UTF-8 in number".to_string()))?;
        
        let value: i32 = num_str.parse()
            .map_err(|_| ConversionError::LexerError("Invalid number".to_string()))?;
        
        Ok(Some(if is_negative { -value } else { value }))
    }
    
    fn read_hex_value(&mut self) -> ConversionResult<RtfToken> {
        self.position += 1;
        
        if self.position + 2 > self.input.len() {
            return Err(ConversionError::LexerError(
                "Expected two hex digits after \\'".to_string()
            ));
        }
        
        let hex_str = std::str::from_utf8(&self.input[self.position..self.position + 2])
            .map_err(|_| ConversionError::LexerError("Invalid UTF-8 in hex value".to_string()))?;
        
        let value = u8::from_str_radix(hex_str, 16)
            .map_err(|_| ConversionError::LexerError(format!("Invalid hex value: {}", hex_str)))?;
        
        self.position += 2;
        Ok(RtfToken::HexValue(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simd_tokenize_simple() {
        let tokens = tokenize_simd("Hello World").unwrap();
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            RtfToken::Text(text) => assert_eq!(text, "Hello World"),
            _ => panic!("Expected text token"),
        }
    }
    
    #[test]
    fn test_simd_tokenize_control_chars() {
        let tokens = tokenize_simd(r"{\rtf1 Hello}").unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0], RtfToken::GroupStart));
        assert!(matches!(tokens[1], RtfToken::ControlWord { name, parameter } if name == "rtf" && *parameter == Some(1)));
        assert!(matches!(tokens[2], RtfToken::Text(_)));
        assert!(matches!(tokens[3], RtfToken::GroupEnd));
    }
    
    #[test]
    fn test_simd_performance_characteristics() {
        // Test that SIMD version produces same results as scalar
        let test_cases = vec![
            r"{\rtf1\ansi\deff0 {\fonttbl {\f0 Times;}}Hello World}",
            r"{\b Bold text} and {\i italic text}",
            r"Special chars: \{, \}, \\",
            r"Unicode: \'e9\'e8",
        ];
        
        for input in test_cases {
            let scalar_tokens = super::super::rtf_lexer::tokenize(input).unwrap();
            let simd_tokens = tokenize_simd(input).unwrap();
            
            assert_eq!(scalar_tokens.len(), simd_tokens.len(), 
                      "Token count mismatch for input: {}", input);
            
            for (i, (scalar, simd)) in scalar_tokens.iter().zip(simd_tokens.iter()).enumerate() {
                assert_eq!(scalar, simd, 
                          "Token mismatch at position {} for input: {}", i, input);
            }
        }
    }
}