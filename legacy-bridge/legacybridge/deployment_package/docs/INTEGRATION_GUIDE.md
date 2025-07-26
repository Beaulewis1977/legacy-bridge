# LegacyBridge Integration Guide for VB6/VFP9 Developers

## Overview

LegacyBridge is a high-performance DLL for converting between RTF and Markdown formats. This guide will help you integrate LegacyBridge into your VB6 or Visual FoxPro 9 applications.

## System Requirements

- Windows XP or later (32-bit or 64-bit)
- Visual Basic 6.0 Runtime or Visual FoxPro 9.0
- Minimum 512MB RAM
- 10MB free disk space

## Installation

1. Copy `legacybridge.dll` to your application directory or `C:\Windows\System32`
2. Copy `legacybridge.h` header file for reference
3. Copy the appropriate wrapper file (VB6 or VFP9) to your project

## Function Categories

### Core Conversion Functions (7)
- `Rtf2MD` / `legacybridge_rtf_to_markdown` - Convert RTF to Markdown
- `MD2Rtf` / `legacybridge_markdown_to_rtf` - Convert Markdown to RTF
- `ConvertRtfFileToMd` / `legacybridge_convert_rtf_file_to_md` - File-based RTF to MD
- `ConvertMdFileToRtf` / `legacybridge_convert_md_file_to_rtf` - File-based MD to RTF
- `GetLastError` / `legacybridge_get_last_error` - Get error details
- `TestConnection` / `legacybridge_test_connection` - Test DLL loading
- `GetVersionInfo` / `legacybridge_get_version_info` - Get version information

### Validation Functions (3)
- `ValidateRtfDocument` / `legacybridge_validate_rtf_document` - Validate RTF syntax
- `ValidateMarkdownDocument` / `legacybridge_validate_markdown_document` - Validate Markdown
- `ExtractPlainText` / `legacybridge_extract_plain_text` - Extract plain text from RTF

### Batch Processing Functions (4)
- `ConvertFolderRtfToMd` / `legacybridge_convert_folder_rtf_to_md` - Batch RTF conversion
- `ConvertFolderMdToRtf` / `legacybridge_convert_folder_md_to_rtf` - Batch MD conversion
- `GetBatchProgress` / `legacybridge_get_batch_progress` - Get batch progress
- `CancelBatchOperation` / `legacybridge_cancel_batch_operation` - Cancel batch operation

### Processing Functions (2)
- `CleanRtfFormatting` / `legacybridge_clean_rtf_formatting` - Clean RTF formatting
- `NormalizeMarkdown` / `legacybridge_normalize_markdown` - Normalize Markdown

### Template Functions (5)
- `ApplyRtfTemplate` / `legacybridge_apply_rtf_template` - Apply RTF template
- `CreateRtfTemplate` / `legacybridge_create_rtf_template` - Create RTF template
- `ListAvailableTemplates` / `legacybridge_list_available_templates` - List templates
- `ApplyMarkdownTemplate` / `legacybridge_apply_markdown_template` - Apply MD template
- `ValidateTemplate` / `legacybridge_validate_template` - Validate template

### Database Functions (4)
- `ExportToCSV` / `legacybridge_export_to_csv` - Export RTF tables to CSV
- `ImportFromCSV` / `legacybridge_import_from_csv` - Import CSV to RTF table
- `ConvertTableToRtf` / `legacybridge_convert_table_to_rtf` - Convert table to RTF
- `ExtractTablesFromRtf` / `legacybridge_extract_tables_from_rtf` - Extract RTF tables

## VB6 Integration

### Basic Setup

1. Add `LegacyBridge.bas` module to your project
2. Ensure `legacybridge.dll` is in your app directory

### Simple Example

