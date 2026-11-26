# Multi-language demos with proof

`sh demos/run_all.sh` runs one end-to-end demo per SDK binding against the
deterministic `demos/fake-cli.sh`. It records stdout, stderr, exit codes and
SHA-256 digests in `proof/demos/results.json`.

This is **SDK plumbing proof only**: the fake CLI writes a synthetic
`report.json`, so no recompiler or gameplay result is claimed.

| Binding | Command |
|---|---|
| C | `cc -std=c11 -Iinclude src/gamerecomp_sdk.c c/examples/basic.c -o basic && ./basic <cli> <pe> <project>` |
| C++ | `c++ -std=c++17 -Iinclude -Icpp/include src/gamerecomp_sdk.c cpp/examples/basic.cpp -o basic_cpp && ./basic_cpp <cli> <pe> <project>` |
| Python | `PYTHONPATH=python python3 python/examples/basic.py --cli <cli> --pe <pe> --project <project>` |
| JavaScript/TypeScript | `node js/examples/basic.js <cli> <pe> <project>` |
| Rust | `cd rust && blaze run --example basic -- <cli> <pe> <project>` |

Observed result (all five exit 0, stdout hashes recomputed from captured
bytes): `proof/demos/results.json`. Browser screenshot proof:
`proof/screenshots/sdk-demos.jpeg`.
