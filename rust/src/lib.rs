//! Rust client SDK for GameReComp.
//!
//! Thin wrapper around `game-recomp-cli run`; all recompilation is performed
//! by GameReComp itself.
use std::path::{Path, PathBuf};
use std::process::Command;

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
    pub fn report_json(&self) -> String {
        std::fs::read_to_string(&self.path).unwrap_or_default()
    }
}

#[derive(Debug)]
pub enum Error {
    MissingCli(String),
    MissingInput(String),
    Failed(String),
    NoReport(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingCli(p) => write!(f, "CLI binary does not exist: {p}"),
            Error::MissingInput(p) => write!(f, "PE32 input does not exist: {p}"),
            Error::Failed(s) => write!(f, "GameReComp CLI failed: {s}"),
            Error::NoReport(p) => write!(f, "pipeline produced no report: {p}"),
        }
    }
}
impl std::error::Error for Error {}

pub struct Client {
    cli: PathBuf,
}

impl Client {
    pub fn new(cli: impl AsRef<Path>) -> Result<Self, Error> {
        let cli = cli.as_ref().to_path_buf();
        if !cli.is_file() {
            return Err(Error::MissingCli(cli.display().to_string()));
        }
        Ok(Self { cli })
    }

    pub fn recompile(&self, pe: impl AsRef<Path>, project: impl AsRef<Path>) -> Result<Report, Error> {
        let pe = pe.as_ref();
        let project = project.as_ref();
        if !pe.is_file() {
            return Err(Error::MissingInput(pe.display().to_string()));
        }
        let out = Command::new(&self.cli)
            .arg("run").arg(pe)
            .arg("--project-dir").arg(project)
            .arg("--no-table").arg("--no-progress")
            .output()
            .map_err(|e| Error::Failed(e.to_string()))?;
        if !out.status.success() {
            return Err(Error::Failed(format!("exit {:?}", out.status.code())));
        }
        let path = project.join("reports").join("report.json");
        let raw = std::fs::read_to_string(&path)
            .map_err(|_| Error::NoReport(path.display().to_string()))?;
        let score = parse_score(&raw);
        Ok(Report { path, score })
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
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock is after the epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "gamerecomp-sdk-test-{}-{nanos}-{id}",
                std::process::id()
            ));
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
        assert!(Client::new("/definitely/not/a/cli").is_err());
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
        assert_eq!(parse_score("{\"score\":9223372036854775807}"), Some(i64::MAX));
        assert_eq!(parse_score("{\"score\":-9223372036854775808}"), Some(i64::MIN));
        assert_eq!(parse_score("{\"score\":9223372036854775808}"), None);
    }

    #[test]
    fn report_json_returns_exact_file_contents() {
        let tmp = TempDir::new();
        let raw = "{\n  \"score\" : 11,\n  \"stages\" : []\n}\n";
        let path = tmp.write_report(raw);

        // Build the report directly: this keeps the two-field public struct
        // literal working for downstream callers.
        let report = Report { path, score: parse_score(raw) };
        assert_eq!(report.score, Some(11));
        assert_eq!(report.report_json(), raw);
    }

    #[test]
    fn report_json_missing_file_is_empty() {
        let tmp = TempDir::new();
        let report = Report { path: tmp.path.join("nope.json"), score: None };
        assert_eq!(report.report_json(), "");
    }

    #[test]
    fn missing_input_is_rejected() {
        let tmp = TempDir::new();
        let cli = tmp.path.join("cli");
        std::fs::write(&cli, "#!/bin/sh\nexit 0\n").expect("write cli");
        let client = Client::new(&cli).expect("client");
        assert!(client.recompile(tmp.path.join("missing.exe"), &tmp.path).is_err());
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
            let staging = root.join(format!("game-recomp-cli.tmp-{}", std::process::id()));
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
        fn recompile_fails_without_report() {
            let tmp = TempDir::new();
            let cli = fake_cli(&tmp.path, "exit 0");
            let err = Client::new(&cli)
                .expect("client")
                .recompile(pe(&tmp.path), tmp.path.join("empty"))
                .unwrap_err();
            assert!(matches!(err, Error::NoReport(_)), "got {err:?}");
        }

        #[test]
        fn recompile_reports_cli_failure() {
            let tmp = TempDir::new();
            let cli = fake_cli(&tmp.path, "exit 7");
            let err = Client::new(&cli)
                .expect("client")
                .recompile(pe(&tmp.path), &tmp.path)
                .unwrap_err();
            assert!(matches!(err, Error::Failed(_)), "got {err:?}");
        }
    }
}
