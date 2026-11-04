# Reproduce the reviewed native source tree without relying on commit timestamps.
param([switch]$Check, [switch]$Build)
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$native = Join-Path $repoRoot 'upstream\windows-chewing-tsf'
$patchDir = Join-Path $repoRoot 'docs\patches-windows-tsf'
$series = Get-Content -LiteralPath (Join-Path $patchDir 'series.json') -Raw -Encoding UTF8 | ConvertFrom-Json
if ($series.schema_version -ne 1 -or $series.baseline -cnotmatch '^[a-f0-9]{40}$' -or
    $series.tree -cnotmatch '^[a-f0-9]{40}$' -or @($series.patches).Count -eq 0) { throw 'Invalid source series.' }
function Get-GitOutput([string[]]$Arguments) {
    $result = & git -C $native @Arguments
    if ($LASTEXITCODE -ne 0) { throw 'Git command failed.' }
    return $result
}
$dirty = Get-GitOutput -Arguments @('status', '--porcelain', '--untracked-files=normal')
if ($dirty) { throw 'Native worktree has local changes; use a separate clean worktree.' }
foreach ($patch in $series.patches) {
    if ($patch.file -cnotmatch '^\d{4}-[A-Za-z0-9_.-]+\.patch$' -or $patch.sha256 -cnotmatch '^[a-f0-9]{64}$') { throw 'Invalid patch entry.' }
    if ((Get-FileHash -LiteralPath (Join-Path $patchDir $patch.file) -Algorithm SHA256).Hash -ine $patch.sha256) { throw 'Patch hash mismatch.' }
}
if (@(Get-ChildItem -LiteralPath $patchDir -Filter '*.patch').Count -ne @($series.patches).Count) { throw 'Unexpected patch files.' }
$head = Get-GitOutput -Arguments @('rev-parse', 'HEAD')
$tree = Get-GitOutput -Arguments @('rev-parse', 'HEAD^{tree}')
if ($tree -eq $series.tree) { Write-Output 'Native source tree matches the reviewed series.' }
elseif ($Check) { throw 'Native source does not match the reviewed series.' }
else {
    if ($head -ne $series.baseline) { throw 'Unknown native HEAD; start from the pinned baseline in a new worktree.' }
    $branch = Get-GitOutput -Arguments @('branch', '--show-current')
    if ($branch -notmatch '^(feature|fix|research|release)/') { throw 'Use an isolated development branch.' }
    foreach ($patch in $series.patches) {
        & git -C $native am --3way (Join-Path $patchDir $patch.file)
        if ($LASTEXITCODE -ne 0) { throw 'Patch application stopped. Inspect the conflict; git am --abort can restore the starting state.' }
    }
    if ((Get-GitOutput -Arguments @('rev-parse', 'HEAD^{tree}')) -ne $series.tree) { throw 'Reproduced source tree differs from the reviewed tree.' }
    Write-Output 'Patch series reproduced the reviewed native source tree.'
}
if ($Build -and -not $Check) {
    Push-Location $native
    try {
        & cargo check --workspace --locked
        if ($LASTEXITCODE -ne 0) { throw 'Native cargo check failed.' }
    } finally { Pop-Location }
}
