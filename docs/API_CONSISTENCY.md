# API consistency across bindings

This document maps the **actual** public API of the four bindings in this
repository at base commit `d931c02`. It is the cross-language reference for
installation, error handling, platform support, versioning, troubleshooting and
limitations. The per-language guides ([C](C.md), [C++](CPP.md),
[Rust](RUST.md), [Python](PYTHON.md)) remain the detailed references for a
single binding.

Everything here is derived from the code in this repository: the C header and
implementation (`include/gamerecomp_sdk.h`, `src/gamerecomp_sdk.c`), the C++
header (`cpp/include/gamerecomp_sdk.hpp`), the Rust crate (`rust/src/lib.rs`,
`rust/Cargo.toml`), the Python package (`python/gamerecomp_sdk/__init__.py`),
the examples, the proof scripts in `tests/`, and
[`.github/workflows/ci.yml`](../.github/workflows/ci.yml). Nothing here is a
claim about behaviour of the upstream GameReComp pipeline that this SDK wraps;
see [ARCHITECTURE.md](ARCHITECTURE.md) for that boundary.

## Contract

Every binding follows the same small contract:

1. It takes a CLI path (an explicit argument in C/C++/Rust; an explicit
   argument, `$GAMERECOMP_CLI`, or the bare name `game-recomp-cli` in Python,
   in that order of precedence) and rejects a missing/non-file path before
   running anything.
2. It starts the CLI as a child process with an argument vector — **no shell
   interpolation** of the paths:
   `run <pe> --project-dir <project> --no-table --no-progress`.
3. It requires a zero CLI exit status.
4. It requires `<project>/reports/report.json` to exist after that successful
   exit.
5. Every failure is returned explicitly (`-1`, exception, `Result::Err`, or
   `RecompileError`). No binding fabricates a report.

Beyond this contract the four surfaces are **not** identical. In particular,
there is no single "three operations" shared by all bindings: only Python
exposes PE inspection, and the returned report shape differs in every language.

## API map

| Concern | C | C++ | Rust | Python | JS/TS |
|---|---|---|---|---|---|
| Create / validate client | `gr_client_init_default(&c)` → `0`/`-1` | `gamerecomp::Client()` throws on failure | `Client::from_env()` → `Result<Self, Error>` | `Client()` raises `RecompileError` | `new Client()` throws `RecompileError` |
| Run pipeline | `gr_recompile(&c, pe, proj, buf, size)` → `0`/`-1` | `client.recompile(pe, proj)` → `Report` (throws) | `client.recompile(pe, proj)` → `Result<Report, Error>` | `client.recompile_to_path(pe, proj)` → `Path` | `client.recompileToPath(pe, proj)` → `string` |
| Parsed report body | none; path only | `Report::score()` → `int` | `Report::score: Option<i64>` + `report_json()` → `String` | `client.recompile(pe, proj)` → `dict` | `client.recompile(pe, proj)` → object (typed in `index.d.ts`) |
| Inspect a PE without a project | — | — | — | `client.inspect(pe)` → `dict` | `client.inspect(pe)` → object |
| Read raw report text | caller opens the path | caller opens `Report::path` | `Report::report_json()` | `report_path.read_text()` | `fs.readFileSync(path)` |
| Last error | `gr_last_error(&c)` → `const char *` | exception `what()` | `Error` value | `RecompileError` message | `RecompileError` with typed fields |
| Clean up | `gr_client_free(&c)` | RAII destructor | RAII (drop) | RAII (GC) | GC |

Two Python methods run the pipeline: `recompile()` delegates to
`recompile_to_path()` and then parses the JSON, so both require the same
inputs. `inspect()` is a separate CLI subcommand (`inspect <pe> --json`) and
does not create or require a project directory.

## Report representations

A successful call never returns a whole uniform report object:

| Binding | Success value | "Score absent" convention | Raw JSON access |
|---|---|---|---|
| C | report path copied into a caller buffer | n/a — no parsing | open the returned path |
| C++ | `Report{ std::string path }` | `score()` returns `-1` (also used for unreadable/invalid reports) | open `Report::path` |
| Rust | `Report { path: PathBuf, score: Option<i64> }` | `score == None` | `report_json()` (empty string if unreadable) |
| Python | parsed `dict` from `json.loads` | key simply absent from the dict | `recompile_to_path()` then `read_text()` |

The C++ `-1` sentinel means a genuine score of `-1` cannot be distinguished
from a missing score. Rust's `Option<i64>` and Python's missing key do not have
that ambiguity. The Rust `Report` intentionally keeps two public fields so a
`Report { path, score }` struct literal still compiles.

