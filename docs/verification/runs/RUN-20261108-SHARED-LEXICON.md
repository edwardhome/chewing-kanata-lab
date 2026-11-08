# Shared lexicon development verification

Updated: 2026-11-08T19:28:56+08:00

- Windows isolated test snapshot: 270 passed, 1 ignored, 0 failed. Core 156, native workspace 94, preferences 20. No installed IME or private dictionaries accessed.
- Frontend tsc/Vite build: PASS; existing large chunk warning remains.
- Seven synthetic entries: Windows Rust export -> actual Mac Swift decode/export -> Windows Rust decode/export -> full JSON equality PASS.
- Native source commit: 2febd01. Full 30-patch replay from pinned baseline: PASS; tree 60c424a14d71f37c4030bdb52801b8a391024c88.
- Preferences source commit: 7c6e173. Full 8-patch replay from pinned baseline: PASS; tree c8d563151416db96e06efc7325889251d2f0070e.
- All patch SHA-256 values verified before replay. Nested source changes are retained in outer repository patches.
- Release-mode builds: PASS on WorkstationX with --locked, native workspace and ChewingPreferences with tauri/custom-protocol. Existing warnings remain. Packaging, GUI, physical-key acceptance, installation and publication: NOT_RUN.

AGY documentation/negative fixtures received through commit 9f299cc. AGY checklist eeb7983 reviewed; Codex corrected candidate projection, idle disable, and private-data verification boundaries. Test details and limitations: docs/shared-lexicon/VALIDATION.md.

## Integration delivery

Updated: 2026-11-08T20:18:54+08:00

The user explicitly authorized Windows canonical main integration and source push through the Mac integration handoff. No installation, IME restart, tag or GitHub Release is authorized or performed in this delivery.

- Development version: `1.1.1-dev.shared.20261108`; native, preferences Cargo/Tauri/npm versions agree.
- Package outer source: `5478af1590f4cb8499bc76bae971b1e8e538db52`. Later commits add documentation only; product source and patch series are unchanged.
- Native: `b2b258edbdc9d2432e4a4585f6d95b9fa528e55c`, tree `ab512993523410284319161e6a193ca4253148fe`; 31 patches fully replayed.
- Preferences: `0edbe1577be9f7d4ff23086672b36e3f92bff8db`, tree `d9734c0e909679e3695267c7ee75ad9ad6a8a4d0`; 9 patches fully replayed.
- Final development-version rerun: 270 passed, 1 ignored, 0 failed; both locked Release builds passed.
- Package guard: 23 cases passed. 203 tracked source files matched the actual build snapshot byte for byte; frontend dist SHA-256 matched the local Vite output. Public dictionary hashes matched the prior published package manifest.
- Initial package source check rejected a Windows checkout line-ending difference. No guard was bypassed: preserved that clone, created a new clone with core.autocrlf=false, and passed exact byte checks.
- Verified package directory on WorkstationX: `C:\Users\Defu\Documents\AI-Chewing\1.1.1-dev.shared.20261108`.
- DLL SHA-256: `57bc320269d904cdf95a88d899fbb6ce43a87c779ab8dc3f89a1be39c44d5fda`.
- Preferences EXE SHA-256: `d956f12780d98a81833bb1984b25ac9edf468438d04b114d4212b54da23824c8`.

Manual GUI, physical keys, real installation/removal and Mac GUI remain NOT_RUN. Format roundtrip equality is not deployment-metadata equality: Mac approval clears usage and changes review status; Windows retains source metadata and uses an independent local receipt. v1 rejects zh_aliases; explicit omission export preserves originals and reports exact omissions. Automatic private Chinese dictionary export is not implemented.

## Canonical integration confirmation

- Windows canonical main integration commit: `3d331c7e0890105ed8ce436db914c3a3346fe42c`. Push succeeded to `edwardhome/chewing-kanata-lab`; `git ls-remote` independently returned the same main SHA.
- Compared package source `5478af1` with integrated main: no differences in policy-core, either nested patch series, or packaging scripts. Remaining differences are review/documentation history. The package manifest intentionally pins the exact build source, not a later documentation commit.
- Mac integration was reported by its owning thread as main `af1417343f6888af5cb8665b2f5606dfba7149c4`, 292 passed / 2 ignored, Release and bundle verification passed. Windows did not modify or install Mac sources; this Mac result is a handoff report, not a Windows-run test.
- Development archive (not a GitHub Release): `C:\Users\Defu\Documents\AI-Chewing\AI-Chewing-1.1.1-dev.shared.20261108-portable.zip`. Archive extracted into a new temporary directory and passed full package validation; no installer executed.

- Portable ZIP: 9,536,323 bytes; SHA-256 `f7bcb5f0352e467942c55bf34483ef7b7e3b9730fbdb7a10a634b76ca836eece`. Local copy: `dist/windows/shared-lexicon/AI-Chewing-1.1.1-dev.shared.20261108-portable.zip`; independently verified same SHA-256 after transfer.
- PowerShell Compress-Archive stalled; only that owned compression process was stopped. Its incomplete output is not a deliverable. The portable ZIP above was created with tar and validated after extraction.
- Final source delivery includes AGY follow-up `1923c23`, documentation only. Windows shared checkout synchronization preserves the two existing untracked verification reports.
