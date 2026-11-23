//! Example: call GameReComp from Rust.
use gamerecomp_sdk::Client;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        eprintln!("usage: basic <cli> <pe> <project>");
        std::process::exit(2);
    }
    match Client::new(&args[0]).and_then(|c| c.recompile(&args[1], &args[2])) {
        Ok(report) => println!("report={} score={:?}", report.path.display(), report.score),
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    }
}
