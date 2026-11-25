# GameReComp SDK quickstart (v0.3.0)

The SDK is a thin client for the `game-recomp-cli` shipped by GameReComp
`v0.3.0` or newer. It does not contain a second recompiler. Every binding uses
the same stable contract:

```
<cli> run <pe32-x86.exe> --project-dir <dir> --no-table --no-progress
```

A successful run requires exit code 0 and `<dir>/reports/report.json`.
Failures never produce a fake report.

## 1. Get a CLI

Build `game-recomp-cli` from the GameReComp repository at tag `v0.3.0`.
See `docs/REAL_CLI_INTEGRATION.md` for a real end-to-end run.

```sh
export GAMERECOMP_CLI=/abs/path/to/game-recomp-cli
```

## 2. Pick a language

| Language | Install | Example |
|---|---|---|
| Python | `pip install -e python` | `PYTHONPATH=python python3 python/examples/basic.py --pe game.exe --project /tmp/gr` |
| Rust | path dependency on `rust/` | `cd rust && blaze run --example basic -- "$GAMERECOMP_CLI" game.exe /tmp/gr` |
| C | compile `src/gamerecomp_sdk.c` + `c/examples/basic.c` | `docs/C.md` |
| C++ | compile `src/gamerecomp_sdk.c` + `cpp/examples/basic.cpp` | `docs/CPP.md` |

Never hard-code a CLI path or credential in shipped code; pass it via an
argument or the `GAMERECOMP_CLI` environment variable. Sanitize error text
before sharing it (`docs/SECURITY.md`).

## 3. Verify locally

```sh
python3 -m pytest python/tests -q          # Python contract tests
cmake -S . -B build && cmake --build build # C/C++ SDK and examples
cd rust && blaze test                       # Rust SDK via Blaze, no cargo/rustc
```

Blaze is the local Rust build tool for this SDK (`curl -fsSL https://ix.dev/blaze | sh`).
CI cross-platform matrices use their own toolchains; see `docs/CI.md`.

## 4. Read the report

All bindings return the same `report.json` path or parsed data. The report's
`score` is a process-level measure, not gameplay equivalence. Keep PARTIAL and
BLOCKED verdicts; do not promote them to PASS. See `docs/API_CONSISTENCY.md`
for exact per-language differences and `docs/EXAMPLES.md` for copy-paste code.
