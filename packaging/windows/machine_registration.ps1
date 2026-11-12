# Installation helpers. Importing this file has no side effects.
Set-StrictMode -Version Latest

function Get-ChewingMachineRoot {
    if (-not [Environment]::Is64BitProcess) { throw 'Use 64-bit PowerShell for the x64 package. No registration changes made.' }
    $programFiles = [Environment]::GetFolderPath([Environment+SpecialFolder]::ProgramFiles)
    if (-not $programFiles) { throw 'Cannot locate the protected Program Files directory.' }
    Join-Path $programFiles 'ChewingDev'
}

function Assert-ChewingNoLinks([string]$Path) {
    $candidate = [IO.Path]::GetFullPath($Path)
    while ($candidate) {
        if (Test-Path -LiteralPath $candidate) {
            if ((Get-Item -LiteralPath $candidate -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) {
                throw 'Machine payload paths must not traverse links.'
            }
        }
        $parent = Split-Path -Parent $candidate
        if ($parent -eq $candidate) { break }
        $candidate = $parent
    }
}

function Assert-ChewingTrustedAcl([Security.AccessControl.FileSystemSecurity]$Acl) {
    $trusted = @('S-1-5-18', 'S-1-5-32-544', 'S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464')
    if ($Acl.GetOwner([Security.Principal.SecurityIdentifier]).Value -notin $trusted) {
        throw 'Machine payload must have a trusted system/administrator owner.'
    }
    $writeMask = [Security.AccessControl.FileSystemRights]::Write -bor
        [Security.AccessControl.FileSystemRights]::Delete -bor
        [Security.AccessControl.FileSystemRights]::DeleteSubdirectoriesAndFiles -bor
        [Security.AccessControl.FileSystemRights]::ChangePermissions -bor
        [Security.AccessControl.FileSystemRights]::TakeOwnership
    foreach ($rule in $Acl.GetAccessRules($true, $true, [Security.Principal.SecurityIdentifier])) {
        if ($rule.AccessControlType -eq 'Allow' -and
            ($rule.FileSystemRights -band $writeMask) -ne 0 -and
            $rule.IdentityReference.Value -notin $trusted) {
            throw 'Machine payload permits a non-administrator to modify executable content.'
        }
    }
}

function Initialize-ChewingMachineRoot {
    $root = Get-ChewingMachineRoot
    Assert-ChewingNoLinks $root
    if (-not (Test-Path -LiteralPath $root)) {
        # A new, product-specific directory only; never change Program Files itself.
        $null = New-Item -ItemType Directory -Path $root
        $acl = New-Object Security.AccessControl.DirectorySecurity
        $acl.SetSecurityDescriptorSddlForm('O:BAG:BAD:P(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)(A;OICI;0x1200a9;;;BU)(A;OICI;0x1200a9;;;AC)')
        Set-Acl -LiteralPath $root -AclObject $acl
    }
    Assert-ChewingTrustedAcl (Get-Acl -LiteralPath $root)
    $versions = Join-Path $root 'versions'
    Assert-ChewingNoLinks $versions
    if (-not (Test-Path -LiteralPath $versions)) {
        $null = New-Item -ItemType Directory -Path $versions
    }
    # Validate an existing parent before copying privileged executable content.
    Assert-ChewingTrustedAcl (Get-Acl -LiteralPath $versions)
    return $root
}

function Assert-ChewingMachinePackage([string]$Directory) {
    $root = [IO.Path]::GetFullPath((Get-ChewingMachineRoot))
    $full = [IO.Path]::GetFullPath($Directory)
    $versions = Join-Path $root 'versions'
    if (-not $full.StartsWith($versions + '\', [StringComparison]::OrdinalIgnoreCase) -or
        (Split-Path -Parent $full) -ine $versions) {
        throw 'Machine COM DLL must be in a protected ChewingDev version directory.'
    }
    Assert-ChewingNoLinks $full
    $manifest = Assert-ChewingDevPackage -Directory $full
    # Include the manifest, directory chain and every public file. No private paths.
    $paths = @($root, $versions, $full, (Join-Path $full 'package-manifest.json'))
    foreach ($entry in $manifest.files.PSObject.Properties) {
        $path = Join-Path $full $entry.Name
        $paths += $path
        $paths += Split-Path -Parent $path
    }
    foreach ($path in @($paths | Sort-Object -Unique)) {
        Assert-ChewingTrustedAcl (Get-Acl -LiteralPath $path)
    }
    return $manifest
}

function Get-ChewingRegistrySnapshot([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) { return @{Exists=$false; Values=@(); Children=@{}} }
    $key = Get-Item -LiteralPath $Path
    $values = @()
    foreach ($name in $key.GetValueNames()) {
        $values += @{Name=$name; Value=$key.GetValue($name, $null, [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames); Kind=$key.GetValueKind($name).ToString()}
    }
    $children = @{}
    foreach ($name in $key.GetSubKeyNames()) { $children[$name] = Get-ChewingRegistrySnapshot (Join-Path $Path $name) }
    return @{Exists=$true; Values=$values; Children=$children}
}

function Restore-ChewingRegistrySnapshot([string]$Path, $Snapshot) {
    # Callers supply only the isolated product keys snapshotted before mutation.
    if (-not $Snapshot.Exists) {
        if (Test-Path -LiteralPath $Path) { Remove-Item -LiteralPath $Path -Recurse -Force }
        return
    }
    # Preserve the ACLs of pre-existing keys by restoring their values in place.
    if (-not (Test-Path -LiteralPath $Path)) { $null = New-Item -Path $Path -Force }
    $key = (Get-Item -LiteralPath $Path).OpenSubKey('', $true)
    try {
        foreach ($name in $key.GetValueNames()) {
            if ($name -notin @($Snapshot.Values | ForEach-Object { $_.Name })) { $key.DeleteValue($name) }
        }
        foreach ($value in $Snapshot.Values) { $key.SetValue($value.Name, $value.Value, [Microsoft.Win32.RegistryValueKind]$value.Kind) }
        foreach ($name in $key.GetSubKeyNames()) {
            if (-not $Snapshot.Children.ContainsKey($name)) { Remove-Item -LiteralPath (Join-Path $Path $name) -Recurse -Force }
        }
    } finally {
        $key.Dispose()
    }
    foreach ($name in $Snapshot.Children.Keys) { Restore-ChewingRegistrySnapshot (Join-Path $Path $name) $Snapshot.Children[$name] }
}

function Assert-ChewingMachineRegistrationOwner([string]$ClassPath, [string]$OwnerSid) {
    if (-not (Test-Path -LiteralPath $ClassPath)) { return }
    $key = Get-Item -LiteralPath $ClassPath
    if ([string]$key.GetValue('ChewingDevOwnerSid') -cne $OwnerSid) {
        throw 'Machine registration is not owned by this installation account; no changes made.'
    }
}

function Set-ChewingComClass([string]$ClassPath, [string]$Dll, [string]$OwnerSid) {
    if (-not (Test-Path -LiteralPath $ClassPath)) { $null = New-Item -Path $ClassPath -Force }
    Set-Item -LiteralPath $ClassPath -Value 'AI新酷音'
    if ($OwnerSid) {
        $null = New-ItemProperty -LiteralPath $ClassPath -Name ChewingDevOwnerSid -Value $OwnerSid -PropertyType String -Force
    }
    $inproc = Join-Path $ClassPath 'InProcServer32'
    if (-not (Test-Path -LiteralPath $inproc)) { $null = New-Item -Path $inproc -Force }
    Set-Item -LiteralPath $inproc -Value $Dll
    $null = New-ItemProperty -LiteralPath $inproc -Name ThreadingModel -Value Apartment -PropertyType String -Force
}

function Set-ChewingComClasses([string]$UserClass, [string]$MachineClass, [string]$Directory, [string]$OwnerSid) {
    $null = Assert-ChewingMachinePackage $Directory
    Assert-ChewingMachineRegistrationOwner $MachineClass $OwnerSid
    $oldUser = Get-ChewingRegistrySnapshot $UserClass
    $oldMachine = Get-ChewingRegistrySnapshot $MachineClass
    $dll = Join-Path $Directory 'chewing_tip.dll'
    try {
        Set-ChewingComClass $MachineClass $dll $OwnerSid
        Set-ChewingComClass $UserClass $dll ''
    } catch {
        $failure = $_
        Restore-ChewingRegistrySnapshot $UserClass $oldUser
        Restore-ChewingRegistrySnapshot $MachineClass $oldMachine
        throw $failure
    }
}
