#!/bin/bash
# Cross-platform build script for LegacyBridge DLL
# Builds for multiple architectures including critical 32-bit support

echo "==============================================="
echo "LegacyBridge Cross-Platform Build Script"
echo "Building for multiple architectures..."
echo "==============================================="

# Navigate to the dll-build directory
cd dll-build

# Function to check if a target is installed
check_target() {
    if rustup target list | grep -q "$1 (installed)"; then
        echo "✓ Target $1 is already installed"
        return 0
    else
        echo "✗ Target $1 is not installed"
        return 1
    fi
}

# Function to build for a specific target
build_target() {
    local target=$1
    local features=$2
    
    echo ""
    echo "Building for target: $target"
    echo "----------------------------------------"
    
    if cargo build --release --target $target $features; then
        echo "✓ Successfully built for $target"
        return 0
    else
        echo "✗ Failed to build for $target"
        return 1
    fi
}

# Function to verify DLL exports
verify_dll_exports() {
    local dll_path=$1
    local arch=$2
    
    echo ""
    echo "Verifying DLL exports for $arch..."
    
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # On Linux, use nm or objdump if available
        if command -v nm &> /dev/null; then
            echo "Exported functions:"
            nm -D "$dll_path" 2>/dev/null | grep " T " | grep "legacybridge_" | head -10
        elif command -v objdump &> /dev/null; then
            echo "Exported functions:"
            objdump -T "$dll_path" 2>/dev/null | grep "legacybridge_" | head -10
        else
            echo "Warning: nm and objdump not found, cannot verify exports"
        fi
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        # On Windows with MSYS/Cygwin
        if command -v dumpbin &> /dev/null; then
            dumpbin //EXPORTS "$dll_path" | grep "legacybridge_" | head -10
        else
            echo "Warning: dumpbin not found, cannot verify exports"
        fi
    fi
}

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean

# Install required targets
echo ""
echo "Installing required targets..."
echo "=============================="

# Critical 32-bit targets for legacy compatibility
if ! check_target "i686-pc-windows-msvc"; then
    echo "Installing i686-pc-windows-msvc (32-bit Windows)..."
    rustup target add i686-pc-windows-msvc
fi

if ! check_target "i686-unknown-linux-gnu"; then
    echo "Installing i686-unknown-linux-gnu (32-bit Linux)..."
    rustup target add i686-unknown-linux-gnu
fi

# 64-bit targets
if ! check_target "x86_64-pc-windows-msvc"; then
    echo "Installing x86_64-pc-windows-msvc (64-bit Windows)..."
    rustup target add x86_64-pc-windows-msvc
fi

# Build for all targets
echo ""
echo "Starting builds..."
echo "=================="

# Create output directories
mkdir -p ../lib/windows/x86
mkdir -p ../lib/windows/x64
mkdir -p ../lib/linux/x86
mkdir -p ../lib/linux/x64

# Build 32-bit Windows DLL (Critical for VB6/VFP9)
echo ""
echo "=== CRITICAL BUILD: 32-bit Windows DLL ==="
if build_target "i686-pc-windows-msvc" ""; then
    if [ -f "target/i686-pc-windows-msvc/release/legacybridge.dll" ]; then
        cp target/i686-pc-windows-msvc/release/legacybridge.dll ../lib/windows/x86/
        cp target/i686-pc-windows-msvc/release/legacybridge.dll.lib ../lib/windows/x86/ 2>/dev/null || true
        echo "✓ 32-bit Windows DLL copied to: ../lib/windows/x86/"
        verify_dll_exports "target/i686-pc-windows-msvc/release/legacybridge.dll" "32-bit Windows"
    fi
fi

# Build 64-bit Windows DLL
echo ""
echo "=== Building 64-bit Windows DLL ==="
if build_target "x86_64-pc-windows-msvc" "--features dll-export"; then
    if [ -f "target/x86_64-pc-windows-msvc/release/legacybridge.dll" ]; then
        cp target/x86_64-pc-windows-msvc/release/legacybridge.dll ../lib/windows/x64/
        cp target/x86_64-pc-windows-msvc/release/legacybridge.dll.lib ../lib/windows/x64/ 2>/dev/null || true
        echo "✓ 64-bit Windows DLL copied to: ../lib/windows/x64/"
        verify_dll_exports "target/x86_64-pc-windows-msvc/release/legacybridge.dll" "64-bit Windows"
    fi
fi

# Build 32-bit Linux shared library
echo ""
echo "=== Building 32-bit Linux shared library ==="
if build_target "i686-unknown-linux-gnu" "--features dll-export"; then
    if [ -f "target/i686-unknown-linux-gnu/release/liblegacybridge.so" ]; then
        cp target/i686-unknown-linux-gnu/release/liblegacybridge.so ../lib/linux/x86/
        echo "✓ 32-bit Linux library copied to: ../lib/linux/x86/"
        verify_dll_exports "target/i686-unknown-linux-gnu/release/liblegacybridge.so" "32-bit Linux"
    fi
fi

# Build for current platform (64-bit Linux)
echo ""
echo "=== Building for current platform ==="
if cargo build --release; then
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if [ -f "target/release/liblegacybridge.so" ]; then
            cp target/release/liblegacybridge.so ../lib/linux/x64/
            echo "✓ 64-bit Linux library copied to: ../lib/linux/x64/"
            verify_dll_exports "target/release/liblegacybridge.so" "64-bit Linux"
        fi
    fi
fi

