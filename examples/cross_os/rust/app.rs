//! Port one PE32 app through the Rust SDK with explicit input/target options.
use gamerecomp_sdk::{BuildOptions, Client};
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3 { eprintln!("usage: app <direction> <pe> <project>"); std::process::exit(2); }
    let options = BuildOptions::default().input_os("windows").input_arch("x86")
        .target_os("host").target_arch("host").out_mode("full").skip_stages("verify");
    match Client::from_env().and_then(|c| c.build(&args[1], &args[2], options)) {
        Ok(report) => println!("PORT_OK direction={} report={} score={:?}", args[0], report.path.display(), report.score),
        Err(error) => println!("PORT_DEFERRED direction={} reason={}", args[0], error),
    }
}
