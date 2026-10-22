# Chewing ASCII-First Dev (TSF) - Registration Script
# Note: Uses an isolated CLSID in HKCU to avoid conflicting with official Chewing IME.

param (
    [string]$DllPath = "$PSScriptRoot\..\..\dist\windows\chewing_tip.dll"
)

$CLSID_CHEWING_DEV = "{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}"
$DESCRIPTION       = "Chewing (ASCII-First Dev)"

$fullDllPath = [System.IO.Path]::GetFullPath($DllPath)

Write-Host "========================================================="
Write-Host " Registering Isolated Chewing TSF: $DESCRIPTION"
Write-Host " DLL Path: $fullDllPath"
Write-Host " CLSID:    $CLSID_CHEWING_DEV"
Write-Host "========================================================="

if (-not (Test-Path $fullDllPath)) {
    Write-Error "DLL not found: $fullDllPath. Build the DLL first."
    exit 1
}

$regBase = "HKCU:\Software\Classes\CLSID\$CLSID_CHEWING_DEV"
New-Item -Path $regBase -Force | Out-Null
Set-ItemProperty -Path $regBase -Name "(default)" -Value $DESCRIPTION
New-Item -Path "$regBase\InProcServer32" -Force | Out-Null
Set-ItemProperty -Path "$regBase\InProcServer32" -Name "(default)" -Value $fullDllPath
Set-ItemProperty -Path "$regBase\InProcServer32" -Name "ThreadingModel" -Value "Apartment"

Write-Host "Registration completed successfully in HKCU."
Write-Host "Configuration path: $env:APPDATA\ChewingDev\policy.toml"
