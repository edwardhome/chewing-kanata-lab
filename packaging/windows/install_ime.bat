@echo off
chcp 65001 >nul
title 新酷音 (ASCII-First) 一鍵安裝啟用程式

:: 檢查是否具備系統管理員權限，若無則自動彈出 UAC 提權對話框
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo 提示: 向 Windows 註冊輸入法需要系統管理員權限。
    echo 正在為您彈出權限確認視窗，請點擊「是」以繼續...
    powershell -NoProfile -Command "Start-Process cmd.exe -ArgumentList '/c \"\"%~f0\"\"' -Verb RunAs"
    exit /b
)

cd /d "%~dp0"
echo ========================================================
echo  正在安裝新酷音輸入法 (ASCII-First 版) 至 Windows 系統
echo ========================================================

set TARGET_DIR=%LOCALAPPDATA%\Programs\ChewingDev
if not exist "%TARGET_DIR%" mkdir "%TARGET_DIR%"

echo [1/4] 複製核心檔案至 %TARGET_DIR%...
copy /y "chewing_tip.dll" "%TARGET_DIR%\" >nul
copy /y "tsfreg.exe" "%TARGET_DIR%\" >nul
copy /y "chewing.ico" "%TARGET_DIR%\" >nul

echo [2/4] 註冊 COM 元件...
reg add "HKLM\SOFTWARE\Classes\CLSID\{13F2EF08-575C-4D8C-88E0-F67BB8052B84}" /ve /t REG_SZ /d "新酷音輸入法 (ASCII-First)" /f >nul
reg add "HKLM\SOFTWARE\Classes\CLSID\{13F2EF08-575C-4D8C-88E0-F67BB8052B84}\InProcServer32" /ve /t REG_SZ /d "%TARGET_DIR%\chewing_tip.dll" /f >nul
reg add "HKLM\SOFTWARE\Classes\CLSID\{13F2EF08-575C-4D8C-88E0-F67BB8052B84}\InProcServer32" /v "ThreadingModel" /t REG_SZ /d "Apartment" /f >nul

echo [3/4] 註冊 Windows TSF 輸入法設定檔...
"%TARGET_DIR%\tsfreg.exe" -r "%TARGET_DIR%\chewing.ico"

echo [4/4] 啟用輸入法至語言列 (Win + Space 清單)...
"%TARGET_DIR%\tsfreg.exe" -i

:: 部署 ASCII-First 設定檔至 APPDATA
if not exist "%APPDATA%\ChewingDev" mkdir "%APPDATA%\ChewingDev"
if not exist "%APPDATA%\ChewingDev\policy.toml" (
    if exist "..\..\config\policy.example.toml" (
        copy /y "..\..\config\policy.example.toml" "%APPDATA%\ChewingDev\policy.toml" >nul
    )
)

echo ========================================================
echo  恭喜！安裝與註冊已全部完成！
echo.
echo  請按下鍵盤上的 [Win + Space]：
echo  您將會在右下角輸入法清單中看到「新酷音輸入法」！
echo ========================================================
pause
