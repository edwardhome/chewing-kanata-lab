# UTF-8 with BOM
# AI-Bopomofo (AI新酷音) Windows 安裝與註冊腳本
param(
    [switch]$NonInteractive
)

$Host.UI.RawUI.WindowTitle = "AI新酷音輸入法 一鍵安裝程式"

# 1. 檢查系統管理員權限
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "[提示] 需要系統管理員權限以註冊 Windows 語言設定，正在以系統管理員身分重新啟動..." -ForegroundColor Yellow
    $argList = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
    if ($NonInteractive) { $argList += " -NonInteractive" }
    Start-Process powershell.exe -ArgumentList $argList -Verb RunAs
    exit
}

Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "       AI新酷音輸入法 (AI-Bopomofo) 一鍵安裝程式        " -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan

$scriptDir = $PSScriptRoot
if (-not $scriptDir) { $scriptDir = (Get-Location).Path }
$targetDir = "$env:LOCALAPPDATA\Programs\ChewingDev"
$logPath = "$targetDir\install.log"

# 2. 終止可能佔用檔案的現有程序
Write-Host "[1/6] 檢查並釋放程序資源..." -ForegroundColor White
Get-Process -Name "ChewingPreferences", "chewing-editor" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# 3. 建立目標資料夾
if (-not (Test-Path $targetDir)) {
    New-Item -Path $targetDir -ItemType Directory -Force | Out-Null
}
"Starting installation at $(Get-Date)" | Out-File $logPath -Encoding utf8

$dllPath = "$targetDir\chewing_tip.dll"
$icoPath = "$targetDir\chewing.ico"
$tsfreg = "$targetDir\tsfreg.exe"
$clsid = "{13F2EF08-575C-4D8C-88E0-F67BB8052B84}"
$profileZhTw = "{CE45F71D-CE79-41D1-967D-640B65A380E3}"

# 4. 複製核心元件與字典
Write-Host "[2/6] 複製輸入法核心元件至: $targetDir" -ForegroundColor White
$filesToCopy = @(
    "chewing_tip.dll",
    "tsfreg.exe",
    "chewing.ico",
    "ChewingPreferences.exe",
    "chewing-editor.exe",
    "WebView2Loader.dll"
)

foreach ($file in $filesToCopy) {
    $src = Join-Path $scriptDir $file
    if (Test-Path $src) {
        Copy-Item -Path $src -Destination $targetDir -Force
    }
}

$dictSrcDir = Join-Path $scriptDir "Dictionary"
$dictTargetDir = Join-Path $targetDir "Dictionary"
if (Test-Path $dictSrcDir) {
    if (-not (Test-Path $dictTargetDir)) { New-Item -Path $dictTargetDir -ItemType Directory -Force | Out-Null }
    Copy-Item -Path "$dictSrcDir\*" -Destination $dictTargetDir -Recurse -Force
}

# 部署詞庫至使用者目錄 (%APPDATA%\Chewing 與 %APPDATA%\ChewingDev)
Write-Host "[3/6] 部署跨領域工程與自然科學學術專有名詞庫..." -ForegroundColor White
$appdataChewing = "$env:APPDATA\Chewing"
$appdataChewingDev = "$env:APPDATA\ChewingDev"
$appdataV4 = "$appdataChewing\v4"
New-Item -Path $appdataChewing -ItemType Directory -Force -ErrorAction SilentlyContinue | Out-Null
New-Item -Path $appdataChewingDev -ItemType Directory -Force -ErrorAction SilentlyContinue | Out-Null
New-Item -Path $appdataV4 -ItemType Directory -Force -ErrorAction SilentlyContinue | Out-Null

$techTermsSrc = Join-Path $dictSrcDir "technical-terms.txt"
if (Test-Path $techTermsSrc) {
    Copy-Item -Path $techTermsSrc -Destination "$appdataChewing\technical-terms.txt" -Force
    Copy-Item -Path $techTermsSrc -Destination "$appdataChewingDev\technical-terms.txt" -Force
}

$userDictSrc = Join-Path $dictSrcDir "user_dict.csv"
if (Test-Path $userDictSrc) {
    Copy-Item -Path $userDictSrc -Destination "$appdataV4\user_dict.csv" -Force
}

