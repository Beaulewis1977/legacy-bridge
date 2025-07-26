// Memory Pool Monitoring - Track pool utilization and performance metrics

use super::memory_pools::{CONVERSION_POOLS, PoolStats};
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

/// Global pool monitoring statistics
pub struct PoolMonitor {
    // Allocation metrics
    allocations_served_from_pool: AtomicUsize,
    allocations_created_new: AtomicUsize,
    
    // Time metrics
    total_acquire_time_ns: AtomicU64,
    total_release_time_ns: AtomicU64,
    
    // Size metrics
    peak_memory_usage: AtomicUsize,
    current_memory_usage: AtomicUsize,
    
    // Hit rate tracking
    hit_rate_window: RwLock<HitRateWindow>,
}

struct HitRateWindow {
    window_start: Instant,
    hits: usize,
    misses: usize,
}

impl PoolMonitor {
    pub fn new() -> Self {
        Self {
            allocations_served_from_pool: AtomicUsize::new(0),
            allocations_created_new: AtomicUsize::new(0),
            total_acquire_time_ns: AtomicU64::new(0),
            total_release_time_ns: AtomicU64::new(0),
            peak_memory_usage: AtomicUsize::new(0),
            current_memory_usage: AtomicUsize::new(0),
            hit_rate_window: RwLock::new(HitRateWindow {
                window_start: Instant::now(),
                hits: 0,
                misses: 0,
            }),
        }
    }
    
    /// Record a pool hit (object served from pool)
    pub fn record_hit(&self) {
        self.allocations_served_from_pool.fetch_add(1, Ordering::Relaxed);
        
        let mut window = self.hit_rate_window.write();
        window.hits += 1;
        
        // Reset window every minute
        if window.window_start.elapsed() > Duration::from_secs(60) {
            window.window_start = Instant::now();
            window.hits = 1;
            window.misses = 0;
        }
    }
    
    /// Record a pool miss (new object created)
    pub fn record_miss(&self) {
        self.allocations_created_new.fetch_add(1, Ordering::Relaxed);
        
        let mut window = self.hit_rate_window.write();
        window.misses += 1;
        
        // Reset window every minute
        if window.window_start.elapsed() > Duration::from_secs(60) {
            window.window_start = Instant::now();
            window.hits = 0;
            window.misses = 1;
        }
    }
    
    /// Record acquire timing
    pub fn record_acquire_time(&self, duration: Duration) {
        self.total_acquire_time_ns.fetch_add(
            duration.as_nanos() as u64,
            Ordering::Relaxed
        );
    }
    
    /// Record release timing
    pub fn record_release_time(&self, duration: Duration) {
        self.total_release_time_ns.fetch_add(
            duration.as_nanos() as u64,
            Ordering::Relaxed
        );
    }
    
    /// Update memory usage
    pub fn update_memory_usage(&self, bytes: usize) {
        self.current_memory_usage.store(bytes, Ordering::Relaxed);
        
        // Update peak if necessary
        let mut peak = self.peak_memory_usage.load(Ordering::Relaxed);
        while bytes > peak {
            match self.peak_memory_usage.compare_exchange_weak(
                peak,
                bytes,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(current) => peak = current,
            }
        }
    }
    
