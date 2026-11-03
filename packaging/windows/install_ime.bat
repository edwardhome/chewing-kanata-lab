@echo off
:: ============================================================
:: AI-Bopomofo (AI-Chewing) Windows Installer Launcher
:: Pure ASCII Trampoline - Guaranteed CP950 / UTF-8 compatibility
:: ============================================================

setlocal
cd /d "%~dp0"

:: Check for Administrator privileges
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [INFO] Requesting Administrator privileges...
    powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "Start-Process powershell.exe -ArgumentList '-NoProfile -ExecutionPolicy Bypass -File """"%~dp0do_install.ps1""""' -Verb RunAs"
    exit /b
)

:: Run installer script directly
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0do_install.ps1"
if %errorlevel% neq 0 (
    echo [ERROR] Installation script returned error code: %errorlevel%
    pause
    exit /b %errorlevel%
)
