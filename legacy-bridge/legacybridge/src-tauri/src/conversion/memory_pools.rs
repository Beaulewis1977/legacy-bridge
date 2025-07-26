// Memory Pool Integration for Conversion Pipeline
// Provides thread-safe memory pools for high-frequency allocations in conversion operations

use crate::conversion::types::{RtfNode, RtfToken, TableRow, TableCell};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

// Re-export from main memory pool module
use crate::memory_pool_optimization::{ObjectPool, PooledObject};

/// Specialized memory pools for conversion operations
pub struct ConversionMemoryPools {
    // String pools for text content
    pub string_pool: Arc<ObjectPool<String>>,
    pub small_string_pool: Arc<ObjectPool<String>>,
    
    // Buffer pools for parsing and generation
    pub buffer_pool: Arc<ObjectPool<Vec<u8>>>,
    pub token_buffer_pool: Arc<ObjectPool<Vec<RtfToken>>>,
    
    // Node pools for document structure
    pub node_vec_pool: Arc<ObjectPool<Vec<RtfNode>>>,
    pub table_row_pool: Arc<ObjectPool<Vec<TableRow>>>,
    pub table_cell_pool: Arc<ObjectPool<Vec<TableCell>>>,
    
    // Result pools for output
    pub result_string_pool: Arc<ObjectPool<String>>,
}

impl ConversionMemoryPools {
    /// Create a new set of conversion memory pools
    pub fn new() -> Self {
        Self {
            // Large strings (documents, paragraphs)
            string_pool: Arc::new(ObjectPool::with_reset(
                256,  // Pool size
                || String::with_capacity(1024),
                |s| s.clear(),
            )),
            
            // Small strings (tokens, attributes)
            small_string_pool: Arc::new(ObjectPool::with_reset(
                512,
                || String::with_capacity(64),
                |s| s.clear(),
            )),
            
            // Byte buffers for raw data
            buffer_pool: Arc::new(ObjectPool::with_reset(
                128,
                || Vec::with_capacity(4096),
                |v| v.clear(),
            )),
            
            // Token buffers for parsing
            token_buffer_pool: Arc::new(ObjectPool::with_reset(
                64,
                || Vec::with_capacity(1000),
                |v| v.clear(),
            )),
            
            // Node vectors for document structure
            node_vec_pool: Arc::new(ObjectPool::with_reset(
                128,
                || Vec::with_capacity(100),
                |v| v.clear(),
            )),
            
            // Table structure pools
            table_row_pool: Arc::new(ObjectPool::with_reset(
                32,
                || Vec::with_capacity(10),
                |v| v.clear(),
            )),
            
            table_cell_pool: Arc::new(ObjectPool::with_reset(
                64,
                || Vec::with_capacity(5),
                |v| v.clear(),
            )),
            
            // Result string pool for output
            result_string_pool: Arc::new(ObjectPool::with_reset(
                64,
                || String::with_capacity(4096),
                |s| s.clear(),
            )),
        }
    }
    
    /// Get a pooled string buffer with specified capacity
    pub fn get_string_buffer(&self, capacity: usize) -> PooledObject<String> {
        if capacity <= 64 {
            self.small_string_pool.acquire()
        } else {
            let mut buffer = self.string_pool.acquire();
            if buffer.capacity() < capacity {
                buffer.reserve(capacity - buffer.len());
            }
            buffer
        }
    }
    
    /// Get a pooled byte buffer
    pub fn get_byte_buffer(&self, capacity: usize) -> PooledObject<Vec<u8>> {
        let mut buffer = self.buffer_pool.acquire();
        if buffer.capacity() < capacity {
            buffer.reserve(capacity - buffer.len());
        }
        buffer
    }
    
    /// Get a pooled token vector
    pub fn get_token_buffer(&self) -> PooledObject<Vec<RtfToken>> {
        self.token_buffer_pool.acquire()
    }
    
    /// Get a pooled node vector
    pub fn get_node_vec(&self) -> PooledObject<Vec<RtfNode>> {
        self.node_vec_pool.acquire()
    }
    
    /// Get a pooled result string
    pub fn get_result_string(&self) -> PooledObject<String> {
        self.result_string_pool.acquire()
    }
    
    /// Get pool statistics for monitoring
    pub fn get_stats(&self) -> PoolStats {
        PoolStats {
            string_pool_size: self.string_pool.size(),
            small_string_pool_size: self.small_string_pool.size(),
            buffer_pool_size: self.buffer_pool.size(),
            token_buffer_pool_size: self.token_buffer_pool.size(),
            node_vec_pool_size: self.node_vec_pool.size(),
            table_row_pool_size: self.table_row_pool.size(),
            table_cell_pool_size: self.table_cell_pool.size(),
            result_string_pool_size: self.result_string_pool.size(),
        }
    }
}