    /// Get current monitoring statistics
    pub fn get_stats(&self) -> MonitoringStats {
        let pool_stats = CONVERSION_POOLS.get_stats();
        let window = self.hit_rate_window.read();
        
        let total_requests = window.hits + window.misses;
        let hit_rate = if total_requests > 0 {
            (window.hits as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        MonitoringStats {
            allocations_from_pool: self.allocations_served_from_pool.load(Ordering::Relaxed),
            allocations_new: self.allocations_created_new.load(Ordering::Relaxed),
            hit_rate_percent: hit_rate,
            avg_acquire_time_ns: self.calculate_avg_acquire_time(),
            avg_release_time_ns: self.calculate_avg_release_time(),
            peak_memory_bytes: self.peak_memory_usage.load(Ordering::Relaxed),
            current_memory_bytes: self.current_memory_usage.load(Ordering::Relaxed),
            pool_stats,
        }
    }
    
    fn calculate_avg_acquire_time(&self) -> u64 {
        let total_time = self.total_acquire_time_ns.load(Ordering::Relaxed);
        let total_ops = self.allocations_served_from_pool.load(Ordering::Relaxed) 
            + self.allocations_created_new.load(Ordering::Relaxed);
        
        if total_ops > 0 {
            total_time / total_ops as u64
        } else {
            0
        }
    }
    
    fn calculate_avg_release_time(&self) -> u64 {
        let total_time = self.total_release_time_ns.load(Ordering::Relaxed);
        let total_ops = self.allocations_served_from_pool.load(Ordering::Relaxed);
        
        if total_ops > 0 {
            total_time / total_ops as u64
        } else {
            0
        }
    }
    
    /// Reset all statistics
    pub fn reset(&self) {
        self.allocations_served_from_pool.store(0, Ordering::Relaxed);
        self.allocations_created_new.store(0, Ordering::Relaxed);
        self.total_acquire_time_ns.store(0, Ordering::Relaxed);
        self.total_release_time_ns.store(0, Ordering::Relaxed);
        self.current_memory_usage.store(0, Ordering::Relaxed);
        
        let mut window = self.hit_rate_window.write();
        window.window_start = Instant::now();
        window.hits = 0;
        window.misses = 0;
    }
}

/// Monitoring statistics snapshot
#[derive(Debug, Clone)]
pub struct MonitoringStats {
    pub allocations_from_pool: usize,
    pub allocations_new: usize,
    pub hit_rate_percent: f64,
    pub avg_acquire_time_ns: u64,
    pub avg_release_time_ns: u64,
    pub peak_memory_bytes: usize,
    pub current_memory_bytes: usize,
    pub pool_stats: PoolStats,
}

impl MonitoringStats {
    /// Calculate allocation overhead reduction percentage
    pub fn allocation_overhead_reduction(&self) -> f64 {
        let total = self.allocations_from_pool + self.allocations_new;
        if total > 0 {
            (self.allocations_from_pool as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
    
    /// Format stats for reporting
    pub fn format_report(&self) -> String {
        format!(
            r#"Memory Pool Performance Report
================================
Allocation Statistics:
  - Served from pool: {} ({:.1}% hit rate)
  - New allocations: {}
  - Overhead reduction: {:.1}%

Timing Metrics:
  - Avg acquire time: {} ns
  - Avg release time: {} ns

Memory Usage:
  - Current: {} KB
  - Peak: {} KB
  - Pooled objects: {}
  - Est. pool memory: {} KB

Pool Details:
  - String pools: {} + {} (large + small)
  - Buffer pools: {} byte + {} token
  - Node pools: {} + {} (nodes + tables)
"#,
            self.allocations_from_pool,
            self.hit_rate_percent,
            self.allocations_new,
            self.allocation_overhead_reduction(),
            self.avg_acquire_time_ns,
            self.avg_release_time_ns,
            self.current_memory_bytes / 1024,
            self.peak_memory_bytes / 1024,
            self.pool_stats.total_pooled_objects(),
            self.pool_stats.estimated_memory_usage() / 1024,
            self.pool_stats.string_pool_size,
            self.pool_stats.small_string_pool_size,
            self.pool_stats.buffer_pool_size,
            self.pool_stats.token_buffer_pool_size,
            self.pool_stats.node_vec_pool_size,
            self.pool_stats.table_row_pool_size + self.pool_stats.table_cell_pool_size,
        )
    }
}

/// Global pool monitor instance
pub static POOL_MONITOR: once_cell::sync::Lazy<Arc<PoolMonitor>> = 
    once_cell::sync::Lazy::new(|| Arc::new(PoolMonitor::new()));

/// Get current monitoring statistics
pub fn get_monitoring_stats() -> MonitoringStats {
    POOL_MONITOR.get_stats()
}

/// Reset monitoring statistics
pub fn reset_monitoring_stats() {
    POOL_MONITOR.reset();
}

/// Print monitoring report
pub fn print_monitoring_report() {
    let stats = get_monitoring_stats();
    println!("{}", stats.format_report());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_monitoring_stats() {
        let monitor = PoolMonitor::new();
        
        // Record some operations
        monitor.record_hit();
        monitor.record_hit();
        monitor.record_miss();
        
        monitor.record_acquire_time(Duration::from_nanos(1000));
        monitor.record_release_time(Duration::from_nanos(500));
        
        monitor.update_memory_usage(1024 * 1024);
        
        let stats = monitor.get_stats();
        assert_eq!(stats.allocations_from_pool, 2);
        assert_eq!(stats.allocations_new, 1);
        assert!(stats.hit_rate_percent > 60.0);
        assert_eq!(stats.current_memory_bytes, 1024 * 1024);
    }
    
    #[test]
    fn test_allocation_overhead_calculation() {
        let monitor = PoolMonitor::new();
        
        // 80% hit rate scenario
        for _ in 0..80 {
            monitor.record_hit();
        }
        for _ in 0..20 {
            monitor.record_miss();
        }
        
        let stats = monitor.get_stats();
        assert!((stats.allocation_overhead_reduction() - 80.0).abs() < 0.1);
    }
}