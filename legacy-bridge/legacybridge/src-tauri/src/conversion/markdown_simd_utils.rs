// SIMD-Optimized Markdown String Utilities
// High-performance string operations for Markdown parsing

use std::arch::x86_64::*;

/// SIMD-accelerated Markdown character scanner
pub struct SimdMarkdownScanner {
    /// CPU features available
    cpu_features: CpuFeatures,
}

/// CPU feature detection
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

impl SimdMarkdownScanner {
    pub fn new() -> Self {
        Self {
            cpu_features: CpuFeatures::detect(),
        }
    }
    
    /// Find Markdown special characters using SIMD
    /// Returns positions of: * _ # [ ] ` | \
    pub fn find_special_chars(&self, text: &[u8]) -> Vec<usize> {
        if self.cpu_features.has_avx2 {
            unsafe { self.find_special_chars_avx2(text) }
        } else if self.cpu_features.has_sse42 {
            unsafe { self.find_special_chars_sse42(text) }
        } else {
            self.find_special_chars_scalar(text)
        }
    }
    
    /// AVX2 implementation - processes 32 bytes at a time
    #[target_feature(enable = "avx2")]
    unsafe fn find_special_chars_avx2(&self, text: &[u8]) -> Vec<usize> {
        let mut positions = Vec::with_capacity(text.len() / 20); // Estimate
        let mut pos = 0;
        let len = text.len();
        
        // Create vectors for each special character
        let star_vec = _mm256_set1_epi8(b'*' as i8);
        let underscore_vec = _mm256_set1_epi8(b'_' as i8);
        let hash_vec = _mm256_set1_epi8(b'#' as i8);
        let open_bracket_vec = _mm256_set1_epi8(b'[' as i8);
        let close_bracket_vec = _mm256_set1_epi8(b']' as i8);
        let backtick_vec = _mm256_set1_epi8(b'`' as i8);
        let pipe_vec = _mm256_set1_epi8(b'|' as i8);
        let backslash_vec = _mm256_set1_epi8(b'\\' as i8);
        
        while pos + 32 <= len {
            let chunk = _mm256_loadu_si256(text[pos..].as_ptr() as *const __m256i);
            
            // Check for each special character
            let star_mask = _mm256_cmpeq_epi8(chunk, star_vec);
            let underscore_mask = _mm256_cmpeq_epi8(chunk, underscore_vec);
            let hash_mask = _mm256_cmpeq_epi8(chunk, hash_vec);
            let open_mask = _mm256_cmpeq_epi8(chunk, open_bracket_vec);
            let close_mask = _mm256_cmpeq_epi8(chunk, close_bracket_vec);
            let backtick_mask = _mm256_cmpeq_epi8(chunk, backtick_vec);
            let pipe_mask = _mm256_cmpeq_epi8(chunk, pipe_vec);
            let backslash_mask = _mm256_cmpeq_epi8(chunk, backslash_vec);
            
            // Combine all masks
            let combined1 = _mm256_or_si256(
                _mm256_or_si256(star_mask, underscore_mask),
                _mm256_or_si256(hash_mask, open_mask)
            );
            let combined2 = _mm256_or_si256(
                _mm256_or_si256(close_mask, backtick_mask),
                _mm256_or_si256(pipe_mask, backslash_mask)
            );
            let combined = _mm256_or_si256(combined1, combined2);
            
            let mask = _mm256_movemask_epi8(combined);
            
            if mask != 0 {
                // Extract positions from mask
                let mut m = mask;
                let mut bit_pos = 0;
                while m != 0 {
                    if m & 1 != 0 {
                        positions.push(pos + bit_pos);
                    }
                    m >>= 1;
                    bit_pos += 1;
                }
            }
            
            pos += 32;
        }
        
        // Handle remaining bytes with SSE
        if pos + 16 <= len {
            let chunk = _mm_loadu_si128(text[pos..].as_ptr() as *const __m128i);
            
            let star_mask = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'*' as i8));
            let underscore_mask = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'_' as i8));
            let hash_mask = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'#' as i8));
            let open_mask = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'[' as i8));
            let close_mask = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b']' as i8));
            let backtick_mask = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'`' as i8));
            let pipe_mask = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'|' as i8));
            let backslash_mask = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'\\' as i8));
            
            let combined1 = _mm_or_si128(
                _mm_or_si128(star_mask, underscore_mask),
                _mm_or_si128(hash_mask, open_mask)
            );
            let combined2 = _mm_or_si128(
                _mm_or_si128(close_mask, backtick_mask),
                _mm_or_si128(pipe_mask, backslash_mask)
            );
            let combined = _mm_or_si128(combined1, combined2);
            
            let mask = _mm_movemask_epi8(combined);
            
            if mask != 0 {
                let mut m = mask;
                let mut bit_pos = 0;
                while m != 0 {
                    if m & 1 != 0 {
                        positions.push(pos + bit_pos);
                    }
                    m >>= 1;
                    bit_pos += 1;
                }
            }
            
            pos += 16;
        }
        
        // Scalar fallback for remaining
        for i in pos..len {
            match text[i] {
                b'*' | b'_' | b'#' | b'[' | b']' | b'`' | b'|' | b'\\' => {
                    positions.push(i);
                }
                _ => {}
            }
        }
        
        positions
    }
    
    /// SSE4.2 implementation - processes 16 bytes at a time
    #[target_feature(enable = "sse4.2")]
    unsafe fn find_special_chars_sse42(&self, text: &[u8]) -> Vec<usize> {
        let mut positions = Vec::with_capacity(text.len() / 20);
        let mut pos = 0;
        let len = text.len();
        
        while pos + 16 <= len {
            let chunk = _mm_loadu_si128(text[pos..].as_ptr() as *const __m128i);
            
            // Use string search instruction for efficiency
            let special_chars = b"*_#[]`|\\";
            let mut found = false;
            
            for &ch in special_chars {
                let char_vec = _mm_set1_epi8(ch as i8);
                let mask = _mm_cmpeq_epi8(chunk, char_vec);
                let bits = _mm_movemask_epi8(mask);
                
                if bits != 0 {
                    found = true;
                    let mut m = bits;
                    let mut bit_pos = 0;
                    while m != 0 {
                        if m & 1 != 0 {
                            positions.push(pos + bit_pos);
                        }
                        m >>= 1;
                        bit_pos += 1;
                    }
                }
            }
            
            pos += 16;
        }
        
        // Scalar fallback
        for i in pos..len {
            match text[i] {
                b'*' | b'_' | b'#' | b'[' | b']' | b'`' | b'|' | b'\\' => {
                    positions.push(i);
                }
                _ => {}
            }
        }
        
        // Sort and deduplicate since we may have found the same position multiple times
        positions.sort_unstable();
        positions.dedup();
        
        positions
    }
    
    /// Scalar fallback implementation
    fn find_special_chars_scalar(&self, text: &[u8]) -> Vec<usize> {
        let mut positions = Vec::with_capacity(text.len() / 20);
        
        for (i, &byte) in text.iter().enumerate() {
            match byte {
                b'*' | b'_' | b'#' | b'[' | b']' | b'`' | b'|' | b'\\' => {
                    positions.push(i);
                }
                _ => {}
            }
        }
        
        positions
    }
    
    /// Count line breaks using SIMD
    pub fn count_lines(&self, text: &[u8]) -> usize {
        if self.cpu_features.has_avx2 {
            unsafe { self.count_lines_avx2(text) }
        } else if self.cpu_features.has_sse42 {
            unsafe { self.count_lines_sse42(text) }
        } else {
            text.iter().filter(|&&b| b == b'\n').count()
        }
    }
    
    #[target_feature(enable = "avx2")]
    unsafe fn count_lines_avx2(&self, text: &[u8]) -> usize {
        let mut count = 0;
        let mut pos = 0;
        let len = text.len();
        let newline_vec = _mm256_set1_epi8(b'\n' as i8);
        
        while pos + 32 <= len {
            let chunk = _mm256_loadu_si256(text[pos..].as_ptr() as *const __m256i);
            let cmp = _mm256_cmpeq_epi8(chunk, newline_vec);
            let mask = _mm256_movemask_epi8(cmp);
            count += mask.count_ones() as usize;
            pos += 32;
        }
        
        // Handle remainder
        count += text[pos..].iter().filter(|&&b| b == b'\n').count();
        count
    }
    
    #[target_feature(enable = "sse4.2")]
    unsafe fn count_lines_sse42(&self, text: &[u8]) -> usize {
        let mut count = 0;
        let mut pos = 0;
        let len = text.len();
        let newline_vec = _mm_set1_epi8(b'\n' as i8);
        
        while pos + 16 <= len {
            let chunk = _mm_loadu_si128(text[pos..].as_ptr() as *const __m128i);
            let cmp = _mm_cmpeq_epi8(chunk, newline_vec);
            let mask = _mm_movemask_epi8(cmp);
            count += mask.count_ones() as usize;
            pos += 16;
        }
        
        count += text[pos..].iter().filter(|&&b| b == b'\n').count();
        count
    }
}

