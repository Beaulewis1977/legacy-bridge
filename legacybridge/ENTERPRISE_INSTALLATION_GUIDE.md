# LegacyBridge Enterprise Installation Guide

## Table of Contents

1. [System Requirements](#system-requirements)
2. [Pre-Installation Checklist](#pre-installation-checklist)
3. [Installation Methods](#installation-methods)
4. [Step-by-Step Installation](#step-by-step-installation)
5. [Configuration](#configuration)
6. [Integration with Legacy Systems](#integration-with-legacy-systems)
7. [Verification and Testing](#verification-and-testing)
8. [Troubleshooting](#troubleshooting)
9. [Uninstallation](#uninstallation)
10. [Support and Resources](#support-and-resources)

## System Requirements

### Minimum Requirements
- **Operating System**: Windows XP SP3, Windows Server 2003 or later
- **Architecture**: x86 (32-bit) or x64 (64-bit)
- **Memory**: 512 MB RAM
- **Disk Space**: 10 MB available
- **Dependencies**: Visual C++ 2015-2022 Redistributable (x86)

### Recommended Requirements
- **Operating System**: Windows 10/11, Windows Server 2016 or later
- **Architecture**: x64 (64-bit)
- **Memory**: 2 GB RAM or more
- **Disk Space**: 50 MB available
- **Network**: Internet connection for updates (optional)

### Supported Development Environments
- Visual Basic 6.0 SP6
- Visual FoxPro 9.0
- .NET Framework 2.0 or later
- C/C++ with Windows SDK
- Any environment supporting standard DLL calls

## Pre-Installation Checklist

Before installing LegacyBridge, ensure:

- [ ] Administrator privileges on the target system
- [ ] Visual C++ 2015-2022 Redistributable (x86) installed
- [ ] Target application directory identified
- [ ] Backup of existing conversion solutions (if any)
- [ ] Anti-virus software configured to allow DLL installation

## Installation Methods

### Method 1: Manual Installation (Recommended)

Best for single applications or testing environments.

### Method 2: Silent Installation

For enterprise deployment across multiple systems.

### Method 3: Group Policy Deployment

For domain-wide installation in corporate environments.

## Step-by-Step Installation

### Method 1: Manual Installation

1. **Download the Installation Package**
   ```
   legacybridge-v1.0.0-win32.zip
   ```

2. **Extract the Package**
   - Right-click the ZIP file
   - Select "Extract All..."
   - Choose your application directory

3. **Verify Package Contents**
   ```
   legacybridge/
   ├── bin/
   │   └── legacybridge.dll (720 KB)
   ├── include/
   │   └── legacybridge.h
   ├── examples/
   │   ├── vb6/
   │   │   ├── LegacyBridge.bas
   │   │   └── TestLegacyBridge.frm
   │   └── vfp9/
   │       ├── legacybridge.prg
   │       └── test_legacybridge.prg
   └── docs/
       └── INTEGRATION_GUIDE.md
   ```

4. **Copy DLL to Application Directory**
   ```cmd
   copy bin\legacybridge.dll C:\YourApp\
   ```

5. **Register DLL (Optional)**
   
   For system-wide availability:
   ```cmd
   copy bin\legacybridge.dll C:\Windows\System32\
   ```
   
   For 64-bit systems, also copy to:
   ```cmd
   copy bin\legacybridge.dll C:\Windows\SysWOW64\
   ```

### Method 2: Silent Installation

1. **Create Installation Script** (`install_legacybridge.bat`):
   ```batch
   @echo off
   echo Installing LegacyBridge v1.0.0...
   
   :: Check for admin rights
   net session >nul 2>&1
   if %errorLevel% neq 0 (
       echo ERROR: Administrator privileges required
       exit /b 1
   )
   
   :: Set installation directory
   set INSTALL_DIR=%~dp0
   set TARGET_DIR=%1
   if "%TARGET_DIR%"=="" set TARGET_DIR=%ProgramFiles%\LegacyBridge
   
   :: Create directory
   if not exist "%TARGET_DIR%" mkdir "%TARGET_DIR%"
   
   :: Copy files
   xcopy /Y "%INSTALL_DIR%bin\*.*" "%TARGET_DIR%\"
   xcopy /Y /S "%INSTALL_DIR%include\*.*" "%TARGET_DIR%\include\"
   xcopy /Y /S "%INSTALL_DIR%examples\*.*" "%TARGET_DIR%\examples\"
   
   :: Register DLL
   copy /Y "%TARGET_DIR%\legacybridge.dll" "%WINDIR%\System32\"
   if exist "%WINDIR%\SysWOW64" (
       copy /Y "%TARGET_DIR%\legacybridge.dll" "%WINDIR%\SysWOW64\"
   )
   
   :: Set environment variable
   setx LEGACYBRIDGE_PATH "%TARGET_DIR%" /M
   
   echo Installation complete!
   echo DLL Location: %TARGET_DIR%\legacybridge.dll
   ```

2. **Run Silent Installation**
   ```cmd
   install_legacybridge.bat "C:\Program Files\YourApp"
   ```

### Method 3: Group Policy Deployment

1. **Create MSI Package** (using WiX Toolset or similar)

2. **Configure Group Policy**
   - Open Group Policy Management Console
   - Create new GPO or edit existing
   - Navigate to: Computer Configuration → Software Settings → Software Installation
   - Right-click → New → Package
   - Select your MSI file
   - Choose "Assigned" deployment method

## Configuration

### Environment Variables

Set these optional environment variables for advanced configuration:

```cmd
:: Set custom template directory
setx LEGACYBRIDGE_TEMPLATES "C:\LegacyBridge\Templates"

:: Enable debug logging
setx LEGACYBRIDGE_DEBUG "1"

:: Set conversion timeout (milliseconds)
setx LEGACYBRIDGE_TIMEOUT "30000"
```

### Configuration File (Optional)

Create `legacybridge.ini` in the DLL directory:

```ini
[General]
EnableLogging=false
LogPath=C:\Logs\LegacyBridge
MaxFileSize=10485760
DefaultTemplate=professional

[Performance]
EnableParallelProcessing=true
ChunkSize=524288
CacheEnabled=true
MaxThreads=4

[Security]
ValidateInput=true
SanitizeOutput=true
MaxNestingLevel=10
```

## Integration with Legacy Systems

### Visual Basic 6.0 Integration

1. **Add Module to Project**
   - Project → Add Module → Existing
   - Select `LegacyBridge.bas`

2. **Configure Project**
   - Project → References
   - Ensure "Microsoft Scripting Runtime" is checked

3. **Test Integration**
   ```vb
   Private Sub TestConnection()
       If LegacyBridge_TestConnection() Then
           MsgBox "LegacyBridge connected successfully!"
       Else
           MsgBox "Failed to connect to LegacyBridge"
       End If
   End Sub
   ```

### Visual FoxPro 9.0 Integration

1. **Add Class to Project**
   ```foxpro
   SET PROCEDURE TO legacybridge.prg ADDITIVE
   ```

2. **Instantiate Bridge**
   ```foxpro
   LOCAL loBridge
   loBridge = CREATEOBJECT("LegacyBridge")
   
   IF loBridge.TestConnection()
       ? "LegacyBridge connected successfully!"
   ELSE
       ? "Failed to connect: " + loBridge.GetLastError()
   ENDIF
   ```

### .NET Integration

1. **Add DLL Reference**
   ```csharp
   [DllImport("legacybridge.dll", CallingConvention = CallingConvention.Cdecl)]
   private static extern int legacybridge_test_connection();
   ```

2. **Create Wrapper Class**
   ```csharp
   public class LegacyBridge
   {
       public bool TestConnection()
       {
           return legacybridge_test_connection() == 1;
       }
   }
   ```

## Verification and Testing

### 1. Connection Test

Run this test immediately after installation:

```cmd
:: Using provided test utility
legacybridge_test.exe

:: Expected output:
LegacyBridge v1.0.0
Connection: OK
Functions: 29 exported
Memory: OK
Performance: 41,131 ops/sec
```

### 2. Conversion Test

Test basic conversion functionality:

```vb
' VB6 Test
Dim markdown As String
markdown = "# Hello World" & vbCrLf & "This is a **test**."

Dim rtf As String
rtf = ConvertMarkdownToRTF(markdown)

If Len(rtf) > 0 Then
    MsgBox "Conversion successful!"
End If
```

### 3. Performance Test

Verify performance meets requirements:

```vb
' Batch conversion test
Dim startTime As Single
Dim endTime As Single
Dim i As Integer

startTime = Timer
For i = 1 To 1000
    Call ConvertMarkdownToRTF("# Test " & i)
Next i
endTime = Timer

MsgBox "1000 conversions in " & (endTime - startTime) & " seconds"
```

### 4. Memory Test

Ensure no memory leaks:

```vb
' Memory leak test
Dim i As Long
For i = 1 To 10000
    Dim result As String
    result = ConvertMarkdownToRTF("# Memory Test")
    ' Memory is automatically freed
Next i
```

## Troubleshooting

### Common Issues and Solutions

#### DLL Not Found Error

**Error**: "legacybridge.dll not found" or "Error 53"

**Solutions**:
1. Verify DLL is in application directory or system path
2. Check file permissions
3. Ensure Visual C++ Redistributable is installed
4. For 64-bit systems, ensure DLL is in SysWOW64

#### Access Violation or Crash

**Error**: Application crashes when calling DLL functions

**Solutions**:
1. Verify correct calling convention (CDECL)
2. Check parameter types match exactly
3. Ensure strings are properly null-terminated
4. Update to latest DLL version

#### Conversion Failures

**Error**: Conversions return empty or error results

**Solutions**:
1. Check input encoding (must be valid UTF-8)
2. Verify input size is within limits
3. Check for special characters that need escaping
4. Enable debug logging for detailed errors

#### Performance Issues

**Symptoms**: Slow conversions or high memory usage

**Solutions**:
1. Enable batch processing for multiple files
2. Adjust chunk size in configuration
3. Ensure adequate system resources
4. Disable unnecessary validation for trusted input

### Debug Mode

Enable detailed logging for troubleshooting:

```cmd
set LEGACYBRIDGE_DEBUG=1
set LEGACYBRIDGE_LOG_PATH=C:\Temp\LegacyBridge.log
```

## Uninstallation

### Manual Uninstallation

1. **Stop All Applications** using LegacyBridge

2. **Remove DLL Files**
   ```cmd
   del C:\YourApp\legacybridge.dll
   del %WINDIR%\System32\legacybridge.dll
   del %WINDIR%\SysWOW64\legacybridge.dll
   ```

3. **Remove Configuration**
   ```cmd
   del C:\YourApp\legacybridge.ini
   reg delete "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment" /v LEGACYBRIDGE_PATH /f
   ```

4. **Clean Registry** (if applicable)
   ```cmd
   reg delete "HKLM\SOFTWARE\LegacyBridge" /f
   ```

### Automated Uninstallation

Create `uninstall_legacybridge.bat`:

```batch
@echo off
echo Uninstalling LegacyBridge...

:: Remove DLL files
del /Q "%ProgramFiles%\LegacyBridge\*.dll" 2>nul
del /Q "%WINDIR%\System32\legacybridge.dll" 2>nul
del /Q "%WINDIR%\SysWOW64\legacybridge.dll" 2>nul

:: Remove directories
rmdir /S /Q "%ProgramFiles%\LegacyBridge" 2>nul

:: Remove environment variables
reg delete "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment" /v LEGACYBRIDGE_PATH /f 2>nul

echo Uninstallation complete!
```

## Support and Resources

### Documentation
- API Reference: See `API_REFERENCE.md`
- User Guide: See `USER_GUIDE.md`
- Troubleshooting Guide: See `TROUBLESHOOTING_GUIDE.md`

### Technical Support
- Email: support@legacybridge.com
- Knowledge Base: https://docs.legacybridge.com
- Community Forum: https://forum.legacybridge.com

### Training Resources
- Video Tutorials: Available on support portal
- Sample Projects: Included in examples directory
- Integration Workshops: Contact sales for scheduling

### Updates and Patches
- Check for updates: `legacybridge_check_update.exe`
- Download patches: https://downloads.legacybridge.com
- Release notes: See `RELEASE_NOTES.md`

---

**LegacyBridge Enterprise Installation Guide v1.0.0**  
Last Updated: July 24, 2025  
© 2025 LegacyBridge. All rights reserved.