# Architecture and boundary

```
your C/C++/Rust/Python code
           |
           v
    GameRecomp-Recompiler SDK (this repo)
           |  child process: <cli> run <pe> --project-dir <dir> --no-table --no-progress
           v
    GameReComp CLI (separate project; supplied by the caller)
           |
           v
    native C -> native binary -> <project>/reports/report.json
```

The SDK contains **no** PE parser, lifter, CPU emulator, linker or decompiler.
It is a set of client libraries so applications can invoke the existing
pipeline from four languages with one small contract. How recompilation is
implemented is entirely the responsibility of the upstream CLI.

## Contract

- `new`/`init` validates the CLI path before doing anything else; a missing CLI
  is an error, never a fallback. C/C++ check that the path exists, Rust/Python
  require a regular file. Python additionally reads `$GAMERECOMP_CLI` (then
  falls back to the name `game-recomp-cli` on `PATH`).
- `recompile(pe, project)` (C: `gr_recompile`) starts the CLI with an argument
  vector — no shell interpolation — and requires a zero exit status.
- The SDK then requires `<project>/reports/report.json` to exist and returns
  that path, or parsed data read from it.
- Any failure returns an explicit error (`-1`, exception, `Result::Err`, or
  `RecompileError`). No method returns a fabricated report.

The contract is shared, but the surfaces are not: Python alone provides
`inspect()` and `recompile_to_path()`, and the success values differ (path only
in C, `int score()` in C++, `Option<i64>` + raw JSON in Rust, a `dict` in
Python). [API_CONSISTENCY.md](API_CONSISTENCY.md) is the authoritative map;
the per-language guides ([C](C.md), [C++](CPP.md), [Rust](RUST.md),
[Python](PYTHON.md)) are the detailed references.

## Why a separate repository

The recompiler itself lives in GameReComp and must remain the single
implementation. Keeping client bindings here avoids duplicating runtime code
in that repository and gives each language a clean package layout. This is a
hard rule for contributors — see the "no second recompiler" section in
[CONTRIBUTING.md](CONTRIBUTING.md).

## What is actually in the child-process call

All four bindings build the same argument vector:

```
<cli> run <pe> --project-dir <project> --no-table --no-progress
```

Python's optional `inspect()` instead calls `<cli> inspect <pe> --json` and
does not need a project directory. The SDK does not hard-code a CLI build path:
C/C++/Rust take it as an argument and Python takes an argument, an environment
variable, or `PATH`. An upstream build such as `gui/build/game-recomp-cli` is
just one possible value.

## Provenance

- All code here is original thin client code; no third-party runtime source
  (rev.ng, Wine or otherwise) is vendored in this repository.
- The repository is dependency-light: Python uses only the standard library,
  the Rust crate declares no dependencies, and C/C++ use the system compiler
  and POSIX APIs.
- The image in `docs/screenshots/` is an illustrative GameReComp GUI screenshot
  kept for context. The SDK does not depend on the GUI and the image is not a
  verification artefact for any binding.

## Platform boundary

| Binding | Process mechanism | Supported here |
|---|---|---|
| C | `fork`/`execvp`/`pipe`/`waitpid` | POSIX (Linux, macOS); no Windows support |
| C++ | RAII wrapper over the C implementation | POSIX (Linux, macOS) |
| Rust | `std::process::Command` | portable std API; shell-based fake-CLI tests are Unix-only |
| Python | `subprocess.run` | portable stdlib API |

CI runs on `ubuntu-latest` and `macos-latest` only. See
[API_CONSISTENCY.md](API_CONSISTENCY.md#platform-support) for the full matrix.

## Security boundary

The CLI path is the only configuration this SDK consumes (an argument in
C/C++/Rust; argument, `GAMERECOMP_CLI` or `PATH` in Python). There is no config
file and no credential store. Captured CLI output on failure is untrusted text:
C/C++ retain up to 511 bytes in the C error buffer, Python keeps the last 2000
characters, and Rust does not expose captured output at all. Sanitize any
captured tail before logging or sharing; see
[SECURITY.md](SECURITY.md#error-sanitization) and the
[error-handling table](API_CONSISTENCY.md#error-handling).

## Verification

The four binding checks are configured to run on Ubuntu and macOS in
[`.github/workflows/ci.yml`](../.github/workflows/ci.yml); the matrix and
commands are summarized in the [README verification section](../README.md#verification)
and in [CONTRIBUTING.md](CONTRIBUTING.md#test-matrix). At base `d931c02` all
four checks also passed locally on macOS (13 Python tests, 15 Rust tests, plus
the C and C++ proof scripts). Live CI status is not asserted here.

## Not claimed

- No 100% game compatibility, no original-Windows oracle, and no Windows CI
  runner. Ubuntu in CI is a Linux runner; macOS is the other runner.
- No rev.ng-class decompiler or second static recompiler.
- A successful SDK call means a pipeline report exists, not that a game works.
- No claim that a specific PE format is accepted or refused; that decision
  belongs to the upstream CLI.