/// SIMD-optimized UTF-8 validation
pub struct SimdUtf8Validator {
    cpu_features: CpuFeatures,
}

impl SimdUtf8Validator {
    pub fn new() -> Self {
        Self {
            cpu_features: CpuFeatures::detect(),
        }
    }
    
    /// Validate UTF-8 using SIMD
    pub fn is_valid_utf8(&self, text: &[u8]) -> bool {
        if self.cpu_features.has_avx2 {
            unsafe { self.validate_utf8_avx2(text) }
        } else if self.cpu_features.has_sse42 {
            unsafe { self.validate_utf8_sse42(text) }
        } else {
            std::str::from_utf8(text).is_ok()
        }
    }
    
    #[target_feature(enable = "avx2")]
    unsafe fn validate_utf8_avx2(&self, text: &[u8]) -> bool {
        // Fast path: check if all bytes are ASCII
        let mut pos = 0;
        let len = text.len();
        
        while pos + 32 <= len {
            let chunk = _mm256_loadu_si256(text[pos..].as_ptr() as *const __m256i);
            let high_bits = _mm256_movemask_epi8(chunk);
            
            if high_bits == 0 {
                // All ASCII, continue
                pos += 32;
            } else {
                // Found non-ASCII, fall back to standard validation
                return std::str::from_utf8(&text[pos..]).is_ok();
            }
        }
        
        // Check remaining bytes
        std::str::from_utf8(&text[pos..]).is_ok()
    }
    
