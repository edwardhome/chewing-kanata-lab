@echo off
chcp 65001 >nul
title 新酷音 (ASCII-First) 卸載程式

net session >nul 2>&1
if %errorLevel% neq 0 (
    echo 正在請求系統管理員權限以解除註冊...
    powershell -NoProfile -Command "Start-Process cmd.exe -ArgumentList '/c \"\"%~f0\"\"' -Verb RunAs"
    exit /b
)

echo ========================================================
echo  正在自 Windows 系統卸載新酷音輸入法...
echo ========================================================

set TARGET_DIR=%LOCALAPPDATA%\Programs\ChewingDev
if exist "%TARGET_DIR%\tsfreg.exe" (
    "%TARGET_DIR%\tsfreg.exe" -d
    "%TARGET_DIR%\tsfreg.exe" -u
)

reg delete "HKLM\SOFTWARE\Classes\CLSID\{13F2EF08-575C-4D8C-88E0-F67BB8052B84}" /f >nul 2>&1
reg delete "HKCU\Software\Classes\CLSID\{13F2EF08-575C-4D8C-88E0-F67BB8052B84}" /f >nul 2>&1

echo ========================================================
echo  卸載完成！新酷音輸入法已自語言列中乾淨移除。
echo ========================================================
pause