Score parsing is also not shared: Rust's public `parse_score()` tolerates
whitespace, negatives and malformed duplicate keys, ignores floats and
out-of-range integers, and returns `Option<i64>`; C++'s private helper returns
an `int` within `INT_MIN`..`INT_MAX` with `-1` for absent/invalid input; Python
uses `json.loads` and does no score-specific parsing at all.

## Error handling

| Condition | C | C++ | Rust | Python |
|---|---|---|---|---|
| CLI path missing / not a file | `gr_client_init` → `-1`, "CLI binary does not exist" | ctor throws `std::runtime_error` | `Error::MissingCli(path)` | `RecompileError("CLI binary does not exist: …")` |
| PE path missing / not a file | `gr_recompile` → `-1`, "PE32 input does not exist" | `recompile` throws | `Error::MissingInput(path)` | `RecompileError("PE32 input does not exist: …")` |
| `project_dir` missing or not a directory | `-1`, "project_dir is not an existing directory" | rethrows same message | no explicit check; CLI decides | no explicit check; CLI decides |
| CLI cannot be executed | `-1`, "failed to execute CLI …: <OS reason>" | rethrows same message | `Error::Failed(spawn error text)` | `RecompileError` from `subprocess.run` (OSError) |
| CLI exits non-zero | `-1`, error keeps captured output tail | exception keeps captured tail | `Error::Failed("exit <code>")` (no output tail) | `RecompileError("CLI exited N: <tail>")` |
| Exit 0 but no `report.json` | `-1`, "pipeline produced no report.json" | exception with same message | `Error::NoReport(path)` | `RecompileError("pipeline produced no report.json")` |
| `inspect` returns invalid JSON | n/a | n/a | n/a | `RecompileError("CLI produced invalid JSON: <tail>")` |

Notes:

- C uses a fixed 512-byte `gr_client.error` buffer; at most 511 bytes of the
  CLI's combined stdout/stderr are retained. Always call `gr_client_free()`.
- C++ exposes the C buffer text through `std::runtime_error::what()`.
- Python truncates the combined output tail to the last 2000 characters and
  redirects stderr into stdout before capture.
- Rust's `Error::Failed` contains an OS spawn-error string or an exit code, and
  `Error::NoReport` contains the expected report path. Rust does **not** surface
  captured CLI stdout/stderr in its error variants.