    #[target_feature(enable = "sse4.2")]
    unsafe fn validate_utf8_sse42(&self, text: &[u8]) -> bool {
        let mut pos = 0;
        let len = text.len();
        
        while pos + 16 <= len {
            let chunk = _mm_loadu_si128(text[pos..].as_ptr() as *const __m128i);
            let high_bits = _mm_movemask_epi8(chunk);
            
            if high_bits == 0 {
                pos += 16;
            } else {
                return std::str::from_utf8(&text[pos..]).is_ok();
            }
        }
        
        std::str::from_utf8(&text[pos..]).is_ok()
    }
}

/// SIMD-optimized whitespace operations
pub struct SimdWhitespaceOps {
    cpu_features: CpuFeatures,
}

impl SimdWhitespaceOps {
    pub fn new() -> Self {
        Self {
            cpu_features: CpuFeatures::detect(),
        }
    }
    
    /// Normalize whitespace using SIMD
    pub fn normalize_whitespace(&self, text: &str) -> String {
        let bytes = text.as_bytes();
        
        if self.cpu_features.has_avx2 {
            unsafe { self.normalize_whitespace_avx2(bytes) }
        } else if self.cpu_features.has_sse42 {
            unsafe { self.normalize_whitespace_sse42(bytes) }
        } else {
            self.normalize_whitespace_scalar(text)
        }
    }
    
    #[target_feature(enable = "avx2")]
    unsafe fn normalize_whitespace_avx2(&self, bytes: &[u8]) -> String {
        let mut result = Vec::with_capacity(bytes.len());
        let mut pos = 0;
        let len = bytes.len();
        let mut prev_was_space = false;
        
        // Whitespace characters to check
        let space_vec = _mm256_set1_epi8(b' ' as i8);
        let tab_vec = _mm256_set1_epi8(b'\t' as i8);
        let newline_vec = _mm256_set1_epi8(b'\n' as i8);
        let cr_vec = _mm256_set1_epi8(b'\r' as i8);
        
        while pos + 32 <= len {
            let chunk = _mm256_loadu_si256(bytes[pos..].as_ptr() as *const __m256i);
            
            // Check for whitespace characters
            let space_mask = _mm256_cmpeq_epi8(chunk, space_vec);
            let tab_mask = _mm256_cmpeq_epi8(chunk, tab_vec);
            let newline_mask = _mm256_cmpeq_epi8(chunk, newline_vec);
            let cr_mask = _mm256_cmpeq_epi8(chunk, cr_vec);
            
            let whitespace_mask = _mm256_or_si256(
                _mm256_or_si256(space_mask, tab_mask),
                _mm256_or_si256(newline_mask, cr_mask)
            );
            
            let mask = _mm256_movemask_epi8(whitespace_mask);
            
            // Process bytes based on mask
            for i in 0..32 {
                if pos + i >= len {
                    break;
                }
                
                let byte = bytes[pos + i];
                let is_whitespace = (mask & (1 << i)) != 0;
                
                if is_whitespace {
                    if !prev_was_space && !result.is_empty() {
                        result.push(b' ');
                        prev_was_space = true;
                    }
                } else {
                    result.push(byte);
                    prev_was_space = false;
                }
            }
            
            pos += 32;
        }
        
        // Handle remaining bytes
        for &byte in &bytes[pos..] {
            if byte.is_ascii_whitespace() {
                if !prev_was_space && !result.is_empty() {
                    result.push(b' ');
                    prev_was_space = true;
                }
            } else {
                result.push(byte);
                prev_was_space = false;
            }
        }
        
        // Trim trailing space if present
        if result.last() == Some(&b' ') {
            result.pop();
        }
        
        String::from_utf8(result).unwrap_or_else(|_| String::new())
    }
    
