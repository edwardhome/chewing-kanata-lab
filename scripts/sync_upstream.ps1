# Sync Upstream Repositories to Exact Locked Commits

$repos = @(
    @{
        Name = "libchewing"
        Primary = "https://codeberg.org/chewing/libchewing.git"
        Mirror = "https://github.com/chewing/libchewing.git"
        Commit = "ec7f595ea7726fa06a9493e819b81669eba605a5"
    },
    @{
        Name = "windows-chewing-tsf"
        Primary = "https://codeberg.org/chewing/windows-chewing-tsf.git"
        Mirror = "https://github.com/chewing/windows-chewing-tsf.git"
        Commit = "053d31ba10c089ae98508dd4a908ac2c2be3cb49"
    },
    @{
        Name = "fcitx5-chewing"
        Primary = "https://github.com/fcitx/fcitx5-chewing.git"
        Mirror = ""
        Commit = "60096e8ef49df21de6c155d7a0785a58fcccb955"
    },
    @{
        Name = "ibus-chewing"
        Primary = "https://codeberg.org/chewing/ibus-chewing.git"
        Mirror = "https://github.com/chewing/ibus-chewing.git"
        Commit = "6e885f1a8556cb8cfcc657c5b0cfd71b93db4c5c"
    }
)

$upstreamDir = Join-Path $PSScriptRoot "..\upstream"
if (-not (Test-Path $upstreamDir)) {
    New-Item -ItemType Directory -Path $upstreamDir -Force | Out-Null
}

foreach ($r in $repos) {
    $target = Join-Path $upstreamDir $r.Name
    Write-Host "Syncing $($r.Name)..."
    if (-not (Test-Path $target)) {
        git clone $r.Primary $target
        if ($LASTEXITCODE -ne 0 -and $r.Mirror -ne "") {
            Write-Host "Primary clone failed. Trying mirror $($r.Mirror)..."
            git clone $r.Mirror $target
        }
    }
    if (Test-Path $target) {
        Push-Location $target
        git checkout $r.Commit
        Pop-Location
    }
}

Write-Host "Upstream sync completed."
