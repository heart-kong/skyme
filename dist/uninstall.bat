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

:: Unregister the COM DLL (removes CLSID registry entries)
if exist "%TARGET_DIR%\skyme_ime_service.dll" (
    echo Unregistering TSF Text Service...
    regsvr32 /u /s "%TARGET_DIR%\skyme_ime_service.dll"
)

:: Remove files
echo Removing files...
if exist "%TARGET_DIR%" (
    rmdir /s /q "%TARGET_DIR%"
)

:: Clean PATH safely via PowerShell
echo Cleaning system PATH...
powershell -NoProfile -Command "& { $p = [Environment]::GetEnvironmentVariable('PATH','Machine'); $t = '%TARGET_DIR%'; $c = $p.Split(';') | Where-Object { $_ -ne $t -and $_ -ne ($t+'\') -and $_ -ne ($t.TrimEnd('\')) }; $n = $c -join ';'; if ($n -ne $p) { [Environment]::SetEnvironmentVariable('PATH',$n,'Machine'); Write-Host 'PATH cleaned.' } else { Write-Host 'No Skyme entries in PATH.' } }"

echo.
echo ========================================
echo   Uninstallation complete!
echo ========================================
pause