    #[target_feature(enable = "sse4.2")]
    unsafe fn normalize_whitespace_sse42(&self, bytes: &[u8]) -> String {
        let mut result = Vec::with_capacity(bytes.len());
        let mut pos = 0;
        let len = bytes.len();
        let mut prev_was_space = false;
        
        let space_vec = _mm_set1_epi8(b' ' as i8);
        let tab_vec = _mm_set1_epi8(b'\t' as i8);
        let newline_vec = _mm_set1_epi8(b'\n' as i8);
        let cr_vec = _mm_set1_epi8(b'\r' as i8);
        
        while pos + 16 <= len {
            let chunk = _mm_loadu_si128(bytes[pos..].as_ptr() as *const __m128i);
            
            let space_mask = _mm_cmpeq_epi8(chunk, space_vec);
            let tab_mask = _mm_cmpeq_epi8(chunk, tab_vec);
            let newline_mask = _mm_cmpeq_epi8(chunk, newline_vec);
            let cr_mask = _mm_cmpeq_epi8(chunk, cr_vec);
            
            let whitespace_mask = _mm_or_si128(
                _mm_or_si128(space_mask, tab_mask),
                _mm_or_si128(newline_mask, cr_mask)
            );
            
            let mask = _mm_movemask_epi8(whitespace_mask);
            
            for i in 0..16 {
                if pos + i >= len {
                    break;
                }
                
                let byte = bytes[pos + i];
                let is_whitespace = (mask & (1 << i)) != 0;
                
                if is_whitespace {
                    if !prev_was_space && !result.is_empty() {
                        result.push(b' ');
                        prev_was_space = true;
                    }
                } else {
                    result.push(byte);
                    prev_was_space = false;
                }
            }
            
            pos += 16;
        }
        
        for &byte in &bytes[pos..] {
            if byte.is_ascii_whitespace() {
                if !prev_was_space && !result.is_empty() {
                    result.push(b' ');
                    prev_was_space = true;
                }
            } else {
                result.push(byte);
                prev_was_space = false;
            }
        }
        
        if result.last() == Some(&b' ') {
            result.pop();
        }
        
        String::from_utf8(result).unwrap_or_else(|_| String::new())
    }
    
    fn normalize_whitespace_scalar(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut prev_was_space = false;
        
        for ch in text.chars() {
            if ch.is_whitespace() {
                if !prev_was_space && !result.is_empty() {
                    result.push(' ');
                    prev_was_space = true;
                }
            } else {
                result.push(ch);
                prev_was_space = false;
            }
        }
        
        result.trim_end().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_markdown_special_chars() {
        let scanner = SimdMarkdownScanner::new();
        let text = b"# Heading with **bold** and *italic* text [link](url)";
        let positions = scanner.find_special_chars(text);
        
        // Should find: # at 0, ** at 15,16, ** at 22,23, * at 29, * at 36, [ at 43, ] at 47
        assert!(positions.contains(&0));  // #
        assert!(positions.contains(&15)); // *
        assert!(positions.contains(&16)); // *
        assert!(positions.contains(&22)); // *
        assert!(positions.contains(&23)); // *
        assert!(positions.contains(&29)); // *
        assert!(positions.contains(&36)); // *
        assert!(positions.contains(&43)); // [
        assert!(positions.contains(&47)); // ]
    }
    
    #[test]
    fn test_line_counting() {
        let scanner = SimdMarkdownScanner::new();
        let text = b"Line 1\nLine 2\nLine 3\n";
        assert_eq!(scanner.count_lines(text), 3);
        
        let text_no_final_newline = b"Line 1\nLine 2\nLine 3";
        assert_eq!(scanner.count_lines(text_no_final_newline), 2);
    }
    
    #[test]
    fn test_utf8_validation() {
        let validator = SimdUtf8Validator::new();
        
        // Valid ASCII
        assert!(validator.is_valid_utf8(b"Hello World"));
        
        // Valid UTF-8
        assert!(validator.is_valid_utf8("café résumé".as_bytes()));
        
        // Invalid UTF-8
        assert!(!validator.is_valid_utf8(&[0xFF, 0xFE, 0xFD]));
    }
    
    #[test]
    fn test_whitespace_normalization() {
        let ops = SimdWhitespaceOps::new();
        
        assert_eq!(
            ops.normalize_whitespace("Hello   World"),
            "Hello World"
        );
        
        assert_eq!(
            ops.normalize_whitespace("Multiple\n\n\nNewlines"),
            "Multiple Newlines"
        );
        
        assert_eq!(
            ops.normalize_whitespace("\t\tTabs\t\tEverywhere\t"),
            "Tabs Everywhere"
        );
        
        assert_eq!(
            ops.normalize_whitespace("  Leading and trailing  "),
            "Leading and trailing"
        );
    }
}