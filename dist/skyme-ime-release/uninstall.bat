@echo off
setlocal enabledelayedexpansion

echo ========================================
echo   Skyme Input Method - Uninstaller
echo ========================================

net session >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Must be run as Administrator.
    pause
    exit /b 1
)

set "TARGET_DIR=%ProgramFiles%\Skyme"

:: Unregister the COM DLL
if exist "%TARGET_DIR%\skyme_ime_service.dll" (
    echo Unregistering TSF Text Service...
    regsvr32 /u /s "%TARGET_DIR%\skyme_ime_service.dll"
)

:: Remove files
echo Removing files...
if exist "%TARGET_DIR%" (
    rmdir /s /q "%TARGET_DIR%"
)

echo.
echo Uninstallation complete.
pause
