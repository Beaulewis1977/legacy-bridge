// Memory Pool Implementation for LegacyBridge
// Reduces allocation overhead and improves performance

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::mem;
use once_cell::sync::Lazy;

/// Thread-safe object pool for reusable allocations
pub struct ObjectPool<T> {
    pool: Arc<Mutex<VecDeque<T>>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
    reset_fn: Option<Box<dyn Fn(&mut T) + Send + Sync>>,
}

impl<T: Send + 'static> ObjectPool<T> {
    /// Create a new object pool
    pub fn new<F>(max_size: usize, factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            factory: Box::new(factory),
            max_size,
            reset_fn: None,
        }
    }
    
    /// Create a pool with a reset function
    pub fn with_reset<F, R>(max_size: usize, factory: F, reset: R) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
        R: Fn(&mut T) + Send + Sync + 'static,
    {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            factory: Box::new(factory),
            max_size,
            reset_fn: Some(Box::new(reset)),
        }
    }
    
    /// Acquire an object from the pool
    pub fn acquire(&self) -> PooledObject<T> {
        let mut pool = self.pool.lock().unwrap();
        let obj = pool.pop_front().unwrap_or_else(|| (self.factory)());
        
        PooledObject {
            value: Some(obj),
            pool: Arc::clone(&self.pool),
            reset_fn: self.reset_fn.as_ref().map(|f| Arc::new(f.clone())),
        }
    }
    
    /// Get current pool size
    pub fn size(&self) -> usize {
        self.pool.lock().unwrap().len()
    }
}

/// RAII wrapper for pooled objects
pub struct PooledObject<T> {
    value: Option<T>,
    pool: Arc<Mutex<VecDeque<T>>>,
    reset_fn: Option<Arc<Box<dyn Fn(&mut T) + Send + Sync>>>,
}

impl<T> PooledObject<T> {
    /// Get a reference to the pooled value
    pub fn as_ref(&self) -> &T {
        self.value.as_ref().unwrap()
    }
    
    /// Get a mutable reference to the pooled value
    pub fn as_mut(&mut self) -> &mut T {
        self.value.as_mut().unwrap()
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(mut value) = self.value.take() {
            // Reset the object if a reset function is provided
            if let Some(reset_fn) = &self.reset_fn {
                (reset_fn)(&mut value);
            }
            
            // Return to pool if there's space
            let mut pool = self.pool.lock().unwrap();
            if pool.len() < pool.capacity() {
                pool.push_back(value);
            }
        }
    }
}

impl<T> std::ops::Deref for PooledObject<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

/// Global memory pools for common types
pub static STRING_POOL: Lazy<ObjectPool<String>> = Lazy::new(|| {
    ObjectPool::with_reset(
        128,
        || String::with_capacity(256),
        |s| s.clear(),
    )
});

pub static VEC_U8_POOL: Lazy<ObjectPool<Vec<u8>>> = Lazy::new(|| {
    ObjectPool::with_reset(
        64,
        || Vec::with_capacity(4096),
        |v| v.clear(),
    )
});

pub static NODE_VEC_POOL: Lazy<ObjectPool<Vec<crate::RtfNode>>> = Lazy::new(|| {
    ObjectPool::with_reset(
        32,
        || Vec::with_capacity(100),
        |v| v.clear(),
    )
});

/// Arena allocator for temporary allocations
pub struct Arena {
    chunks: Vec<Vec<u8>>,
    current_chunk: usize,
    current_offset: usize,
    chunk_size: usize,
}

impl Arena {
    pub fn new(chunk_size: usize) -> Self {
        let mut chunks = Vec::new();
        chunks.push(vec![0; chunk_size]);
        
        Self {
            chunks,
            current_chunk: 0,
            current_offset: 0,
            chunk_size,
        }
    }
    
