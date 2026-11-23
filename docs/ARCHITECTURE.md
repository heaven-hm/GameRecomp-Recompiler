# Architecture and boundary

```
your C/C++/Rust/Python code
          |
          v
   GameRecomp-Recompiler SDK (this repo)
          |  invokes CLI: game-recomp-cli run <pe> --project-dir <dir>
          v
   GameReComp (separate repository, the real recompiler)
          |
          v
   native C -> native binary -> reports/report.json
```

The SDK contains **no** PE parser, lifter, CPU emulator or linker. It is a
client library so applications can embed access to the existing pipeline from
four languages with one consistent contract.

## Contract

- `new/init` validates the CLI path; a missing CLI is an error, not a fallback.
- `recompile(pe, project)` runs the CLI and requires a zero exit status.
- The SDK then requires `reports/report.json` to exist and returns/reads it.
- Any failure returns an explicit error (`-1`, exception, `Result::Err`, or
  `RecompileError`). No method returns a fabricated report.

## Why a separate repository

The recompiler itself lives in GameReComp and must remain the single
implementation. Keeping client bindings here avoids duplicating runtime code
in that repository and gives each language a clean package layout. This is a
hard rule for contributors — see the "no second recompiler" section in
[CONTRIBUTING.md](CONTRIBUTING.md).

## Provenance

- The CLI invoked is `gui/build/game-recomp-cli` from GameReComp.
- SDK code here is original; no rev.ng or Wine source was copied.
- The GUI screenshot in `docs/screenshots/` was captured from the verified
  GameReComp GUI with CuaDriver (Accessibility + Screen Recording enabled).

## Security boundary

The CLI path is the only configuration the SDK consumes (via `GAMERECOMP_CLI`
in Python, or as an explicit argument in C/C++/Rust). Captured CLI output on
failure is untrusted text and must be sanitized before it is logged or shared;
see [SECURITY.md](SECURITY.md) for the environment-only and error-sanitization
policy.

## Verification

The same four binding checks run on Ubuntu and macOS in
[`.github/workflows/ci.yml`](../.github/workflows/ci.yml); the matrix and the
commands are summarized in the [README](../README.md#verification) and in
[CONTRIBUTING.md](CONTRIBUTING.md#test-matrix).

## Not claimed

- No 100% game compatibility, no original-Windows oracle, no Linux runner.
- No rev.ng-class decompiler or second static recompiler.
- A successful SDK call means a pipeline report exists, not that a game works.
