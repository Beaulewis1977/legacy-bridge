# Unified Error Handling Architecture

## Overview

LegacyBridge implements a comprehensive unified error handling system that maintains consistency and type safety across TypeScript frontend, Rust backend, and C FFI boundaries. This architecture ensures that error information is never lost during cross-language communication and provides actionable feedback to both users and developers.

## Architecture Components

### 1. Rust Core Error Types (`unified_errors.rs`)

The Rust layer defines the canonical error structure used throughout the system:

```rust
pub enum LegacyBridgeError {
    ParseError {
        message: String,
        line: u32,
        column: u32,
        expected: Option<String>,
        found: Option<String>,
        file_path: Option<String>,
    },
    ConversionError {
        source_format: String,
        target_format: String,
        details: String,
        recoverable: bool,
        suggestions: Vec<String>,
    },
    IOError {
        operation: String,
        path: String,
        cause: String,
        error_code: Option<i32>,
    },
    ValidationError {
        field: String,
        expected: String,
        received: String,
        location: Option<ErrorContext>,
    },
    SystemError {
        component: String,
        error_code: u32,
        description: String,
        internal_message: Option<String>,
    },
    ResourceLimitError {
        resource: String,
        limit: String,
        actual: String,
        suggestion: String,
    },
    NotImplementedError {
        feature: String,
        workaround: Option<String>,
        planned_version: Option<String>,
    },
}
```

Each error type includes:
- **Rich Context**: Line/column numbers, file paths, expected vs actual values
- **User Messages**: Safe, friendly messages for end users
- **Developer Details**: Technical information for debugging
- **Recovery Information**: Whether the error is recoverable and suggestions for fixing
- **Error Codes**: Consistent numeric codes for programmatic handling

### 2. FFI Error Bridge (`ffi_error_bridge.rs`)

The FFI bridge ensures safe error propagation across the C boundary:

```rust
// Get last error as JSON string
#[no_mangle]
pub extern "C" fn legacybridge_get_last_error_json() -> *mut c_char;

// Clear last error
#[no_mangle]
pub extern "C" fn legacybridge_clear_last_error();
```

Key features:
- **Thread-Local Storage**: Errors are stored per-thread for safety
- **JSON Serialization**: Errors are serialized to JSON for cross-language compatibility
- **No Information Loss**: All error details are preserved across the FFI boundary
- **Memory Safety**: Proper string allocation and deallocation

### 3. TypeScript Error Types (`errors.ts`)

TypeScript mirrors the Rust error structure with proper typing:

```typescript
export type LegacyBridgeError =
  | ParseError
  | ConversionError
  | IOError
  | ValidationError
  | SystemError
  | ResourceLimitError
  | NotImplementedError;

export interface ErrorResponse {
  errorType: string;
  errorCode: number;
  message: string;
  userMessage: string;
  details: LegacyBridgeError;
  suggestions: string[];
  recoverable: boolean;
  timestamp: string;
}
```

### 4. Error Handler Utilities

The system provides comprehensive utilities for error handling:

```typescript
export class ErrorHandler {
  static parseErrorResponse(response: unknown): ErrorResponse | null;
  static getUserMessage(error: LegacyBridgeError): string;
  static getSuggestions(error: LegacyBridgeError): string[];
  static isRecoverable(error: LegacyBridgeError): boolean;
  static formatError(error: LegacyBridgeError): string;
}
```

## Error Flow

### 1. Error Creation (Rust)
```rust
// Example: Creating a parse error with full context
let error = LegacyBridgeError::ParseError {
    message: "Unexpected token in RTF header".to_string(),
    line: 1,
    column: 5,
    expected: Some("{\\rtf1".to_string()),
    found: Some("RTF".to_string()),
    file_path: Some("document.rtf".to_string()),
};
```

### 2. FFI Propagation
```rust
// Error is automatically stored and converted to JSON
set_last_error(error);
return error.error_code(); // Returns -1001 for ParseError
```

### 3. TypeScript Retrieval
```typescript
// API wrapper automatically retrieves and parses error
const result = await ConversionAPI.convertRtfToMarkdown(content);
if (!result.success) {
  const error = result.error; // Fully typed ErrorResponse
  console.log(error.userMessage); // "The document format is invalid or corrupted"
  console.log(error.suggestions); // ["Check if the file is a valid RTF document", ...]
}
```

### 4. UI Display
```tsx
// React component displays error with appropriate styling and actions
<UnifiedErrorDisplay
  error={error}
  onRetry={handleRetry}
  showDetails={true}
/>
```

## Error Categories and Handling

### Parse Errors
- **When**: Document structure is invalid
- **Context**: Line/column numbers, expected/found tokens
- **Recovery**: Generally not recoverable, require manual fixes
- **User Action**: Fix document format

### Conversion Errors
- **When**: Conversion between formats fails
- **Context**: Source/target formats, specific failure reason
- **Recovery**: Often recoverable with simplified content
- **User Action**: Follow suggestions to simplify content

