* LegacyBridge VFP9 Wrapper Class
* Provides access to Markdown <-> RTF conversion functions
* Version: 1.0.0
*
* This class provides a VFP9-friendly wrapper around the legacybridge.dll
* for converting between Markdown and RTF formats.

DEFINE CLASS LegacyBridge AS Custom
    * Properties
    cDllPath = "legacybridge.dll"
    nLastError = 0
    cLastErrorMessage = ""
    
    * Error codes
    LB_SUCCESS = 0
    LB_ERROR_NULL_POINTER = -1
    LB_ERROR_INVALID_UTF8 = -2
    LB_ERROR_CONVERSION = -3
    LB_ERROR_ALLOCATION = -4
    
    * Constructor
    PROCEDURE Init
        * Declare DLL functions
        DECLARE INTEGER legacybridge_rtf_to_markdown IN (This.cDllPath) ;
            STRING rtf_content, ;
            INTEGER @ output_buffer, ;
            INTEGER @ output_length
            
        DECLARE INTEGER legacybridge_markdown_to_rtf IN (This.cDllPath) ;
            STRING markdown_content, ;
            INTEGER @ output_buffer, ;
            INTEGER @ output_length
            
        DECLARE legacybridge_free_string IN (This.cDllPath) ;
            INTEGER ptr
            
        DECLARE INTEGER legacybridge_get_last_error IN (This.cDllPath) ;
            STRING @ buffer, ;
            INTEGER buffer_size
            
        DECLARE INTEGER legacybridge_get_version IN (This.cDllPath)
        
        DECLARE INTEGER legacybridge_batch_rtf_to_markdown IN (This.cDllPath) ;
            INTEGER rtf_array, ;
            INTEGER count, ;
            INTEGER @ output_array, ;
            INTEGER @ output_lengths
            
        DECLARE INTEGER legacybridge_batch_markdown_to_rtf IN (This.cDllPath) ;
            INTEGER markdown_array, ;
            INTEGER count, ;
            INTEGER @ output_array, ;
            INTEGER @ output_lengths
            
        * Windows API declarations for memory operations
        DECLARE RtlMoveMemory IN kernel32 AS CopyMemory ;
            STRING @ Destination, ;
            INTEGER Source, ;
            INTEGER Length
            
        DECLARE INTEGER lstrlen IN kernel32 ;
            INTEGER lpString
    ENDPROC
    
    * Convert RTF to Markdown
    FUNCTION ConvertRtfToMarkdown(tcRtfContent)
        LOCAL lnOutputBuffer, lnOutputLength, lnResult, lcMarkdown
        
        lnOutputBuffer = 0
        lnOutputLength = 0
        lcMarkdown = ""
        
        * Call the DLL function
        lnResult = legacybridge_rtf_to_markdown(tcRtfContent, @lnOutputBuffer, @lnOutputLength)
        
        IF lnResult = This.LB_SUCCESS AND lnOutputBuffer != 0
            * Allocate string buffer and copy result
            lcMarkdown = SPACE(lnOutputLength)
            CopyMemory(@lcMarkdown, lnOutputBuffer, lnOutputLength)
            
            * Free the unmanaged memory
            legacybridge_free_string(lnOutputBuffer)
        ELSE
            This.SetError(lnResult)
        ENDIF
        
        RETURN lcMarkdown
    ENDFUNC
    
    * Convert Markdown to RTF
    FUNCTION ConvertMarkdownToRtf(tcMarkdownContent)
        LOCAL lnOutputBuffer, lnOutputLength, lnResult, lcRtf
        
        lnOutputBuffer = 0
        lnOutputLength = 0
        lcRtf = ""
        
        * Call the DLL function
        lnResult = legacybridge_markdown_to_rtf(tcMarkdownContent, @lnOutputBuffer, @lnOutputLength)
        
        IF lnResult = This.LB_SUCCESS AND lnOutputBuffer != 0
            * Allocate string buffer and copy result
            lcRtf = SPACE(lnOutputLength)
            CopyMemory(@lcRtf, lnOutputBuffer, lnOutputLength)
            
            * Free the unmanaged memory
            legacybridge_free_string(lnOutputBuffer)
        ELSE
            This.SetError(lnResult)
        ENDIF
        
        RETURN lcRtf
    ENDFUNC
    
    * Get library version
    FUNCTION GetLibraryVersion()
        LOCAL lnVersionPtr, lnVersionLength, lcVersion
        
        lnVersionPtr = legacybridge_get_version()
        
        IF lnVersionPtr != 0
            lnVersionLength = lstrlen(lnVersionPtr)
            lcVersion = SPACE(lnVersionLength)
            CopyMemory(@lcVersion, lnVersionPtr, lnVersionLength)
        ELSE
            lcVersion = "Unknown"
        ENDIF
        
        RETURN lcVersion
    ENDFUNC
    
    * Get last error message
    FUNCTION GetLastError()
        LOCAL lcErrorBuffer, lnResult
        
        lcErrorBuffer = SPACE(1024)
        lnResult = legacybridge_get_last_error(@lcErrorBuffer, 1024)
        
        IF lnResult > 0
            This.cLastErrorMessage = LEFT(lcErrorBuffer, lnResult)
        ELSE
            This.cLastErrorMessage = "Unknown error"
        ENDIF
        
        RETURN This.cLastErrorMessage
    ENDFUNC
    
    * Batch convert RTF to Markdown
    FUNCTION BatchConvertRtfToMarkdown(taRtfArray)
        LOCAL lnCount, i, lnResult
        LOCAL ARRAY laOutputBuffers[1], laOutputLengths[1], laResults[1]
        
        lnCount = ALEN(taRtfArray)
        
        * Dimension output arrays
        DIMENSION laOutputBuffers[lnCount]
        DIMENSION laOutputLengths[lnCount]
        DIMENSION laResults[lnCount]
        
        * Initialize arrays
        FOR i = 1 TO lnCount
            laOutputBuffers[i] = 0
            laOutputLengths[i] = 0
            laResults[i] = ""
        ENDFOR
        
        * Note: Batch conversion is complex in VFP due to pointer array handling
        * For production use, consider processing one at a time
        FOR i = 1 TO lnCount
            laResults[i] = This.ConvertRtfToMarkdown(taRtfArray[i])
        ENDFOR
        
        RETURN @laResults
    ENDFUNC
    
    * Batch convert Markdown to RTF
    FUNCTION BatchConvertMarkdownToRtf(taMarkdownArray)
        LOCAL lnCount, i
        LOCAL ARRAY laResults[1]
        
        lnCount = ALEN(taMarkdownArray)
        DIMENSION laResults[lnCount]
        
        * Note: Batch conversion is complex in VFP due to pointer array handling
        * For production use, consider processing one at a time
        FOR i = 1 TO lnCount
            laResults[i] = This.ConvertMarkdownToRtf(taMarkdownArray[i])
        ENDFOR
        
        RETURN @laResults
    ENDFUNC
    
    * Set error information
    PROTECTED PROCEDURE SetError(tnErrorCode)
        This.nLastError = tnErrorCode
        
        DO CASE
            CASE tnErrorCode = This.LB_ERROR_NULL_POINTER
                This.cLastErrorMessage = "Null pointer error"
            CASE tnErrorCode = This.LB_ERROR_INVALID_UTF8
                This.cLastErrorMessage = "Invalid UTF-8 encoding"
            CASE tnErrorCode = This.LB_ERROR_CONVERSION
                This.cLastErrorMessage = "Conversion failed: " + This.GetLastError()
            CASE tnErrorCode = This.LB_ERROR_ALLOCATION
                This.cLastErrorMessage = "Memory allocation failed"
            OTHERWISE
                This.cLastErrorMessage = "Unknown error: " + TRANSFORM(tnErrorCode)
        ENDCASE
    ENDPROC
    
    * Clear error state
    PROCEDURE ClearError
        This.nLastError = 0
        This.cLastErrorMessage = ""
    ENDPROC
    
