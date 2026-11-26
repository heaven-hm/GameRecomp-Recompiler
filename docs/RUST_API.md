# Rust SDK API reference

The `gamerecomp-sdk` crate is a thin, dependency-free Rust client for the
GameReComp PE32/x86 recompilation pipeline. It locates and validates a
`game-recomp-cli` binary, runs it as a child process, and reads the resulting
`reports/report.json`. All lifting/recompilation is done by GameReComp itself;
this crate never reimplements it.

This document is the authoritative reference for the current API. It replaces
the older, shorter sketch at the top of [`RUST.md`](RUST.md).

## Crate metadata

`rust/Cargo.toml` is publish-ready: it declares the AGPL-3.0-only license,
repository/homepage/documentation URLs, five crates.io keywords, two
categories, a minimum supported Rust version, and an explicit `include` list
so only library sources, examples and tests enter the packaged crate.

```sh
cd rust
cargo test
cargo package --list     # inspect the exact files that would be published
cargo package            # build .crate locally (does not publish)
```

## Public API

```rust
pub struct Client { /* validated CLI path, optional timeout */ }
pub struct Report { pub path: PathBuf, pub score: Option<i64> }

#[non_exhaustive]
pub enum Error {
    MissingCli  { path: PathBuf },
    MissingInput{ path: PathBuf },
    Failed      { message: String, status: Option<ExitStatus>,
                  stderr: String, source: Option<std::io::Error> },
    NoReport    { path: PathBuf, source: std::io::Error },
    Version     { message: String, source: Option<std::io::Error> },
    TimedOut    { duration: Duration },
}

impl Client {
    pub fn new(cli: impl AsRef<Path>) -> Result<Self, Error>;
    pub fn new_with_timeout(cli: impl AsRef<Path>,
                            timeout: Option<Duration>) -> Result<Self, Error>;
    pub fn with_timeout(self, timeout: Duration) -> Self;
    pub fn set_timeout(&mut self, timeout: Option<Duration>);
    pub fn timeout(&self) -> Option<Duration>;
    pub fn cli(&self) -> &Path;
    pub fn version(&self) -> Result<String, Error>;
    pub fn recompile(&self, pe: impl AsRef<Path>,
                     project: impl AsRef<Path>) -> Result<Report, Error>;
}

impl Report {
    /// Raw, unmodified text of report.json (empty if the file is unreadable).
    pub fn report_json(&self) -> String;
}

/// Extract the integer `score` field, or None if absent/not an integer.
pub fn parse_score(json: &str) -> Option<i64>;
```

`Report` keeps its original two public fields, so existing
`Report { path, score }` literals and field accesses remain source-compatible.

`Error` implements `Display` and `std::error::Error`, so it composes with
`?`, `anyhow`, and error reporters. Variants that wrap an I/O failure expose
it via `std::error::Error::source(&err)`, allowing callers to distinguish a
spawn failure from a process failure without parsing prose. The enum is
`#[non_exhaustive]`: add a `_ => {}` arm to exhaustive matches.

## Structured errors

| Condition | Variant | Notable fields |
|---|---|---|
| CLI path missing / not a file | `MissingCli` | `path` |
| PE input missing / not a file | `MissingInput` | `path` |
| Spawn or wait I/O failure | `Failed` | `message`, `source = Some(io::Error)` |
| Non-zero CLI exit | `Failed` | `status = Some(..)`, captured `stderr` |
| No readable `report.json` after exit 0 | `NoReport` | `path`, `source = Some(io::Error)` |
| `--version` failed / empty / non-zero | `Version` | `message`, optional `source` |
| Command exceeded configured timeout | `TimedOut` | `duration` |

Example of inspecting a failure:

```rust
use gamerecomp_sdk::{Client, Error};

# fn run() {
let client = Client::from_env().expect("client");
match client.recompile("game.exe", "out") {
    Ok(report) => println!("score = {:?}", report.score),
    Err(Error::Failed { status, stderr, .. }) => {
        eprintln!("CLI status = {status:?}, stderr = {stderr}");
    }
    Err(Error::NoReport { path, source }) => {
        eprintln!("no report at {}: {source}", path.display());
    }
    Err(e) => eprintln!("{e}"),
}
# }
```

## Timeouts

By default `Client` waits indefinitely for the CLI. Attach a timeout at
construction or with the builder:

```rust
use std::time::Duration;
use gamerecomp_sdk::Client;

# fn build() -> Result<(), gamerecomp_sdk::Error> {
let client = Client::from_env()?
    .with_timeout(Duration::from_secs(300));

assert_eq!(client.timeout(), Some(Duration::from_secs(300)));
# Ok(())
# }
```

The timeout applies to **each** CLI command (`recompile`, `version`). When it
elapses the child is killed and reaped, and the call returns
`Error::TimedOut { duration }`. Internally, stdout and stderr are drained on
separate threads so a verbose CLI cannot deadlock on a full pipe, and stdin is
set to null so a CLI can never block waiting for interactive input.

## CLI version query

`Client::version()` runs `<cli> --version` and returns the trimmed output. It
is useful for recording which CLI build produced a report or for asserting a
minimum version before a pipeline run. A banner written to stderr is used as a
fallback when stdout is empty. Non-zero exits and empty output produce
`Error::Version`.

Note that not every CLI build implements `--version`; callers that must
support such builds should treat `Error::Version` as informational. The
`examples/basic.rs` example demonstrates this.

## Path-with-spaces safety

Every path is passed to the CLI as a **separate argv element** via
`std::process::Command::arg`. No shell string is ever built from user paths,
so CLI paths, PE inputs and project directories containing spaces, quotes or
shell metacharacters are safe. A unit test (`recompile_handles_paths_with_spaces`)
exercises a fake CLI, PE and project directory whose names contain spaces.

## Integration tests (real CLI)

`rust/tests/integration.rs` runs against a real CLI when one is configured.
It reads configuration from the environment; when a value is missing it prints
an explicit skip reason and passes, so `cargo test` remains hermetic:

```sh
GAMERECOMP_CLI=/path/to/game-recomp-cli \
GAMERECOMP_TEST_PE=/path/to/game.exe \
    cargo test --test integration -- --nocapture
```

- `GAMERECOMP_CLI` — path to a real `game-recomp-cli` (required for the tests
  to run).
- `GAMERECOMP_TEST_PE` — path to a real PE32 input (required for the
  recompile test).

The tests create and clean up a temporary project directory, apply a bounded
timeout, and assert that the produced report exists and that its score matches
`parse_score`. Paths are supplied through the environment so no machine-specific
location is baked into the repository.

## Running the example

```sh
cd rust
cargo run --example basic -- /path/to/game-recomp-cli game.exe /tmp/project
# optional per-command timeout:
GAMERECOMP_TIMEOUT_MS=300000 \
  cargo run --example basic -- /path/to/game-recomp-cli game.exe /tmp/project
```
