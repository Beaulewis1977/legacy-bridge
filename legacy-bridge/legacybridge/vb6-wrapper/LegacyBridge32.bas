Attribute VB_Name = "LegacyBridge32"
' LegacyBridge VB6 Wrapper Module - 32-bit Optimized Version
' Provides safe integration with LegacyBridge DLL for 32-bit VB6 applications
'
' Version: 1.0.0
' Architecture: 32-bit optimized
' Last Updated: 2024-01-24

Option Explicit

' ============================================================================
' Constants and Types
' ============================================================================

' Memory management constants for 32-bit systems
Private Const MAX_BUFFER_SIZE As Long = 52428800    ' 50MB max for 32-bit
Private Const CHUNK_SIZE As Long = 1048576          ' 1MB chunks for large operations
Private Const MAX_BATCH_SIZE As Long = 100          ' Limit batch operations

' Error codes
Private Const FFI_SUCCESS As Long = 0
Private Const FFI_NULL_POINTER As Long = -1
Private Const FFI_INVALID_UTF8 As Long = -2
Private Const FFI_CONVERSION_ERROR As Long = -3
Private Const FFI_ALLOCATION_ERROR As Long = -4

' Error handling structure
Private Type LegacyBridgeError
    Code As Long
    Message As String
    Source As String
    Timestamp As Date
End Type

' Performance monitoring
Private Type PerformanceMetrics
    LastOperationTime As Double
    TotalConversions As Long
    TotalErrors As Long
    AverageConversionTime As Double
End Type

' ============================================================================
' Module-level Variables
' ============================================================================

Private m_LastError As LegacyBridgeError
Private m_Metrics As PerformanceMetrics
Private m_DllLoaded As Boolean

' ============================================================================
' API Declarations - 32-bit Safe Versions
' ============================================================================

' Core conversion functions with 32-bit safe signatures
Private Declare Function legacybridge_rtf_to_markdown_32bit_safe Lib "legacybridge.dll" _
    (ByVal rtfContent As String, ByRef outputLength As Long) As Long

Private Declare Function legacybridge_markdown_to_rtf Lib "legacybridge.dll" _
    (ByVal markdownContent As String, ByRef outputBuffer As Long, ByRef outputLength As Long) As Long

Private Declare Sub legacybridge_free_string Lib "legacybridge.dll" _
    (ByVal ptr As Long)

Private Declare Function legacybridge_get_last_error Lib "legacybridge.dll" _
    (ByVal buffer As String, ByVal bufferSize As Long) As Long

Private Declare Function legacybridge_get_version Lib "legacybridge.dll" () As Long

Private Declare Function legacybridge_test_connection Lib "legacybridge.dll" () As Long

' Architecture and memory functions
Private Declare Function legacybridge_get_architecture_bits Lib "legacybridge.dll" () As Long

Private Declare Function legacybridge_get_memory_usage Lib "legacybridge.dll" () As Long

Private Declare Function legacybridge_get_max_string_size Lib "legacybridge.dll" () As Long

Private Declare Sub legacybridge_reset_memory_arena Lib "legacybridge.dll" ()

' Batch operations with 32-bit safe signatures
Private Declare Function legacybridge_batch_rtf_to_markdown_32bit_safe Lib "legacybridge.dll" _
    (ByRef rtfArray As Long, ByVal count As Long, ByRef outputArray As Long, ByRef outputLengths As Long) As Long

' File operations
Private Declare Function legacybridge_convert_rtf_file_to_md Lib "legacybridge.dll" _
    (ByVal inputPath As String, ByVal outputPath As String) As Long

Private Declare Function legacybridge_convert_md_file_to_rtf Lib "legacybridge.dll" _
    (ByVal inputPath As String, ByVal outputPath As String) As Long

' Utility functions
Private Declare Function legacybridge_validate_rtf_document Lib "legacybridge.dll" _
    (ByVal rtfContent As String, ByRef outputBuffer As Long, ByRef outputLength As Long) As Long

Private Declare Function legacybridge_extract_plain_text Lib "legacybridge.dll" _
    (ByVal rtfContent As String, ByRef outputBuffer As Long, ByRef outputLength As Long) As Long

' Windows API for memory operations
Private Declare Sub CopyMemory Lib "kernel32" Alias "RtlMoveMemory" _
    (Destination As Any, Source As Any, ByVal Length As Long)

Private Declare Function lstrlen Lib "kernel32" Alias "lstrlenA" _
    (ByVal lpString As Long) As Long

