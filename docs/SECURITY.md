# Security policy

- Pass the CLI path as an argument to `Client::new`; do not hard-code a user
  path or credential, and do not put secrets in the repository.
- The SDK uses `std::process::Command` with separate arguments. No shell is
  involved, so PE paths and project paths are never interpolated into a command
  string.
- On failure, captured CLI output is included in typed error context. Sanitize
  it before sharing logs; it may contain local paths.
- Do not commit game binaries, proprietary assets or private keys.
- Error variants expose structured path/exit-status context instead of parsed
  prose. See [Rust API](RUST_API.md).
