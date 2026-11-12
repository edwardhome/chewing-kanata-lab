# AI新酷音：保留詞庫、偏好與版本檔案，只解除本產品的隔離註冊。
param([switch]$NonInteractive, [switch]$ValidateOnly, [string]$ExpectedUserSid)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'package_guard.ps1')
. (Join-Path $PSScriptRoot 'machine_registration.ps1')
Assert-ChewingInstallUser $ExpectedUserSid
$machineRoot = Get-ChewingMachineRoot
$baseDir = Join-Path $env:LOCALAPPDATA 'Programs\ChewingDev'
$pointer = Join-Path $baseDir 'current.json'
if (-not (Test-Path -LiteralPath $pointer -PathType Leaf)) {
    throw 'No verified ChewingDev installation record. Legacy registration is left untouched.'
}
$current = Get-Content -LiteralPath $pointer -Raw -Encoding UTF8 | ConvertFrom-Json
$directory = [IO.Path]::GetFullPath($current.version_directory)
$versions = [IO.Path]::GetFullPath((Join-Path $baseDir 'versions')) + [IO.Path]::DirectorySeparatorChar
$machineVersions = [IO.Path]::GetFullPath((Join-Path $machineRoot 'versions')) + [IO.Path]::DirectorySeparatorChar
$isMachine = $directory.StartsWith($machineVersions, [StringComparison]::OrdinalIgnoreCase)
if (-not $isMachine -and -not $directory.StartsWith($versions, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Installation record is outside ChewingDev versions.'
}
$null = Assert-ChewingDevPackage -Directory $directory
if ($isMachine) { $null = Assert-ChewingMachinePackage -Directory $directory }
$machineClass = 'HKLM:\Software\Classes\CLSID\{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}'
$ownerSid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
Assert-ChewingMachineRegistrationOwner $machineClass $ownerSid
if (Test-Path -LiteralPath $machineClass) {
    $machineDll = (Get-Item -LiteralPath (Join-Path $machineClass 'InProcServer32')).GetValue('')
    if ($machineDll -ine (Join-Path $directory 'chewing_tip.dll')) {
        throw 'Machine COM DLL differs from this installation; no changes made.'
    }
}
$regBase = 'HKCU:\Software\Classes\CLSID\{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}'
$inproc = Join-Path $regBase 'InProcServer32'
if (Test-Path -LiteralPath $inproc) {
    $registered = (Get-Item -LiteralPath $inproc).GetValue('')
    if ($registered -ine (Join-Path $directory 'chewing_tip.dll')) {
        throw 'Registered DLL differs from the verified installation; no changes made.'
    }
}
if ($ValidateOnly) { Write-Output 'Uninstall preflight passed; no changes made.'; return }
if (Invoke-ChewingElevationIfNeeded -ScriptPath $PSCommandPath) { return }
Invoke-ChewingDevRegistration -Executable (Join-Path $directory 'tsfreg.exe') -Arguments @('-u')
if (Test-Path -LiteralPath $regBase) { Remove-Item -LiteralPath $regBase -Recurse -Force }
if (Test-Path -LiteralPath $machineClass) { Remove-Item -LiteralPath $machineClass -Recurse -Force }
$progId = 'HKCU:\Software\Classes\AI-Chewing.TextService'
if (Test-Path -LiteralPath $progId) { Remove-Item -LiteralPath $progId -Recurse -Force }
Remove-Item -LiteralPath $pointer
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\AI-Chewing'
if (Test-Path -LiteralPath $uninstallKey) { Remove-Item -LiteralPath $uninstallKey -Recurse -Force }
Write-Output '已解除 AI新酷音隔離註冊，保留詞庫、偏好與版本檔案。'
