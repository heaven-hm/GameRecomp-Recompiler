# Contributing

## Ground rule: no second recompiler

This repository is a **client SDK** around the existing GameReComp CLI. Do not
add a PE parser, lifter, CPU emulator, linker, decompiler, or any code that
re-implements a pipeline stage owned by GameReComp. A binding must invoke
`game-recomp-cli run ...` as a child process and read
`reports/report.json`.

If you need new recompiler behavior, change it in GameReComp and expose it
through the CLI; then consume the new behavior here. See
[ARCHITECTURE.md](ARCHITECTURE.md) for the boundary.

## Repository layout

| Path | Purpose |
|---|---|
| `include/gamerecomp_sdk.h`, `src/gamerecomp_sdk.c` | C implementation (POSIX process APIs) |
| `cpp/include/gamerecomp_sdk.hpp` | Header-only C++ wrapper over the C client |
| `rust/` | Rust crate, library tests and example |
| `python/` | Python package, example and tests |
| `tests/` | Compile-and-run proof scripts (C, C++) |
| `docs/` | Per-language guides, policies and the cross-binding API map |
| `.github/workflows/ci.yml` | Ubuntu + macOS verification matrix |

## Adding a new language binding

1. **Create the package layout** under a new top-level directory (for example
   `go/`): library source, `examples/basic.<ext>`, and a test that uses a fake
   CLI or a mocked process call. Add a proof script under `tests/` following
   `test_c_sdk.sh`.
2. **Implement the one contract** (see
   [API_CONSISTENCY.md](API_CONSISTENCY.md#contract)):
   - `init`/constructor validates the CLI path; a missing CLI is an error,
     never a fallback;
   - `recompile(pe, project)` runs
     `<cli> run <pe> --project-dir <project> --no-table --no-progress` with no
     shell interpolation and requires a zero exit status;
   - it then requires `<project>/reports/report.json` to exist and returns the
     parsed report or its verified path.
3. **Fail explicitly.** Every failure (missing CLI, missing PE, failed exec,
   non-zero exit, missing report) returns an error/exception/`Result::Err` with
   a short message. No method may fabricate a report. See
   [SECURITY.md](SECURITY.md#error-sanitization) for error-sanitization rules
   and the [error-handling matrix](API_CONSISTENCY.md#error-handling) for the
   existing variants.
4. **Configuration stays environment-driven.** Read the CLI location from an
   environment variable where idiomatic (Python reads `GAMERECOMP_CLI`), or
   take it as an explicit argument. Do not hard-code paths, keys, or hotkeys.
5. **Test with a fake CLI or a mock.** A fake CLI writes a `report.json` and
   exits zero; the test asserts the validation and failure cases plus the
   success path. This proves plumbing without a long native rebuild. The Python
   suite mocks `subprocess.run` and asserts the exact command vector; the Rust
   suite launches shell-script fake CLIs in `#[cfg(unix)]` tests.
6. **Document it.** Add `docs/<LANGUAGE>.md` (files, API reference, full
   example, error table, notes), add a row to the language table in
   `README.md`, add the language to the verification matrix below and to the
   CI matrix in `.github/workflows/ci.yml`, add a row to the API map in
   [API_CONSISTENCY.md](API_CONSISTENCY.md#api-map), and add a runnable example.
7. **Keep it dependency-light.** Prefer the language standard library; the
   existing bindings use no third-party runtime crates or packages (pytest is
   a test-runner dependency installed by CI, not a runtime dependency).

## Test matrix

The same four checks are configured to run on both supported runners
(`ubuntu-latest`, `macos-latest`). CI is one matrix job with
`fail-fast: false`, so one OS never hides a failure on the other. The counts
below are the local results observed at base `d931c02`; the live CI status
must be checked at the repository's CI page.

| Check | Command | Ubuntu | macOS | Verifies | Local result |
|---|---|:---:|:---:|---|---|
| Python | `PYTHONPATH=python python3 -m pytest python/tests -q` | ✓ | ✓ | 13 tests: CLI/PE validation, success, CLI failure, `inspect()` (4), `recompile_to_path()` (4), delegation | 13 passed |
| Rust | `cd rust && cargo test` | ✓ | ✓ | 15 unit tests (incl. 3 Unix fake-CLI subprocess tests) and example build; no external crates | 15 passed |
| C | `sh tests/test_c_sdk.sh` | ✓ | ✓ | `-Wall -Wextra` compile, fake-CLI report path, project-dir validation, exec-error reason, `EINTR` retry | PASS |
| C++ | `sh tests/test_cpp_sdk.sh` | ✓ | ✓ | `-Wall -Wextra` compile, fake-CLI report, `Report::score()` with score present (`70`) and absent (`-1`); no exception test | PASS |

Notes:

- Python needs `PYTHONPATH=python` so `gamerecomp_sdk` is importable from the
  repository root; CI sets that as a step environment variable and installs
  `pytest` in a separate step.
- The C/C++ proof scripts create their own temporary fake CLI and binaries and
  clean them up; no leftover artifacts are committed.
- Unix fake CLIs need a shebang and the executable bit; the Rust tests publish
  their script by rename to avoid `ETXTBSY` flakes.
- A historical end-to-end run is recorded in [REAL_RUN.md](REAL_RUN.md), and
  exact per-language commands are in [EXAMPLES.md](EXAMPLES.md). Those are
  evidence for the pipeline, not for a binding's own correctness.
- Light local sanity for doc/CI-only changes: check that all markdown links
  resolve and that commands quoted still match the source; no full build is
  required.

## Before you open a PR

- [ ] No recompiler, lifter, emulator, or PE-parsing code was added here.
- [ ] New behavior is covered by a fake-CLI or mocked-subprocess test and an
      example.
- [ ] All four checks pass locally, and the README verification matrix and the
      API map are accurate.
- [ ] No secrets, tokens, or machine-specific paths appear in the diff; error
      text is sanitized per [SECURITY.md](SECURITY.md).
- [ ] New guides are linked from `README.md`; existing guides stay accurate.

## Scope of claims

A successful SDK call means the pipeline produced a report — not that a game
renders or plays correctly. Keep wording in docs and PRs consistent with the
[honest limitations](../README.md#honest-limitations) and the
[limitations](API_CONSISTENCY.md#limitations) section. Do not claim a live CI
result, a specific PE format acceptance, or a Windows runner that does not
exist in the CI configuration.
