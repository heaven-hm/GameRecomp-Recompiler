# GameRecomp-SDK

**Pure Rust client SDK for GameReComp v0.3.0.** Built from the ground up on
PCRecomp's mechanical recompiler; this crate is the reusable front door, not a
second recompiler, loader or lifter.

Every call validates the CLI path, spawns
`game-recomp-cli run <pe32-x86.exe> --project-dir <dir> --no-table --no-progress`
without a shell, requires exit 0 and a real `<dir>/reports/report.json`, and
returns typed errors instead of a fake score.

## Install

```toml
[dependencies]
gamerecomp-sdk = { git = "https://github.com/heaven-hm/GameRecomp-SDK.git", tag = "v0.3.0" }
```

## Example

```rust,no_run
use gamerecomp_sdk::Client;

fn main() -> Result<(), gamerecomp_sdk::Error> {
    let client = Client::new("/abs/path/to/game-recomp-cli")?;
    let report = client.recompile("game.exe", "out")?;
    println!("report: {}", report.path.display());
    println!("score: {:?}", report.score);
    Ok(())
}
```

Runnable example: `cd rust && blaze run --example basic -- <cli> <pe> <project>`.

## Build and test with Blaze

```sh
curl -fsSL https://ix.dev/blaze | sh     # once
cd rust && blaze test && blaze check
```

Blaze is the only supported Rust build path here; no cargo or rustc command is
used. CI runs the same commands on Linux and macOS.

## Docs

- [Quickstart](docs/QUICKSTART.md)
- [Architecture and boundary](docs/ARCHITECTURE.md)
- [Rust guide](docs/RUST.md) and [API reference](docs/RUST_API.md)
- [Security](docs/SECURITY.md) and [Contributing](docs/CONTRIBUTING.md)

## Scope and limits

Verified scope: PE32/i386 inputs through GameReComp CLI v0.3.0 on the listed
host platforms. This SDK does not claim universal PC-game support, gameplay
equivalence, PE32+/Win64 execution or any renderer compatibility. A report
`score` is a process-level measure; PARTIAL and BLOCKED verdicts stay unchanged.
