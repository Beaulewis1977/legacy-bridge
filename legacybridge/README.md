# LegacyBridge - Enterprise RTF ↔ Markdown Converter

[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](RELEASE_NOTES.md)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)](ENTERPRISE_INSTALLATION_GUIDE.md)
[![License](https://img.shields.io/badge/license-Commercial-green.svg)](LICENSE)
[![Performance](https://img.shields.io/badge/performance-41%2C000%2B%20ops%2Fsec-orange.svg)](FINAL_TEST_REPORT.md)

## Overview

LegacyBridge is a high-performance, enterprise-grade document conversion library that seamlessly bridges modern Markdown formatting with legacy RTF (Rich Text Format) systems. Designed specifically for integration with Visual Basic 6, Visual FoxPro 9, and other legacy platforms, LegacyBridge enables organizations to modernize their document workflows without replacing existing systems.

### Key Features

- 🚀 **Lightning-Fast Performance**: 41,000+ conversions per second
- 🔄 **Bidirectional Conversion**: Full RTF ↔ Markdown support
- 🏢 **Enterprise Ready**: Production-tested with comprehensive error handling
- 💾 **Legacy Compatible**: Native support for VB6, VFP9, and 32-bit systems
- 🛡️ **Security First**: Protection against malicious content and buffer overflows
- 📊 **Rich Format Support**: Tables, lists, Unicode, templates, and more
- 🔧 **Simple Integration**: 29 well-documented API functions
- 📦 **Minimal Footprint**: Only 720KB DLL size

## Quick Start

### For VB6 Developers

```vb
' Add LegacyBridge.bas to your project
' Convert Markdown to RTF
Dim rtf As String
rtf = ConvertMarkdownToRTF("# Hello World" & vbCrLf & "This is **bold** text.")

' Convert RTF to Markdown
Dim markdown As String
markdown = ConvertRTFToMarkdown(RichTextBox1.TextRTF)
```

### For VFP9 Developers

```foxpro
* Include legacybridge.prg
SET PROCEDURE TO legacybridge.prg ADDITIVE

* Create bridge instance
LOCAL loBridge
loBridge = CREATEOBJECT("LegacyBridge")

* Convert documents
lcRTF = loBridge.ConvertMarkdownToRTF("# Hello World")
lcMarkdown = loBridge.ConvertRTFToMarkdown(lcRTFContent)
```

## System Requirements

### Minimum
- Windows XP SP3 or later
- 512MB RAM
- 10MB disk space
- Visual C++ 2015-2022 Redistributable (x86)

### Recommended
- Windows 10/11 or Windows Server 2016+
- 2GB+ RAM
- 50MB disk space

## Installation

### Quick Installation

1. Download `legacybridge-v1.0.0-win32.zip`
2. Extract to your application directory
3. Copy `legacybridge.dll` to your application folder
4. Add the appropriate wrapper module to your project

For detailed installation instructions, see the [Enterprise Installation Guide](ENTERPRISE_INSTALLATION_GUIDE.md).

## Documentation

- 📖 [User Guide](USER_GUIDE.md) - Complete usage guide with examples
- 🔧 [API Reference](API_REFERENCE.md) - Detailed documentation for all 29 functions
- 🚀 [Installation Guide](ENTERPRISE_INSTALLATION_GUIDE.md) - Enterprise deployment instructions
- 🐛 [Troubleshooting Guide](TROUBLESHOOTING_GUIDE.md) - Common issues and solutions
- 📝 [Release Notes](RELEASE_NOTES.md) - Version history and changes

## Performance Benchmarks

| Operation | Documents/Second | Latency | Memory Usage |
|-----------|-----------------|---------|--------------|
| Markdown → RTF | 41,131 | 0.024ms | <30MB |
| RTF → Markdown | 20,535 | 0.049ms | <30MB |
| Batch Processing | 1,800+ | - | <100MB |
| File Operations | 450+ | - | <50MB |

## Supported Formats

### Markdown Elements
- ✅ Headings (H1-H6)
- ✅ Bold, Italic, Combined formatting
- ✅ Ordered and unordered lists
- ✅ Tables with formatting
- ✅ Line breaks and paragraphs
- ✅ Horizontal rules
- ✅ Unicode and emoji
- ⚠️ Code blocks (as plain text)
- ⚠️ Links (text only)
- ❌ Images (placeholder text)

### RTF Features
- ✅ Font formatting (bold, italic, underline)
- ✅ Font sizes and colors
- ✅ Tables and cells
- ✅ Lists (bulleted and numbered)
- ✅ Text alignment
- ✅ Page breaks
- ✅ Unicode text

## API Overview

### Core Functions
```c
// Basic conversions
legacybridge_rtf_to_markdown()
legacybridge_markdown_to_rtf()

// File operations
legacybridge_convert_rtf_file_to_md()
legacybridge_convert_md_file_to_rtf()

// Batch processing
legacybridge_batch_rtf_to_markdown()
legacybridge_convert_folder_rtf_to_md()

// Validation
legacybridge_validate_rtf_document()
legacybridge_validate_markdown_document()
```

See the complete [API Reference](API_REFERENCE.md) for all 29 functions.

## Integration Examples

### Document Management System
```vb
' Modernize legacy documents
Public Function ModernizeDocuments(folderPath As String) As Long
    Dim count As Long
    count = ConvertFolderRTFToMD(folderPath, folderPath & "_modern")
    ModernizeDocuments = count
End Function
```

### Report Generation
```vb
' Generate RTF reports from Markdown templates
Public Function GenerateReport(template As String, data As Dictionary) As String
    Dim markdown As String
    markdown = ProcessTemplate(template, data)
    
    Dim rtf As String
    rtf = ConvertMarkdownToRTF(markdown)
    rtf = ApplyRTFTemplate(rtf, "professional")
    
    GenerateReport = rtf
End Function
```

### Batch Processing
```vb
' Convert entire database of documents
Public Sub ConvertDatabase()
    Dim files As Collection
    Set files = GetAllDocuments()
    
    Dim results() As String
    BatchConvertMarkdownToRTF files, results
    
    SaveResults results
End Sub
```

## Testing & Quality

- ✅ **58 comprehensive tests** - 100% pass rate
- ✅ **Security validated** - No vulnerabilities found
- ✅ **Memory safe** - No leaks detected
- ✅ **Performance verified** - Exceeds all targets
- ✅ **Production ready** - Extensively field tested

See the [Final Test Report](FINAL_TEST_REPORT.md) for detailed results.

## Support

### Documentation
- Comprehensive guides included
- API reference with examples
- Troubleshooting guide
- Video tutorials available

### Technical Support
- Email: support@legacybridge.com
- Knowledge Base: https://docs.legacybridge.com
- Priority support for enterprise customers

### Community
- Forum: https://forum.legacybridge.com
- Stack Overflow: Tag `legacybridge`

## License

LegacyBridge is available under commercial license. Contact sales@legacybridge.com for pricing and terms.

### Trial Version
A 30-day trial version is available for evaluation. The trial version is fully functional with a small watermark in converted documents.

## Contributing

While LegacyBridge is a commercial product, we welcome:
- Bug reports
- Feature requests
- Documentation improvements
- Community examples

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Roadmap

### Version 1.1 (Q3 2025)
- Strikethrough text support
- Enhanced code block formatting
- Performance optimizations

### Version 1.2 (Q4 2025)
- Footnote support
- Image embedding
- Extended templates

### Version 2.0 (2026)
- Syntax highlighting
- Advanced tables
- Plugin architecture

## Credits

LegacyBridge is built with:
- Rust for performance and safety
- Careful attention to legacy system compatibility
- Extensive real-world testing
- Feedback from enterprise users

---

**LegacyBridge** - *Bridging the gap between modern and legacy document formats*

© 2025 LegacyBridge. All rights reserved.