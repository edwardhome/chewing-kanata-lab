#!/usr/bin/env pwsh
# apply_patches.ps1 — 將 docs/patches-windows-tsf/ 套用至 upstream/windows-chewing-tsf
# 用法: .\scripts\apply_patches.ps1 [-Check] [-BaselineSha <sha>]
#
# CKAI-AGENTS-001 合規:
#   - 不安裝、不 push、不覆寫使用者配置
#   - 失敗時中止並提示 git am --abort
#
# 預期基準 commit（上游最後一個乾淨 commit）：
param(
    [switch]$Check,
    [string]$BaselineSha = "053d31ba10c089ae98508dd4a908ac2c2be3cb49",
    [string]$ExpectedHead = "2232ff9b91efae8247842eb0d34b8880c30d8e50"
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$upstreamPath = Join-Path $repoRoot "upstream\windows-chewing-tsf"
$patchDir = Join-Path $repoRoot "docs\patches-windows-tsf"

# 1. 確認 upstream 目錄存在
if (-not (Test-Path (Join-Path $upstreamPath ".git"))) {
    Write-Error @"
upstream/windows-chewing-tsf 不是有效的 Git repo。
請先執行：
  git clone https://codeberg.org/chewing/windows-chewing-tsf.git upstream\windows-chewing-tsf
"@
    exit 1
}

Set-Location $upstreamPath

# 2. 若僅做 check，驗證 HEAD 是否為預期
if ($Check) {
    $head = git rev-parse HEAD
    if ($head -eq $ExpectedHead) {
        Write-Host "CHECK OK: HEAD = $head (符合預期 $ExpectedHead)" -ForegroundColor Green
        exit 0
    } elseif ($head -eq $BaselineSha) {
        Write-Host "CHECK: HEAD = $head (基準 commit，尚未套用 patches)" -ForegroundColor Yellow
        exit 2
    } else {
        Write-Host "CHECK: HEAD = $head (未知狀態)" -ForegroundColor Red
        exit 3
    }
}

# 3. 確認目前處於基準 commit
$currentHead = git rev-parse HEAD
Write-Host "目前 HEAD: $currentHead"

if ($currentHead -eq $ExpectedHead) {
    Write-Host "Patches 已套用，HEAD 符合預期。若需重新套用，請先 reset 至基準 commit。" -ForegroundColor Yellow
    exit 0
}

if ($currentHead -ne $BaselineSha) {
    Write-Warning "目前 HEAD 非基準 commit ($BaselineSha)，將嘗試套用 patches 但可能發生衝突。"
    $confirm = Read-Host "繼續？(y/N)"
    if ($confirm -ne 'y') { exit 1 }
}

# 4. 確認 patches 存在
$patches = Get-ChildItem $patchDir -Filter "*.patch" | Sort-Object Name
if ($patches.Count -eq 0) {
    Write-Error "找不到任何 patch 檔於 $patchDir"
    exit 1
}

Write-Host "準備套用 $($patches.Count) 個 patches..."

# 5. 逐一套用
foreach ($patch in $patches) {
    Write-Host "套用: $($patch.Name)" -ForegroundColor Cyan
    git am --3way $patch.FullName
    if ($LASTEXITCODE -ne 0) {
        Write-Error @"
套用失敗: $($patch.Name)
請執行以下指令還原：
  cd $upstreamPath
  git am --abort
然後手動解決衝突後重試。
"@
        exit 1
    }
}

# 6. 驗證最終 SHA
$finalHead = git rev-parse HEAD
if ($finalHead -ne $ExpectedHead) {
    Write-Warning "最終 HEAD ($finalHead) 與預期 ($ExpectedHead) 不符。Patches 可能有變動。"
    exit 2
} else {
    Write-Host "SUCCESS: 所有 patches 套用完成。HEAD = $finalHead" -ForegroundColor Green
}

# 7. 執行基礎 build 驗證（不安裝）
Write-Host "執行 cargo check 驗證建置..." -ForegroundColor Cyan
$env:PATH += ";$env:USERPROFILE\.cargo\bin"
cargo check --workspace 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Error "cargo check 失敗，請查看錯誤訊息。"
    exit 1
}
Write-Host "cargo check 通過。" -ForegroundColor Green