# Generate DEF files for Windows builds
echo ""
echo "Generating DEF files for Windows compatibility..."
cat > ../lib/windows/legacybridge.def << 'EOF'
LIBRARY legacybridge
EXPORTS
    ; Core conversion functions
    legacybridge_rtf_to_markdown
    legacybridge_markdown_to_rtf
    legacybridge_free_string
    legacybridge_get_last_error
    legacybridge_get_version
    legacybridge_get_version_info
    legacybridge_test_connection
    
    ; Batch operations
    legacybridge_batch_rtf_to_markdown
    legacybridge_batch_markdown_to_rtf
    legacybridge_get_batch_progress
    legacybridge_cancel_batch_operation
    
    ; File operations
    legacybridge_convert_rtf_file_to_md
    legacybridge_convert_md_file_to_rtf
    legacybridge_convert_folder_rtf_to_md
    legacybridge_convert_folder_md_to_rtf
    
    ; Validation functions
    legacybridge_validate_rtf_document
    legacybridge_validate_markdown_document
    
    ; Utility functions
    legacybridge_extract_plain_text
    legacybridge_clean_rtf_formatting
    legacybridge_normalize_markdown
    
    ; Template functions (stubs to be implemented)
    legacybridge_apply_rtf_template
    legacybridge_create_rtf_template
    legacybridge_list_available_templates
    legacybridge_apply_markdown_template
    legacybridge_validate_template
    
    ; CSV/Table functions (stubs to be implemented)
    legacybridge_export_to_csv
    legacybridge_import_from_csv
    legacybridge_convert_table_to_rtf
    legacybridge_extract_tables_from_rtf
EOF

# Copy DEF file to both x86 and x64 directories
cp ../lib/windows/legacybridge.def ../lib/windows/x86/
cp ../lib/windows/legacybridge.def ../lib/windows/x64/

# Create a build summary
echo ""
echo "==============================================="
echo "Build Summary"
echo "==============================================="
echo ""
echo "Build outputs:"
echo "--------------"

if [ -f "../lib/windows/x86/legacybridge.dll" ]; then
    echo "✓ 32-bit Windows DLL: ../lib/windows/x86/legacybridge.dll"
    ls -lh ../lib/windows/x86/legacybridge.dll 2>/dev/null
else
    echo "✗ 32-bit Windows DLL: NOT BUILT (Critical for VB6/VFP9!)"
fi

if [ -f "../lib/windows/x64/legacybridge.dll" ]; then
    echo "✓ 64-bit Windows DLL: ../lib/windows/x64/legacybridge.dll"
    ls -lh ../lib/windows/x64/legacybridge.dll 2>/dev/null
else
    echo "✗ 64-bit Windows DLL: NOT BUILT"
fi

if [ -f "../lib/linux/x86/liblegacybridge.so" ]; then
    echo "✓ 32-bit Linux SO: ../lib/linux/x86/liblegacybridge.so"
    ls -lh ../lib/linux/x86/liblegacybridge.so 2>/dev/null
else
    echo "✗ 32-bit Linux SO: NOT BUILT"
fi

if [ -f "../lib/linux/x64/liblegacybridge.so" ]; then
    echo "✓ 64-bit Linux SO: ../lib/linux/x64/liblegacybridge.so"
    ls -lh ../lib/linux/x64/liblegacybridge.so 2>/dev/null
else
    echo "✗ 64-bit Linux SO: NOT BUILT"
fi

echo ""
echo "Header file: ../include/legacybridge.h"
echo "DEF files: ../lib/windows/x86/legacybridge.def, ../lib/windows/x64/legacybridge.def"
echo ""

# Test script for 32-bit compatibility
cat > ../test_32bit_compatibility.sh << 'EOF'
#!/bin/bash
# Test script for verifying 32-bit compatibility

echo "Testing 32-bit compatibility..."

# Check if 32-bit DLL exists
if [ -f "lib/windows/x86/legacybridge.dll" ]; then
    echo "✓ 32-bit Windows DLL found"
    
    # If on Windows, try to load it
    if command -v file &> /dev/null; then
        file_info=$(file "lib/windows/x86/legacybridge.dll")
        if [[ $file_info == *"PE32 executable"* ]] && [[ $file_info == *"Intel 80386"* ]]; then
            echo "✓ Confirmed: 32-bit PE executable for Intel 80386"
        else
            echo "✗ Warning: DLL may not be proper 32-bit format"
            echo "  File info: $file_info"
        fi
    fi
else
    echo "✗ 32-bit Windows DLL not found!"
fi

# Check exported functions count
if command -v nm &> /dev/null || command -v objdump &> /dev/null; then
    echo ""
    echo "Checking exported functions..."
    for lib in lib/windows/x86/legacybridge.dll lib/linux/x86/liblegacybridge.so; do
        if [ -f "$lib" ]; then
            echo "Checking $lib..."
            if command -v nm &> /dev/null; then
                count=$(nm -D "$lib" 2>/dev/null | grep " T " | grep -c "legacybridge_" || echo "0")
            else
                count=$(objdump -T "$lib" 2>/dev/null | grep -c "legacybridge_" || echo "0")
            fi
            echo "  Exported functions found: $count (expected: 29)"
        fi
    done
fi
EOF

chmod +x ../test_32bit_compatibility.sh

echo "==============================================="
echo "Build complete!"
echo ""
echo "To test 32-bit compatibility, run:"
echo "  ./test_32bit_compatibility.sh"
echo ""
echo "For VB6/VFP9 integration:"
echo "  Use files from lib/windows/x86/ (32-bit)"
echo "==============================================="