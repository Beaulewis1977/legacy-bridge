# 5-Minute Quick Start Guide

Get LegacyBridge up and running in just 5 minutes! This guide will walk you through the fastest path to converting your first document.

## Step 1: Download (1 minute)

Download the latest LegacyBridge release for your platform:

- **Windows**: [legacybridge-v1.0.0-windows-x86.zip](#)
- **Linux**: [legacybridge-v1.0.0-linux-x64.tar.gz](#)
- **macOS**: [legacybridge-v1.0.0-macos.dmg](#)

## Step 2: Extract and Install (2 minutes)

### Windows
1. Extract the ZIP file to `C:\LegacyBridge`
2. Run `install.bat` as Administrator
3. The installer will:
   - Copy files to the correct location
   - Register the DLL
   - Add to system PATH
   - Run validation tests

### Linux/macOS
```bash
# Extract the archive
tar -xzf legacybridge-v1.0.0-linux-x64.tar.gz
cd legacybridge

# Run the installer
sudo ./install.sh

# The installer will:
# - Copy files to /usr/local/lib
# - Set up permissions
# - Run validation tests
```

## Step 3: Verify Installation (30 seconds)

### Quick Test
Run the validation tool to ensure everything is working:

```bash
# Windows
validate_installation.exe

# Linux/macOS
./validate_installation
```

You should see:
```
✓ DLL loaded successfully
✓ All 29 functions exported correctly
✓ Basic conversion test passed
✓ Performance benchmark passed
LegacyBridge v1.0.0 is ready to use!
```

## Step 4: Your First Conversion (1.5 minutes)

### Using the Command Line Tool

```bash
# Convert RTF to Markdown
legacybridge convert input.rtf output.md

# Convert Markdown to RTF
legacybridge convert input.md output.rtf

# Batch convert a folder
legacybridge batch /path/to/rtf/files /path/to/output
```

### Using VB6

```vb
' Add LegacyBridge.bas to your project, then:
Private Sub ConvertDocument()
    Dim markdown As String
    
    ' Convert RTF from RichTextBox to Markdown
    markdown = ConvertRTFToMarkdown(RichTextBox1.TextRTF)
    
    ' Display result
    Text1.Text = markdown
    
    ' Convert back to RTF
    RichTextBox2.TextRTF = ConvertMarkdownToRTF(markdown)
End Sub
```

### Using VFP9

```foxpro
* Create LegacyBridge instance
LOCAL loBridge, lcMarkdown, lcRTF
loBridge = CREATEOBJECT("LegacyBridge")

* Convert RTF file to Markdown
IF loBridge.ConvertRTFFileToMD("document.rtf", "output.md")
    ? "Conversion successful!"
ENDIF

* Convert Markdown string to RTF
lcRTF = loBridge.ConvertMarkdownToRTF("# Hello World")
```

### Using .NET

```csharp
using LegacyBridge;

class Program
{
    static void Main()
    {
        var converter = new LegacyBridgeConverter();
        
        // Convert Markdown file to RTF
        converter.ConvertFile("README.md", "output.rtf");
        
        // Convert RTF string to Markdown
        string markdown = converter.ConvertRtfToMarkdown(rtfContent);
        
        Console.WriteLine("Conversion complete!");
    }
}
```

## Step 5: Explore Advanced Features (30 seconds)

### Batch Processing
Convert entire folders of documents:

```vb
' VB6 Example
Dim filesConverted As Long
filesConverted = ConvertFolderRTFToMD("C:\Documents\RTF", "C:\Documents\Markdown")
MsgBox "Converted " & filesConverted & " files!"
```

### Template System
Apply professional formatting:

```csharp
// Apply a template during conversion
string styledRtf = converter.ConvertMarkdownToRtf(markdown, "professional");
```

### Error Handling
Always check for errors:

```vb
' VB6 Error Handling
Dim result As String
result = ConvertMarkdownToRTF(inputText)

If result = "" Then
    MsgBox "Conversion failed: " & GetLastError()
Else
    ' Use the result
End If
```

## Common Quick Start Issues

### "DLL not found" Error
- **Windows**: Ensure `legacybridge.dll` is in your application directory or system PATH
- **Linux**: Run `sudo ldconfig` after installation

### "Function not exported" Error
- Make sure you're using the exact function names from the API
- Check that you have the correct 32-bit or 64-bit version

### Slow Performance
- Enable optimizations: Set environment variable `LEGACYBRIDGE_ENABLE_SIMD=1`
- Use batch operations for multiple files

## What's Next?

Now that you have LegacyBridge running:

1. **Read the Full Documentation**
   - [API Reference](/docs/api/reference/README.md) - All 29 functions explained
   - [Integration Guides](/docs/guides/legacy-integration/) - Platform-specific guides
   - [Best Practices](/docs/guides/performance/optimization-guide.md) - Performance tips

2. **Try Advanced Features**
   - [Template System](/docs/guides/legacy-integration/vb6-complete.md#templates)
   - [Batch Processing](/docs/api/reference/core-functions.md#batch-processing)
   - [Error Recovery](/docs/troubleshooting/common-issues.md)

3. **Get Help**
   - [Interactive Troubleshooter](/docs/troubleshooting/interactive-wizard/)
   - [Community Forum](#)
   - [Email Support](mailto:blewisxx@gmail.com)

## Quick Reference Card

### Essential Functions
```c
// Core conversions
legacybridge_rtf_to_markdown(rtf_content)
legacybridge_markdown_to_rtf(markdown_content)

// File operations
legacybridge_convert_rtf_file_to_md(input_path, output_path)
legacybridge_convert_md_file_to_rtf(input_path, output_path)

// Batch processing
legacybridge_convert_folder_rtf_to_md(input_folder, output_folder)
legacybridge_convert_folder_md_to_rtf(input_folder, output_folder)

// Utilities
legacybridge_test_connection()
legacybridge_get_version()
legacybridge_get_last_error()
legacybridge_free_string(ptr)
```

---

**Congratulations!** You've successfully set up LegacyBridge and converted your first documents. 

Ready to dive deeper? Check out the [First Conversion Tutorial](first-conversion.md) for a more detailed walkthrough.