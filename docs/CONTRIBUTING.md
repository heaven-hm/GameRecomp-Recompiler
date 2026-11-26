# Contributing

## Layout

- `rust/src/lib.rs` — the entire SDK implementation.
- `rust/tests/` — unit and opt-in integration tests.
- `rust/examples/basic.rs` — runnable example.
- `docs/` — Rust-only guides and security policy.

## Rules

1. Never add a second recompiler, loader or lifter. Delegate to
   `game-recomp-cli`.
2. Keep the contract fail-closed: validate paths, use
   `std::process::Command` without a shell, require exit 0 and a real report.
3. No new dependencies without a measured reason; the crate is std-only.
4. Build and test with Blaze: `cd rust && blaze check && blaze test`. Do not
   use cargo or rustc commands.
5. Never claim gameplay equivalence, 100% support or a PASS verdict from a
   successful SDK call.

## Test matrix

| Check | Command | Platforms | Verifies |
|---|---|---|---|
| Rust check | `cd rust && blaze check` | Linux, macOS | type-check and example build |
| Rust tests | `cd rust && blaze test` | Linux, macOS | unit tests, validation and error paths |

CI runs both checks on `ubuntu-latest` and `macos-latest`.
