# RUN-20261022-WIN-TSF-BUILD-01 (Windows TSF Release Build)

- Tester / agent: Antigravity Autonomous Agent
- Execution Time: 2026-10-22 18:45:51 (Asia/Taipei)
- Platform: Windows 10/11 x86_64
- Toolchain:
  - Rust: `rustc 1.99.1` (stable-x86_64-pc-windows-msvc)
  - MSVC: `cl.exe 19.50.35721` (Visual Studio 2026 v18.1.1)
  - Cargo: `1.99.1`
- Main integration SHA: `bad328c`
- Upstream SHA: `windows-chewing-tsf@053d31ba` + `chewing-policy-core@0.1.0`

## Build Command
```powershell
cmd /c 'call "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvarsall.bat" x64 && cargo build --package chewing_tip --release && cargo build --package tsfreg --release'
```

## Generated Artifacts
1. `dist/windows/chewing_tip.dll`
   - Size: 4,297,216 bytes
   - SHA256: `F8ACC7FFCD5DA3232D01655AE49EC82F847E775F32D5787D00ACDC66FF84F7F2`
   - COM Exports: `DllGetClassObject`, `DllMain`, `policy_should_bypass_key_rs`, `policy_transform_commit_rs`
2. `dist/windows/tsfreg.exe`
   - Size: 221,184 bytes
   - SHA256: `520C70390880E7F28C302DBB954C63AA3FD10876972C4AB1A87C6BA00A2022D6`

## Registration Status
- Registered to HKCU: `HKCU:\Software\Classes\CLSID\{D4C3B2A1-0E9E-4340-9E11-209B0E9C7E2E}`
- Configuration deployed: `%APPDATA%\ChewingDev\policy.toml`
- Decision: **PASS for Gate-B (Windows TSF Native Binary Ready for Use)**
