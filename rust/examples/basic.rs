//! Example: call GameReComp from Rust.
//!
//! Usage:
//!   basic <cli> <pe> <project>
//!
//! Optional environment variable:
//!   GAMERECOMP_TIMEOUT_MS  per-command timeout in milliseconds
//!                          (unset/invalid means no timeout)
use gamerecomp_sdk::Client;
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        eprintln!("usage: basic <cli> <pe> <project>");
        std::process::exit(2);
    }

    let client = match Client::new(&args[0]) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("init: {e}");
            std::process::exit(1);
        }
    };

    let client = match std::env::var("GAMERECOMP_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|&ms| ms > 0)
    {
        Some(ms) => client.with_timeout(Duration::from_millis(ms)),
        None => client,
    };

    // A version query is optional: some CLI builds do not implement
    // `--version`. Treat a failure as informational, not fatal.
    match client.version() {
        Ok(version) => println!("cli={version}"),
        Err(e) => eprintln!("version query unavailable: {e}"),
    }

    match client.recompile(&args[1], &args[2]) {
        Ok(report) => {
            println!("report={} score={:?}", report.path.display(), report.score);
            // The untouched report.json text is available when the SDK's
            // parsed fields are not enough for your use case.
            println!("json={}", report.report_json().trim());
        }
        Err(e) => {
            eprintln!("recompile: {e}");
            // Structured errors expose their underlying I/O cause when one
            // exists (for example a spawn failure).
            if let Some(source) = std::error::Error::source(&e) {
                eprintln!("caused by: {source}");
            }
            std::process::exit(1);
        }
    }
}
