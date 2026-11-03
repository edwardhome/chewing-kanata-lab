@echo off
title AI新酷音 一鍵安裝啟用程式

:: 檢查是否具備系統管理員權限，若無則自動彈出 UAC 權限確認視窗
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo 提示: 向 Windows 註冊輸入法需要系統管理員權限。
    echo 正在為您彈出權限確認視窗，請點擊「是」以繼續...
    powershell -NoProfile -Command "Start-Process cmd.exe -ArgumentList '/c \"\"%~f0\"\"' -Verb RunAs"
    exit /b
)

cd /d "%~dp0"
echo ========================================================
echo  正在安裝 AI新酷音 到 Windows 系統
echo ========================================================

set TARGET_DIR=%LOCALAPPDATA%\Programs\ChewingDev
if not exist "%TARGET_DIR%" mkdir "%TARGET_DIR%"

echo [1/5] 複製核心檔案至 %TARGET_DIR%...
copy /y "chewing_tip.dll" "%TARGET_DIR%\" >nul
copy /y "tsfreg.exe" "%TARGET_DIR%\" >nul
copy /y "chewing.ico" "%TARGET_DIR%\" >nul
if exist "ChewingPreferences.exe" copy /y "ChewingPreferences.exe" "%TARGET_DIR%\" >nul
if exist "chewing-editor.exe" copy /y "chewing-editor.exe" "%TARGET_DIR%\" >nul
if exist "WebView2Loader.dll" copy /y "WebView2Loader.dll" "%TARGET_DIR%\" >nul

if exist "Dictionary" (
    if not exist "%TARGET_DIR%\Dictionary" mkdir "%TARGET_DIR%\Dictionary"
    copy /y "Dictionary\*.*" "%TARGET_DIR%\Dictionary\" >nul
    if not exist "%APPDATA%\Chewing" mkdir "%APPDATA%\Chewing"
    if exist "Dictionary\technical-terms.txt" copy /y "Dictionary\technical-terms.txt" "%APPDATA%\Chewing\" >nul
)

echo [2/5] 寫入 COM 伺服器註冊表...
reg add "HKLM\SOFTWARE\Classes\CLSID\{13F2EF08-575C-4D8C-88E0-F67BB8052B84}" /ve /t REG_SZ /d "AI新酷音" /f >nul
reg add "HKLM\SOFTWARE\Classes\CLSID\{13F2EF08-575C-4D8C-88E0-F67BB8052B84}\InProcServer32" /ve /t REG_SZ /d "%TARGET_DIR%\chewing_tip.dll" /f >nul
reg add "HKLM\SOFTWARE\Classes\CLSID\{13F2EF08-575C-4D8C-88E0-F67BB8052B84}\InProcServer32" /v "ThreadingModel" /t REG_SZ /d "Apartment" /f >nul

reg add "HKCU\Software\Classes\CLSID\{13F2EF08-575C-4D8C-88E0-F67BB8052B84}" /ve /t REG_SZ /d "AI新酷音" /f >nul
reg add "HKCU\Software\Classes\CLSID\{13F2EF08-575C-4D8C-88E0-F67BB8052B84}\InProcServer32" /ve /t REG_SZ /d "%TARGET_DIR%\chewing_tip.dll" /f >nul
reg add "HKCU\Software\Classes\CLSID\{13F2EF08-575C-4D8C-88E0-F67BB8052B84}\InProcServer32" /v "ThreadingModel" /t REG_SZ /d "Apartment" /f >nul

echo [3/5] 註冊 Windows TSF 輸入法設定檔...
"%TARGET_DIR%\tsfreg.exe" -r "%TARGET_DIR%\chewing.ico"

echo [4/5] 確保語言列名稱寫入 HKLM / HKCU...
reg add "HKLM\SOFTWARE\Microsoft\CTF\TIP\{13F2EF08-575C-4D8C-88E0-F67BB8052B84}\LanguageProfile\0x00000404\{CE45F71D-CE79-41D1-967D-640B65A380E3}" /v "Description" /t REG_SZ /d "AI新酷音" /f >nul
reg add "HKCU\Software\Microsoft\CTF\TIP\{13F2EF08-575C-4D8C-88E0-F67BB8052B84}\LanguageProfile\0x00000404\{CE45F71D-CE79-41D1-967D-640B65A380E3}" /v "Description" /t REG_SZ /d "AI新酷音" /f >nul

echo [5/5] 啟用輸入法至語言列 (Win + Space 清單)...
"%TARGET_DIR%\tsfreg.exe" -i

:: 寫入使用者預設值 (預設注音模式、F延遲350ms、AI關閉)
reg add "HKCU\Software\ChewingTextService" /v "DefaultEnglish" /t REG_DWORD /d 0 /f >nul
reg add "HKCU\Software\ChewingTextService" /v "FHoldDelayMs" /t REG_DWORD /d 350 /f >nul
reg add "HKCU\Software\ChewingTextService" /v "EnableAI" /t REG_DWORD /d 0 /f >nul

echo ========================================================
echo  恭喜！安裝與註冊已全部順利完成！
echo.
echo  請按下鍵盤上的 [Win + Space]：
echo  您將會在右下角輸入法清單中看到「AI新酷音」！
echo ========================================================
pause
