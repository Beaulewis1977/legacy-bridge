// Zero-Copy String Operations for LegacyBridge
// Minimizes allocations and improves performance

use std::borrow::Cow;
use std::ops::Range;
use bstr::{ByteSlice, BStr};

/// Zero-copy string view with efficient operations
#[derive(Clone)]
pub struct StringView<'a> {
    source: &'a str,
    ranges: Vec<Range<usize>>,
}

impl<'a> StringView<'a> {
    /// Create a new string view
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            ranges: vec![0..source.len()],
        }
    }
    
    /// Create a view from a slice
    pub fn from_slice(source: &'a str, start: usize, end: usize) -> Self {
        Self {
            source,
            ranges: vec![start..end],
        }
    }
    
    /// Split view without allocation
    pub fn split_at(&self, index: usize) -> (Self, Self) {
        let mut left_ranges = Vec::new();
        let mut right_ranges = Vec::new();
        let mut current_len = 0;
        
        for range in &self.ranges {
            let range_len = range.end - range.start;
            
            if current_len + range_len <= index {
                left_ranges.push(range.clone());
            } else if current_len >= index {
                right_ranges.push(range.clone());
            } else {
                let split_point = range.start + (index - current_len);
                left_ranges.push(range.start..split_point);
                right_ranges.push(split_point..range.end);
            }
            
            current_len += range_len;
        }
        
        (
            Self { source: self.source, ranges: left_ranges },
            Self { source: self.source, ranges: right_ranges },
        )
    }
    
    /// Get total length without materializing
    pub fn len(&self) -> usize {
        self.ranges.iter().map(|r| r.end - r.start).sum()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty() || self.len() == 0
    }
    
    /// Materialize to owned string only when necessary
    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(self.len());
        for range in &self.ranges {
            result.push_str(&self.source[range.clone()]);
        }
        result
    }
    
    /// Iterate over string slices without allocation
    pub fn iter_slices(&self) -> impl Iterator<Item = &'a str> {
        let source = self.source;
        self.ranges.iter().map(move |range| &source[range.clone()])
    }
}

/// Zero-copy RTF text processor
pub struct ZeroCopyProcessor<'a> {
    input: &'a [u8],
    position: usize,
}

impl<'a> ZeroCopyProcessor<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            position: 0,
        }
    }
    
    /// Read until delimiter without allocation
    pub fn read_until(&mut self, delimiter: u8) -> Option<&'a str> {
        let start = self.position;
        
        while self.position < self.input.len() {
            if self.input[self.position] == delimiter {
                let result = &self.input[start..self.position];
                self.position += 1; // Skip delimiter
                return result.to_str().ok();
            }
            self.position += 1;
        }
        
        if start < self.input.len() {
            let result = &self.input[start..];
            self.position = self.input.len();
            result.to_str().ok()
        } else {
            None
        }
    }
    
    /// Read control word without allocation
    pub fn read_control_word(&mut self) -> Option<(&'a str, Option<i32>)> {
        if self.position >= self.input.len() || self.input[self.position] != b'\\' {
            return None;
        }
        
        self.position += 1; // Skip backslash
        let cmd_start = self.position;
        
        // Read command letters
        while self.position < self.input.len() {
            match self.input[self.position] {
                b'a'..=b'z' | b'A'..=b'Z' => self.position += 1,
                _ => break,
            }
        }
        
        if cmd_start == self.position {
            return None;
        }
        
        let cmd = self.input[cmd_start..self.position].to_str().ok()?;
        
        // Check for numeric parameter
        let param_start = self.position;
        let mut has_minus = false;
        
        if self.position < self.input.len() && self.input[self.position] == b'-' {
            has_minus = true;
            self.position += 1;
        }
        
        while self.position < self.input.len() {
            match self.input[self.position] {
                b'0'..=b'9' => self.position += 1,
                _ => break,
            }
        }
        
        let param = if self.position > param_start + (has_minus as usize) {
            let param_str = self.input[param_start..self.position].to_str().ok()?;
            param_str.parse().ok()
        } else {
            None
        };
        
        // Skip trailing space if present
        if self.position < self.input.len() && self.input[self.position] == b' ' {
            self.position += 1;
        }
        
        Some((cmd, param))
    }
    
    /// Extract text span without allocation
    pub fn extract_span(&self, start: usize, end: usize) -> &'a str {
        self.input[start..end].to_str().unwrap_or("")
    }
}

