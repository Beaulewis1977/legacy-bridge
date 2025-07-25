// Unified configuration for conversion operations
//
// This module provides a single configuration system that replaces
// the duplicate secure/standard implementations with runtime-configurable
// security levels and performance options.

use std::time::Duration;
use super::security::{SecurityLimits, ControlWordSecurity};

/// Security level for conversion operations
#[derive(Debug, Clone)]
pub enum SecurityLevel {
    /// Maximum security with strict validation and limits
    /// Slowest performance but highest security
    Paranoid {
        limits: SecurityLimits,
        control_words: ControlWordSecurity,
    },
    
    /// Balanced security and performance (default)
    /// Suitable for most use cases
    Enhanced {
        limits: SecurityLimits,
    },
    
    /// Basic validation for trusted content
    /// Fastest performance with minimal security checks
    Standard,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        SecurityLevel::Enhanced {
            limits: SecurityLimits::default(),
        }
    }
}

impl SecurityLevel {
    /// Create a paranoid security level with custom limits
    pub fn paranoid() -> Self {
        SecurityLevel::Paranoid {
            limits: SecurityLimits::default(),
            control_words: ControlWordSecurity::whitelist(),
        }
    }
    
    /// Get security limits for this level
    pub fn limits(&self) -> Option<&SecurityLimits> {
        match self {
            SecurityLevel::Paranoid { limits, .. } => Some(limits),
            SecurityLevel::Enhanced { limits } => Some(limits),
            SecurityLevel::Standard => None,
        }
    }
    
    /// Get control word security for this level
    pub fn control_words(&self) -> Option<&ControlWordSecurity> {
        match self {
            SecurityLevel::Paranoid { control_words, .. } => Some(control_words),
            _ => None,
        }
    }
    
    /// Check if timeout enforcement is enabled
    pub fn enforce_timeout(&self) -> bool {
        !matches!(self, SecurityLevel::Standard)
    }
    
    /// Check if size limits should be enforced
    pub fn enforce_size_limits(&self) -> bool {
        !matches!(self, SecurityLevel::Standard)
    }
}

/// Memory pool configuration for optimized allocations
#[derive(Debug, Clone)]
pub struct MemoryPool {
    /// Initial pool size in bytes
    pub initial_size: usize,
    /// Maximum pool size in bytes
    pub max_size: usize,
    /// Whether to pre-allocate memory
    pub pre_allocate: bool,
}

impl Default for MemoryPool {
    fn default() -> Self {
        Self {
            initial_size: 1024 * 1024,     // 1MB
            max_size: 10 * 1024 * 1024,    // 10MB
            pre_allocate: false,
        }
    }
}

/// Unified configuration for all conversion operations
#[derive(Debug, Clone)]
pub struct ConversionConfig {
    /// Security level to use
    pub security_level: SecurityLevel,
    
    /// Whether to enable input validation
    pub validation_enabled: bool,
    
    /// Optional memory pool for performance
    pub memory_pool: Option<MemoryPool>,
    
    /// Optional timeout for operations
    pub timeout: Option<Duration>,
    
    /// Whether to enable detailed logging
    pub logging_enabled: bool,
    
    /// Whether to use pipeline for complex documents
    pub use_pipeline: bool,
    
    /// Whether to preserve exact formatting
    pub preserve_formatting: bool,
    
    /// Whether to auto-recover from errors
    pub auto_recovery: bool,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            security_level: SecurityLevel::default(),
            validation_enabled: true,
            memory_pool: None,
            timeout: Some(Duration::from_secs(30)),
            logging_enabled: false,
            use_pipeline: true,
            preserve_formatting: true,
            auto_recovery: true,
        }
    }
}

impl ConversionConfig {
    /// Create a high-security configuration
    pub fn high_security() -> Self {
        Self {
            security_level: SecurityLevel::paranoid(),
            validation_enabled: true,
            memory_pool: None,
            timeout: Some(Duration::from_secs(30)),
            logging_enabled: true,
            use_pipeline: false, // Direct conversion for better control
            preserve_formatting: true,
            auto_recovery: false, // Fail on errors
        }
    }
    
