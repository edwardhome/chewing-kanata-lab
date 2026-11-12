# Read-only diagnostic: never activates, registers or restarts an input method.
# No typed text, dictionaries, credentials or complete event messages are read.
param([string]$OutputPath = (Join-Path $PSScriptRoot ('search-diagnostic-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '.json')))
$ErrorActionPreference = 'Stop'
function PublicPath([string]$Path) {
    if (!$Path) { return $null }
    if ($env:USERPROFILE) { $Path = $Path.Replace($env:USERPROFILE, '%USERPROFILE%') }
    return $Path
}
function FileSummary([string]$Path) {
    if (!(Test-Path -LiteralPath $Path -PathType Leaf)) { return @{ present=$false; path=(PublicPath $Path) } }
    $f = Get-Item -LiteralPath $Path
    $sig = Get-AuthenticodeSignature -LiteralPath $Path
    $rules = (Get-Acl -LiteralPath $Path).GetAccessRules($true,$true,[Security.Principal.SecurityIdentifier])
    $appRules = @($rules | Where-Object {$_.IdentityReference.Value -in @('S-1-15-2-1','S-1-15-2-2')} | ForEach-Object {
        @{ sid=$_.IdentityReference.Value; type=$_.AccessControlType.ToString(); rights=$_.FileSystemRights.ToString() }
    })
    return @{ present=$true; path=(PublicPath $Path); version=$f.VersionInfo.ProductVersion; sha256=(Get-FileHash -LiteralPath $Path).Hash; signature=$sig.Status.ToString(); appContainerRules=$appRules }
}
$r = [ordered]@{ schema=1; time=[DateTimeOffset]::Now.ToString('o'); purpose='Start/Search IME activation diagnosis; no input text collected'; errors=@() }
$os = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
$r.os = @{ build=$os.CurrentBuildNumber; revision=$os.UBR; displayVersion=$os.DisplayVersion; process64bit=[Environment]::Is64BitProcess }
$clsid='{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}'
$r.registrations=@()
foreach($view in @([Microsoft.Win32.RegistryView]::Registry64,[Microsoft.Win32.RegistryView]::Registry32)) {
    $hive=$null;$key=$null
    try {
        $hive=[Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::CurrentUser,$view)
        $key=$hive.OpenSubKey("Software\Classes\CLSID\$clsid\InProcServer32")
        $dll=if($key){[string]$key.GetValue('')}else{''}
        $entry=@{ view=$view.ToString(); present=!!$key }
        if($dll){$entry.dll=FileSummary $dll}
        $r.registrations+=,$entry
    } catch { $r.errors+=('registration:'+$view+':'+$_.Exception.GetType().Name) }
    finally {if($key){$key.Dispose()};if($hive){$hive.Dispose()}}
}
$r.hosts=@()
foreach($p in @(Get-Process SearchHost,StartMenuExperienceHost,ctfmon -ErrorAction SilentlyContinue)) {
    $entry=@{name=$p.ProcessName;pid=$p.Id;moduleStatus='unavailable';chewingModules=@()}
    try {
        $entry.chewingModules=@($p.Modules | Where-Object {$_.ModuleName -eq 'chewing_tip.dll'} | ForEach-Object { PublicPath $_.FileName })
        $entry.moduleStatus='queried'
    } catch {$entry.moduleStatus=$_.Exception.GetType().Name}
    $r.hosts+=,$entry
}
# Query only crash metadata for known hosts. Omit user data and event message bodies.
$r.crashes=@()
try {
    $events=Get-WinEvent -FilterHashtable @{LogName='Application';Id=1000;StartTime=(Get-Date).AddDays(-2)} -MaxEvents 200 -ErrorAction Stop
    foreach($event in $events) {
        [xml]$xml=$event.ToXml();$fields=@{}
        foreach($d in $xml.Event.EventData.Data){$fields[[string]$d.Name]=[string]$d.'#text'}
        if($fields.AppName -in @('SearchHost.exe','StartMenuExperienceHost.exe','ctfmon.exe') -or $fields.ModuleName -eq 'chewing_tip.dll') {
            $r.crashes+=@{time=$event.TimeCreated.ToString('o');app=$fields.AppName;module=$fields.ModuleName;exceptionCode=$fields.ExceptionCode;appVersion=$fields.AppVersion;moduleVersion=$fields.ModuleVersion}
        }
    }
} catch { $r.errors+=('crash-query:'+$_.FullyQualifiedErrorId.Split(',')[0]) }
$json=$r | ConvertTo-Json -Depth 8
# CreateNew prevents replacing an existing report or unrelated file.
$stream=[IO.File]::Open($OutputPath,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write)
try {$bytes=[Text.UTF8Encoding]::new($false).GetBytes($json);$stream.Write($bytes,0,$bytes.Length)} finally {$stream.Dispose()}
Write-Output ('Diagnostic saved: '+(PublicPath $OutputPath))
