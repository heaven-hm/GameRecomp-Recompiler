//! Rust client SDK for GameReComp.
//!
//! Thin wrapper around `game-recomp-cli run`; all recompilation is performed
//! by GameReComp itself. The crate has no third-party dependencies: subprocess
//! management, timeouts and error context are all implemented with `std`.
//!
//! # Example
//!
//! ```no_run
//! use gamerecomp_sdk::Client;
//!
//! # fn main() -> Result<(), gamerecomp_sdk::Error> {
//! let client = Client::new("/path/to/game-recomp-cli")?;
//! let report = client.recompile("game.exe", "out")?;
//! println!("score = {:?}", report.score);
//! # Ok(())
//! # }
//! ```

use std::fmt;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// How often a timed CLI wait checks whether the child has exited.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Result of a successful [`Client::recompile`] call.
///
/// The public shape stays the original two-field struct so existing callers
/// keep compiling, including those that build a `Report { path, score }`
/// literal themselves.
#[derive(Debug, Clone)]
pub struct Report {
    pub path: PathBuf,
    pub score: Option<i64>,
}

impl Report {
    /// Return the raw text of the `report.json` file for this report.
    ///
    /// This is the unmodified document produced by the CLI, useful when a
    /// caller needs data the SDK does not model (for example `stages`). The
    /// file is read on demand from [`Report::path`]; if it can no longer be
    /// read, an empty string is returned.
    #[must_use]
    pub fn report_json(&self) -> String {
        std::fs::read_to_string(&self.path).unwrap_or_default()
    }
}

