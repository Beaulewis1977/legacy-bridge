// Simple C program to test 32-bit compatibility
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

// Function prototypes from legacybridge.dll
extern int legacybridge_test_connection(void);
extern const char* legacybridge_get_version(void);
extern uint32_t legacybridge_get_architecture_bits(void);
extern uint32_t legacybridge_get_max_string_size(void);
extern int legacybridge_rtf_to_markdown(const char* rtf, char** output, int* length);
extern void legacybridge_free_string(char* str);

int main() {
    printf("=== LegacyBridge 32-bit Compatibility Test ===\n\n");
    
    // Test 1: Connection test
    printf("Test 1: Connection test\n");
    int connected = legacybridge_test_connection();
    printf("Connection status: %s\n\n", connected == 1 ? "SUCCESS" : "FAILED");
    
    // Test 2: Version info
    printf("Test 2: Version information\n");
    const char* version = legacybridge_get_version();
    printf("Version: %s\n\n", version ? version : "Unknown");
    
    // Test 3: Architecture check
    printf("Test 3: Architecture check\n");
    uint32_t arch_bits = legacybridge_get_architecture_bits();
    printf("Architecture: %u-bit\n", arch_bits);
    printf("Compiled as: %lu-bit\n", sizeof(void*) * 8);
    
    if (arch_bits != sizeof(void*) * 8) {
        printf("WARNING: Architecture mismatch!\n");
    }
    printf("\n");
    
    // Test 4: Memory constraints
    printf("Test 4: Memory constraints\n");
    uint32_t max_string = legacybridge_get_max_string_size();
    printf("Max string size: %u bytes (%.2f MB)\n\n", max_string, max_string / 1048576.0);
    
    // Test 5: Simple conversion
    printf("Test 5: Simple RTF conversion\n");
    const char* test_rtf = "{\\rtf1\\ansi\\deff0 {\\fonttbl{\\f0 Times New Roman;}}Hello 32-bit World!\\par}";
    char* output = NULL;
    int length = 0;
    
    int result = legacybridge_rtf_to_markdown(test_rtf, &output, &length);
    
    if (result == 0 && output != NULL) {
        printf("Conversion successful!\n");
        printf("Output: %s\n", output);
        printf("Length: %d\n", length);
        
        // Free the allocated memory
        legacybridge_free_string(output);
    } else {
        printf("Conversion failed with code: %d\n", result);
    }
    
    printf("\n=== Test Complete ===\n");
    
    return 0;
}

// Makefile content for building 32-bit test
/*
Makefile:

# 32-bit compilation flags
CC = gcc
CFLAGS_32 = -m32 -Wall -O2
LDFLAGS_32 = -m32 -L./lib/windows/x86 -llegacybridge

# Targets
test32: test_32bit_compatibility.c
	$(CC) $(CFLAGS_32) -o test32 test_32bit_compatibility.c $(LDFLAGS_32)

clean:
	rm -f test32

.PHONY: clean
*/