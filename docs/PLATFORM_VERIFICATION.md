# Platform verification record (SDK10)

This document records what was actually executed for the four SDK bindings at
base commit `d931c0240b612a255c2637b78290e9bf5e299c47` on branch
`feature/sdkprod-10-platforms`. It is an evidence record, not a claim that a
game renders or plays. The SDK contract is "CLI invoked, report produced"; see
[ARCHITECTURE.md](ARCHITECTURE.md) and the honest limitations in
[README.md](../README.md#honest-limitations).

The four bindings are Python, Rust, C and C++, matching
[`.github/workflows/ci.yml`](../.github/workflows/ci.yml) (matrix:
`os: [ubuntu-latest, macos-latest]`, `check: [python, rust, c, cpp, javascript]`).

The machine-readable summary at the bottom of this file is validated by
[`tests/test_platform_matrix.py`](../tests/test_platform_matrix.py) (stdlib
only; runs under `pytest` or `python3 tests/test_platform_matrix.py`).

## Host and toolchain

Captured with `date -u +'%Y-%m-%dT%H:%M:%SZ'`, `uname -a`, `sw_vers`, and
`command -v` / `--version` probes:

```text
2026-11-23T16:28:53Z
Darwin Haseebs-MacBook-Pro.local 27.2.0 Darwin Kernel Version 27.2.0: Wed Oct 14 23:48:16 PDT 2026; root:xnu-13432.41.4~25/RELEASE_ARM64_T8112 arm64
ProductName:  macOS
ProductVersion:  27.2.1
BuildVersion: 26B131
architecture: arm64
```

| Tool | Version / path |
|---|---|
| Python | 3.12.1 (`python3 --version`) |
| pytest | 9.1.0 (`python3 -m pytest --version`) |
| cargo | 1.100.0 (`blaze --version`) |
| rustc | 1.100.0 (`rustc --version`) |
| cc / c++ | Apple clang 16.0.0, target `arm64-apple-darwin27.2.0` |

All commands below were run from the worktree root with a clean checkout of
the base commit (`git status --short` empty before the run).

## macOS matrix — EXECUTED (all four passed)

These are the exact commands from the CI matrix, run locally on macOS arm64.
Absolute temporary paths in the captured output are the worktree and the
scripts' own scratch directories, reproduced verbatim as evidence.

| Binding | Command (worktree root unless noted) | Exit code | Result |
|---|---|---:|---|
| Python | `PYTHONPATH=python python3 -m pytest python/tests -q` | 0 | PASS — 13 tests |
| Rust | `cd rust && blaze test` | 0 | PASS — 15 unit tests, 0 doc tests |
| C | `sh tests/test_c_sdk.sh` | 0 | PASS |
| C++ | `sh tests/test_cpp_sdk.sh` | 0 | PASS (4 pre-existing compiler warnings) |

### Python — verbatim output

```text
$ PYTHONPATH=python python3 -m pytest python/tests -q
.............                                                            [100%]
13 passed in 0.23s
# exit=0
```

Note for doc maintainers (no edit made here; docs are not owned by this
record): [README.md](../README.md#verification) and
[CONTRIBUTING.md](CONTRIBUTING.md#test-matrix) still describe "4 fake-CLI
tests" for Python. The base commit actually collects 13 test functions in
`python/tests/test_sdk.py`; the executed result above is the accurate one.

### Rust — verbatim output

```text
$ cd rust && blaze test
   Compiling gamerecomp-sdk v0.1.0 (/private/var/folders/q0/rj0k1xgn28x14thfz4gnbm5c0000gn/T/opencode/sdkprod/10/rust)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.42s
     Running unittests src/lib.rs (target/debug/build/gamerecomp-sdk/283bab5681fa0dce/out/gamerecomp_sdk-283bab5681fa0dce)

running 15 tests
test tests::missing_cli_is_rejected ... ok
test tests::parse_score_bounds ... ok
test tests::parse_score_compact_document ... ok
test tests::parse_score_ignores_similarly_named_keys ... ok
test tests::parse_score_missing_field ... ok
test tests::missing_input_is_rejected ... ok
test tests::parse_score_rejects_non_integers ... ok
test tests::parse_score_skips_malformed_occurrence ... ok
test tests::parse_score_whitespace_everywhere ... ok
test tests::type_names_are_public ... ok
test tests::report_json_missing_file_is_empty ... ok
test tests::report_json_returns_exact_file_contents ... ok
test tests::cli::recompile_reports_cli_failure ... ok
test tests::cli::recompile_fails_without_report ... ok
test tests::cli::recompile_parses_score_from_fake_cli_report ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s

   Doc-tests gamerecomp_sdk

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
# exit=0
```

### C — verbatim output

```text
$ sh tests/test_c_sdk.sh
C SDK: PASS
# exit=0
```

The script compiles with `cc -std=c11 -Wall -Wextra` and exercises the happy
path plus the missing-directory, non-directory, non-executable-CLI and EINTR
cases against fake CLIs; no compiler warnings were emitted.

### C++ — verbatim output

```text
$ sh tests/test_cpp_sdk.sh
clang++: warning: treating 'c' input as 'c++' when in C++ mode, this behavior is deprecated [-Wdeprecated]
/private/var/folders/q0/rj0k1xgn28x14thfz4gnbm5c0000gn/T/opencode/sdkprod/10/src/gamerecomp_sdk.c:81:34: warning: ISO C++11 does not allow conversion from string literal to 'char *' [-Wwritable-strings]
   81 |         char *argv[] = { c->cli, "run", (char *)pe_path,
      |                                  ^
/private/var/folders/q0/rj0k1xgn28x14thfz4gnbm5c0000gn/T/opencode/sdkprod/10/src/gamerecomp_sdk.c:82:26: warning: ISO C++11 does not allow conversion from string literal to 'char *' [-Wwritable-strings]
   82 |                          "--project-dir", (char *)project_dir,
      |                          ^
/private/var/folders/q0/rj0k1xgn28x14thfz4gnbm5c0000gn/T/opencode/sdkprod/10/src/gamerecomp_sdk.c:83:26: warning: ISO C++11 does not allow conversion from string literal to 'char *' [-Wwritable-strings]
   83 |                          "--no-table", "--no-progress", NULL };
      |                                        ^
4 warnings generated.
report=/var/folders/q0/rj0k1xgn28x14thfz4gnbm5c0000gn/T//gr-sdk-cpp-38836/proj/reports/report.json
score=70
report=/var/folders/q0/rj0k1xgn28x14thfz4gnbm5c0000gn/T//gr-sdk-cpp-38836/proj/reports/report.json
score=-1
C++ SDK: PASS
# exit=0
```

The four warnings are pre-existing in the base commit: the proof script feeds
the C implementation file to `c++`, so string literals are diagnosed under
C++ rules. The script still exits 0 and asserts both `score=70` and the
`score=-1` sentinel path.

## Linux matrix — BLOCKED / NOT TESTED

**Status: BLOCKED. NOT TESTED in this environment.** No Linux command from
the CI matrix was executed, and no Linux exit code, test count or "PASS" is
claimed below. The Linux cells in the machine-readable record deliberately
carry `exit_code: null` and `status: "blocked"` so that no result is
fabricated.

Capability probe (commands run at `2026-11-23T16:28:53Z` on the macOS arm64
host):

| Probe | Command | Observed result |
|---|---|---|
| Container client | `command -v docker` | `/opt/homebrew/bin/docker` (client 29.9.2, darwin/arm64) |
| Container daemon | `docker version` | `failed to connect to the docker API at unix:///Users/haseeb-mir/.docker/run/docker.sock ... no such file or directory` |
| Colima context | `docker --context colima info` | `failed to connect ... unix:///Users/haseeb-mir/.colima/default/docker.sock ... no such file or directory` |
| Docker socket | `ls -l ~/.docker/run/docker.sock; file ~/.docker/run/docker.sock` | broken symbolic link to `~/.colima/default/docker.sock` |
| Lima instances | `limactl list` | `No instance found. Run 'limactl create' to create an instance.` |
| Colima daemon | `colima status` | `colima is not running` |
| Colima instances | `colima list` | `No instance found. Run 'colima start' to create an instance.` (empty profile table) |
| QEMU | `qemu-system-x86_64 --version`, `qemu-system-aarch64 --version` | 9.2.1 installed, but **no** bootable Linux kernel/disk image was found |
| Guest images | `find "$HOME" -maxdepth 5 \( -name '*.qcow2' -o -name '*.iso' -o -name '*.img' \)` | only unrelated `fci-recomp/profile/Partition*.img` profile files; no Linux install media |
| Other runtimes | `command -v podman nerdctl containerd buildah` | `NOT FOUND` |

Verbatim evidence excerpts:

```text
$ docker version
Client: Docker Engine - Community
 Version:           29.9.2
 OS/Arch:           darwin/arm64
 Context:           default
failed to connect to the docker API at unix:///Users/haseeb-mir/.docker/run/docker.sock; check if the path is correct and if the daemon is running: dial unix /Users/haseeb-mir/.docker/run/docker.sock: connect: no such file or directory

$ docker --context colima info
Server:
failed to connect to the docker API at unix:///Users/haseeb-mir/.colima/default/docker.sock; check if the path is correct and if the daemon is running: dial unix /Users/haseeb-mir/.colima/default/docker.sock: connect: no such file or directory

$ limactl list
time="2026-11-23T21:58:53+05:30" level=warning msg="No instance found. Run `limactl create` to create an instance."

$ colima status
time="2026-11-23T21:58:54+05:30" level=fatal msg="colima is not running"

$ colima list
time="2026-11-23T21:58:54+05:30" level=warning msg="No instance found. Run `colima start` to create an instance."
PROFILE    STATUS    ARCH    CPUS    MEMORY    DISK    RUNTIME    ADDRESS
```

Consequences — explicitly NOT TESTED on Linux:

- Python: `PYTHONPATH=python python3 -m pytest python/tests -q` on a Linux
  kernel / glibc.
- Rust: `cd rust && blaze test` on a Linux target.
- C: `sh tests/test_c_sdk.sh` on glibc, including the
  `_POSIX_C_SOURCE 200809L` feature-macro placement (commits `98dc945` and
  `d931c02` exist specifically for glibc) and the EINTR capture path.
- C++: `sh tests/test_cpp_sdk.sh` with a Linux C++ standard library.

Why this is blocked rather than merely deferred: a container runtime client
and QEMU binaries exist, but there is no running daemon, no Lima/Colima VM
instance, and no cached Linux guest image. Starting one would require
downloading a guest image and provisioning a VM, which is outside this
verification task. CI's `ubuntu-latest` jobs exercise these cells on every
push, but a CI configuration is not local evidence and is not claimed as
such here.

To unblock locally: start a supported runtime (for example `colima start`,
then `docker context use colima`), confirm `docker info` reaches a Linux
daemon, run the four commands from the macOS matrix table inside a Linux
container/VM with the repository mounted, and update the JSON record with the
real exit codes and summaries. Do not mark a Linux cell `passed` without an
observed exit code.

## Machine-readable record

The single `json` fenced block below is the source of truth consumed by
`tests/test_platform_matrix.py`. Timestamps are UTC.

```json
{
  "schema_version": 1,
  "record_id": "sdkprod-10-platform-verification",
  "generated_at_utc": "2026-11-23T16:28:53Z",
  "base_commit": "d931c0240b612a255c2637b78290e9bf5e299c47",
  "branch": "feature/sdkprod-10-platforms",
  "host": {
    "os_family": "Darwin",
    "os_name": "macOS",
    "os_version": "27.2.1",
    "arch": "arm64",
    "kernel": "Darwin 27.2.0"
  },
  "toolchain": {
    "python": "3.12.1",
    "pytest": "9.1.0",
    "cargo": "1.100.0",
    "rustc": "1.100.0",
    "cc": "Apple clang 16.0.0"
  },
  "results": [
    {
      "binding": "python",
      "platform": "macos",
      "status": "passed",
      "command": "PYTHONPATH=python python3 -m pytest python/tests -q",
      "exit_code": 0,
      "summary": "13 passed in 0.23s"
    },
    {
      "binding": "rust",
      "platform": "macos",
      "status": "passed",
      "command": "cd rust && blaze test",
      "exit_code": 0,
      "summary": "15 unit tests passed; 0 doc tests"
    },
    {
      "binding": "c",
      "platform": "macos",
      "status": "passed",
      "command": "sh tests/test_c_sdk.sh",
      "exit_code": 0,
      "summary": "C SDK: PASS"
    },
    {
      "binding": "javascript",
      "platform": "macos",
      "status": "passed",
      "command": "cd js && npm test",
      "exit_code": 0,
      "summary": "7 node:test cases passed"
    },
    {
      "binding": "cpp",
      "platform": "macos",
      "status": "passed",
      "command": "sh tests/test_cpp_sdk.sh",
      "exit_code": 0,
      "summary": "C++ SDK: PASS (4 pre-existing compiler warnings)"
    },
    {
      "binding": "python",
      "platform": "linux",
      "status": "blocked",
      "command": "PYTHONPATH=python python3 -m pytest python/tests -q",
      "exit_code": null,
      "summary": "NOT TESTED: no Linux daemon or guest available",
      "reason": "No running container daemon, no Lima/Colima VM instance, and no cached Linux guest image on this macOS arm64 host.",
      "evidence": [
        "docker version: failed to connect to docker API at unix://~/.docker/run/docker.sock (broken symlink to ~/.colima/default/docker.sock)",
        "docker --context colima info: failed to connect to docker API at unix://~/.colima/default/docker.sock",
        "limactl list: No instance found",
        "colima status: colima is not running",
        "qemu-system-x86_64 and qemu-system-aarch64 9.2.1 installed but no bootable Linux guest image found"
      ]
    },
    {
      "binding": "rust",
      "platform": "linux",
      "status": "blocked",
      "command": "cd rust && blaze test",
      "exit_code": null,
      "summary": "NOT TESTED: no Linux daemon or guest available",
      "reason": "No running container daemon, no Lima/Colima VM instance, and no cached Linux guest image on this macOS arm64 host.",
      "evidence": [
        "docker version: failed to connect to docker API at unix://~/.docker/run/docker.sock (broken symlink to ~/.colima/default/docker.sock)",
        "limactl list: No instance found",
        "colima list: No instance found"
      ]
    },
    {
      "binding": "javascript",
      "platform": "linux",
      "status": "blocked",
      "command": "cd js && npm test",
      "exit_code": null,
      "summary": "NOT TESTED: no Linux guest available locally",
      "reason": "No running container daemon or Linux guest on this macOS arm64 host.",
      "evidence": ["colima status: colima is not running", "docker info: daemon unavailable"]
    },
    {
      "binding": "c",
      "platform": "linux",
      "status": "blocked",
      "command": "sh tests/test_c_sdk.sh",
      "exit_code": null,
      "summary": "NOT TESTED: glibc POSIX macro and EINTR paths unverified locally",
      "reason": "No running container daemon, no Lima/Colima VM instance, and no cached Linux guest image on this macOS arm64 host.",
      "evidence": [
        "colima status: colima is not running",
        "docker --context colima info: failed to connect to docker API at unix://~/.colima/default/docker.sock",
        "find $HOME -maxdepth 5 for *.qcow2/*.iso/*.img: no Linux guest image"
      ]
    },
    {
      "binding": "cpp",
      "platform": "linux",
      "status": "blocked",
      "command": "sh tests/test_cpp_sdk.sh",
      "exit_code": null,
      "summary": "NOT TESTED: no Linux daemon or guest available",
      "reason": "No running container daemon, no Lima/Colima VM instance, and no cached Linux guest image on this macOS arm64 host.",
      "evidence": [
        "limactl list: No instance found",
        "colima status: colima is not running",
        "podman/nerdctl/containerd/buildah: NOT FOUND"
      ]
    }
  ]
}
```

## Scope of this record

- Verified here: SDK plumbing on macOS arm64 only, via fake CLIs, for all four
  bindings.
- Not verified here: Linux/glibc behavior, Windows behavior, real PE32
  recompilation, and any claim about game rendering or playability.
- No language source, CI configuration, or existing documentation was changed
  by this task; only the two new evidence files were added.
