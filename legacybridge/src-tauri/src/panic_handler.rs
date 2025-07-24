// Panic Handler Module
// Prevents application crashes and ensures graceful degradation

use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use tracing::{error, warn};

static PANIC_HANDLER_INSTALLED: AtomicBool = AtomicBool::new(false);

/// Install a secure panic handler that prevents crashes
pub fn install_panic_handler() {
    // Only install once
    if PANIC_HANDLER_INSTALLED.compare_exchange(
        false,
        true,
        Ordering::SeqCst,
        Ordering::SeqCst
    ).is_err() {
        return;
    }

    let default_hook = panic::take_hook();
    
    panic::set_hook(Box::new(move |panic_info| {
        // Extract panic information securely
        let thread = thread::current();
        let thread_name = thread.name().unwrap_or("unnamed");
        
        let location = panic_info.location()
            .map(|loc| format!("{}:{}", loc.file(), loc.line()))
            .unwrap_or_else(|| "unknown location".to_string());
        
        // Extract panic message without exposing sensitive details
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            sanitize_panic_message(s)
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            sanitize_panic_message(s)
        } else {
            "Unknown panic".to_string()
        };
        
        // Log panic information internally
        error!(
            thread_name = thread_name,
            location = location,
            message = message,
            "Thread panic occurred"
        );
        
        // Check if this is the main thread
        if thread_name == "main" {
            error!("Main thread panicked - application may be unstable");
            // For main thread panics, we might want to trigger a graceful shutdown
            // but we don't want to expose the panic to users
        }
        
        // Call the default hook for development/debugging
        #[cfg(debug_assertions)]
        default_hook(panic_info);
        
        // In release mode, suppress panic output
        #[cfg(not(debug_assertions))]
        {
            // Don't print anything to stderr in production
            // The error has been logged internally
        }
    }));
}

/// Sanitize panic messages to remove sensitive information
fn sanitize_panic_message(message: &str) -> String {
    // Remove file paths
    let mut sanitized = message.to_string();
    
    // Common patterns to remove
    let patterns = [
        // File paths
        (r"(/[a-zA-Z0-9_\-./]+)+", "[path]"),
        (r"([a-zA-Z]:\\[a-zA-Z0-9_\-.\\ ]+)+", "[path]"),
        // Memory addresses
        (r"0x[0-9a-fA-F]+", "[address]"),
        // IP addresses
        (r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b", "[ip]"),
        // Line numbers in stack traces
        (r":\d+:\d+", ":[line]:[column]"),
    ];
    
    for (pattern, replacement) in patterns.iter() {
        if let Ok(re) = regex::Regex::new(pattern) {
            sanitized = re.replace_all(&sanitized, *replacement).to_string();
        }
    }
    
    // Truncate very long messages
    if sanitized.len() > 200 {
        sanitized.truncate(200);
        sanitized.push_str("...");
    }
    
    sanitized
}

/// Catch panics in a closure and convert to Result
pub fn catch_panic<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce() -> R + panic::UnwindSafe,
{
    match panic::catch_unwind(f) {
        Ok(result) => Ok(result),
        Err(panic_obj) => {
            // Extract panic message
            let message = if let Some(s) = panic_obj.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_obj.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };
            
            // Log the panic
            error!(
                panic_message = %sanitize_panic_message(&message),
                "Panic caught and recovered"
            );
            
            // Return generic error message
            Err("An internal error occurred".to_string())
        }
    }
}

/// Guard that ensures panic handler is installed
pub struct PanicGuard;

impl PanicGuard {
    pub fn new() -> Self {
        install_panic_handler();
        PanicGuard
    }
}

/// Macro to safely execute code that might panic
#[macro_export]
macro_rules! safe_execute {
    ($expr:expr) => {
        $crate::panic_handler::catch_panic(|| $expr)
    };
    ($expr:expr, $default:expr) => {
        match $crate::panic_handler::catch_panic(|| $expr) {
            Ok(val) => val,
            Err(_) => $default,
        }
    };
}

/// Thread spawn wrapper that installs panic handler
pub fn spawn_safe<F, T>(name: &str, f: F) -> thread::JoinHandle<Result<T, String>>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let thread_name = name.to_string();
    
    thread::Builder::new()
        .name(thread_name.clone())
        .spawn(move || {
            // Install panic handler for this thread
            let _guard = PanicGuard::new();
            
            // Catch any panics
            catch_panic(f)
        })
        .unwrap_or_else(|e| {
            error!(
                thread_name = thread_name,
                error = %e,
                "Failed to spawn thread"
            );
            panic!("Critical: Cannot spawn required thread");
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_panic_message_sanitization() {
        let message = "Error at /home/user/project/src/main.rs:42:15";
        let sanitized = sanitize_panic_message(message);
        assert_eq!(sanitized, "Error at [path]:[line]:[column]");
        
        let message = "Memory access at 0xDEADBEEF failed";
        let sanitized = sanitize_panic_message(message);
        assert_eq!(sanitized, "Memory access at [address] failed");
        
        let message = "Connection to 192.168.1.100 refused";
        let sanitized = sanitize_panic_message(message);
        assert_eq!(sanitized, "Connection to [ip] refused");
    }
    
    #[test]
    fn test_catch_panic() {
        // Test successful execution
        let result = catch_panic(|| 42);
        assert_eq!(result, Ok(42));
        
        // Test panic catching
        let result = catch_panic(|| panic!("test panic"));
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "An internal error occurred");
    }
    
    #[test]
    fn test_safe_execute_macro() {
        // Test successful execution
        let result = safe_execute!(1 + 1);
        assert_eq!(result, Ok(2));
        
        // Test with default value
        let result = safe_execute!(panic!("oops"), 42);
        assert_eq!(result, 42);
    }
}