Private Declare Function GetTickCount Lib "kernel32" () As Long

' ============================================================================
' Initialization and Cleanup
' ============================================================================

Public Function InitializeLegacyBridge() As Boolean
    On Error GoTo ErrorHandler
    
    ' Test DLL connection
    If legacybridge_test_connection() = 1 Then
        m_DllLoaded = True
        
        ' Check architecture
        Dim archBits As Long
        archBits = legacybridge_get_architecture_bits()
        
        If archBits <> 32 Then
            Debug.Print "Warning: DLL is " & archBits & "-bit, expected 32-bit"
        End If
        
        ' Initialize metrics
        With m_Metrics
            .TotalConversions = 0
            .TotalErrors = 0
            .AverageConversionTime = 0
        End With
        
        InitializeLegacyBridge = True
    Else
        m_DllLoaded = False
        InitializeLegacyBridge = False
    End If
    
    Exit Function
    
ErrorHandler:
    SetError Err.Number, "Failed to initialize LegacyBridge: " & Err.Description
    InitializeLegacyBridge = False
End Function

Public Sub CleanupLegacyBridge()
    ' Reset memory arena for 32-bit systems
    If m_DllLoaded Then
        legacybridge_reset_memory_arena
    End If
    m_DllLoaded = False
End Sub

' ============================================================================
' Core Conversion Functions - 32-bit Optimized
' ============================================================================

Public Function ConvertRTFToMarkdown(ByVal rtfContent As String) As String
    On Error GoTo ErrorHandler
    
    Dim startTime As Long
    Dim outputPtr As Long
    Dim outputLength As Long
    Dim result As String
    
    ' Validate input
    If Not ValidateInput(rtfContent) Then
        ConvertRTFToMarkdown = ""
        Exit Function
    End If
    
    startTime = GetTickCount()
    
    ' Use 32-bit safe version
    outputPtr = legacybridge_rtf_to_markdown_32bit_safe(rtfContent, outputLength)
    
    If outputPtr <> 0 Then
        ' Convert pointer to string
        result = PointerToString(outputPtr, outputLength)
        
        ' Free the allocated memory
        legacybridge_free_string outputPtr
        
        ' Update metrics
        UpdateMetrics GetTickCount() - startTime, True
        
        ConvertRTFToMarkdown = result
    Else
        SetError FFI_CONVERSION_ERROR, "RTF to Markdown conversion failed"
        UpdateMetrics GetTickCount() - startTime, False
        ConvertRTFToMarkdown = ""
    End If
    
    Exit Function
    
ErrorHandler:
    SetError Err.Number, "ConvertRTFToMarkdown error: " & Err.Description
    UpdateMetrics 0, False
    ConvertRTFToMarkdown = ""
End Function

Public Function ConvertMarkdownToRTF(ByVal markdownContent As String) As String
    On Error GoTo ErrorHandler
    
    Dim startTime As Long
    Dim outputPtr As Long
    Dim outputLength As Long
    Dim result As Long
    Dim output As String
    
    ' Validate input
    If Not ValidateInput(markdownContent) Then
        ConvertMarkdownToRTF = ""
        Exit Function
    End If
    
    startTime = GetTickCount()
    
    ' Call DLL function
    result = legacybridge_markdown_to_rtf(markdownContent, outputPtr, outputLength)
    
    If result = FFI_SUCCESS And outputPtr <> 0 Then
        ' Convert pointer to string
        output = PointerToString(outputPtr, outputLength)
        
        ' Free the allocated memory
        legacybridge_free_string outputPtr
        
        ' Update metrics
        UpdateMetrics GetTickCount() - startTime, True
        
        ConvertMarkdownToRTF = output
    Else
        SetError result, "Markdown to RTF conversion failed"
        UpdateMetrics GetTickCount() - startTime, False
        ConvertMarkdownToRTF = ""
    End If
    
    Exit Function
    
ErrorHandler:
    SetError Err.Number, "ConvertMarkdownToRTF error: " & Err.Description
    UpdateMetrics 0, False
    ConvertMarkdownToRTF = ""
End Function

' ============================================================================
' Batch Operations - 32-bit Safe
' ============================================================================