/// Errors returned by this SDK.
///
/// Every variant carries structured context rather than a single opaque
/// string, so callers can inspect paths, exit statuses and captured stderr
/// without parsing prose. Variants that wrap an underlying I/O failure expose
/// it through [`std::error::Error::source`].
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The configured CLI path does not exist or is not a regular file.
    MissingCli { path: PathBuf },
    /// The requested PE32 input path does not exist or is not a regular file.
    MissingInput { path: PathBuf },
    /// The CLI could not be spawned/waited on, or exited unsuccessfully.
    Failed {
        /// Human-readable summary of what went wrong.
        message: String,
        /// Process exit status, when a process was actually launched.
        status: Option<ExitStatus>,
        /// Captured stderr (may be empty).
        stderr: String,
        /// Underlying I/O error for spawn/wait failures, if any.
        source: Option<io::Error>,
    },
    /// The pipeline exited successfully but no readable report was produced.
    NoReport {
        path: PathBuf,
        /// The I/O error encountered while reading the expected report.
        source: io::Error,
    },
    /// A `--version` query failed or returned an unusable response.
    Version {
        message: String,
        /// Underlying I/O error for spawn/wait failures, if any.
        source: Option<io::Error>,
    },
    /// A CLI invocation exceeded the configured timeout.
    TimedOut { duration: Duration },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingCli { path } => {
                write!(
                    f,
                    "CLI binary does not exist or is not a file: {}",
                    path.display()
                )
            }
            Error::MissingInput { path } => {
                write!(
                    f,
                    "PE32 input does not exist or is not a file: {}",
                    path.display()
                )
            }
            Error::Failed {
                message,
                status,
                stderr,
                ..
            } => {
                write!(f, "GameReComp CLI failed: {message}")?;
                if let Some(status) = status {
                    write!(f, " (status {status})")?;
                }
                if !stderr.is_empty() {
                    write!(f, "\nCLI stderr: {}", stderr.trim())?;
                }
                Ok(())
            }
            Error::NoReport { path, .. } => {
                write!(
                    f,
                    "pipeline produced no readable report: {}",
                    path.display()
                )
            }
            Error::Version { message, .. } => write!(f, "CLI version query failed: {message}"),
            Error::TimedOut { duration } => {
                write!(f, "CLI did not finish within {:?}", duration)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Failed {
                source: Some(source),
                ..
            }
            | Error::Version {
                source: Some(source),
                ..
            } => Some(source),
            Error::NoReport { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// A validated handle to a `game-recomp-cli` binary.
///
/// Paths are passed to the CLI as individual argv elements (never through a
/// shell), so CLI paths, PE paths and project directories containing spaces or
/// other shell metacharacters are handled safely.
#[derive(Debug)]
pub struct Client {
    cli: PathBuf,
    timeout: Option<Duration>,
}

/// High-level build options: guest/host formats and pipeline shape.
/// `None` means "let the engine choose its default".
#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    pub input_os: Option<String>,
    pub input_arch: Option<String>,
    pub target_os: Option<String>,
    pub target_arch: Option<String>,
    pub out_mode: Option<String>,
    pub skip_stages: Option<String>,
}

impl BuildOptions {
    #[must_use] pub fn input_os(mut self, v: impl Into<String>) -> Self { self.input_os = Some(v.into()); self }
    #[must_use] pub fn input_arch(mut self, v: impl Into<String>) -> Self { self.input_arch = Some(v.into()); self }
    #[must_use] pub fn target_os(mut self, v: impl Into<String>) -> Self { self.target_os = Some(v.into()); self }
    #[must_use] pub fn target_arch(mut self, v: impl Into<String>) -> Self { self.target_arch = Some(v.into()); self }
    #[must_use] pub fn out_mode(mut self, v: impl Into<String>) -> Self { self.out_mode = Some(v.into()); self }
    #[must_use] pub fn skip_stages(mut self, v: impl Into<String>) -> Self { self.skip_stages = Some(v.into()); self }
}

impl Client {
    /// Construct a client without an engine path. The SDK discovers the
    /// engine from `GAMERECOMP_CLI` or PATH; callers never hard-code a path.
    pub fn from_env() -> Result<Self, Error> {
        let cli = std::env::var_os("GAMERECOMP_CLI")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("game-recomp-cli"));
        Self::new(cli)
    }

    /// Validate `cli` and construct a client with no timeout (blocking mode).
    pub fn new(cli: impl AsRef<Path>) -> Result<Self, Error> {
        Self::new_with_timeout(cli, None)
    }

    /// Validate `cli` and construct a client with an optional per-command
    /// timeout. Pass `None` for the default, unbounded wait.
    pub fn new_with_timeout(
        cli: impl AsRef<Path>,
        timeout: Option<Duration>,
    ) -> Result<Self, Error> {
        let cli = cli.as_ref().to_path_buf();
        if !cli.is_file() {
            return Err(Error::MissingCli { path: cli });
        }
        Ok(Self { cli, timeout })
    }

    /// Attach a timeout to every CLI command issued by this client.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Replace or clear the current timeout (`None` means unbounded).
    pub fn set_timeout(&mut self, timeout: Option<Duration>) {
        self.timeout = timeout;
    }

    /// Return the currently configured timeout, if any.
    #[must_use]
    pub fn timeout(&self) -> Option<Duration> {
        self.timeout
    }

    /// Return the validated CLI path.
    #[must_use]
    pub fn cli(&self) -> &Path {
        &self.cli
    }

    /// Query the CLI version by running `<cli> --version`.
    ///
    /// Returns the trimmed version text on a successful, non-empty response.
    /// Some tools write their banner to stderr, which is used as a fallback.
    pub fn version(&self) -> Result<String, Error> {
        let mut cmd = Command::new(&self.cli);
        cmd.arg("--version");
        let output = self.run_command(&mut cmd, "version query")?;
        if !output.status.success() {
            return Err(Error::Version {
                message: format!("`--version` exited with status {}", output.status),
                source: None,
            });
        }
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !stdout.is_empty() {
            return Ok(stdout);
        }
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(Error::Version {
                message: "`--version` produced no output".to_string(),
                source: None,
            })
        } else {
            Ok(stderr)
        }
    }

    /// Recompile with engine defaults (convenience wrapper over [`Client::build`]).
    pub fn recompile(
        &self,
        pe: impl AsRef<Path>,
        project: impl AsRef<Path>,
    ) -> Result<Report, Error> {
        self.build(pe, project, BuildOptions::default())
    }

    /// Build `pe` into `project` with explicit guest/host options.
    /// The path to the engine is managed by the SDK, not by the caller.
    pub fn build(
        &self,
        pe: impl AsRef<Path>,
        project: impl AsRef<Path>,
        opts: BuildOptions,
    ) -> Result<Report, Error> {
        let pe = pe.as_ref();
        let project = project.as_ref();
        if !pe.is_file() {
            return Err(Error::MissingInput { path: pe.to_path_buf() });
        }
        let mut cmd = Command::new(&self.cli);
        cmd.arg("run").arg(pe).arg("--project-dir").arg(project)
           .arg("--no-table").arg("--no-progress");
        for (flag, value) in [
            ("--input-os", opts.input_os), ("--input-arch", opts.input_arch),
            ("--target-os", opts.target_os), ("--target-arch", opts.target_arch),
            ("--out-mode", opts.out_mode), ("--skip-stages", opts.skip_stages),
        ] {
            if let Some(v) = value { cmd.arg(flag).arg(v); }
        }
        let output = self.run_command(&mut cmd, "build")?;
        if !output.status.success() {
            return Err(Error::Failed {
                message: "CLI exited unsuccessfully".to_string(),
                status: Some(output.status),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                source: None,
            });
        }
        let path = project.join("reports").join("report.json");
        let raw = std::fs::read_to_string(&path).map_err(|source| Error::NoReport { path: path.clone(), source })?;
        Ok(Report { path, score: parse_score(&raw) })
    }

    /// Spawn `cmd`, capture its output, and enforce the configured timeout.
    ///
    /// Stdout and stderr are drained on dedicated threads so a chatty CLI
    /// cannot deadlock on a full pipe while the parent waits for exit.
    fn run_command(&self, cmd: &mut Command, context: &str) -> Result<Output, Error> {
        let mut child = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| Error::Failed {
                message: format!("failed to spawn CLI for {context}"),
                status: None,
                stderr: String::new(),
                source: Some(source),
            })?;

        let mut stdout_pipe = child.stdout.take().expect("stdout is piped");
        let mut stderr_pipe = child.stderr.take().expect("stderr is piped");
        let stdout_thread = thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stdout_pipe.read_to_end(&mut buf);
            buf
        });
        let stderr_thread = thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stderr_pipe.read_to_end(&mut buf);
            buf
        });

        let wait_result = match self.timeout {
            Some(timeout) => wait_with_timeout(&mut child, timeout),
            None => child.wait(),
        };

        match wait_result {
            Ok(status) => Ok(Output {
                status,
                stdout: stdout_thread.join().unwrap_or_default(),
                stderr: stderr_thread.join().unwrap_or_default(),
            }),
            Err(source) if source.kind() == io::ErrorKind::TimedOut => {
                // A killed child may leave grandchildren holding the pipe write
                // ends open, so detach the readers instead of joining them.
                drop(stdout_thread);
                drop(stderr_thread);
                Err(Error::TimedOut {
                    duration: self.timeout.expect("timeout is set on this path"),
                })
            }
            Err(source) => Err(Error::Failed {
                message: format!("failed while waiting for CLI during {context}"),
                status: None,
                stderr: String::new(),
                source: Some(source),
            }),
        }
    }
}

