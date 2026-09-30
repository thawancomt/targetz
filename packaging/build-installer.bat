@echo off
setlocal enabledelayedexpansion

echo ========================================================
echo Building Targetz Windows Binary and Setup Installer
echo ========================================================

echo [1/3] Compiling Release Binary for x86_64-pc-windows-msvc...
cargo build --release --bin targetz --target x86_64-pc-windows-msvc
if %ERRORLEVEL% neq 0 (
    echo Error: Cargo build failed!
    exit /b %ERRORLEVEL%
)

if not exist "dist" mkdir dist

echo [2/3] Checking for Installer Compilers (Inno Setup or NSIS)...
where iscc >nul 2>nul
if %ERRORLEVEL% equ 0 (
    echo [3/3] Compiling with Inno Setup...
    iscc packaging\installer.iss
    goto :done
)

where makensis >nul 2>nul
if %ERRORLEVEL% equ 0 (
    echo [3/3] Compiling with NSIS...
    makensis packaging\installer.nsi
    goto :done
)

echo.
echo [!] Neither Inno Setup (iscc) nor NSIS (makensis) was found in PATH.
echo [i] The compiled standalone executable is ready at:
echo     target\x86_64-pc-windows-msvc\release\targetz.exe
echo.
echo To generate the setup wizard, install Inno Setup:
echo     winget install JRSoftware.InnoSetup
echo and re-run this script.
exit /b 0

:done
echo.
echo ========================================================
echo SUCCESS: Installer generated in dist\Targetz-Setup-x86_64.exe
echo ========================================================
