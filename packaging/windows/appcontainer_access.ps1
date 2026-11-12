# Explicitly invoked by installation after package validation. No changes on import.
function Grant-ChewingPackageReadAccess {
    param([Parameter(Mandatory=$true)][string]$Directory)
    $manifest = Assert-ChewingDevPackage -Directory $Directory
    $root = (Get-Item -LiteralPath $Directory).FullName
    # Use non-inheriting rules on the exact public payload set. Unknown files,
    # private data, parent directories and registry security remain untouched.
    $paths = @($root)
    foreach ($entry in $manifest.files.PSObject.Properties) {
        $path = Join-Path $root $entry.Name
        $paths += $path
        $paths += Split-Path -Parent $path
    }
    $paths = @($paths | Sort-Object -Unique)
    # Complete validation before the first mutation (including intermediate paths).
    foreach ($path in $paths) {
        if ((Get-Item -LiteralPath $path).Attributes -band [IO.FileAttributes]::ReparsePoint) {
            throw 'AppContainer package access cannot follow links.'
        }
    }
    $sid = New-Object Security.Principal.SecurityIdentifier('S-1-15-2-1')
    $rule = New-Object Security.AccessControl.FileSystemAccessRule(
        $sid, [Security.AccessControl.FileSystemRights]::ReadAndExecute,
        [Security.AccessControl.InheritanceFlags]::None,
        [Security.AccessControl.PropagationFlags]::None,
        [Security.AccessControl.AccessControlType]::Allow)
    foreach ($path in $paths) {
        $acl = Get-Acl -LiteralPath $path
        $acl.AddAccessRule($rule)
        Set-Acl -LiteralPath $path -AclObject $acl
        $actual = (Get-Acl -LiteralPath $path).GetAccessRules($true, $false, [Security.Principal.SecurityIdentifier])
        $found = @($actual | Where-Object {
            $_.IdentityReference.Value -eq $sid.Value -and
            $_.AccessControlType -eq [Security.AccessControl.AccessControlType]::Allow -and
            ($_.FileSystemRights -band [Security.AccessControl.FileSystemRights]::ReadAndExecute) -eq [Security.AccessControl.FileSystemRights]::ReadAndExecute
        })
        if ($found.Count -eq 0) { throw 'Unable to verify package AppContainer read access.' }
    }
}