```vb
' Convert RTF to Markdown
Dim rtfContent As String
Dim markdownResult As String

rtfContent = "{\rtf1\ansi\deff0 {\fonttbl{\f0 Times New Roman;}} Hello World\par}"
markdownResult = ConvertRtfToMarkdown(rtfContent)

If Len(markdownResult) > 0 Then
    MsgBox "Conversion successful: " & markdownResult
Else
    MsgBox "Conversion failed: " & GetLastError()
End If
```

### File Conversion Example

```vb
' Convert RTF file to Markdown file
Dim result As Long
result = ConvertRtfFileToMd("C:\input.rtf", "C:\output.md")

If result = 0 Then
    MsgBox "File converted successfully!"
Else
    MsgBox "Conversion failed with error code: " & result
End If
```

### Batch Processing Example

```vb
' Convert all RTF files in a folder
Dim filesProcessed As Long
filesProcessed = ConvertFolderRtfToMd("C:\RTF_Files", "C:\MD_Files")

MsgBox "Processed " & filesProcessed & " files"
```

## VFP9 Integration

### Basic Setup

1. Copy `legacybridge.prg` to your project directory
2. Include it in your project: `SET PROCEDURE TO legacybridge.prg ADDITIVE`

### Simple Example

```foxpro
* Convert RTF to Markdown
LOCAL lcRtf, lcMarkdown
lcRtf = "{\rtf1\ansi\deff0 {\fonttbl{\f0 Times New Roman;}} Hello World\par}"
lcMarkdown = ConvertRtfToMarkdown(lcRtf)

IF !EMPTY(lcMarkdown)
    ? "Conversion successful:", lcMarkdown
ELSE
    ? "Conversion failed:", GetLastError()
ENDIF
```

### Database Integration

```foxpro
* Convert memo field containing RTF to Markdown
USE myTable
SCAN
    REPLACE markdown_field WITH ConvertRtfToMarkdown(rtf_field)
ENDSCAN
```

## Error Handling

### Error Codes

- `0` - Success
- `-1` - Null pointer error
- `-2` - Invalid UTF-8 encoding
- `-3` - Conversion error
- `-4` - Memory allocation error

### Best Practices

1. Always check return values
2. Use `GetLastError()` for detailed error messages
3. Free memory when using batch operations
4. Test with `TestConnection()` before first use

## Memory Management

The DLL handles all memory allocation internally. When receiving string outputs:

1. The DLL allocates memory for output strings
2. VB6/VFP9 wrapper automatically handles memory
3. No manual memory management required

## Performance Tips

1. Use batch operations for multiple files
2. Process large documents in chunks if needed
3. Use file-based operations for very large documents
4. Monitor batch progress with `GetBatchProgress()`

## Troubleshooting

### DLL Not Found
- Ensure `legacybridge.dll` is in the application directory or system path
- Check if Visual C++ Runtime is installed
- Use Dependency Walker to check for missing dependencies

### Conversion Failures
- Validate input with `ValidateRtfDocument()` or `ValidateMarkdownDocument()`
- Check encoding - DLL expects UTF-8
- Use `CleanRtfFormatting()` for problematic RTF

### Memory Issues
- Ensure adequate memory for large documents
- Use file-based operations for documents > 10MB
- Monitor system resources during batch operations

## Examples Directory

The deployment package includes complete working examples:

- `/examples/vb6/TestLegacyBridge.frm` - VB6 form with all features
- `/examples/vb6/LegacyBridge.bas` - VB6 wrapper module
- `/examples/vfp9/test_legacybridge.prg` - VFP9 test program
- `/examples/vfp9/legacybridge.prg` - VFP9 wrapper class

## Support

For issues or questions:
1. Check the error code reference
2. Review the examples
3. Ensure you're using the latest DLL version
4. Contact support with specific error messages

## Version History

- v1.0.0 - Initial release with 25 core functions
  - Full RTF ↔ Markdown conversion
  - Batch processing
  - Template support
  - Table handling
  - Error recovery

## License

This DLL is provided for use in legacy applications. See LICENSE file for details.