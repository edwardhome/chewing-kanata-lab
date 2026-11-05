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
    if ($manifest.PSObject.Properties['preferences_source_commit'] -or $manifest.PSObject.Properties['preferences_runtime']) {
        if (-not $manifest.PSObject.Properties['preferences_source_commit'] -or
            -not $manifest.PSObject.Properties['preferences_runtime'] -or
            $manifest.preferences_source_commit -cnotmatch '^[a-f0-9]{40}$' -or
            $manifest.preferences_runtime -cne 'WebView2') {
            throw 'Preferences provenance or runtime is invalid.'
        }
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
    # PowerShell can return immediately for a Windows-subsystem executable.
    # Wait for the actual process and read its exit code, not stale LASTEXITCODE.
    $quoted = foreach ($argument in $Arguments) {
        '"' + ($argument -replace '(\\*)"', '$1$1\"' -replace '(\\+)$', '$1$1') + '"'
    }
    $process = New-Object System.Diagnostics.Process
    $process.StartInfo.FileName = $Executable
    $process.StartInfo.Arguments = $quoted -join ' '
    $process.StartInfo.UseShellExecute = $false
    $process.StartInfo.CreateNoWindow = $true
    try {
        if (-not $process.Start()) { throw 'Unable to start TSF registration.' }
        $process.WaitForExit()
        if ($process.ExitCode -ne 0) {
            throw "TSF registration failed with exit code $($process.ExitCode)."
        }
    } finally {
        $process.Dispose()
    }
}

# TSF profile registration is machine-level even though the COM server and
# payload are per-user. Keep elevation in the same user account.
function Assert-ChewingInstallUser([string]$ExpectedUserSid) {
    if ($ExpectedUserSid -and [Security.Principal.WindowsIdentity]::GetCurrent().User.Value -cne $ExpectedUserSid) {
        throw 'Please approve elevation with the same Windows account that owns this installation.'
    }
}

function Invoke-ChewingElevationIfNeeded([string]$ScriptPath, [hashtable]$Parameters = @{}) {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    if ($principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { return $false }
    $Parameters['ExpectedUserSid'] = $identity.User.Value
    $command = "& '" + $ScriptPath.Replace("'", "''") + "'"
    foreach ($entry in $Parameters.GetEnumerator()) {
        if ($entry.Key -notmatch '^[A-Za-z]+$') { throw 'Invalid elevation parameter.' }
        $command += " -" + $entry.Key + " '" + ([string]$entry.Value).Replace("'", "''") + "'"
    }
    $command = "try { `$ErrorActionPreference='Stop'; " + $command + "; exit 0 } catch { Write-Error `$_; exit 1 }"
    $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($command))
    $process = Start-Process -FilePath (Join-Path $PSHOME 'powershell.exe') -Verb RunAs -Wait -PassThru -ArgumentList @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-EncodedCommand', $encoded)
    if ($process.ExitCode -ne 0) { throw 'The elevated install or uninstall did not complete.' }
    return $true
}
