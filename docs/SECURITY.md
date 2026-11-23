# Security policy

## Scope

This repository is a thin client SDK around the GameReComp CLI. It does not
ship a recompiler, does not handle game keys or license data, and does not
authenticate to any service. The security-relevant surfaces are therefore
**configuration** and **the text a failing CLI returns**. See
[ARCHITECTURE.md](ARCHITECTURE.md) for the trust boundary and
[CONTRIBUTING.md](CONTRIBUTING.md) for the rules that keep this repository thin.

## Keys and credentials: environment only

- The only runtime configuration this SDK consumes is the CLI location. In
  Python it is read from the `GAMERECOMP_CLI` environment variable, falling back
  to `game-recomp-cli` on `PATH`; the C, C++ and Rust bindings take the CLI path
  as an explicit argument. There is no config file and no credential store.
- If your deployment needs a credential to reach a private CLI or artifact
  store, inject it as an environment variable or CI secret at runtime. Do not
  add a file containing real values to this repository.
- Never write a token, password, API key, private key, or a machine-specific
  absolute path into source, tests, examples, fixtures, or documentation. Use
  placeholders such as `/path/to/game-recomp-cli`.
- Keep secrets out of the command line the SDK constructs. It builds
  `game-recomp-cli run <pe> --project-dir <project> --no-table --no-progress`
  from the paths you supply; treat those paths as untrusted input.
- Prefer short-lived, least-privilege credentials. Rotate anything that may
  have been exposed.

## Never commit secrets

Forbidden in any commit, including tests and "temporary" fixtures:

- API keys, access tokens, passwords, private keys, and `.env` files with real
  values (a committed `.env.example` with placeholders is fine);
- absolute paths that identify a particular developer account, Wine prefix, or
  licensed game install;
- captured CLI logs, reports, or crash dumps containing customer or site data.

`.gitignore` excludes local build and test artifacts, but it is not a secret
scanner. Review `git diff --cached` before committing. If a secret is committed:

1. Rotate/revoke it first — history rewrites do not undo a leak.
2. Remove it from the branch; if it reached a shared branch, purge it from
   history (`git filter-repo` or equivalent) and ask maintainers to re-clone.
3. Note the rotation in the PR description without pasting the secret value.

## Error sanitization

On failure the bindings deliberately surface the captured CLI output tail so a
failed pipeline can be diagnosed:

| Binding | Where the captured tail is exposed |
|---|---|
| C | `gr_last_error()` — fixed-size buffer; does not store secrets by design |
| C++ | `std::runtime_error::what()` — SDK message plus captured tail |
| Rust | `Error::Failed(String)` / `Error::NoReport(String)` |
| Python | `RecompileError` — includes the last 2000 characters of combined output |

That text is **untrusted**. A child process can print absolute paths, file
names, or secrets that were present in its environment. The SDK truncates tails
but does not perform secret detection, so sanitize at the call site before you
log, file an issue, or send telemetry:

1. Redact anything that looks like a token, key, password, or credential.
2. Replace user names and absolute paths with a placeholder.
3. Keep only the short tail needed to diagnose the failure; do not paste whole
   logs or reports into a public issue.

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability. Use a private
security advisory on this repository (or contact the maintainers directly) and
include: affected binding and version, reproduction steps, impact, and a
sanitized error sample. You should receive an acknowledgement before any
public disclosure; please allow maintainers time to prepare a fix.
