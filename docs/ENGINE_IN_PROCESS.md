# In-process engine

The GameReComp recompiler source is vendored in this repository under `tools/`
and `runtime/`. The Python binding exposes it directly through `Engine.build`,
which imports `tools.autorecomp.pipeline` and runs stages in the caller's
process. No CLI binary, subprocess or shell is involved:

```python
from gamerecomp_sdk import BuildOptions, Engine
report = Engine().build("game.exe", "out", BuildOptions(
    input_os="windows", input_arch="x86",
    target_os="host", target_arch="host", out_mode="full"))
```

Verified with a real PE32 `arith-calc.exe`: score 85, 6 stages, real
`reports/report.json`. The C/C++/Rust/JS demo apps were cleaned up and use the
same explicit build options; their engine bridge is still the process runner
and will be replaced with the same in-process engine behind the shared ABI.

Generated recompiler projects (`projects/`, `*-bin/`, `build/`, `reports/`)
are gitignored and must never be committed to the SDK.
