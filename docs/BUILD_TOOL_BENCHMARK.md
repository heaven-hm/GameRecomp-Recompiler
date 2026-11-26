# Rust build tool benchmark: Kache vs Blaze

Measured on macOS arm64, 2026-11-26, with the exact repositories at their
current heads. Times are wall-clock from `/usr/bin/time -p`.

| Workload | Tool | Cold | Warm | Build storage |
|---|---|---:|---:|---:|
| GameRecomp-SDK `rust/` (`cargo check` / `blaze check`) | Kache 2.3.1 | 8.25 s | **0.88 s** | **72 KB** target, 29.8 KB cache |
| GameRecomp-SDK `rust/` | Blaze 0.20261105.4 | 2.89 s (cached daemon) / 36.81 s (restart) | 36.81 s | 1.8 GB shared Blaze store |
| GameReComp (`cargo check` / `blaze check`) | Kache 2.3.1 | 542.57 s | **1.25 s** | 1.2 GB target, 1003 MiB cached blobs |
| GameReComp | Blaze 0.20261105.4 | — | — | **failed**: daemon exited 1 (`USAGE`) before listening |

Kache is faster on warm builds, uses dramatically less storage for the small
crate, and works on both repositories. Blaze's daemon repeatedly hung up or
failed on the larger GameReComp crate on this host.

**Decision:** use Kache (`kunobi-ninja/kache`) for Rust builds in this SDK and
its CI. Cargo remains the front-end command; Kache caches compiler outputs.

Reproduce:

```sh
brew install kache || cargo install kache --locked
kache init && kache doctor --probe
CARGO_TARGET_DIR=/tmp/gr-sdk-target cargo check
```
