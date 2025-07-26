#!/bin/bash
# Unified build script for LegacyBridge
# Handles all build scenarios with proper error handling and dependency management

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_TYPE="${1:-release}"
TARGET_PLATFORM="${2:-native}"
BUILD_MODULE="${3:-all}"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "info") echo -e "${BLUE}[INFO]${NC} $message" ;;
        "success") echo -e "${GREEN}[SUCCESS]${NC} $message" ;;
        "warning") echo -e "${YELLOW}[WARNING]${NC} $message" ;;
        "error") echo -e "${RED}[ERROR]${NC} $message" ;;
    esac
}

# Function to validate environment
validate_environment() {
    print_status "info" "Validating build environment..."
    
    local missing_deps=()
    
    # Check Node.js
    if ! command -v node >/dev/null 2>&1; then
        missing_deps+=("Node.js")
    else
        print_status "success" "Node.js $(node --version) found"
    fi
    
    # Check npm
    if ! command -v npm >/dev/null 2>&1; then
        missing_deps+=("npm")
    else
        print_status "success" "npm $(npm --version) found"
    fi
    
    # Check Rust
    if ! command -v cargo >/dev/null 2>&1; then
        missing_deps+=("Rust/Cargo")
    else
        print_status "success" "Rust $(rustc --version | cut -d' ' -f2) found"
    fi
    
    # Check for cross-compilation tools if needed
    if [[ "$TARGET_PLATFORM" != "native" ]]; then
        if ! command -v rustup >/dev/null 2>&1; then
            missing_deps+=("rustup")
        fi
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_status "error" "Missing required dependencies: ${missing_deps[*]}"
        print_status "info" "Please install the missing dependencies and try again"
        exit 1
    fi
    
    print_status "success" "Environment validation complete"
}

# Function to clean build environment
clean_build() {
    print_status "info" "Cleaning build environment..."
    
    # Clean Node.js artifacts
    if [[ "$BUILD_MODULE" == "all" || "$BUILD_MODULE" == "frontend" ]]; then
        print_status "info" "Removing Node.js artifacts..."
        rm -rf node_modules package-lock.json .next
        rm -rf .jest-cache coverage
        npm cache clean --force 2>/dev/null || true
    fi
    
    # Clean Rust artifacts
    if [[ "$BUILD_MODULE" == "all" || "$BUILD_MODULE" == "backend" || "$BUILD_MODULE" == "dll" ]]; then
        print_status "info" "Removing Rust artifacts..."
        cd "$PROJECT_ROOT/src-tauri"
        cargo clean
        cd "$PROJECT_ROOT"
        
        if [ -d "$PROJECT_ROOT/dll-build" ]; then
            cd "$PROJECT_ROOT/dll-build"
            cargo clean
            cd "$PROJECT_ROOT"
        fi
    fi
    
    # Clean output directories
    rm -rf lib dist target
    
    print_status "success" "Clean complete"
}

# Function to install Node.js dependencies
install_node_dependencies() {
    print_status "info" "Installing Node.js dependencies..."
    
    # First, ensure we have a clean slate
    rm -rf node_modules package-lock.json
    
    # Install dependencies
    if npm ci 2>/dev/null; then
        print_status "success" "Dependencies installed using npm ci"
    else
        print_status "warning" "npm ci failed, falling back to npm install"
        npm install
    fi
    
    # Verify installation
    local unmet_count=$(npm list --depth=0 2>&1 | grep -c "UNMET DEPENDENCY" || true)
    if [ "$unmet_count" -eq 0 ]; then
        print_status "success" "All Node.js dependencies resolved"
    else
        print_status "error" "$unmet_count unmet dependencies found"
        npm list --depth=0
        exit 1
    fi
}

# Function to install Rust dependencies
install_rust_dependencies() {
    print_status "info" "Installing Rust dependencies..."
    
    # Fetch dependencies for main project
    cd "$PROJECT_ROOT/src-tauri"
    cargo fetch
    
    # Fetch dependencies for DLL build if it exists
    if [ -d "$PROJECT_ROOT/dll-build" ]; then
        cd "$PROJECT_ROOT/dll-build"
        cargo fetch
    fi
    
    cd "$PROJECT_ROOT"
    print_status "success" "Rust dependencies fetched"
}

# Function to build frontend
build_frontend() {
    print_status "info" "Building frontend..."
    
    npm run build
    
    if [ -d ".next" ]; then
        print_status "success" "Frontend build complete"
    else
        print_status "error" "Frontend build failed"
        exit 1
    fi
}

