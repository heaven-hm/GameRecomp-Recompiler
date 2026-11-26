# Security policy

## Scope

This repository is a thin client SDK around the GameReComp CLI. It does not
ship a recompiler, does not handle game keys or license data, and does not
authenticate to any service. The security-relevant surfaces are therefore
**configuration** and **the text a failing CLI returns**. See
[ARCHITECTURE.md](ARCHITECTURE.md) for the trust boundary,
[API_CONSISTENCY.md](API_CONSISTENCY.md#cli-output-capture) for exactly what
each binding captures, and [CONTRIBUTING.md](CONTRIBUTING.md) for the rules
that keep this repository thin.

## Keys and credentials: environment only

- The only runtime configuration this SDK consumes is the CLI location. In
  Python an explicit argument (the example's `--cli`) takes precedence, then
  the `GAMERECOMP_CLI` environment variable, then the bare name
  `game-recomp-cli`; the C, C++ and Rust bindings take the CLI path as an
  explicit argument. There is no config file and no credential store. Note
  that `Client` validates the value with `Path(cli).is_file()` before spawning,
  so a bare name is only accepted if such a file exists in the current working
  directory — in practice, supply an absolute path or set `GAMERECOMP_CLI` to
  one.
- If your deployment needs a credential to reach a private CLI or artifact
  store, inject it as an environment variable or CI secret at runtime. Do not
  add a file containing real values to this repository.
- Never write a token, password, API key, private key, or a machine-specific
  absolute path into source, tests, examples, fixtures, or documentation. Use
  placeholders such as `/path/to/game-recomp-cli`.
- Keep secrets out of the command line the SDK constructs. All bindings build
  an argument vector — `run <pe> --project-dir <project> --no-table
  --no-progress` — with no shell interpolation, but the paths you supply are
  still untrusted input and child processes inherit the parent environment.
- Prefer short-lived, least-privilege credentials. Rotate anything that may
  have been exposed.

## Never commit secrets

Forbidden in any commit, including tests and "temporary" fixtures:

- API keys, access tokens, passwords, private keys, and `.env` files with real
  values (a committed `.env.example` with placeholders is fine);
- absolute paths that identify a particular developer account, Wine prefix, or
  licensed game install;
- captured CLI logs, reports, or crash dumps containing customer or site data.

`.gitignore` excludes local build and test artifacts (`rust/target/`,
`__pycache__/`, `.pytest_cache/`, `basic`, `basic_cpp`, etc.), but it is not a
secret scanner. Review `git diff --cached` before committing. If a secret is
committed:

1. Rotate/revoke it first — history rewrites do not undo a leak.
2. Remove it from the branch; if it reached a shared branch, purge it from
   history (`git filter-repo` or equivalent) and ask maintainers to re-clone.
3. Note the rotation in the PR description without pasting the secret value.

## Error sanitization

On failure the C/C++/Python bindings deliberately surface captured CLI output
so a failed pipeline can be diagnosed. **That text is untrusted**: a child
process can print absolute paths, file names, or secrets present in its
environment. The SDK truncates tails but performs no secret detection.

| Binding | What failure text actually contains |
|---|---|
| C | `gr_last_error()` — fixed 512-byte `error` buffer; on a CLI failure it stores up to **511 bytes of combined stdout/stderr**. The header's "never contains secrets" intent is not a guarantee: sanitize it. |
| C++ | `std::runtime_error::what()` — SDK validation message or the C buffer's captured tail. |
| Rust | `Error::Failed` holds only a spawn/IO error string or `exit <code>`; `Error::NoReport` holds the expected report path. **No captured CLI stdout/stderr is exposed by Rust.** |
| Python | `RecompileError` — includes the last **2000 characters** of combined output on non-zero exit, or on invalid JSON from `inspect()`. |

Before you log, file an issue, or send telemetry containing this text:

1. Redact anything that looks like a token, key, password, or credential.
2. Replace user names and absolute paths with a placeholder.
3. Keep only the short tail needed to diagnose the failure; do not paste whole
   logs or reports into a public issue.

Rust callers still need to sanitize paths embedded in `MissingCli`,
`MissingInput` and `NoReport` messages.

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability. Use a private
security advisory on this repository (or contact the maintainers directly) and
include: affected binding and version, reproduction steps, impact, and a
sanitized error sample. You should receive an acknowledgement before any
public disclosure; please allow maintainers time to prepare a fix.
