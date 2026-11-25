# GameRecomp-Recompiler SDK

**A general-purpose recompilation SDK, built from the ground up on PCRecomp.**

This is the client layer of the GameReComp project: one stable, language-neutral
contract for driving a real PE32/x86 static recompiler from C, C++, Rust and
Python. PCRecomp is the baseline recompiler underneath; this repository adds
the portable SDK, the cross-language API discipline and the evidence contract.
The long-term goal is general-purpose PC software and game recompilation; the
**verified scope today is PE32/i386 through GameReComp CLI v0.3.0**. claim of
universal game support is made: architectures, formats and GameBox/D3D-class
targets that have not passed their own gates stay BLOCKED or DEFERRED.

> **Boundary:** this repository does **not** contain a second recompiler,
> loader or lifter. Every binding invokes `game-recomp-cli run` as a child
> process and reads `<project>/reports/report.json`. A rev.ng-class decompiler
> is a separate, much larger project and is not vendored here. See
> [ARCHITECTURE.md](docs/ARCHITECTURE.md) and
> [CONTRIBUTING.md](docs/CONTRIBUTING.md).

## What makes it different

1. **One contract, four languages.** C, C++, Rust and Python all validate the
   CLI path, spawn `run <pe> --project-dir <dir> --no-table --no-progress`
   without a shell, and require both exit 0 and a real `report.json`. No
   binding can quietly report success from a dry run or a missing file.
2. **No second recompiler.** PCRecomp remains the mechanical recompiler and
   Win32 runtime in GameReComp. This SDK is the reusable front door, not a fork
   of the lifter or a competing runtime.
3. **Fail-closed by construction.** Missing CLI, missing PE, failed exec,
   non-zero pipeline exit, invalid JSON or missing report all produce explicit
   errors. The SDK never invents a score, a frame or a gameplay verdict.
