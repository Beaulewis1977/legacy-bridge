// SIMD-Optimized RTF Parser
// High-performance RTF parsing using SIMD instructions for string processing

use std::arch::x86_64::*;
use std::borrow::Cow;

/// SIMD-accelerated RTF token scanner
pub struct SimdRtfScanner {
    /// Input buffer
    buffer: Vec<u8>,
    /// Current position
    position: usize,
    /// SIMD control characters lookup
    control_chars: __m128i,
}

impl SimdRtfScanner {
    pub fn new(input: &str) -> Self {
        unsafe {
            // Load control characters for SIMD comparison
            // RTF control: \ { } \r \n
            let control_bytes = [b'\\', b'{', b'}', b'\r', b'\n', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            let control_chars = _mm_loadu_si128(control_bytes.as_ptr() as *const __m128i);
            
            Self {
                buffer: input.as_bytes().to_vec(),
                position: 0,
                control_chars,
            }
        }
    }
    
    /// Find next control character using SIMD
    #[inline]
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn find_next_control(&self, start: usize) -> Option<(usize, u8)> {
        let mut pos = start;
        let len = self.buffer.len();
        
        // Process 16 bytes at a time with SIMD
        while pos + 16 <= len {
            let chunk = _mm_loadu_si128(self.buffer[pos..].as_ptr() as *const __m128i);
            
            // Check for backslash
            let backslash = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'\\' as i8));
            let backslash_mask = _mm_movemask_epi8(backslash);
            
            if backslash_mask != 0 {
                let offset = backslash_mask.trailing_zeros() as usize;
                return Some((pos + offset, b'\\'));
            }
            
            // Check for braces
            let open_brace = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'{' as i8));
            let close_brace = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'}' as i8));
            let brace_mask = _mm_movemask_epi8(_mm_or_si128(open_brace, close_brace));
            
            if brace_mask != 0 {
                let offset = brace_mask.trailing_zeros() as usize;
                return Some((pos + offset, self.buffer[pos + offset]));
            }
            
            pos += 16;
        }
        
        // Handle remaining bytes
        for i in pos..len {
            match self.buffer[i] {
                b'\\' | b'{' | b'}' => return Some((i, self.buffer[i])),
                _ => continue,
            }
        }
        
        None
    }
    
    /// Extract text between control sequences using SIMD
    #[inline]
    pub fn extract_text_simd(&self, start: usize, end: usize) -> Cow<str> {
        if start >= end {
            return Cow::Borrowed("");
        }
        
        let slice = &self.buffer[start..end];
        
        // Fast path: check if conversion is needed
        if self.is_pure_ascii_simd(slice) {
            // Safe because we verified it's ASCII
            unsafe { Cow::Borrowed(std::str::from_utf8_unchecked(slice)) }
        } else {
            // Slow path: handle Unicode
            String::from_utf8_lossy(slice)
        }
    }
    
    /// Check if slice is pure ASCII using SIMD
    #[inline]
    #[cfg(target_arch = "x86_64")]
    fn is_pure_ascii_simd(&self, slice: &[u8]) -> bool {
        unsafe {
            let mut i = 0;
            let len = slice.len();
            
            // Process 16 bytes at a time
            while i + 16 <= len {
                let chunk = _mm_loadu_si128(slice[i..].as_ptr() as *const __m128i);
                let high_bits = _mm_movemask_epi8(chunk);
                
                // If any high bit is set, it's not ASCII
                if high_bits != 0 {
                    return false;
                }
                
                i += 16;
            }
            
            // Check remaining bytes
            slice[i..].iter().all(|&b| b < 128)
        }
    }
}

/// SIMD-optimized string operations for RTF
pub struct SimdStringOps;

impl SimdStringOps {
    /// Count occurrences of a character using SIMD
    #[inline]
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn count_char(text: &[u8], target: u8) -> usize {
        let mut count = 0;
        let mut i = 0;
        let len = text.len();
        let target_vec = _mm_set1_epi8(target as i8);
        
        // Process 16 bytes at a time
        while i + 16 <= len {
            let chunk = _mm_loadu_si128(text[i..].as_ptr() as *const __m128i);
            let cmp = _mm_cmpeq_epi8(chunk, target_vec);
            let mask = _mm_movemask_epi8(cmp);
            count += mask.count_ones() as usize;
            i += 16;
        }
        
        // Handle remaining bytes
        count += text[i..].iter().filter(|&&b| b == target).count();
        count
    }
    
