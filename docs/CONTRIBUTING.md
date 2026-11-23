# Contributing

## Ground rule: no second recompiler

This repository is a **client SDK** around the existing GameReComp CLI. Do not
add a PE parser, lifter, CPU emulator, linker, decompiler, or any code that
re-implements a pipeline stage owned by GameReComp. A binding must invoke
`game-recomp-cli run ...` as a child process and read `reports/report.json`.

If you need new recompiler behavior, change it in GameReComp and expose it
through the CLI; then consume the new behavior here. See
[ARCHITECTURE.md](ARCHITECTURE.md) for the boundary.

## Repository layout

| Path | Purpose |
|---|---|
| `include/gamerecomp_sdk.h`, `src/gamerecomp_sdk.c` | C implementation |
| `cpp/include/gamerecomp_sdk.hpp` | Header-only C++ wrapper |
| `rust/` | Rust crate and example |
| `python/` | Python package and example |
| `tests/` | Compile-and-run proof scripts (C, C++) |
| `docs/` | Per-language guides and policies |
| `.github/workflows/ci.yml` | Ubuntu + macOS verification matrix |

## Adding a new language binding

1. **Create the package layout** under a new top-level directory (for example
   `go/`): library source, `examples/basic.<ext>`, and a test that uses a fake
   CLI. Add a proof script under `tests/` following `test_c_sdk.sh`.
2. **Implement the one contract**. Every binding exposes the same three
   operations:
   - `init`/constructor validates that the CLI path is an existing file; a
     missing CLI is an error, never a fallback;
   - `recompile(pe, project)` runs
     `<cli> run <pe> --project-dir <project> --no-table --no-progress` with no
     shell interpolation, and requires a zero exit status;
   - it then requires `<project>/reports/report.json` to exist and returns the
     parsed report or its verified path.
3. **Fail explicitly.** Every failure (missing CLI, missing PE, non-zero exit,
   missing report) returns an error/exception/`Result::Err` with a short,
   sanitized message. No method may fabricate a report. See
   [SECURITY.md](SECURITY.md) for error-sanitization rules.
4. **Configuration stays environment-driven.** Read the CLI location from an
   environment variable where idiomatic (Python uses `GAMERECOMP_CLI`), or take
   it as an explicit argument. Do not hard-code paths, keys, or hotkeys.
5. **Test with a fake CLI.** The fake writes a `report.json` and exits zero;
   the test asserts the four error cases plus the success path. This proves
   plumbing without a long native rebuild.
6. **Document it.** Add `docs/<LANGUAGE>.md` (files, API reference, full
   example, error table, notes), add a row to the language table in
   `README.md`, and add the language to the verification matrix below and to
   the CI matrix in `.github/workflows/ci.yml`. Add a runnable example.
7. **Keep it dependency-light.** Prefer the language standard library; the
   existing bindings use no third-party runtime crates or packages.

## Test matrix

The same four checks run on both supported runners. CI is configured with
`fail-fast: false` so one OS never hides a failure on the other.

| Check | Command | Ubuntu | macOS | Verifies |
|---|---|:---:|:---:|---|
| Python | `PYTHONPATH=python python3 -m pytest python/tests -q` | ✓ | ✓ | 4 tests: missing CLI, missing PE, success, CLI failure |
| Rust | `cd rust && cargo test` | ✓ | ✓ | Unit tests + example build, no external crates |
| C | `sh tests/test_c_sdk.sh` | ✓ | ✓ | `-Wall -Wextra` compile + fake-CLI report path |
| C++ | `sh tests/test_cpp_sdk.sh` | ✓ | ✓ | `-Wall -Wextra` compile + exception/error path |

Notes:

- Python needs `PYTHONPATH=python` so `gamerecomp_sdk` is importable from the
  repository root (CI sets this as a step environment variable).
- The C/C++ proof scripts create their own temporary fake CLI and binaries; no
  leftover artifacts are committed.
- A real end-to-end run is documented in [REAL_RUN.md](REAL_RUN.md), and exact
  per-language commands are in [EXAMPLES.md](EXAMPLES.md). Those are evidence
  for the pipeline, not for a binding's own correctness.
- Light local sanity for doc/CI-only changes: validate the YAML workflow and
  any JSON fixtures; no full build is required.

## Before you open a PR

- [ ] No recompiler, lifter, emulator, or PE-parsing code was added here.
- [ ] New behavior is covered by a fake-CLI test and an example.
- [ ] All four checks pass locally, and the README verification matrix is
      accurate.
- [ ] No secrets, tokens, or machine-specific paths appear in the diff; error
      text is sanitized per [SECURITY.md](SECURITY.md).
- [ ] New guides are linked from `README.md`; existing guides stay accurate.

## Scope of claims

A successful SDK call means the pipeline produced a report — not that a game
renders or plays correctly. Keep wording in docs and PRs consistent with the
[honest limitations](../README.md#honest-limitations).
