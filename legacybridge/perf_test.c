#include <stdio.h>
#include <stdlib.h>
#include <dlfcn.h>
#include <string.h>
#include <time.h>

typedef int (*markdown_to_rtf_fn)(const char*, char**, int*);
typedef int (*rtf_to_markdown_fn)(const char*, char**, int*);
typedef void (*free_string_fn)(char*);

// Test data
const char* test_markdown = "# Performance Test Document\n\n"
    "This is a **comprehensive** test document designed to measure conversion performance.\n\n"
    "## Features\n\n"
    "- **Bold text**\n"
    "- *Italic text*\n"
    "- `Code snippets`\n"
    "- [Links](https://example.com)\n\n"
    "### Tables\n\n"
    "| Column 1 | Column 2 | Column 3 |\n"
    "|----------|----------|----------|\n"
    "| Data 1   | Data 2   | Data 3   |\n"
    "| Data 4   | Data 5   | Data 6   |\n\n"
    "#### Code Block\n\n"
    "```rust\n"
    "fn main() {\n"
    "    println!(\"Hello, world!\");\n"
    "}\n"
    "```\n\n"
    "> This is a blockquote with multiple lines\n"
    "> that should be properly converted to RTF format.\n";

const char* test_rtf = "{\\rtf1\\ansi\\deff0{\\fonttbl{\\f0\\froman\\fcharset0 Times New Roman;}"
    "{\\f1\\fswiss\\fcharset0 Arial;}}{\\colortbl;\\red0\\green0\\blue0;\\red0\\green0\\blue255;}"
    "\\viewkind4\\uc1\\pard\\f0\\fs24 {\\b\\fs32 Test RTF Document\\par}\\par "
    "This is a test document with {\\b bold} and {\\i italic} text.\\par\\par "
    "{\\f1\\fs20 • Item 1\\par • Item 2\\par • Item 3\\par}\\par "
    "This document tests various RTF features.\\par}";

double get_time_ms() {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000.0 + ts.tv_nsec / 1000000.0;
}

int main() {
    void* handle = dlopen("./lib/liblegacybridge.so", RTLD_LAZY);
    if (!handle) {
        fprintf(stderr, "Cannot open library: %s\n", dlerror());
        return 1;
    }
    
    // Load functions
    markdown_to_rtf_fn markdown_to_rtf = (markdown_to_rtf_fn)dlsym(handle, "legacybridge_markdown_to_rtf");
    rtf_to_markdown_fn rtf_to_markdown = (rtf_to_markdown_fn)dlsym(handle, "legacybridge_rtf_to_markdown");
    free_string_fn free_string = (free_string_fn)dlsym(handle, "legacybridge_free_string");
    
    if (!markdown_to_rtf || !rtf_to_markdown || !free_string) {
        fprintf(stderr, "Cannot load required functions\n");
        dlclose(handle);
        return 1;
    }
    
    printf("=== LegacyBridge Performance Test ===\n\n");
    
    // Warm-up runs
    printf("Warming up...\n");
    for (int i = 0; i < 10; i++) {
        char* output = NULL;
        int length = 0;
        markdown_to_rtf(test_markdown, &output, &length);
        if (output) free_string(output);
        rtf_to_markdown(test_rtf, &output, &length);
        if (output) free_string(output);
    }
    
    // Markdown to RTF performance test
    printf("\nMarkdown to RTF Conversion:\n");
    double total_time = 0;
    int iterations = 1000;
    
    for (int i = 0; i < iterations; i++) {
        char* output = NULL;
        int length = 0;
        
        double start = get_time_ms();
        int result = markdown_to_rtf(test_markdown, &output, &length);
        double end = get_time_ms();
        
        if (result == 0 && output) {
            total_time += (end - start);
            if (i == 0) {
                printf("  Output size: %d bytes\n", length);
            }
            free_string(output);
        }
    }
    
    printf("  Average time: %.3f ms\n", total_time / iterations);
    printf("  Total time for %d iterations: %.3f ms\n", iterations, total_time);
    printf("  Throughput: %.0f conversions/second\n", iterations / (total_time / 1000.0));
    
    // RTF to Markdown performance test
    printf("\nRTF to Markdown Conversion:\n");
    total_time = 0;
    
    for (int i = 0; i < iterations; i++) {
        char* output = NULL;
        int length = 0;
        
        double start = get_time_ms();
        int result = rtf_to_markdown(test_rtf, &output, &length);
        double end = get_time_ms();
        
        if (result == 0 && output) {
            total_time += (end - start);
            if (i == 0) {
                printf("  Output size: %d bytes\n", length);
            }
            free_string(output);
        }
    }
    
    printf("  Average time: %.3f ms\n", total_time / iterations);
    printf("  Total time for %d iterations: %.3f ms\n", iterations, total_time);
    printf("  Throughput: %.0f conversions/second\n", iterations / (total_time / 1000.0));
    
    // Memory test
    printf("\nMemory allocation test (100 concurrent conversions):\n");
    char* outputs[100];
    int lengths[100];
    
    double start = get_time_ms();
    for (int i = 0; i < 100; i++) {
        markdown_to_rtf(test_markdown, &outputs[i], &lengths[i]);
    }
    double end = get_time_ms();
    
    printf("  Time to allocate 100 conversions: %.3f ms\n", end - start);
    
    // Free all
    for (int i = 0; i < 100; i++) {
        if (outputs[i]) free_string(outputs[i]);
    }
    
    dlclose(handle);
    
    printf("\n=== Test Complete ===\n");
    return 0;
}