    /// Find all positions of a character using SIMD
    #[inline]
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn find_all_positions(text: &[u8], target: u8) -> Vec<usize> {
        let mut positions = Vec::new();
        let mut i = 0;
        let len = text.len();
        let target_vec = _mm_set1_epi8(target as i8);
        
        // Process 16 bytes at a time
        while i + 16 <= len {
            let chunk = _mm_loadu_si128(text[i..].as_ptr() as *const __m128i);
            let cmp = _mm_cmpeq_epi8(chunk, target_vec);
            let mask = _mm_movemask_epi8(cmp);
            
            if mask != 0 {
                // Extract positions from mask
                let mut m = mask;
                let mut bit_pos = 0;
                while m != 0 {
                    if m & 1 != 0 {
                        positions.push(i + bit_pos);
                    }
                    m >>= 1;
                    bit_pos += 1;
                }
            }
            
            i += 16;
        }
        
        // Handle remaining bytes
        for (j, &b) in text[i..].iter().enumerate() {
            if b == target {
                positions.push(i + j);
            }
        }
        
        positions
    }
    
    /// Replace characters using SIMD
    #[inline]
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn replace_char(text: &mut [u8], find: u8, replace: u8) {
        let mut i = 0;
        let len = text.len();
        let find_vec = _mm_set1_epi8(find as i8);
        let replace_vec = _mm_set1_epi8(replace as i8);
        
        // Process 16 bytes at a time
        while i + 16 <= len {
            let chunk = _mm_loadu_si128(text[i..].as_ptr() as *const __m128i);
            let cmp = _mm_cmpeq_epi8(chunk, find_vec);
            
            // Create replacement mask
            let replacement = _mm_blendv_epi8(chunk, replace_vec, cmp);
            _mm_storeu_si128(text[i..].as_mut_ptr() as *mut __m128i, replacement);
            
            i += 16;
        }
        
        // Handle remaining bytes
        for j in i..len {
            if text[j] == find {
                text[j] = replace;
            }
        }
    }
}

/// Memory pool for RTF parsing buffers
pub struct RtfBufferPool {
    small_buffers: Vec<Vec<u8>>,
    medium_buffers: Vec<Vec<u8>>,
    large_buffers: Vec<Vec<u8>>,
}

impl RtfBufferPool {
    pub fn new() -> Self {
        Self {
            small_buffers: Vec::with_capacity(32),
            medium_buffers: Vec::with_capacity(16),
            large_buffers: Vec::with_capacity(8),
        }
    }
    
    pub fn acquire(&mut self, size: usize) -> Vec<u8> {
        if size <= 1024 {
            self.small_buffers.pop().unwrap_or_else(|| Vec::with_capacity(1024))
        } else if size <= 16384 {
            self.medium_buffers.pop().unwrap_or_else(|| Vec::with_capacity(16384))
        } else {
            self.large_buffers.pop().unwrap_or_else(|| Vec::with_capacity(size))
        }
    }
    
    pub fn release(&mut self, mut buffer: Vec<u8>) {
        buffer.clear();
        
        match buffer.capacity() {
            0..=1024 => {
                if self.small_buffers.len() < 32 {
                    self.small_buffers.push(buffer);
                }
            }
            1025..=16384 => {
                if self.medium_buffers.len() < 16 {
                    self.medium_buffers.push(buffer);
                }
            }
            _ => {
                if self.large_buffers.len() < 8 {
                    self.large_buffers.push(buffer);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simd_find_control() {
        let scanner = SimdRtfScanner::new(r"{\rtf1\ansi\deff0 {\fonttbl {\f0 Times New Roman;}}");
        
        unsafe {
            let result = scanner.find_next_control(0);
            assert_eq!(result, Some((0, b'{')));
            
            let result = scanner.find_next_control(1);
            assert_eq!(result, Some((1, b'\\')));
        }
    }
    
    #[test]
    fn test_simd_string_ops() {
        let mut text = b"Hello World! This is a test.".to_vec();
        
        unsafe {
            let count = SimdStringOps::count_char(&text, b'l');
            assert_eq!(count, 3);
            
            let positions = SimdStringOps::find_all_positions(&text, b' ');
            assert_eq!(positions.len(), 5);
            
            SimdStringOps::replace_char(&mut text, b' ', b'_');
            assert_eq!(&text[5], &b'_');
        }
    }
}