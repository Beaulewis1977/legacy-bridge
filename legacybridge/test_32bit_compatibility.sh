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
