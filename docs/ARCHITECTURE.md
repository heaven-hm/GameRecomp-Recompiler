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
in that repository and gives each language a clean package layout.

## Provenance

- The CLI invoked is `gui/build/game-recomp-cli` from GameReComp.
- SDK code here is original; no rev.ng or Wine source was copied.
- The GUI screenshot in `docs/screenshots/` was captured from the verified
  GameReComp GUI with CuaDriver (Accessibility + Screen Recording enabled).

## Not claimed

- No 100% game compatibility, no original-Windows oracle, no Linux runner.
- No rev.ng-class decompiler or second static recompiler.
- A successful SDK call means a pipeline report exists, not that a game works.
