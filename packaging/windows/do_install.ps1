# AI新酷音：僅安裝經隔離身分稽核的使用者層級套件。
param(
    [switch]$NonInteractive,
    [string]$PackageDirectory = $PSScriptRoot,
    [switch]$ValidateOnly
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'package_guard.ps1')

# 必須在任何登錄、複製、程序操作之前完成。舊套件沒有 manifest，直接拒絕。
$manifest = Assert-ChewingDevPackage -Directory $PackageDirectory
if ($ValidateOnly) { Write-Output 'Package preflight passed; no changes made.'; return }

$baseDir = Join-Path $env:LOCALAPPDATA 'Programs\ChewingDev'
$targetDir = Join-Path $baseDir ('versions\' + $manifest.version + '-' + $manifest.windows_source_commit.Substring(0, 12))
$regBase = 'HKCU:\Software\Classes\CLSID\{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}'
$inproc = Join-Path $regBase 'InProcServer32'
$previousDll = $null
if (Test-Path -LiteralPath $inproc) { $previousDll = (Get-Item -LiteralPath $inproc).GetValue('') }

# 同版本檔案不可覆寫，保留仍被應用程式載入的舊 DLL。
if (Test-Path -LiteralPath $targetDir) {
    $installed = Assert-ChewingDevPackage -Directory $targetDir
    if ($installed.source_commit -cne $manifest.source_commit -or
        (Get-FileHash (Join-Path $targetDir 'package-manifest.json')).Hash -ne
        (Get-FileHash (Join-Path $PackageDirectory 'package-manifest.json')).Hash) {
        throw 'This version directory already contains a different package.'
    }
} else {
    $stagingDir = $targetDir + '.staging-' + [Guid]::NewGuid().ToString('N')
    New-Item -ItemType Directory -Path $stagingDir -Force | Out-Null
    try {
        foreach ($entry in $manifest.files.PSObject.Properties) {
            $destination = Join-Path $stagingDir $entry.Name
            New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force | Out-Null
            Copy-Item -LiteralPath (Join-Path $PackageDirectory $entry.Name) -Destination $destination
        }
        Copy-Item -LiteralPath (Join-Path $PackageDirectory 'package-manifest.json') -Destination $stagingDir
        $null = Assert-ChewingDevPackage -Directory $stagingDir
        Move-Item -LiteralPath $stagingDir -Destination $targetDir
    } finally {
        if (Test-Path -LiteralPath $stagingDir) { Remove-Item -LiteralPath $stagingDir -Recurse -Force }
    }
}

# 只寫獨立的 HKCU COM 身分，不修改正式新酷音、詞庫、設定或系統預設來源。
try {
    New-Item -Path $regBase -Force | Out-Null
    Set-Item -LiteralPath $regBase -Value 'AI新酷音'
    New-Item -Path $inproc -Force | Out-Null
    Set-Item -LiteralPath $inproc -Value (Join-Path $targetDir 'chewing_tip.dll')
    New-ItemProperty -LiteralPath $inproc -Name ThreadingModel -Value Apartment -PropertyType String -Force | Out-Null
    Invoke-ChewingDevRegistration -Executable (Join-Path $targetDir 'tsfreg.exe') -Arguments @('-r', (Join-Path $targetDir 'chewing.ico'))
    # 不呼叫 -i：保留使用者目前輸入來源，由 Win+Space 主動選擇。
    $pointerTemp = Join-Path $baseDir ('current-' + [Guid]::NewGuid().ToString('N') + '.tmp')
    Set-Content -LiteralPath $pointerTemp -Encoding UTF8 -Value (@{
        version_directory = $targetDir
        previous_dll = $previousDll
    } | ConvertTo-Json)
    Move-Item -LiteralPath $pointerTemp -Destination (Join-Path $baseDir 'current.json') -Force
} catch {
    $failure = $_
    if ($previousDll) {
        Set-Item -LiteralPath $inproc -Value $previousDll
    } else {
        # Registration can fail after partially creating the isolated TSF profile.
        try { Invoke-ChewingDevRegistration -Executable (Join-Path $targetDir 'tsfreg.exe') -Arguments @('-u') }
        catch { Write-Warning 'Unable to remove the incomplete ChewingDev profile.' }
        if (Test-Path -LiteralPath $regBase) { Remove-Item -LiteralPath $regBase -Recurse -Force }
    }
    throw $failure
}
Write-Output ('已註冊 AI新酷音 ' + $manifest.version + '。請自行用 Win+Space 切換；既有應用可能需要重新開啟。')
