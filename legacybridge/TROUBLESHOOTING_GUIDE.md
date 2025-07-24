# 🐛 LegacyBridge Troubleshooting Guide

*Common issues and solutions for smooth operation*

---

## 📚 Navigation

**🏠 [← Back to Main README](README.md)**

### 📋 Documentation Links
- **[📖 User Guide](USER_GUIDE.md)** - Complete usage guide with examples
- **[🔌 API Reference](API_REFERENCE.md)** - Detailed documentation for all 29 functions
- **[🚀 Installation Guide](ENTERPRISE_INSTALLATION_GUIDE.md)** - Enterprise deployment instructions
- **[📝 Release Notes](RELEASE_NOTES.md)** - Version history and changes

---

## 👨‍💻 About the Developer

**Built with ❤️ by [Beau Lewis](mailto:blewisxx@gmail.com)**

*If LegacyBridge has helped you, consider supporting the project:*
- ☕ **[Ko-fi](https://ko-fi.com/beaulewis)** - Buy me a coffee
- 💳 **[Venmo](https://venmo.com/beauintulsa)** - Quick thanks

---

## 📚 Table of Contents

1. [Common Issues](#common-issues)
2. [Installation Problems](#installation-problems)
3. [Runtime Errors](#runtime-errors)
4. [Conversion Issues](#conversion-issues)
5. [Performance Problems](#performance-problems)
6. [Integration Challenges](#integration-challenges)
7. [Diagnostic Tools](#diagnostic-tools)
8. [Error Code Reference](#error-code-reference)
9. [Getting Help](#getting-help)

## Common Issues

### Issue: DLL Not Found

**Symptoms:**
- Error 53: File not found
- "legacybridge.dll could not be found"
- "The specified module could not be found"

**Solutions:**

1. **Verify DLL Location**
   ```vb
   ' Check if DLL exists in application directory
   If Dir(App.Path & "\legacybridge.dll") = "" Then
       MsgBox "DLL not found in: " & App.Path
   End If
   ```

2. **Check System PATH**
   ```cmd
   echo %PATH%
   where legacybridge.dll
   ```

3. **Copy to System Directory**
   ```cmd
   copy legacybridge.dll C:\Windows\System32\
   copy legacybridge.dll C:\Windows\SysWOW64\  (for 64-bit systems)
   ```

### Issue: Missing Dependencies

**Symptoms:**
- "The application failed to initialize properly"
- Error code 0xc0000135
- "VCRUNTIME140.dll not found"

**Solution:**
Install Visual C++ 2015-2022 Redistributable (x86):
```
https://aka.ms/vs/17/release/vc_redist.x86.exe
```

### Issue: Access Violation

**Symptoms:**
- Application crashes when calling DLL
- "Access violation reading location 0x00000000"
- Unexpected program termination

**Solutions:**

1. **Check Null Pointers**
   ```vb
   ' Always check for empty strings
   If Len(inputText) = 0 Then
       MsgBox "Input cannot be empty"
       Exit Sub
   End If
   ```

2. **Verify Calling Convention**
   ```vb
   ' Ensure CDECL calling convention in declarations
   Private Declare Function legacybridge_test_connection _
       Lib "legacybridge.dll" () As Long
   ```

3. **Use Proper String Handling**
   ```vb
   ' For VB6, ensure proper string conversion
   Dim result As String
   result = Space$(MaxBufferSize)
   ' ... use result buffer
   result = Left$(result, InStr(result, Chr$(0)) - 1)
   ```

## Installation Problems

### Problem: Installation Fails on Windows XP

**Cause:** Missing service pack or updates

**Solution:**
1. Ensure Windows XP SP3 is installed
2. Install .NET Framework 2.0 or higher
3. Update Internet Explorer to version 8

### Problem: Anti-virus Blocks Installation

**Symptoms:**
- "Threat detected" warning
- DLL quarantined or deleted
- Installation interrupted

**Solution:**
1. Temporarily disable anti-virus
2. Add exception for legacybridge.dll
3. Whitelist installation directory
4. Re-enable anti-virus after installation

### Problem: Insufficient Privileges

**Symptoms:**
- "Access denied" errors
- Cannot write to Program Files
- Registry access failures

**Solution:**
```cmd
:: Run as Administrator
runas /user:Administrator "install_legacybridge.bat"

:: Or right-click installer and select "Run as Administrator"
```

## Runtime Errors

### Error: Invalid UTF-8 Encoding

**Error Code:** -2 (LB_ERROR_INVALID_UTF8)

**Cause:** Input contains invalid character encoding

**Solutions:**

1. **Validate Input Encoding**
   ```vb
   Function IsValidUTF8(text As String) As Boolean
       On Error GoTo InvalidUTF8
       Dim bytes() As Byte
       bytes = StrConv(text, vbFromUnicode)
       ' Additional validation logic
       IsValidUTF8 = True
       Exit Function
   InvalidUTF8:
       IsValidUTF8 = False
   End Function
   ```

2. **Clean Input Text**
   ```vb
   Function CleanText(input As String) As String
       Dim i As Long
       Dim result As String
       
       For i = 1 To Len(input)
           If AscW(Mid$(input, i, 1)) < 128 Then
               result = result & Mid$(input, i, 1)
           End If
       Next i
       
       CleanText = result
   End Function
   ```

### Error: Memory Allocation Failed

**Error Code:** -4 (LB_ERROR_ALLOCATION)

**Cause:** Insufficient memory or very large document

**Solutions:**

1. **Check Available Memory**
   ```vb
   ' Use Windows API to check memory
   Dim memStatus As MEMORYSTATUS
   GlobalMemoryStatus memStatus
   
   If memStatus.dwAvailPhys < 100000000 Then ' 100MB
       MsgBox "Low memory warning"
   End If
   ```

2. **Process in Chunks**
   ```vb
   ' For large documents, split into sections
   Const CHUNK_SIZE = 1048576 ' 1MB chunks
   Dim chunks() As String
   ' ... split document into chunks
   ```

3. **Use File-Based Operations**
   ```vb
   ' Instead of loading entire file
   ConvertRTFFileToMD largeFile, outputFile
   ```

## Conversion Issues

### Issue: Formatting Lost During Conversion

**Symptoms:**
- Bold/italic text not preserved
- Tables appear as plain text
- Lists lose structure

**Solutions:**

1. **Verify Input Format**
   ```vb
   ' Validate RTF before conversion
   Dim validation As String
   validation = ValidateRTFDocument(rtfContent)
   
   If InStr(validation, "error") > 0 Then
       MsgBox "Invalid RTF: " & validation
   End If
   ```

2. **Use Appropriate Template**
   ```vb
   ' Apply template for better formatting
   Dim rtf As String
   rtf = ConvertMarkdownToRTF(markdown)
   rtf = ApplyRTFTemplate(rtf, "professional")
   ```

### Issue: Unicode Characters Corrupted

**Symptoms:**
- Question marks instead of special characters
- Asian languages display incorrectly
- Emoji become garbage characters

**Solutions:**

1. **Ensure UTF-8 Throughout**
   ```vb
   ' Save files with UTF-8 encoding
   Dim fso As New FileSystemObject
   Dim stream As TextStream
   Set stream = fso.CreateTextFile(filename, True, True) ' Unicode=True
   stream.Write content
   stream.Close
   ```

2. **Check Regional Settings**
   - Control Panel → Region and Language
   - Set "Language for non-Unicode programs" appropriately

### Issue: Tables Not Converting Properly

**Symptoms:**
- Table structure lost
- Cells merged incorrectly
- Alignment issues

**Solutions:**

1. **Simplify Table Structure**
   ```markdown
   | Simple | Table |
   |--------|-------|
   | Data 1 | Data 2|
   ```

2. **Use Table Extraction**
   ```vb
   ' Extract and rebuild tables
   Dim tables As String
   tables = ExtractTablesFromRTF(rtfContent)
   ' Process tables separately
   ```

## Performance Problems

### Issue: Slow Conversion Speed

**Symptoms:**
- Conversions take several seconds
- Application freezes during conversion
- High CPU usage

**Solutions:**

1. **Enable Batch Processing**
   ```vb
   ' Convert multiple files at once
   Dim files() As String
   ' ... populate file array
   BatchConvertMarkdownToRTF files, results
   ```

2. **Optimize Document Size**
   ```vb
   ' Check document size before processing
   If FileLen(docPath) > 5242880 Then ' 5MB
       ' Use file-based conversion
       ConvertRTFFileToMD docPath, outputPath
   Else
       ' Use memory-based conversion
       content = LoadFile(docPath)
       result = ConvertRTFToMarkdown(content)
   End If
   ```

3. **Disable Unnecessary Features**
   ```ini
   ; In legacybridge.ini
   [Performance]
   ValidateInput=false
   EnableCache=true
   ChunkSize=1048576
   ```

### Issue: High Memory Usage

**Symptoms:**
- Memory usage grows continuously
- Out of memory errors
- System becomes sluggish

**Solutions:**

1. **Proper Memory Management**
   ```vb
   ' Always process and release
   Dim result As String
   result = ConvertMarkdownToRTF(input)
   ProcessResult result
   result = "" ' Release string
   ```

2. **Monitor Memory Usage**
   ```vb
   Private Sub MonitorMemory()
       Dim memBefore As Long
       Dim memAfter As Long
       
       memBefore = GetCurrentMemoryUsage()
       ' ... perform operations
       memAfter = GetCurrentMemoryUsage()
       
       If memAfter - memBefore > 10485760 Then ' 10MB
           MsgBox "Possible memory leak detected"
       End If
   End Sub
   ```

## Integration Challenges

### VB6 Specific Issues

**Problem: String Length Limitations**
```vb
' VB6 has string length limits
Const MAX_STRING_LENGTH = 2147483647

Function SafeConvert(input As String) As String
    If Len(input) > MAX_STRING_LENGTH / 2 Then
        ' Use file-based conversion instead
        Dim tempFile As String
        tempFile = GetTempFileName()
        SaveToFile input, tempFile
        ' ... convert file
    Else
        SafeConvert = ConvertMarkdownToRTF(input)
    End If
End Function
```

**Problem: IDE Crashes During Debug**
```vb
' Disable error handling in IDE
#If DEBUGMODE Then
    On Error GoTo 0  ' Let IDE handle errors
#Else
    On Error GoTo ErrorHandler
#End If
```

### VFP9 Specific Issues

**Problem: Object Reference Errors**
```foxpro
* Ensure proper instantiation
LOCAL loBridge
loBridge = CREATEOBJECT("LegacyBridge")

IF TYPE("loBridge") != "O"
    MESSAGEBOX("Failed to create LegacyBridge object")
    RETURN
ENDIF

* Always check before use
IF !ISNULL(loBridge)
    loBridge.ConvertMarkdownToRTF(lcContent)
ENDIF
```

**Problem: Path Issues**
```foxpro
* Use full paths for file operations
lcFullPath = FULLPATH("legacybridge.dll")
IF !FILE(lcFullPath)
    MESSAGEBOX("DLL not found: " + lcFullPath)
ENDIF
```

## Diagnostic Tools

### Built-in Diagnostics

1. **Connection Test**
   ```vb
   Private Sub DiagnoseConnection()
       Dim connected As Boolean
       Dim version As String
       Dim lastError As String
       
       connected = TestConnection()
       version = GetVersion()
       
       If Not connected Then
           lastError = GetLastError()
           MsgBox "Diagnostics:" & vbCrLf & _
                  "Connected: No" & vbCrLf & _
                  "Error: " & lastError
       Else
           MsgBox "Diagnostics:" & vbCrLf & _
                  "Connected: Yes" & vbCrLf & _
                  "Version: " & version
       End If
   End Sub
   ```

2. **Performance Test**
   ```vb
   Private Sub TestPerformance()
       Dim i As Integer
       Dim startTime As Single
       Dim endTime As Single
       Dim testDoc As String
       
       testDoc = "# Test Document" & vbCrLf & "Test content"
       
       startTime = Timer
       For i = 1 To 1000
           ConvertMarkdownToRTF testDoc
       Next i
       endTime = Timer
       
       MsgBox "1000 conversions in " & _
              Format(endTime - startTime, "0.00") & " seconds"
   End Sub
   ```

### Debug Logging

Enable debug logging for detailed diagnostics:

1. **Set Environment Variable**
   ```cmd
   set LEGACYBRIDGE_DEBUG=1
   set LEGACYBRIDGE_LOG_PATH=C:\Temp\legacybridge.log
   ```

2. **Analyze Log File**
   ```
   [2025-07-24 10:15:32] INFO: LegacyBridge initialized v1.0.0
   [2025-07-24 10:15:33] DEBUG: Converting markdown (2048 bytes)
   [2025-07-24 10:15:33] DEBUG: Parse time: 0.5ms
   [2025-07-24 10:15:33] DEBUG: Generate time: 1.2ms
   [2025-07-24 10:15:33] INFO: Conversion complete
   ```

### Memory Leak Detection

```vb
' Simple memory leak detector
Private Type PROCESS_MEMORY_COUNTERS
    cb As Long
    PageFaultCount As Long
    PeakWorkingSetSize As Long
    WorkingSetSize As Long
    QuotaPeakPagedPoolUsage As Long
    QuotaPagedPoolUsage As Long
    QuotaPeakNonPagedPoolUsage As Long
    QuotaNonPagedPoolUsage As Long
    PagefileUsage As Long
    PeakPagefileUsage As Long
End Type

Private Declare Function GetProcessMemoryInfo Lib "psapi.dll" _
    (ByVal hProcess As Long, _
     ByRef ppsmemCounters As PROCESS_MEMORY_COUNTERS, _
     ByVal cb As Long) As Long

Function CheckMemoryLeak() As Boolean
    Dim pmc As PROCESS_MEMORY_COUNTERS
    Dim initialMemory As Long
    Dim finalMemory As Long
    Dim i As Integer
    
    pmc.cb = Len(pmc)
    GetProcessMemoryInfo -1, pmc, Len(pmc)
    initialMemory = pmc.WorkingSetSize
    
    ' Perform many conversions
    For i = 1 To 10000
        ConvertMarkdownToRTF "# Test"
    Next i
    
    GetProcessMemoryInfo -1, pmc, Len(pmc)
    finalMemory = pmc.WorkingSetSize
    
    ' Check for significant increase (>50MB)
    CheckMemoryLeak = (finalMemory - initialMemory) > 52428800
End Function
```

## Error Code Reference

| Code | Constant | Description | Common Causes |
|------|----------|-------------|---------------|
| 0 | LB_SUCCESS | Operation successful | - |
| -1 | LB_ERROR_NULL_POINTER | Null pointer passed | Empty string, uninitialized variable |
| -2 | LB_ERROR_INVALID_UTF8 | Invalid UTF-8 encoding | Wrong code page, corrupted text |
| -3 | LB_ERROR_CONVERSION | Conversion failed | Malformed input, unsupported features |
| -4 | LB_ERROR_ALLOCATION | Memory allocation failed | Out of memory, very large document |

### Extended Error Information

Get detailed error messages:

```vb
Function GetDetailedError() As String
    Dim buffer As String * 1024
    Dim length As Long
    
    length = legacybridge_get_last_error(buffer, 1024)
    
    If length > 0 Then
        GetDetailedError = Left$(buffer, length)
    Else
        GetDetailedError = "Unknown error"
    End If
End Function
```

## Getting Help

### Self-Help Resources

1. **Check Documentation**
   - API_REFERENCE.md for function details
   - USER_GUIDE.md for usage examples
   - RELEASE_NOTES.md for known issues

2. **Search Knowledge Base**
   - https://support.legacybridge.com/kb
   - Common issues and solutions
   - Video tutorials

3. **Community Forum**
   - https://forum.legacybridge.com
   - User discussions
   - Tips and tricks

### Contacting Support

**Before Contacting Support, Gather:**

1. **System Information**
   ```vb
   Private Sub GatherSystemInfo()
       Dim info As String
       
       info = "LegacyBridge Diagnostic Report" & vbCrLf & _
              "================================" & vbCrLf & _
              "Version: " & GetVersion() & vbCrLf & _
              "OS: " & GetWindowsVersion() & vbCrLf & _
              "Memory: " & GetAvailableMemory() & " MB" & vbCrLf & _
              "DLL Path: " & GetDLLPath() & vbCrLf & _
              "Last Error: " & GetLastError()
       
       SaveToFile info, "diagnostic_report.txt"
   End Sub
   ```

2. **Error Logs**
   - Application event logs
   - LegacyBridge debug logs
   - Screenshot of error messages

3. **Minimal Reproduction Case**
   ```vb
   ' Provide simplest code that reproduces issue
   Sub ReproduceIssue()
       Dim result As String
       result = ConvertMarkdownToRTF("Problem input here")
       ' Error occurs here
   End Sub
   ```

### Support Channels

- **Email**: support@legacybridge.com
- **Phone**: 1-800-LEGACY-1 (Business hours)
- **Priority Support**: For enterprise customers
- **Bug Reports**: https://github.com/legacybridge/issues

### Response Times

- **Critical Issues**: 4 hours
- **High Priority**: 1 business day
- **Normal Priority**: 3 business days
- **Feature Requests**: Evaluated quarterly

---

**LegacyBridge Troubleshooting Guide v1.0.0**  
Last Updated: July 24, 2025  
© 2025 LegacyBridge. All rights reserved.