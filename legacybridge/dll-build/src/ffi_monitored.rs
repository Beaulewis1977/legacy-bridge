// Enhanced FFI module with monitoring integration
use crate::ffi::*;
use crate::monitoring::{
    track_function_call, ACTIVE_CONNECTIONS, CONVERSION_COUNTER, CONVERSION_DURATION,
    CONVERSION_ERRORS, MEMORY_USAGE, CPU_USAGE,
};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::time::Instant;
use sysinfo::{System, SystemExt};

/// Monitored version of RTF to Markdown conversion
#[no_mangle]
pub unsafe extern "C" fn legacybridge_rtf_to_markdown_monitored(
    rtf_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    let start = Instant::now();
    let _timer = CONVERSION_DURATION.start_timer();
    
    CONVERSION_COUNTER.inc();
    ACTIVE_CONNECTIONS.inc();
    
    // Update memory usage
    let mut system = System::new_all();
    system.refresh_memory();
    MEMORY_USAGE.set(system.used_memory() as f64);
    
    // Call original function
    let result = legacybridge_rtf_to_markdown(rtf_content, output_buffer, output_length);
    
    // Track metrics
    let success = result == 0;
    if !success {
        CONVERSION_ERRORS.inc();
    }
    
    track_function_call("legacybridge_rtf_to_markdown", start.elapsed(), success);
    
    ACTIVE_CONNECTIONS.dec();
    result
}

/// Monitored version of Markdown to RTF conversion
#[no_mangle]
pub unsafe extern "C" fn legacybridge_markdown_to_rtf_monitored(
    markdown_content: *const c_char,
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    let start = Instant::now();
    let _timer = CONVERSION_DURATION.start_timer();
    
    CONVERSION_COUNTER.inc();
    ACTIVE_CONNECTIONS.inc();
    
    // Update system metrics
    let mut system = System::new_all();
    system.refresh_all();
    MEMORY_USAGE.set(system.used_memory() as f64);
    CPU_USAGE.set(system.global_cpu_info().cpu_usage() as f64);
    
    // Call original function
    let result = legacybridge_markdown_to_rtf(markdown_content, output_buffer, output_length);
    
    // Track metrics
    let success = result == 0;
    if !success {
        CONVERSION_ERRORS.inc();
    }
    
    track_function_call("legacybridge_markdown_to_rtf", start.elapsed(), success);
    
    ACTIVE_CONNECTIONS.dec();
    result
}

/// Get current performance metrics as JSON
#[no_mangle]
pub unsafe extern "C" fn legacybridge_get_performance_metrics(
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    use crate::monitoring::get_performance_metrics;
    
    let metrics = get_performance_metrics();
    let json = match serde_json::to_string(&metrics) {
        Ok(s) => s,
        Err(_) => return FFIErrorCode::ConversionError as c_int,
    };
    
    let c_str = match CString::new(json.clone()) {
        Ok(s) => s.into_raw(),
        Err(_) => return FFIErrorCode::AllocationError as c_int,
    };
    
    *output_buffer = c_str;
    *output_length = json.len() as c_int;
    FFIErrorCode::Success as c_int
}

/// Get function call statistics as JSON
#[no_mangle]
pub unsafe extern "C" fn legacybridge_get_function_stats(
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    use crate::monitoring::get_function_stats;
    
    let stats = get_function_stats();
    let json = match serde_json::to_string(&stats) {
        Ok(s) => s,
        Err(_) => return FFIErrorCode::ConversionError as c_int,
    };
    
    let c_str = match CString::new(json.clone()) {
        Ok(s) => s.into_raw(),
        Err(_) => return FFIErrorCode::AllocationError as c_int,
    };
    
    *output_buffer = c_str;
    *output_length = json.len() as c_int;
    FFIErrorCode::Success as c_int
}

/// Get system health status as JSON
#[no_mangle]
pub unsafe extern "C" fn legacybridge_get_system_health(
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    use crate::monitoring::get_system_health;
    
    let health = get_system_health();
    let json = match serde_json::to_string(&health) {
        Ok(s) => s,
        Err(_) => return FFIErrorCode::ConversionError as c_int,
    };
    
    let c_str = match CString::new(json.clone()) {
        Ok(s) => s.into_raw(),
        Err(_) => return FFIErrorCode::AllocationError as c_int,
    };
    
    *output_buffer = c_str;
    *output_length = json.len() as c_int;
    FFIErrorCode::Success as c_int
}

/// Export metrics in Prometheus format
#[no_mangle]
pub unsafe extern "C" fn legacybridge_export_prometheus_metrics(
    output_buffer: *mut *mut c_char,
    output_length: *mut c_int,
) -> c_int {
    use crate::monitoring::export_metrics;
    
    let metrics = export_metrics();
    let c_str = match CString::new(metrics.clone()) {
        Ok(s) => s.into_raw(),
        Err(_) => return FFIErrorCode::AllocationError as c_int,
    };
    
    *output_buffer = c_str;
    *output_length = metrics.len() as c_int;
    FFIErrorCode::Success as c_int
}