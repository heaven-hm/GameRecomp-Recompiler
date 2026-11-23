# Real end-to-end SDK run (Python binding)

Command actually executed:

```sh
GAMERECOMP_CLI=/path/to/game-recomp-cli \
PYTHONPATH=python python3 python/examples/basic.py \
  --pe /path/to/dxIka.exe \
  --project /tmp/sdk-real/ikachan \
  --cli /path/to/game-recomp-cli
```

Input: `dxIka.exe`, a real PE32 x86 game. The SDK invoked the real GameReComp
CLI; no fake CLI was used for this run.

Observed CLI output (tail):

```
stages: inspect(ok), recover(ok), translate(ok), project(ok), compile(ok),
        run(ok), verify(ok), report(ok)
score: 93/100
verification: partial
```

The SDK returned the parsed report (`score=93`, 7 stages recorded in the JSON).
`verification: partial` means the process launched and exited 1 with no oracle;
this is **not** a gameplay claim.

Other bindings were verified with a fake CLI (see `docs/EXAMPLES.md`), which
proves their plumbing without repeating the long native build.
