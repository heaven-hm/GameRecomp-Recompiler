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

Client.recompile_to_path(pe, project) -> Path
# Runs the same pipeline as recompile(), but returns the pathlib.Path of the
# generated <project>/reports/report.json instead of parsing its contents.
# Raises the same RecompileError cases as recompile().

Client.inspect(pe) -> dict
# Runs: <cli> inspect <pe> --json
# Returns the parsed JSON object printed by the CLI (metadata about the PE).
# No project directory is needed and no files are written.
# Raises RecompileError on a missing PE, non-zero exit, or invalid JSON.
```

`RecompileError` is a `RuntimeError` subclass with the CLI output tail included
on failure. CLI and report failures are surfaced through it by all three
methods.

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

## Inspecting a PE and locating the report

`inspect()` returns the CLI's PE metadata without running a recompile or
needing a project directory:

```python
from gamerecomp_sdk import Client

client = Client()
info = client.inspect('game.exe')
print(info.get('machine'), len(info.get('sections', [])))
```

`recompile_to_path()` is useful when you want the generated report file itself
(for archiving, diffing, or passing to another tool) rather than the parsed
dictionary returned by `recompile()`:

```python
import json

report_path = client.recompile_to_path('game.exe', '/tmp/project')
print(report_path)                       # /tmp/project/reports/report.json
report = json.loads(report_path.read_text())
# Equivalent to: report = client.recompile('game.exe', '/tmp/project')
```

The example script exposes both through flags:

```sh
PYTHONPATH=python python3 python/examples/basic.py --pe game.exe \
    --project /tmp/project --inspect
PYTHONPATH=python python3 python/examples/basic.py --pe game.exe \
    --project /tmp/project --report-path
```

Verify the SDK plumbing without a long build:

```sh
PYTHONPATH=python python3 -m pytest python/tests -q
# 13 passed
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
| Inspect output not JSON | `RecompileError("CLI produced invalid JSON: <tail>")` |

## Real proven run

See `docs/REAL_RUN.md` for a real `dxIka.exe` recompile through this binding:
score 93/100, all stages ok, `verification: partial`.
