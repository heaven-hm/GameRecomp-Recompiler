# Rust SDK guide

## Files

- Crate: `rust/Cargo.toml`, library: `rust/src/lib.rs`
- Example: `rust/examples/basic.rs`
- Tests: `rust/src/lib.rs` (`#[cfg(test)]` module)

No external crates are required; the client uses `std::process::Command`.
Unit tests create their own temporary directories and report files with
`std`, and a fake CLI script (on Unix) exercises the full `recompile` path.

## API reference

```rust
pub struct Client { /* cli path */ }
pub struct Report { pub path: PathBuf, pub score: Option<i64> }

pub enum Error {
    MissingCli(String),
    MissingInput(String),
    Failed(String),
    NoReport(String),
}

impl Client {
    pub fn new(cli: impl AsRef<Path>) -> Result<Self, Error>;
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

`Error` implements `Display` and `std::error::Error`, so it composes with
`anyhow`, `?`, and error reporters.

`recompile` succeeds only when the CLI exits successfully and
`<project>/reports/report.json` exists. `score` is extracted by `parse_score`:
whitespace around the key, colon and value is tolerated (pretty-printed
reports work), negative values are supported, similarly named keys such as
`score_notes` are ignored, and floats/overflow/missing fields yield `None`.
`None` means "not parsed", not "zero".

The `Report` struct is unchanged: its two public fields (`path`, `score`) and
direct struct-literal construction remain source-compatible. `report_json()`
re-reads `path` on demand, so no new field was added to the struct.

## Full example

```rust
use gamerecomp_sdk::Client;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        eprintln!("usage: basic <cli> <game.exe> <project-dir>");
        std::process::exit(2);
    }
    let client = match Client::new(&args[0]) {
        Ok(c) => c,
        Err(e) => { eprintln!("init: {e}"); std::process::exit(1); }
    };
    match client.recompile(&args[1], &args[2]) {
        Ok(report) => {
            println!("report={} score={:?}",
                     report.path.display(), report.score);
            println!("json={}", report.report_json().trim());
        }
        Err(e) => { eprintln!("recompile: {e}"); std::process::exit(1); }
    }
}
```

Run:

```sh
cd rust
blaze test
blaze run --example basic -- /path/to/game-recomp-cli /path/to/game.exe /tmp/project
```

## Adding the crate as a dependency

In your `Cargo.toml`:

```toml
[dependencies]
gamerecomp-sdk = { path = "../GameRecomp-Recompiler/rust" }
```

For a published version, replace `path` with a version once released.

## Error cases

| Condition | Variant |
|---|---|
| CLI path not a file | `Error::MissingCli` |
| PE path not a file | `Error::MissingInput` |
| CLI spawn failure or non-zero exit | `Error::Failed` |
| No `report.json` after exit 0 | `Error::NoReport` |

## Notes

- `Client` is `Send`-compatible (it only owns a `PathBuf`); construct one per
  thread for parallel pipelines.
- A successful `Report` is a verified file path, not a gameplay claim.
