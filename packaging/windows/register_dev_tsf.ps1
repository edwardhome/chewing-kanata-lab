# Compatibility entry point: use verified, versioned package installation.
param([string]$DllPath = "$PSScriptRoot\..\..\dist\windows\chewing_tip.dll")
$ErrorActionPreference = 'Stop'
& (Join-Path $PSScriptRoot 'do_install.ps1') -PackageDirectory (Split-Path -Parent $DllPath) -NonInteractive
