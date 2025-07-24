# .NET Integration Guide

## Overview

This guide provides comprehensive instructions for integrating LegacyBridge with .NET applications using P/Invoke and managed wrappers. Whether you're using .NET Framework 4.7.2+ or .NET Core 3.1+, this guide will help you seamlessly integrate RTF ↔ Markdown conversion capabilities into your application.

## Table of Contents

1. [Requirements](#requirements)
2. [Installation](#installation)
3. [P/Invoke Declarations](#pinvoke-declarations)
4. [Managed Wrapper Class](#managed-wrapper-class)
5. [Basic Usage Examples](#basic-usage-examples)
6. [Advanced Scenarios](#advanced-scenarios)
7. [Error Handling](#error-handling)
8. [Performance Optimization](#performance-optimization)
9. [Troubleshooting](#troubleshooting)
10. [Complete Sample Application](#complete-sample-application)

## Requirements

- **.NET Framework**: 4.7.2 or later
- **.NET Core/5+**: 3.1 or later
- **Visual Studio**: 2019 or later (recommended)
- **LegacyBridge DLL**: 32-bit or 64-bit matching your application architecture
- **Platform**: Windows, Linux, or macOS

## Installation

### Step 1: Download LegacyBridge

Download the appropriate LegacyBridge package for your platform:
- Windows: `legacybridge-win-x86.zip` or `legacybridge-win-x64.zip`
- Linux: `legacybridge-linux-x64.tar.gz`
- macOS: `legacybridge-macos.dmg`

### Step 2: Configure Your Project

#### For .NET Framework Projects

1. Copy `legacybridge.dll` to your project's output directory
2. Set "Copy to Output Directory" to "Copy always" in Visual Studio
3. Ensure the Visual C++ 2015-2022 Redistributable is installed

#### For .NET Core/.NET 5+ Projects

Add the following to your `.csproj` file:

```xml
<ItemGroup>
  <!-- Windows -->
  <ContentFiles Include="legacybridge.dll" Condition="'$(OS)' == 'Windows_NT'">
    <CopyToOutputDirectory>Always</CopyToOutputDirectory>
  </ContentFiles>
  
  <!-- Linux -->
  <ContentFiles Include="liblegacybridge.so" Condition="'$([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform($([System.Runtime.InteropServices.OSPlatform]::Linux)))' == 'true'">
    <CopyToOutputDirectory>Always</CopyToOutputDirectory>
  </ContentFiles>
  
  <!-- macOS -->
  <ContentFiles Include="liblegacybridge.dylib" Condition="'$([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform($([System.Runtime.InteropServices.OSPlatform]::OSX)))' == 'true'">
    <CopyToOutputDirectory>Always</CopyToOutputDirectory>
  </ContentFiles>
</ItemGroup>
```

## P/Invoke Declarations

Create a new class file `LegacyBridgeNative.cs`:

```csharp
using System;
using System.Runtime.InteropServices;
using System.Text;

namespace LegacyBridge.Interop
{
    internal static class LegacyBridgeNative
    {
        private const string DLL_NAME_WINDOWS = "legacybridge.dll";
        private const string DLL_NAME_LINUX = "liblegacybridge.so";
        private const string DLL_NAME_MACOS = "liblegacybridge.dylib";
        
        // Helper to get the correct library name
        private static string GetLibraryName()
        {
            if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
                return DLL_NAME_WINDOWS;
            if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
                return DLL_NAME_LINUX;
            if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
                return DLL_NAME_MACOS;
            throw new PlatformNotSupportedException("Unsupported platform");
        }
        
        // Core conversion functions
        [DllImport(DLL_NAME_WINDOWS, EntryPoint = "legacybridge_rtf_to_markdown", CallingConvention = CallingConvention.Cdecl)]
        [DllImport(DLL_NAME_LINUX, EntryPoint = "legacybridge_rtf_to_markdown", CallingConvention = CallingConvention.Cdecl)]
        [DllImport(DLL_NAME_MACOS, EntryPoint = "legacybridge_rtf_to_markdown", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr legacybridge_rtf_to_markdown(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string rtfContent);
        
        [DllImport(DLL_NAME_WINDOWS, EntryPoint = "legacybridge_markdown_to_rtf", CallingConvention = CallingConvention.Cdecl)]
        [DllImport(DLL_NAME_LINUX, EntryPoint = "legacybridge_markdown_to_rtf", CallingConvention = CallingConvention.Cdecl)]
        [DllImport(DLL_NAME_MACOS, EntryPoint = "legacybridge_markdown_to_rtf", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr legacybridge_markdown_to_rtf(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string markdownContent);
        
        // File operations
        [DllImport(DLL_NAME_WINDOWS, CallingConvention = CallingConvention.Cdecl)]
        public static extern int legacybridge_convert_rtf_file_to_md(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string inputPath,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string outputPath);
        
        [DllImport(DLL_NAME_WINDOWS, CallingConvention = CallingConvention.Cdecl)]
        public static extern int legacybridge_convert_md_file_to_rtf(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string inputPath,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string outputPath);
        
        // Validation functions
        [DllImport(DLL_NAME_WINDOWS, CallingConvention = CallingConvention.Cdecl)]
        public static extern int legacybridge_validate_rtf_document(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string rtfContent);
        
        [DllImport(DLL_NAME_WINDOWS, CallingConvention = CallingConvention.Cdecl)]
        public static extern int legacybridge_validate_markdown_document(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string markdownContent);
        
        // Batch processing
        [DllImport(DLL_NAME_WINDOWS, CallingConvention = CallingConvention.Cdecl)]
        public static extern int legacybridge_convert_folder_rtf_to_md(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string inputFolder,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string outputFolder);
        
        [DllImport(DLL_NAME_WINDOWS, CallingConvention = CallingConvention.Cdecl)]
        public static extern int legacybridge_convert_folder_md_to_rtf(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string inputFolder,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string outputFolder);
        
        // Template functions
        [DllImport(DLL_NAME_WINDOWS, CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr legacybridge_apply_rtf_template(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string rtfContent,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string templateName);
        
        // Utility functions
        [DllImport(DLL_NAME_WINDOWS, CallingConvention = CallingConvention.Cdecl)]
        public static extern void legacybridge_free_string(IntPtr ptr);
        
        [DllImport(DLL_NAME_WINDOWS, CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr legacybridge_get_last_error();
        
        [DllImport(DLL_NAME_WINDOWS, CallingConvention = CallingConvention.Cdecl)]
        public static extern int legacybridge_test_connection();
        
        [DllImport(DLL_NAME_WINDOWS, CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr legacybridge_get_version();
    }
}
```

## Managed Wrapper Class

Create `LegacyBridgeConverter.cs`:

```csharp
using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using LegacyBridge.Interop;

namespace LegacyBridge
{
    /// <summary>
    /// Provides managed access to LegacyBridge RTF/Markdown conversion functionality
    /// </summary>
    public class LegacyBridgeConverter : IDisposable
    {
        private bool _disposed;

        /// <summary>
        /// Converts RTF content to Markdown format
        /// </summary>
        /// <param name="rtfContent">RTF content to convert</param>
        /// <returns>Converted Markdown content</returns>
        /// <exception cref="ArgumentException">Thrown when input is null or empty</exception>
        /// <exception cref="ConversionException">Thrown when conversion fails</exception>
        public string ConvertRtfToMarkdown(string rtfContent)
        {
            if (string.IsNullOrEmpty(rtfContent))
                throw new ArgumentException("RTF content cannot be null or empty", nameof(rtfContent));

            IntPtr resultPtr = IntPtr.Zero;
            try
            {
                resultPtr = LegacyBridgeNative.legacybridge_rtf_to_markdown(rtfContent);
                
                if (resultPtr == IntPtr.Zero)
                {
                    string error = GetLastError();
                    throw new ConversionException($"RTF to Markdown conversion failed: {error}");
                }

                return Marshal.PtrToStringUTF8(resultPtr) ?? string.Empty;
            }
            finally
            {
                if (resultPtr != IntPtr.Zero)
                    LegacyBridgeNative.legacybridge_free_string(resultPtr);
            }
        }

        /// <summary>
        /// Converts Markdown content to RTF format
        /// </summary>
        /// <param name="markdownContent">Markdown content to convert</param>
        /// <returns>Converted RTF content</returns>
        /// <exception cref="ArgumentException">Thrown when input is null or empty</exception>
        /// <exception cref="ConversionException">Thrown when conversion fails</exception>
        public string ConvertMarkdownToRtf(string markdownContent)
        {
            if (string.IsNullOrEmpty(markdownContent))
                throw new ArgumentException("Markdown content cannot be null or empty", nameof(markdownContent));

            IntPtr resultPtr = IntPtr.Zero;
            try
            {
                resultPtr = LegacyBridgeNative.legacybridge_markdown_to_rtf(markdownContent);
                
                if (resultPtr == IntPtr.Zero)
                {
                    string error = GetLastError();
                    throw new ConversionException($"Markdown to RTF conversion failed: {error}");
                }

                return Marshal.PtrToStringUTF8(resultPtr) ?? string.Empty;
            }
            finally
            {
                if (resultPtr != IntPtr.Zero)
                    LegacyBridgeNative.legacybridge_free_string(resultPtr);
            }
        }

        /// <summary>
        /// Converts an RTF file to Markdown file
        /// </summary>
        /// <param name="inputPath">Path to input RTF file</param>
        /// <param name="outputPath">Path for output Markdown file</param>
        /// <exception cref="FileNotFoundException">Thrown when input file doesn't exist</exception>
        /// <exception cref="ConversionException">Thrown when conversion fails</exception>
        public void ConvertRtfFile(string inputPath, string outputPath)
        {
            if (!File.Exists(inputPath))
                throw new FileNotFoundException("Input file not found", inputPath);

            int result = LegacyBridgeNative.legacybridge_convert_rtf_file_to_md(inputPath, outputPath);
            
            if (result != 0)
            {
                string error = GetLastError();
                throw new ConversionException($"File conversion failed: {error}");
            }
        }

        /// <summary>
        /// Converts a Markdown file to RTF file
        /// </summary>
        /// <param name="inputPath">Path to input Markdown file</param>
        /// <param name="outputPath">Path for output RTF file</param>
        /// <exception cref="FileNotFoundException">Thrown when input file doesn't exist</exception>
        /// <exception cref="ConversionException">Thrown when conversion fails</exception>
        public void ConvertMarkdownFile(string inputPath, string outputPath)
        {
            if (!File.Exists(inputPath))
                throw new FileNotFoundException("Input file not found", inputPath);

            int result = LegacyBridgeNative.legacybridge_convert_md_file_to_rtf(inputPath, outputPath);
            
            if (result != 0)
            {
                string error = GetLastError();
                throw new ConversionException($"File conversion failed: {error}");
            }
        }

        /// <summary>
        /// Converts all RTF files in a folder to Markdown
        /// </summary>
        /// <param name="inputFolder">Source folder containing RTF files</param>
        /// <param name="outputFolder">Destination folder for Markdown files</param>
        /// <returns>Number of files successfully converted</returns>
        public int ConvertFolder(string inputFolder, string outputFolder, ConversionDirection direction)
        {
            if (!Directory.Exists(inputFolder))
                throw new DirectoryNotFoundException($"Input folder not found: {inputFolder}");

            if (!Directory.Exists(outputFolder))
                Directory.CreateDirectory(outputFolder);

            int result = direction == ConversionDirection.RtfToMarkdown
                ? LegacyBridgeNative.legacybridge_convert_folder_rtf_to_md(inputFolder, outputFolder)
                : LegacyBridgeNative.legacybridge_convert_folder_md_to_rtf(inputFolder, outputFolder);

            if (result < 0)
            {
                string error = GetLastError();
                throw new ConversionException($"Folder conversion failed: {error}");
            }

            return result;
        }

        /// <summary>
        /// Validates RTF document structure
        /// </summary>
        /// <param name="rtfContent">RTF content to validate</param>
        /// <returns>True if valid, false otherwise</returns>
        public bool ValidateRtf(string rtfContent)
        {
            if (string.IsNullOrEmpty(rtfContent))
                return false;

            return LegacyBridgeNative.legacybridge_validate_rtf_document(rtfContent) == 0;
        }

        /// <summary>
        /// Validates Markdown document structure
        /// </summary>
        /// <param name="markdownContent">Markdown content to validate</param>
        /// <returns>True if valid, false otherwise</returns>
        public bool ValidateMarkdown(string markdownContent)
        {
            if (string.IsNullOrEmpty(markdownContent))
                return false;

            return LegacyBridgeNative.legacybridge_validate_markdown_document(markdownContent) == 0;
        }

        /// <summary>
        /// Applies a template to RTF content
        /// </summary>
        /// <param name="rtfContent">RTF content</param>
        /// <param name="templateName">Template name (minimal, professional, academic)</param>
        /// <returns>RTF content with template applied</returns>
        public string ApplyTemplate(string rtfContent, string templateName)
        {
            if (string.IsNullOrEmpty(rtfContent))
                throw new ArgumentException("RTF content cannot be null or empty", nameof(rtfContent));

            IntPtr resultPtr = IntPtr.Zero;
            try
            {
                resultPtr = LegacyBridgeNative.legacybridge_apply_rtf_template(rtfContent, templateName);
                
                if (resultPtr == IntPtr.Zero)
                {
                    string error = GetLastError();
                    throw new ConversionException($"Template application failed: {error}");
                }

                return Marshal.PtrToStringUTF8(resultPtr) ?? string.Empty;
            }
            finally
            {
                if (resultPtr != IntPtr.Zero)
                    LegacyBridgeNative.legacybridge_free_string(resultPtr);
            }
        }

        /// <summary>
        /// Tests the connection to LegacyBridge DLL
        /// </summary>
        /// <returns>True if connection successful</returns>
        public bool TestConnection()
        {
            return LegacyBridgeNative.legacybridge_test_connection() == 1;
        }

        /// <summary>
        /// Gets the LegacyBridge version
        /// </summary>
        /// <returns>Version string</returns>
        public string GetVersion()
        {
            IntPtr versionPtr = LegacyBridgeNative.legacybridge_get_version();
            if (versionPtr == IntPtr.Zero)
                return "Unknown";

            return Marshal.PtrToStringAnsi(versionPtr) ?? "Unknown";
        }

        /// <summary>
        /// Gets the last error message
        /// </summary>
        /// <returns>Error message or "Unknown error"</returns>
        private string GetLastError()
        {
            IntPtr errorPtr = LegacyBridgeNative.legacybridge_get_last_error();
            if (errorPtr == IntPtr.Zero)
                return "Unknown error";

            try
            {
                return Marshal.PtrToStringUTF8(errorPtr) ?? "Unknown error";
            }
            finally
            {
                LegacyBridgeNative.legacybridge_free_string(errorPtr);
            }
        }

        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        protected virtual void Dispose(bool disposing)
        {
            if (!_disposed)
            {
                _disposed = true;
            }
        }
    }

    /// <summary>
    /// Specifies the direction of conversion
    /// </summary>
    public enum ConversionDirection
    {
        RtfToMarkdown,
        MarkdownToRtf
    }

    /// <summary>
    /// Exception thrown when document conversion fails
    /// </summary>
    public class ConversionException : Exception
    {
        public ConversionException(string message) : base(message) { }
        public ConversionException(string message, Exception innerException) : base(message, innerException) { }
    }
}
```

## Basic Usage Examples

### Simple String Conversion

```csharp
using LegacyBridge;

class Program
{
    static void Main()
    {
        using var converter = new LegacyBridgeConverter();
        
        // Test connection
        if (!converter.TestConnection())
        {
            Console.WriteLine("Failed to connect to LegacyBridge DLL");
            return;
        }
        
        Console.WriteLine($"LegacyBridge version: {converter.GetVersion()}");
        
        // Convert Markdown to RTF
        string markdown = "# Hello World\n\nThis is **bold** and *italic* text.";
        string rtf = converter.ConvertMarkdownToRtf(markdown);
        Console.WriteLine("Converted RTF:");
        Console.WriteLine(rtf);
        
        // Convert RTF back to Markdown
        string convertedMarkdown = converter.ConvertRtfToMarkdown(rtf);
        Console.WriteLine("\nConverted back to Markdown:");
        Console.WriteLine(convertedMarkdown);
    }
}
```

### File Conversion

```csharp
using LegacyBridge;
using System;
using System.IO;

class FileConversionExample
{
    static void Main()
    {
        using var converter = new LegacyBridgeConverter();
        
        try
        {
            // Convert single file
            converter.ConvertMarkdownFile("README.md", "README.rtf");
            Console.WriteLine("File converted successfully!");
            
            // Convert RTF file to Markdown
            converter.ConvertRtfFile("document.rtf", "document.md");
            Console.WriteLine("RTF file converted to Markdown!");
        }
        catch (ConversionException ex)
        {
            Console.WriteLine($"Conversion failed: {ex.Message}");
        }
        catch (FileNotFoundException ex)
        {
            Console.WriteLine($"File not found: {ex.FileName}");
        }
    }
}
```

### Batch Processing

```csharp
using LegacyBridge;
using System;

class BatchProcessingExample
{
    static void Main()
    {
        using var converter = new LegacyBridgeConverter();
        
        try
        {
            // Convert all RTF files in a folder to Markdown
            int filesConverted = converter.ConvertFolder(
                @"C:\Documents\RTF", 
                @"C:\Documents\Markdown",
                ConversionDirection.RtfToMarkdown
            );
            
            Console.WriteLine($"Successfully converted {filesConverted} files!");
        }
        catch (ConversionException ex)
        {
            Console.WriteLine($"Batch conversion failed: {ex.Message}");
        }
    }
}
```

## Advanced Scenarios

### Async Processing

```csharp
using LegacyBridge;
using System;
using System.Threading.Tasks;

public class AsyncConversionService
{
    private readonly LegacyBridgeConverter _converter;
    
    public AsyncConversionService()
    {
        _converter = new LegacyBridgeConverter();
    }
    
    public async Task<string> ConvertRtfToMarkdownAsync(string rtfContent)
    {
        return await Task.Run(() => _converter.ConvertRtfToMarkdown(rtfContent));
    }
    
    public async Task<string> ConvertMarkdownToRtfAsync(string markdownContent)
    {
        return await Task.Run(() => _converter.ConvertMarkdownToRtf(markdownContent));
    }
    
    public async Task ConvertFileAsync(string inputPath, string outputPath)
    {
        await Task.Run(() =>
        {
            if (inputPath.EndsWith(".rtf", StringComparison.OrdinalIgnoreCase))
                _converter.ConvertRtfFile(inputPath, outputPath);
            else if (inputPath.EndsWith(".md", StringComparison.OrdinalIgnoreCase))
                _converter.ConvertMarkdownFile(inputPath, outputPath);
            else
                throw new ArgumentException("Unsupported file type");
        });
    }
}

// Usage
class Program
{
    static async Task Main()
    {
        var service = new AsyncConversionService();
        
        string markdown = "# Async Example\n\nConverting documents asynchronously.";
        string rtf = await service.ConvertMarkdownToRtfAsync(markdown);
        
        Console.WriteLine("Conversion completed!");
        Console.WriteLine(rtf);
    }
}
```

### Dependency Injection (ASP.NET Core)

```csharp
// Startup.cs or Program.cs
public class Startup
{
    public void ConfigureServices(IServiceCollection services)
    {
        // Register as singleton (thread-safe)
        services.AddSingleton<IDocumentConverter, LegacyBridgeService>();
        
        services.AddControllers();
    }
}

// Service interface
public interface IDocumentConverter
{
    string ConvertRtfToMarkdown(string rtfContent);
    string ConvertMarkdownToRtf(string markdownContent);
    Task<ConversionResult> ConvertFileAsync(string inputPath, string outputPath);
}

// Service implementation
public class LegacyBridgeService : IDocumentConverter, IDisposable
{
    private readonly LegacyBridgeConverter _converter;
    private readonly ILogger<LegacyBridgeService> _logger;
    
    public LegacyBridgeService(ILogger<LegacyBridgeService> logger)
    {
        _logger = logger;
        _converter = new LegacyBridgeConverter();
        
        if (!_converter.TestConnection())
        {
            _logger.LogError("Failed to initialize LegacyBridge connection");
            throw new InvalidOperationException("LegacyBridge initialization failed");
        }
        
        _logger.LogInformation($"LegacyBridge initialized. Version: {_converter.GetVersion()}");
    }
    
    public string ConvertRtfToMarkdown(string rtfContent)
    {
        try
        {
            return _converter.ConvertRtfToMarkdown(rtfContent);
        }
        catch (ConversionException ex)
        {
            _logger.LogError(ex, "RTF to Markdown conversion failed");
            throw;
        }
    }
    
    public string ConvertMarkdownToRtf(string markdownContent)
    {
        try
        {
            return _converter.ConvertMarkdownToRtf(markdownContent);
        }
        catch (ConversionException ex)
        {
            _logger.LogError(ex, "Markdown to RTF conversion failed");
            throw;
        }
    }
    
    public async Task<ConversionResult> ConvertFileAsync(string inputPath, string outputPath)
    {
        var result = new ConversionResult { Success = false };
        
        try
        {
            await Task.Run(() =>
            {
                if (Path.GetExtension(inputPath).ToLower() == ".rtf")
                    _converter.ConvertRtfFile(inputPath, outputPath);
                else
                    _converter.ConvertMarkdownFile(inputPath, outputPath);
            });
            
            result.Success = true;
            result.OutputPath = outputPath;
            _logger.LogInformation($"File converted successfully: {inputPath} -> {outputPath}");
        }
        catch (Exception ex)
        {
            result.ErrorMessage = ex.Message;
            _logger.LogError(ex, $"File conversion failed: {inputPath}");
        }
        
        return result;
    }
    
    public void Dispose()
    {
        _converter?.Dispose();
    }
}

// Controller
[ApiController]
[Route("api/[controller]")]
public class ConversionController : ControllerBase
{
    private readonly IDocumentConverter _converter;
    
    public ConversionController(IDocumentConverter converter)
    {
        _converter = converter;
    }
    
    [HttpPost("rtf-to-markdown")]
    public IActionResult ConvertRtfToMarkdown([FromBody] ConversionRequest request)
    {
        try
        {
            var markdown = _converter.ConvertRtfToMarkdown(request.Content);
            return Ok(new { output = markdown });
        }
        catch (ConversionException ex)
        {
            return BadRequest(new { error = ex.Message });
        }
    }
    
    [HttpPost("markdown-to-rtf")]
    public IActionResult ConvertMarkdownToRtf([FromBody] ConversionRequest request)
    {
        try
        {
            var rtf = _converter.ConvertMarkdownToRtf(request.Content);
            return Ok(new { output = rtf });
        }
        catch (ConversionException ex)
        {
            return BadRequest(new { error = ex.Message });
        }
    }
}

public class ConversionRequest
{
    public string Content { get; set; }
}

public class ConversionResult
{
    public bool Success { get; set; }
    public string OutputPath { get; set; }
    public string ErrorMessage { get; set; }
}
```

### WPF Application Integration

```csharp
using System;
using System.Windows;
using System.Windows.Documents;
using System.Windows.Controls;
using LegacyBridge;

namespace WpfDocumentConverter
{
    public partial class MainWindow : Window
    {
        private readonly LegacyBridgeConverter _converter;
        
        public MainWindow()
        {
            InitializeComponent();
            _converter = new LegacyBridgeConverter();
            
            if (!_converter.TestConnection())
            {
                MessageBox.Show("Failed to initialize LegacyBridge", "Error", 
                    MessageBoxButton.OK, MessageBoxImage.Error);
                Close();
            }
        }
        
        private void ConvertToMarkdown_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                // Get RTF from RichTextBox
                var textRange = new TextRange(
                    RichTextEditor.Document.ContentStart,
                    RichTextEditor.Document.ContentEnd
                );
                
                string rtf;
                using (var stream = new System.IO.MemoryStream())
                {
                    textRange.Save(stream, DataFormats.Rtf);
                    rtf = System.Text.Encoding.UTF8.GetString(stream.ToArray());
                }
                
                // Convert to Markdown
                string markdown = _converter.ConvertRtfToMarkdown(rtf);
                
                // Display result
                MarkdownOutput.Text = markdown;
                StatusLabel.Content = "Conversion successful!";
            }
            catch (ConversionException ex)
            {
                MessageBox.Show($"Conversion failed: {ex.Message}", "Error",
                    MessageBoxButton.OK, MessageBoxImage.Error);
                StatusLabel.Content = "Conversion failed";
            }
        }
        
        private void ConvertToRtf_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                string markdown = MarkdownInput.Text;
                
                // Validate markdown
                if (!_converter.ValidateMarkdown(markdown))
                {
                    MessageBox.Show("Invalid Markdown syntax", "Validation Error",
                        MessageBoxButton.OK, MessageBoxImage.Warning);
                    return;
                }
                
                // Convert to RTF
                string rtf = _converter.ConvertMarkdownToRtf(markdown);
                
                // Load into RichTextBox
                var textRange = new TextRange(
                    RichTextEditor.Document.ContentStart,
                    RichTextEditor.Document.ContentEnd
                );
                
                using (var stream = new System.IO.MemoryStream(
                    System.Text.Encoding.UTF8.GetBytes(rtf)))
                {
                    textRange.Load(stream, DataFormats.Rtf);
                }
                
                StatusLabel.Content = "Conversion successful!";
            }
            catch (ConversionException ex)
            {
                MessageBox.Show($"Conversion failed: {ex.Message}", "Error",
                    MessageBoxButton.OK, MessageBoxImage.Error);
                StatusLabel.Content = "Conversion failed";
            }
        }
    }
}
```

## Error Handling

### Comprehensive Error Handling Pattern

```csharp
public class DocumentConversionService
{
    private readonly LegacyBridgeConverter _converter;
    private readonly ILogger<DocumentConversionService> _logger;
    
    public DocumentConversionService(ILogger<DocumentConversionService> logger)
    {
        _logger = logger;
        _converter = new LegacyBridgeConverter();
    }
    
    public ConversionResult SafeConvert(string content, ConversionDirection direction)
    {
        var result = new ConversionResult();
        
        try
        {
            // Validate input
            if (string.IsNullOrWhiteSpace(content))
            {
                result.Error = "Input content cannot be empty";
                return result;
            }
            
            // Check content size (10MB limit)
            if (System.Text.Encoding.UTF8.GetByteCount(content) > 10 * 1024 * 1024)
            {
                result.Error = "Content exceeds 10MB limit";
                return result;
            }
            
            // Validate format
            bool isValid = direction == ConversionDirection.RtfToMarkdown
                ? _converter.ValidateRtf(content)
                : _converter.ValidateMarkdown(content);
                
            if (!isValid)
            {
                result.Error = $"Invalid {(direction == ConversionDirection.RtfToMarkdown ? "RTF" : "Markdown")} format";
                return result;
            }
            
            // Perform conversion
            result.Output = direction == ConversionDirection.RtfToMarkdown
                ? _converter.ConvertRtfToMarkdown(content)
                : _converter.ConvertMarkdownToRtf(content);
                
            result.Success = true;
            
            _logger.LogInformation($"Successfully converted {content.Length} characters");
        }
        catch (ConversionException ex)
        {
            result.Error = ex.Message;
            _logger.LogError(ex, "Conversion failed");
        }
        catch (OutOfMemoryException ex)
        {
            result.Error = "Out of memory - document too large";
            _logger.LogError(ex, "Memory allocation failed during conversion");
        }
        catch (AccessViolationException ex)
        {
            result.Error = "Critical error - please restart the application";
            _logger.LogCritical(ex, "Access violation in native code");
        }
        catch (Exception ex)
        {
            result.Error = "Unexpected error occurred";
            _logger.LogError(ex, "Unexpected error during conversion");
        }
        
        return result;
    }
    
    public class ConversionResult
    {
        public bool Success { get; set; }
        public string Output { get; set; }
        public string Error { get; set; }
    }
}
```

### Retry Logic for Transient Failures

```csharp
using Polly;
using Polly.Retry;

public class ResilientConversionService
{
    private readonly LegacyBridgeConverter _converter;
    private readonly AsyncRetryPolicy _retryPolicy;
    
    public ResilientConversionService()
    {
        _converter = new LegacyBridgeConverter();
        
        // Configure retry policy
        _retryPolicy = Policy
            .Handle<ConversionException>()
            .WaitAndRetryAsync(
                retryCount: 3,
                sleepDurationProvider: retryAttempt => TimeSpan.FromSeconds(Math.Pow(2, retryAttempt)),
                onRetry: (exception, timeSpan, retryCount, context) =>
                {
                    Console.WriteLine($"Retry {retryCount} after {timeSpan} seconds");
                });
    }
    
    public async Task<string> ConvertWithRetryAsync(string content, ConversionDirection direction)
    {
        return await _retryPolicy.ExecuteAsync(async () =>
        {
            return await Task.Run(() =>
            {
                return direction == ConversionDirection.RtfToMarkdown
                    ? _converter.ConvertRtfToMarkdown(content)
                    : _converter.ConvertMarkdownToRtf(content);
            });
        });
    }
}
```

## Performance Optimization

### Memory-Efficient Batch Processing

```csharp
public class OptimizedBatchProcessor
{
    private readonly LegacyBridgeConverter _converter;
    private readonly int _batchSize;
    
    public OptimizedBatchProcessor(int batchSize = 100)
    {
        _converter = new LegacyBridgeConverter();
        _batchSize = batchSize;
    }
    
    public async Task ProcessLargeDatasetAsync(
        IEnumerable<string> documents, 
        ConversionDirection direction,
        IProgress<BatchProgress> progress = null)
    {
        var batches = documents.Batch(_batchSize);
        int totalProcessed = 0;
        int totalDocuments = documents.Count();
        
        await Parallel.ForEachAsync(batches, async (batch, ct) =>
        {
            var results = new List<string>();
            
            foreach (var document in batch)
            {
                try
                {
                    var result = direction == ConversionDirection.RtfToMarkdown
                        ? _converter.ConvertRtfToMarkdown(document)
                        : _converter.ConvertMarkdownToRtf(document);
                    
                    results.Add(result);
                    
                    Interlocked.Increment(ref totalProcessed);
                    
                    progress?.Report(new BatchProgress
                    {
                        ProcessedCount = totalProcessed,
                        TotalCount = totalDocuments,
                        PercentComplete = (totalProcessed * 100.0) / totalDocuments
                    });
                }
                catch (ConversionException ex)
                {
                    // Log error and continue
                    Console.WriteLine($"Failed to convert document: {ex.Message}");
                }
            }
            
            // Process results (save to files, database, etc.)
            await ProcessResultsAsync(results);
        });
    }
    
    private async Task ProcessResultsAsync(List<string> results)
    {
        // Implementation for saving results
        await Task.CompletedTask;
    }
}

public class BatchProgress
{
    public int ProcessedCount { get; set; }
    public int TotalCount { get; set; }
    public double PercentComplete { get; set; }
}

// Extension method for batching
public static class EnumerableExtensions
{
    public static IEnumerable<IEnumerable<T>> Batch<T>(this IEnumerable<T> source, int batchSize)
    {
        using var enumerator = source.GetEnumerator();
        while (enumerator.MoveNext())
        {
            yield return YieldBatchElements(enumerator, batchSize - 1);
        }
    }
    
    private static IEnumerable<T> YieldBatchElements<T>(IEnumerator<T> enumerator, int batchSize)
    {
        yield return enumerator.Current;
        for (int i = 0; i < batchSize && enumerator.MoveNext(); i++)
        {
            yield return enumerator.Current;
        }
    }
}
```

### Caching for Repeated Conversions

```csharp
using Microsoft.Extensions.Caching.Memory;
using System.Security.Cryptography;
using System.Text;

public class CachedConversionService : IDisposable
{
    private readonly LegacyBridgeConverter _converter;
    private readonly IMemoryCache _cache;
    private readonly MemoryCacheEntryOptions _cacheOptions;
    
    public CachedConversionService()
    {
        _converter = new LegacyBridgeConverter();
        _cache = new MemoryCache(new MemoryCacheOptions
        {
            SizeLimit = 1000 // Maximum number of cached items
        });
        
        _cacheOptions = new MemoryCacheEntryOptions
        {
            SlidingExpiration = TimeSpan.FromMinutes(15),
            Size = 1 // Each entry counts as 1 toward the size limit
        };
    }
    
    public string ConvertRtfToMarkdown(string rtfContent)
    {
        string cacheKey = GenerateCacheKey("rtf-to-md", rtfContent);
        
        if (_cache.TryGetValue<string>(cacheKey, out var cachedResult))
        {
            return cachedResult;
        }
        
        var result = _converter.ConvertRtfToMarkdown(rtfContent);
        _cache.Set(cacheKey, result, _cacheOptions);
        
        return result;
    }
    
    public string ConvertMarkdownToRtf(string markdownContent)
    {
        string cacheKey = GenerateCacheKey("md-to-rtf", markdownContent);
        
        if (_cache.TryGetValue<string>(cacheKey, out var cachedResult))
        {
            return cachedResult;
        }
        
        var result = _converter.ConvertMarkdownToRtf(markdownContent);
        _cache.Set(cacheKey, result, _cacheOptions);
        
        return result;
    }
    
    private string GenerateCacheKey(string prefix, string content)
    {
        using var sha256 = SHA256.Create();
        var hash = sha256.ComputeHash(Encoding.UTF8.GetBytes(content));
        return $"{prefix}:{Convert.ToBase64String(hash)}";
    }
    
    public void ClearCache()
    {
        (_cache as MemoryCache)?.Clear();
    }
    
    public void Dispose()
    {
        _cache?.Dispose();
        _converter?.Dispose();
    }
}
```

## Troubleshooting

### Common Issues and Solutions

#### DLL Not Found Exception

```csharp
public class DllResolver
{
    public static void ResolveLegacyBridgeDll()
    {
        // Add custom DLL resolution
        AppDomain.CurrentDomain.AssemblyResolve += (sender, args) =>
        {
            if (args.Name.Contains("legacybridge"))
            {
                string dllPath = Path.Combine(
                    AppDomain.CurrentDomain.BaseDirectory,
                    Environment.Is64BitProcess ? "x64" : "x86",
                    "legacybridge.dll"
                );
                
                if (File.Exists(dllPath))
                {
                    return Assembly.LoadFrom(dllPath);
                }
            }
            return null;
        };
    }
}

// Call this in your application startup
DllResolver.ResolveLegacyBridgeDll();
```

#### Platform-Specific Loading

```csharp
public static class PlatformHelper
{
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool SetDllDirectory(string lpPathName);
    
    public static void ConfigureDllPath()
    {
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            string dllPath = Path.Combine(
                AppDomain.CurrentDomain.BaseDirectory,
                Environment.Is64BitProcess ? "x64" : "x86"
            );
            
            SetDllDirectory(dllPath);
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
        {
            // Set LD_LIBRARY_PATH for Linux
            string currentPath = Environment.GetEnvironmentVariable("LD_LIBRARY_PATH") ?? "";
            string newPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "lib");
            Environment.SetEnvironmentVariable("LD_LIBRARY_PATH", 
                $"{newPath}{Path.PathSeparator}{currentPath}");
        }
    }
}
```

### Debugging Native Interop Issues

```csharp
public class InteropDebugger
{
    private static readonly bool EnableDebugLogging = true;
    
    [DllImport("kernel32.dll")]
    static extern uint GetLastError();
    
    public static void LogNativeCall(string functionName, params object[] parameters)
    {
        if (!EnableDebugLogging) return;
        
        Console.WriteLine($"[NATIVE CALL] {functionName}");
        for (int i = 0; i < parameters.Length; i++)
        {
            Console.WriteLine($"  Param {i}: {parameters[i]?.ToString() ?? "null"}");
        }
    }
    
    public static void LogNativeResult(string functionName, object result)
    {
        if (!EnableDebugLogging) return;
        
        Console.WriteLine($"[NATIVE RESULT] {functionName}: {result?.ToString() ?? "null"}");
        
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            uint error = GetLastError();
            if (error != 0)
            {
                Console.WriteLine($"  Last Win32 Error: {error}");
            }
        }
    }
}
```

## Complete Sample Application

Here's a complete console application demonstrating all features:

```csharp
using System;
using System.IO;
using System.Threading.Tasks;
using LegacyBridge;

namespace LegacyBridgeDemo
{
    class Program
    {
        static async Task Main(string[] args)
        {
            Console.WriteLine("LegacyBridge .NET Demo Application");
            Console.WriteLine("==================================\n");
            
            try
            {
                using var converter = new LegacyBridgeConverter();
                
                // Test connection
                if (!converter.TestConnection())
                {
                    Console.WriteLine("Error: Failed to connect to LegacyBridge DLL");
                    Console.WriteLine("Make sure legacybridge.dll is in the application directory");
                    return;
                }
                
                Console.WriteLine($"✓ Connected to LegacyBridge v{converter.GetVersion()}\n");
                
                while (true)
                {
                    ShowMenu();
                    var choice = Console.ReadLine();
                    
                    switch (choice)
                    {
                        case "1":
                            await DemoStringConversion(converter);
                            break;
                        case "2":
                            await DemoFileConversion(converter);
                            break;
                        case "3":
                            await DemoBatchProcessing(converter);
                            break;
                        case "4":
                            await DemoValidation(converter);
                            break;
                        case "5":
                            await DemoTemplates(converter);
                            break;
                        case "6":
                            await DemoPerformanceTest(converter);
                            break;
                        case "0":
                            return;
                        default:
                            Console.WriteLine("Invalid choice. Please try again.");
                            break;
                    }
                    
                    Console.WriteLine("\nPress any key to continue...");
                    Console.ReadKey();
                    Console.Clear();
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Fatal error: {ex.Message}");
            }
        }
        
        static void ShowMenu()
        {
            Console.WriteLine("Choose an option:");
            Console.WriteLine("1. String Conversion Demo");
            Console.WriteLine("2. File Conversion Demo");
            Console.WriteLine("3. Batch Processing Demo");
            Console.WriteLine("4. Validation Demo");
            Console.WriteLine("5. Template Demo");
            Console.WriteLine("6. Performance Test");
            Console.WriteLine("0. Exit");
            Console.Write("\nYour choice: ");
        }
        
        static async Task DemoStringConversion(LegacyBridgeConverter converter)
        {
            Console.WriteLine("\n=== String Conversion Demo ===\n");
            
            // Sample Markdown
            string markdown = @"# LegacyBridge Demo

This demonstrates **bold**, *italic*, and ***bold italic*** text.

## Features

- Fast conversion
- High fidelity
- Enterprise ready

### Code Example

```csharp
var result = converter.ConvertMarkdownToRtf(input);
```

> This is a blockquote with important information.

Visit [our website](https://example.com) for more details.";
            
            Console.WriteLine("Original Markdown:");
            Console.WriteLine("------------------");
            Console.WriteLine(markdown);
            
            // Convert to RTF
            string rtf = converter.ConvertMarkdownToRtf(markdown);
            Console.WriteLine("\n\nConverted RTF:");
            Console.WriteLine("--------------");
            Console.WriteLine(rtf.Substring(0, Math.Min(500, rtf.Length)) + "...");
            
            // Convert back to Markdown
            string backToMarkdown = converter.ConvertRtfToMarkdown(rtf);
            Console.WriteLine("\n\nConverted back to Markdown:");
            Console.WriteLine("---------------------------");
            Console.WriteLine(backToMarkdown);
            
            // Check fidelity
            double similarity = CalculateSimilarity(markdown, backToMarkdown);
            Console.WriteLine($"\n\nConversion fidelity: {similarity:P1}");
        }
        
        static async Task DemoFileConversion(LegacyBridgeConverter converter)
        {
            Console.WriteLine("\n=== File Conversion Demo ===\n");
            
            // Create test file
            string testFile = "test_document.md";
            string outputFile = "test_document.rtf";
            
            await File.WriteAllTextAsync(testFile, @"# Test Document

This is a test document for file conversion.

## Section 1

Some content here with **formatting**.

## Section 2

More content with *different* formatting.");
            
            Console.WriteLine($"Created test file: {testFile}");
            
            // Convert file
            try
            {
                converter.ConvertMarkdownFile(testFile, outputFile);
                Console.WriteLine($"✓ Successfully converted to: {outputFile}");
                
                var info = new FileInfo(outputFile);
                Console.WriteLine($"  Output size: {info.Length:N0} bytes");
            }
            catch (ConversionException ex)
            {
                Console.WriteLine($"✗ Conversion failed: {ex.Message}");
            }
            finally
            {
                // Cleanup
                if (File.Exists(testFile)) File.Delete(testFile);
                if (File.Exists(outputFile)) File.Delete(outputFile);
            }
        }
        
        static async Task DemoBatchProcessing(LegacyBridgeConverter converter)
        {
            Console.WriteLine("\n=== Batch Processing Demo ===\n");
            
            // Create test directory with files
            string testDir = "test_batch";
            string outputDir = "test_batch_output";
            
            Directory.CreateDirectory(testDir);
            
            // Create test files
            for (int i = 1; i <= 5; i++)
            {
                string content = $"# Document {i}\n\nThis is test document number {i}.";
                await File.WriteAllTextAsync(Path.Combine(testDir, $"doc{i}.md"), content);
            }
            
            Console.WriteLine($"Created {5} test files in '{testDir}'");
            
            try
            {
                var sw = System.Diagnostics.Stopwatch.StartNew();
                int converted = converter.ConvertFolder(testDir, outputDir, ConversionDirection.MarkdownToRtf);
                sw.Stop();
                
                Console.WriteLine($"✓ Converted {converted} files in {sw.ElapsedMilliseconds}ms");
                Console.WriteLine($"  Average: {sw.ElapsedMilliseconds / converted}ms per file");
            }
            catch (ConversionException ex)
            {
                Console.WriteLine($"✗ Batch conversion failed: {ex.Message}");
            }
            finally
            {
                // Cleanup
                if (Directory.Exists(testDir)) Directory.Delete(testDir, true);
                if (Directory.Exists(outputDir)) Directory.Delete(outputDir, true);
            }
        }
        
        static async Task DemoValidation(LegacyBridgeConverter converter)
        {
            Console.WriteLine("\n=== Validation Demo ===\n");
            
            // Valid documents
            string validRtf = @"{\rtf1\ansi\deff0 {\fonttbl {\f0 Times New Roman;}}
\f0\fs24 This is valid RTF.}";
            
            string validMarkdown = "# Valid Markdown\n\nThis is **valid** markdown.";
            
            // Invalid documents
            string invalidRtf = @"{\rtf1\ansi\deff0 Missing closing brace";
            string invalidMarkdown = "# \n\n[Broken link](";
            
            Console.WriteLine("Validating documents...\n");
            
            Console.WriteLine($"Valid RTF: {(converter.ValidateRtf(validRtf) ? "✓ PASS" : "✗ FAIL")}");
            Console.WriteLine($"Invalid RTF: {(converter.ValidateRtf(invalidRtf) ? "✗ UNEXPECTED PASS" : "✓ CORRECTLY FAILED")}");
            Console.WriteLine($"Valid Markdown: {(converter.ValidateMarkdown(validMarkdown) ? "✓ PASS" : "✗ FAIL")}");
            Console.WriteLine($"Invalid Markdown: {(converter.ValidateMarkdown(invalidMarkdown) ? "✓ PASS (Markdown is permissive)" : "✗ FAIL")}");
        }
        
        static async Task DemoTemplates(LegacyBridgeConverter converter)
        {
            Console.WriteLine("\n=== Template Demo ===\n");
            
            string baseRtf = @"{\rtf1\ansi\deff0 {\fonttbl {\f0 Times New Roman;}}
\f0\fs24 This is a document that will have templates applied.}";
            
            string[] templates = { "minimal", "professional", "academic" };
            
            foreach (var template in templates)
            {
                try
                {
                    Console.WriteLine($"\nApplying '{template}' template...");
                    string result = converter.ApplyTemplate(baseRtf, template);
                    Console.WriteLine($"✓ Template applied successfully");
                    Console.WriteLine($"  Result size: {result.Length} characters");
                }
                catch (ConversionException ex)
                {
                    Console.WriteLine($"✗ Failed to apply template: {ex.Message}");
                }
            }
        }
        
        static async Task DemoPerformanceTest(LegacyBridgeConverter converter)
        {
            Console.WriteLine("\n=== Performance Test ===\n");
            
            string testMarkdown = @"# Performance Test Document

This document is used to test conversion performance.

## Section with Formatting

Here we have **bold text**, *italic text*, and `inline code`.

### Lists

- Item 1
- Item 2
  - Nested item
- Item 3

### Code Block

```csharp
public void TestMethod()
{
    Console.WriteLine(""Performance test"");
}
```";
            
            int iterations = 1000;
            Console.WriteLine($"Running {iterations} conversions...\n");
            
            // Markdown to RTF
            var sw = System.Diagnostics.Stopwatch.StartNew();
            for (int i = 0; i < iterations; i++)
            {
                _ = converter.ConvertMarkdownToRtf(testMarkdown);
            }
            sw.Stop();
            
            double mdToRtfRate = iterations / (sw.ElapsedMilliseconds / 1000.0);
            Console.WriteLine($"Markdown → RTF: {mdToRtfRate:N0} conversions/second");
            Console.WriteLine($"  Average time: {sw.ElapsedMilliseconds / (double)iterations:N2}ms");
            
            // RTF to Markdown
            string testRtf = converter.ConvertMarkdownToRtf(testMarkdown);
            sw.Restart();
            for (int i = 0; i < iterations; i++)
            {
                _ = converter.ConvertRtfToMarkdown(testRtf);
            }
            sw.Stop();
            
            double rtfToMdRate = iterations / (sw.ElapsedMilliseconds / 1000.0);
            Console.WriteLine($"\nRTF → Markdown: {rtfToMdRate:N0} conversions/second");
            Console.WriteLine($"  Average time: {sw.ElapsedMilliseconds / (double)iterations:N2}ms");
        }
        
        static double CalculateSimilarity(string str1, string str2)
        {
            if (string.IsNullOrEmpty(str1) || string.IsNullOrEmpty(str2))
                return 0;
            
            int maxLen = Math.Max(str1.Length, str2.Length);
            if (maxLen == 0) return 1.0;
            
            int distance = LevenshteinDistance(str1, str2);
            return 1.0 - (distance / (double)maxLen);
        }
        
        static int LevenshteinDistance(string s, string t)
        {
            int n = s.Length;
            int m = t.Length;
            int[,] d = new int[n + 1, m + 1];
            
            if (n == 0) return m;
            if (m == 0) return n;
            
            for (int i = 0; i <= n; d[i, 0] = i++) { }
            for (int j = 0; j <= m; d[0, j] = j++) { }
            
            for (int i = 1; i <= n; i++)
            {
                for (int j = 1; j <= m; j++)
                {
                    int cost = (t[j - 1] == s[i - 1]) ? 0 : 1;
                    d[i, j] = Math.Min(
                        Math.Min(d[i - 1, j] + 1, d[i, j - 1] + 1),
                        d[i - 1, j - 1] + cost);
                }
            }
            
            return d[n, m];
        }
    }
}
```

## Best Practices

1. **Always use using statements** or implement IDisposable properly
2. **Validate input** before conversion to avoid exceptions
3. **Handle errors gracefully** with try-catch blocks
4. **Use async methods** for file operations and batch processing
5. **Implement caching** for frequently converted documents
6. **Monitor memory usage** for large batch operations
7. **Use dependency injection** in web applications
8. **Test thoroughly** on all target platforms

## Conclusion

This guide provides comprehensive coverage of integrating LegacyBridge with .NET applications. The provided examples and patterns can be adapted to your specific use case, whether you're building a desktop application, web service, or batch processing tool.

For additional support or questions, contact [blewisxx@gmail.com](mailto:blewisxx@gmail.com).

---

Built with ❤️ by [Beau Lewis](mailto:blewisxx@gmail.com)