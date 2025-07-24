# 🔌 LegacyBridge API Reference

*Complete documentation for all 29 API functions*

---

## 📚 Navigation

**🏠 [← Back to Main README](README.md)**

### 📋 Documentation Links
- **[📖 User Guide](USER_GUIDE.md)** - Complete usage guide with examples
- **[🚀 Installation Guide](ENTERPRISE_INSTALLATION_GUIDE.md)** - Enterprise deployment instructions
- **[🐛 Troubleshooting Guide](TROUBLESHOOTING_GUIDE.md)** - Common issues and solutions
- **[📝 Release Notes](RELEASE_NOTES.md)** - Version history and changes

---

## 📚 Table of Contents

1. [🎯 Overview](#-overview)
2. [❌ Error Codes](#-error-codes)
3. [🔄 Core Conversion Functions](#-core-conversion-functions)
4. [📁 File Operations](#-file-operations)
5. [🔍 Validation Functions](#-validation-functions)
6. [📦 Batch Processing](#-batch-processing)
7. [✏️ Text Processing](#️-text-processing)
8. [🎨 Template Functions](#-template-functions)
9. [📊 Data Import/Export](#-data-importexport)
10. [🔧 Utility Functions](#-utility-functions)
11. [💻 Language-Specific Wrappers](#-language-specific-wrappers)

---

## 👨‍💻 About the Developer

**Built with ❤️ by [Beau Lewis](mailto:blewisxx@gmail.com)**

*If LegacyBridge has helped you, consider supporting the project:*
- ☕ **[Ko-fi](https://ko-fi.com/beaulewis)** - Buy me a coffee
- 💳 **[Venmo](https://venmo.com/beauintulsa)** - Quick thanks

---

## Overview

LegacyBridge provides 29 exported functions for comprehensive document conversion between RTF and Markdown formats. All functions use C-compatible exports for maximum compatibility with legacy systems.

### Calling Conventions

- **Convention**: CDECL (C Declaration)
- **Character Encoding**: UTF-8
- **String Parameters**: Null-terminated C strings
- **Memory Management**: Caller must free returned strings using `legacybridge_free_string()`

### Return Values

Most functions return an integer status code:
- `0`: Success
- Negative values: Error codes (see [Error Codes](#error-codes))
- Positive values: Count or specific success indicators

## Error Codes

```c
#define LB_SUCCESS               0    // Operation completed successfully
#define LB_ERROR_NULL_POINTER   -1    // Null pointer passed to function
#define LB_ERROR_INVALID_UTF8   -2    // Invalid UTF-8 encoding in input
#define LB_ERROR_CONVERSION     -3    // Conversion failed
#define LB_ERROR_ALLOCATION     -4    // Memory allocation failed
```

## Core Conversion Functions

### legacybridge_rtf_to_markdown

Converts RTF content to Markdown format.

```c
int legacybridge_rtf_to_markdown(
    const char* rtf_content,
    char** output_buffer,
    int* output_length
);
```

**Parameters:**
- `rtf_content`: Input RTF document as null-terminated string
- `output_buffer`: Pointer to receive the allocated output buffer
- `output_length`: Pointer to receive the output length (excluding null terminator)

**Returns:**
- `0`: Success
- Error code on failure

**Example (VB6):**
```vb
Dim rtf As String
Dim markdown As String
Dim result As Long

rtf = "{\rtf1\ansi\b Hello World\b0}"
result = ConvertRTFToMarkdown(rtf, markdown)

If result = 0 Then
    ' markdown now contains "**Hello World**"
End If
```

### legacybridge_markdown_to_rtf

Converts Markdown content to RTF format.

```c
int legacybridge_markdown_to_rtf(
    const char* markdown_content,
    char** output_buffer,
    int* output_length
);
```

**Parameters:**
- `markdown_content`: Input Markdown document as null-terminated string
- `output_buffer`: Pointer to receive the allocated output buffer
- `output_length`: Pointer to receive the output length

**Returns:**
- `0`: Success
- Error code on failure

**Example (VFP9):**
```foxpro
LOCAL lcMarkdown, lcRTF, lnResult

lcMarkdown = "# Hello World" + CHR(10) + "This is **bold** text."
lnResult = loBridge.ConvertMarkdownToRTF(lcMarkdown)

IF lnResult = 0
    * RTF conversion successful
ENDIF
```

## File Operations

### legacybridge_convert_rtf_file_to_md

Converts an RTF file directly to a Markdown file.

```c
int legacybridge_convert_rtf_file_to_md(
    const char* input_path,
    const char* output_path
);
```

**Parameters:**
- `input_path`: Path to input RTF file
- `output_path`: Path for output Markdown file

**Returns:**
- `0`: Success
- Error code on failure

**Benefits:**
- More efficient for large files
- No memory allocation needed
- Progress tracking available

### legacybridge_convert_md_file_to_rtf

Converts a Markdown file directly to an RTF file.

```c
int legacybridge_convert_md_file_to_rtf(
    const char* input_path,
    const char* output_path
);
```

**Parameters:**
- `input_path`: Path to input Markdown file
- `output_path`: Path for output RTF file

**Returns:**
- `0`: Success
- Error code on failure

## Validation Functions

### legacybridge_validate_rtf_document

Validates RTF document structure and syntax.

```c
int legacybridge_validate_rtf_document(
    const char* rtf_content,
    char** output_buffer,
    int* output_length
);
```

**Parameters:**
- `rtf_content`: RTF content to validate
- `output_buffer`: Buffer to receive validation report
- `output_length`: Length of validation report

**Returns:**
- `0`: Document is valid
- Error code if invalid

**Validation Report Format:**
```json
{
    "valid": true,
    "errors": [],
    "warnings": ["Missing font table"],
    "statistics": {
        "groups": 45,
        "controls": 123,
        "text_length": 5678
    }
}
```

### legacybridge_validate_markdown_document

Validates Markdown document syntax.

```c
int legacybridge_validate_markdown_document(
    const char* markdown_content,
    char** output_buffer,
    int* output_length
);
```

**Parameters:**
- `markdown_content`: Markdown content to validate
- `output_buffer`: Buffer to receive validation report
- `output_length`: Length of validation report

**Returns:**
- `0`: Document is valid
- Error code if invalid

### legacybridge_extract_plain_text

Extracts plain text from RTF document, removing all formatting.

```c
int legacybridge_extract_plain_text(
    const char* rtf_content,
    char** output_buffer,
    int* output_length
);
```

**Use Cases:**
- Full-text search indexing
- Content preview generation
- Word count and statistics

## Batch Processing

### legacybridge_batch_rtf_to_markdown

Converts multiple RTF documents to Markdown in a single operation.

```c
int legacybridge_batch_rtf_to_markdown(
    const char** rtf_array,
    int count,
    char** output_array,
    int* output_lengths
);
```

**Parameters:**
- `rtf_array`: Array of RTF content strings
- `count`: Number of documents to convert
- `output_array`: Array to receive output pointers
- `output_lengths`: Array to receive output lengths

**Returns:**
- Number of successful conversions

**Performance:**
- Optimized for parallel processing
- Up to 10x faster than individual conversions
- Automatic load balancing

### legacybridge_batch_markdown_to_rtf

Converts multiple Markdown documents to RTF in a single operation.

```c
int legacybridge_batch_markdown_to_rtf(
    const char** markdown_array,
    int count,
    char** output_array,
    int* output_lengths
);
```

### legacybridge_convert_folder_rtf_to_md

Processes all RTF files in a folder.

```c
int legacybridge_convert_folder_rtf_to_md(
    const char* input_folder,
    const char* output_folder
);
```

**Parameters:**
- `input_folder`: Source folder containing RTF files
- `output_folder`: Destination folder for Markdown files

**Returns:**
- Number of files processed
- Negative error code on failure

**Features:**
- Recursive subfolder processing
- Preserves folder structure
- Progress tracking available

### legacybridge_convert_folder_md_to_rtf

Processes all Markdown files in a folder.

```c
int legacybridge_convert_folder_md_to_rtf(
    const char* input_folder,
    const char* output_folder
);
```

### legacybridge_get_batch_progress

Returns current progress of batch operation.

```c
int legacybridge_get_batch_progress(void);
```

**Returns:**
- Number of files processed so far
- `-1` if no batch operation active

### legacybridge_cancel_batch_operation

Cancels the current batch operation.

```c
int legacybridge_cancel_batch_operation(void);
```

**Returns:**
- `0`: Cancellation successful
- `-1`: No operation to cancel

## Text Processing

### legacybridge_clean_rtf_formatting

Cleans and normalizes RTF formatting.

```c
int legacybridge_clean_rtf_formatting(
    const char* rtf_content,
    char** output_buffer,
    int* output_length
);
```

**Cleaning Operations:**
- Removes redundant control words
- Consolidates formatting
- Fixes malformed structures
- Optimizes file size

### legacybridge_normalize_markdown

Normalizes Markdown formatting for consistency.

```c
int legacybridge_normalize_markdown(
    const char* markdown_content,
    char** output_buffer,
    int* output_length
);
```

**Normalization Operations:**
- Consistent heading markers
- Standardized list formatting
- Uniform line endings
- Proper spacing

## Template Functions

### legacybridge_apply_rtf_template

Applies a formatting template to RTF content.

```c
int legacybridge_apply_rtf_template(
    const char* rtf_content,
    const char* template_name,
    char** output_buffer,
    int* output_length
);
```

**Built-in Templates:**
- `"minimal"`: Clean, simple formatting
- `"professional"`: Business document style
- `"academic"`: Formal academic papers

### legacybridge_create_rtf_template

Creates a custom template from RTF sample.

```c
int legacybridge_create_rtf_template(
    const char* template_name,
    const char* rtf_content
);
```

**Parameters:**
- `template_name`: Name for the new template
- `rtf_content`: Sample RTF to use as template

**Returns:**
- `0`: Template created successfully
- Error code on failure

### legacybridge_list_available_templates

Lists all available templates.

```c
int legacybridge_list_available_templates(
    char** output_buffer,
    int* output_length
);
```

**Output Format:**
```
minimal,professional,academic,custom1,custom2
```

### legacybridge_apply_markdown_template

Applies formatting rules to Markdown content.

```c
int legacybridge_apply_markdown_template(
    const char* markdown_content,
    const char* template_name,
    char** output_buffer,
    int* output_length
);
```

### legacybridge_validate_template

Validates a template by name.

```c
int legacybridge_validate_template(
    const char* template_name
);
```

**Returns:**
- `0`: Template exists and is valid
- `-1`: Template not found
- `-2`: Template corrupted

## Data Import/Export

### legacybridge_export_to_csv

Exports tables from RTF to CSV format.

```c
int legacybridge_export_to_csv(
    const char* rtf_content,
    char** output_buffer,
    int* output_length
);
```

**Features:**
- Extracts all tables from RTF
- Preserves cell data
- Handles merged cells
- UTF-8 CSV output

### legacybridge_import_from_csv

Imports CSV data as RTF table.

```c
int legacybridge_import_from_csv(
    const char* csv_content,
    char** output_buffer,
    int* output_length
);
```

**CSV Format:**
- Standard RFC 4180 compliant
- UTF-8 encoding
- Quoted fields supported
- Header row assumed

### legacybridge_convert_table_to_rtf

Converts structured table data to RTF format.

```c
int legacybridge_convert_table_to_rtf(
    const char* table_data,
    char** output_buffer,
    int* output_length
);
```

**Input Format (JSON):**
```json
{
    "columns": ["Name", "Age", "City"],
    "rows": [
        ["John Doe", "30", "New York"],
        ["Jane Smith", "25", "Los Angeles"]
    ]
}
```

### legacybridge_extract_tables_from_rtf

Extracts all tables from RTF as structured data.

```c
int legacybridge_extract_tables_from_rtf(
    const char* rtf_content,
    char** output_buffer,
    int* output_length
);
```

**Output Format (JSON):**
```json
{
    "tables": [
        {
            "index": 0,
            "rows": 5,
            "columns": 3,
            "data": [...]
        }
    ]
}
```

## Utility Functions

### legacybridge_test_connection

Tests if DLL is loaded and functioning correctly.

```c
int legacybridge_test_connection(void);
```

**Returns:**
- `1`: Connection successful
- `0`: Connection failed

**Use for:**
- Installation verification
- Runtime health checks
- Debugging connection issues

### legacybridge_get_version

Returns the library version string.

```c
const char* legacybridge_get_version(void);
```

**Returns:**
- Version string (e.g., "1.0.0")
- Do not free this string

### legacybridge_get_version_info

Gets detailed version information.

```c
int legacybridge_get_version_info(
    int* major,
    int* minor,
    int* patch
);
```

**Parameters:**
- `major`: Pointer to receive major version
- `minor`: Pointer to receive minor version
- `patch`: Pointer to receive patch version

**Returns:**
- `0`: Success
- Error code on failure

### legacybridge_get_last_error

Retrieves the last error message.

```c
int legacybridge_get_last_error(
    char* buffer,
    int buffer_size
);
```

**Parameters:**
- `buffer`: Buffer to receive error message
- `buffer_size`: Size of buffer

**Returns:**
- Number of bytes written
- `-1` if buffer too small

**Error Message Format:**
```
"Conversion failed: Invalid RTF structure at position 1234"
```

### legacybridge_free_string

Frees memory allocated by the library.

```c
void legacybridge_free_string(char* ptr);
```

**Parameters:**
- `ptr`: Pointer to string allocated by LegacyBridge

**Important:**
- Always call this for strings returned by the library
- Do not use standard `free()` function
- Safe to call with NULL pointer

## Language-Specific Wrappers

### VB6 Wrapper Functions

The VB6 wrapper (`LegacyBridge.bas`) provides these convenience functions:

```vb
' Simple conversion functions
Function ConvertRTFToMarkdown(rtfContent As String) As String
Function ConvertMarkdownToRTF(markdownContent As String) As String

' File operations
Function ConvertRTFFileToMD(inputPath As String, outputPath As String) As Boolean
Function ConvertMDFileToRTF(inputPath As String, outputPath As String) As Boolean

' Validation
Function ValidateRTFDocument(rtfContent As String) As String
Function ValidateMarkdownDocument(markdownContent As String) As String

' Batch operations
Function ConvertFolderRTFToMD(inputFolder As String, outputFolder As String) As Long
Function ConvertFolderMDToRTF(inputFolder As String, outputFolder As String) As Long

' Utilities
Function TestConnection() As Boolean
Function GetVersion() As String
Function GetLastError() As String
```

### VFP9 Class Methods

The VFP9 wrapper (`legacybridge.prg`) provides an object-oriented interface:

```foxpro
* Core methods
ConvertRTFToMarkdown(tcRTFContent)
ConvertMarkdownToRTF(tcMarkdownContent)

* File operations
ConvertRTFFileToMD(tcInputPath, tcOutputPath)
ConvertMDFileToRTF(tcInputPath, tcOutputPath)

* Properties
LastError           && Last error message
IsConnected        && Connection status
Version            && Library version

* Batch processing
ConvertFolderRTFToMD(tcInputFolder, tcOutputFolder)
GetBatchProgress()
CancelBatchOperation()
```

## Performance Considerations

### Memory Usage

- **Small documents (<100KB)**: ~2-5MB overhead
- **Medium documents (100KB-1MB)**: ~10-30MB overhead
- **Large documents (>1MB)**: Use file-based operations

### Processing Speed

- **Single conversion**: 0.024ms (MD→RTF), 0.049ms (RTF→MD)
- **Batch processing**: Up to 41,000 conversions/second
- **File operations**: Limited by disk I/O

### Best Practices

1. **Use appropriate function for document size**
   - <100KB: In-memory conversion
   - >100KB: File-based conversion
   - Multiple files: Batch operations

2. **Free memory promptly**
   ```c
   char* result = NULL;
   int length = 0;
   
   if (legacybridge_markdown_to_rtf(input, &result, &length) == 0) {
       // Use result
       process_rtf(result);
       
       // Free immediately
       legacybridge_free_string(result);
   }
   ```

3. **Handle errors gracefully**
   ```vb
   Dim result As String
   result = ConvertMarkdownToRTF(input)
   
   If result = "" Then
       MsgBox "Error: " & GetLastError()
   End If
   ```

---

**LegacyBridge API Reference v1.0.0**  
Last Updated: July 24, 2025  
© 2025 LegacyBridge. All rights reserved.