    /// Create a high-performance configuration
    pub fn high_performance() -> Self {
        Self {
            security_level: SecurityLevel::Standard,
            validation_enabled: false,
            memory_pool: Some(MemoryPool::default()),
            timeout: None,
            logging_enabled: false,
            use_pipeline: true,
            preserve_formatting: true,
            auto_recovery: true,
        }
    }
    
    /// Create a balanced configuration (alias for default)
    pub fn balanced() -> Self {
        Self::default()
    }
    
    /// Check if we should validate input
    pub fn should_validate(&self) -> bool {
        self.validation_enabled || !matches!(self.security_level, SecurityLevel::Standard)
    }
    
    /// Get timeout duration
    pub fn timeout_duration(&self) -> Option<Duration> {
        match (&self.timeout, &self.security_level) {
            (Some(timeout), _) => Some(*timeout),
            (None, SecurityLevel::Paranoid { .. }) => Some(Duration::from_secs(30)),
            (None, SecurityLevel::Enhanced { .. }) => Some(Duration::from_secs(60)),
            (None, SecurityLevel::Standard) => None,
        }
    }
}

/// Builder for ConversionConfig
pub struct ConversionConfigBuilder {
    config: ConversionConfig,
}

impl ConversionConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: ConversionConfig::default(),
        }
    }
    
    pub fn security_level(mut self, level: SecurityLevel) -> Self {
        self.config.security_level = level;
        self
    }
    
    pub fn validation(mut self, enabled: bool) -> Self {
        self.config.validation_enabled = enabled;
        self
    }
    
    pub fn memory_pool(mut self, pool: MemoryPool) -> Self {
        self.config.memory_pool = Some(pool);
        self
    }
    
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.config.timeout = Some(duration);
        self
    }
    
    pub fn logging(mut self, enabled: bool) -> Self {
        self.config.logging_enabled = enabled;
        self
    }
    
    pub fn pipeline(mut self, enabled: bool) -> Self {
        self.config.use_pipeline = enabled;
        self
    }
    
    pub fn preserve_formatting(mut self, enabled: bool) -> Self {
        self.config.preserve_formatting = enabled;
        self
    }
    
    pub fn auto_recovery(mut self, enabled: bool) -> Self {
        self.config.auto_recovery = enabled;
        self
    }
    
    pub fn build(self) -> ConversionConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_security_levels() {
        let paranoid = SecurityLevel::paranoid();
        assert!(paranoid.limits().is_some());
        assert!(paranoid.control_words().is_some());
        assert!(paranoid.enforce_timeout());
        
        let standard = SecurityLevel::Standard;
        assert!(standard.limits().is_none());
        assert!(!standard.enforce_timeout());
    }
    
    #[test]
    fn test_config_builder() {
        let config = ConversionConfigBuilder::new()
            .security_level(SecurityLevel::paranoid())
            .validation(true)
            .timeout(Duration::from_secs(10))
            .logging(true)
            .build();
            
        assert!(matches!(config.security_level, SecurityLevel::Paranoid { .. }));
        assert!(config.validation_enabled);
        assert_eq!(config.timeout, Some(Duration::from_secs(10)));
        assert!(config.logging_enabled);
    }
    
    #[test]
    fn test_preset_configs() {
        let high_sec = ConversionConfig::high_security();
        assert!(matches!(high_sec.security_level, SecurityLevel::Paranoid { .. }));
        assert!(high_sec.validation_enabled);
        assert!(!high_sec.auto_recovery);
        
        let high_perf = ConversionConfig::high_performance();
        assert!(matches!(high_perf.security_level, SecurityLevel::Standard));
        assert!(!high_perf.validation_enabled);
        assert!(high_perf.memory_pool.is_some());
    }
}