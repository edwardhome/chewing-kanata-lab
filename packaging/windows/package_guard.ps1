# This file performs read-only checks. Do not invoke an unverified tsfreg.exe:
# older builds treat unknown arguments as an instruction to unregister the IME.
Set-StrictMode -Version Latest

function Assert-ChewingDevPackage {
    param([Parameter(Mandatory=$true)][string]$Directory)

    $root = Get-Item -LiteralPath $Directory
    if (($root.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw 'Package directory must not be a link.'
    }
    $manifestPath = Join-Path $Directory 'package-manifest.json'
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
        throw 'Missing package-manifest.json. Legacy binaries are not safe to register.'
    }
    $manifest = Get-Content -LiteralPath $manifestPath -Raw -Encoding UTF8 | ConvertFrom-Json
    if ($manifest.schema_version -ne 1 -or $manifest.product -cne 'AI-Chewing' -or
        $manifest.clsid -ine '{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}' -or
        $manifest.registry_namespace -cne 'Software\ChewingDev' -or
        $manifest.data_directory -cne 'ChewingDev') {
        throw 'Package identity is not the isolated ChewingDev identity.'
    }
    if ($manifest.version -cnotmatch '^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$' -or
        $manifest.source_commit -cnotmatch '^[a-f0-9]{40}$' -or
        $manifest.windows_source_commit -cnotmatch '^[a-f0-9]{40}$') {
        throw 'Package version or source provenance is invalid.'
    }
    foreach ($required in @('chewing_tip.dll', 'tsfreg.exe', 'chewing.ico', 'ChewingPreferences.exe', 'Dictionary/swkb.dat', 'Dictionary/symbols.dat', 'Dictionary/static_dict.bin', 'Dictionary/rare_dict.bin', 'Dictionary/static_words.bin', 'Dictionary/static_lm.bin')) {
        if ($null -eq $manifest.files.PSObject.Properties[$required]) {
            throw "Missing required package file: $required"
        }
    }
    foreach ($entry in $manifest.files.PSObject.Properties) {
        # A package may contain payload files, never absolute paths or parent traversal.
        if ($entry.Name -cnotmatch '^(?:Dictionary/)?[A-Za-z0-9][A-Za-z0-9_.-]*$' -or
            $entry.Name -match '\.\.' -or $entry.Value -cnotmatch '^[a-f0-9]{64}$') {
            throw 'Invalid payload path or SHA-256 in package manifest.'
        }
        $path = Join-Path $Directory $entry.Name
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "Package file not found: $($entry.Name)"
        }
        $item = Get-Item -LiteralPath $path
        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw 'Package payload must not contain links.'
        }
        $parent = Get-Item -LiteralPath (Split-Path -Parent $path)
        if (($parent.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw 'Package payload directory must not be a link.'
        }
        if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -ine $entry.Value) {
            throw "Package hash mismatch: $($entry.Name)"
        }
    }
    # Inspect bytes without executing the registration tool. A manifest alone must
    # never make an old official-identity binary eligible for installation.
    $binaryEncoding = [Text.Encoding]::GetEncoding(28591)
    $devGuid = $binaryEncoding.GetString(([Guid]$manifest.clsid).ToByteArray())
    $officialGuid = $binaryEncoding.GetString(([Guid]'{13F2EF08-575C-4D8C-88E0-F67BB8052B84}').ToByteArray())
    foreach ($name in @('chewing_tip.dll', 'tsfreg.exe')) {
        $bytes = [IO.File]::ReadAllBytes((Join-Path $Directory $name))
        if ($bytes.Length -lt 2 -or $bytes[0] -ne 0x4d -or $bytes[1] -ne 0x5a) {
            throw "Not a Windows payload: $name"
        }
        $binary = $binaryEncoding.GetString($bytes)
        if (-not $binary.Contains($devGuid) -or $binary.Contains($officialGuid)) {
            throw "Payload does not exclusively use the isolated COM identity: $name"
        }
    }
    return $manifest
}

function Invoke-ChewingDevRegistration {
    param([string]$Executable, [string[]]$Arguments)
    & $Executable @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "TSF registration failed with exit code $LASTEXITCODE."
    }
}
