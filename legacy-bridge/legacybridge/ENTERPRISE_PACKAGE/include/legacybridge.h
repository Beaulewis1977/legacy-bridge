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

/*
 * Test connection to verify DLL is loaded properly
 * 
 * Returns:
 *   1 on success
 */
LEGACYBRIDGE_API int legacybridge_test_connection(void);

/*
 * Get detailed version information
 * 
 * Parameters:
 *   major - Pointer to store major version
 *   minor - Pointer to store minor version
 *   patch - Pointer to store patch version
 * 
 * Returns:
 *   0 on success, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_get_version_info(
    int* major,
    int* minor,
    int* patch
);

/*
 * Convert RTF file to Markdown file
 * 
 * Parameters:
 *   input_path - Path to input RTF file
 *   output_path - Path to output Markdown file
 * 
 * Returns:
 *   0 on success, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_convert_rtf_file_to_md(
    const char* input_path,
    const char* output_path
);

/*
 * Convert Markdown file to RTF file
 * 
 * Parameters:
 *   input_path - Path to input Markdown file
 *   output_path - Path to output RTF file
 * 
 * Returns:
 *   0 on success, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_convert_md_file_to_rtf(
    const char* input_path,
    const char* output_path
);

/*
 * Validate RTF document
 * 
 * Parameters:
 *   rtf_content - RTF content to validate
 *   output_buffer - Buffer to store validation result
 *   output_length - Length of validation result
 * 
 * Returns:
 *   0 on success, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_validate_rtf_document(
    const char* rtf_content,
    char** output_buffer,
    int* output_length
);

/*
 * Validate Markdown document
 * 
 * Parameters:
 *   markdown_content - Markdown content to validate
 *   output_buffer - Buffer to store validation result
 *   output_length - Length of validation result
 * 
 * Returns:
 *   0 on success, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_validate_markdown_document(
    const char* markdown_content,
    char** output_buffer,
    int* output_length
);

/*
 * Extract plain text from RTF
 * 
 * Parameters:
 *   rtf_content - RTF content
 *   output_buffer - Buffer to store plain text
 *   output_length - Length of plain text
 * 
 * Returns:
 *   0 on success, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_extract_plain_text(
    const char* rtf_content,
    char** output_buffer,
    int* output_length
);

/*
 * Convert folder of RTF files to Markdown
 * 
 * Parameters:
 *   input_folder - Path to folder containing RTF files
 *   output_folder - Path to output folder for Markdown files
 * 
 * Returns:
 *   Number of files processed, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_convert_folder_rtf_to_md(
    const char* input_folder,
    const char* output_folder
);

/*
 * Convert folder of Markdown files to RTF
 * 
 * Parameters:
 *   input_folder - Path to folder containing Markdown files
 *   output_folder - Path to output folder for RTF files
 * 
 * Returns:
 *   Number of files processed, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_convert_folder_md_to_rtf(
    const char* input_folder,
    const char* output_folder
);

/*
 * Get batch operation progress
 * 
 * Returns:
 *   Number of files processed so far
 */
LEGACYBRIDGE_API int legacybridge_get_batch_progress(void);

/*
 * Cancel batch operation
 * 
 * Returns:
 *   0 on success
 */
LEGACYBRIDGE_API int legacybridge_cancel_batch_operation(void);

/*
 * Clean RTF formatting
 * 
 * Parameters:
 *   rtf_content - RTF content to clean
 *   output_buffer - Buffer to store cleaned RTF
 *   output_length - Length of cleaned RTF
 * 
 * Returns:
 *   0 on success, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_clean_rtf_formatting(
    const char* rtf_content,
    char** output_buffer,
    int* output_length
);

/*
 * Normalize Markdown formatting
 * 
 * Parameters:
 *   markdown_content - Markdown content to normalize
 *   output_buffer - Buffer to store normalized Markdown
 *   output_length - Length of normalized Markdown
 * 
 * Returns:
 *   0 on success, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_normalize_markdown(
    const char* markdown_content,
    char** output_buffer,
    int* output_length
);

/*
 * Apply RTF template
 * 
 * Parameters:
 *   rtf_content - RTF content
 *   template_name - Name of template to apply
 *   output_buffer - Buffer to store result
 *   output_length - Length of result
 * 
 * Returns:
 *   0 on success, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_apply_rtf_template(
    const char* rtf_content,
    const char* template_name,
    char** output_buffer,
    int* output_length
);

/*
 * Create RTF template
 * 
 * Parameters:
 *   template_name - Name for the template
 *   rtf_content - RTF content to use as template
 * 
 * Returns:
 *   0 on success, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_create_rtf_template(
    const char* template_name,
    const char* rtf_content
);

/*
 * List available templates
 * 
 * Parameters:
 *   output_buffer - Buffer to store template list
 *   output_length - Length of template list
 * 
 * Returns:
 *   0 on success, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_list_available_templates(
    char** output_buffer,
    int* output_length
);

/*
 * Apply Markdown template
 * 
 * Parameters:
 *   markdown_content - Markdown content
 *   template_name - Name of template to apply
 *   output_buffer - Buffer to store result
 *   output_length - Length of result
 * 
 * Returns:
 *   0 on success, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_apply_markdown_template(
    const char* markdown_content,
    const char* template_name,
    char** output_buffer,
    int* output_length
);

/*
 * Validate template
 * 
 * Parameters:
 *   template_name - Name of template to validate
 * 
 * Returns:
 *   0 if valid, negative error code if invalid
 */
LEGACYBRIDGE_API int legacybridge_validate_template(
    const char* template_name
);

/*
 * Export RTF to CSV
 * 
 * Parameters:
 *   rtf_content - RTF content containing tables
 *   output_buffer - Buffer to store CSV data
 *   output_length - Length of CSV data
 * 
 * Returns:
 *   0 on success, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_export_to_csv(
    const char* rtf_content,
    char** output_buffer,
    int* output_length
);

/*
 * Import CSV to RTF
 * 
 * Parameters:
 *   csv_content - CSV content
 *   output_buffer - Buffer to store RTF data
 *   output_length - Length of RTF data
 * 
 * Returns:
 *   0 on success, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_import_from_csv(
    const char* csv_content,
    char** output_buffer,
    int* output_length
);

/*
 * Convert table data to RTF
 * 
 * Parameters:
 *   table_data - Table data (JSON format)
 *   output_buffer - Buffer to store RTF table
 *   output_length - Length of RTF table
 * 
 * Returns:
 *   0 on success, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_convert_table_to_rtf(
    const char* table_data,
    char** output_buffer,
    int* output_length
);

/*
 * Extract tables from RTF
 * 
 * Parameters:
 *   rtf_content - RTF content
 *   output_buffer - Buffer to store extracted tables (JSON format)
 *   output_length - Length of extracted data
 * 
 * Returns:
 *   0 on success, negative error code on failure
 */
LEGACYBRIDGE_API int legacybridge_extract_tables_from_rtf(
    const char* rtf_content,
    char** output_buffer,
    int* output_length
);

#ifdef __cplusplus
}
#endif

#endif /* LEGACYBRIDGE_H */