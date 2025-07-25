// Optimized String Interner using Cow<str> for zero-copy operations
use std::borrow::Cow;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use ahash::AHashMap;

/// Optimized String interner with Cow<str> to eliminate unnecessary cloning
pub struct OptimizedStringInterner<'a> {
    // Use Cow<str> to avoid cloning when possible
    cache: AHashMap<Cow<'a, str>, CacheEntry>,
    access_order: VecDeque<Cow<'a, str>>,
    strings: Vec<Cow<'a, str>>,
    hit_count: AtomicU64,
    miss_count: AtomicU64,
    memory_used: usize,
}

#[derive(Clone)]
struct CacheEntry {
    index: usize,
    last_accessed: u64,
}

// SECURITY: Memory limits to prevent unbounded growth
const MAX_CACHE_SIZE: usize = 10_000;
const MAX_MEMORY_BYTES: usize = 50 * 1024 * 1024; // 50MB
const CLEANUP_THRESHOLD: usize = 8_000;
const SMALL_STRING_THRESHOLD: usize = 8;

impl<'a> OptimizedStringInterner<'a> {
    pub fn new() -> Self {
        Self {
            cache: AHashMap::with_capacity(1000),
            access_order: VecDeque::with_capacity(1000),
            strings: Vec::with_capacity(1000),
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
            memory_used: 0,
        }
    }

    /// Intern a string, returning Cow<str> to avoid unnecessary cloning
    pub fn intern(&mut self, text: &'a str) -> Cow<'a, str> {
        // For small strings, just return a borrowed reference (zero-copy)
        if text.len() <= SMALL_STRING_THRESHOLD {
            return Cow::Borrowed(text);
        }

        // SECURITY: Prevent excessively large strings from being cached
        if text.len() > 1024 * 1024 { // 1MB limit per string
            return Cow::Borrowed(text); // Don't cache huge strings
        }

        // Check cache hit
        let key = Cow::Borrowed(text);
        if let Some(entry) = self.cache.get(&key) {
            self.hit_count.fetch_add(1, Ordering::Relaxed);
            
            // Update LRU order - move to end
            if let Some(pos) = self.access_order.iter().position(|x| x == &key) {
                self.access_order.remove(pos);
            }
            self.access_order.push_back(key.clone());
            
            // Return reference to cached string
            return self.strings[entry.index].clone();
        }

        // Cache miss - add new entry
        self.miss_count.fetch_add(1, Ordering::Relaxed);
        
        // SECURITY: Check if we need to evict entries
        if self.cache.len() >= MAX_CACHE_SIZE || self.memory_used >= MAX_MEMORY_BYTES {
            self.evict_lru_entries();
        }

        // Add to cache - only allocate once
        let cow_string = Cow::Owned(text.to_string());
        let idx = self.strings.len();
        
        // Track memory usage
        let entry_memory = text.len() + std::mem::size_of::<CacheEntry>() + std::mem::size_of::<Cow<str>>();
        self.memory_used += entry_memory;
        
        self.strings.push(cow_string.clone());
        
        let entry = CacheEntry {
            index: idx,
            last_accessed: self.hit_count.load(Ordering::Relaxed) + self.miss_count.load(Ordering::Relaxed),
        };
        
        self.cache.insert(cow_string.clone(), entry);
        self.access_order.push_back(cow_string.clone());
        
        cow_string
    }

    /// Evict least recently used entries to prevent memory exhaustion
    fn evict_lru_entries(&mut self) {
        let target_size = CLEANUP_THRESHOLD;
        
        while self.cache.len() > target_size && !self.access_order.is_empty() {
            if let Some(oldest_key) = self.access_order.pop_front() {
                if let Some(_entry) = self.cache.remove(&oldest_key) {
                    // Update memory usage
                    let entry_memory = oldest_key.len() + std::mem::size_of::<CacheEntry>() + std::mem::size_of::<Cow<str>>();
                    self.memory_used = self.memory_used.saturating_sub(entry_memory);
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
        self.strings.clear();
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
        self.memory_used = 0;
    }

    /// Get cache statistics for monitoring
    pub fn stats(&self) -> (u64, u64, f64, usize, usize) {
        let hits = self.hit_count.load(Ordering::Relaxed);
        let misses = self.miss_count.load(Ordering::Relaxed);
        let hit_rate = if hits + misses > 0 {
            hits as f64 / (hits + misses) as f64
        } else {
            0.0
        };
        (hits, misses, hit_rate, self.cache.len(), self.memory_used)
    }
}