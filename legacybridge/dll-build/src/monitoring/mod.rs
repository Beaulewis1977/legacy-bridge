// Monitoring and metrics collection module
use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, IntCounter, IntGauge, Registry,
    register_counter_with_registry, register_gauge_with_registry,
    register_histogram_with_registry, register_int_counter_with_registry,
    register_int_gauge_with_registry,
};
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use sysinfo::{System, SystemExt, ProcessExt};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

lazy_static! {
    pub static ref METRICS_REGISTRY: Registry = Registry::new();
    
    // Conversion metrics
    pub static ref CONVERSION_COUNTER: IntCounter = register_int_counter_with_registry!(
        "legacybridge_conversions_total",
        "Total number of conversions processed",
        METRICS_REGISTRY
    ).unwrap();
    
    pub static ref CONVERSION_DURATION: Histogram = register_histogram_with_registry!(
        HistogramOpts::new(
            "legacybridge_conversion_duration_seconds",
            "Time spent processing conversions"
        ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0]),
        METRICS_REGISTRY
    ).unwrap();
    
    pub static ref CONVERSION_ERRORS: IntCounter = register_int_counter_with_registry!(
        "legacybridge_conversion_errors_total",
        "Total number of conversion errors",
        METRICS_REGISTRY
    ).unwrap();
    
    // System metrics
    pub static ref ACTIVE_CONNECTIONS: IntGauge = register_int_gauge_with_registry!(
        "legacybridge_active_connections",
        "Number of active DLL connections",
        METRICS_REGISTRY
    ).unwrap();
    
    pub static ref MEMORY_USAGE: Gauge = register_gauge_with_registry!(
        "legacybridge_memory_usage_bytes",
        "Current memory usage in bytes",
        METRICS_REGISTRY
    ).unwrap();
    
    pub static ref CPU_USAGE: Gauge = register_gauge_with_registry!(
        "legacybridge_cpu_usage_percent",
        "Current CPU usage percentage",
        METRICS_REGISTRY
    ).unwrap();
    
    // Build metrics
    pub static ref BUILD_STATUS: IntGauge = register_int_gauge_with_registry!(
        "legacybridge_build_status",
        "Current build status (0=idle, 1=building, 2=success, 3=failed)",
        METRICS_REGISTRY
    ).unwrap();
    
    pub static ref BUILD_DURATION: Histogram = register_histogram_with_registry!(
        HistogramOpts::new(
            "legacybridge_build_duration_seconds",
            "Time spent building DLL"
        ).buckets(vec![1.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0]),
        METRICS_REGISTRY
    ).unwrap();
    
    pub static ref BUILD_PROGRESS: Gauge = register_gauge_with_registry!(
        "legacybridge_build_progress_percent",
        "Current build progress percentage",
        METRICS_REGISTRY
    ).unwrap();
    
    // Function call metrics
    pub static ref FUNCTION_CALLS: Arc<Mutex<HashMap<String, FunctionMetrics>>> = 
        Arc::new(Mutex::new(HashMap::new()));
    
    // System information
    static ref SYSTEM: Mutex<System> = Mutex::new(System::new_all());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetrics {
    pub function_name: String,
    pub call_count: u64,
    pub total_duration_ms: u64,
    pub average_response_time: f64,
    pub error_count: u64,
    pub error_rate: f64,
    pub last_called: DateTime<Utc>,
    pub peak_usage: u64, // calls per minute
    pub status: FunctionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionStatus {
    Active,
    Idle,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStatus {
    pub compilation: BuildState,
    pub progress: f64,
    pub time_elapsed: u64,
    pub estimated_time_remaining: u64,
    pub current_step: String,
    pub errors: Vec<CompilationError>,
    pub warnings: Vec<CompilationWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildState {
    Idle,
    Building,
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationError {
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationWarning {
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub conversions_per_second: f64,
    pub memory_usage: MemoryInfo,
    pub cpu_utilization: f64,
    pub active_connections: i64,
    pub queued_jobs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub used: u64,
    pub total: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: HealthStatus,
    pub uptime: u64,
    pub version: String,
    pub environment: Environment,
    pub last_error: Option<SystemError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemError {
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

// Monitoring functions
pub fn track_function_call(function_name: &str, duration: Duration, success: bool) {
    let mut functions = FUNCTION_CALLS.lock().unwrap();
    let metrics = functions.entry(function_name.to_string()).or_insert(FunctionMetrics {
        function_name: function_name.to_string(),
        call_count: 0,
        total_duration_ms: 0,
        average_response_time: 0.0,
        error_count: 0,
        error_rate: 0.0,
        last_called: Utc::now(),
        peak_usage: 0,
        status: FunctionStatus::Active,
    });
    
    metrics.call_count += 1;
    metrics.total_duration_ms += duration.as_millis() as u64;
    metrics.average_response_time = metrics.total_duration_ms as f64 / metrics.call_count as f64;
    
    if !success {
        metrics.error_count += 1;
    }
    
    metrics.error_rate = (metrics.error_count as f64 / metrics.call_count as f64) * 100.0;
    metrics.last_called = Utc::now();
    metrics.status = if metrics.error_rate > 50.0 {
        FunctionStatus::Error
    } else {
        FunctionStatus::Active
    };
}

pub fn get_performance_metrics() -> PerformanceMetrics {
    let mut system = SYSTEM.lock().unwrap();
    system.refresh_all();
    
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    let memory_percentage = (used_memory as f64 / total_memory as f64) * 100.0;
    
    let cpu_usage = system.global_cpu_info().cpu_usage();
    
    PerformanceMetrics {
        conversions_per_second: calculate_conversions_per_second(),
        memory_usage: MemoryInfo {
            used: used_memory,
            total: total_memory,
            percentage: memory_percentage,
        },
        cpu_utilization: cpu_usage as f64,
        active_connections: ACTIVE_CONNECTIONS.get(),
        queued_jobs: 0, // TODO: Implement job queue
    }
}

pub fn get_function_stats() -> Vec<FunctionMetrics> {
    let functions = FUNCTION_CALLS.lock().unwrap();
    functions.values().cloned().collect()
}

pub fn get_system_health() -> SystemHealth {
    let mut system = SYSTEM.lock().unwrap();
    system.refresh_all();
    
    let uptime = system.uptime();
    
    SystemHealth {
        status: determine_health_status(),
        uptime,
        version: "1.0.0".to_string(),
        environment: Environment::Production,
        last_error: None,
    }
}

fn calculate_conversions_per_second() -> f64 {
    // Simple calculation - in production, this would track over a time window
    static LAST_COUNT: Mutex<(i64, Instant)> = Mutex::new((0, Instant::now()));
    
    let current_count = CONVERSION_COUNTER.get();
    let mut last = LAST_COUNT.lock().unwrap();
    
    let elapsed = last.1.elapsed().as_secs_f64();
    if elapsed > 0.0 {
        let rate = (current_count - last.0) as f64 / elapsed;
        *last = (current_count, Instant::now());
        rate
    } else {
        0.0
    }
}

fn determine_health_status() -> HealthStatus {
    let error_rate = get_overall_error_rate();
    let cpu_usage = CPU_USAGE.get();
    let memory_usage = MEMORY_USAGE.get();
    
    if error_rate > 10.0 || cpu_usage > 90.0 || memory_usage > 90.0 {
        HealthStatus::Critical
    } else if error_rate > 5.0 || cpu_usage > 70.0 || memory_usage > 70.0 {
        HealthStatus::Warning
    } else {
        HealthStatus::Healthy
    }
}

fn get_overall_error_rate() -> f64 {
    let total_conversions = CONVERSION_COUNTER.get();
    let total_errors = CONVERSION_ERRORS.get();
    
    if total_conversions > 0 {
        (total_errors as f64 / total_conversions as f64) * 100.0
    } else {
        0.0
    }
}

// Build monitoring
pub fn start_build_monitoring() {
    BUILD_STATUS.set(1); // Building
    BUILD_PROGRESS.set(0.0);
}

pub fn update_build_progress(progress: f64, step: &str) {
    BUILD_PROGRESS.set(progress);
    // In production, we'd also update the current step
}

pub fn complete_build_monitoring(success: bool) {
    BUILD_STATUS.set(if success { 2 } else { 3 });
    BUILD_PROGRESS.set(100.0);
}

// Export metrics in Prometheus format
pub fn export_metrics() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = METRICS_REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}