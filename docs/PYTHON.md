# Python SDK guide

## Files

- Package: `python/gamerecomp_sdk/__init__.py`
- Example: `python/examples/basic.py`
- Tests: `python/tests/test_sdk.py`

Only the standard library is used (`subprocess`, `json`, `pathlib`).

## Installation

```sh
# Directly from this repository:
PYTHONPATH=python python3 python/examples/basic.py --help

# Or editable install:
python3 -m pip install -e python
```

## API reference

```python
from gamerecomp_sdk import Client, RecompileError

Client(cli: str | os.PathLike | None = None)
# cli defaults to $GAMERECOMP_CLI, then "game-recomp-cli" on PATH.
# Raises RecompileError if the CLI is not an existing file.

Client.recompile(pe, project) -> dict
# Runs: <cli> run <pe> --project-dir <project> --no-table --no-progress
# Raises RecompileError on non-zero exit or missing report.json.
# Returns the parsed reports/report.json dictionary.
```

`RecompileError` is a `RuntimeError` subclass with the CLI output tail included
on failure.

## Full example

```python
import argparse
from gamerecomp_sdk import Client, RecompileError

ap = argparse.ArgumentParser()
ap.add_argument('--pe', required=True)
ap.add_argument('--project', required=True)
ap.add_argument('--cli')
a = ap.parse_args()

try:
    report = Client(a.cli).recompile(a.pe, a.project)
except RecompileError as exc:
    raise SystemExit(f'GameReComp failed: {exc}')

print(f"score={report.get('score')} stages={len(report.get('stages', []))}")
```

Run:

```sh
GAMERECOMP_CLI=/path/to/game-recomp-cli \
python3 python/examples/basic.py --pe game.exe --project /tmp/project
```

Verify the SDK plumbing without a long build:

```sh
PYTHONPATH=python python3 -m pytest python/tests -q
# 4 passed
```

## Reading common report fields

```python
report = client.recompile(pe, project)
print(report.get('score'))
print([s['name'] for s in report.get('stages', [])])
outcome = report.get('outcome') or {}
print(outcome.get('status'), outcome.get('reason'))
```

The report is whatever GameReComp wrote; the SDK does not reinterpret or
inflate it. `verification: partial` is not a gameplay pass.

## Error cases

| Condition | Exception |
|---|---|
| CLI missing | `RecompileError("CLI binary does not exist: ...")` |
| PE missing | `RecompileError("PE32 input does not exist: ...")` |
| CLI non-zero | `RecompileError("CLI exited N: <tail>")` |
| No report | `RecompileError("pipeline produced no report.json")` |

## Real proven run

See `docs/REAL_RUN.md` for a real `dxIka.exe` recompile through this binding:
score 93/100, all stages ok, `verification: partial`.
