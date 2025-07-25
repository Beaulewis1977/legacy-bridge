# Unified Conversion Architecture Design

## Overview
Consolidate duplicate conversion implementations into a single, configurable architecture that provides security by default with performance optimization options.

## Core Design Principles
1. **Security by Default**: All conversions use secure settings unless explicitly configured otherwise
2. **Single Implementation**: One parser, one generator, one FFI interface
3. **Configurable Security Levels**: Runtime configuration for security vs performance trade-offs
4. **Feature Flags**: Compile-time and runtime feature configuration
5. **Backwards Compatibility**: Maintain existing APIs through thin wrappers

## Unified Architecture Components

### 1. Security Configuration
```rust
pub enum SecurityLevel {
    /// Maximum security, slower performance
    Paranoid {
        limits: SecurityLimits,
        control_words: ControlWordSecurity,
    },
    /// Balanced security and performance (default)
    Enhanced {
        limits: SecurityLimits,
    },
    /// Basic validation, faster performance
    Standard,
}

pub struct ConversionConfig {
    pub security_level: SecurityLevel,
    pub validation_enabled: bool,
    pub memory_pool: Option<MemoryPool>,
    pub timeout: Option<Duration>,
    pub logging_enabled: bool,
}
```

### 2. Unified Parser
```rust
pub struct UnifiedParser {
    config: ConversionConfig,
    state: ParserState,
}

impl UnifiedParser {
    pub fn parse_rtf(tokens: Vec<RtfToken>) -> ConversionResult<RtfDocument>;
    pub fn parse_markdown(content: &str) -> ConversionResult<RtfDocument>;
}
```

### 3. Unified Generator
```rust
pub struct UnifiedGenerator {
    config: ConversionConfig,
    output_tracker: OutputTracker,
}

impl UnifiedGenerator {
    pub fn generate_rtf(document: &RtfDocument) -> ConversionResult<String>;
    pub fn generate_markdown(document: &RtfDocument) -> ConversionResult<String>;
}
```

### 4. Unified FFI Interface
```rust
pub struct UnifiedFFI {
    config: ConversionConfig,
    error_handler: ErrorHandler,
}

// Single entry points with optional security configuration
pub extern "C" fn legacybridge_convert(
    input: *const c_char,
    output: *mut *mut c_char,
    config: *const FFIConfig,
) -> c_int;
```

## Migration Strategy

### Phase 1: Create Unified Components
1. Implement `UnifiedParser` combining secure and standard logic
2. Implement `UnifiedGenerator` with configurable security
3. Create `ConversionConfig` and `SecurityLevel` types

### Phase 2: Refactor Existing Code
1. Replace duplicate implementations with unified versions
2. Update existing APIs to use unified components internally
3. Add configuration parameters to public APIs

### Phase 3: Clean Up
1. Remove obsolete secure_* modules
2. Consolidate test suites
3. Update documentation

## Benefits
- **Code Reduction**: 20-30% fewer lines of code
- **Maintainability**: Single implementation to maintain
- **Flexibility**: Runtime security configuration
- **Performance**: Optimized paths for different use cases
- **Testing**: Unified test suite covers all scenarios