Treat any captured CLI text as untrusted and sanitize it before logging or
sharing; see [SECURITY.md](SECURITY.md#error-sanitization).

## CLI output capture

| Binding | Captures child stdout/stderr | Where it is exposed |
|---|---|---|
| C | yes, combined into one pipe, max 511 bytes kept, `EINTR`-safe | `gr_last_error()` |
| C++ | inherited from C | `what()` |
| Rust | `Command::output()` captures it but the bytes are discarded on failure | nothing; only status/path/IO text in `Error` |
| Python | yes, combined via `stderr=STDOUT`, last 2000 chars on failure | `RecompileError` message |

`gr_client_init` validates that the CLI path exists (`stat`), not that it is an
executable file; the OS `execvp` errno is reported separately through a
close-on-exec pipe. Rust and Python use `is_file()` checks, so a directory or
missing path is rejected before spawning.

## Validation differences

- C/C++ require `project_dir` to be an **existing directory** before invoking
  the CLI. Rust and Python do not pre-check it; the CLI's behaviour decides.
- C/C++ check the PE with `stat()` (any existing path passes). Rust/Python use
  `is_file()`.
- Only Python reads configuration from an environment variable
  (`GAMERECOMP_CLI`); the other bindings take the CLI path as an argument.

## Installation and versioning

There are no published packages for any binding at this base. Install from the
checkout:

| Binding | Use | Install |
|---|---|---|
| C | compile `src/gamerecomp_sdk.c` into your program with `-Iinclude` (C11, POSIX) | no package; copy/build from source |
| C++ | include `cpp/include/gamerecomp_sdk.hpp` and also compile `src/gamerecomp_sdk.c` (C++17) | no package; header + C source |
| Rust | `blaze run --example basic -- <cli> <pe> <project>` inside `rust/`, or add a path dependency | `gamerecomp-sdk = { path = ".../rust" }` |
| Python | `PYTHONPATH=python python3 …` or `python3 -m pip install -e python` | editable/local install only |

A working `game-recomp-cli` binary must be supplied by the caller in every
case; this repository never bundles or downloads it.

Version status:

- The Rust crate declares `version = "0.3.0"` in `rust/Cargo.toml` and is not
  published to a registry.
- The C header has no version macro; the Python package has no `__version__`.
- Public shapes are deliberately kept stable for existing callers: the C
  struct layout and function signatures are unchanged, the Rust `Report` keeps
  its two public fields, and the C++ `Report` stays an aggregate. Treat all of
  this as pre-1.0: source compatibility is intended, but no published semver
  contract or release process exists yet.

## Platform support

| Binding | Implementation | CI runners | Notes |
|---|---|---|---|
| C | `fork`, `execvp`, `pipe`, `waitpid` (POSIX) | `ubuntu-latest`, `macos-latest` | Unix/POSIX only; no Windows support |
| C++ | wraps the POSIX C implementation | `ubuntu-latest`, `macos-latest` | compiled from C++17 source plus the C file |
| Rust | `std::process::Command` (portable std API) | `ubuntu-latest`, `macos-latest` | shell-script fake-CLI tests are `#[cfg(unix)]` |
| Python | `subprocess` (portable stdlib) | `ubuntu-latest`, `macos-latest` | tests mock `subprocess.run` |

The CI workflow is a single matrix job (`fail-fast: false`) with 8
combinations: 2 operating systems × 4 checks. It is the source of truth for
what is automated; local results at this base are recorded in the
[README verification section](../README.md#verification). Live CI status is
not asserted by this document.

## Troubleshooting

| Symptom | Likely cause / action |
|---|---|
| `CLI binary does not exist` even though the CLI is on `PATH` | Python validates with `Path(cli).is_file()`; a bare name is not searched on `PATH`. Set `GAMERECOMP_CLI` to an absolute path or pass `--cli` with one. |
| `CLI binary does not exist` for other reasons | Rust/Python require a regular file; C/C++ require an existing path. |
| `failed to execute CLI …: Permission denied` | The file exists but is not executable; `chmod +x` it. C reports the OS reason explicitly. |
| `project_dir is not an existing directory` (C/C++) | Create the project directory first; C/C++ require it before spawning the CLI. |
| `pipeline produced no report.json` / `Error::NoReport` | CLI exited 0 but wrote no report: check the CLI build/version, arguments and project path. |
| Non-zero exit with a captured tail | Read the tail for the CLI's own error; redact paths/credentials before sharing ([SECURITY.md](SECURITY.md#error-sanitization)). |
| C++ `score=-1` vs Rust `score=None` | Different absent-value conventions; `-1` is also the C++ value for a real score of `-1`. |
| Rust fake-CLI tests skipped | They are `#[cfg(unix)]`; on Windows there is no equivalent script-based test. |
| C++ build emits `-Wwritable-strings` with clang | The C source is compiled as C++ by the proof script; it builds with warnings. Use a C compiler for `src/gamerecomp_sdk.c` where possible. |
| Unix fake CLI "Text file busy" | Give the script a shebang, set the executable bit, and publish it by rename (the Rust tests do this). |

## Limitations

- This is a thin client SDK: no PE parsing, lifting, decompilation, emulation,
  linking or game acceptance lives here.
- A successful call means the CLI produced a `report.json`; it is not evidence
  that a game renders or plays.
- The SDK does not validate PE architecture/format. Whether a particular PE
  (for example PE32+, .NET or 16-bit NE) is accepted is entirely up to the
  upstream CLI.
- C/C++ are POSIX-only; there is no Windows runner in CI.
- No live CI result, original-Windows oracle, or 100% game compatibility is
  claimed here; [REAL_RUN.md](REAL_RUN.md) is a historical record, not a
  reproducible guarantee.
- Python's automated tests mock `subprocess.run`; they prove command
  construction and error handling, not an executed fake-CLI child process.

## Verification

At this base the four local checks were run and all passed:

```sh
PYTHONPATH=python python3 -m pytest python/tests -q   # 13 passed
(cd rust && blaze test)                               # 15 unit tests + example build
sh tests/test_c_sdk.sh                                # C SDK: PASS
sh tests/test_cpp_sdk.sh                              # C++ SDK: PASS
```

See [EXAMPLES.md](EXAMPLES.md) for the commands and
[CONTRIBUTING.md](CONTRIBUTING.md#test-matrix) for what each check covers.

## References

- [README](../README.md) — overview and verification matrix
- [Architecture and boundary](ARCHITECTURE.md)
- [Proven examples](EXAMPLES.md)
- [Security policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)
- [Real end-to-end run](REAL_RUN.md)
- Per-language guides: [C](C.md), [C++](CPP.md), [Rust](RUST.md), [Python](PYTHON.md)
