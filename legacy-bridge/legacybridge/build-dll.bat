@echo off
REM Build script for LegacyBridge DLL (Windows)

echo Building LegacyBridge DLL...

REM Navigate to the dll-build directory
cd dll-build

REM Clean previous builds
echo Cleaning previous builds...
cargo clean

REM Build the DLL
echo Building release version...
cargo build --release

REM Check if build was successful
if %ERRORLEVEL% EQU 0 (
    echo Build successful!
    
    REM Create output directory
    if not exist ..\lib mkdir ..\lib
    
    REM Copy the DLL and lib files
    copy target\release\legacybridge.dll ..\lib\
    copy target\release\legacybridge.dll.lib ..\lib\
    
    echo Library copied to: ..\lib\legacybridge.dll
    echo Import library copied to: ..\lib\legacybridge.dll.lib
    echo Header file available at: ..\include\legacybridge.h
    
    REM Generate DEF file for VB6 compatibility
    echo Generating DEF file...
    echo LIBRARY legacybridge > ..\lib\legacybridge.def
    echo EXPORTS >> ..\lib\legacybridge.def
    echo    legacybridge_rtf_to_markdown >> ..\lib\legacybridge.def
    echo    legacybridge_markdown_to_rtf >> ..\lib\legacybridge.def
    echo    legacybridge_free_string >> ..\lib\legacybridge.def
    echo    legacybridge_get_last_error >> ..\lib\legacybridge.def
    echo    legacybridge_get_version >> ..\lib\legacybridge.def
    echo    legacybridge_batch_rtf_to_markdown >> ..\lib\legacybridge.def
    echo    legacybridge_batch_markdown_to_rtf >> ..\lib\legacybridge.def
    
    echo DEF file created at: ..\lib\legacybridge.def
) else (
    echo Build failed!
    exit /b 1
)

echo Build complete!
pause