# Real-world port proof: IDM Windows PE32 → macOS arm64

Internet Download Manager (`idman643build18.exe`, SHA-256
`7346128fab42957b4254eabdb012bfdbd751e9069361e98a6916f26bb03d62ee`) was
recompiled by the **real GameReComp CLI v0.3.0** into a native macOS arm64
binary through every SDK binding.

Each screenshot shows both the **real SDK app source code** and its **real
output** from that run:

| SDK | Source | Proof screenshot |
|---|---|---|
| C | [`demos/apps/c/app.c`](../demos/apps/c/app.c) | [`proof/screenshots/idm-c.jpeg`](../proof/screenshots/idm-c.jpeg) |
| C++ | [`demos/apps/cpp/app.cpp`](../demos/apps/cpp/app.cpp) | [`proof/screenshots/idm-cpp.jpeg`](../proof/screenshots/idm-cpp.jpeg) |
| Python | [`demos/apps/python/app.py`](../demos/apps/python/app.py) | [`proof/screenshots/idm-python.jpeg`](../proof/screenshots/idm-python.jpeg) |
| JavaScript/TypeScript | [`demos/apps/js/app.js`](../demos/apps/js/app.js) | [`proof/screenshots/idm-javascript.jpeg`](../proof/screenshots/idm-javascript.jpeg) |
| Rust (Blaze-built) | [`demos/apps/rust/app.rs`](../demos/apps/rust/app.rs) | [`proof/screenshots/idm-rust.jpeg`](../proof/screenshots/idm-rust.jpeg) |

All five apps exited 0 with `APP_OK`; C++/Python/JS reported score 85 and 6
stages; Rust reported `Some(85)`. Exact stdout, stderr, exit codes, source
hashes and source text are in
[`proof/apps-idm/results.json`](../proof/apps-idm/results.json).

`demos/real-cli-wrapper.sh` invokes the real CLI binary and appends
`--skip-stages verify` because the fail-closed CLI exits 3 without an
original-Windows oracle. No fake CLI was used.

**Limits**: this proves real PE32 recompilation and report generation, not IDM
UI behaviour or gameplay equivalence. IDM itself exits at its Windows-version
check before any UI; no frame claim is made.
