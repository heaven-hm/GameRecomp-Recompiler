# Real CLI integration checks (inspect-only)

This directory documents a **shared, real-CLI integration harness** for the
SDK bindings. It complements (never replaces) the fake-CLI unit tests in
`tests/test_c_sdk.sh`, `tests/test_cpp_sdk.sh`, the Python `pytest` suite and
the Rust `blaze test` suite.

## Scope — read this first

> **This harness runs only `game-recomp-cli inspect --json`.**
> It does **not** run `game-recomp-cli run`, it does **not** recompile a game,
> and it does **not** produce a project or a `report.json`. A PASS here means
> the real CLI's inspect JSON contract held — nothing more. See
> [REAL_RUN.md](REAL_RUN.md) for a genuine end-to-end recompilation example,
> which is a separate, opt-in activity.

This repository contains no second recompiler, loader, lifter or PE parser.
`tests/fixtures/minimal_pe.py` emits a handful of header bytes purely so the
*real* CLI's fail-closed PE32 check can be exercised without shipping a
copyrighted game binary. All verification is performed by the external CLI.

## Files

| File | Role |
|---|---|
| `tests/real_cli_integration.sh` | Shared POSIX `sh` harness. Discovers the CLI from `GAMERECOMP_CLI`, validates `--version`, runs `inspect --json`, checks the **exact** JSON key set and values, records SHA-256 hashes and full logs, and emits `results.json`. |
| `tests/fixtures/minimal_pe.py` | Dependency-free generator for a deterministic minimal PE32 x86 header image used as the default inspect input. |
| `docs/REAL_CLI_INTEGRATION.md` | This document. |

## Running it

The CLI location is **configuration, never a hard-coded path**. If the CLI is
not available the harness reports an explicit `SKIP` and exits 0, so it is safe
to include in local scripts and CI without provisioning the native binary.

```sh
# Generates a minimal PE32 fixture automatically and inspects it.
GAMERECOMP_CLI=/path/to/game-recomp-cli \
  sh tests/real_cli_integration.sh

# Inspect a specific, supplied PE32 (e.g. a game you own).
GAMERECOMP_CLI=/path/to/game-recomp-cli \
  sh tests/real_cli_integration.sh /path/to/game.exe /tmp/gr-integration

# Same, via environment variables (useful in CI step definitions).
GAMERECOMP_CLI=/path/to/game-recomp-cli \
GR_PE_FILE=/path/to/game.exe \
GR_CLI_INTEGRATION_OUT=/tmp/gr-integration \
  sh tests/real_cli_integration.sh
```

### Configuration

| Variable | Meaning | Default |
|---|---|---|
| `GAMERECOMP_CLI` | Path to the real `game-recomp-cli`. **Required**; unset/missing/non-executable => `SKIP`. | none |
| `GR_PE_FILE` | PE32 to pass to `inspect --json`. | generate via `minimal_pe.py` |
| `GR_CLI_INTEGRATION_OUT` | Artifact/output directory. | a fresh temp directory under `${TMPDIR:-/tmp}` |
| `GR_EXPECT_IMAGE_BASE` | Expected `image_base` when using the generated fixture. | `0x00400000` |

Positional arguments are `[PE32_FILE] [OUT_DIR]` and take precedence over the
matching environment variables.

### Exit status

| Code | Meaning |
|---|---|
| 0 | `PASS` (contract held) **or** `SKIP` (CLI absent) — always printed explicitly. |
| 1 | `FAIL` — CLI was present but a contract check failed. |
| 2 | Harness usage/setup error. |

Because `SKIP` and `PASS` both exit 0, callers that need to distinguish them
should read the `status` field in `results.json` or match the printed
`SKIP:`/`PASS:` prefix.

## What is verified

1. **Discovery** — `GAMERECOMP_CLI` must name an existing, executable file.
   Otherwise: `SKIP`.
2. **Version** — `<cli> --version` must exit 0 and print a
   `game-recomp-cli <major.minor...>` string (ANSI colour codes stripped).
3. **Inspect** — `<cli> inspect --json <pe32>` must exit 0 and emit one JSON
   object on stdout.
4. **Exact JSON keys** — the object's key set must equal exactly these ten keys
   (source: GameReComp `src/cli.rs::run_inspect_cmd`), with no extras or
   omissions:

   ```
   path  ok  error  magic  machine  pe32_x86  image_base  size  input_os  input_arch
   ```

5. **Values** — for every input: `ok == true`, `pe32_x86 == true`,
   `magic == "PE32"`, `machine == "i386"`, `input_os == "windows"`,
   `input_arch == "x86"`, `error` empty, `size` a positive integer, and `path`
   resolving to the inspected file. For the generated fixture `image_base`
   must equal `GR_EXPECT_IMAGE_BASE` (`0x00400000`).
6. **Hashes/logs** — SHA-256 of the CLI binary, the PE32 input and the raw
   captured inspect JSON are computed and written to the log and to
   `results.json`.

The harness deliberately does **not** invoke `run`, `batch`, `readability` or
any other subcommand. Recompilation claims belong only to REAL_RUN.md.

## Artifacts

With `OUT_DIR` set to (say) `/tmp/gr-integration`:

```
/tmp/gr-integration/
  version.txt             raw stdout of `--version`
  version.stderr.log      raw stderr of `--version`
  inspect.json            raw stdout of `inspect --json` (hashed)
  inspect.stderr.log      raw stderr of `inspect --json`
  run.log                 full human-readable transcript
  results.json            machine-readable summary incl. status + hashes
  minimal-pe32.exe        present only when the fixture was auto-generated
```

`results.json` example (abridged):

```json
{
  "status": "PASS",
  "mode": "inspect-only",
  "cli": "/path/to/game-recomp-cli",
  "pe": "/tmp/gr-integration/minimal-pe32.exe",
  "cli_sha256": "...",
  "pe_sha256": "...",
  "inspect_json_sha256": "...",
  "cli_version": "game-recomp-cli 0.0.1",
  "harness": "tests/real_cli_integration.sh"
}
```

## Using the harness from other bindings

The script is a **shared** harness: every binding's proof script can invoke the
same command (`sh tests/real_cli_integration.sh [PE32] [OUT_DIR]`) instead of
duplicating CLI discovery, version validation, key lists, hashing and logging
logic. Each binding keeps its own report/exec plumbing in its existing
fake-CLI test; this harness only adds the real-CLI inspect contract check on
top. Its helper functions (`sha256_file`, `write_summary`, the SKIP/FAIL/PASS
helpers and the embedded JSON validator) live in one place so the contract
cannot drift between bindings.

To wire into CI, add a step that sets `GAMERECOMP_CLI` when the native binary is
available (e.g. a cached build artifact) and calls the script; when it is not
available the step self-skips with exit 0. Keep the existing four-binding
matrix unchanged — this check is additive and opt-in.

## Limitations / non-claims

- A PASS proves only the **inspect** JSON contract. It does not prove that any
  PE32 recompiles, links, renders or runs.
- No game asset is bundled; the generated PE32 is a header-only stub.
- Hash values are environment-specific (CLI build, chosen PE). They are
  recorded for provenance/audit, not asserted against golden constants.
- The exact-key assertion mirrors the current CLI. If GameReComp intentionally
  adds an `inspect --json` field, update `EXPECTED_KEYS` here together with the
  validator in the same commit.
- When `GAMERECOMP_CLI` is absent the result is `SKIP`, not a silent pass of the
  real-CLI checks; the four fake-CLI binding tests remain the always-on gate.
