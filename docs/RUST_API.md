# Rust API reference

## `Client::new(cli_path) -> Result<Client, Error>`

Validates that `cli_path` is a regular file. Use an absolute path.

## `Client::inspect(pe) -> Result<Value, Error>`

Runs `<cli> inspect <pe> --json` and parses the CLI's JSON output.

## `Client::recompile(pe, project) -> Result<Report, Error>`

Runs the stable SDK contract, then returns:

```rust
pub struct Report {
    pub path: PathBuf,    // <project>/reports/report.json
    pub score: Option<i64>,
}
impl Report { pub fn report_json(&self) -> String; }
```

## `Error`

Typed variants carry the CLI path, PE path, exit status or captured output
context. No variant asserts a gameplay verdict.

## Packaging and verification

```sh
cd rust && blaze check && blaze test
```

Blaze is the supported build path. The published artifact is the crate source
from `rust/`; no binary is claimed as universally portable.
