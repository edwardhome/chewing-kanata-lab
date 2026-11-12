# Tests use fresh HKCU test keys and temporary/in-memory ACLs. No HKLM writes,
# production COM/TSF registration, IME execution or application restarts.
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'machine_registration.ps1')
$testId = [Guid]::NewGuid().ToString('N')
$testKey = 'HKCU:\Software\ChewingDev-PackagingTest-' + $testId
$testDir = Join-Path ([IO.Path]::GetTempPath()) ('ckai-machine-tests-' + $testId)
$owner = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$script:passed = 0
function Check($Condition, [string]$Message) { if (-not $Condition) { throw $Message } }
function Case([string]$Name, [scriptblock]$Body) {
    & $Body
    $script:passed++
    Write-Output ('PASS: ' + $Name)
}
function Reject([scriptblock]$Body, [string]$Pattern) {
    try { & $Body } catch {
        if ($_.Exception.Message -notmatch $Pattern) { throw }
        return
    }
    throw 'Expected rejection did not occur.'
}
function Test-Acl([string]$Sddl) {
    $acl = New-Object Security.AccessControl.DirectorySecurity
    $acl.SetSecurityDescriptorSddlForm($Sddl)
    return $acl
}
try {
    $null = New-Item -Path $testKey -Force
    $null = New-Item -ItemType Directory -Path $testDir
    Case 'Protected public ACL allows only administrators/system to write' {
        Assert-ChewingTrustedAcl (Test-Acl 'O:BAG:BAD:P(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)(A;OICI;0x1200a9;;;BU)(A;OICI;0x1200a9;;;AC)')
    }
    Case 'User-writable machine DLL is rejected' {
        Reject { Assert-ChewingTrustedAcl (Test-Acl 'O:BAG:BAD:P(A;OICI;FA;;;BA)(A;OICI;FA;;;BU)') } 'non-administrator'
    }
    Case 'User-owned machine payload is rejected even if its DACL is read-only' {
        Reject { Assert-ChewingTrustedAcl (Test-Acl ('O:'+$owner+'G:BAD:P(A;OICI;0x1200a9;;;BU)')) } 'trusted'
    }
    Case 'Creator-owner writable inheritance is rejected' {
        Reject { Assert-ChewingTrustedAcl (Test-Acl 'O:BAG:BAD:P(A;OICI;FA;;;BA)(A;OICIIO;FA;;;CO)') } 'non-administrator'
    }
    Case 'Machine payload cannot point into a user profile or a prefix sibling' {
        Reject { Assert-ChewingMachinePackage $testDir } 'protected ChewingDev version'
        Reject { Assert-ChewingMachinePackage ((Get-ChewingMachineRoot)+'-other\versions\test') } 'protected ChewingDev version'
    }
    Case 'Intermediate directory junction is rejected before any file access' {
        $real = Join-Path $testDir 'real'
        $link = Join-Path $testDir 'link'
        $null = New-Item -ItemType Directory -Path $real
        $null = New-Item -ItemType Junction -Path $link -Target $real
        try { Reject { Assert-ChewingNoLinks (Join-Path $link 'missing\file.dll') } 'links' }
        finally { Remove-Item -LiteralPath $link -Force }
    }
    Case 'Snapshot rollback preserves values, kinds, subkeys and pre-existing ACLs' {
        $path = Join-Path $testKey 'Snapshot'
        $null = New-Item -Path (Join-Path $path 'Child') -Force
        $key = (Get-Item -LiteralPath $path).OpenSubKey('', $true)
        $key.SetValue('', 'old.dll', [Microsoft.Win32.RegistryValueKind]::String)
        $key.SetValue('Expand', '%LOCALAPPDATA%\unchanged', [Microsoft.Win32.RegistryValueKind]::ExpandString)
        $key.SetValue('Binary', [byte[]](1,2,3), [Microsoft.Win32.RegistryValueKind]::Binary)
        $key.SetValue('Multi', [string[]]('a','b'), [Microsoft.Win32.RegistryValueKind]::MultiString)
        $oldAcl = (Get-Acl -Path $path).Sddl
        $snapshot = Get-ChewingRegistrySnapshot $path
        $key.SetValue('', 'changed')
        $key.SetValue('Added', 17)
        $null = New-Item -Path (Join-Path $path 'AddedChild')
        $key.Dispose()
        Restore-ChewingRegistrySnapshot $path $snapshot
        $key = Get-Item -LiteralPath $path
        Check ($key.GetValue('') -ceq 'old.dll') 'Default value not restored'
        Check ($key.GetValueKind('Expand') -eq 'ExpandString') 'Value kind changed'
        Check ($key.GetValue('Expand',$null,[Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames) -ceq '%LOCALAPPDATA%\unchanged') 'Expanded value lost'
        Check (($key.GetValue('Binary') -join ',') -ceq '1,2,3') 'Binary value changed'
        Check (($key.GetValue('Multi') -join ',') -ceq 'a,b') 'Multi-string changed'
        Check ($key.GetValue('Added') -eq $null) 'New value survived rollback'
        Check (-not (Test-Path -LiteralPath (Join-Path $path 'AddedChild'))) 'New child survived rollback'
        Check (Test-Path -LiteralPath (Join-Path $path 'Child')) 'Old child lost'
        Check ((Get-Acl -Path $path).Sddl -ceq $oldAcl) 'Registry ACL changed'
    }
    # Only the protected-filesystem precondition is stubbed below; registry
    # operations run against isolated real HKCU keys representing the two views.
    function Assert-ChewingMachinePackage([string]$Directory) { return @{} }
    $userClass = Join-Path $testKey 'UserClass'
    $machineClass = Join-Path $testKey 'MachineClass'
    $directory = Join-Path $testDir 'public-version'
    Case 'Both COM views publish the same DLL and Apartment threading model' {
        Set-ChewingComClasses $userClass $machineClass $directory $owner
        foreach ($path in @($userClass,$machineClass)) {
            $key = Get-Item -LiteralPath (Join-Path $path 'InProcServer32')
            Check ($key.GetValue('') -ceq (Join-Path $directory 'chewing_tip.dll')) 'DLL mismatch'
            Check ($key.GetValue('ThreadingModel') -ceq 'Apartment') 'Threading model mismatch'
        }
        Check ((Get-Item -LiteralPath $machineClass).GetValue('ChewingDevOwnerSid') -ceq $owner) 'Missing ownership marker'
        Check ((Get-Item -LiteralPath $userClass).GetValue('ChewingDevOwnerSid') -eq $null) 'Unexpected user marker'
    }
    Case 'Existing machine registration owned by another account is not overwritten' {
        Reject { Set-ChewingComClasses $userClass $machineClass 'C:\unused' 'S-1-0-0' } 'not owned'
        Check ((Get-Item -LiteralPath (Join-Path $machineClass 'InProcServer32')).GetValue('') -ceq (Join-Path $directory 'chewing_tip.dll')) 'Foreign-owner rejection changed DLL'
    }
    Case 'Repeated registration preserves pre-existing properties, child keys and ACLs' {
        $child = Join-Path $machineClass 'PreservedChild'
        $null = New-Item -Path $child
        $null = New-ItemProperty -LiteralPath $machineClass -Name PreservedValue -Value retained -PropertyType String
        $oldAcl = (Get-Acl -Path $child).Sddl
        Set-ChewingComClasses $userClass $machineClass $directory $owner
        Check ((Get-Item -LiteralPath $machineClass).GetValue('PreservedValue') -ceq 'retained') 'Existing property was replaced'
        Check (Test-Path -LiteralPath $child) 'Existing child key was replaced'
        Check ((Get-Acl -Path $child).Sddl -ceq $oldAcl) 'Existing child ACL changed'
    }
    Case 'Unmarked pre-existing machine class is not adopted' {
        $path = Join-Path $testKey 'Unmarked'
        $null = New-Item -Path $path
        Reject { Set-ChewingComClasses $userClass $path $directory $owner } 'not owned'
    }
    $originalWriter = (Get-Item -LiteralPath Function:\Set-ChewingComClass).ScriptBlock
    function Set-ChewingComClass([string]$ClassPath, [string]$Dll, [string]$OwnerSid) {
        if ($ClassPath -eq $userClass) { throw 'Injected second-view write failure' }
        & $originalWriter $ClassPath $Dll $OwnerSid
    }
    Case 'Second-view failure restores an existing machine DLL and user DLL' {
        Reject { Set-ChewingComClasses $userClass $machineClass 'C:\replacement' $owner } 'Injected'
        foreach ($path in @($userClass,$machineClass)) {
            Check ((Get-Item -LiteralPath (Join-Path $path 'InProcServer32')).GetValue('') -ceq (Join-Path $directory 'chewing_tip.dll')) 'Rollback changed previous registration'
        }
    }
    Case 'Second-view failure removes a newly created machine class' {
        $newMachine = Join-Path $testKey 'NewMachine'
        Reject { Set-ChewingComClasses $userClass $newMachine $directory $owner } 'Injected'
        Check (-not (Test-Path -LiteralPath $newMachine)) 'New machine class survived failed install'
    }
    Write-Output "$script:passed machine registration cases passed."
} finally {
    if ($testKey -cne ('HKCU:\Software\ChewingDev-PackagingTest-' + $testId)) { throw 'Unsafe registry cleanup path' }
    if (Test-Path -LiteralPath $testKey) { Remove-Item -LiteralPath $testKey -Recurse -Force }
    $full = [IO.Path]::GetFullPath($testDir)
    $expected = [IO.Path]::GetFullPath((Join-Path ([IO.Path]::GetTempPath()) ('ckai-machine-tests-' + $testId)))
    if ($full -cne $expected) { throw 'Unsafe test cleanup path' }
    if (Test-Path -LiteralPath $full) { Remove-Item -LiteralPath $full -Recurse -Force }
}
