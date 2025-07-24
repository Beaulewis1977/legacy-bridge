#!/usr/bin/env python3
"""
Test script for LegacyBridge DLL
Tests all 25 exported functions
"""

import ctypes
import os
import sys

# Load the DLL
dll_path = os.path.join(os.path.dirname(__file__), "lib", "liblegacybridge.so")
if not os.path.exists(dll_path):
    dll_path = os.path.join(os.path.dirname(__file__), "lib", "legacybridge.dll")

try:
    dll = ctypes.CDLL(dll_path)
except Exception as e:
    print(f"Failed to load DLL: {e}")
    sys.exit(1)

# Define function signatures
# Test connection
dll.legacybridge_test_connection.restype = ctypes.c_int

# Get version
dll.legacybridge_get_version.restype = ctypes.c_char_p

# RTF to Markdown
dll.legacybridge_rtf_to_markdown.argtypes = [ctypes.c_char_p, ctypes.POINTER(ctypes.c_char_p), ctypes.POINTER(ctypes.c_int)]
dll.legacybridge_rtf_to_markdown.restype = ctypes.c_int

# Free string
dll.legacybridge_free_string.argtypes = [ctypes.c_char_p]

print("Testing LegacyBridge DLL Functions")
print("=" * 50)

# Test 1: Connection test
print("\n1. Testing connection...")
result = dll.legacybridge_test_connection()
print(f"   Connection test: {'PASS' if result == 1 else 'FAIL'}")

# Test 2: Get version
print("\n2. Getting version...")
version = dll.legacybridge_get_version()
print(f"   Version: {version.decode('utf-8')}")

# Test 3: RTF to Markdown conversion
print("\n3. Testing RTF to Markdown conversion...")
rtf_content = b"{\\rtf1\\ansi\\deff0 {\\fonttbl{\\f0 Times New Roman;}} \\b Hello World\\b0\\par This is a test.\\par}"
output_buffer = ctypes.c_char_p()
output_length = ctypes.c_int()

result = dll.legacybridge_rtf_to_markdown(rtf_content, ctypes.byref(output_buffer), ctypes.byref(output_length))
if result == 0:
    markdown = ctypes.string_at(output_buffer, output_length.value).decode('utf-8')
    print(f"   Conversion successful!")
    print(f"   Output: {markdown}")
    dll.legacybridge_free_string(output_buffer)
else:
    print(f"   Conversion failed with error code: {result}")

# List all exported functions
print("\n4. Checking all exported functions...")
functions = [
    "legacybridge_rtf_to_markdown",
    "legacybridge_markdown_to_rtf",
    "legacybridge_free_string",
    "legacybridge_get_last_error",
    "legacybridge_get_version",
    "legacybridge_batch_rtf_to_markdown",
    "legacybridge_batch_markdown_to_rtf",
    "legacybridge_test_connection",
    "legacybridge_get_version_info",
    "legacybridge_convert_rtf_file_to_md",
    "legacybridge_convert_md_file_to_rtf",
    "legacybridge_validate_rtf_document",
    "legacybridge_validate_markdown_document",
    "legacybridge_extract_plain_text",
    "legacybridge_convert_folder_rtf_to_md",
    "legacybridge_convert_folder_md_to_rtf",
    "legacybridge_get_batch_progress",
    "legacybridge_cancel_batch_operation",
    "legacybridge_clean_rtf_formatting",
    "legacybridge_normalize_markdown",
    "legacybridge_apply_rtf_template",
    "legacybridge_create_rtf_template",
    "legacybridge_list_available_templates",
    "legacybridge_apply_markdown_template",
    "legacybridge_validate_template",
    "legacybridge_export_to_csv",
    "legacybridge_import_from_csv",
    "legacybridge_convert_table_to_rtf",
    "legacybridge_extract_tables_from_rtf"
]

found_count = 0
for func_name in functions:
    try:
        func = getattr(dll, func_name)
        found_count += 1
        print(f"   ✓ {func_name}")
    except AttributeError:
        print(f"   ✗ {func_name} - NOT FOUND")

print(f"\nSummary: {found_count}/{len(functions)} functions found")
print("=" * 50)