/// Cow-based string builder for conditional allocation
pub struct CowStringBuilder<'a> {
    base: Option<&'a str>,
    modifications: Vec<Modification<'a>>,
}

#[derive(Debug)]
enum Modification<'a> {
    Append(Cow<'a, str>),
    Replace(usize, usize, Cow<'a, str>),
    Insert(usize, Cow<'a, str>),
}

impl<'a> CowStringBuilder<'a> {
    pub fn new(base: &'a str) -> Self {
        Self {
            base: Some(base),
            modifications: Vec::new(),
        }
    }
    
    pub fn append(&mut self, text: &'a str) -> &mut Self {
        self.modifications.push(Modification::Append(Cow::Borrowed(text)));
        self
    }
    
    pub fn append_owned(&mut self, text: String) -> &mut Self {
        self.modifications.push(Modification::Append(Cow::Owned(text)));
        self
    }
    
    pub fn replace(&mut self, start: usize, end: usize, text: &'a str) -> &mut Self {
        self.modifications.push(Modification::Replace(start, end, Cow::Borrowed(text)));
        self
    }
    
    pub fn build(&self) -> Cow<'a, str> {
        if self.modifications.is_empty() {
            return Cow::Borrowed(self.base.unwrap_or(""));
        }
        
        // Only allocate if modifications exist
        let mut result = String::from(self.base.unwrap_or(""));
        
        for modification in &self.modifications {
            match modification {
                Modification::Append(text) => result.push_str(text),
                Modification::Replace(start, end, text) => {
                    result.replace_range(*start..*end, text);
                }
                Modification::Insert(pos, text) => {
                    result.insert_str(*pos, text);
                }
            }
        }
        
        Cow::Owned(result)
    }
}

/// Efficient string interning with zero-copy returns
pub struct ZeroCopyInterner {
    storage: String,
    indices: std::collections::HashMap<u64, Range<usize>>,
}

impl ZeroCopyInterner {
    pub fn new() -> Self {
        Self {
            storage: String::with_capacity(4096),
            indices: std::collections::HashMap::new(),
        }
    }
    
    /// Intern a string and return a zero-copy reference
    pub fn intern<'a>(&'a mut self, text: &str) -> &'a str {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        // Hash the string
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Check if already interned
        if let Some(range) = self.indices.get(&hash) {
            return &self.storage[range.clone()];
        }
        
        // Add to storage
        let start = self.storage.len();
        self.storage.push_str(text);
        let end = self.storage.len();
        
        self.indices.insert(hash, start..end);
        
        // Return reference to interned string
        &self.storage[start..end]
    }
    
    /// Clear the interner
    pub fn clear(&mut self) {
        self.storage.clear();
        self.indices.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_string_view() {
        let source = "Hello, World!";
        let view = StringView::new(source);
        
        assert_eq!(view.len(), 13);
        
        let (left, right) = view.split_at(7);
        assert_eq!(left.to_string(), "Hello, ");
        assert_eq!(right.to_string(), "World!");
    }
    
    #[test]
    fn test_zero_copy_processor() {
        let input = r"\rtf1\ansi\deff0 Hello";
        let mut processor = ZeroCopyProcessor::new(input);
        
        let (cmd, param) = processor.read_control_word().unwrap();
        assert_eq!(cmd, "rtf");
        assert_eq!(param, Some(1));
        
        let (cmd, param) = processor.read_control_word().unwrap();
        assert_eq!(cmd, "ansi");
        assert_eq!(param, None);
    }
    
    #[test]
    fn test_cow_builder() {
        let base = "Hello";
        let mut builder = CowStringBuilder::new(base);
        
        // No modifications - should return borrowed
        match builder.build() {
            Cow::Borrowed(s) => assert_eq!(s, "Hello"),
            Cow::Owned(_) => panic!("Expected borrowed"),
        }
        
        // With modifications - should return owned
        builder.append(", World!");
        match builder.build() {
            Cow::Owned(s) => assert_eq!(s, "Hello, World!"),
            Cow::Borrowed(_) => panic!("Expected owned"),
        }
    }
}