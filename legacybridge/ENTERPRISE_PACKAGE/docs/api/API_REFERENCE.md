# LegacyBridge API Reference v1.0.0

## Table of Contents
1. [Overview](#overview)
2. [Core Functions](#core-functions)
3. [Memory Management](#memory-management)
4. [Batch Processing](#batch-processing)
5. [Utility Functions](#utility-functions)
6. [Error Handling](#error-handling)
7. [Constants](#constants)
8. [Language Bindings](#language-bindings)

## Overview

LegacyBridge provides a C-compatible API for converting between RTF and Markdown formats. All functions are exported with C linkage and use standard C types for maximum compatibility.

### Function Naming Convention
All exported functions follow the pattern: `legacybridge_[operation]`

### Memory Management
- All string returns are dynamically allocated
- Caller must free returned strings using `legacybridge_free_string()`
- Input strings are not modified

### Thread Safety
- All functions are thread-safe
- Error messages are stored per-thread

## Core Functions

### rtf_to_markdown
```c
char* rtf_to_markdown(const char* rtf_content);
```
Converts RTF content to Markdown format.

**Parameters:**
- `rtf_content`: Null-terminated UTF-8 string containing RTF

**Returns:**
- Success: Newly allocated string with Markdown content
- Failure: NULL (check error with `get_last_error()`)

**Example:**
```c
char* markdown = rtf_to_markdown("{\\rtf1 Hello World}");
if (markdown) {
    printf("%s\n", markdown);
    free_string(markdown);
}
```

---

### markdown_to_rtf
```c
char* markdown_to_rtf(const char* markdown_content);
```
Converts Markdown content to RTF format.

**Parameters:**
- `markdown_content`: Null-terminated UTF-8 string containing Markdown

**Returns:**
- Success: Newly allocated string with RTF content
- Failure: NULL (check error with `get_last_error()`)

**Example:**
```c
char* rtf = markdown_to_rtf("# Hello World");
if (rtf) {
    printf("%s\n", rtf);
    free_string(rtf);
}
```

---

### rtf_to_markdown_safe
```c
int32_t rtf_to_markdown_safe(const char* rtf_content, 
                             char* output_buffer, 
                             size_t buffer_size, 
                             size_t* bytes_written);
```
Thread-safe RTF to Markdown conversion with buffer management.

**Parameters:**
- `rtf_content`: Input RTF content
- `output_buffer`: Pre-allocated output buffer
- `buffer_size`: Size of output buffer
- `bytes_written`: Actual bytes written (can be NULL)

**Returns:**
- 0: Success
- Non-zero: Error code

---

### markdown_to_rtf_safe
```c
int32_t markdown_to_rtf_safe(const char* markdown_content,
                             char* output_buffer,
                             size_t buffer_size,
                             size_t* bytes_written);
```
Thread-safe Markdown to RTF conversion with buffer management.

**Parameters:**
- `markdown_content`: Input Markdown content
- `output_buffer`: Pre-allocated output buffer
- `buffer_size`: Size of output buffer
- `bytes_written`: Actual bytes written (can be NULL)

**Returns:**
- 0: Success
- Non-zero: Error code

## Memory Management

### free_string
```c
void free_string(char* ptr);
```
Frees memory allocated by LegacyBridge functions.

**Parameters:**
- `ptr`: Pointer returned by conversion functions

**Important:** Always use this function to free strings returned by LegacyBridge.

## Batch Processing

### batch_rtf_to_markdown
```c
char** batch_rtf_to_markdown(const char** rtf_contents, 
                             size_t count, 
                             size_t* success_count);
```
Converts multiple RTF documents to Markdown in a single call.

**Parameters:**
- `rtf_contents`: Array of RTF strings
- `count`: Number of documents
- `success_count`: Number of successful conversions

**Returns:**
- Array of Markdown strings (NULL for failed conversions)

**Example:**
```c
const char* rtf_docs[] = {rtf1, rtf2, rtf3};
size_t converted;
char** results = batch_rtf_to_markdown(rtf_docs, 3, &converted);
for (size_t i = 0; i < 3; i++) {
    if (results[i]) {
        printf("Document %zu: %s\n", i, results[i]);
        free_string(results[i]);
    }
}
free(results);
```

---

### batch_markdown_to_rtf
```c
char** batch_markdown_to_rtf(const char** markdown_contents,
                            size_t count,
                            size_t* success_count);
```
Converts multiple Markdown documents to RTF in a single call.

## Utility Functions

### get_version
```c
const char* get_version(void);
```
Returns the LegacyBridge version string.

**Returns:** Static string (do not free)

---

### get_last_error
```c
const char* get_last_error(void);
```
Retrieves the last error message for the current thread.

**Returns:** Static error message or "No error"

---

### clear_error
```c
void clear_error(void);
```
Clears the error state for the current thread.

---

### validate_rtf
```c
int32_t validate_rtf(const char* rtf_content);
```
Validates RTF content without converting.

**Returns:**
- 0: Valid RTF
- Non-zero: Invalid RTF

---

### validate_markdown
```c
int32_t validate_markdown(const char* markdown_content);
```
Validates Markdown content without converting.

**Returns:**
- 0: Valid Markdown
- Non-zero: Invalid Markdown

## Error Handling

### Error Codes
```c
#define LEGACYBRIDGE_SUCCESS              0
#define LEGACYBRIDGE_ERROR_INVALID_INPUT  1
#define LEGACYBRIDGE_ERROR_CONVERSION     2
#define LEGACYBRIDGE_ERROR_MEMORY         3
#define LEGACYBRIDGE_ERROR_BUFFER_TOO_SMALL 4
#define LEGACYBRIDGE_ERROR_ENCODING       5
```

### Error Handling Pattern
```c
char* result = rtf_to_markdown(input);
if (!result) {
    const char* error = get_last_error();
    fprintf(stderr, "Conversion failed: %s\n", error);
    return -1;
}
// Use result
free_string(result);
```

## Constants

### Version Information
```c
#define LEGACYBRIDGE_VERSION_MAJOR 1
#define LEGACYBRIDGE_VERSION_MINOR 0
#define LEGACYBRIDGE_VERSION_PATCH 0
```

### Buffer Sizes
```c
#define LEGACYBRIDGE_MAX_ERROR_LENGTH 256
#define LEGACYBRIDGE_RECOMMENDED_BUFFER_SIZE 65536
```

## Language Bindings

### Visual Basic 6
```vb
' Declaration
Private Declare Function rtf_to_markdown Lib "legacybridge.dll" _
    (ByVal rtfContent As String) As Long

' Usage with wrapper
Dim markdown As String
markdown = ConvertRTFToMarkdown(rtfContent)
```

### Visual FoxPro 9
```foxpro
* Declaration
DECLARE STRING rtf_to_markdown IN legacybridge.dll STRING rtfContent

* Usage with wrapper
lcMarkdown = oLegacyBridge.RtfToMarkdown(lcRTF)
```

### Python (ctypes)
```python
import ctypes

# Load library
lib = ctypes.CDLL('./legacybridge.dll')

# Configure function
lib.rtf_to_markdown.argtypes = [ctypes.c_char_p]
lib.rtf_to_markdown.restype = ctypes.c_char_p

# Use
result = lib.rtf_to_markdown(rtf_content.encode('utf-8'))
markdown = result.decode('utf-8')
lib.free_string(result)
```

### C#
```csharp
[DllImport("legacybridge.dll", CharSet = CharSet.Ansi)]
private static extern IntPtr rtf_to_markdown(string rtfContent);

[DllImport("legacybridge.dll")]
private static extern void free_string(IntPtr ptr);

// Usage
IntPtr result = rtf_to_markdown(rtfContent);
string markdown = Marshal.PtrToStringAnsi(result);
free_string(result);
```

## Performance Considerations

1. **Batch Processing**: Use batch functions for multiple documents
2. **Buffer Reuse**: Use `_safe` functions with pre-allocated buffers
3. **Memory**: Free strings promptly to avoid memory leaks
4. **Threading**: Functions are thread-safe but maintain separate error states

## Best Practices

1. Always check return values
2. Free all allocated strings
3. Use appropriate buffer sizes (64KB recommended)
4. Handle errors gracefully
5. Validate input when necessary
6. Use batch operations for multiple conversions

## Troubleshooting

### Common Issues

**NULL returns:**
- Check input validity
- Verify DLL is loaded
- Check error message

**Memory leaks:**
- Ensure all strings are freed
- Use memory profiler

**Performance:**
- Use batch operations
- Pre-allocate buffers
- Profile bottlenecks

---

For additional support, refer to the integration guides and example code in the package.