/// Wait for `child`, killing and reaping it if `timeout` elapses first.
fn wait_with_timeout(child: &mut Child, timeout: Duration) -> io::Result<ExitStatus> {
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "timeout is too large"))?;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        let now = Instant::now();
        if now >= deadline {
            let _ = child.kill();
            let _ = child.wait(); // reap to avoid a zombie
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "CLI did not exit before the timeout",
            ));
        }
        thread::sleep(deadline.saturating_duration_since(now).min(POLL_INTERVAL));
    }
}

/// Extract the integer value of the `"score"` field from a report document.
///
/// The parser deliberately stays small: it looks for the exact JSON key
/// `"score"`, skips any whitespace around the key, the colon and the value,
/// then reads an optional minus sign followed by decimal digits. It returns
/// `None` when the key is absent, the value is not an integer, or the number
/// does not fit in `i64`. A key such as `"score_notes"` is not confused with
/// `"score"`, and a malformed occurrence is skipped in favour of a later
/// valid one.
#[must_use]
pub fn parse_score(json: &str) -> Option<i64> {
    const KEY: &[u8] = b"\"score\"";
    let bytes = json.as_bytes();
    let mut cursor = 0usize;

    while cursor + KEY.len() <= bytes.len() {
        let offset = bytes[cursor..]
            .windows(KEY.len())
            .position(|window| window == KEY)?;
        let after_key = cursor + offset + KEY.len();
        cursor = after_key; // always advance so the loop cannot stall

        let mut i = skip_ws(bytes, after_key);
        if i >= bytes.len() || bytes[i] != b':' {
            continue;
        }
        i = skip_ws(bytes, i + 1);

        let value_start = i;
        if i < bytes.len() && bytes[i] == b'-' {
            i += 1;
        }
        let digits_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == digits_start {
            continue;
        }
        // `score` is an integer field: ignore floats and exponent notation.
        if matches!(bytes.get(i), Some(&b'.') | Some(&b'e') | Some(&b'E')) {
            continue;
        }

        let token = std::str::from_utf8(&bytes[value_start..i]).ok()?;
        if let Ok(score) = token.parse::<i64>() {
            return Some(score);
        }
        // Overflow (or an out-of-range signed token): try a later occurrence.
    }
    None
}

fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && (bytes[i] as char).is_whitespace() {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    /// Minimal temporary directory: no external crates needed.
    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            Self::with_name("gamerecomp-sdk-test")
        }

        /// Create a temp directory whose name contains a space, to exercise
        /// path-with-spaces handling end to end.
        fn with_space() -> Self {
            Self::with_name("gamerecomp sdk test with spaces")
        }

        fn with_name(prefix: &str) -> Self {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock is after the epoch")
                .as_nanos();
            let path =
                std::env::temp_dir().join(format!("{prefix}-{}-{nanos}-{id}", std::process::id()));
            std::fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }

        fn write_report(&self, json: &str) -> PathBuf {
            let reports = self.path.join("reports");
            std::fs::create_dir_all(&reports).expect("create reports dir");
            let file = reports.join("report.json");
            std::fs::write(&file, json).expect("write report");
            file
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn missing_cli_is_rejected() {
        let err = Client::new("/definitely/not/a/cli").unwrap_err();
        assert!(matches!(err, Error::MissingCli { .. }), "got {err:?}");
    }

    #[test]
    fn type_names_are_public() {
        let _: Option<Error> = None;
        let _ = std::mem::size_of::<Report>();
    }

    #[test]
    fn parse_score_compact_document() {
        assert_eq!(parse_score(r#"{"score":70,"stages":[]}"#), Some(70));
        assert_eq!(parse_score(r#"{"score":0}"#), Some(0));
    }

    #[test]
    fn parse_score_whitespace_everywhere() {
        let json = "{\n  \"score\"  :  \t -12 ,\n  \"stages\" : []\n}\n";
        assert_eq!(parse_score(json), Some(-12));
        assert_eq!(parse_score("\"score\"\r\n:\n\n 42\t"), Some(42));
    }

    #[test]
    fn parse_score_missing_field() {
        assert_eq!(parse_score(r#"{"stages":[],"ok":true}"#), None);
        assert_eq!(parse_score(""), None);
    }

    #[test]
    fn parse_score_ignores_similarly_named_keys() {
        assert_eq!(parse_score(r#"{"score_notes":5,"score_total":9}"#), None);
        assert_eq!(parse_score(r#"{"score_detail":{"score":3}}"#), Some(3));
    }

    #[test]
    fn parse_score_skips_malformed_occurrence() {
        assert_eq!(parse_score(r#"{"score":true,"score":42}"#), Some(42));
        assert_eq!(parse_score(r#"{"score":7.5,"score":8}"#), Some(8));
    }

    #[test]
    fn parse_score_rejects_non_integers() {
        assert_eq!(parse_score(r#"{"score":"70"}"#), None);
        assert_eq!(parse_score(r#"{"score":70.5}"#), None);
        assert_eq!(parse_score(r#"{"score":null}"#), None);
        assert_eq!(parse_score(r#"{"score": }"#), None);
    }

    #[test]
    fn parse_score_bounds() {
        assert_eq!(
            parse_score("{\"score\":9223372036854775807}"),
            Some(i64::MAX)
        );
        assert_eq!(
            parse_score("{\"score\":-9223372036854775808}"),
            Some(i64::MIN)
        );
        assert_eq!(parse_score("{\"score\":9223372036854775808}"), None);
    }

    #[test]
    fn report_json_returns_exact_file_contents() {
        let tmp = TempDir::new();
        let raw = "{\n  \"score\" : 11,\n  \"stages\" : []\n}\n";
        let path = tmp.write_report(raw);

        // Build the report directly: this keeps the two-field public struct
        // literal working for downstream callers.
        let report = Report {
            path,
            score: parse_score(raw),
        };
        assert_eq!(report.score, Some(11));
        assert_eq!(report.report_json(), raw);
    }

    #[test]
    fn report_json_missing_file_is_empty() {
        let tmp = TempDir::new();
        let report = Report {
            path: tmp.path.join("nope.json"),
            score: None,
        };
        assert_eq!(report.report_json(), "");
    }

    #[test]
    fn missing_input_is_rejected() {
        let tmp = TempDir::new();
        let cli = tmp.path.join("cli");
        std::fs::write(&cli, "#!/bin/sh\nexit 0\n").expect("write cli");
        let client = Client::new(&cli).expect("client");
        let err = client
            .recompile(tmp.path.join("missing.exe"), &tmp.path)
            .unwrap_err();
        assert!(matches!(err, Error::MissingInput { .. }), "got {err:?}");
    }

    #[test]
    fn timeout_accessors() {
        let tmp = TempDir::new();
        let cli = tmp.path.join("cli");
        std::fs::write(&cli, "#!/bin/sh\nexit 0\n").expect("write cli");
        let client = Client::new(&cli).expect("client");
        assert_eq!(client.timeout(), None);
        assert_eq!(client.cli(), cli.as_path());

        let client = client.with_timeout(Duration::from_secs(3));
        assert_eq!(client.timeout(), Some(Duration::from_secs(3)));
    }

    #[cfg(unix)]
    mod cli {
        use super::*;

        fn fake_cli(root: &Path, body: &str) -> PathBuf {
            // Write via a unique temp name then rename. On Linux, executing a
            // script whose inode was just written can fail with ETXTBSY when
            // another test/thread still holds it open; rename publishes a new
            // inode and avoids that flake.
            let final_path = root.join("game-recomp-cli");
            // Unique per call: tests run in parallel within one process, so a
            // PID-only staging name can collide and leave ETXTBSY on Linux.
            static CLI_SEQ: std::sync::atomic::AtomicUsize =
                std::sync::atomic::AtomicUsize::new(0);
            let seq = CLI_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let staging = root.join(format!(
                "game-recomp-cli.tmp-{}-{seq}", std::process::id()));
            std::fs::write(&staging, format!("#!/bin/sh\n{body}\n")).expect("write cli");
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&staging, std::fs::Permissions::from_mode(0o755))
                .expect("chmod cli");
            std::fs::rename(&staging, &final_path).expect("publish cli");
            final_path
        }

        fn pe(root: &Path) -> PathBuf {
            let input = root.join("game.exe");
            std::fs::write(&input, b"MZ").expect("write pe");
            input
        }

        #[test]
        fn recompile_parses_score_from_fake_cli_report() {
            let tmp = TempDir::new();
            let cli = fake_cli(&tmp.path,
                "while [ \"$#\" -gt 0 ]; do\n\
                   case \"$1\" in --project-dir) proj=\"$2\"; shift 2 ;; *) shift ;; esac\n\
                 done\n\
                 mkdir -p \"$proj/reports\"\n\
                 printf '%s' '{\"score\" :  87 , \"stages\" : []}' > \"$proj/reports/report.json\"\n\
                 exit 0");
            let project = tmp.path.join("proj");
            let report = Client::new(&cli)
                .expect("client")
                .recompile(pe(&tmp.path), &project)
                .expect("recompile");

            assert_eq!(report.score, Some(87));
            assert_eq!(report.path, project.join("reports").join("report.json"));
            assert!(report.report_json().contains("\"stages\""));
            assert_eq!(parse_score(&report.report_json()), Some(87));
        }

        #[test]
        fn recompile_handles_paths_with_spaces() {
            let tmp = TempDir::with_space();
            let nested = tmp.path.join("cli dir");
            std::fs::create_dir_all(&nested).expect("create nested dir");
            let cli = fake_cli(
                &nested,
                "while [ \"$#\" -gt 0 ]; do\n\
                   case \"$1\" in --project-dir) proj=\"$2\"; shift 2 ;; *) shift ;; esac\n\
                 done\n\
                 mkdir -p \"$proj/reports\"\n\
                 printf '%s' '{\"score\" : 5}' > \"$proj/reports/report.json\"\n\
                 exit 0",
            );
            let spaced_pe = nested.join("my game.exe");
            std::fs::write(&spaced_pe, b"MZ").expect("write pe");
            let project = nested.join("output project");

            let report = Client::new(&cli)
                .expect("client")
                .recompile(&spaced_pe, &project)
                .expect("recompile");
            assert_eq!(report.score, Some(5));
            assert_eq!(report.path, project.join("reports").join("report.json"));
            assert!(report.path.is_file());
        }

        #[test]
        fn recompile_fails_without_report() {
            let tmp = TempDir::new();
            let cli = fake_cli(&tmp.path, "exit 0");
            let err = Client::new(&cli)
                .expect("client")
                .recompile(pe(&tmp.path), tmp.path.join("empty"))
                .unwrap_err();
            assert!(matches!(err, Error::NoReport { .. }), "got {err:?}");
            assert!(
                std::error::Error::source(&err).is_some(),
                "NoReport must expose its IO source"
            );
        }

        #[test]
        fn recompile_reports_cli_failure_with_stderr() {
            let tmp = TempDir::new();
            let cli = fake_cli(&tmp.path, "printf 'boom' >&2\nexit 7");
            let err = Client::new(&cli)
                .expect("client")
                .recompile(pe(&tmp.path), &tmp.path)
                .unwrap_err();
            match &err {
                Error::Failed {
                    status,
                    stderr,
                    source,
                    ..
                } => {
                    assert_eq!(status.as_ref().and_then(|s| s.code()), Some(7));
                    assert!(stderr.contains("boom"), "stderr = {stderr:?}");
                    assert!(source.is_none());
                }
                other => panic!("unexpected error: {other:?}"),
            }
        }

        #[test]
        fn version_returns_trimmed_banner() {
            let tmp = TempDir::new();
            let cli = fake_cli(&tmp.path, "printf 'game-recomp-cli 1.2.3\\n'");
            assert_eq!(
                Client::new(&cli)
                    .expect("client")
                    .version()
                    .expect("version"),
                "game-recomp-cli 1.2.3"
            );
        }

        #[test]
        fn version_falls_back_to_stderr() {
            let tmp = TempDir::new();
            let cli = fake_cli(&tmp.path, "printf ' banner-on-stderr \n' >&2");
            assert_eq!(
                Client::new(&cli)
                    .expect("client")
                    .version()
                    .expect("version"),
                "banner-on-stderr"
            );
        }

        #[test]
        fn version_reports_empty_and_failure() {
            let tmp = TempDir::new();
            let empty = fake_cli(&tmp.path, "exit 0");
            let err = Client::new(&empty).expect("client").version().unwrap_err();
            assert!(matches!(err, Error::Version { .. }), "got {err:?}");

            let failing = tmp.path.join("failing-cli");
            std::fs::write(&failing, "#!/bin/sh\nexit 3\n").expect("write cli");
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&failing, std::fs::Permissions::from_mode(0o755))
                .expect("chmod cli");
            let err = Client::new(&failing)
                .expect("client")
                .version()
                .unwrap_err();
            assert!(matches!(err, Error::Version { .. }), "got {err:?}");
        }

        #[test]
        fn build_passes_guest_and_host_options() {
            let tmp = TempDir::new();
            let log = tmp.path.join("argv.txt");
            let body = format!(
                "printf '%s\\n' \"$@\" > \"{}\"\n\
                 while [ \"$#\" -gt 0 ]; do\n\
                   case \"$1\" in --project-dir) proj=\"$2\"; shift 2 ;; *) shift ;; esac\n\
                 done\n\
                 mkdir -p \"$proj/reports\"\n\
                 printf '{{\"score\":70}}' > \"$proj/reports/report.json\"\n\
                 exit 0\n", log.display());
            let cli = fake_cli(&tmp.path, &body);
            let opts = BuildOptions::default()
                .input_os("windows").input_arch("x86")
                .target_os("macos").target_arch("arm64")
                .out_mode("full").skip_stages("verify");
            let report = Client::new(&cli).expect("client")
                .build(pe(&tmp.path), &tmp.path, opts).expect("build");
            assert_eq!(report.score, Some(70));
            let args = std::fs::read_to_string(&log).expect("argv log");
            for pair in [("--input-os", "windows"), ("--input-arch", "x86"),
                         ("--target-os", "macos"), ("--target-arch", "arm64"),
                         ("--out-mode", "full"), ("--skip-stages", "verify")] {
                assert!(args.contains(pair.0), "missing {}", pair.0);
                assert!(args.contains(pair.1), "missing {}", pair.1);
            }
        }

        #[test]
        fn timeout_kills_slow_cli() {
            let tmp = TempDir::new();
            let cli = fake_cli(&tmp.path, "sleep 2\nexit 0");
            let client =
                Client::new_with_timeout(&cli, Some(Duration::from_millis(100))).expect("client");
            let err = client.recompile(pe(&tmp.path), &tmp.path).unwrap_err();
            assert!(matches!(err, Error::TimedOut { .. }), "got {err:?}");
        }
    }
}
