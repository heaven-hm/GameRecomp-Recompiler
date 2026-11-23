# Proven examples

All examples call the same CLI contract. The tests use a fake CLI that writes
a `report.json`, so they prove SDK plumbing without a long recompile.

## Python

```sh
python3 -m pip install -e python
python3 python/tests/test_sdk.py
GAMERECOMP_CLI=/path/to/game-recomp-cli \
  python3 python/examples/basic.py --pe game.exe --project /tmp/project
```

Verified: 4 tests pass (missing CLI, missing PE, successful report, CLI failure).

## Rust

```sh
cd rust
cargo test
cargo run --example basic -- /path/to/game-recomp-cli game.exe /tmp/project
```

Verified: `cargo test` passes; `basic.rs` prints the report path and score.

## C

```sh
sh tests/test_c_sdk.sh
# manual:
cc -std=c11 -Iinclude src/gamerecomp_sdk.c c/examples/basic.c -o basic
./basic /path/to/game-recomp-cli game.exe /tmp/project
```

Verified: the script compiles cleanly with `-Wall -Wextra` and prints the
report path from a fake CLI.

## C++

```sh
sh tests/test_cpp_sdk.sh
c++ -std=c++17 -Iinclude -Icpp/include \
  src/gamerecomp_sdk.c cpp/examples/basic.cpp -o basic
```

Verified: compiles and prints the report path; failures throw
`std::runtime_error`.

## Real game example (from the parent project)

The underlying CLI was used to rebuild OUCC Tetris 1.3 x86: 97 imports bound,
0 unresolved, 3 non-blank 310×418 frames. That is evidence for the pipeline,
not for this SDK alone; see GameReComp's `docs/NEW_GAME_DOWNLOADS.md` and
`docs/PROOF_EVIDENCE.md`.
