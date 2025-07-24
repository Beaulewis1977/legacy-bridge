# Security Implementation Guide for LegacyBridge

## Table of Contents
- [Overview](#overview)
- [1. Tauri Security Configuration](#1-tauri-security-configuration)
  - [Current Implementation](#current-implementation)
  - [Key Security Features](#key-security-features)
- [2. Security Module Integration](#2-security-module-integration)
- [3. Input Validation Best Practices](#3-input-validation-best-practices)
- [4. XSS Prevention](#4-xss-prevention)
- [5. Monitoring and Logging](#5-monitoring-and-logging)
- [6. Security Checklist](#6-security-checklist)
- [7. Incident Response](#7-incident-response)

## Overview

This guide provides detailed instructions for implementing and maintaining security measures in the LegacyBridge RTF conversion application.

## 1. Tauri Security Configuration

### Current Implementation

The `tauri.conf.json` file has been hardened with the following security measures:

```json
{
  "tauri": {
    "allowlist": {
      "all": false,  // NEVER set to true
      "shell": {
        "all": false,
        "open": false  // Prevents shell execution
      },
      "fs": {
        "all": false,
        "scope": ["$APPDATA/*", "$DOCUMENT/*"]  // Restricted paths
      }
    },
    "security": {
      "csp": "strict CSP policy...",
      "dangerousDisableAssetCspModification": false,
      "freezePrototype": true
    }
  }
}
```

### Key Security Features

1. **No Global Permissions**: Every API must be explicitly allowed
2. **File System Sandboxing**: Access limited to APPDATA and DOCUMENT directories
3. **Shell Execution Blocked**: Prevents command injection attacks
4. **Strict CSP**: Prevents XSS and injection attacks
5. **Prototype Freezing**: Prevents prototype pollution attacks

## 2. Security Module Integration

### File: `src/security.rs`

The security module provides:

- Path validation functions
- Input size validation
- Rate limiting implementation
- Security event logging
- Security headers configuration

### Usage Example:

```rust
use crate::security::{validate_file_path, validate_input_size, RateLimiter};

#[tauri::command]
pub fn secure_command(
    input: String,
    rate_limiter: tauri::State<Arc<RateLimiter>>
) -> Result<String, String> {
    // Rate limiting
    rate_limiter.check_rate_limit()?;
    
    // Input validation
    validate_input_size(&input)?;
    
    // Process safely
    Ok(process_input(&input))
}
```

## 3. Secure Parser Implementation

### File: `src/conversion/secure_parser.rs`

The secure parser includes:

- Recursion depth tracking (max: 50 levels)
- Parsing timeout (30 seconds)
- Text size limits (1MB)
- Control word whitelisting
- Integer overflow protection

### Key Security Controls:

```rust
const MAX_NESTING_DEPTH: usize = 50;
const MAX_TEXT_SIZE: usize = 1_000_000;
const PARSING_TIMEOUT: Duration = Duration::from_secs(30);
```

## 4. Rate Limiting Configuration

### Implementation:

```rust
pub const RATE_LIMIT_PER_SECOND: u64 = 10;
pub const RATE_LIMIT_BURST_SIZE: u32 = 20;
```

### Integration in main.rs:

```rust
fn main() {
    let rate_limiter = Arc::new(RateLimiter::new());
    
    tauri::Builder::default()
        .manage(rate_limiter)
        .invoke_handler(/* handlers */)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## 5. Input Validation Best Practices

### File Path Validation:

```rust
pub fn validate_file_path(path: &str) -> Result<PathBuf, String> {
    // Check for directory traversal
    if path.contains("..") {
        return Err("Invalid path: parent directory reference");
    }
    
    // Verify extension
    let ext = Path::new(path).extension()
        .and_then(|e| e.to_str())
        .ok_or("Missing file extension")?;
        
    if !["rtf", "md", "txt"].contains(&ext) {
        return Err("Invalid file type");
    }
    
    Ok(PathBuf::from(path))
}
```

### Size Validation:

```rust
pub fn validate_input_size(input: &str) -> Result<(), String> {
    const MAX_SIZE: usize = 10 * 1024 * 1024; // 10MB
    
    if input.len() > MAX_SIZE {
        return Err("Input exceeds maximum size");
    }
    Ok(())
}
```

## 6. Security Headers

### Required Headers:

```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: camera=(), microphone=(), geolocation=()
Strict-Transport-Security: max-age=31536000; includeSubDomains
```

## 7. Logging and Monitoring

### Security Event Logging:

```rust
#[derive(Debug)]
pub enum SecurityEventType {
    InvalidPath,
    OversizedInput,
    RateLimitExceeded,
    InvalidFileType,
    ParsingTimeout,
}

impl SecurityEvent {
    pub fn log(&self) {
        eprintln!("[SECURITY] {:?}: {}", self.event_type, self.details);
        // In production, send to logging service
    }
}
```

### Metrics to Monitor:

1. Failed validation attempts
2. Rate limit violations
3. Parsing timeouts
4. Memory usage spikes
5. Unusual file access patterns

## 8. Testing Security Features

### Unit Tests:

```rust
#[test]
fn test_path_traversal_blocked() {
    assert!(validate_file_path("../etc/passwd").is_err());
    assert!(validate_file_path("/etc/passwd").is_err());
}

#[test]
fn test_rate_limiter() {
    let limiter = RateLimiter::new();
    
    // Should allow first 10 requests
    for _ in 0..10 {
        assert!(limiter.check_rate_limit().is_ok());
    }
    
    // 11th request should fail
    assert!(limiter.check_rate_limit().is_err());
}
```

### Integration Tests:

```rust
#[test]
async fn test_secure_conversion() {
    let malicious_rtf = "{\\rtf1\\ansi{" + &"\\{".repeat(100) + "}}";
    let result = secure_rtf_to_markdown(&malicious_rtf);
    assert!(result.is_err());
}
```

## 9. Deployment Security

### Environment Variables:

```bash
# Production settings
export RTF_MAX_FILE_SIZE=10485760
export RTF_MAX_NESTING_DEPTH=50
export RTF_PARSING_TIMEOUT=30
export RTF_RATE_LIMIT_PER_SECOND=10
export RTF_LOG_LEVEL=warn
```

### Build Commands:

```bash
# Development build with security checks
cargo build --features security-dev

# Production build with all optimizations
cargo build --release --features security-prod

# Security audit
cargo audit

# Static analysis
cargo clippy -- -D warnings
```

## 10. Incident Response

### If a Security Issue is Detected:

1. **Immediate Actions**:
   - Enable emergency rate limiting (1 req/sec)
   - Increase logging verbosity
   - Alert security team

2. **Investigation**:
   - Review security logs
   - Analyze failed validations
   - Check for patterns in requests

3. **Mitigation**:
   - Deploy hotfix if needed
   - Update security rules
   - Block malicious IPs/patterns

4. **Post-Incident**:
   - Document findings
   - Update security tests
   - Review and improve controls

## 11. Security Maintenance Schedule

### Daily:
- Review security logs
- Check rate limit violations
- Monitor resource usage

### Weekly:
- Run `cargo audit`
- Review dependency updates
- Analyze security metrics

### Monthly:
- Security configuration review
- Penetration testing
- Update threat model

### Quarterly:
- Full security audit
- Update security documentation
- Security training for team

## 12. Common Security Pitfalls to Avoid

1. **Never use `all: true` in Tauri allowlist**
2. **Always validate file paths before access**
3. **Never trust user input - validate everything**
4. **Don't expose detailed error messages to users**
5. **Always use the secure parser for RTF conversion**
6. **Keep dependencies updated**
7. **Log security events but not sensitive data**
8. **Test with malicious inputs regularly**

## Contact Information

For security concerns or questions:

- Security Team: security@legacybridge.com
- Emergency: security-oncall@legacybridge.com
- Report vulnerabilities: security-reports@legacybridge.com

---

**Remember**: Security is not a one-time implementation but an ongoing process. Stay vigilant and keep improving!