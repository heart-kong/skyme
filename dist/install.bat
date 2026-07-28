@echo off
setlocal enabledelayedexpansion

echo ========================================
echo   Skyme Input Method - Installer
echo ========================================
echo.

:: Check for admin rights
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: This installer must be run as Administrator.
    echo Right-click install.bat and select "Run as administrator".
    pause
    exit /b 1
)

set "TARGET_DIR=%ProgramFiles%\Skyme"

echo Installing to: %TARGET_DIR%
echo.

:: Create target directory
if not exist "%TARGET_DIR%" mkdir "%TARGET_DIR%"

:: Copy DLLs
echo Copying Skyme IME service...
copy /Y "%~dp0skyme_ime_service.dll" "%TARGET_DIR%\" || exit /b 1

if exist "%~dp0rime.dll" (
    echo Copying Rime engine...
    copy /Y "%~dp0rime.dll" "%TARGET_DIR%\" || exit /b 1
)

:: Register the COM DLL
echo.
echo Registering TSF Text Service...
regsvr32 /s "%TARGET_DIR%\skyme_ime_service.dll"
if %errorlevel% equ 0 (
    echo SUCCESS: Skyme IME registered with Windows.
) else (
    echo WARNING: Registration may have failed (error %errorlevel%).
    echo Try running: regsvr32 "%TARGET_DIR%\skyme_ime_service.dll"
)

:: Add to PATH for runtime loading
echo.
echo Adding to PATH...
setx /m PATH "!PATH!;%TARGET_DIR%" >nul 2>&1

echo.
echo ========================================
echo   Installation complete!
echo.
echo   To use Skyme:
echo   1. Open Language Settings
echo   2. Add "Skyme Input Method" to your input methods
echo   3. Switch to it with Win+Space
echo.
echo   To uninstall, run uninstall.bat
echo ========================================
pause
