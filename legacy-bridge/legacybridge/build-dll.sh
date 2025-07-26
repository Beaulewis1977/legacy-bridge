#!/bin/bash
# Build script for LegacyBridge DLL (Linux/macOS)

echo "Building LegacyBridge DLL..."

# Navigate to the dll-build directory
cd dll-build

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean

# Build the DLL
echo "Building release version..."
cargo build --release

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "Build successful!"
    
    # Copy the library to the output directory
    mkdir -p ../lib
    
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        cp target/release/liblegacybridge.so ../lib/
        echo "Library copied to: ../lib/liblegacybridge.so"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        cp target/release/liblegacybridge.dylib ../lib/
        echo "Library copied to: ../lib/liblegacybridge.dylib"
    fi
    
    # Copy header file
    echo "Header file available at: ../include/legacybridge.h"
else
    echo "Build failed!"
    exit 1
fi

echo "Build complete!"