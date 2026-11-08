# Shared lexicon development verification

Updated: 2026-11-08T19:28:56+08:00

- Windows isolated test snapshot: 270 passed, 1 ignored, 0 failed. Core 156, native workspace 94, preferences 20. No installed IME or private dictionaries accessed.
- Frontend tsc/Vite build: PASS; existing large chunk warning remains.
- Seven synthetic entries: Windows Rust export -> actual Mac Swift decode/export -> Windows Rust decode/export -> full JSON equality PASS.
- Native source commit: 2febd01. Full 30-patch replay from pinned baseline: PASS; tree 60c424a14d71f37c4030bdb52801b8a391024c88.
- Preferences source commit: 7c6e173. Full 8-patch replay from pinned baseline: PASS; tree c8d563151416db96e06efc7325889251d2f0070e.
- All patch SHA-256 values verified before replay. Nested source changes are retained in outer repository patches.
- Release-mode build: pending. Packaging, GUI, physical-key acceptance, installation and publication: NOT_RUN.

AGY documentation/negative fixtures received through commit 9f299cc. Manual acceptance checklist delegated through the same persistent conversation, awaiting review. Test details and limitations: docs/shared-lexicon/VALIDATION.md.
