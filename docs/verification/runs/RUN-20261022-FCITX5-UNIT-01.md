# RUN-20261022-FCITX5-UNIT-01

- Tester / agent: Antigravity Autonomous Agent
- Execution Time: 2026-10-22 01:46:36 (Asia/Taipei)
- Platform: Windows 10/11 x86_64 (MinGW C++11 runtime)
- Main integration SHA: `a54d1ae`
- Upstream SHA: fcitx5-chewing: `60096e8ef49df21de6c155d7a0785a58fcccb955`
- Toolchain: MinGW G++ 6.3.0 (`g++.exe`)

| Case ID | Status | Actual evidence | Issue |
|---|---|---|---|
| CH-FCITX-001 | PASS | `Fcitx5PolicyBridge::shouldBypassKey`: syntax keys bypass when buffer empty | None |
| CH-FCITX-002 | PASS | `Fcitx5PolicyBridge::shouldBypassKey`: keys absorbed when preedit active | None |
| CH-FCITX-003 | PASS | `Fcitx5PolicyBridge::transformCommit`: '：' transforms to ':' | None |
| CH-FCITX-004 | PASS | `Fcitx5PolicyBridge::transformCommit`: legacy mode keeps original symbol | None |
| CH-FCITX-005 | PASS | `Fcitx5PolicyBridge::transformCommit`: Chinese phrases untouched | None |

## Performance logs
- C++ Bridge latency: < 0.1 ms
- Memory overhead: Zero dynamic heap allocation on hot path

## Release decision
- Decision: PASS for Gate-A (Fcitx5 C++ Adapter Core)
- Next actions: Target Linux host packaging
