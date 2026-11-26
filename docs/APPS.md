# Working apps with real recompilation proof

Each SDK has a small working app under `demos/apps/`. All five were run against
the **real GameReComp CLI v0.3.0** and the same real PE32 x86 input:

- input: `arith-calc.exe`, SHA-256 `9b59ee613acc7b5a1512f8a5fcc5f65317f0fa79452d01728e1715b63f23d7a6`
- CLI: `GameReComp-cli-v0.3.0-macos-arm64` from the GameReComp v0.3.0 release
- real stages: inspect, recover, translate, project, compile, run, report
- report score: 85; verification skipped because no Windows oracle exists

`demos/real-cli-wrapper.sh` adds `--skip-stages verify` because the fail-closed
CLI exits 3 without an original-Windows oracle. It invokes the real binary and
sets the GameReComp checkout as cwd; no fake CLI is involved.

| SDK | App | Screenshot | stdout SHA-256 (see `proof/apps-real/results.json`) |
|---|---|---|---|
| C | `demos/apps/c/app.c` | `proof/screenshots/c-real.jpeg` | recorded |
| C++ | `demos/apps/cpp/app.cpp` | `proof/screenshots/cpp-real.jpeg` | recorded |
| Python | `demos/apps/python/app.py` | `proof/screenshots/python-real.jpeg` | recorded |
| JavaScript/TypeScript | `demos/apps/js/app.js` | `proof/screenshots/javascript-real.jpeg` | recorded |
| Rust | `demos/apps/rust/app.rs` (built with Blaze) | `proof/screenshots/rust-real.jpeg` | recorded |

All five apps exited 0 and printed `APP_OK`. Exact stdout/stderr, hashes and
exit codes are in `proof/apps-real/results.json`.

**Limits**: this proves real PE32 recompilation through each SDK binding. It
does not prove gameplay equivalence or universal PC-game support, and no
original-Windows oracle was used.
