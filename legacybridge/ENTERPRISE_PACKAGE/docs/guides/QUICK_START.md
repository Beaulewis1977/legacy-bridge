# LegacyBridge Quick Start Guide

Get up and running with LegacyBridge in minutes!

## 1. Installation (< 2 minutes)

### Windows
```batch
cd ENTERPRISE_PACKAGE\installation
install.bat
```

### Linux
```bash
cd ENTERPRISE_PACKAGE/installation
sudo ./install.sh
```

## 2. Verify Installation (< 1 minute)

### Windows
```batch
cd C:\Program Files\LegacyBridge\tools
perf_test.exe
```

### Linux
```bash
legacybridge-perf
```

You should see output showing successful conversions and performance metrics.

## 3. Your First Conversion

### Visual Basic 6

1. Add to your project:
```vb
' Add LegacyBridge.bas module to your project
```

2. Convert RTF to Markdown:
```vb
Dim markdown As String
markdown = ConvertRTFToMarkdown("{\rtf1 Hello World!}")
MsgBox markdown
```

### Visual FoxPro 9

1. Include the wrapper:
```foxpro
SET PROCEDURE TO legacybridge.prg ADDITIVE
```

2. Create instance and convert:
```foxpro
oLB = CREATEOBJECT("LegacyBridge")
lcMarkdown = oLB.RtfToMarkdown("{\rtf1 Hello World!}")
? lcMarkdown
```

### C/C++

1. Include header:
```c
#include <legacybridge.h>
```

2. Convert and free memory:
```c
char* markdown = rtf_to_markdown("{\\rtf1 Hello World!}");
printf("%s\n", markdown);
free_string(markdown);
```

### Python

```python
from ctypes import *
lib = CDLL('./legacybridge.dll')  # or .so on Linux

# Setup
lib.rtf_to_markdown.restype = c_char_p
lib.rtf_to_markdown.argtypes = [c_char_p]

# Convert
result = lib.rtf_to_markdown(b"{\\rtf1 Hello World!}")
print(result.decode('utf-8'))
lib.free_string(result)
```

## 4. Batch Processing

Convert multiple documents at once:

### VB6
```vb
Dim results() As String
results = BatchConvertRTFToMarkdown(rtfArray)
```

### C
```c
size_t count;
char** results = batch_rtf_to_markdown(rtf_docs, 10, &count);
```

## 5. Error Handling

Always check for errors:

### VB6
```vb
Dim result As String
result = ConvertRTFToMarkdown(rtfContent)
If result = "" Then
    MsgBox "Error: " & GetLastError()
End If
```

### C
```c
char* result = rtf_to_markdown(input);
if (!result) {
    printf("Error: %s\n", get_last_error());
}
```

## 6. Best Practices

1. **Always free memory** - Use `free_string()` in C or rely on wrappers
2. **Check errors** - Conversion can fail on invalid input
3. **Use batch operations** - More efficient for multiple documents
4. **Validate input** - Use validation functions when unsure

## 7. Common Conversions

### RTF → Markdown
```
{\rtf1\ansi\b Bold text\b0}  →  **Bold text**
{\rtf1\ansi\i Italic\i0}     →  *Italic*
{\rtf1\ansi\ul Underline\ul0} →  <u>Underline</u>
```

### Markdown → RTF
```
# Heading 1     →  {\rtf1\ansi\fs48 Heading 1\par}
**Bold**        →  {\rtf1\ansi\b Bold\b0}
- List item     →  {\rtf1\ansi\bullet List item\par}
```

## 8. Performance Tips

- **Reuse buffers** with `_safe` functions
- **Batch similar operations** together
- **Pre-validate** complex documents
- **Monitor memory** in long-running processes

## 9. Next Steps

- Review [API Reference](../api/API_REFERENCE.md) for all functions
- See [Examples](../../examples/) for complete applications
- Read [Integration Guide](./INTEGRATION_GUIDE.md) for detailed setup
- Run performance tests with your data

## 10. Getting Help

- **Documentation**: `docs/` directory
- **Examples**: Working code in `examples/`
- **Support**: support@legacybridge.com

---

Congratulations! You're now ready to use LegacyBridge in your applications.