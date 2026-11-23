# Licence inventory and verification

This document records the licence inventory for the **GameRecomp-Recompiler
SDK**, the upstream attribution chain, and the checks run at commit
`d931c0240b612a255c2637b78290e9bf5e299c47` (branch
`feature/sdkprod-9-licenses`). It is inventory documentation, not legal advice.

## 1. Project licence

| Item | Value | Source |
|---|---|---|
| SPDX identifier | `AGPL-3.0-only` | `LICENSE` (first line) |
| Human-readable statement | GNU Affero General Public License v3.0 only; full text at https://www.gnu.org/licenses/agpl-3.0.txt | `LICENSE` |
| Rust crate metadata | `license = "AGPL-3.0-only"` | `rust/Cargo.toml` |
| Notice / upstream attribution | `NOTICE` | repository root |
| Scope statement | Thin client SDK; no recompiler source copied | `LICENSE`, `README.md`, `NOTICE` |

The SDK is a client of GameReComp, which is also `AGPL-3.0-only` (confirmed in
GameReComp's `LICENSE` and `Cargo.toml`).

## 2. Licence consistency checks

| Manifest / doc | Present? | Declared licence | Consistent with `AGPL-3.0-only`? |
|---|:---:|---|---|
| `LICENSE` | yes | `AGPL-3.0-only` (SPDX) | yes — source of truth |
| `rust/Cargo.toml` | yes | `license = "AGPL-3.0-only"` | yes |
| `rust/Cargo.lock` | yes | no licence field (generated; package `gamerecomp-sdk` 0.1.0 only, no external crates) | n/a — nothing to conflict |
| `pyproject.toml` / `setup.py` / `setup.cfg` | no | — | no conflicting metadata; Python package ships no packaging metadata, so no licence field is published to PyPI |
| `CMakeLists.txt` / `CMakePresets.json` | no | — | no conflicting metadata; C/C++ are built by `tests/test_c_sdk.sh` and `tests/test_cpp_sdk.sh` with `cc`/`c++` |
| `README.md`, `docs/*.md` | yes | no licence statement that conflicts with AGPL-3.0-only | yes |
| `.github/workflows/ci.yml` | yes | no licence statement | n/a |

Result: **no licence conflict found**. Every licence declaration that exists
uses `AGPL-3.0-only`.

Follow-up outside this change's scope (source/manifests are not owned here):
per-file SPDX headers are absent from the C, C++, Rust and Python source files,
and there is no `pyproject.toml` or CMake metadata in which to repeat the
`AGPL-3.0-only` identifier. Adding those is a separate task.

## 3. Vendored code and dependencies

| Area | Contents | Third-party licence obligations in this repo |
|---|---|---|
| `include/`, `src/` (C) | Thin CLI invoker: `fork`/`exec`, `waitpid`, standard C/POSIX headers only | none |
| `cpp/` | Header-only RAII wrapper; C++ standard library only | none |
| `rust/` | One crate, zero external dependencies (`Cargo.lock` lists only `gamerecomp-sdk`) | none |
| `python/` | One package; `argparse`, `json`, `os`, `pathlib`, `subprocess`, `unittest` (stdlib) | none |
| `tests/`, `docs/`, `c/examples/`, `cpp/examples/` | Shell test drivers, docs, examples | none |

No third-party source file, binary, or copied licence text is tracked in this
repository. `docs/screenshots/gui-functions-tab.jpg` is a screenshot of the
GameReComp GUI used as visual proof, not third-party source code.

## 4. Upstream attribution: PCRecomp → GameReComp → SDK

| Component | Upstream | Licence | Relation |
|---|---|---|---|
| PCRecomp | https://github.com/sp00nznet/pcrecomp — Ned Heller (@sp00nznet) and contributors | MIT, Copyright (c) 2026 sp00nz | Upstream toolbox. GameReComp's `tools/` and `runtime/` trees are based on PCRecomp and are used/extended there under GameReComp's `NOTICE` and `LICENSES/`. No PCRecomp source is copied into this SDK. |
| GameReComp | https://github.com/heaven-hm/GameReComp | AGPL-3.0-only | The actual recompiler and CLI (`game-recomp-cli run`) that this SDK invokes. Not bundled: users supply the CLI path via `GAMERECOMP_CLI`. |
| GameReComp third-party code | See GameReComp `NOTICE` and `LICENSES/` | zlib licence (`blast-zlib.txt`), igi-recomp MIT (`igi-recomp-MIT.txt`), prior-MIT (`GameReComp-prior-MIT.txt`) | Held by GameReComp, not by this SDK; recorded here so the chain is traceable. |
| This SDK | https://github.com/heaven-hm/GameRecomp-Recompiler | AGPL-3.0-only | Language bindings + examples only; delegates all lifting and verification to GameReComp. |

The SDK therefore has an **indirect** upstream relationship to PCRecomp: it
wraps GameReComp's CLI, and GameReComp builds on PCRecomp. This SDK itself
contains no PCRecomp-derived code and makes no claim to be a recompiler.

## 5. Generated-artifact verification

Command run in the worktree at the base commit:

```sh
git ls-files | wc -l
git ls-files | grep -Ec '(^|/)(target|__pycache__)(/|$)|(\.pyc$)'
```

| Check | Result |
|---|---|
| Tracked files total | 28 |
| Tracked paths matching `target/`, `__pycache__/` or `*.pyc` | **0** |
| Broader scan (also `htmlcov/`, `.pytest_cache/`, `*.o`, `basic`, `basic_cpp`, `.coverage`, `coverage.xml`) | 0 |
| Untracked, non-ignored files | 0 |
| `git check-ignore rust/target/debug/x` | ignored (via `.gitignore`) |
| `git check-ignore python/gamerecomp_sdk/__pycache__/x.pyc` | ignored (via `.gitignore`) |

Result: **no generated artifacts remain tracked**. `.gitignore` already
covers `rust/target/`, `__pycache__/`, `*.pyc`, `.pytest_cache/`, `*.o`,
`basic`, `basic_cpp` and coverage output; a generic `target/` rule was added
so a Cargo target directory is ignored regardless of where the build was
invoked from.

## 6. Limits

- Verification is static (tracked-file scan and manifest/doc inspection); no
  live game, CLI binary or network access was used.
- Upstream licence statements about PCRecomp, GameReComp and their
  third-party files were read from the local checkouts of those projects, not
  re-fetched from the network; versions are the ones present locally.
- This document does not change any source file, manifest or CI workflow.
