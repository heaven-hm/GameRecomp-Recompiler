//! Working Rust app: typed report plus raw JSON from the SDK.
use gamerecomp_sdk::Client;
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3 { eprintln!("usage: app <cli> <pe> <project>"); std::process::exit(2); }
    let client = match Client::new(&args[0]) { Ok(c) => c, Err(e) => { eprintln!("init: {e}"); std::process::exit(1); } };
    let report = match client.recompile(&args[1], &args[2]) { Ok(r) => r, Err(e) => { eprintln!("recompile: {e}"); std::process::exit(1); } };
    let raw = report.report_json();
    println!("APP_OK sdk=rust score={:?} report={} json_bytes={}", report.score, report.path.display(), raw.len());
}
