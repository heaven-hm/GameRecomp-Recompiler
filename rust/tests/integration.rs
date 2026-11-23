//! Integration tests against a *real* `game-recomp-cli`.
//!
//! These tests are deliberately opt-in so that `cargo test` stays hermetic in
//! CI. Supply a real CLI path via `GAMERECOMP_CLI` and a PE32 input via
//! `GAMERECOMP_TEST_PE`; when either is absent the test prints an explicit
//! skip reason and passes.
//!
//! ```sh
//! GAMERECOMP_CLI=/path/to/game-recomp-cli \
//! GAMERECOMP_TEST_PE=/path/to/game.exe \
//!     cargo test --test integration -- --nocapture
//! ```
//!
//! Both variables are read from the environment rather than hard-coded, and
//! the values are passed to the CLI as individual argv elements, so paths with
//! spaces are handled correctly.

use gamerecomp_sdk::{Client, Error};
use std::path::PathBuf;
use std::time::Duration;

/// Generous ceiling for a full recompilation of a real game PE.
const INTEGRATION_TIMEOUT: Duration = Duration::from_secs(900);

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
}

fn skip(reason: &str) {
    eprintln!("SKIP integration test: {reason}");
}

/// Remove the scratch project directory on drop, even if the test panics.
struct ScratchProject(PathBuf);

impl ScratchProject {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "gamerecomp-sdk-integration-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock is after the epoch")
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("create scratch project dir");
        Self(path)
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for ScratchProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn real_cli_version_query() {
    let Some(cli) = env_path("GAMERECOMP_CLI") else {
        skip("GAMERECOMP_CLI is not set; set it to a real game-recomp-cli path to run this test");
        return;
    };
    if !cli.is_file() {
        panic!("GAMERECOMP_CLI is set but is not a file: {}", cli.display());
    }

    let client = Client::new_with_timeout(&cli, Some(Duration::from_secs(30)))
        .expect("construct client for real CLI");
    match client.version() {
        Ok(version) => println!("real CLI version: {version}"),
        // A absent `--version` support is not a recompilation failure; report
        // it clearly instead of failing the whole integration suite.
        Err(e) => eprintln!("real CLI did not answer --version: {e}"),
    }
}

#[test]
fn real_cli_recompile() {
    let Some(cli) = env_path("GAMERECOMP_CLI") else {
        skip("GAMERECOMP_CLI is not set; set it to a real game-recomp-cli path to run this test");
        return;
    };
    let Some(pe) = env_path("GAMERECOMP_TEST_PE") else {
        skip("GAMERECOMP_TEST_PE is not set; set it to a real PE32 input to run this test");
        return;
    };
    if !cli.is_file() {
        panic!("GAMERECOMP_CLI is set but is not a file: {}", cli.display());
    }
    if !pe.is_file() {
        panic!(
            "GAMERECOMP_TEST_PE is set but is not a file: {}",
            pe.display()
        );
    }

    let project = ScratchProject::new();
    let client = Client::new_with_timeout(&cli, Some(INTEGRATION_TIMEOUT))
        .expect("construct client for real CLI");

    let report = client
        .recompile(&pe, project.path())
        .expect("real recompile must succeed");
    assert!(report.path.is_file(), "report.json was not produced");
    let raw = report.report_json();
    assert!(!raw.is_empty(), "report.json was empty");
    // A report without a score is legal; when present it must agree with the
    // SDK's parse of the same document.
    assert_eq!(report.score, gamerecomp_sdk::parse_score(&raw));

    match client.version() {
        Ok(version) => println!("real CLI version: {version}"),
        Err(Error::Version { message, .. }) => eprintln!("version query unavailable: {message}"),
        Err(e) => panic!("unexpected version error: {e}"),
    }
}
