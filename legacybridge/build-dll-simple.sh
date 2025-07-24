#!/bin/bash
# Simple build script for LegacyBridge DLL

echo "Building LegacyBridge DLL..."

cd src-tauri

# Build with minimal features
echo "Building release version..."
cargo build --release --no-default-features --features dll-export

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
    
    echo "Header file available at: ../include/legacybridge.h"
else
    echo "Build failed!"
    exit 1
fi

echo "Build complete!"