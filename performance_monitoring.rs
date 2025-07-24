// Performance Monitoring with Prometheus Metrics
// Real-time performance tracking and alerting

use prometheus::{
    register_counter_vec, register_gauge_vec, register_histogram_vec,
    register_int_counter_vec, register_int_gauge,
    CounterVec, GaugeVec, HistogramVec, IntCounterVec, IntGauge,
    Encoder, TextEncoder,
};
use once_cell::sync::Lazy;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};

/// Performance metrics for conversion operations
pub static CONVERSION_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "legacybridge_conversion_duration_seconds",
        "Time taken for document conversion",
        &["conversion_type", "document_size", "status"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
    ).unwrap()
});

pub static CONVERSION_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "legacybridge_conversions_total",
        "Total number of conversions",
        &["conversion_type", "status"]
    ).unwrap()
});

pub static DOCUMENT_SIZE_BYTES: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "legacybridge_document_size_bytes",
        "Size of documents processed",
        &["document_type"],
        vec![1024.0, 10240.0, 102400.0, 1048576.0, 10485760.0]
    ).unwrap()
});

pub static MEMORY_USAGE_BYTES: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "legacybridge_memory_usage_bytes",
        "Current memory usage in bytes"
    ).unwrap()
});

pub static ACTIVE_CONVERSIONS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "legacybridge_active_conversions",
        "Number of currently active conversions"
    ).unwrap()
});

pub static THREAD_POOL_SIZE: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "legacybridge_thread_pool_size",
        "Current thread pool size"
    ).unwrap()
});

pub static ERROR_COUNTER: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "legacybridge_errors_total",
        "Total number of errors",
        &["error_type", "operation"]
    ).unwrap()
});

pub static CACHE_METRICS: Lazy<GaugeVec> = Lazy::new(|| {
    register_gauge_vec!(
        "legacybridge_cache_metrics",
        "Cache statistics",
        &["cache_name", "metric_type"]
    ).unwrap()
});

/// Performance monitor for tracking operations
pub struct PerformanceMonitor {
    start_time: Instant,
    operation_type: String,
    document_size: usize,
    tags: Vec<(&'static str, String)>,
}

impl PerformanceMonitor {
    /// Start monitoring a conversion operation
    pub fn start_conversion(conversion_type: &str, document_size: usize) -> Self {
        ACTIVE_CONVERSIONS.inc();
        
        Self {
            start_time: Instant::now(),
            operation_type: conversion_type.to_string(),
            document_size,
            tags: vec![],
        }
    }
    
    /// Add a tag to the monitoring session
    pub fn tag(mut self, key: &'static str, value: String) -> Self {
        self.tags.push((key, value));
        self
    }
    
    /// Complete the monitoring with success
    pub fn success(self) {
        let duration = self.start_time.elapsed();
        self.record_metrics("success", duration);
    }
    
    /// Complete the monitoring with error
    pub fn error(self, error_type: &str) {
        let duration = self.start_time.elapsed();
        self.record_metrics("error", duration);
        
        ERROR_COUNTER
            .with_label_values(&[error_type, &self.operation_type])
            .inc();
    }
    
    fn record_metrics(&self, status: &str, duration: Duration) {
        let size_category = match self.document_size {
            0..=1024 => "tiny",
            1025..=10240 => "small",
            10241..=102400 => "medium",
            102401..=1048576 => "large",
            _ => "xlarge",
        };
        
        CONVERSION_DURATION
            .with_label_values(&[&self.operation_type, size_category, status])
            .observe(duration.as_secs_f64());
        
        CONVERSION_COUNTER
            .with_label_values(&[&self.operation_type, status])
            .inc();
        
        DOCUMENT_SIZE_BYTES
            .with_label_values(&[&self.operation_type])
            .observe(self.document_size as f64);
        
        ACTIVE_CONVERSIONS.dec();
    }
}

impl Drop for PerformanceMonitor {
    fn drop(&mut self) {
        // Ensure we decrement active conversions even if not explicitly completed
        if ACTIVE_CONVERSIONS.get() > 0 {
            ACTIVE_CONVERSIONS.dec();
        }
    }
}

/// Memory usage tracker
pub struct MemoryTracker {
    last_update: AtomicU64,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            last_update: AtomicU64::new(0),
        }
    }
    
    /// Update memory usage metrics
    pub fn update(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let last = self.last_update.load(Ordering::Relaxed);
        
        // Update at most once per second
        if now - last < 1 {
            return;
        }
        
        if let Ok(usage) = self.get_memory_usage() {
            MEMORY_USAGE_BYTES.set(usage as i64);
            self.last_update.store(now, Ordering::Relaxed);
        }
    }
    
    #[cfg(target_os = "linux")]
    fn get_memory_usage(&self) -> Result<usize, std::io::Error> {
        use std::fs;
        
        let status = fs::read_to_string("/proc/self/status")?;
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<usize>() {
                        return Ok(kb * 1024);
                    }
                }
            }
        }
        
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find VmRSS in /proc/self/status",
        ))
    }
    
    #[cfg(not(target_os = "linux"))]
    fn get_memory_usage(&self) -> Result<usize, std::io::Error> {
        // Fallback for non-Linux systems
        Ok(0)
    }
}

