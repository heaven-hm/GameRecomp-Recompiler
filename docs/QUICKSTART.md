# Quickstart (pure Rust SDK)

1. Build `game-recomp-cli` from GameReComp `v0.3.0`.
2. Install Blaze: `curl -fsSL https://ix.dev/blaze | sh`.
3. Add the dependency from `GameRecomp-SDK` tag `v0.3.0`.
4. Run `cd rust && blaze test` to verify the SDK locally.
5. Run the example: `blaze run --example basic -- /abs/path/to/game-recomp-cli game.exe out`.

The CLI path must be absolute. Errors are typed; never convert a failed spawn
or a missing `report.json` into a successful result. See `docs/RUST_API.md`.
