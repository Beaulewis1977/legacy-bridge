# LegacyBridge DLL Integration Guide

## Overview

LegacyBridge provides a C-compatible DLL for converting between Markdown and RTF formats, specifically designed for integration with legacy systems like Visual Basic 6 (VB6) and Visual FoxPro 9 (VFP9).

## Features

- **Bidirectional Conversion**: Convert RTF to Markdown and Markdown to RTF
- **Batch Processing**: Convert multiple documents in a single call
- **Error Handling**: Comprehensive error codes and error message retrieval
- **Memory Management**: Safe memory allocation and deallocation
- **Cross-Platform**: Supports Windows, Linux, and macOS

## Building the DLL

### Windows
```batch
build-dll.bat
```

### Linux/macOS
```bash
chmod +x build-dll.sh
./build-dll.sh
```

## File Structure

```
legacybridge/
├── lib/                    # Built DLL files
│   ├── legacybridge.dll   # Windows DLL
│   ├── legacybridge.dll.lib # Windows import library
│   └── legacybridge.def   # Windows DEF file
├── include/               # C header files
│   └── legacybridge.h
├── vb6-wrapper/          # VB6 integration
│   └── LegacyBridge.bas
├── vfp9-wrapper/         # VFP9 integration
│   └── legacybridge.prg
└── examples/             # Example code
    ├── vb6/
    └── vfp9/
```

## API Reference

### Core Functions

#### `legacybridge_rtf_to_markdown`
```c
int legacybridge_rtf_to_markdown(
    const char* rtf_content,
    char** output_buffer,
    int* output_length
);
```
Converts RTF content to Markdown format.

#### `legacybridge_markdown_to_rtf`
```c
int legacybridge_markdown_to_rtf(
    const char* markdown_content,
    char** output_buffer,
    int* output_length
);
```
Converts Markdown content to RTF format.

#### `legacybridge_free_string`
```c
void legacybridge_free_string(char* ptr);
```
Frees memory allocated by the library.

### Error Codes

- `0`: Success
- `-1`: Null pointer error
- `-2`: Invalid UTF-8 encoding
- `-3`: Conversion error
- `-4`: Memory allocation error

## VB6 Integration

### Setup

1. Copy `legacybridge.dll` to your application directory or System32
2. Add `LegacyBridge.bas` module to your VB6 project
3. Use the wrapper functions in your code

### Example Usage

```vb
' Simple conversion
Dim markdown As String
markdown = ConvertRtfToMarkdown("{\rtf1 Hello World\par}")

' Error handling
If Len(markdown) = 0 Then
    MsgBox "Conversion failed: " & GetLastError()
Else
    MsgBox "Converted: " & markdown
End If
```

### Deployment

For VB6 applications, include these files:
- `legacybridge.dll`
- Your VB6 executable

## VFP9 Integration

### Setup

1. Copy `legacybridge.dll` to your application directory
2. Include `legacybridge.prg` in your project
3. Instantiate the LegacyBridge class

### Example Usage

```foxpro
* Create converter instance
loConverter = CREATEOBJECT("LegacyBridge")

* Convert RTF to Markdown
lcMarkdown = loConverter.ConvertRtfToMarkdown(lcRtfContent)

* Check for errors
IF EMPTY(lcMarkdown)
    ? "Error: " + loConverter.cLastErrorMessage
ENDIF
```

### Deployment

For VFP9 applications, include:
- `legacybridge.dll`
- `legacybridge.prg` (can be compiled into your app)

## Supported RTF Features

- Text formatting (bold, italic, underline)
- Paragraphs and line breaks
- Headings
- Lists (bulleted and numbered)
- Basic tables
- Font specifications
- Color information

## Supported Markdown Features

- Headers (H1-H6)
- Bold and italic text
- Lists (ordered and unordered)
- Code blocks and inline code
- Links
- Tables (basic support)
- Line breaks and paragraphs

## Performance Considerations

- For large documents (>50KB), the library automatically uses an optimized pipeline
- Batch operations are more efficient than individual conversions
- Memory is allocated by the library and must be freed using `legacybridge_free_string`

## Error Handling Best Practices

1. Always check return codes
2. Free allocated memory even on error
3. Use `GetLastError()` for detailed error messages
4. Implement proper exception handling in your application

## Thread Safety

The library functions are thread-safe for different input/output buffers. However, error messages are stored per-thread.

## Troubleshooting

### Common Issues

1. **DLL not found**: Ensure the DLL is in the application directory or system PATH
2. **Access violation**: Check for null pointers and proper memory management
3. **Conversion errors**: Verify input format is valid RTF or Markdown
4. **Character encoding**: Ensure input strings are properly encoded

### Debug Tips

- Use `GetLibraryVersion()` to verify the DLL is loaded correctly
- Start with simple conversions before complex documents
- Check error codes and messages for specific issues

## License and Support

This library is part of the LegacyBridge project. For support and updates, visit the project repository.