# Function to build backend
build_backend() {
    print_status "info" "Building backend..."
    
    cd "$PROJECT_ROOT/src-tauri"
    
    if [[ "$BUILD_TYPE" == "release" ]]; then
        cargo build --release
    else
        cargo build
    fi
    
    cd "$PROJECT_ROOT"
    print_status "success" "Backend build complete"
}

# Function to build DLL
build_dll() {
    print_status "info" "Building DLL for platform: $TARGET_PLATFORM..."
    
    local build_dir="$PROJECT_ROOT/src-tauri"
    if [ -d "$PROJECT_ROOT/dll-build" ]; then
        build_dir="$PROJECT_ROOT/dll-build"
    fi
    
    cd "$build_dir"
    
    # Create output directories
    mkdir -p "$PROJECT_ROOT/lib/windows/x86"
    mkdir -p "$PROJECT_ROOT/lib/windows/x64"
    mkdir -p "$PROJECT_ROOT/lib/linux/x86"
    mkdir -p "$PROJECT_ROOT/lib/linux/x64"
    mkdir -p "$PROJECT_ROOT/lib/darwin"
    
    case "$TARGET_PLATFORM" in
        "native")
            # Build for current platform
            if [[ "$BUILD_TYPE" == "release" ]]; then
                cargo build --release --features dll-export
            else
                cargo build --features dll-export
            fi
            
            # Copy library to appropriate location
            local target_dir="target/$BUILD_TYPE"
            if [[ "$OSTYPE" == "linux-gnu"* ]]; then
                cp "$target_dir/liblegacybridge.so" "$PROJECT_ROOT/lib/linux/x64/"
                print_status "success" "Linux x64 library built"
            elif [[ "$OSTYPE" == "darwin"* ]]; then
                cp "$target_dir/liblegacybridge.dylib" "$PROJECT_ROOT/lib/darwin/"
                print_status "success" "macOS library built"
            elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
                cp "$target_dir/legacybridge.dll" "$PROJECT_ROOT/lib/windows/x64/"
                print_status "success" "Windows x64 DLL built"
            fi
            ;;
            
        "all")
            # Install required targets
            print_status "info" "Installing cross-compilation targets..."
            rustup target add i686-pc-windows-msvc x86_64-pc-windows-msvc
            rustup target add i686-unknown-linux-gnu x86_64-unknown-linux-gnu
            
            # Build for all targets
            local targets=(
                "i686-pc-windows-msvc:windows/x86:legacybridge.dll"
                "x86_64-pc-windows-msvc:windows/x64:legacybridge.dll"
                "i686-unknown-linux-gnu:linux/x86:liblegacybridge.so"
                "x86_64-unknown-linux-gnu:linux/x64:liblegacybridge.so"
            )
            
            for target_spec in "${targets[@]}"; do
                IFS=':' read -r target output_dir lib_name <<< "$target_spec"
                print_status "info" "Building for $target..."
                
                if cargo build --release --target "$target" --features dll-export; then
                    cp "target/$target/release/$lib_name" "$PROJECT_ROOT/lib/$output_dir/" 2>/dev/null || true
                    print_status "success" "Built for $target"
                else
                    print_status "warning" "Failed to build for $target"
                fi
            done
            ;;
            
        *)
            print_status "error" "Unknown target platform: $TARGET_PLATFORM"
            exit 1
            ;;
    esac
    
    cd "$PROJECT_ROOT"
    
    # Generate DEF file for Windows
    generate_def_file
}

# Function to generate Windows DEF file
generate_def_file() {
    print_status "info" "Generating Windows DEF file..."
    
    cat > "$PROJECT_ROOT/lib/windows/legacybridge.def" << 'EOF'
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
    
    ; Template functions
    legacybridge_apply_rtf_template
    legacybridge_create_rtf_template
    legacybridge_list_available_templates
    legacybridge_apply_markdown_template
    legacybridge_validate_template
    
    ; CSV/Table functions
    legacybridge_export_to_csv
    legacybridge_import_from_csv
    legacybridge_convert_table_to_rtf
    legacybridge_extract_tables_from_rtf
EOF
    
    # Copy to subdirectories
    for dir in x86 x64; do
        if [ -d "$PROJECT_ROOT/lib/windows/$dir" ]; then
            cp "$PROJECT_ROOT/lib/windows/legacybridge.def" "$PROJECT_ROOT/lib/windows/$dir/"
        fi
    done
    
    print_status "success" "DEF file generated"
}

