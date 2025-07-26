#include <stdio.h>
#include <stdlib.h>
#include <dlfcn.h>
#include <string.h>

typedef int (*test_connection_fn)();
typedef const char* (*get_version_fn)();
typedef int (*markdown_to_rtf_fn)(const char*, char**, int*);
typedef void (*free_string_fn)(char*);

int main() {
    // Load the library
    void* handle = dlopen("./lib/liblegacybridge.so", RTLD_LAZY);
    if (!handle) {
        fprintf(stderr, "Cannot open library: %s\n", dlerror());
        return 1;
    }
    
    // Test connection
    test_connection_fn test_connection = (test_connection_fn)dlsym(handle, "legacybridge_test_connection");
    if (!test_connection) {
        fprintf(stderr, "Cannot load symbol 'legacybridge_test_connection': %s\n", dlerror());
        dlclose(handle);
        return 1;
    }
    
    int result = test_connection();
    printf("Test connection result: %d\n", result);
    
    // Get version
    get_version_fn get_version = (get_version_fn)dlsym(handle, "legacybridge_get_version");
    if (!get_version) {
        fprintf(stderr, "Cannot load symbol 'legacybridge_get_version': %s\n", dlerror());
        dlclose(handle);
        return 1;
    }
    
    const char* version = get_version();
    printf("Library version: %s\n", version);
    
    // Test markdown to RTF conversion
    markdown_to_rtf_fn markdown_to_rtf = (markdown_to_rtf_fn)dlsym(handle, "legacybridge_markdown_to_rtf");
    free_string_fn free_string = (free_string_fn)dlsym(handle, "legacybridge_free_string");
    
    if (markdown_to_rtf && free_string) {
        const char* test_markdown = "# Hello World\nThis is a **test** document.";
        char* output_buffer = NULL;
        int output_length = 0;
        
        int conv_result = markdown_to_rtf(test_markdown, &output_buffer, &output_length);
        if (conv_result == 0 && output_buffer) {
            printf("\nMarkdown to RTF conversion successful!\n");
            printf("Output length: %d\n", output_length);
            printf("First 100 chars: %.100s\n", output_buffer);
            
            // Free the allocated string
            free_string(output_buffer);
        } else {
            printf("Conversion failed with code: %d\n", conv_result);
        }
    }
    
    // Close the library
    dlclose(handle);
    
    return 0;
}