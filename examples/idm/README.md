# IDM port through each SDK language

These are **SDK source files** that call the GameReComp engine to port the
real `idman643build18.exe` from Windows PE32 to macOS arm64. No generated
recompiler output, lifted app source or build artifacts live in this repo.

| Language | SDK source | Build call |
|---|---|---|
| C | [`demos/apps/c/app.c`](../../demos/apps/c/app.c) | `gr_build(&client, pe, project, &options, ...)` |
| C++ | [`demos/apps/cpp/app.cpp`](../../demos/apps/cpp/app.cpp) | `client.build(pe, project, options)` |
| Python | [`demos/apps/python/app.py`](../../demos/apps/python/app.py) | `Client().build(pe, project, BuildOptions(...))` |
| JavaScript/TypeScript | [`demos/apps/js/app.js`](../../demos/apps/js/app.js) | `client.build(pe, project, {...})` |
| Rust | [`demos/apps/rust/app.rs`](../../demos/apps/rust/app.rs) | `Client::from_env()?.build(pe, project, BuildOptions...)` |

Run all five with `sh demos/run_apps_real.sh` after setting
`GAMERECOMP_REAL_CLI`, `GAMERECOMP_SOURCE_ROOT` and `GAMERECOMP_REAL_PE`.
Real CLI proof and per-SDK terminal screenshots are in
[`docs/IDM_PORT_PROOF.md`](../../docs/IDM_PORT_PROOF.md).

The SDK passes guest/host options (input OS/arch, target OS/arch, out mode,
skipped stages) to the engine; the engine path is discovered internally and is
not part of the public API.
