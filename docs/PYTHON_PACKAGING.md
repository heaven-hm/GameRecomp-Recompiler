# Python SDK packaging and release guide

This document covers the packaging layer of the Python SDK: metadata, version
consistency, typing, the report validation contract, timeouts/cancellation, the
opt-in real-CLI integration hook, and building wheels/sdists.

The API guide stays in [`docs/PYTHON.md`](PYTHON.md). This document is about
**how the package is built and verified**, not how to call it.

## Ownership boundary

The Python SDK is split across two coordinated changes:

| Concern | Files |
|---|---|
| SDK implementation (`Client`, `RecompileError`, helpers, `py.typed`) | `python/gamerecomp_sdk/__init__.py` (SDK-code change) |
| Packaging, metadata, docs, packaging/integration tests | `pyproject.toml`, `python/README.md`, `python/tests/test_packaging.py`, `python/tests/test_integration_real_cli.py`, this file |

The packaging tests assert the **contract** the SDK implementation must meet.
Checks for features supplied by the SDK-code change skip with an explicit
reason when that change is not present, so the suite is green both before and
after the two changes are merged. CI is not modified by this change; the
existing `python` job already runs `python3 -m pytest python/tests -q`.

## Layout

```
pyproject.toml                              # build-system + project metadata (root)
python/README.md                            # long description for the package
python/gamerecomp_sdk/__init__.py           # SDK implementation (separate owner)
python/gamerecomp_sdk/py.typed              # PEP 561 marker (separate owner)
python/tests/test_sdk.py                    # existing fake-CLI unit tests
python/tests/test_packaging.py              # metadata + contract tests (this change)
python/tests/test_integration_real_cli.py   # opt-in real-CLI test (this change)
docs/PYTHON_PACKAGING.md                    # this file
```

The build backend is **setuptools** (`setuptools.build_meta`). Package discovery
is rooted at `python/`, so the import name is `gamerecomp_sdk` when installed,
matching the `PYTHONPATH=python` workflow used in CI.

## Metadata and dependencies

- `name = "gamerecomp-sdk"`, `license = "AGPL-3.0-only"` (SPDX, matching
  `rust/Cargo.toml`), `requires-python = ">=3.10"`.
- The code uses PEP 604 unions (`str | os.PathLike`) and built-in generics
  (`list[str]`), so 3.10 is the honest floor.
- **Runtime dependencies are empty** — the SDK is standard-library only.
- Extras: `test` (`pytest`, `tomli` on Python < 3.11), `build`, and `dev`
  (superset of both, plus `twine`).

```sh
python3 -m pip install -e '.[dev]'
```

## Version consistency

`pyproject.toml` is the single source of truth for the Python package version.
It must match the other bindings' versions; `test_packaging.py` enforces this
against `rust/Cargo.toml`, and checks the SDK's `__version__` when exposed:

| Location | Value |
|---|---|
| `pyproject.toml` → `[project].version` | `0.3.0` |
| `rust/Cargo.toml` → `[package].version` | `0.3.0` |
| `gamerecomp_sdk.__version__` (SDK-code change) | expected `0.3.0` |

When releasing, bump **all** rows together in one commit, then rebuild the
distributions. Do not bump only one binding.

## Type annotations (PEP 561)

The package declares `Typing :: Typed` and ships `py.typed` in
`[tool.setuptools.package-data]`. The SDK-code change owns the marker file and
the inline annotations on `Client.__init__`, `inspect`, `recompile`,
`recompile_to_path`. `test_packaging.py` verifies public-method annotations via
`inspect.signature` and skips the marker check when the marker is not yet
present.

## Report validation helper

`report.json` is pipeline-owned, but callers need a stable way to sanity-check
it. The packaging test module provides `validate_report(report)` — a strict-on-
type, tolerant-of-extra-fields validator:

- `report` must be a JSON object (dict);
- `score` must be an `int` (not `bool`) in `[0, 100]`;
- `stages`, when present, must be a list of objects;
- `outcome`, when present, must be an object.

