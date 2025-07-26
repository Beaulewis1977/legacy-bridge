# 🧪 LegacyBridge Comprehensive Testing Guide

## 🎯 Overview

This guide covers testing the complete LegacyBridge application across all platforms:
- **Web Application** (Next.js frontend)
- **Desktop Application** (Tauri)
- **CLI Tool** (Rust binary)
- **DLL Library** (Windows integration)

## 🐛 Bug Fix Applied

**Fixed TypeError**: `Cannot read properties of undefined (reading 'name')`
- **Location**: `ConversionProgress.tsx` line 130
- **Solution**: Added null safety checks for `file.name` and `file.size`
- **Root Cause**: File objects missing expected properties during conversion

## 📁 File Output Location Strategy

### Current Behavior
- **Default**: Files are saved in the same directory as the source file
- **Naming**: `original-name.converted-extension`
- **Example**: `document.rtf` → `document.md` (same folder)

### Planned Improvements
1. **User-selectable output directory**
2. **Batch output folder option**
3. **Preserve/modify filename patterns**
4. **Conflict resolution (overwrite/rename)**

## 🧪 Testing Strategy

### 1. Web Application Testing (Next.js)

#### A. Development Server Testing
```bash
# Start development server
npm run dev:frontend

# Test in browser
# http://localhost:3000
```

#### B. Production Build Testing
```bash
# Build for production
npm run build:frontend

# Start production server
npm run start

# Test production build
# http://localhost:3000
```

#### C. Playwright E2E Testing
```bash
# Install Playwright browsers
npx playwright install

# Run all E2E tests
npm run test:e2e

# Run specific test suites
npm run test:e2e -- --grep "conversion"
npm run test:e2e -- --grep "file-upload"
npm run test:e2e -- --grep "download"
```

### 2. Desktop Application Testing (Tauri)

#### A. Development Mode
```bash
# Start Tauri development
npm run tauri:dev

# This opens the desktop app window
# Test all desktop-specific features
```

#### B. Production Build
```bash
# Build desktop application
npm run tauri:build

# Find built app in:
# src-tauri/target/release/bundle/
# - Windows: .msi installer
# - macOS: .app bundle
# - Linux: .deb/.rpm packages
```

#### C. Desktop-Specific Testing
- **File system access**
- **Native file dialogs**
- **System tray integration**
- **Auto-updater functionality**
- **Performance with large files**

### 3. CLI Tool Testing

#### A. Build CLI Binary
```bash
# Build Rust CLI
cd dll-build
cargo build --release --bin legacybridge-cli

# Binary location:
# target/release/legacybridge-cli.exe (Windows)
# target/release/legacybridge-cli (Unix)
```

#### B. CLI Testing Commands
```bash
# Test RTF to Markdown conversion
./target/release/legacybridge-cli convert --input document.rtf --output document.md

# Test Markdown to RTF conversion
./target/release/legacybridge-cli convert --input document.md --output document.rtf

# Batch conversion
./target/release/legacybridge-cli batch --input-dir ./rtf-files --output-dir ./md-files --format md

# Test with various file sizes
./target/release/legacybridge-cli convert --input large-document.rtf --output large-document.md

# Performance testing
time ./target/release/legacybridge-cli convert --input test.rtf --output test.md
```

### 4. DLL Library Testing

#### A. Build DLL
```bash
# Build the DLL
npm run build:dll

# Output: lib/legacybridge.dll
```

#### B. VB6 Integration Testing
```vb
' VB6 test code (save as test-dll.vb6)
Private Declare Function ConvertRtfToMarkdown Lib "legacybridge.dll" _
    (ByVal rtfPath As String, ByVal outputPath As String) As Long

Private Sub TestConversion()
    Dim result As Long
    result = ConvertRtfToMarkdown("C:\test\document.rtf", "C:\test\document.md")
    
    If result = 0 Then
        MsgBox "Conversion successful!"
    Else
        MsgBox "Conversion failed with code: " & result
    End If
End Sub
```

#### C. VFP9 Integration Testing
```foxpro
* VFP9 test code (save as test-dll.prg)
DECLARE INTEGER ConvertRtfToMarkdown IN legacybridge.dll ;
    STRING rtfPath, STRING outputPath

LOCAL result
result = ConvertRtfToMarkdown("C:\test\document.rtf", "C:\test\document.md")

IF result = 0
    MESSAGEBOX("Conversion successful!")
ELSE
    MESSAGEBOX("Conversion failed with code: " + TRANSFORM(result))
ENDIF
```

## 🔧 Automated Testing Suite

### 1. Unit Tests
```bash
# Run React component tests
npm run test:unit

# Run Rust unit tests
cd dll-build && cargo test
```

### 2. Integration Tests
```bash
# Test complete workflows
npm run test:integration

# Test API endpoints
npm run test:integration:api

# Test system integration
npm run test:system
```

### 3. Performance Tests
```bash
# Rust performance benchmarks
npm run test:performance

# Memory usage testing
npm run test:performance:memory

# Regression testing
npm run test:performance:regression
```

