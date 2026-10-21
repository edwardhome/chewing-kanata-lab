# Chewing ASCII-First Dev (TSF) - 隔離註冊腳本
# 注意：本腳本採用獨立的開發版 GUID，絕不覆蓋系統正式新酷音

param (
    [string]$DllPath = "$PSScriptRoot\chewing_tip.dll"
)

$CLSID_CHEWING_DEV = "{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}"
$PROFILE_ID_DEV    = "{D4C3B2A2-0E9E-4340-9E11-209B0E9C7E2E}"
$DESCRIPTION       = "新酷音 (ASCII-First Dev)"

Write-Host "========================================================="
Write-Host " 註冊開發隔離版 Chewing TSF: $DESCRIPTION"
Write-Host " CLSID: $CLSID_CHEWING_DEV"
Write-Host "========================================================="

if (-not (Test-Path $DllPath)) {
    Write-Warning "DLL 未找到: $DllPath。請先完成二進位建置後再行註冊。"
    Write-Host "若僅於本機測試，請使用 mock 或隔離環境。"
    exit 0
}

# 僅寫入目前使用者的獨立登錄項 (HKCU，不破壞全域 HKLM)
$regBase = "HKCU:\Software\Classes\CLSID\$CLSID_CHEWING_DEV"
New-Item -Path $regBase -Force | Out-Null
Set-ItemProperty -Path $regBase -Name "(default)" -Value $DESCRIPTION
New-Item -Path "$regBase\InProcServer32" -Force | Out-Null
Set-ItemProperty -Path "$regBase\InProcServer32" -Name "(default)" -Value $DllPath
Set-ItemProperty -Path "$regBase\InProcServer32" -Name "ThreadingModel" -Value "Apartment"

Write-Host "開發版 TSF 註冊已建立至 HKCU (與正式版新酷音完全隔離)。"
