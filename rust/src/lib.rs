//! Rust client SDK for GameReComp.
//!
//! Thin wrapper around `game-recomp-cli run`; all recompilation is performed
//! by GameReComp itself.
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Report {
    pub path: PathBuf,
    pub score: Option<i64>,
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
        if !path.is_file() {
            return Err(Error::NoReport(path.display().to_string()));
        }
        let score = std::fs::read_to_string(&path).ok().and_then(|s| {
            s.split("\"score\"").nth(1)
                .and_then(|t| t.split(|c: char| c.is_ascii_digit() || c == ',').nth(1)
                    .and_then(|n| n.parse::<i64>().ok()))
        });
        Ok(Report { path, score })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_cli_is_rejected() {
        assert!(Client::new("/definitely/not/a/cli").is_err());
    }

    #[test]
    fn type_names_are_public() {
        let _: Option<Error> = None;
        let _ = std::mem::size_of::<Report>();
    }
}