    /// Allocate bytes from the arena
    pub fn alloc(&mut self, size: usize) -> &mut [u8] {
        let align = mem::align_of::<usize>();
        let aligned_size = (size + align - 1) & !(align - 1);
        
        // Check if we need a new chunk
        if self.current_offset + aligned_size > self.chunk_size {
            self.chunks.push(vec![0; self.chunk_size.max(aligned_size)]);
            self.current_chunk += 1;
            self.current_offset = 0;
        }
        
        let chunk = &mut self.chunks[self.current_chunk];
        let start = self.current_offset;
        self.current_offset += aligned_size;
        
        &mut chunk[start..start + size]
    }
    
    /// Allocate a string in the arena
    pub fn alloc_str(&mut self, s: &str) -> &str {
        let bytes = self.alloc(s.len());
        bytes.copy_from_slice(s.as_bytes());
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }
    
    /// Reset the arena for reuse
    pub fn reset(&mut self) {
        self.current_chunk = 0;
        self.current_offset = 0;
        
        // Keep only the first chunk to avoid reallocation
        self.chunks.truncate(1);
    }
}

/// Zero-copy string builder using memory pool
pub struct PooledStringBuilder {
    buffer: PooledObject<String>,
}

impl PooledStringBuilder {
    pub fn new() -> Self {
        Self {
            buffer: STRING_POOL.acquire(),
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        let mut builder = Self::new();
        builder.buffer.reserve(capacity);
        builder
    }
    
    pub fn push_str(&mut self, s: &str) {
        self.buffer.push_str(s);
    }
    
    pub fn push(&mut self, ch: char) {
        self.buffer.push(ch);
    }
    
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }
    
    pub fn finish(self) -> String {
        let mut buffer = self.buffer;
        mem::take(&mut *buffer)
    }
}

/// Optimized converter using memory pools
pub struct PooledRtfConverter {
    string_pool: ObjectPool<String>,
    vec_pool: ObjectPool<Vec<u8>>,
    arena: Arena,
}

impl PooledRtfConverter {
    pub fn new() -> Self {
        Self {
            string_pool: ObjectPool::with_reset(16, || String::with_capacity(256), |s| s.clear()),
            vec_pool: ObjectPool::with_reset(16, || Vec::with_capacity(1024), |v| v.clear()),
            arena: Arena::new(64 * 1024), // 64KB chunks
        }
    }
    
    pub fn convert(&mut self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Use pooled resources
        let mut output = self.string_pool.acquire();
        let mut temp_buffer = self.vec_pool.acquire();
        
        // Process conversion using pooled resources
        self.process_with_pools(input, &mut output, &mut temp_buffer)?;
        
        // Extract result
        Ok(mem::take(&mut *output))
    }
    
    fn process_with_pools(
        &mut self,
        input: &str,
        output: &mut String,
        temp: &mut Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Use arena for temporary string allocations
        let temp_str = self.arena.alloc_str("temporary");
        
        // Process input...
        output.push_str("Processed: ");
        output.push_str(input);
        
        Ok(())
    }
    
    pub fn reset(&mut self) {
        self.arena.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_object_pool() {
        let pool: ObjectPool<Vec<u8>> = ObjectPool::with_reset(
            2,
            || vec![0; 100],
            |v| v.clear(),
        );
        
        {
            let mut obj1 = pool.acquire();
            obj1.push(1);
            assert_eq!(obj1.len(), 1);
        }
        
        assert_eq!(pool.size(), 1);
        
        {
            let obj2 = pool.acquire();
            assert_eq!(obj2.len(), 0); // Should be reset
        }
    }
    
    #[test]
    fn test_arena_allocator() {
        let mut arena = Arena::new(1024);
        
        let s1 = arena.alloc_str("hello");
        let s2 = arena.alloc_str("world");
        
        assert_eq!(s1, "hello");
        assert_eq!(s2, "world");
        
        arena.reset();
        
        let s3 = arena.alloc_str("reused");
        assert_eq!(s3, "reused");
    }
    
    #[test]
    fn test_pooled_string_builder() {
        let mut builder = PooledStringBuilder::new();
        builder.push_str("Hello, ");
        builder.push_str("World!");
        
        let result = builder.finish();
        assert_eq!(result, "Hello, World!");
    }
}