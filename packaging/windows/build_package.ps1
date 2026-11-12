# Build a portable, verifiable payload. This script never registers or installs it.
param(
    [Parameter(Mandatory=$true)][string]$NativeDirectory,
    [Parameter(Mandatory=$true)][string]$BinaryDirectory,
    [Parameter(Mandatory=$true)][string]$DictionaryDirectory,
    [Parameter(Mandatory=$true)][string]$PreferencesDirectory,
    [Parameter(Mandatory=$true)][string]$PreferencesExecutable,
    [Parameter(Mandatory=$true)][string]$OutputDirectory,
    [Parameter(Mandatory=$true)][ValidatePattern('^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$')][string]$Version
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'package_guard.ps1')
$sourceRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
function Get-CleanCommit([string]$Root) {
    $commit = & git -C $Root rev-parse HEAD
    if ($LASTEXITCODE -ne 0 -or $commit -cnotmatch '^[a-f0-9]{40}$') { throw 'Cannot determine source commit.' }
    $dirty = & git -C $Root status --porcelain --untracked-files=normal
    if ($LASTEXITCODE -ne 0 -or $dirty) { throw "Source must be committed before packaging: $Root" }
    return $commit
}
$sourceCommit = Get-CleanCommit $sourceRoot
$nativeCommit = Get-CleanCommit $NativeDirectory
$preferencesCommit = Get-CleanCommit $PreferencesDirectory
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Output directory must be new.' }
$payload = [ordered]@{}
foreach ($name in @('chewing_tip.dll', 'tsfreg.exe')) {
    $payload[$name] = Join-Path $BinaryDirectory $name
}
$payload['ChewingPreferences.exe'] = $PreferencesExecutable
foreach ($name in @('GPL-3.0-or-later.txt', '0BSD.txt', 'CC-BY-SA-4.0.txt')) {
    $payload['preferences-' + $name] = Join-Path $PreferencesDirectory ('LICENSES\' + $name)
}
$payload['preferences-REUSE.toml'] = Join-Path $PreferencesDirectory 'REUSE.toml'
$payload['chewing.ico'] = Join-Path $NativeDirectory 'tip\rc\im.chewing.Chewing.ico'
$payload['libchewing-data-CC-BY-4.0.txt'] = Join-Path $PSScriptRoot 'licenses\libchewing-data-CC-BY-4.0.txt'
$payload['libchewing-data-NOTICE.txt'] = Join-Path $PSScriptRoot 'licenses\libchewing-data-NOTICE.txt'
$payload['rusqlite-MIT.txt'] = Join-Path $PSScriptRoot 'licenses\rusqlite-MIT.txt'
$payload['COPYING.txt'] = Join-Path $NativeDirectory 'COPYING.txt'
$payload['AI-Bopomofo-LICENSE.txt'] = Join-Path $sourceRoot 'policy-core\resources\AI-Bopomofo-LICENSE.txt'
# Only the named public distribution dictionaries are eligible; never copy a
# user directory, private user dictionary, history, or environment-derived path.
foreach ($name in @('swkb.dat', 'symbols.dat', 'static_dict.bin', 'rare_dict.bin', 'static_words.bin', 'static_lm.bin')) {
    $payload['Dictionary/' + $name] = Join-Path $DictionaryDirectory $name
}
foreach ($name in @('package_guard.ps1', 'appcontainer_access.ps1', 'do_install.ps1', 'do_uninstall.ps1', 'install_ime.bat', 'uninstall_ime.bat')) {
    $payload[$name] = Join-Path $PSScriptRoot $name
}
foreach ($source in $payload.Values) {
    $item = Get-Item -LiteralPath $source
    if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw 'Payload source must be a regular file.'
    }
}
New-Item -ItemType Directory -Path $OutputDirectory | Out-Null
$hashes = [ordered]@{}
foreach ($entry in $payload.GetEnumerator()) {
    $destination = Join-Path $OutputDirectory $entry.Key
    New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force | Out-Null
    Copy-Item -LiteralPath $entry.Value -Destination $destination
    $hashes[$entry.Key] = (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash.ToLowerInvariant()
}
$versionPath = Join-Path $OutputDirectory 'version.json'
@{
    product_version = $Version
    build_date = [DateTimeOffset]::Now.ToOffset([TimeSpan]::FromHours(8)).ToString('yyyy-MM-ddTHH:mm:sszzz')
} | ConvertTo-Json | Set-Content -LiteralPath $versionPath -Encoding UTF8
$hashes['version.json'] = (Get-FileHash -LiteralPath $versionPath -Algorithm SHA256).Hash.ToLowerInvariant()
$manifest = [ordered]@{
    schema_version = 1
    product = 'AI-Chewing'
    version = $Version
    clsid = '{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}'
    registry_namespace = 'Software\ChewingDev'
    data_directory = 'ChewingDev'
    source_commit = $sourceCommit
    windows_source_commit = $nativeCommit
    preferences_source_commit = $preferencesCommit
    preferences_runtime = 'WebView2'
    files = $hashes
}
$manifest | ConvertTo-Json -Depth 5 | Set-Content -Encoding UTF8 -LiteralPath (Join-Path $OutputDirectory 'package-manifest.json')
$null = Assert-ChewingDevPackage -Directory $OutputDirectory
Write-Output ('Verified package created: ' + $OutputDirectory)
