# Proven examples

All four bindings call the same CLI contract:
`<cli> run <pe> --project-dir <project> --no-table --no-progress`, then check
for `<project>/reports/report.json`. The automated checks use fake CLIs or a
mocked subprocess so they prove SDK plumbing without a long recompile.

Pass the CLI location via `GAMERECOMP_CLI` (Python) or an argument — never
hard-code a path or credential — and sanitize captured error text before
sharing it; see [SECURITY.md](SECURITY.md#error-sanitization). The full
Ubuntu/macOS check matrix is in the [README](../README.md#verification) and in
[CONTRIBUTING.md](CONTRIBUTING.md#test-matrix).

## Installation

No packages are published. Work from a checkout of this repository and supply
a `game-recomp-cli` binary built by the separate GameReComp project.

```sh
# Python
PYTHONPATH=python python3 python/examples/basic.py --help
python3 -m pip install -e python          # optional editable install

# Rust (no external crates)
cd rust && cargo test

# C / C++ (POSIX compilers)
cc -std=c11 -Wall -Wextra -Iinclude src/gamerecomp_sdk.c c/examples/basic.c -o basic
c++ -std=c++17 -Wall -Wextra -Iinclude -Icpp/include \
  src/gamerecomp_sdk.c cpp/examples/basic.cpp -o basic_cpp
```

## Python

```sh
PYTHONPATH=python python3 python/tests/test_sdk.py
PYTHONPATH=python python3 -m pytest python/tests -q
GAMERECOMP_CLI=/path/to/game-recomp-cli \
  PYTHONPATH=python python3 python/examples/basic.py --pe game.exe --project /tmp/project
```

The example also supports `--cli`, `--inspect` (print the CLI's PE inspection
JSON) and `--report-path` (print the `report.json` path instead of a score
summary).

Verified: **13 tests pass**. They cover missing CLI, missing PE, successful
report, CLI failure, `inspect()` success/missing-PE/failure/invalid-JSON, and
`recompile_to_path()` success/missing-PE/failure/missing-report plus the
delegating `recompile()`. The tests mock `subprocess.run` with
`unittest.mock`; they assert the exact command vector but do not execute the
fake CLI script created in `setUp`.

## Rust

```sh
cd rust
cargo test                                                  # 15 unit tests
cargo run --example basic -- /path/to/game-recomp-cli game.exe /tmp/project
```

Verified: `cargo test` passes 15 tests — 12 parser/validation tests plus 3
Unix-only tests that launch a shell-script fake CLI end to end. `cargo test`
also builds `examples/basic.rs`. The example prints the report path, the
parsed score (`Option<i64>`) and the raw `report.json` text via
`Report::report_json()`.

## C

```sh
sh tests/test_c_sdk.sh
# manual:
cc -std=c11 -Wall -Wextra -Iinclude src/gamerecomp_sdk.c c/examples/basic.c -o basic
./basic /path/to/game-recomp-cli game.exe /tmp/project
```

Verified: the script compiles cleanly with `-Wall -Wextra` under a C compiler,
runs a fake CLI that writes a report, and checks the returned report path, a
missing/non-directory `project_dir`, a non-executable CLI (OS exec reason), and
`EINTR` handling during output capture. It prints `C SDK: PASS`.

## C++

```sh
sh tests/test_cpp_sdk.sh
c++ -std=c++17 -Wall -Wextra -Iinclude -Icpp/include \
  src/gamerecomp_sdk.c cpp/examples/basic.cpp -o basic_cpp
```

Verified: the wrapper compiles and the example prints the report path and
`Report::score()`. The proof script checks two cases: a report with
`"score":70` (`score=70`) and one with no score (`score=-1`). It prints
`C++ SDK: PASS`. It does **not** test the exception path, even though both
`Client` construction and `recompile()` throw `std::runtime_error` on failure.
Compiling `src/gamerecomp_sdk.c` as C++ with clang emits `-Wwritable-strings`
warnings; the build still succeeds.

## Error handling

The failure surface differs by binding; see the
[error-handling table](API_CONSISTENCY.md#error-handling) for the full matrix.

| Condition | C | C++ | Rust | Python |
|---|---|---|---|---|
| Missing CLI | `-1` from `gr_client_init` | exception | `Error::MissingCli` | `RecompileError` |
| Missing PE | `-1`, "PE32 input does not exist" | exception | `Error::MissingInput` | `RecompileError` |
| Non-zero CLI exit | `-1`, captured tail in `gr_last_error()` | exception with tail | `Error::Failed` (exit code, no tail) | `RecompileError` with tail |
| No `report.json` | `-1`, "pipeline produced no report.json" | exception | `Error::NoReport` | `RecompileError` |

A successful call is a report path or parsed report — never a gameplay result.

## Troubleshooting

| Symptom | Action |
|---|---|
| `CLI binary does not exist` | Set `GAMERECOMP_CLI` or pass a valid path; Rust/Python require a regular file. |
| `failed to execute CLI …: Permission denied` | Make the CLI executable; C reports the OS reason via a separate error pipe. |
| `project_dir is not an existing directory` | Create the project directory before calling C/C++ (they pre-check it). |
| `pipeline produced no report.json` | Check that the CLI build accepts the arguments and writes the report. |
| `score=-1` / `score=None` / missing key | Different "score absent" conventions per binding; see [API_CONSISTENCY.md](API_CONSISTENCY.md#report-representations). |

## Real game example (from the parent project)

The upstream GameReComp project documents rebuilding OUCC Tetris 1.3 x86
(97 imports bound, 0 unresolved, 3 non-blank 310×418 frames) in its own
`docs/NEW_GAME_DOWNLOADS.md` and `docs/PROOF_EVIDENCE.md`. Those files are in
the GameReComp repository, not this SDK. That is evidence for the pipeline,
not for this SDK alone. A separate historical SDK invocation is recorded in
[REAL_RUN.md](REAL_RUN.md).