Public Function BatchConvertRTFToMarkdown(rtfDocuments() As String) As String()
    On Error GoTo ErrorHandler
    
    Dim count As Long
    Dim i As Long
    Dim results() As String
    Dim actualCount As Long
    
    count = UBound(rtfDocuments) - LBound(rtfDocuments) + 1
    
    ' Enforce 32-bit batch size limit
    If count > MAX_BATCH_SIZE Then
        SetError FFI_ALLOCATION_ERROR, "Batch size exceeds 32-bit limit of " & MAX_BATCH_SIZE
        Exit Function
    End If
    
    ReDim results(LBound(rtfDocuments) To UBound(rtfDocuments))
    
    ' Process in chunks if needed
    For i = LBound(rtfDocuments) To UBound(rtfDocuments)
        results(i) = ConvertRTFToMarkdown(rtfDocuments(i))
        
        ' Periodically reset arena on 32-bit systems
        If (i - LBound(rtfDocuments) + 1) Mod 10 = 0 Then
            legacybridge_reset_memory_arena
        End If
    Next i
    
    BatchConvertRTFToMarkdown = results
    Exit Function
    
ErrorHandler:
    SetError Err.Number, "BatchConvertRTFToMarkdown error: " & Err.Description
    Exit Function
End Function

' ============================================================================
' File Operations
' ============================================================================

Public Function ConvertRTFFileToMarkdown(ByVal inputPath As String, ByVal outputPath As String) As Boolean
    On Error GoTo ErrorHandler
    
    Dim result As Long
    
    ' Validate paths
    If Len(inputPath) = 0 Or Len(outputPath) = 0 Then
        SetError FFI_NULL_POINTER, "Invalid file paths"
        ConvertRTFFileToMarkdown = False
        Exit Function
    End If
    
    result = legacybridge_convert_rtf_file_to_md(inputPath, outputPath)
    
    If result = FFI_SUCCESS Then
        ConvertRTFFileToMarkdown = True
    Else
        SetError result, "File conversion failed"
        ConvertRTFFileToMarkdown = False
    End If
    
    Exit Function
    
ErrorHandler:
    SetError Err.Number, "ConvertRTFFileToMarkdown error: " & Err.Description
    ConvertRTFFileToMarkdown = False
End Function

' ============================================================================
' Utility Functions
' ============================================================================

Public Function ValidateRTFDocument(ByVal rtfContent As String) As String
    On Error GoTo ErrorHandler
    
    Dim outputPtr As Long
    Dim outputLength As Long
    Dim result As Long
    
    result = legacybridge_validate_rtf_document(rtfContent, outputPtr, outputLength)
    
    If result = FFI_SUCCESS And outputPtr <> 0 Then
        ValidateRTFDocument = PointerToString(outputPtr, outputLength)
        legacybridge_free_string outputPtr
    Else
        ValidateRTFDocument = "Validation failed"
    End If
    
    Exit Function
    
ErrorHandler:
    ValidateRTFDocument = "Validation error: " & Err.Description
End Function

Public Function ExtractPlainText(ByVal rtfContent As String) As String
    On Error GoTo ErrorHandler
    
    Dim outputPtr As Long
    Dim outputLength As Long
    Dim result As Long
    
    result = legacybridge_extract_plain_text(rtfContent, outputPtr, outputLength)
    
    If result = FFI_SUCCESS And outputPtr <> 0 Then
        ExtractPlainText = PointerToString(outputPtr, outputLength)
        legacybridge_free_string outputPtr
    Else
        ExtractPlainText = ""
    End If
    
    Exit Function
    
ErrorHandler:
    ExtractPlainText = ""
End Function

' ============================================================================
' Memory and Performance Functions
' ============================================================================

Public Function GetMemoryUsage() As Long
    GetMemoryUsage = legacybridge_get_memory_usage()
End Function

Public Function GetMaxStringSize() As Long
    GetMaxStringSize = legacybridge_get_max_string_size()
End Function

Public Sub ResetMemoryArena()
    legacybridge_reset_memory_arena
End Sub

Public Function GetPerformanceMetrics() As String
    Dim output As String
    
    With m_Metrics
        output = "Total Conversions: " & .TotalConversions & vbCrLf
        output = output & "Total Errors: " & .TotalErrors & vbCrLf
        output = output & "Success Rate: " & Format$((.TotalConversions - .TotalErrors) / IIf(.TotalConversions = 0, 1, .TotalConversions) * 100, "0.00") & "%" & vbCrLf
        output = output & "Average Time: " & Format$(.AverageConversionTime, "0.00") & " ms" & vbCrLf
        output = output & "Memory Usage: " & Format$(GetMemoryUsage() / 1024 / 1024, "0.00") & " MB"
    End With
    
    GetPerformanceMetrics = output
End Function

' ============================================================================
' Error Handling
' ============================================================================

