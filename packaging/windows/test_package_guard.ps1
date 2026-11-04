# Headless tests: synthetic files only; never execute an IME or registration tool.
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'package_guard.ps1')
$testRoot = Join-Path ([IO.Path]::GetTempPath()) ('ckai-package-tests-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $testRoot | Out-Null
$script:passed = 0
$script:fixtureNumber = 0

function New-Fixture {
    $script:fixtureNumber += 1
    $dir = Join-Path $testRoot ([string]$script:fixtureNumber)
    New-Item -ItemType Directory -Path $dir | Out-Null
    $guid = [Guid]'{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}'
    $bytes = [byte[]](@(0x4d, 0x5a) + $guid.ToByteArray())
    $files = @{}
    foreach ($name in @('chewing_tip.dll', 'tsfreg.exe', 'chewing.ico', 'ChewingPreferences.exe', 'Dictionary/swkb.dat', 'Dictionary/symbols.dat', 'Dictionary/static_dict.bin', 'Dictionary/rare_dict.bin', 'Dictionary/static_words.bin', 'Dictionary/static_lm.bin')) {
        New-Item -ItemType Directory -Path (Split-Path -Parent (Join-Path $dir $name)) -Force | Out-Null
        [IO.File]::WriteAllBytes((Join-Path $dir $name), $bytes)
        $files[$name] = (Get-FileHash (Join-Path $dir $name)).Hash.ToLowerInvariant()
    }
    $manifest = @{
        schema_version = 1; product = 'AI-Chewing'; clsid = $guid.ToString('B').ToUpperInvariant()
        registry_namespace = 'Software\ChewingDev'; data_directory = 'ChewingDev'
        version = '0.1.0-dev.1'; source_commit = ('a' * 40); windows_source_commit = ('b' * 40)
        files = $files
    }
    return @{Directory = $dir; Manifest = $manifest}
}
function Save-Fixture($fixture) {
    $fixture.Manifest | ConvertTo-Json -Depth 6 | Set-Content -Encoding UTF8 (Join-Path $fixture.Directory 'package-manifest.json')
}
function Expect-Rejection($fixture, [string]$Reason) {
    Save-Fixture $fixture
    $rejected = $false
    try { $null = Assert-ChewingDevPackage -Directory $fixture.Directory }
    catch {
        if ($_.Exception.Message -notmatch $Reason) { throw }
        $rejected = $true
    }
    if (-not $rejected) { throw 'Unsafe package was unexpectedly accepted.' }
}
function Case([string]$Name, [scriptblock]$Body) {
    & $Body
    $script:passed += 1
    Write-Output "PASS: $Name"
}

try {
    Case 'Valid isolated synthetic package' {
        $f = New-Fixture; Save-Fixture $f
        $null = Assert-ChewingDevPackage -Directory $f.Directory
    }
    Case 'Missing manifest never reaches executable' {
        $f = New-Fixture; $rejected = $false
        try { $null = Assert-ChewingDevPackage -Directory $f.Directory }
        catch { if ($_.Exception.Message -notmatch 'Missing package-manifest') { throw }; $rejected = $true }
        if (-not $rejected) { throw 'Missing manifest accepted.' }
    }
    Case 'Official COM identity rejected' {
        $f = New-Fixture; $f.Manifest.clsid = '{13F2EF08-575C-4D8C-88E0-F67BB8052B84}'
        Expect-Rejection $f 'isolated ChewingDev identity'
    }
    Case 'Official preference namespace rejected' {
        $f = New-Fixture; $f.Manifest.registry_namespace = 'Software\ChewingTextService'
        Expect-Rejection $f 'isolated ChewingDev identity'
    }
    Case 'Official dictionary directory rejected' {
        $f = New-Fixture; $f.Manifest.data_directory = 'Chewing'
        Expect-Rejection $f 'isolated ChewingDev identity'
    }
    Case 'Parent traversal rejected' {
        $f = New-Fixture; $f.Manifest.files['../private.txt'] = ('a' * 64)
        Expect-Rejection $f 'Invalid payload path'
    }
    Case 'Absolute path rejected' {
        $f = New-Fixture; $f.Manifest.files['C:\private.txt'] = ('a' * 64)
        Expect-Rejection $f 'Invalid payload path'
    }
    Case 'Required file missing from manifest rejected' {
        $f = New-Fixture; $f.Manifest.files.Remove('tsfreg.exe')
        Expect-Rejection $f 'Missing required package file'
    }
    Case 'Missing runtime dictionary rejected' {
        $f = New-Fixture; $f.Manifest.files.Remove('Dictionary/static_dict.bin')
        Expect-Rejection $f 'Missing required package file'
    }
    Case 'Modified payload rejected' {
        $f = New-Fixture
        [IO.File]::WriteAllText((Join-Path $f.Directory 'chewing_tip.dll'), 'tampered')
        Expect-Rejection $f 'hash mismatch'
    }
    Case 'Official binary rejected even with correct hash and claimed dev identity' {
        $f = New-Fixture
        $path = Join-Path $f.Directory 'tsfreg.exe'
        $old = ([Guid]'{13F2EF08-575C-4D8C-88E0-F67BB8052B84}').ToByteArray()
        $bytes = [byte[]]([IO.File]::ReadAllBytes($path) + $old)
        [IO.File]::WriteAllBytes($path, $bytes)
        $f.Manifest.files['tsfreg.exe'] = (Get-FileHash $path).Hash.ToLowerInvariant()
        Expect-Rejection $f 'isolated COM identity'
    }
    Case 'Payload without isolated binary identity rejected' {
        $f = New-Fixture; $path = Join-Path $f.Directory 'chewing_tip.dll'
        [IO.File]::WriteAllBytes($path, [byte[]]@(0x4d, 0x5a, 0x00))
        $f.Manifest.files['chewing_tip.dll'] = (Get-FileHash $path).Hash.ToLowerInvariant()
        Expect-Rejection $f 'isolated COM identity'
    }
    Case 'Version path injection rejected' {
        $f = New-Fixture; $f.Manifest.version = '../official'
        Expect-Rejection $f 'version or source provenance'
    }
    Case 'Unpinned source rejected' {
        $f = New-Fixture; $f.Manifest.windows_source_commit = 'main'
        Expect-Rejection $f 'version or source provenance'
    }
    Case 'Bundled dictionary allowed without deploying to private data' {
        $f = New-Fixture
        New-Item -ItemType Directory (Join-Path $f.Directory 'Dictionary') -Force | Out-Null
        $path = Join-Path $f.Directory 'Dictionary/technical-terms.txt'
        [IO.File]::WriteAllText($path, 'synthetic')
        $f.Manifest.files['Dictionary/technical-terms.txt'] = (Get-FileHash $path).Hash.ToLowerInvariant()
        Save-Fixture $f; $null = Assert-ChewingDevPackage -Directory $f.Directory
    }
    Case 'ValidateOnly does not execute synthetic registration tool or install' {
        $f = New-Fixture; Save-Fixture $f
        $saved = $env:LOCALAPPDATA
        $env:LOCALAPPDATA = Join-Path $testRoot 'must-not-create'
        try {
            $null = & (Join-Path $PSScriptRoot 'do_install.ps1') -PackageDirectory $f.Directory -ValidateOnly
            if (Test-Path -LiteralPath $env:LOCALAPPDATA) { throw 'Validation mutated the installation.' }
        } finally { $env:LOCALAPPDATA = $saved }
    }
    Case 'All packaging scripts parse' {
        foreach ($file in Get-ChildItem $PSScriptRoot -Filter '*.ps1') {
            $parseErrors = $null; $tokens = $null
            $null = [Management.Automation.Language.Parser]::ParseFile($file.FullName, [ref]$tokens, [ref]$parseErrors)
            if ($parseErrors.Count -gt 0) { throw ($parseErrors | Out-String) }
        }
    }
    Write-Output "$script:passed package guard cases passed."
} finally {
    Remove-Item -LiteralPath $testRoot -Recurse -Force
}
