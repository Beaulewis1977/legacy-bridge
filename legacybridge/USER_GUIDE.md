# LegacyBridge User Guide

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Basic Conversions](#basic-conversions)
4. [Advanced Features](#advanced-features)
5. [Working with Templates](#working-with-templates)
6. [Batch Processing](#batch-processing)
7. [Format Support](#format-support)
8. [Best Practices](#best-practices)
9. [Examples](#examples)
10. [FAQ](#frequently-asked-questions)

## Introduction

Welcome to LegacyBridge, the high-performance RTF ↔ Markdown converter designed for seamless integration with legacy systems. This guide will help you understand and utilize all features of LegacyBridge effectively.

### What is LegacyBridge?

LegacyBridge is a specialized document conversion library that enables:
- **Bidirectional conversion** between RTF and Markdown formats
- **Legacy system integration** with VB6, VFP9, and other older platforms
- **Enterprise-grade performance** with 41,000+ conversions per second
- **Comprehensive format support** including tables, lists, and Unicode

### Key Benefits

- **Preserve Formatting**: Maintain document structure and styling
- **Fast Processing**: Convert documents in milliseconds
- **Easy Integration**: Simple API for all programming environments
- **Reliable**: Robust error handling and recovery
- **Secure**: Protection against malicious content

## Getting Started

### Quick Start Example (VB6)

```vb
' Load the LegacyBridge module
' (Already done if you included LegacyBridge.bas)

' Convert Markdown to RTF
Dim markdown As String
Dim rtf As String

markdown = "# Welcome to LegacyBridge" & vbCrLf & _
           "This is **bold** and this is *italic*."

rtf = ConvertMarkdownToRTF(markdown)

' Display result
RichTextBox1.TextRTF = rtf
```

### Quick Start Example (VFP9)

```foxpro
* Create bridge instance
LOCAL loBridge, lcMarkdown, lcRTF

loBridge = CREATEOBJECT("LegacyBridge")

* Convert Markdown to RTF
lcMarkdown = "# Welcome to LegacyBridge" + CHR(13) + CHR(10) + ;
             "This is **bold** and this is *italic*."

lcRTF = loBridge.ConvertMarkdownToRTF(lcMarkdown)

* Display result
ThisForm.RichText1.Value = lcRTF
```

## Basic Conversions

### Markdown to RTF

Convert Markdown-formatted text to Rich Text Format:

```vb
' Simple conversion
Dim rtf As String
rtf = ConvertMarkdownToRTF("# My Document")

' With error handling
Dim rtf As String
Dim errorMsg As String

rtf = ConvertMarkdownToRTF("# My Document")
If rtf = "" Then
    errorMsg = GetLastError()
    MsgBox "Conversion failed: " & errorMsg
End If
```

### RTF to Markdown

Convert Rich Text Format documents to Markdown:

```vb
' Convert RTF to Markdown
Dim markdown As String
markdown = ConvertRTFToMarkdown(RichTextBox1.TextRTF)

' Save to file
Open "output.md" For Output As #1
Print #1, markdown
Close #1
```

### File-Based Conversions

Convert files directly without loading content into memory:

```vb
' Convert RTF file to Markdown file
Dim success As Boolean
success = ConvertRTFFileToMD("input.rtf", "output.md")

' Convert Markdown file to RTF file
success = ConvertMDFileToRTF("input.md", "output.rtf")
```

## Advanced Features

### Document Validation

Validate documents before conversion:

```vb
' Validate RTF document
Dim isValid As Boolean
Dim validationResult As String

validationResult = ValidateRTFDocument(rtfContent)
isValid = (InStr(validationResult, "valid") > 0)

' Validate Markdown document
validationResult = ValidateMarkdownDocument(mdContent)
```

### Text Extraction

Extract plain text from RTF documents:

```vb
' Extract plain text from RTF
Dim plainText As String
plainText = ExtractPlainText(rtfContent)

' Use for searching or indexing
If InStr(plainText, "important") > 0 Then
    MsgBox "Document contains important information"
End If
```

### Format Cleaning

Clean and normalize document formatting:

```vb
' Clean RTF formatting
Dim cleanRTF As String
cleanRTF = CleanRTFFormatting(messyRTF)

' Normalize Markdown
Dim normalizedMD As String
normalizedMD = NormalizeMarkdown(rawMarkdown)
```

## Working with Templates

### Built-in Templates

LegacyBridge includes three professional templates:

1. **Minimal**: Clean, simple formatting
2. **Professional**: Business-ready styling
3. **Academic**: Formal document structure

### Applying Templates

```vb
' Apply professional template to RTF
Dim styledRTF As String
styledRTF = ApplyRTFTemplate(plainRTF, "professional")

' Apply template to Markdown before conversion
Dim styledMD As String
styledMD = ApplyMarkdownTemplate(markdown, "academic")
```

### Custom Templates

Create and use custom templates:

```vb
' Create custom template from existing RTF
Dim success As Boolean
success = CreateRTFTemplate("company_style", sampleRTF)

' List available templates
Dim templates As String
templates = ListAvailableTemplates()
' Returns: "minimal,professional,academic,company_style"

' Apply custom template
Dim branded As String
branded = ApplyRTFTemplate(content, "company_style")
```

## Batch Processing

### Folder Conversion

Convert entire folders of documents:

```vb
' Convert all RTF files in folder to Markdown
Dim filesProcessed As Integer
filesProcessed = ConvertFolderRTFToMD("C:\Input", "C:\Output")

MsgBox "Processed " & filesProcessed & " files"

' Monitor progress
Dim progress As Integer
Do
    progress = GetBatchProgress()
    StatusBar1.Text = "Processing: " & progress & " files completed"
    DoEvents
Loop While progress < filesProcessed
```

### Batch Array Processing

Process multiple documents in memory:

```vb
' Prepare arrays
Dim markdownDocs(2) As String
markdownDocs(0) = "# Document 1"
markdownDocs(1) = "# Document 2"
markdownDocs(2) = "# Document 3"

' Convert batch
Dim rtfResults() As String
Dim count As Integer
count = BatchConvertMarkdownToRTF(markdownDocs, rtfResults)

' Process results
Dim i As Integer
For i = 0 To count - 1
    SaveRTFFile rtfResults(i), "output_" & i & ".rtf"
Next i
```

### Canceling Batch Operations

```vb
' Start batch operation in background
ConvertFolderRTFToMD "C:\LargeFolder", "C:\Output"

' User clicks cancel button
Private Sub btnCancel_Click()
    CancelBatchOperation
    MsgBox "Batch operation cancelled"
End Sub
```

## Format Support

### Supported Markdown Elements

#### Text Formatting
- **Bold**: `**text**` or `__text__`
- *Italic*: `*text*` or `_text_`
- ***Bold Italic***: `***text***`
- `Inline Code`: `` `code` ``

#### Headings
```markdown
# Heading 1
## Heading 2
### Heading 3
#### Heading 4
##### Heading 5
###### Heading 6
```

#### Lists

**Unordered Lists:**
```markdown
- Item 1
- Item 2
  - Nested item
  - Another nested item
- Item 3
```

**Ordered Lists:**
```markdown
1. First item
2. Second item
   1. Nested item
   2. Another nested item
3. Third item
```

#### Tables
```markdown
| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Data 1   | Data 2   | Data 3   |
| Data 4   | Data 5   | Data 6   |
```

#### Special Elements
- Horizontal rules: `---` or `***`
- Line breaks: Two spaces at end of line
- Blockquotes: `> Quote text`

### Supported RTF Features

- **Font Formatting**: Bold, italic, underline
- **Font Sizes**: Converted to appropriate headings
- **Colors**: Basic color support
- **Tables**: Full table structure
- **Lists**: Bulleted and numbered
- **Alignment**: Left, center, right
- **Unicode**: Full international character support

### Limitations

#### Partially Supported
- **Code blocks**: Rendered as plain text
- **Links**: Text preserved, URLs removed
- **Images**: Placeholder text only

#### Not Supported
- Strikethrough text
- Footnotes
- Syntax highlighting
- Custom fonts
- Advanced table features (merged cells)

## Best Practices

### Performance Optimization

1. **Use Batch Processing** for multiple files:
   ```vb
   ' Good: Process entire folder
   ConvertFolderRTFToMD sourceDir, destDir
   
   ' Avoid: Converting files one by one in loop
   ```

2. **Reuse Converter Instance** in VFP9:
   ```foxpro
   * Good: Create once, use many times
   PUBLIC goBridge
   goBridge = CREATEOBJECT("LegacyBridge")
   
   * Use goBridge throughout application
   ```

3. **Handle Large Documents** appropriately:
   ```vb
   ' For documents over 1MB, use file-based conversion
   If FileLen(filePath) > 1048576 Then
       ConvertRTFFileToMD filePath, outputPath
   Else
       ' Load and convert in memory
   End If
   ```

### Error Handling

Always implement proper error handling:

```vb
Public Function SafeConvert(markdown As String) As String
    On Error GoTo ErrorHandler
    
    Dim result As String
    result = ConvertMarkdownToRTF(markdown)
    
    If result = "" Then
        Dim errorMsg As String
        errorMsg = GetLastError()
        MsgBox "Conversion failed: " & errorMsg, vbExclamation
    End If
    
    SafeConvert = result
    Exit Function
    
ErrorHandler:
    MsgBox "Unexpected error: " & Err.Description
    SafeConvert = ""
End Function
```

### Memory Management

LegacyBridge handles memory automatically, but follow these guidelines:

1. **Don't Store References** to returned strings
2. **Process Results Immediately**
3. **Use File-Based Operations** for very large documents

### Unicode Handling

Ensure proper encoding:

```vb
' Save UTF-8 files properly
Dim fso As New FileSystemObject
Dim ts As TextStream
Set ts = fso.CreateTextFile("output.md", True, True) ' Unicode=True
ts.Write markdownContent
ts.Close
```

## Examples

### Example 1: Simple Document Converter

```vb
' Complete VB6 document converter
Private Sub ConvertDocument()
    ' Get input file
    CommonDialog1.Filter = "RTF Files|*.rtf|Markdown Files|*.md"
    CommonDialog1.ShowOpen
    
    If CommonDialog1.FileName = "" Then Exit Sub
    
    Dim inputFile As String
    Dim outputFile As String
    inputFile = CommonDialog1.FileName
    
    ' Determine conversion direction
    If LCase(Right(inputFile, 4)) = ".rtf" Then
        ' RTF to Markdown
        outputFile = Left(inputFile, Len(inputFile) - 4) & ".md"
        If ConvertRTFFileToMD(inputFile, outputFile) Then
            MsgBox "Converted to: " & outputFile
        End If
    Else
        ' Markdown to RTF
        outputFile = Left(inputFile, Len(inputFile) - 3) & ".rtf"
        If ConvertMDFileToRTF(inputFile, outputFile) Then
            MsgBox "Converted to: " & outputFile
        End If
    End If
End Sub
```

### Example 2: Batch Report Generator

```vb
' Generate RTF reports from Markdown templates
Private Sub GenerateReports()
    Dim template As String
    Dim reports As Collection
    Set reports = New Collection
    
    ' Load template
    template = LoadFile("report_template.md")
    
    ' Generate reports with different data
    Dim i As Integer
    For i = 1 To 100
        Dim report As String
        report = Replace(template, "{{REPORT_NUMBER}}", CStr(i))
        report = Replace(report, "{{DATE}}", Format(Now, "yyyy-mm-dd"))
        report = Replace(report, "{{STATUS}}", "Complete")
        
        ' Convert to RTF
        Dim rtf As String
        rtf = ConvertMarkdownToRTF(report)
        
        ' Apply professional template
        rtf = ApplyRTFTemplate(rtf, "professional")
        
        ' Save report
        SaveRTFFile rtf, "reports\report_" & i & ".rtf"
    Next i
    
    MsgBox "Generated 100 reports"
End Sub
```

### Example 3: Document Processing Pipeline

```vb
' Clean and standardize documents
Private Sub ProcessDocuments()
    Dim sourceFolder As String
    Dim processedFolder As String
    
    sourceFolder = "C:\Documents\Raw"
    processedFolder = "C:\Documents\Processed"
    
    ' Get all RTF files
    Dim file As String
    file = Dir(sourceFolder & "\*.rtf")
    
    Do While file <> ""
        ' Load RTF
        Dim rtf As String
        rtf = LoadFile(sourceFolder & "\" & file)
        
        ' Clean formatting
        rtf = CleanRTFFormatting(rtf)
        
        ' Convert to Markdown for processing
        Dim md As String
        md = ConvertRTFToMarkdown(rtf)
        
        ' Normalize Markdown
        md = NormalizeMarkdown(md)
        
        ' Convert back to RTF with template
        rtf = ConvertMarkdownToRTF(md)
        rtf = ApplyRTFTemplate(rtf, "professional")
        
        ' Save processed file
        SaveRTFFile rtf, processedFolder & "\" & file
        
        file = Dir
    Loop
    
    MsgBox "Processing complete"
End Sub
```

## Frequently Asked Questions

### General Questions

**Q: What's the maximum file size LegacyBridge can handle?**
A: LegacyBridge can handle files up to 100MB efficiently. For larger files, use streaming mode or split into chunks.

**Q: Does LegacyBridge preserve all formatting?**
A: Most common formatting is preserved. See [Format Support](#format-support) for details on supported features.

**Q: Can I use LegacyBridge in a multi-threaded application?**
A: Yes, LegacyBridge is thread-safe. Each thread should use its own instance in object-oriented wrappers.

### Technical Questions

**Q: Why does my Unicode text appear corrupted?**
A: Ensure your input is valid UTF-8. LegacyBridge expects UTF-8 encoded text for all operations.

**Q: How do I handle errors in VFP9?**
A: Check the LastError property after each operation:
```foxpro
IF NOT loBridge.ConvertMarkdownToRTF(lcMarkdown)
    ? "Error: " + loBridge.LastError
ENDIF
```

**Q: Can I convert multiple documents in parallel?**
A: Yes, use batch operations or create multiple instances for parallel processing.

### Performance Questions

**Q: How can I improve conversion speed?**
A: 
1. Use batch operations for multiple files
2. Enable caching in configuration
3. Use file-based operations for large documents
4. Ensure adequate system memory

**Q: Why is the first conversion slower?**
A: The first call initializes internal structures. Subsequent conversions will be faster.

### Integration Questions

**Q: Can I use LegacyBridge with .NET Core?**
A: Yes, use P/Invoke to call the DLL functions. See the Integration Guide for examples.

**Q: Is there a COM interface available?**
A: No, LegacyBridge uses standard C exports for maximum compatibility.

**Q: How do I deploy LegacyBridge with my application?**
A: Simply include legacybridge.dll with your application. No registration required.

---

**LegacyBridge User Guide v1.0.0**  
Last Updated: July 24, 2025  
© 2025 LegacyBridge. All rights reserved.