ENDDEFINE

* Example usage function
FUNCTION TestLegacyBridge()
    LOCAL loConverter, lcRtf, lcMarkdown, lcVersion
    
    * Create converter instance
    loConverter = CREATEOBJECT("LegacyBridge")
    
    * Get version
    lcVersion = loConverter.GetLibraryVersion()
    ? "LegacyBridge Version: " + lcVersion
    
    * Test RTF to Markdown conversion
    lcRtf = "{\rtf1\ansi\deff0 {\fonttbl{\f0 Times New Roman;}} \b Hello World\b0\par}"
    lcMarkdown = loConverter.ConvertRtfToMarkdown(lcRtf)
    
    IF !EMPTY(lcMarkdown)
        ? "RTF to Markdown conversion successful:"
        ? lcMarkdown
    ELSE
        ? "RTF to Markdown conversion failed: " + loConverter.cLastErrorMessage
    ENDIF
    
    * Test Markdown to RTF conversion
    lcMarkdown = "# Hello World" + CHR(13) + CHR(10) + "This is a **test**."
    lcRtf = loConverter.ConvertMarkdownToRtf(lcMarkdown)
    
    IF !EMPTY(lcRtf)
        ? "Markdown to RTF conversion successful:"
        ? lcRtf
    ELSE
        ? "Markdown to RTF conversion failed: " + loConverter.cLastErrorMessage
    ENDIF
    
    RELEASE loConverter
ENDFUNC