### 4. Load Testing
```bash
# Install k6 (if not installed)
# Windows: choco install k6
# macOS: brew install k6
# Linux: sudo apt install k6

# Run load tests
npm run test:load
```

### 5. Accessibility Testing
```bash
# Run accessibility tests
npm run test:a11y

# Keyboard navigation testing
npm run test:a11y:keyboard

# Screen reader testing
npm run test:a11y:screen-reader
```

### 6. Visual Regression Testing
```bash
# Component visual testing
npm run test:visual:components

# Responsive design testing
npm run test:visual:responsive

# Full visual regression suite
npm run test:visual
```

### 7. Chaos Testing
```bash
# Test error recovery
npm run test:chaos:recovery

# Test system resilience
npm run test:chaos:resilience

# Full chaos testing suite
npm run test:chaos
```

## 📋 Manual Testing Checklist

### Web Application
- [ ] File upload (drag & drop)
- [ ] File upload (click to select)
- [ ] RTF to Markdown conversion
- [ ] Markdown to RTF conversion
- [ ] Preview functionality
- [ ] Download converted files
- [ ] Error handling
- [ ] Progress tracking
- [ ] Batch conversion
- [ ] Mobile responsiveness

### Desktop Application
- [ ] Native file dialogs
- [ ] System file associations
- [ ] Drag & drop from file explorer
- [ ] Menu bar functionality
- [ ] Keyboard shortcuts
- [ ] Window management
- [ ] System tray integration
- [ ] Auto-updater
- [ ] Performance with large files
- [ ] Cross-platform compatibility

### CLI Tool
- [ ] Single file conversion
- [ ] Batch conversion
- [ ] Command-line arguments
- [ ] Error messages
- [ ] Help documentation
- [ ] Performance benchmarks
- [ ] Memory usage
- [ ] Large file handling
- [ ] Cross-platform execution

### DLL Library
- [ ] VB6 integration
- [ ] VFP9 integration
- [ ] C/C++ integration
- [ ] Memory management
- [ ] Error codes
- [ ] Thread safety
- [ ] Performance
- [ ] 32-bit compatibility
- [ ] 64-bit compatibility

## 🚀 Continuous Integration Testing

### GitHub Actions Workflow
```yaml
name: Comprehensive Testing
on: [push, pull_request]

jobs:
  test-web:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - run: npm ci
      - run: npm run test:unit
      - run: npm run test:integration
      - run: npm run build:frontend

  test-desktop:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - run: npm ci
      - run: npm run tauri:build

  test-cli:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
      - run: cd dll-build && cargo test
      - run: cd dll-build && cargo build --release

  test-dll:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm run build:dll
      - run: # Test DLL integration
```

## 🔍 Debugging & Troubleshooting

### Common Issues

#### 1. File Conversion Errors
```bash
# Check file permissions
ls -la input-file.rtf

# Verify file format
file input-file.rtf

# Test with minimal file
echo "{\rtf1 Hello World}" > test.rtf
```

#### 2. Memory Issues
```bash
# Monitor memory usage
# Windows: Task Manager
# Linux: htop or top
# macOS: Activity Monitor

# Rust memory profiling
cargo install cargo-profiler
cargo profiler callgrind --bin legacybridge-cli
```

#### 3. Performance Issues
```bash
# Profile Rust code
cargo install cargo-flamegraph
cargo flamegraph --bin legacybridge-cli

# Profile React components
# Use React DevTools Profiler
```

## 📊 Test Coverage Goals

- **Unit Tests**: 95%+ coverage
- **Integration Tests**: 90%+ critical paths
- **E2E Tests**: 100% user workflows
- **Performance Tests**: All conversion scenarios
- **Accessibility**: WCAG 2.1 AA compliance
- **Cross-platform**: Windows, macOS, Linux
- **Browser Support**: Chrome, Firefox, Safari, Edge

## 🎯 Success Criteria

### Functional
- [ ] All conversion formats work correctly
- [ ] File I/O operations are reliable
- [ ] Error handling is comprehensive
- [ ] Performance meets benchmarks

### Non-Functional
- [ ] Application starts in <3 seconds
- [ ] File conversion <2 seconds for typical files
- [ ] Memory usage <100MB for normal operations
- [ ] Zero memory leaks detected
- [ ] 99.9% uptime in production

### User Experience
- [ ] Intuitive interface
- [ ] Clear error messages
- [ ] Responsive design
- [ ] Accessibility compliance
- [ ] Cross-platform consistency

## 🔄 Testing Workflow

### Development Cycle
1. **Write failing test**
2. **Implement feature**
3. **Make test pass**
4. **Refactor code**
5. **Run full test suite**
6. **Manual testing**
7. **Performance validation**
8. **Accessibility check**

### Release Cycle
1. **Full automated test suite**
2. **Manual testing checklist**
3. **Performance benchmarks**
4. **Security audit**
5. **Cross-platform validation**
6. **User acceptance testing**
7. **Production deployment**
8. **Post-deployment monitoring**

---

**Next Steps**: Run the test suite and validate all functionality works as expected across all platforms.