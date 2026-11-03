# UTF-8 with BOM
# AI-Bopomofo (AI新酷音) Windows 移除腳本
param(
    [switch]$NonInteractive
)

$Host.UI.RawUI.WindowTitle = "AI新酷音輸入法 移除程式"

# 1. 檢查管理員權限
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "[提示] 需要系統管理員權限以移除 Windows 語言設定，正在以系統管理員身分重新啟動..." -ForegroundColor Yellow
    $argList = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
    if ($NonInteractive) { $argList += " -NonInteractive" }
    Start-Process powershell.exe -ArgumentList $argList -Verb RunAs
    exit
}

Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "           AI新酷音輸入法 移除程式                      " -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan

# 2. 終止可能佔用檔案的現有程序
Write-Host "[1/4] 關閉相關程序..." -ForegroundColor White
Get-Process -Name "ChewingPreferences", "chewing-editor" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

$targetDir = "$env:LOCALAPPDATA\Programs\ChewingDev"
$tsfreg = "$targetDir\tsfreg.exe"
$clsid = "{13F2EF08-575C-4D8C-88E0-F67BB8052B84}"

# 3. 停用與取消註冊 TSF Profile
Write-Host "[2/4] 取消 Windows 語言列輸入法註冊..." -ForegroundColor White
if (Test-Path $tsfreg) {
    Start-Process -FilePath $tsfreg -ArgumentList "-d" -Wait -NoNewWindow -ErrorAction SilentlyContinue
    Start-Process -FilePath $tsfreg -ArgumentList "-u" -Wait -NoNewWindow -ErrorAction SilentlyContinue
}

# 4. 清理註冊表
Write-Host "[3/4] 清理 COM CLSID 登錄項目..." -ForegroundColor White
Remove-Item -Path "HKLM:\SOFTWARE\Classes\CLSID\$clsid" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "HKCU:\Software\Classes\CLSID\$clsid" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "HKLM:\SOFTWARE\Microsoft\CTF\TIP\$clsid" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "HKCU:\Software\Microsoft\CTF\TIP\$clsid" -Recurse -Force -ErrorAction SilentlyContinue

# 5. 提示完成
Write-Host "[4/4] 移除程序完成。" -ForegroundColor White
Write-Host ""
Write-Host "========================================================" -ForegroundColor Green
Write-Host " [完成] AI新酷音輸入法 已從 Windows 系統乾淨移除。" -ForegroundColor Green
Write-Host "========================================================" -ForegroundColor Green
Write-Host ""

if (-not $NonInteractive -and -not $env:CI) {
    Read-Host "請按 Enter 鍵結束..."
}
