# AI新酷音：僅安裝經隔離身分稽核的使用者層級套件。
param(
    [switch]$NonInteractive,
    [string]$PackageDirectory = $PSScriptRoot,
    [switch]$ValidateOnly,
    [string]$ExpectedUserSid
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'package_guard.ps1')
Assert-ChewingInstallUser $ExpectedUserSid

# 必須在任何登錄、複製、程序操作之前完成。舊套件沒有 manifest，直接拒絕。
$manifest = Assert-ChewingDevPackage -Directory $PackageDirectory
if ($ValidateOnly) { Write-Output 'Package preflight passed; no changes made.'; return }
if ($manifest.PSObject.Properties['preferences_runtime'] -and $manifest.preferences_runtime -eq 'WebView2') {
    $webView = @('HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients',
        'HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients',
        'HKCU:\Software\Microsoft\EdgeUpdate\Clients') | ForEach-Object {
        Get-ChildItem $_ -ErrorAction SilentlyContinue | Get-ItemProperty -ErrorAction SilentlyContinue
    } | Where-Object { $_.PSObject.Properties['name'] -and $_.PSObject.Properties['pv'] -and $_.name -like '*WebView2*' -and $_.pv -and $_.pv -ne '0.0.0.0' }
    if (-not $webView) { throw 'AI新酷音設定需要 Microsoft Edge WebView2 執行階段；請先安裝後再重試。尚未變更輸入法註冊。' }
}
if (Invoke-ChewingElevationIfNeeded -ScriptPath $PSCommandPath -Parameters @{PackageDirectory = (Resolve-Path -LiteralPath $PackageDirectory).Path}) { return }

$baseDir = Join-Path $env:LOCALAPPDATA 'Programs\ChewingDev'
$targetDir = Join-Path $baseDir ('versions\' + $manifest.version + '-' + $manifest.windows_source_commit.Substring(0, 12))
$regBase = 'HKCU:\Software\Classes\CLSID\{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}'
$inproc = Join-Path $regBase 'InProcServer32'
$progId = 'HKCU:\Software\Classes\AI-Chewing.TextService'
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\AI-Chewing'
$previousUninstall = @{}
$hadUninstallEntry = Test-Path -LiteralPath $uninstallKey
if ($hadUninstallEntry) {
    $key = Get-Item -LiteralPath $uninstallKey
    foreach ($name in $key.GetValueNames()) {
        $previousUninstall[$name] = @{Value = $key.GetValue($name); Kind = $key.GetValueKind($name).ToString()}
    }
}
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

# SearchHost/AppContainer must be able to load the DLL and public dictionaries.
# Grant only the verified package; never relax private dictionaries or parent folders.
. (Join-Path $PSScriptRoot 'appcontainer_access.ps1')
Grant-ChewingPackageReadAccess -Directory $targetDir

# 只寫獨立的 HKCU COM 身分，不修改正式新酷音、詞庫、設定或系統預設來源。
try {
    New-Item -Path $regBase -Force | Out-Null
    Set-Item -LiteralPath $regBase -Value 'AI新酷音'
    New-Item -Path (Join-Path $regBase 'ProgID') -Force | Out-Null
    Set-Item -LiteralPath (Join-Path $regBase 'ProgID') -Value 'AI-Chewing.TextService'
    New-Item -Path (Join-Path $progId 'CLSID') -Force | Out-Null
    Set-Item -LiteralPath (Join-Path $progId 'CLSID') -Value '{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}'
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
    # Expose a supported removal entry in Windows Installed apps. Do not rely
    # on the keyboard list to remove a registered third-party TSF service.
    New-Item -Path $uninstallKey -Force | Out-Null
    $uninstallCommand = '"' + (Join-Path $PSHOME 'powershell.exe') + '" -NoProfile -ExecutionPolicy Bypass -File "' + (Join-Path $targetDir 'do_uninstall.ps1') + '"'
    foreach ($entry in @{
        DisplayName = 'AI新酷音'; DisplayVersion = $manifest.version
        Publisher = 'AI-Chewing contributors'; InstallLocation = $targetDir
        DisplayIcon = (Join-Path $targetDir 'chewing.ico'); UninstallString = $uninstallCommand
    }.GetEnumerator()) {
        New-ItemProperty -LiteralPath $uninstallKey -Name $entry.Key -Value $entry.Value -PropertyType String -Force | Out-Null
    }
    foreach ($name in @('NoModify', 'NoRepair')) {
        New-ItemProperty -LiteralPath $uninstallKey -Name $name -Value 1 -PropertyType DWord -Force | Out-Null
    }
    Move-Item -LiteralPath $pointerTemp -Destination (Join-Path $baseDir 'current.json') -Force
} catch {
    $failure = $_
    if (Test-Path -LiteralPath $uninstallKey) { Remove-Item -LiteralPath $uninstallKey -Recurse -Force }
    if ($hadUninstallEntry) {
        New-Item -Path $uninstallKey -Force | Out-Null
        foreach ($entry in $previousUninstall.GetEnumerator()) {
            New-ItemProperty -LiteralPath $uninstallKey -Name $entry.Key -Value $entry.Value.Value -PropertyType $entry.Value.Kind -Force | Out-Null
        }
    }
    if ($previousDll) {
        Set-Item -LiteralPath $inproc -Value $previousDll
    } else {
        # Registration can fail after partially creating the isolated TSF profile.
        try { Invoke-ChewingDevRegistration -Executable (Join-Path $targetDir 'tsfreg.exe') -Arguments @('-u') }
        catch { Write-Warning 'Unable to remove the incomplete ChewingDev profile.' }
        if (Test-Path -LiteralPath $regBase) { Remove-Item -LiteralPath $regBase -Recurse -Force }
        if (Test-Path -LiteralPath $progId) { Remove-Item -LiteralPath $progId -Recurse -Force }
    }
    throw $failure
}
Write-Output ('已註冊 AI新酷音 ' + $manifest.version + '。請自行用 Win+Space 切換；既有應用可能需要重新開啟。')
