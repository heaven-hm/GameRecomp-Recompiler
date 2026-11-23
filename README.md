# GameRecomp-Recompiler SDK

Language bindings and examples for the existing **GameReComp** PE32/x86 static
recompilation pipeline.

> **Scope (important):** this repository does **not** contain a second
> recompiler, loader or lifter. It is a thin, tested client SDK that invokes
> GameReComp's real CLI (`game-recomp-cli run`) and reads its report. A
> rev.ng-class decompiler is a separate, much larger project; this SDK does not
> claim to be one.

## Languages

| Language | Location | Entry point |
|---|---|---|
| C | `include/gamerecomp_sdk.h`, `src/gamerecomp_sdk.c` | `gr_client_init`, `gr_recompile` |
| C++ | `cpp/include/gamerecomp_sdk.hpp` | `gamerecomp::Client::recompile` |
| Rust | `rust/` | `gamerecomp_sdk::Client::recompile` |
| Python | `python/` | `gamerecomp_sdk.Client.recompile` |

Each binding exposes the same three operations:

1. locate/validate the GameReComp CLI binary,
2. run a PE32 x86 file into a project directory,
3. read `reports/report.json` and return it as structured data (or a path).

Failures are explicit: a missing CLI, a non-zero pipeline exit, or a missing
report return an error rather than a fake success.

## Quick start

```sh
# Python
export GAMERECOMP_CLI=/path/to/game-recomp-cli
python3 python/examples/basic.py --pe /path/to/game.exe --project /tmp/project

# Rust
cd rust && cargo run --example basic -- /path/to/game.exe /tmp/project

# C / C++
sh tests/test_c_sdk.sh
sh tests/test_cpp_sdk.sh
```

See `docs/EXAMPLES.md` for exact commands and `docs/ARCHITECTURE.md` for the
boundary between this SDK and the recompiler itself.

## Verification

```sh
python3 -m pytest python/tests -q
cd rust && cargo test
sh tests/test_c_sdk.sh
sh tests/test_cpp_sdk.sh
```

CI runs the same commands. Screenshots of the verified GameReComp GUI are in
`docs/screenshots/` as visual proof that the CLI this SDK wraps is real.

## Honest limitations

- This SDK does not implement instruction lifting, decompilation, or game
  acceptance; it delegates all of that to GameReComp.
- A successful call means the pipeline produced a report, not that a game
  renders or plays correctly.
- PE32+ x64, .NET and Win16/NE are refused by the underlying pipeline.
