# GameReComp Python SDK

`gamerecomp-sdk` is a thin, typed Python client for the **GameReComp** PE32/x86
static recompilation pipeline. It does not reimplement lifting or decompilation:
it invokes the real `game-recomp-cli` and returns the machine-readable report
the pipeline writes.

> Scope: a successful call means the pipeline produced `reports/report.json`.
> It does **not** mean a game renders or plays correctly. `verification: partial`
> is not a gameplay pass.

## Install

```sh
# From a built wheel or sdist:
python3 -m pip install dist/gamerecomp_sdk-0.1.0-py3-none-any.whl

# From this repository (editable):
python3 -m pip install -e .

# With test/build tooling:
python3 -m pip install -e '.[dev]'
```

Requires Python 3.10+. There are **no runtime dependencies** — only the
standard library (`subprocess`, `json`, `pathlib`).

## Quick start

```python
from gamerecomp_sdk import Client, RecompileError

client = Client()                       # uses $GAMERECOMP_CLI, else PATH
try:
    report = client.recompile('game.exe', '/tmp/project')
except RecompileError as exc:
    raise SystemExit(f'GameReComp failed: {exc}')

print(report.get('score'), len(report.get('stages', [])))
```

Set the CLI location through the environment (no hard-coded paths):

```sh
export GAMERECOMP_CLI=/path/to/game-recomp-cli
```

## API

```python
Client(cli: str | os.PathLike | None = None)
# cli defaults to $GAMERECOMP_CLI, then "game-recomp-cli" on PATH.
# Raises RecompileError if the CLI is not an existing file.

Client.inspect(pe) -> dict
# Runs <cli> inspect <pe> --json; returns the parsed PE metadata.

Client.recompile(pe, project) -> dict
# Runs the pipeline and returns the parsed reports/report.json dict.

Client.recompile_to_path(pe, project) -> Path
# Same pipeline, returns the pathlib.Path of reports/report.json instead.
```

All failures (missing CLI/PE, non-zero exit, missing or invalid report) are
surfaced as `RecompileError`, a `RuntimeError` subclass that includes the CLI
output tail. The package is PEP 561 typed (`py.typed`).

## Development

```sh
python3 -m pip install -e '.[dev]'
python3 -m pytest python/tests -q
python3 -m build            # writes dist/*.whl and dist/*.tar.gz
```

The real-CLI integration test is **opt-in** and skipped by default; see
[`docs/PYTHON_PACKAGING.md`](../docs/PYTHON_PACKAGING.md) for the environment
variables and the version-consistency and release policy.

## License

AGPL-3.0-only. See [`LICENSE`](../LICENSE).
