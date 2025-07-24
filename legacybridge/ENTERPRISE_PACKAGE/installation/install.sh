#!/bin/bash
# LegacyBridge Enterprise Edition Installation Script for Linux
# Version 1.0.0

set -e

echo
echo "============================================"
echo "LegacyBridge Enterprise Edition Installer"
echo "Version 1.0.0"
echo "============================================"
echo

# Check for root privileges
if [ "$EUID" -ne 0 ]; then 
    echo "ERROR: This installer requires root privileges."
    echo "Please run with sudo."
    exit 1
fi

# Detect system architecture
ARCH=$(uname -m)
if [ "$ARCH" != "x86_64" ]; then
    echo "ERROR: LegacyBridge requires x86_64 architecture."
    echo "Current architecture: $ARCH"
    exit 1
fi

# Set installation paths
INSTALL_DIR="/opt/legacybridge"
LIB_DIR="/usr/local/lib"
INCLUDE_DIR="/usr/local/include"
BIN_DIR="/usr/local/bin"

echo "Installation directories:"
echo "  Main: $INSTALL_DIR"
echo "  Libraries: $LIB_DIR"
echo "  Headers: $INCLUDE_DIR"
echo

# Create installation directories
echo "Creating installation directories..."
mkdir -p "$INSTALL_DIR"/{bin,include,examples,docs,tools}
mkdir -p "$LIB_DIR"
mkdir -p "$INCLUDE_DIR"

# Copy files
echo
echo "Installing LegacyBridge components..."

echo "- Installing shared library..."
cp -f ../bin/liblegacybridge.so "$LIB_DIR/"
chmod 755 "$LIB_DIR/liblegacybridge.so"

echo "- Installing header files..."
cp -f ../include/legacybridge.h "$INCLUDE_DIR/"
chmod 644 "$INCLUDE_DIR/legacybridge.h"

echo "- Installing examples..."
cp -r ../examples/* "$INSTALL_DIR/examples/"
find "$INSTALL_DIR/examples" -type f -exec chmod 644 {} \;
find "$INSTALL_DIR/examples" -type d -exec chmod 755 {} \;

echo "- Installing documentation..."
cp -r ../docs/* "$INSTALL_DIR/docs/" 2>/dev/null || true
find "$INSTALL_DIR/docs" -type f -exec chmod 644 {} \; 2>/dev/null || true
find "$INSTALL_DIR/docs" -type d -exec chmod 755 {} \; 2>/dev/null || true

echo "- Installing tools..."
cp -f ../tools/* "$INSTALL_DIR/tools/" 2>/dev/null || true
chmod 755 "$INSTALL_DIR/tools/"* 2>/dev/null || true

# Update library cache
echo
echo "Updating library cache..."
ldconfig

# Create pkg-config file
echo
echo "Creating pkg-config file..."
cat > /usr/local/lib/pkgconfig/legacybridge.pc << EOF
prefix=/usr/local
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: LegacyBridge
Description: High-Performance RTF ↔ Markdown Converter
Version: 1.0.0
Libs: -L\${libdir} -llegacybridge
Cflags: -I\${includedir}
EOF

# Create symbolic links
echo
echo "Creating symbolic links..."
ln -sf "$LIB_DIR/liblegacybridge.so" "$LIB_DIR/liblegacybridge.so.1"
ln -sf "$LIB_DIR/liblegacybridge.so" "$LIB_DIR/liblegacybridge.so.1.0.0"

# Create wrapper script for tools
if [ -f "$INSTALL_DIR/tools/perf_test" ]; then
    cat > "$BIN_DIR/legacybridge-perf" << 'EOF'
#!/bin/bash
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
exec /opt/legacybridge/tools/perf_test "$@"
EOF
    chmod 755 "$BIN_DIR/legacybridge-perf"
fi

# Create uninstall script
echo
echo "Creating uninstall script..."
cat > "$INSTALL_DIR/uninstall.sh" << 'EOF'
#!/bin/bash
echo "Uninstalling LegacyBridge..."

# Remove files
rm -f /usr/local/lib/liblegacybridge.so*
rm -f /usr/local/include/legacybridge.h
rm -f /usr/local/lib/pkgconfig/legacybridge.pc
rm -f /usr/local/bin/legacybridge-*
rm -rf /opt/legacybridge

# Update library cache
ldconfig

echo "Uninstallation complete."
EOF
chmod 755 "$INSTALL_DIR/uninstall.sh"

# Verify installation
echo
echo "Verifying installation..."
INSTALL_ERROR=0

if [ -f "$LIB_DIR/liblegacybridge.so" ]; then
    echo "- Library installed successfully"
else
    echo "- ERROR: Library installation failed"
    INSTALL_ERROR=1
fi

if [ -f "$INCLUDE_DIR/legacybridge.h" ]; then
    echo "- Headers installed successfully"
else
    echo "- ERROR: Header installation failed"
    INSTALL_ERROR=1
fi

if ldconfig -p | grep -q liblegacybridge; then
    echo "- Library registered successfully"
else
    echo "- ERROR: Library registration failed"
    INSTALL_ERROR=1
fi

# Create desktop entries (if desktop environment exists)
if [ -d "/usr/share/applications" ]; then
    echo
    echo "Creating desktop entries..."
    cat > /usr/share/applications/legacybridge-docs.desktop << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=LegacyBridge Documentation
Comment=View LegacyBridge documentation
Exec=xdg-open /opt/legacybridge/docs
Icon=folder-documents
Terminal=false
Categories=Development;Documentation;
EOF

    cat > /usr/share/applications/legacybridge-examples.desktop << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=LegacyBridge Examples
Comment=Browse LegacyBridge example code
Exec=xdg-open /opt/legacybridge/examples
Icon=folder-development
Terminal=false
Categories=Development;
EOF
fi

echo
echo "============================================"
if [ $INSTALL_ERROR -eq 0 ]; then
    echo "INSTALLATION COMPLETED SUCCESSFULLY!"
    echo
    echo "LegacyBridge has been installed to:"
    echo "  Main directory: $INSTALL_DIR"
    echo "  Library: $LIB_DIR/liblegacybridge.so"
    echo "  Header: $INCLUDE_DIR/legacybridge.h"
    echo
    echo "To use LegacyBridge in your applications:"
    echo "  C/C++: #include <legacybridge.h>"
    echo "  Link with: -llegacybridge"
    echo
    echo "Run 'pkg-config --libs --cflags legacybridge' for compiler flags"
else
    echo "INSTALLATION COMPLETED WITH ERRORS"
    echo "Please check the error messages above."
fi
echo "============================================"
echo