# Function to run tests
run_tests() {
    print_status "info" "Running tests..."
    
    # Run Node.js tests
    if [[ "$BUILD_MODULE" == "all" || "$BUILD_MODULE" == "frontend" ]]; then
        print_status "info" "Running frontend tests..."
        npm test -- --passWithNoTests
    fi
    
    # Run Rust tests
    if [[ "$BUILD_MODULE" == "all" || "$BUILD_MODULE" == "backend" ]]; then
        print_status "info" "Running backend tests..."
        cd "$PROJECT_ROOT/src-tauri"
        cargo test
        cd "$PROJECT_ROOT"
    fi
    
    print_status "success" "All tests passed"
}

# Function to validate build
validate_build() {
    print_status "info" "Validating build..."
    
    local validation_passed=true
    
    # Check frontend build
    if [[ "$BUILD_MODULE" == "all" || "$BUILD_MODULE" == "frontend" ]]; then
        if [ ! -d ".next" ]; then
            print_status "error" "Frontend build artifacts not found"
            validation_passed=false
        fi
    fi
    
    # Check backend build
    if [[ "$BUILD_MODULE" == "all" || "$BUILD_MODULE" == "backend" ]]; then
        if [ ! -f "src-tauri/target/$BUILD_TYPE/legacybridge" ] && [ ! -f "src-tauri/target/$BUILD_TYPE/legacybridge.exe" ]; then
            print_status "warning" "Backend executable not found (may be normal for library builds)"
        fi
    fi
    
    # Check DLL build
    if [[ "$BUILD_MODULE" == "dll" ]]; then
        local found_libs=0
        for lib in lib/**/*.{dll,so,dylib}; do
            if [ -f "$lib" ]; then
                ((found_libs++))
                print_status "success" "Found library: $lib"
            fi
        done
        
        if [ $found_libs -eq 0 ]; then
            print_status "error" "No library files found"
            validation_passed=false
        fi
    fi
    
    if [ "$validation_passed" = true ]; then
        print_status "success" "Build validation passed"
    else
        print_status "error" "Build validation failed"
        exit 1
    fi
}

# Function to print build summary
print_summary() {
    echo ""
    echo "==============================================="
    echo "Build Summary"
    echo "==============================================="
    echo "Build Type: $BUILD_TYPE"
    echo "Target Platform: $TARGET_PLATFORM"
    echo "Build Module: $BUILD_MODULE"
    echo ""
    
    if [[ "$BUILD_MODULE" == "all" || "$BUILD_MODULE" == "frontend" ]]; then
        echo "Frontend:"
        echo "  - Next.js build: .next/"
        echo ""
    fi
    
    if [[ "$BUILD_MODULE" == "all" || "$BUILD_MODULE" == "backend" ]]; then
        echo "Backend:"
        echo "  - Tauri app: src-tauri/target/$BUILD_TYPE/"
        echo ""
    fi
    
    if [[ "$BUILD_MODULE" == "dll" || "$BUILD_MODULE" == "all" ]]; then
        echo "Libraries:"
        for lib in lib/**/*.{dll,so,dylib}; do
            if [ -f "$lib" ]; then
                echo "  - $lib"
            fi
        done
        echo ""
    fi
    
    echo "==============================================="
}

# Main execution
main() {
    print_status "info" "Starting LegacyBridge unified build..."
    echo "Build configuration: TYPE=$BUILD_TYPE, PLATFORM=$TARGET_PLATFORM, MODULE=$BUILD_MODULE"
    echo ""
    
    # Validate environment
    validate_environment
    
    # Clean if requested
    if [[ "${4:-}" == "clean" ]]; then
        clean_build
    fi
    
    # Install dependencies
    if [[ "$BUILD_MODULE" == "all" || "$BUILD_MODULE" == "frontend" ]]; then
        install_node_dependencies
    fi
    
    if [[ "$BUILD_MODULE" == "all" || "$BUILD_MODULE" == "backend" || "$BUILD_MODULE" == "dll" ]]; then
        install_rust_dependencies
    fi
    
    # Build components
    case "$BUILD_MODULE" in
        "frontend")
            build_frontend
            ;;
        "backend")
            build_backend
            ;;
        "dll")
            build_dll
            ;;
        "all")
            build_frontend
            build_backend
            if [[ "$TARGET_PLATFORM" != "native" ]] || [[ "${INCLUDE_DLL:-}" == "true" ]]; then
                build_dll
            fi
            ;;
        *)
            print_status "error" "Unknown build module: $BUILD_MODULE"
            echo "Valid modules: all, frontend, backend, dll"
            exit 1
            ;;
    esac
    
    # Run tests if not explicitly skipped
    if [[ "${SKIP_TESTS:-}" != "true" ]]; then
        run_tests
    fi
    
    # Validate build
    validate_build
    
    # Print summary
    print_summary
    
    print_status "success" "Build complete!"
}

# Run main function
main "$@"