# Working apps with real proof

Each SDK has a working app under `demos/apps/`. Run all five against the real
GameReComp CLI with `sh demos/run_apps_real.sh` after setting:

- `GAMERECOMP_REAL_CLI` — path to the real `game-recomp-cli` v0.3.0 binary
- `GAMERECOMP_SOURCE_ROOT` — GameReComp checkout (the CLI launcher needs it as cwd)
- `GAMERECOMP_REAL_PE` — a real PE32 x86 input (proof used `arith-calc.exe`)

`demos/real-cli-wrapper.sh` appends `--skip-stages verify` because the
fail-closed CLI exits 3 without an original-Windows oracle. It invokes the real
binary; no fake CLI is used.

| SDK | App | Screenshot |
|---|---|---|
| C | `demos/apps/c/app.c` | `proof/screenshots/terminal-c.png` |
| C++ | `demos/apps/cpp/app.cpp` | `proof/screenshots/terminal-cpp.png` |
| Python | `demos/apps/python/app.py` | `proof/screenshots/terminal-python.png` |
| JavaScript/TypeScript | `demos/apps/js/app.js` | `proof/screenshots/terminal-javascript.png` |
| Rust | `demos/apps/rust/app.rs` (Blaze-built) | `proof/screenshots/terminal-rust.png` |

Exact stdout/stderr, hashes and exit codes: `proof/apps-real/results.json`.
Scope: real PE32 recompilation proof only, no gameplay equivalence claim.
