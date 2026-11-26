# Architecture and boundary

GameRecomp-SDK is a pure Rust client library. All recompilation is performed
by the separate `game-recomp-cli` binary from GameReComp v0.3.0, which is built
on PCRecomp's mechanical recompiler and Win32 runtime. This crate contains no
lifter, loader or second runtime.

## Contract

1. Validate that the CLI path is a regular file.
2. Validate that the PE32/x86 input exists.
3. Spawn exactly `run <pe> --project-dir <dir> --no-table --no-progress` using
   `std::process::Command` — no shell interpolation.
4. Require exit code 0.
5. Require `<dir>/reports/report.json` to exist; return its path, optional
   score and raw JSON.

Every failure is a typed `Error` with path and exit-status context. A missing
CLI, a failed spawn, a non-zero exit or a missing report never produces a fake
success.

## Scope

Verified scope is PE32/i386 through GameReComp CLI v0.3.0 on Linux and macOS.
PE32+/Win64, gameplay equivalence, universal PC-game support and renderer
compatibility are out of scope and are not claimed. See [README](../README.md).
