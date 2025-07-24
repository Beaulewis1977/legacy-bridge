@echo off
REM LegacyBridge Enterprise Edition Installation Script for Windows
REM Version 1.0.0

echo.
echo ============================================
echo LegacyBridge Enterprise Edition Installer
echo Version 1.0.0
echo ============================================
echo.

REM Check for administrator privileges
net session >nul 2>&1
if %errorLevel% NEQ 0 (
    echo ERROR: This installer requires administrator privileges.
    echo Please run as Administrator.
    pause
    exit /b 1
)

REM Set installation paths
set INSTALL_DIR=%ProgramFiles%\LegacyBridge
set SYSTEM_DIR=%SystemRoot%\System32

echo Installation directory: %INSTALL_DIR%
echo.

REM Create installation directory
echo Creating installation directory...
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"
if not exist "%INSTALL_DIR%\bin" mkdir "%INSTALL_DIR%\bin"
if not exist "%INSTALL_DIR%\include" mkdir "%INSTALL_DIR%\include"
if not exist "%INSTALL_DIR%\examples" mkdir "%INSTALL_DIR%\examples"
if not exist "%INSTALL_DIR%\docs" mkdir "%INSTALL_DIR%\docs"

REM Copy files
echo.
echo Installing LegacyBridge components...

echo - Installing DLL...
copy /Y "..\bin\legacybridge.dll" "%INSTALL_DIR%\bin\" >nul
copy /Y "..\bin\legacybridge.dll" "%SYSTEM_DIR%\" >nul

echo - Installing header files...
copy /Y "..\include\legacybridge.h" "%INSTALL_DIR%\include\" >nul

echo - Installing examples...
xcopy /E /Y /Q "..\examples\*" "%INSTALL_DIR%\examples\" >nul

echo - Installing documentation...
xcopy /E /Y /Q "..\docs\*" "%INSTALL_DIR%\docs\" >nul

echo - Installing tools...
if not exist "%INSTALL_DIR%\tools" mkdir "%INSTALL_DIR%\tools"
copy /Y "..\tools\*" "%INSTALL_DIR%\tools\" >nul 2>&1

REM Register DLL
echo.
echo Registering LegacyBridge DLL...
regsvr32 /s "%SYSTEM_DIR%\legacybridge.dll"

REM Add to PATH
echo.
echo Adding LegacyBridge to system PATH...
setx /M PATH "%PATH%;%INSTALL_DIR%\bin" >nul 2>&1

REM Create registry entries
echo.
echo Creating registry entries...
reg add "HKLM\SOFTWARE\LegacyBridge" /v "Version" /t REG_SZ /d "1.0.0" /f >nul
reg add "HKLM\SOFTWARE\LegacyBridge" /v "InstallPath" /t REG_SZ /d "%INSTALL_DIR%" /f >nul
reg add "HKLM\SOFTWARE\LegacyBridge" /v "DLLPath" /t REG_SZ /d "%SYSTEM_DIR%\legacybridge.dll" /f >nul

REM Create uninstaller
echo.
echo Creating uninstaller...
echo @echo off > "%INSTALL_DIR%\uninstall.bat"
echo echo Uninstalling LegacyBridge... >> "%INSTALL_DIR%\uninstall.bat"
echo regsvr32 /u /s "%SYSTEM_DIR%\legacybridge.dll" >> "%INSTALL_DIR%\uninstall.bat"
echo del /F /Q "%SYSTEM_DIR%\legacybridge.dll" >> "%INSTALL_DIR%\uninstall.bat"
echo reg delete "HKLM\SOFTWARE\LegacyBridge" /f >> "%INSTALL_DIR%\uninstall.bat"
echo rd /S /Q "%INSTALL_DIR%" >> "%INSTALL_DIR%\uninstall.bat"
echo echo Uninstallation complete. >> "%INSTALL_DIR%\uninstall.bat"
echo pause >> "%INSTALL_DIR%\uninstall.bat"

REM Verify installation
echo.
echo Verifying installation...
if exist "%SYSTEM_DIR%\legacybridge.dll" (
    echo - DLL installed successfully
) else (
    echo - ERROR: DLL installation failed
    set INSTALL_ERROR=1
)

if exist "%INSTALL_DIR%\include\legacybridge.h" (
    echo - Headers installed successfully
) else (
    echo - ERROR: Header installation failed
    set INSTALL_ERROR=1
)

REM Create shortcuts
echo.
echo Creating shortcuts...
powershell -Command "$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('%USERPROFILE%\Desktop\LegacyBridge Examples.lnk'); $Shortcut.TargetPath = '%INSTALL_DIR%\examples'; $Shortcut.Save()"
powershell -Command "$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('%USERPROFILE%\Desktop\LegacyBridge Documentation.lnk'); $Shortcut.TargetPath = '%INSTALL_DIR%\docs'; $Shortcut.Save()"

echo.
echo ============================================
if defined INSTALL_ERROR (
    echo INSTALLATION COMPLETED WITH ERRORS
    echo Please check the error messages above.
) else (
    echo INSTALLATION COMPLETED SUCCESSFULLY!
    echo.
    echo LegacyBridge has been installed to:
    echo %INSTALL_DIR%
    echo.
    echo The DLL has been registered and is ready for use.
    echo Shortcuts have been created on your desktop.
)
echo ============================================
echo.

pause