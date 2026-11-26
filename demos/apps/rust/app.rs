//! app.rs - Rust example for the GameRecomp SDK.
//!
//! Usage: app <game-recomp-cli> <pe32-x86.exe> <existing-project-dir>
//!
//! The CLI path is supplied by this demo harness. Production callers use
//! `Client::from_env()`, which discovers the engine internally.

use gamerecomp_sdk::{BuildOptions, Client};

fn build_options() -> BuildOptions {
    BuildOptions::default()
        .input_os("windows")
        .input_arch("x86")
        .target_os("host")
        .target_arch("host")
        .out_mode("full")
}

fn run(cli: &str, pe: &str, project: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(cli)?;
    let report = client.build(pe, project, build_options())?;
    let raw_json = report.report_json();
    let score = report
        .score
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_owned());

    println!(
        "APP_OK sdk=rust score={} report={} json_bytes={}",
        score,
        report.path.display(),
        raw_json.len()
    );
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        eprintln!("usage: app <cli> <pe> <project>");
        std::process::exit(2);
    }

    if let Err(error) = run(&args[0], &args[1], &args[2]) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
