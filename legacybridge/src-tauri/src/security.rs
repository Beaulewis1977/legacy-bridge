use tauri::http::header::{HeaderMap, HeaderValue, CONTENT_SECURITY_POLICY};
use std::time::Duration;

/// Security configuration constants
pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
pub const MAX_REQUEST_SIZE: usize = 10 * 1024 * 1024; // 10MB
pub const PARSING_TIMEOUT: Duration = Duration::from_secs(30);
pub const MAX_NESTING_DEPTH: usize = 50;
pub const MAX_TEXT_SIZE: usize = 1_000_000; // 1MB

/// Rate limiting configuration
pub const RATE_LIMIT_PER_SECOND: u64 = 10;
pub const RATE_LIMIT_BURST_SIZE: u32 = 20;

/// Allowed file extensions for RTF conversion
pub const ALLOWED_EXTENSIONS: &[&str] = &["rtf", "md", "txt"];

/// Security headers configuration
pub fn configure_security_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    
    // Content Security Policy - Already set in tauri.conf.json but can be reinforced here
    headers.insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline' 'unsafe-eval' https://localhost:*; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https://localhost:*; \
             font-src 'self'; \
             connect-src 'self' https://localhost:* http://localhost:*; \
             media-src 'self'; \
             object-src 'none'; \
             base-uri 'self'; \
             form-action 'self'; \
             frame-ancestors 'none'; \
             frame-src 'none'; \
             worker-src 'none'"
        ),
    );
    
    // X-Content-Type-Options
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    
    // X-Frame-Options
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY"),
    );
    
    // X-XSS-Protection
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    
    // Referrer-Policy
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    
    // Permissions-Policy
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static(
            "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()"
        ),
    );
    
    // Strict-Transport-Security (for HTTPS)
    headers.insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    
    headers
}

/// Path validation to prevent directory traversal
pub fn validate_file_path(path: &str) -> Result<std::path::PathBuf, String> {
    use std::path::{Path, Component};
    
    let path = Path::new(path);
    
    // Check for directory traversal attempts
    for component in path.components() {
        match component {
            Component::ParentDir => {
                return Err("Invalid path: contains parent directory references".to_string());
            }
            Component::Prefix(_) | Component::RootDir => {
                return Err("Invalid path: absolute paths not allowed".to_string());
            }
            _ => {}
        }
    }
    
    // Check file extension
    if let Some(extension) = path.extension() {
        let ext_str = extension.to_str().unwrap_or("").to_lowercase();
        if !ALLOWED_EXTENSIONS.contains(&ext_str.as_str()) {
            return Err(format!("Invalid file extension: {}", ext_str));
        }
    } else {
        return Err("File must have an extension".to_string());
    }
    
    Ok(path.to_path_buf())
}

/// Input size validation
pub fn validate_input_size(input: &str) -> Result<(), String> {
    if input.len() > MAX_FILE_SIZE {
        return Err(format!(
            "Input size {} exceeds maximum allowed size of {} bytes",
            input.len(),
            MAX_FILE_SIZE
        ));
    }
    Ok(())
}

/// Rate limiter implementation
pub struct RateLimiter {
    last_request_time: std::sync::Mutex<std::time::Instant>,
    request_count: std::sync::Mutex<u64>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            last_request_time: std::sync::Mutex::new(std::time::Instant::now()),
            request_count: std::sync::Mutex::new(0),
        }
    }
    
    pub fn check_rate_limit(&self) -> Result<(), String> {
        let mut last_time = self.last_request_time.lock().unwrap();
        let mut count = self.request_count.lock().unwrap();
        
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(*last_time);
        
        if elapsed >= Duration::from_secs(1) {
            // Reset counter after 1 second
            *last_time = now;
            *count = 1;
            Ok(())
        } else if *count < RATE_LIMIT_PER_SECOND {
            *count += 1;
            Ok(())
        } else {
            Err("Rate limit exceeded. Please try again later.".to_string())
        }
    }
}

/// Security event logging
#[derive(Debug)]
pub struct SecurityEvent {
    pub event_type: SecurityEventType,
    pub details: String,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug)]
pub enum SecurityEventType {
    InvalidPath,
    OversizedInput,
    RateLimitExceeded,
    InvalidFileType,
    ParsingTimeout,
    MemoryLimitExceeded,
}

impl SecurityEvent {
    pub fn log(&self) {
        eprintln!(
            "[SECURITY] {:?} at {:?}: {}",
            self.event_type, self.timestamp, self.details
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_path_validation() {
        // Valid paths
        assert!(validate_file_path("document.rtf").is_ok());
        assert!(validate_file_path("folder/document.md").is_ok());
        
        // Invalid paths
        assert!(validate_file_path("../document.rtf").is_err());
        assert!(validate_file_path("/etc/passwd").is_err());
        assert!(validate_file_path("document.exe").is_err());
        assert!(validate_file_path("document").is_err());
    }
    
    #[test]
    fn test_input_size_validation() {
        let small_input = "x".repeat(1000);
        assert!(validate_input_size(&small_input).is_ok());
        
        let large_input = "x".repeat(MAX_FILE_SIZE + 1);
        assert!(validate_input_size(&large_input).is_err());
    }
    
    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new();
        
        // First few requests should succeed
        for _ in 0..RATE_LIMIT_PER_SECOND {
            assert!(limiter.check_rate_limit().is_ok());
        }
        
        // Next request should fail
        assert!(limiter.check_rate_limit().is_err());
    }
}