It returns the report unchanged on success and raises `AssertionError` with a
descriptive message on the first violation. It is used by the packaging tests
and is the reference shape for the opt-in integration test. If the SDK-code
change exposes its own `validate_report`, the packaging test exercises that
implementation instead and skips otherwise.

## Timeout and cancellation

Long recompiles must be boundable. The SDK-code contract is:

- `Client(..., timeout: float | None = ...)` accepts a per-call subprocess
  timeout (seconds); `None` means no timeout;
- a cancel mechanism (for example `Client.cancel()`) requests cancellation of
  an in-flight run and surfaces it through `RecompileError`.

`test_packaging.py` inspects the signatures and skips each check with an
explicit reason until the SDK-code change provides it. Neither the default
timeout nor cancellation changes the existing fake-CLI behavior in
`test_sdk.py`.

## Real-CLI integration hook (opt-in)

`python/tests/test_integration_real_cli.py` never runs in the default suite:
it is skipped unless every required condition is explicitly set. This keeps CI
fast and deterministic while leaving a documented hook for a real end-to-end
run.

```sh
GAMERECOMP_RUN_REAL_INTEGRATION=1 \
GAMERECOMP_CLI=/path/to/game-recomp-cli \
GAMERECOMP_TEST_PE=/path/to/game.exe \
python3 -m pytest python/tests/test_integration_real_cli.py -q
```

| Variable | Meaning |
|---|---|
| `GAMERECOMP_RUN_REAL_INTEGRATION` | Set to `1`/`true`/`yes` to opt in. Without it the whole module is skipped. |
| `GAMERECOMP_CLI` | Path to a real `game-recomp-cli` binary (required when opted in). |
| `GAMERECOMP_TEST_PE` | Path to a real PE32 input (required when opted in). |
| `GAMERECOMP_TEST_PROJECT` | Optional project directory; a temp dir is used when unset. |
| `GAMERECOMP_TEST_TIMEOUT` | Optional timeout in seconds (`GAMERECOMP_TEST_TIMEOUT=600`). |

When opted in, the test calls `inspect()` and `recompile()` against the real
CLI and validates the returned report shape. The skip reason is always
explicit so `pytest -rs` shows exactly what was missing. No real CLI, PE
binary, or game asset is required to build or unit-test the package.

## Build and verify

```sh
# Unit/metadata tests (no native toolchain needed):
python3 -m pytest python/tests -q

# Wheel + sdist from the repository root:
python3 -m build --outdir dist

# Offline / no-network build using the already-installed backend:
python3 -m build --no-isolation --outdir dist
```

The isolated build (the default) is the supported path: it installs the
declared backend into a clean venv. `--no-isolation` works only when the host's
setuptools environment is clean; on hosts with a stray `pbr` setuptools entry
point and a setuptools release that no longer ships `pkg_resources`, it can
fail while loading `pbr`'s `egg_info` writer. That is a host-environment issue,
not a package issue — use the default isolated build in that case.

`python -m build` produces:

- `dist/gamerecomp_sdk-<version>-py3-none-any.whl`
- `dist/gamerecomp_sdk-<version>.tar.gz`

The wheel is pure Python (`py3-none-any`) because the SDK only shells out to the
CLI. Install the wheel into a fresh environment and run `python -c "import
gamerecomp_sdk"` as a smoke test before publishing. The `build` extra pins the
tools needed for these commands.

The sdist ships the package source, `pyproject.toml`, `python/README.md` and
`LICENSE`. It does **not** ship `python/tests/` (setuptools only includes tests
when a `MANIFEST.in` or SCM plugin is added) — tests are always run from the
source tree. The `py.typed` marker and SDK helpers land with the SDK-code
change; until then the built wheel declares `Typing :: Typed` but cannot include
the marker file it does not yet own.

## Releasing

1. Bump the version everywhere listed under [Version consistency](#version-consistency).
2. Run `python3 -m pytest python/tests -q` and `python3 -m build`.
3. (Optional) `python3 -m twine check dist/*` from the `dev` extra.
4. Verify the wheel imports in a clean venv.
5. Upload with twine when a repository is configured. This change does **not**
   configure CI publishing.
