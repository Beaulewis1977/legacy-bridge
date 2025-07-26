# LegacyBridge Build Instructions

## Overview

LegacyBridge is a Tauri-based application that provides bidirectional conversion between RTF and Markdown formats. The implementation includes a complete MD→RTF conversion pipeline with error recovery, validation, and formatting preservation.

## Prerequisites

### System Dependencies

For Ubuntu/Debian systems:
```bash
sudo apt-get update
sudo apt-get install -y \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    librsvg2-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev
```

### Rust & Cargo

Install Rust via rustup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Node.js & npm

Required for the frontend:
```bash
# Install Node.js 18+ 
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs
```

## Build Instructions

### 1. Clone and Navigate
```bash
cd legacybridge
```

### 2. Install Frontend Dependencies
```bash
npm install
```

### 3. Build Tauri Application

#### Development Build
```bash
npm run tauri dev
```

#### Production Build
```bash
npm run tauri build
```

## Current Implementation Status

### ✅ Completed Features

1. **MD→RTF Conversion Pipeline**
   - Full Markdown parsing with pulldown-cmark
   - RTF document generation with proper formatting
   - Support for:
     - Headers (h1-h6)
     - Bold, italic, underline formatting
     - Lists (ordered and unordered)
     - Tables
     - Code blocks
     - Links and images (placeholders)

2. **Error Recovery System**
   - Automatic recovery from malformed input
   - Multiple recovery strategies (Skip, Replace, Fix, BestEffort)
   - Detailed error reporting and recovery actions

3. **Validation Layer**
   - Pre and post-validation
   - Structure validation
   - Character set validation
   - Nesting depth checks

4. **Template System**
   - Template-based document generation
   - Custom styling support
   - Content transformations

5. **Formatting Engine**
   - High-fidelity formatting preservation
   - Custom RTF properties support
   - Font and color table management

### 🚧 Known Issues

1. **Compilation Warnings**
   - Some unused imports in optimized modules
   - These don't affect core functionality

2. **Test Coverage**
   - Core conversion tests are implemented
   - Integration tests pending due to module dependencies

## Testing the Conversion

### Unit Tests
```bash
cd src-tauri
cargo test --lib conversion::markdown_parser::tests
cargo test --lib conversion::rtf_generator::tests
```

### Manual Testing
Create a test file `test.md`:
```markdown
# Test Document
This is **bold** and *italic* text.

## Lists
- Item 1
- Item 2
  - Nested item

| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
```

Then use the Tauri command interface or API to convert.

## Architecture

The conversion pipeline follows this flow:

```
Markdown Input
    ↓
Markdown Parser (pulldown-cmark)
    ↓
Document Structure (RtfDocument)
    ↓
Validation Layer
    ↓
Template System (optional)
    ↓
RTF Generator
    ↓
Error Recovery (if needed)
    ↓
RTF Output
```

## Troubleshooting

### Missing System Libraries
If you encounter linking errors, ensure all system dependencies are installed:
```bash
pkg-config --libs webkit2gtk-4.1
pkg-config --libs gtk+-3.0
```

### Rust Compilation Errors
Ensure you're using a recent stable Rust version:
```bash
rustc --version  # Should be 1.70+
cargo --version
```

### Frontend Build Issues
Clear npm cache and reinstall:
```bash
npm cache clean --force
rm -rf node_modules package-lock.json
npm install
```

## Contributing

When making changes:
1. Run `cargo fmt` before committing
2. Ensure `cargo clippy` passes
3. Add tests for new functionality
4. Update this documentation as needed

## License

See LICENSE file in the repository root.