/// Pool statistics for monitoring
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub string_pool_size: usize,
    pub small_string_pool_size: usize,
    pub buffer_pool_size: usize,
    pub token_buffer_pool_size: usize,
    pub node_vec_pool_size: usize,
    pub table_row_pool_size: usize,
    pub table_cell_pool_size: usize,
    pub result_string_pool_size: usize,
}

impl PoolStats {
    /// Calculate total objects in all pools
    pub fn total_pooled_objects(&self) -> usize {
        self.string_pool_size
            + self.small_string_pool_size
            + self.buffer_pool_size
            + self.token_buffer_pool_size
            + self.node_vec_pool_size
            + self.table_row_pool_size
            + self.table_cell_pool_size
            + self.result_string_pool_size
    }
    
    /// Estimate memory usage of pooled objects
    pub fn estimated_memory_usage(&self) -> usize {
        // Rough estimates based on typical sizes
        self.string_pool_size * 1024
            + self.small_string_pool_size * 64
            + self.buffer_pool_size * 4096
            + self.token_buffer_pool_size * 1000 * std::mem::size_of::<RtfToken>()
            + self.node_vec_pool_size * 100 * std::mem::size_of::<RtfNode>()
            + self.table_row_pool_size * 10 * std::mem::size_of::<TableRow>()
            + self.table_cell_pool_size * 5 * std::mem::size_of::<TableCell>()
            + self.result_string_pool_size * 4096
    }
}

/// Global conversion memory pools
pub static CONVERSION_POOLS: Lazy<ConversionMemoryPools> = Lazy::new(ConversionMemoryPools::new);

/// Pooled string builder optimized for conversion operations
pub struct PooledStringBuilder {
    buffer: PooledObject<String>,
}

impl PooledStringBuilder {
    /// Create a new pooled string builder
    pub fn new() -> Self {
        Self {
            buffer: CONVERSION_POOLS.get_result_string(),
        }
    }
    
    /// Create with specific capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: CONVERSION_POOLS.get_string_buffer(capacity),
        }
    }
    
    /// Push a string slice
    pub fn push_str(&mut self, s: &str) {
        self.buffer.push_str(s);
    }
    
    /// Push a single character
    pub fn push(&mut self, ch: char) {
        self.buffer.push(ch);
    }
    
    /// Get current length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    
    /// Clear the builder
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
    
    /// Finish and return the string
    pub fn finish(mut self) -> String {
        std::mem::take(&mut *self.buffer)
    }
    
    /// Get a reference to the current string
    pub fn as_str(&self) -> &str {
        &self.buffer
    }
}

/// Node builder using memory pools
pub struct PooledNodeBuilder {
    nodes: PooledObject<Vec<RtfNode>>,
}

impl PooledNodeBuilder {
    /// Create a new pooled node builder
    pub fn new() -> Self {
        Self {
            nodes: CONVERSION_POOLS.get_node_vec(),
        }
    }
    
    /// Add a text node (avoiding allocation when possible)
    pub fn add_text(&mut self, text: String) {
        self.nodes.push(RtfNode::Text(text));
    }
    
    /// Add a text node from &str (will allocate)
    pub fn add_text_str(&mut self, text: &str) {
        self.nodes.push(RtfNode::Text(text.to_string()));
    }
    
    /// Add any node
    pub fn add_node(&mut self, node: RtfNode) {
        self.nodes.push(node);
    }
    
    /// Get current node count
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    
    /// Finish and return the nodes
    pub fn finish(mut self) -> Vec<RtfNode> {
        std::mem::take(&mut *self.nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_string_pool_allocation() {
        let pools = ConversionMemoryPools::new();
        
        // Test small string allocation
        {
            let mut small = pools.get_string_buffer(32);
            small.push_str("Hello, World!");
            assert_eq!(small.as_str(), "Hello, World!");
        }
        
        // Test large string allocation
        {
            let mut large = pools.get_string_buffer(2048);
            large.push_str("Large content...");
            assert!(large.capacity() >= 2048);
        }
        
        // Verify pools are being reused
        let stats1 = pools.get_stats();
        {
            let _temp = pools.get_string_buffer(32);
        }
        let stats2 = pools.get_stats();
        assert!(stats2.small_string_pool_size >= stats1.small_string_pool_size);
    }
    
    #[test]
    fn test_pooled_string_builder() {
        let mut builder = PooledStringBuilder::new();
        builder.push_str("Hello");
        builder.push(' ');
        builder.push_str("World!");
        
        let result = builder.finish();
        assert_eq!(result, "Hello World!");
    }
    
    #[test]
    fn test_pooled_node_builder() {
        let mut builder = PooledNodeBuilder::new();
        builder.add_text_str("Hello");
        builder.add_node(RtfNode::LineBreak);
        builder.add_text_str("World");
        
        let nodes = builder.finish();
        assert_eq!(nodes.len(), 3);
    }
}