# Cross-OS port examples (10 directions)

Each SDK language has an example that takes a direction id, a real PE32 input
and a project directory:

| Language | Source |
|---|---|
| C | [`c/app.c`](c/app.c) |
| C++ | [`cpp/app.cpp`](cpp/app.cpp) |
| Python | [`python/app.py`](python/app.py) |
| JavaScript/TypeScript | [`javascript/app.js`](javascript/app.js) |
| Rust | [`rust/app.rs`](rust/app.rs) |

The canonical list of 10 directions is [`matrix.json`](matrix.json). Run the
Python example with `PYTHONPATH=<sdk>:/<sdk>/python`:

```sh
python3 examples/cross_os/python/app.py <direction> <pe32.exe> <project-dir>
```

Real proof for all 10 directions is in
[`proof/cross-os/results.json`](../../proof/cross-os/results.json). It records
exact stdout, SHA-256 digests, exit codes and report hashes. One direction
(Windows PE32 x86 → host macOS arm64) produced a real report with score 85 and
6 stages; the other nine returned explicit `PORT_DEFERRED` reasons because
this engine currently only lifts Windows PE32 x86 and only builds for the
current host. No fake PASS is recorded.

C/C++/Rust/JS examples use the same SDK build options; their engine bridge is
still the process runner documented in `docs/ENGINE_IN_PROCESS.md`.
