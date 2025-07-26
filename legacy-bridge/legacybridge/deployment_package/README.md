# LegacyBridge DLL Deployment Package

## Table of Contents
- [Contents](#contents)
  - [Directory Structure](#directory-structure)
- [Quick Start](#quick-start)
  - [For VB6 Developers](#for-vb6-developers)
  - [For VFP9 Developers](#for-vfp9-developers)
- [Features](#features)
- [System Requirements](#system-requirements)
- [Function Summary](#function-summary)
- [Testing](#testing)
- [Version](#version)
- [Support](#support)
- [License](#license)

## Contents

This deployment package contains everything needed to integrate LegacyBridge into VB6 and VFP9 applications.

### Directory Structure

```
deployment_package/
├── bin/
│   └── legacybridge.dll          # Main DLL (32-bit compatible)
├── include/
│   └── legacybridge.h            # C header file (for reference)
├── examples/
│   ├── vb6/
│   │   ├── TestLegacyBridge.frm  # VB6 test form
│   │   └── LegacyBridge.bas      # VB6 wrapper module
│   └── vfp9/
│       ├── test_legacybridge.prg # VFP9 test program
│       └── legacybridge.prg      # VFP9 wrapper class
└── docs/
    └── INTEGRATION_GUIDE.md      # Complete integration guide
```

## Quick Start

### For VB6 Developers

1. Copy `bin/legacybridge.dll` to your application directory
2. Add `examples/vb6/LegacyBridge.bas` to your VB6 project
3. Start using the conversion functions:

```vb
Dim markdown As String
markdown = ConvertRtfToMarkdown("{\rtf1 Hello World\par}")
```

### For VFP9 Developers

1. Copy `bin/legacybridge.dll` to your application directory
2. Copy `examples/vfp9/legacybridge.prg` to your project
3. Use the conversion functions:

```foxpro
SET PROCEDURE TO legacybridge.prg ADDITIVE
lcMarkdown = ConvertRtfToMarkdown("{\rtf1 Hello World\par}")
```

## Features

- **High Performance**: Optimized for speed with large documents
- **Full RTF Support**: Handles complex RTF formatting
- **Markdown Support**: CommonMark compliant with extensions
- **Batch Processing**: Convert entire folders
- **Error Handling**: Detailed error messages
- **Memory Safe**: Automatic memory management
- **32-bit Compatible**: Works with legacy systems

## System Requirements

- Windows XP or later
- 32-bit or 64-bit Windows
- Visual Basic 6.0 Runtime or Visual FoxPro 9.0
- 512MB RAM minimum
- 10MB disk space

## Function Summary

The DLL exports 25 functions organized into 6 categories:

1. **Core Functions** (7): Basic conversion operations
2. **Validation Functions** (3): Document validation
3. **Batch Functions** (4): Folder operations
4. **Processing Functions** (2): Format cleaning
5. **Template Functions** (5): Template management
6. **Database Functions** (4): Table operations

See `docs/INTEGRATION_GUIDE.md` for complete documentation.

## Testing

Use the provided examples to test the DLL:

1. **VB6**: Open `examples/vb6/TestLegacyBridge.frm` in VB6
2. **VFP9**: Run `examples/vfp9/test_legacybridge.prg` in VFP9

## Version

LegacyBridge DLL v1.0.0
- Release Date: July 2024
- Architecture: 32-bit
- Size: ~730KB

## Support

For detailed integration instructions, see `docs/INTEGRATION_GUIDE.md`

## License

See LICENSE file for terms of use.