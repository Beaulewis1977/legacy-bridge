// Optimized String Interner with LRU Cache and Bounded Memory
//
// Key features:
// 1. LRU eviction policy to bound memory usage
// 2. Metrics tracking for hit/miss rates
// 3. Periodic cleanup to prevent unbounded growth
// 4. Arc<str> for zero-copy string sharing
// 5. Configurable size limits

use lru::LruCache;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::num::NonZeroUsize;
use parking_lot::RwLock;

/// Maximum number of strings to cache
const MAX_CACHE_SIZE: usize = 10_000;

/// Threshold for triggering cache cleanup
const CACHE_CLEANUP_THRESHOLD: usize = 8_000;

/// Minimum string length to intern (shorter strings are not worth the overhead)
const MIN_INTERN_LENGTH: usize = 8;

/// Maximum total memory usage for interned strings (100MB)
const MAX_MEMORY_BYTES: usize = 100 * 1024 * 1024;

/// Optimized string interner with LRU eviction
pub struct OptimizedStringInterner {
    /// LRU cache for interned strings
    cache: RwLock<LruCache<String, Arc<str>>>,
    /// Cache hit counter
    hit_count: AtomicU64,
    /// Cache miss counter
    miss_count: AtomicU64,
    /// Total memory used by interned strings
    memory_usage: AtomicUsize,
    /// Number of strings evicted
    eviction_count: AtomicU64,
}

impl OptimizedStringInterner {
    /// Create a new string interner with default settings
    pub fn new() -> Self {
        Self::with_capacity(MAX_CACHE_SIZE)
    }
    
    /// Create a new string interner with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let cache_size = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(1).unwrap());
        
