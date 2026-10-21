# Chewing ASCII-First Dev (TSF) - 隔離卸載腳本
# 乾淨移除開發版登錄項，100% 保持原版輸入法完整

$CLSID_CHEWING_DEV = "{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}"

Write-Host "========================================================="
Write-Host " 移除開發隔離版 Chewing TSF 註冊"
Write-Host "========================================================="

$regBase = "HKCU:\Software\Classes\CLSID\$CLSID_CHEWING_DEV"
if (Test-Path $regBase) {
    Remove-Item -Path $regBase -Recurse -Force
    Write-Host "已乾淨移除開發版登錄項: $regBase"
} else {
    Write-Host "未找到開發版登錄項，無須清理。"
}

$devConfig = "$env:APPDATA\ChewingDev"
if (Test-Path $devConfig) {
    Write-Host "提示：開發版設定檔目錄仍保留於 $devConfig，如需徹底刪除請手動清理。"
}