4. **Evidence over claims.** Scores are process-level measures. See
   [REAL_RUN.md](docs/REAL_RUN.md) for a real PE32 game and
   [RELEASE_0.3.0](https://github.com/heaven-hm/GameReComp/blob/v0.3.0/docs/RELEASE_0.3.0.md)
   for the current evidence and limits. No 100% or gameplay-equivalence claim
   is derived from a successful SDK call.

## Minimal examples

```python
# Python
from gamerecomp_sdk import Client
report = Client("/abs/path/to/game-recomp-cli").recompile("game.exe", "out")
print(report["score"])
```

```rust
// Rust (cd rust && blaze run --example basic -- <cli> <pe> <project>)
use gamerecomp_sdk::Client;
let report = Client::new("/abs/path/to/game-recomp-cli")?.recompile("game.exe", "out")?;
println!("{:?}", report.score);
```

```c
/* C: see docs/C.md and c/examples/basic.c */
gr_client *c = gr_client_init("/abs/path/to/game-recomp-cli");
char *report = gr_recompile(c, "game.exe", "out");
```

```cpp
// C++: see docs/CPP.md and cpp/examples/basic.cpp
gamerecomp::Client c("/abs/path/to/game-recomp-cli");
auto report = c.recompile("game.exe", "out");
```

Full copy-paste commands for all four languages are in
[QUICKSTART.md](docs/QUICKSTART.md) and [EXAMPLES.md](docs/EXAMPLES.md).

## Quickstart

## Languages

| Language | Location | Entry point | Detailed guide |
|---|---|---|---|
| C | `include/gamerecomp_sdk.h`, `src/gamerecomp_sdk.c` | `gr_client_init`, `gr_recompile` | [`docs/C.md`](docs/C.md) |
| C++ | `cpp/include/gamerecomp_sdk.hpp` | `gamerecomp::Client::recompile` | [`docs/CPP.md`](docs/CPP.md) |
| Rust | `rust/` | `gamerecomp_sdk::Client::recompile` | [`docs/RUST.md`](docs/RUST.md) |
| Python | `python/` | `gamerecomp_sdk.Client.recompile` | [`docs/PYTHON.md`](docs/PYTHON.md) |

There is a shared **contract** across the bindings:

1. validate the CLI path before running anything,
2. run a PE32 x86 file into a project directory by invoking
   `run <pe> --project-dir <project> --no-table --no-progress` as a child
   process (no shell interpolation),
3. require a zero exit status and the existence of
   `<project>/reports/report.json`, returning it as a path or structured data.

Failures are explicit: a missing CLI, a missing PE, a failed exec, a non-zero
pipeline exit, or a missing report return an error rather than a fake success.

The bindings are **not** API-identical. Only Python offers `inspect()` and
`recompile_to_path()`; C returns a report path only, C++ adds `Report::score()`
(`int`, `-1` when absent), Rust returns `Report { path, score: Option<i64> }`
plus `Report::report_json()`, and Python returns the parsed report `dict`. See
[API_CONSISTENCY.md](docs/API_CONSISTENCY.md#api-map) for the full comparison.

## Installation

No packages are published; use the checkout and supply your own
`game-recomp-cli` binary (built by the separate GameReComp project).

```sh
# Python: add the package to PYTHONPATH, or install it editable
PYTHONPATH=python python3 python/examples/basic.py --help
python3 -m pip install -e python

# Rust: run the example in-tree, or add a path dependency in your Cargo.toml
cd rust && blaze test && blaze run --example basic -- <cli> <pe> <project>

# C / C++: compile the C implementation into your program (POSIX, C11/C++17)
cc -std=c11 -Wall -Wextra -Iinclude src/gamerecomp_sdk.c c/examples/basic.c -o basic
c++ -std=c++17 -Wall -Wextra -Iinclude -Icpp/include \
  src/gamerecomp_sdk.c cpp/examples/basic.cpp -o basic_cpp
```

The C/C++ bindings use POSIX process APIs (`fork`/`execvp`/`pipe`/`waitpid`)
and are intended for Linux and macOS, not Windows. The Rust and Python
bindings use portable standard-library process APIs. Configuration is
environment-driven where idiomatic: Python takes an explicit `--cli` argument,
then reads `GAMERECOMP_CLI`, then uses the bare name `game-recomp-cli`; the
other bindings take the CLI path as an argument. Because `Client` validates the
path with `Path(...).is_file()` before spawning, pass an absolute path (a bare
name is only accepted if it exists in the current directory). Never hard-code a
user path or credential.

Version status: the Rust crate declares `0.1.0`; there is no C header version
macro, no Python `__version__`, and no published release process. See
[API_CONSISTENCY.md](docs/API_CONSISTENCY.md#installation-and-versioning).

## Quick start

```sh
# Python (exports + example flags: --pe, --project, --cli, --inspect, --report-path)
GAMERECOMP_CLI=/path/to/game-recomp-cli \
  PYTHONPATH=python python3 python/examples/basic.py --pe game.exe --project /tmp/project

# Rust
cd rust && blaze run --example basic -- /path/to/game-recomp-cli game.exe /tmp/project

# C / C++ proof scripts (use a temporary fake CLI; no real recompile)
sh tests/test_c_sdk.sh
sh tests/test_cpp_sdk.sh
```

Exact per-language commands are in [EXAMPLES.md](docs/EXAMPLES.md).

## Guides

- [API consistency across bindings](docs/API_CONSISTENCY.md) — API map, error
  handling, error-output capture, platform support, versioning, troubleshooting
  and limitations.
- [Architecture and boundary](docs/ARCHITECTURE.md) — what lives here vs. in
  GameReComp, and the SDK contract.
- [Proven examples](docs/EXAMPLES.md) — exact commands for all four bindings.
- [C SDK guide](docs/C.md), [C++ SDK guide](docs/CPP.md),
  [Rust SDK guide](docs/RUST.md), [Python SDK guide](docs/PYTHON.md) — API
  reference, full example, and error cases per language.
- [Real end-to-end run](docs/REAL_RUN.md) — a historical record of a real PE32
  game recompiled through the Python binding.
- [Security policy](docs/SECURITY.md) — environment-only configuration, never
  commit secrets, and error sanitization.
- [Contributing](docs/CONTRIBUTING.md) — the no-second-recompiler rule, how to
  add a binding, and the test matrix.

## Verification

Run all four checks locally:

```sh
PYTHONPATH=python python3 -m pytest python/tests -q   # 13 passed
(cd rust && blaze test)                               # 15 unit tests + example build
sh tests/test_c_sdk.sh                                # C SDK: PASS
sh tests/test_cpp_sdk.sh                              # C++ SDK: PASS
```

CI is configured in [`.github/workflows/ci.yml`](.github/workflows/ci.yml) to
run these same four commands on every push and pull request. It is one matrix
job with `fail-fast: false` and 8 independent combinations (2 runners × 4
checks). The table below records what each check covers and the **local**
result observed at base `d931c02` on macOS; it does not assert a live CI
status, which must be checked at the repository's CI page.

| Check | Command | Ubuntu | macOS | What it verifies | Local result |
|---|---|:---:|:---:|---|---|
| Python | `PYTHONPATH=python python3 -m pytest python/tests -q` | ✓ | ✓ | 13 tests: CLI/PE validation, success, CLI failure, `inspect`, `recompile_to_path` (subprocess is mocked) | 13 passed |
| Rust | `cd rust && blaze test` | ✓ | ✓ | 15 unit tests, including 3 Unix fake-CLI subprocess tests; example is built | 15 passed |
| C | `sh tests/test_c_sdk.sh` | ✓ | ✓ | `-Wall -Wextra` compile, fake-CLI report path, project-dir validation, exec error, `EINTR` handling | PASS |
| C++ | `sh tests/test_cpp_sdk.sh` | ✓ | ✓ | `-Wall -Wextra` compile, fake-CLI report, `Report::score()` present/absent (no exception test) | PASS |

The source of truth for the CI matrix is
[`.github/workflows/ci.yml`](.github/workflows/ci.yml); see
[CONTRIBUTING.md](docs/CONTRIBUTING.md#test-matrix) for what each check
verifies. The image in [`docs/screenshots/`](docs/screenshots/) is an
illustrative screenshot of the GameReComp GUI; it is context for the CLI this
SDK wraps, not evidence of this SDK's correctness.

## Honest limitations

- This SDK does not implement instruction lifting, decompilation, or game
  acceptance; it delegates all of that to GameReComp.
- A successful call means the pipeline produced a report, not that a game
  renders or plays correctly.
- The SDK does not validate PE architecture or format; acceptance of PE32+,
  .NET, 16-bit NE or any other input is the upstream CLI's responsibility.
- C/C++ are POSIX-only; CI covers Ubuntu and macOS, not Windows.
- No live CI status, original-Windows oracle, or 100% game compatibility is
  claimed. See [API_CONSISTENCY.md](docs/API_CONSISTENCY.md#limitations).
