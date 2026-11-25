# CI: SDK verification matrix

[`.github/workflows/ci.yml`](../.github/workflows/ci.yml) is the source of truth
for automated verification. It is intentionally small: it runs the same four
fake-CLI checks that a developer runs locally, on both supported runners, and
adds one secret-gated end-to-end job. It does not build or re-implement the
GameReComp pipeline (see [ARCHITECTURE.md](ARCHITECTURE.md)).

## Triggers and defaults

| Trigger | Behaviour |
|---|---|
| `push` | Full matrix on every branch. |
| `pull_request` | Full matrix, including forks. |
| `workflow_dispatch` | Manual re-run, useful after configuring the real-CLI secret. |

- Permissions are limited to `contents: read`; no write token is granted to a
  workflow run.
- `concurrency` cancels superseded runs on the same ref, so a force-push does
  not leave stale checks running.
- `fail-fast: false` keeps both operating systems reporting independently.
- Every job has a `timeout-minutes` bound.
- All `run` steps use `bash`, so `pipefail` is available on both runners.
- `actions/checkout` uses `persist-credentials: false`; no CI token is left in
  the checked-out repository's git config.

## Matrix

Four independent jobs per runner (`ubuntu-latest`, `macos-latest`):

| Check | Command | What it proves |
|---|---|---|
| Python | `python3 -m pytest python/tests -q` with `PYTHONPATH=python` | Missing CLI, missing PE, success and CLI-failure paths. |
| Rust | `blaze test` inside `rust/` | Unit tests and example build, no external crates. |
| C | `sh tests/test_c_sdk.sh` | `-Wall -Wextra` compile, fake-CLI report path, EINTR handling. |
| C++ | `sh tests/test_cpp_sdk.sh` | `-Wall -Wextra` compile, score present/absent paths. |

The checks are unchanged from the documented local commands in
[README.md](../README.md#verification) and
[CONTRIBUTING.md](CONTRIBUTING.md#test-matrix). CI only wraps them with tool
setup, caching and log capture.

## Pinned tool versions

Actions are referenced by full commit SHA (with a human-readable version
comment) rather than a floating tag, and language tools are pinned to exact
versions in one top-level `env` block:

| Tool | Pinned version |
|---|---|
| `actions/checkout` | `v4.2.2` (`11bd7190…`) |
| `actions/setup-python` | `v5.6.0` (`a26af69b…`) |
| `actions/cache` | `v4.2.4` (`0400d5f6…`) |
| `actions/upload-artifact` | `v4.6.2` (`ea165f8d…`) |
| Python | `3.12.9` |
| pytest | `8.3.4` |
| Rust | `1.83.0` (installed with `rustup`, `minimal` profile) |

The Rust crate uses edition 2021, so the pinned compiler is sufficient. To bump
a tool, change only the value in the `env` block (or the action SHA plus its
comment) in a dedicated commit and verify the matrix before merging. Runner
labels (`ubuntu-latest`, `macos-latest`) are deliberately left floating; the
toolchain and the actions that run on them are pinned.

## Caching policy

Caches are opt-in per check and scoped so they cannot hide a toolchain change:

- **Python:** `pip`'s download cache lives in `PIP_CACHE_DIR` under the runner's
  temp directory (the macOS default `~/Library/Caches/pip` differs from
  Linux's `~/.cache/pip`, so an explicit path keeps both runners identical).
  The key includes runner OS, Python version and pytest version.
- **Rust:** Cargo registry/index/git state and `rust/target`, keyed by runner
  OS, pinned Rust version and `hashFiles('rust/Cargo.lock', 'rust/Cargo.toml')`.

There are **no `restore-keys`**, so a cache from another OS, Python/pytest
version, Rust version or dependency graph is never restored. Cross-OS archive
is explicitly disabled. No secret, token, CLI binary or report is ever placed
in a cache path. Caches are a speed optimisation only; a cold cache must still
pass every check.

## Test logs

Each check pipes stdout and stderr through `tee` into a per-job log directory
under `runner.temp` (never the checkout). `pipefail` preserves a failing check's
exit status through the pipe. The log is uploaded with `actions/upload-artifact`
using `if: always()`, so a failed matrix cell still produces a downloadable
`sdk-log-<check>-<os>` artifact. Artifacts are retained for 14 days and contain
only captured test output.

## Real CLI integration job

`real-cli` runs after `verify` succeeds. It invokes
`tests/real_cli_integration.sh`, which is the integration workstream's entry
point for a genuine end-to-end run; CI does not inspect or duplicate that
script's logic. The CLI is passed through the `GAMERECOMP_CLI` environment
variable, resolved from either the `GAMERECOMP_CLI` repository secret or a
repository variable of the same name (secret wins).

GitHub Actions does not allow the `secrets` context in a job-level `if`, so
presence is detected in a step and written to `$GITHUB_OUTPUT`; the run and
upload steps are gated on that output. The secret's value is never echoed.

Behaviour:

- **CLI configured:** the script runs with `PYTHONPATH=python` and
  `GAMERECOMP_CLI` set. Its stdout/stderr and its own evidence files
  (`GR_CLI_INTEGRATION_OUT`, including `results.json` and captured JSON) are
  uploaded together as `sdk-log-real-cli`.
- **CLI not configured:** the job reports an explicit `::notice` and runs a
  green "Skip real CLI integration" step. The integration script is not
  called, and no artifact is uploaded. Fork PRs, which cannot read secrets,
  take this path automatically.
- **CLI configured but the script fails:** the job fails and the log artifact
  is still uploaded. A configured CLI is an explicit opt-in; a broken
  integration must not be silently skipped.

Fork PRs cannot access repository secrets, so the real-CLI job always skips
there. Configure it on the base repository (or supply a repository variable)
to enable end-to-end verification on trusted runs.

## Local validation for workflow-only changes

No build is required for a CI/doc-only change. Validate the workflow syntax
and review the pinned SHAs:

```sh
python3 -c 'import pathlib, yaml; yaml.safe_load(pathlib.Path(".github/workflows/ci.yml").read_text()); print("YAML OK")'
```

The same four checks can be run locally with the commands in the matrix table
above.

## Limitations

- A green matrix proves SDK plumbing with fake CLIs; it is not a gameplay or
  decompilation claim. See [REAL_RUN.md](REAL_RUN.md) for a documented real run
  and [README.md](../README.md#honest-limitations) for scope.
- The real-CLI job needs a CLI binary/URL supplied out of band; CI does not
  build one and will skip cleanly when none is configured.
- Action SHAs and tool versions are pinned but must be bumped manually; this
  workflow does not auto-update dependencies.
