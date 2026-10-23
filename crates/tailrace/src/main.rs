use std::io::Write;

use clap::Parser;
use tailrace::{run, Cli};

/// The command line over no binding: no host binds its seams until its
/// bindings file does (118), so publish and verify answer absent and
/// status reads the host's record.
fn main() {
    let cli = Cli::parse();
    let at = std::env::current_dir().expect("a working directory");
    let mut out = std::io::stdout().lock();
    let code = run(&cli, None, &at, &mut out);
    let _ = out.flush();
    std::process::exit(code);
}