### IO Errors
- **When**: File operations fail
- **Context**: Operation type, file path, system error code
- **Recovery**: Retry after fixing permissions/path
- **User Action**: Check file permissions and paths

### Validation Errors
- **When**: Input doesn't meet requirements
- **Context**: Field name, expected vs received values
- **Recovery**: Usually recoverable by fixing input
- **User Action**: Correct the invalid input

### System Errors
- **When**: Internal errors occur
- **Context**: Component name, error code
- **Recovery**: Generally not recoverable
- **User Action**: Report to support

### Resource Limit Errors
- **When**: Limits exceeded (file size, memory, etc.)
- **Context**: Resource type, limit, actual value
- **Recovery**: Not directly recoverable
- **User Action**: Reduce resource usage

### Not Implemented Errors
- **When**: Feature not available
- **Context**: Feature name, workaround, planned version
- **Recovery**: Use workaround if available
- **User Action**: Use alternative approach

## Security Considerations

1. **No Sensitive Data in User Messages**: User-facing messages never contain file paths, internal details, or stack traces
2. **Sanitized Error Details**: Internal error messages are sanitized before external exposure
3. **Rate Limiting**: Error reporting includes built-in rate limiting to prevent DoS
4. **Secure Logging**: Developer details are logged securely, not exposed to users

## Best Practices

### For Rust Development
```rust
// Always provide context when creating errors
let error = LegacyBridgeError::ParseError {
    message: format!("Invalid control word: {}", word),
    line: current_line,
    column: current_column,
    expected: Some("valid RTF control word".to_string()),
    found: Some(word.to_string()),
    file_path: file_path.clone(),
};

// Use helper functions for common patterns
let error = create_io_error("read", file_path, io_error);
```

### For TypeScript Development
```typescript
// Always handle both success and error cases
const result = await convertFile(inputPath, outputPath);
if (result.success) {
  console.log('Conversion successful:', result.data);
} else {
  // Error is fully typed
  handleError(result.error);
}

// Use error recovery when available
if (ErrorHandler.isRecoverable(error)) {
  const recovery = await ErrorRecovery.attemptRecovery(error, content);
  if (recovery.success) {
    // Continue with recovered content
  }
}
```

### For UI Development
```tsx
// Provide appropriate error display based on severity
<UnifiedErrorDisplay
  error={error}
  onRetry={canRetry ? handleRetry : undefined}
  showDetails={isDevelopment}
/>

// Subscribe to error notifications for global handling
useEffect(() => {
  const unsubscribe = ErrorNotificationService.subscribe((error) => {
    // Log to analytics
    analytics.track('error', {
      type: error.errorType,
      code: error.errorCode,
      recoverable: error.recoverable,
    });
  });
  
  return unsubscribe;
}, []);
```

## Testing

### Unit Tests
```rust
#[test]
fn test_error_serialization() {
    let error = create_parse_error("Test error", 10, 5, Some("test.rtf"));
    let json = error.to_json().unwrap();
    assert!(json.contains("ParseError"));
    assert!(json.contains("\"line\":10"));
}
```

### Integration Tests
```typescript
it('should properly handle conversion errors', async () => {
  const result = await convertRtfToMarkdown('invalid rtf content');
  expect(result.success).toBe(false);
  expect(result.error.errorType).toBe('ParseError');
  expect(result.error.suggestions.length).toBeGreaterThan(0);
});
```

### End-to-End Tests
```typescript
it('should display errors with recovery options', async () => {
  // Trigger a recoverable error
  const { getByText, getByRole } = render(<ConversionForm />);
  
  // Submit invalid content
  fireEvent.click(getByText('Convert'));
  
  // Wait for error display
  await waitFor(() => {
    expect(getByText(/failed to convert/i)).toBeInTheDocument();
    expect(getByRole('button', { name: /attempt recovery/i })).toBeInTheDocument();
  });
});
```

## Migration Guide

### From Old Error System
```rust
// Old
return Err(ConversionError::ParseError("Invalid RTF".to_string()));

// New
return Err(LegacyBridgeError::ParseError {
    message: "Invalid RTF header".to_string(),
    line: 1,
    column: 1,
    expected: Some("{\\rtf".to_string()),
    found: Some(actual_header),
    file_path: current_file.clone(),
});
```

### FFI Functions
```c
// Old
int result = legacybridge_rtf_to_markdown(input, &output, &length);
if (result < 0) {
    char error[256];
    legacybridge_get_last_error(error, 256);
}

// New
int result = legacybridge_rtf_to_markdown(input, &output, &length);
if (result < 0) {
    char* error_json = legacybridge_get_last_error_json();
    // Parse JSON to get structured error information
    legacybridge_free_string(error_json);
}
```

## Future Enhancements

1. **Error Analytics**: Automatic error tracking and analytics
2. **Error Recovery Strategies**: Pluggable recovery strategies for different error types
3. **Localization**: Multi-language error messages
4. **Error Chaining**: Support for nested errors with full context
5. **Performance Monitoring**: Error correlation with performance metrics