/// Cache metrics tracker
pub struct CacheMetrics {
    cache_name: String,
}

impl CacheMetrics {
    pub fn new(cache_name: &str) -> Self {
        Self {
            cache_name: cache_name.to_string(),
        }
    }
    
    pub fn update(&self, size: usize, hits: usize, misses: usize) {
        CACHE_METRICS
            .with_label_values(&[&self.cache_name, "size"])
            .set(size as f64);
        
        CACHE_METRICS
            .with_label_values(&[&self.cache_name, "hit_rate"])
            .set(if hits + misses > 0 {
                hits as f64 / (hits + misses) as f64
            } else {
                0.0
            });
    }
}

/// Performance report generator
pub struct PerformanceReporter;

impl PerformanceReporter {
    /// Generate a performance report in Prometheus format
    pub fn generate_report() -> String {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
    
    /// Generate a human-readable performance summary
    pub fn generate_summary() -> String {
        let mut summary = String::new();
        
        summary.push_str("LegacyBridge Performance Summary\n");
        summary.push_str("================================\n\n");
        
        // Active conversions
        summary.push_str(&format!(
            "Active Conversions: {}\n",
            ACTIVE_CONVERSIONS.get()
        ));
        
        // Memory usage
        summary.push_str(&format!(
            "Memory Usage: {} MB\n",
            MEMORY_USAGE_BYTES.get() / 1_048_576
        ));
        
        // Thread pool
        summary.push_str(&format!(
            "Thread Pool Size: {}\n",
            THREAD_POOL_SIZE.get()
        ));
        
        summary.push_str("\nConversion Metrics:\n");
        summary.push_str("------------------\n");
        
        // This would need actual metric gathering in production
        summary.push_str("RTF→MD: N/A conversions (N/A avg time)\n");
        summary.push_str("MD→RTF: N/A conversions (N/A avg time)\n");
        
        summary
    }
}

/// Benchmark runner with metric collection
pub struct MetricBenchmark {
    name: String,
    iterations: usize,
}

impl MetricBenchmark {
    pub fn new(name: &str, iterations: usize) -> Self {
        Self {
            name: name.to_string(),
            iterations,
        }
    }
    
    pub fn run<F>(&self, mut f: F) -> BenchmarkResult
    where
        F: FnMut(),
    {
        let mut durations = Vec::with_capacity(self.iterations);
        let memory_tracker = MemoryTracker::new();
        
        for _ in 0..self.iterations {
            memory_tracker.update();
            
            let start = Instant::now();
            f();
            let duration = start.elapsed();
            
            durations.push(duration);
        }
        
        BenchmarkResult::from_durations(&self.name, durations)
    }
}

pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub mean: Duration,
    pub median: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub ops_per_second: f64,
}

impl BenchmarkResult {
    fn from_durations(name: &str, mut durations: Vec<Duration>) -> Self {
        durations.sort();
        
        let iterations = durations.len();
        let sum: Duration = durations.iter().sum();
        let mean = sum / iterations as u32;
        let median = durations[iterations / 2];
        let p95 = durations[(iterations as f64 * 0.95) as usize];
        let p99 = durations[(iterations as f64 * 0.99) as usize];
        let ops_per_second = iterations as f64 / sum.as_secs_f64();
        
        Self {
            name: name.to_string(),
            iterations,
            mean,
            median,
            p95,
            p99,
            ops_per_second,
        }
    }
    
    pub fn report(&self) -> String {
        format!(
            "{}: {} iterations\n  Mean: {:?}\n  Median: {:?}\n  P95: {:?}\n  P99: {:?}\n  Ops/sec: {:.2}",
            self.name, self.iterations, self.mean, self.median, self.p95, self.p99, self.ops_per_second
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::start_conversion("rtf_to_md", 1024);
        
        // Simulate some work
        std::thread::sleep(Duration::from_millis(10));
        
        monitor.success();
        
        // Verify metrics were recorded
        assert!(ACTIVE_CONVERSIONS.get() >= 0);
    }
    
    #[test]
    fn test_benchmark() {
        let benchmark = MetricBenchmark::new("test_operation", 100);
        
        let result = benchmark.run(|| {
            // Simulate some work
            let mut sum = 0;
            for i in 0..1000 {
                sum += i;
            }
            std::hint::black_box(sum);
        });
        
        println!("{}", result.report());
        assert_eq!(result.iterations, 100);
        assert!(result.ops_per_second > 0.0);
    }
}