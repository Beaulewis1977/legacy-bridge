Attribute VB_Name = "LegacyBridge"
' LegacyBridge VB6 Module
' Provides access to Markdown <-> RTF conversion functions
' Version: 1.0.0
' 
' This module provides a VB6-friendly wrapper around the legacybridge.dll
' for converting between Markdown and RTF formats.

Option Explicit

' Error codes returned by the DLL
Public Const LB_SUCCESS As Long = 0
Public Const LB_ERROR_NULL_POINTER As Long = -1
Public Const LB_ERROR_INVALID_UTF8 As Long = -2
Public Const LB_ERROR_CONVERSION As Long = -3
Public Const LB_ERROR_ALLOCATION As Long = -4

' DLL function declarations
Private Declare Function legacybridge_rtf_to_markdown Lib "legacybridge.dll" ( _
    ByVal rtf_content As String, _
    ByRef output_buffer As Long, _
    ByRef output_length As Long _
) As Long

Private Declare Function legacybridge_markdown_to_rtf Lib "legacybridge.dll" ( _
    ByVal markdown_content As String, _
    ByRef output_buffer As Long, _
    ByRef output_length As Long _
) As Long

Private Declare Sub legacybridge_free_string Lib "legacybridge.dll" ( _
    ByVal ptr As Long _
)

Private Declare Function legacybridge_get_last_error Lib "legacybridge.dll" ( _
    ByVal buffer As String, _
    ByVal buffer_size As Long _
) As Long

Private Declare Function legacybridge_get_version Lib "legacybridge.dll" () As Long

Private Declare Function legacybridge_batch_rtf_to_markdown Lib "legacybridge.dll" ( _
    ByRef rtf_array As Long, _
    ByVal count As Long, _
    ByRef output_array As Long, _
    ByRef output_lengths As Long _
) As Long

Private Declare Function legacybridge_batch_markdown_to_rtf Lib "legacybridge.dll" ( _
    ByRef markdown_array As Long, _
    ByVal count As Long, _
    ByRef output_array As Long, _
    ByRef output_lengths As Long _
) As Long

' Windows API for memory operations
Private Declare Sub CopyMemory Lib "kernel32" Alias "RtlMoveMemory" ( _
    ByVal Destination As String, _
    ByVal Source As Long, _
    ByVal Length As Long _
)

Private Declare Function lstrlen Lib "kernel32" Alias "lstrlenA" ( _
    ByVal lpString As Long _
) As Long

' Convert RTF to Markdown
' Returns the converted Markdown string or empty string on error
Public Function ConvertRtfToMarkdown(ByVal rtfContent As String) As String
    Dim outputBuffer As Long
    Dim outputLength As Long
    Dim result As Long
    Dim markdownResult As String
    
    On Error GoTo ErrorHandler
    
    ' Call the DLL function
    result = legacybridge_rtf_to_markdown(rtfContent, outputBuffer, outputLength)
    
    If result = LB_SUCCESS And outputBuffer <> 0 Then
        ' Allocate string buffer
        markdownResult = Space$(outputLength)
        
        ' Copy the result from unmanaged memory
        Call CopyMemory(markdownResult, outputBuffer, outputLength)
        
        ' Free the unmanaged memory
        Call legacybridge_free_string(outputBuffer)
        
        ConvertRtfToMarkdown = markdownResult
    Else
        ConvertRtfToMarkdown = ""
        Call RaiseConversionError(result)
    End If
    
    Exit Function
    
ErrorHandler:
    ConvertRtfToMarkdown = ""
    Err.Raise Err.Number, "LegacyBridge.ConvertRtfToMarkdown", Err.Description
End Function

' Convert Markdown to RTF
' Returns the converted RTF string or empty string on error
Public Function ConvertMarkdownToRtf(ByVal markdownContent As String) As String
    Dim outputBuffer As Long
    Dim outputLength As Long
    Dim result As Long
    Dim rtfResult As String
    
    On Error GoTo ErrorHandler
    
    ' Call the DLL function
    result = legacybridge_markdown_to_rtf(markdownContent, outputBuffer, outputLength)
    
    If result = LB_SUCCESS And outputBuffer <> 0 Then
        ' Allocate string buffer
        rtfResult = Space$(outputLength)
        
        ' Copy the result from unmanaged memory
        Call CopyMemory(rtfResult, outputBuffer, outputLength)
        
        ' Free the unmanaged memory
        Call legacybridge_free_string(outputBuffer)
        
        ConvertMarkdownToRtf = rtfResult
    Else
        ConvertMarkdownToRtf = ""
        Call RaiseConversionError(result)
    End If
    
    Exit Function
    
ErrorHandler:
    ConvertMarkdownToRtf = ""
    Err.Raise Err.Number, "LegacyBridge.ConvertMarkdownToRtf", Err.Description
End Function

' Get the library version
Public Function GetLibraryVersion() As String
    Dim versionPtr As Long
    Dim versionLength As Long
    Dim versionStr As String
    
    On Error GoTo ErrorHandler
    
    versionPtr = legacybridge_get_version()
    
    If versionPtr <> 0 Then
        versionLength = lstrlen(versionPtr)
        versionStr = Space$(versionLength)
        Call CopyMemory(versionStr, versionPtr, versionLength)
        GetLibraryVersion = versionStr
    Else
        GetLibraryVersion = "Unknown"
    End If
    
    Exit Function
    
