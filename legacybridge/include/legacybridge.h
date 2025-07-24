/*
 * LegacyBridge C Header File
 * Version: 1.0.0
 * 
 * This header provides C-compatible declarations for the LegacyBridge
 * Markdown <-> RTF conversion library.
 */

#ifndef LEGACYBRIDGE_H
#define LEGACYBRIDGE_H

#ifdef __cplusplus
extern "C" {
#endif

/* Error codes */
#define LB_SUCCESS           0
#define LB_ERROR_NULL_POINTER    -1
#define LB_ERROR_INVALID_UTF8    -2
#define LB_ERROR_CONVERSION      -3
#define LB_ERROR_ALLOCATION      -4

/* Windows DLL export/import macros */
#ifdef _WIN32
    #ifdef LEGACYBRIDGE_EXPORTS
        #define LEGACYBRIDGE_API __declspec(dllexport)
    #else
        #define LEGACYBRIDGE_API __declspec(dllimport)
    #endif
#else
    #define LEGACYBRIDGE_API
#endif

/*
 * Convert RTF to Markdown
 * 
 * Parameters:
 *   rtf_content - Null-terminated string containing RTF content
 *   output_buffer - Pointer to store the output buffer address
 *   output_length - Pointer to store the output length
 * 
 * Returns:
 *   0 on success, negative error code on failure
 * 
 * Note: Caller must free the output buffer using legacybridge_free_string()
 */
LEGACYBRIDGE_API int legacybridge_rtf_to_markdown(
    const char* rtf_content,
    char** output_buffer,
    int* output_length
);

/*
 * Convert Markdown to RTF
 * 
 * Parameters:
 *   markdown_content - Null-terminated string containing Markdown content
 *   output_buffer - Pointer to store the output buffer address
 *   output_length - Pointer to store the output length
 * 
 * Returns:
 *   0 on success, negative error code on failure
 * 
 * Note: Caller must free the output buffer using legacybridge_free_string()
 */
LEGACYBRIDGE_API int legacybridge_markdown_to_rtf(
    const char* markdown_content,
    char** output_buffer,
    int* output_length
);

/*
 * Free a string allocated by the library
 * 
 * Parameters:
 *   ptr - Pointer to the string to free
 */
LEGACYBRIDGE_API void legacybridge_free_string(char* ptr);

/*
 * Get the last error message
 * 
 * Parameters:
 *   buffer - Buffer to store the error message
 *   buffer_size - Size of the buffer
 * 
 * Returns:
 *   Number of bytes written (excluding null terminator)
 *   -1 if buffer is too small
 */
LEGACYBRIDGE_API int legacybridge_get_last_error(
    char* buffer,
    int buffer_size
);

/*
 * Get library version
 * 
 * Returns:
 *   Version string (do not free)
 */
LEGACYBRIDGE_API const char* legacybridge_get_version(void);

/*
 * Batch convert RTF files to Markdown
 * 
 * Parameters:
 *   rtf_array - Array of RTF content strings
 *   count - Number of items to convert
 *   output_array - Array to store output pointers
 *   output_lengths - Array to store output lengths
 * 
 * Returns:
 *   Number of successful conversions
 * 
 * Note: Caller must free each output string using legacybridge_free_string()
 */
LEGACYBRIDGE_API int legacybridge_batch_rtf_to_markdown(
    const char** rtf_array,
    int count,
    char** output_array,
    int* output_lengths
);

/*
 * Batch convert Markdown files to RTF
 * 
 * Parameters:
 *   markdown_array - Array of Markdown content strings
 *   count - Number of items to convert
 *   output_array - Array to store output pointers
 *   output_lengths - Array to store output lengths
 * 
 * Returns:
 *   Number of successful conversions
 * 
 * Note: Caller must free each output string using legacybridge_free_string()
 */
LEGACYBRIDGE_API int legacybridge_batch_markdown_to_rtf(
    const char** markdown_array,
    int count,
    char** output_array,
    int* output_lengths
);

#ifdef __cplusplus
}
#endif

#endif /* LEGACYBRIDGE_H */