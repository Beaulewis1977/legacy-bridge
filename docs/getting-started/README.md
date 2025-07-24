# Getting Started with LegacyBridge

Welcome to LegacyBridge! This guide will help you get up and running with the world's fastest RTF ↔ Markdown converter in just a few minutes.

## What is LegacyBridge?

LegacyBridge is a lightning-fast, enterprise-grade document conversion solution that bridges modern Markdown with legacy RTF systems. It delivers superior performance in just 720KB compared to Pandoc's 100MB solution.

## Quick Navigation

### 🚀 For Immediate Start
- [5-Minute Quick Start Guide](quick-start.md) - Get converting documents right away
- [First Conversion Tutorial](first-conversion.md) - Step-by-step walkthrough

### 💻 Installation Guides
Choose your platform:
- [Windows Installation](installation/windows.md) - Windows 7/8/10/11 and Server editions
- [Linux Installation](installation/linux.md) - Ubuntu, Debian, CentOS, and others
- [macOS Installation](installation/macos.md) - macOS 10.9 and later
- [Docker Deployment](installation/docker.md) - Container-based deployment

### 🎯 Choose Your Path

#### I'm a VB6 Developer
1. Read the [VB6 Quick Start](#vb6-quick-start) section below
2. Follow the [Windows Installation Guide](installation/windows.md)
3. Check out [VB6 Complete Integration Guide](/docs/guides/legacy-integration/vb6-complete.md)

#### I'm a VFP9 Developer
1. Read the [VFP9 Quick Start](#vfp9-quick-start) section below
2. Follow the [Windows Installation Guide](installation/windows.md)
3. Check out [VFP9 Complete Integration Guide](/docs/guides/legacy-integration/vfp9-complete.md)

#### I'm a .NET Developer
1. Read the [.NET Quick Start](#net-quick-start) section below
2. Choose your platform installation guide
3. Check out [.NET Integration Guide](/docs/guides/legacy-integration/dotnet-integration.md)

#### I'm a System Administrator
1. Review [System Requirements](#system-requirements)
2. Read [Enterprise Deployment Guide](/docs/guides/enterprise-deployment/architecture-planning.md)
3. Follow [Enterprise Installation Guide](/legacybridge/ENTERPRISE_INSTALLATION_GUIDE.md)

## System Requirements

### Minimum Requirements
- **OS**: Windows XP SP3 / Linux kernel 2.6+ / macOS 10.9+
- **RAM**: 512MB available memory
- **Storage**: 10MB disk space
- **Runtime**: Visual C++ 2015-2022 Redistributable (Windows x86)

### Recommended Specifications
- **OS**: Windows 10/11 or Windows Server 2016+
- **RAM**: 2GB+ available memory
- **Storage**: 50MB disk space
- **CPU**: Multi-core processor for optimal batch performance

## VB6 Quick Start

```vb
' 1. Add LegacyBridge.bas to your project
' 2. Convert Markdown to RTF
Dim rtfContent As String
rtfContent = ConvertMarkdownToRTF("# Hello World" & vbCrLf & "This is **bold** text.")

' 3. Convert RTF to Markdown  
Dim markdownContent As String
markdownContent = ConvertRTFToMarkdown(RichTextBox1.TextRTF)

' 4. Test the connection
If TestConnection() Then
    MsgBox "LegacyBridge is ready!"
End If
```

## VFP9 Quick Start

```foxpro
* 1. Include legacybridge.prg in your project
SET PROCEDURE TO legacybridge.prg ADDITIVE

* 2. Create bridge instance
LOCAL loBridge
loBridge = CREATEOBJECT("LegacyBridge")

* 3. Convert documents
lcRTF = loBridge.ConvertMarkdownToRTF("# Welcome to LegacyBridge")
lcMarkdown = loBridge.ConvertRTFToMarkdown(lcRTFContent)

* 4. Check connection
IF loBridge.TestConnection()
    ? "LegacyBridge is ready!"
ENDIF
```

## .NET Quick Start

```csharp
using LegacyBridge;

// Create converter instance
var converter = new LegacyBridgeConverter();

// Convert Markdown to RTF
string rtfContent = converter.ConvertMarkdownToRtf("# Hello World\n\nThis is **bold** text.");

// Convert RTF to Markdown
string markdownContent = converter.ConvertRtfToMarkdown(rtfContent);

// Test connection
if (converter.TestConnection())
{
    Console.WriteLine("LegacyBridge is ready!");
}
```

## Key Features at a Glance

### 🚀 Performance
- 41,000+ conversions per second
- 720KB DLL size (99.3% smaller than Pandoc)
- Zero memory leaks

### 🔄 Conversion Capabilities
- Bidirectional RTF ↔ Markdown conversion
- 95%+ conversion fidelity
- Batch processing support
- Template system for consistent formatting

### 🛡️ Enterprise Ready
- Thread-safe operations
- Comprehensive error handling
- Security audited
- Production hardened

### 🔌 Integration Support
- VB6 and VFP9 native support
- .NET Framework and Core
- C/C++ direct integration
- Python and JavaScript wrappers

## Next Steps

1. **Install LegacyBridge**: Follow the installation guide for your platform
2. **Try Your First Conversion**: Use the [First Conversion Tutorial](first-conversion.md)
3. **Explore the API**: Browse the [API Reference](/docs/api/reference/README.md)
4. **Check Examples**: Review code examples for your language
5. **Join the Community**: Get help and share your experience

## Need Help?

- 📖 Check the [Troubleshooting Guide](/docs/troubleshooting/common-issues.md)
- 🔍 Use the [Interactive Troubleshooting Wizard](/docs/troubleshooting/interactive-wizard/)
- 💬 Contact support at [blewisxx@gmail.com](mailto:blewisxx@gmail.com)

---

Ready to get started? Head to the [Quick Start Guide](quick-start.md) →