Public Function GetLastError() As String
    If m_LastError.Code <> 0 Then
        GetLastError = "[" & m_LastError.Code & "] " & m_LastError.Message & _
                      " (Source: " & m_LastError.Source & " at " & m_LastError.Timestamp & ")"
    Else
        GetLastError = "No error"
    End If
End Function

Public Sub ClearError()
    With m_LastError
        .Code = 0
        .Message = ""
        .Source = ""
        .Timestamp = 0
    End With
End Sub

' ============================================================================
' Helper Functions
' ============================================================================

Private Function ValidateInput(ByVal content As String) As Boolean
    ' Check for null/empty
    If Len(content) = 0 Then
        SetError FFI_NULL_POINTER, "Input content is empty"
        ValidateInput = False
        Exit Function
    End If
    
    ' Check size limit for 32-bit systems
    If Len(content) > MAX_BUFFER_SIZE Then
        SetError FFI_ALLOCATION_ERROR, "Input exceeds 32-bit buffer limit"
        ValidateInput = False
        Exit Function
    End If
    
    ValidateInput = True
End Function

Private Function PointerToString(ByVal ptr As Long, ByVal length As Long) As String
    Dim buffer As String
    
    If ptr = 0 Or length = 0 Then
        PointerToString = ""
        Exit Function
    End If
    
    ' Create buffer
    buffer = Space$(length)
    
    ' Copy memory
    CopyMemory ByVal StrPtr(buffer), ByVal ptr, length
    
    PointerToString = buffer
End Function

Private Sub SetError(ByVal code As Long, ByVal message As String)
    With m_LastError
        .Code = code
        .Message = message
        .Source = "LegacyBridge32"
        .Timestamp = Now
    End With
End Sub

Private Sub UpdateMetrics(ByVal operationTime As Long, ByVal success As Boolean)
    With m_Metrics
        .LastOperationTime = operationTime
        .TotalConversions = .TotalConversions + 1
        
        If Not success Then
            .TotalErrors = .TotalErrors + 1
        End If
        
        ' Update average (simple moving average)
        If .AverageConversionTime = 0 Then
            .AverageConversionTime = operationTime
        Else
            .AverageConversionTime = (.AverageConversionTime * (.TotalConversions - 1) + operationTime) / .TotalConversions
        End If
    End With
End Sub

' ============================================================================
' Version Information
' ============================================================================

Public Function GetVersion() As String
    Dim versionPtr As Long
    Dim versionStr As String
    
    versionPtr = legacybridge_get_version()
    
    If versionPtr <> 0 Then
        ' Get string length
        Dim length As Long
        length = lstrlen(versionPtr)
        
        If length > 0 Then
            versionStr = PointerToString(versionPtr, length)
        Else
            versionStr = "Unknown"
        End If
    Else
        versionStr = "Unknown"
    End If
    
    GetVersion = versionStr & " (32-bit optimized)"
End Function

' ============================================================================
' Test Functions
' ============================================================================

Public Sub TestLegacyBridge32()
    Dim testRTF As String
    Dim testMD As String
    Dim result As String
    
    Debug.Print "=== LegacyBridge 32-bit Test Suite ==="
    Debug.Print "Architecture: " & legacybridge_get_architecture_bits() & "-bit"
    Debug.Print "Max String Size: " & Format$(GetMaxStringSize() / 1024 / 1024, "0.00") & " MB"
    Debug.Print ""
    
    ' Test 1: Simple conversion
    testRTF = "{\rtf1\ansi\deff0 {\fonttbl{\f0 Times New Roman;}}Hello World\par}"
    Debug.Print "Test 1 - Simple RTF conversion:"
    result = ConvertRTFToMarkdown(testRTF)
    Debug.Print "Result: " & result
    Debug.Print ""
    
    ' Test 2: Memory usage
    Debug.Print "Test 2 - Memory usage:"
    Debug.Print "Before: " & GetMemoryUsage() & " bytes"
    
    Dim i As Long
    For i = 1 To 10
        result = ConvertRTFToMarkdown(testRTF)
    Next i
    
    Debug.Print "After 10 conversions: " & GetMemoryUsage() & " bytes"
    ResetMemoryArena
    Debug.Print "After arena reset: " & GetMemoryUsage() & " bytes"
    Debug.Print ""
    
    ' Test 3: Performance metrics
    Debug.Print "Test 3 - Performance metrics:"
    Debug.Print GetPerformanceMetrics()
    
    Debug.Print "=== Test Complete ==="
End Sub