        Self {
            cache: RwLock::new(LruCache::new(cache_size)),
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
            memory_usage: AtomicUsize::new(0),
            eviction_count: AtomicU64::new(0),
        }
    }
    
    /// Intern a string, returning an Arc<str> for efficient sharing
    pub fn intern(&self, text: &str) -> Arc<str> {
        // Short strings aren't worth interning
        if text.len() < MIN_INTERN_LENGTH {
            return Arc::from(text);
        }
        
        // Try to get from cache (read lock)
        {
            let mut cache = self.cache.write();
            if let Some(interned) = cache.get(text) {
                self.hit_count.fetch_add(1, Ordering::Relaxed);
                return Arc::clone(interned);
            }
        }
        
        // Cache miss - create new interned string
        self.miss_count.fetch_add(1, Ordering::Relaxed);
        
        // Check memory limit before adding
        let text_size = text.len();
        let current_memory = self.memory_usage.load(Ordering::Relaxed);
        
        if current_memory + text_size > MAX_MEMORY_BYTES {
            // Evict entries until we have space
            self.evict_until_memory_available(text_size);
        }
        
        // Create Arc<str> and add to cache
        let interned: Arc<str> = Arc::from(text);
        let result = Arc::clone(&interned);
        
        {
            let mut cache = self.cache.write();
            
            // Check if we need to evict based on count
            if cache.len() >= CACHE_CLEANUP_THRESHOLD {
                self.cleanup_cache(&mut cache);
            }
            
            // Add to cache (may evict LRU entry)
            if let Some((evicted_key, evicted_value)) = cache.push(text.to_string(), interned) {
                // Update memory usage for evicted entry
                let evicted_size = evicted_key.len() + evicted_value.len();
                self.memory_usage.fetch_sub(evicted_size, Ordering::Relaxed);
                self.eviction_count.fetch_add(1, Ordering::Relaxed);
            }
        }
        
        // Update memory usage
        self.memory_usage.fetch_add(text_size * 2, Ordering::Relaxed); // Key + value
        
        result
    }
    
    /// Evict entries until enough memory is available
    fn evict_until_memory_available(&self, required_size: usize) {
        let mut cache = self.cache.write();
        let mut current_memory = self.memory_usage.load(Ordering::Relaxed);
        
        while current_memory + required_size > MAX_MEMORY_BYTES && cache.len() > 0 {
            // Pop LRU entry
            if let Some((key, value)) = cache.pop_lru() {
                let freed_size = key.len() + value.len();
                current_memory = self.memory_usage.fetch_sub(freed_size, Ordering::Relaxed) - freed_size;
                self.eviction_count.fetch_add(1, Ordering::Relaxed);
            } else {
                break;
            }
        }
    }
    
    /// Clean up cache by removing least recently used entries
    fn cleanup_cache(&self, cache: &mut LruCache<String, Arc<str>>) {
        let target_size = MAX_CACHE_SIZE * 3 / 4; // Keep 75% after cleanup
        let mut evicted = 0;
        
        while cache.len() > target_size {
            if let Some((key, value)) = cache.pop_lru() {
                let freed_size = key.len() + value.len();
                self.memory_usage.fetch_sub(freed_size, Ordering::Relaxed);
                evicted += 1;
            } else {
                break;
            }
        }
        
        self.eviction_count.fetch_add(evicted, Ordering::Relaxed);
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> InternerStats {
        let cache = self.cache.read();
        let total_requests = self.hit_count.load(Ordering::Relaxed) + self.miss_count.load(Ordering::Relaxed);
        let hit_rate = if total_requests > 0 {
            (self.hit_count.load(Ordering::Relaxed) as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        InternerStats {
            cache_size: cache.len(),
            hit_count: self.hit_count.load(Ordering::Relaxed),
            miss_count: self.miss_count.load(Ordering::Relaxed),
            hit_rate,
            memory_usage_bytes: self.memory_usage.load(Ordering::Relaxed),
            eviction_count: self.eviction_count.load(Ordering::Relaxed),
        }
    }
    
    /// Clear the cache and reset statistics
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();
        
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
        self.memory_usage.store(0, Ordering::Relaxed);
        self.eviction_count.store(0, Ordering::Relaxed);
    }
    
    /// Intern a string and return String (for backward compatibility)
    pub fn intern_to_string(&self, text: &str) -> String {
        self.intern(text).to_string()
    }
}

/// Statistics for the string interner
#[derive(Debug, Clone)]
pub struct InternerStats {
    pub cache_size: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
    pub memory_usage_bytes: usize,
    pub eviction_count: u64,
}

impl Default for OptimizedStringInterner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_interning() {
        let interner = OptimizedStringInterner::new();
        
        let s1 = interner.intern("Hello, World!");
        let s2 = interner.intern("Hello, World!");
        
        // Should return the same Arc
        assert!(Arc::ptr_eq(&s1, &s2));
        
        let stats = interner.stats();
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 1);
    }
    
    #[test]
    fn test_short_string_not_interned() {
        let interner = OptimizedStringInterner::new();
        
        let s1 = interner.intern("short");
        let s2 = interner.intern("short");
        
        // Short strings should not be interned
        assert!(!Arc::ptr_eq(&s1, &s2));
        
        let stats = interner.stats();
        assert_eq!(stats.cache_size, 0);
    }
    
    #[test]
    fn test_lru_eviction() {
        let interner = OptimizedStringInterner::with_capacity(3);
        
        // Fill cache
        let _s1 = interner.intern("First long string");
        let _s2 = interner.intern("Second long string");
        let _s3 = interner.intern("Third long string");
        
        // This should evict the first string
        let _s4 = interner.intern("Fourth long string");
        
        // First string should be evicted, so this is a miss
        let _s1_again = interner.intern("First long string");
        
        let stats = interner.stats();
        assert_eq!(stats.eviction_count, 1);
        assert_eq!(stats.cache_size, 3);
    }
    
    #[test]
    fn test_memory_limit() {
        let interner = OptimizedStringInterner::with_capacity(1000);
        
        // Create strings that will exceed memory limit
        let large_string = "x".repeat(1024); // 1KB string
        
        // Add many large strings
        for i in 0..200 {
            let unique_string = format!("{}{}", large_string, i);
            interner.intern(&unique_string);
        }
        
        let stats = interner.stats();
        
        // Should have evicted some entries to stay under memory limit
        assert!(stats.eviction_count > 0);
        assert!(stats.memory_usage_bytes <= MAX_MEMORY_BYTES);
    }
    
    #[test]
    fn test_hit_rate_calculation() {
        let interner = OptimizedStringInterner::new();
        
        // Create pattern with 80% hits
        for _ in 0..8 {
            interner.intern("Repeated string content");
        }
        interner.intern("Unique string one");
        interner.intern("Unique string two");
        
        let stats = interner.stats();
        assert!(stats.hit_rate >= 70.0 && stats.hit_rate <= 80.0);
    }
}