# LegacyBridge Production Build Guide

## Overview

LegacyBridge is a high-performance RTF ↔ Markdown converter that can be built as:
1. A standalone DLL/shared library for integration with legacy systems (VB6, VFP9)
2. A Tauri desktop application with GUI

## Build Configurations

### DLL Build (Recommended for Production)

The DLL build creates a minimal shared library without GUI dependencies.

**Target Size**: < 5MB (Current: 720KB)
**Performance**: 
- Markdown→RTF: ~41,000 conversions/second
- RTF→Markdown: ~20,500 conversions/second

### System Requirements

#### Linux (Ubuntu 22.04+)
```bash
# Core build tools
sudo apt-get update
sudo apt-get install -y build-essential pkg-config

# For Tauri builds only
sudo apt-get install -y libglib2.0-dev libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev
```

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# For Tauri builds (via Homebrew)
brew install pkg-config
```

#### Windows
- Visual Studio 2019+ with C++ build tools
- Windows SDK

### Building the DLL

#### Quick Build (No Tauri Dependencies)
```bash
cd src-tauri
cargo build --release --no-default-features
```

#### Optimized Build for Production
```bash
cd src-tauri
cargo build --release --no-default-features

# The optimized library will be at:
# Linux: target/release/liblegacybridge.so
# macOS: target/release/liblegacybridge.dylib  
# Windows: target/release/legacybridge.dll
```

#### Size-Optimized Build
```bash
# Use the release-min profile for smallest size
cargo build --profile release-min --no-default-features
```

### Build Features

The project uses Cargo features to control dependencies:

- `default`: Includes Tauri app dependencies
- `tauri-app`: Full Tauri application with GUI
- `dll-export`: Standalone DLL (currently empty, build with --no-default-features)

### Optimization Flags

The release profile includes aggressive optimizations:

```toml
[profile.release]
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit for better optimization
opt-level = 3        # Maximum optimization
strip = true         # Strip debug symbols
panic = "abort"      # Smaller panic handler
overflow-checks = false  # Disable overflow checks
```

### Performance Benchmarks

Run benchmarks with:
```bash
cargo bench
```

### Testing the Build

1. **Basic Test** (C):
```bash
gcc -o test_dll test_dll.c -ldl
./test_dll
```

2. **Performance Test** (C):
```bash
gcc -O3 -o perf_test perf_test.c -ldl
./perf_test
```

3. **Rust Tests**:
```bash
cargo test --release --no-default-features
```

### Cross-Platform Building

#### Linux → Windows Cross-Compilation
```bash
# Install cross-compilation tools
rustup target add x86_64-pc-windows-gnu
sudo apt-get install mingw-w64

# Build
cargo build --release --target x86_64-pc-windows-gnu --no-default-features
```

#### Building Universal Binary (macOS)
```bash
# Add targets
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build for both architectures
cargo build --release --target x86_64-apple-darwin --no-default-features
cargo build --release --target aarch64-apple-darwin --no-default-features

# Create universal binary
lipo -create \
  target/x86_64-apple-darwin/release/liblegacybridge.dylib \
  target/aarch64-apple-darwin/release/liblegacybridge.dylib \
  -output lib/liblegacybridge.dylib
```

### Integration

See the following guides for platform-specific integration:
- [DLL Integration Guide](DLL_INTEGRATION_GUIDE.md) - General DLL usage
- [VB6 Integration](vb6-wrapper/README.md) - Visual Basic 6
- [VFP9 Integration](vfp9-wrapper/README.md) - Visual FoxPro 9

### Troubleshooting

#### Missing Dependencies
If you see errors about missing system libraries:
1. Ensure all system dependencies are installed (see System Requirements)
2. For DLL builds, use `--no-default-features` to avoid GUI dependencies

#### Build Failures
1. Clean the build directory: `cargo clean`
2. Update dependencies: `cargo update`
3. Check Rust version: `rustc --version` (requires 1.70+)

#### Performance Issues
1. Ensure you're building in release mode
2. Check that LTO is enabled in Cargo.toml
3. Consider using the `release-min` profile for embedded systems

### Deployment Checklist

- [ ] Build with `--release` flag
- [ ] Test on target platform
- [ ] Verify library size < 5MB
- [ ] Run performance benchmarks
- [ ] Test with example integration code
- [ ] Include required headers (legacybridge.h)
- [ ] Document any platform-specific requirements

### Build Artifacts

After a successful build, you'll have:
- **Library**: `liblegacybridge.{so,dylib,dll}`
- **Header**: `include/legacybridge.h`
- **Examples**: Integration examples for various platforms
- **Tests**: Test binaries for validation

### Continuous Integration

For CI/CD pipelines, use:
```bash
# Minimal build without user interaction
cargo build --release --no-default-features --locked
```

This ensures reproducible builds with locked dependencies.