# 5. 註冊 COM CLSID
Write-Host "[4/6] 寫入 Windows COM CLSID 註冊資訊..." -ForegroundColor White
try {
    # HKLM
    $hklmBase = "HKLM:\SOFTWARE\Classes\CLSID\$clsid"
    if (Test-Path "HKLM:\SOFTWARE\Classes\CLSID") {
        New-Item -Path $hklmBase -Force -ErrorAction SilentlyContinue | Out-Null
        Set-ItemProperty -Path $hklmBase -Name "(default)" -Value "AI新酷音" -ErrorAction SilentlyContinue
        New-Item -Path "$hklmBase\InProcServer32" -Force -ErrorAction SilentlyContinue | Out-Null
        Set-ItemProperty -Path "$hklmBase\InProcServer32" -Name "(default)" -Value $dllPath -ErrorAction SilentlyContinue
        Set-ItemProperty -Path "$hklmBase\InProcServer32" -Name "ThreadingModel" -Value "Apartment" -ErrorAction SilentlyContinue
    }

    # HKCU
    $hkcuBase = "HKCU:\Software\Classes\CLSID\$clsid"
    New-Item -Path $hkcuBase -Force | Out-Null
    Set-ItemProperty -Path $hkcuBase -Name "(default)" -Value "AI新酷音"
    New-Item -Path "$hkcuBase\InProcServer32" -Force | Out-Null
    Set-ItemProperty -Path "$hkcuBase\InProcServer32" -Name "(default)" -Value $dllPath
    Set-ItemProperty -Path "$hkcuBase\InProcServer32" -Name "ThreadingModel" -Value "Apartment"
    "COM CLSID registered successfully." | Out-File $logPath -Append -Encoding utf8
} catch {
    "COM CLSID registration error: $_" | Out-File $logPath -Append -Encoding utf8
}

# 6. 呼叫 tsfreg 註冊 TSF Profile
Write-Host "[5/6] 註冊 Windows TSF 語言設定檔並設定顯示名稱為「AI新酷音」..." -ForegroundColor White
if (Test-Path $tsfreg) {
    # 註冊設定檔
    $p1 = Start-Process -FilePath $tsfreg -ArgumentList "-r `"$icoPath`"" -Wait -PassThru -NoNewWindow
    "tsfreg -r ExitCode: $($p1.ExitCode)" | Out-File $logPath -Append -Encoding utf8

    # 強制修正註冊表 LanguageProfile Description
    $hklmTip = "HKLM:\SOFTWARE\Microsoft\CTF\TIP\$clsid\LanguageProfile\0x00000404\$profileZhTw"
    if (Test-Path $hklmTip) {
        Set-ItemProperty -Path $hklmTip -Name "Description" -Value "AI新酷音" -ErrorAction SilentlyContinue
    }
    $hkcuTip = "HKCU:\Software\Microsoft\CTF\TIP\$clsid\LanguageProfile\0x00000404\$profileZhTw"
    if (-not (Test-Path $hkcuTip)) { New-Item -Path $hkcuTip -Force | Out-Null }
    Set-ItemProperty -Path $hkcuTip -Name "Description" -Value "AI新酷音"

    # 啟用輸入法
    $p2 = Start-Process -FilePath $tsfreg -ArgumentList "-i" -Wait -PassThru -NoNewWindow
    "tsfreg -i ExitCode: $($p2.ExitCode)" | Out-File $logPath -Append -Encoding utf8
}

# 7. 設定偏好預設值 (注音優先、F延遲350ms、AI輔助可依需求切換)
Write-Host "[6/6] 配置偏好設定預設值..." -ForegroundColor White
$regPath = "HKCU:\Software\ChewingTextService"
if (-not (Test-Path $regPath)) { New-Item -Path $regPath -Force | Out-Null }
Set-ItemProperty -Path $regPath -Name "DefaultEnglish" -Value 0 -Type DWord
Set-ItemProperty -Path $regPath -Name "FHoldDelayMs" -Value 350 -Type DWord
Set-ItemProperty -Path $regPath -Name "EnableAI" -Value 0 -Type DWord

Write-Host ""
Write-Host "========================================================" -ForegroundColor Green
Write-Host " [完成] AI新酷音輸入法 已成功安裝並啟用！" -ForegroundColor Green
Write-Host "========================================================" -ForegroundColor Green
Write-Host ""
Write-Host " 使用指南：" -ForegroundColor Yellow
Write-Host " 1. 請按鍵盤 [Win + Space] 切換輸入法，右下角語言列將顯示「AI新酷音」。" -ForegroundColor White
Write-Host " 2. 專業名詞快速補全：支援 I²C、I³C、SiO₂、FinFET、Wi-Fi 等 231 條專有名詞。" -ForegroundColor White
Write-Host " 3. 台灣教育部/國研院學術規範：雷射、電漿、繞射、電晶體等 109 條詞彙已就緒。" -ForegroundColor White
Write-Host " 4. 偏好設定：可執行 ChewingPreferences.exe 或右鍵點擊語言列開啟設定。" -ForegroundColor White
Write-Host ""

if (-not $NonInteractive -and -not $env:CI) {
    Read-Host "請按 Enter 鍵結束安裝..."
}
