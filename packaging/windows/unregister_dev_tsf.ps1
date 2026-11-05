# Compatibility entry point: preserve legacy and official registrations.
$ErrorActionPreference = 'Stop'
& (Join-Path $PSScriptRoot 'do_uninstall.ps1') -NonInteractive
