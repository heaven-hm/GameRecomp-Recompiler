# Real end-to-end SDK run (Python binding)

This is a **historical record** of a run performed with the Python binding
against a real PE32 game through the GameReComp CLI. It was not produced by the
fake-CLI/mocked tests. The CLI and toolchain versions were not pinned in the
original record, so the numbers below are evidence that the plumbing worked at
that time, not a reproducible benchmark or a gameplay claim.

## Command

```sh
GAMERECOMP_CLI=/path/to/game-recomp-cli \
  PYTHONPATH=python python3 python/examples/basic.py \
  --pe /path/to/dxIka.exe \
  --project /tmp/sdk-real/ikachan \
  --cli /path/to/game-recomp-cli
```

The explicit `--cli` argument and the `GAMERECOMP_CLI` environment variable are
redundant here: `python/examples/basic.py` passes `--cli` to `Client(...)`,
which takes precedence over the environment variable. Either form alone is
sufficient.

## Input and result

Input: `dxIka.exe`, recorded as a real PE32 x86 game. The SDK invoked the real
GameReComp CLI; no fake CLI was used for this run.

Observed CLI output (tail, as recorded):

```
stages: inspect(ok), recover(ok), translate(ok), project(ok), compile(ok),
        run(ok), verify(ok), report(ok)
score: 93/100
verification: partial
```

The SDK returned the parsed report (`score=93`). The recorded output lists
**eight** stages (`inspect`, `recover`, `translate`, `project`, `compile`,
`run`, `verify`, `report`); the SDK itself does not enumerate or validate
stages — it only reads the report file the CLI wrote. `verification: partial`
was interpreted at the time as "the process launched and exited 1 with no
oracle"; this is **not** a claim that the game plays correctly.

The other bindings were exercised with fake CLIs (see [EXAMPLES.md](EXAMPLES.md)
and [API_CONSISTENCY.md](API_CONSISTENCY.md#verification)), which proves their
plumbing without repeating a long native build.
