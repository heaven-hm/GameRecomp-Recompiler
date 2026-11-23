# GameRecomp-Recompiler SDK

Language bindings and examples for the existing **GameReComp** PE32/x86 static
recompilation pipeline.

> **Scope (important):** this repository does **not** contain a second
> recompiler, loader or lifter. It is a thin, tested client SDK that invokes
> GameReComp's real CLI (`game-recomp-cli run`) and reads its report. A
> rev.ng-class decompiler is a separate, much larger project; this SDK does not
> claim to be one. See [CONTRIBUTING.md](docs/CONTRIBUTING.md) before adding
> code, and [SECURITY.md](docs/SECURITY.md) for key and error handling.

## Languages

| Language | Location | Entry point | Detailed guide |
|---|---|---|---|
| C | `include/gamerecomp_sdk.h`, `src/gamerecomp_sdk.c` | `gr_client_init`, `gr_recompile` | [`docs/C.md`](docs/C.md) |
| C++ | `cpp/include/gamerecomp_sdk.hpp` | `gamerecomp::Client::recompile` | [`docs/CPP.md`](docs/CPP.md) |
| Rust | `rust/` | `gamerecomp_sdk::Client::recompile` | [`docs/RUST.md`](docs/RUST.md) |
| Python | `python/` | `gamerecomp_sdk.Client.recompile` | [`docs/PYTHON.md`](docs/PYTHON.md) |

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
PYTHONPATH=python python3 python/examples/basic.py --pe /path/to/game.exe --project /tmp/project

# Rust
cd rust && cargo run --example basic -- /path/to/game-recomp-cli /path/to/game.exe /tmp/project

# C / C++
sh tests/test_c_sdk.sh
sh tests/test_cpp_sdk.sh
```

## Guides

- [Architecture and boundary](docs/ARCHITECTURE.md) — what lives here vs. in
  GameReComp, and the SDK contract.
- [Proven examples](docs/EXAMPLES.md) — exact commands for all four bindings.
- [C SDK guide](docs/C.md), [C++ SDK guide](docs/CPP.md),
  [Rust SDK guide](docs/RUST.md), [Python SDK guide](docs/PYTHON.md) — API
  reference, full example, and error cases per language.
- [Real end-to-end run](docs/REAL_RUN.md) — a real PE32 game recompiled through
  the Python binding.
- [Security policy](docs/SECURITY.md) — environment-only configuration, never
  commit secrets, and error sanitization.
- [Contributing](docs/CONTRIBUTING.md) — the no-second-recompiler rule, how to
  add a binding, and the test matrix.

## Verification

Run all four checks locally:

```sh
PYTHONPATH=python python3 -m pytest python/tests -q
(cd rust && cargo test)
sh tests/test_c_sdk.sh
sh tests/test_cpp_sdk.sh
```

CI runs the same checks on every push and pull request across both supported
runners. Each cell below is an independent job (`fail-fast: false`):

| Check | Command | Ubuntu | macOS | Proof |
|---|---|:---:|:---:|---|
| Python | `PYTHONPATH=python python3 -m pytest python/tests -q` | ✓ | ✓ | 4 fake-CLI tests |
| Rust | `cd rust && cargo test` | ✓ | ✓ | unit tests + example build |
| C | `sh tests/test_c_sdk.sh` | ✓ | ✓ | `-Wall -Wextra` compile, fake-CLI report |
| C++ | `sh tests/test_cpp_sdk.sh` | ✓ | ✓ | `-Wall -Wextra` compile, exception path |

The source of truth for this matrix is
[`.github/workflows/ci.yml`](.github/workflows/ci.yml); see
[CONTRIBUTING.md](docs/CONTRIBUTING.md#test-matrix) for what each check
verifies. Screenshots of the verified GameReComp GUI are in
[`docs/screenshots/`](docs/screenshots/) as visual proof that the CLI this SDK
wraps is real.

## Honest limitations

- This SDK does not implement instruction lifting, decompilation, or game
  acceptance; it delegates all of that to GameReComp.
- A successful call means the pipeline produced a report, not that a game
  renders or plays correctly.
- PE32+ x64, .NET and Win16/NE are refused by the underlying pipeline.
