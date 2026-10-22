# Chewing ASCII-First Dev (TSF) - Unregistration Script
# Cleanly removes isolated dev registration from HKCU without touching official Chewing.

$CLSID_CHEWING_DEV = "{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}"

Write-Host "========================================================="
Write-Host " Unregistering Isolated Chewing TSF"
Write-Host " CLSID: $CLSID_CHEWING_DEV"
Write-Host "========================================================="

$regBase = "HKCU:\Software\Classes\CLSID\$CLSID_CHEWING_DEV"
if (Test-Path $regBase) {
    Remove-Item -Path $regBase -Recurse -Force
    Write-Host "Successfully removed registry entry: $regBase"
} else {
    Write-Host "No dev registry entry found. Nothing to clean."
}