ErrorHandler:
    GetLibraryVersion = "Error"
End Function

' Get the last error message
Public Function GetLastError() As String
    Dim errorBuffer As String * 1024
    Dim result As Long
    
    result = legacybridge_get_last_error(errorBuffer, 1024)
    
    If result > 0 Then
        GetLastError = Left$(errorBuffer, result)
    Else
        GetLastError = "Unknown error"
    End If
End Function

' Batch convert RTF files to Markdown
' Takes an array of RTF strings and returns an array of Markdown strings
Public Function BatchConvertRtfToMarkdown(rtfArray() As String) As String()
    Dim count As Long
    Dim i As Long
    Dim result As Long
    Dim outputArray() As Long
    Dim outputLengths() As Long
    Dim rtfPointers() As Long
    Dim markdownArray() As String
    
    On Error GoTo ErrorHandler
    
    count = UBound(rtfArray) - LBound(rtfArray) + 1
    
    ' Allocate arrays
    ReDim outputArray(0 To count - 1)
    ReDim outputLengths(0 To count - 1)
    ReDim rtfPointers(0 To count - 1)
    ReDim markdownArray(LBound(rtfArray) To UBound(rtfArray))
    
    ' Convert VB strings to C strings (this is simplified - in production you'd need proper conversion)
    For i = 0 To count - 1
        rtfPointers(i) = StrPtr(rtfArray(LBound(rtfArray) + i))
    Next i
    
    ' Call batch conversion
    result = legacybridge_batch_rtf_to_markdown(rtfPointers(0), count, outputArray(0), outputLengths(0))
    
    ' Process results
    For i = 0 To count - 1
        If outputArray(i) <> 0 Then
            markdownArray(LBound(rtfArray) + i) = Space$(outputLengths(i))
            Call CopyMemory(markdownArray(LBound(rtfArray) + i), outputArray(i), outputLengths(i))
            Call legacybridge_free_string(outputArray(i))
        Else
            markdownArray(LBound(rtfArray) + i) = ""
        End If
    Next i
    
    BatchConvertRtfToMarkdown = markdownArray
    Exit Function
    
ErrorHandler:
    ReDim markdownArray(LBound(rtfArray) To UBound(rtfArray))
    BatchConvertRtfToMarkdown = markdownArray
End Function

' Batch convert Markdown files to RTF
' Takes an array of Markdown strings and returns an array of RTF strings
Public Function BatchConvertMarkdownToRtf(markdownArray() As String) As String()
    Dim count As Long
    Dim i As Long
    Dim result As Long
    Dim outputArray() As Long
    Dim outputLengths() As Long
    Dim markdownPointers() As Long
    Dim rtfArray() As String
    
    On Error GoTo ErrorHandler
    
    count = UBound(markdownArray) - LBound(markdownArray) + 1
    
    ' Allocate arrays
    ReDim outputArray(0 To count - 1)
    ReDim outputLengths(0 To count - 1)
    ReDim markdownPointers(0 To count - 1)
    ReDim rtfArray(LBound(markdownArray) To UBound(markdownArray))
    
    ' Convert VB strings to C strings (this is simplified - in production you'd need proper conversion)
    For i = 0 To count - 1
        markdownPointers(i) = StrPtr(markdownArray(LBound(markdownArray) + i))
    Next i
    
    ' Call batch conversion
    result = legacybridge_batch_markdown_to_rtf(markdownPointers(0), count, outputArray(0), outputLengths(0))
    
    ' Process results
    For i = 0 To count - 1
        If outputArray(i) <> 0 Then
            rtfArray(LBound(markdownArray) + i) = Space$(outputLengths(i))
            Call CopyMemory(rtfArray(LBound(markdownArray) + i), outputArray(i), outputLengths(i))
            Call legacybridge_free_string(outputArray(i))
        Else
            rtfArray(LBound(markdownArray) + i) = ""
        End If
    Next i
    
    BatchConvertMarkdownToRtf = rtfArray
    Exit Function
    
ErrorHandler:
    ReDim rtfArray(LBound(markdownArray) To UBound(markdownArray))
    BatchConvertMarkdownToRtf = rtfArray
End Function

' Helper function to raise appropriate error based on error code
Private Sub RaiseConversionError(ByVal errorCode As Long)
    Dim errorMessage As String
    
    Select Case errorCode
        Case LB_ERROR_NULL_POINTER
            errorMessage = "Null pointer error"
        Case LB_ERROR_INVALID_UTF8
            errorMessage = "Invalid UTF-8 encoding"
        Case LB_ERROR_CONVERSION
            errorMessage = "Conversion failed: " & GetLastError()
        Case LB_ERROR_ALLOCATION
            errorMessage = "Memory allocation failed"
        Case Else
            errorMessage = "Unknown error: " & errorCode
    End Select
    
    Err.Raise vbObjectError + errorCode, "LegacyBridge", errorMessage
End Sub