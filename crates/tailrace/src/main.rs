use std::io::Write;

use clap::Parser;
use tailrace::bindings::{self, For};
use tailrace::{run, Cli, Command};

/// The command line over the host's bindings file (118): publish, verify,
/// sync, push, rebuild, promote, status and policy are bound by it, only
/// publish and push making the layout it names, and status and policy
/// bound on a host with none to what they ask (60).
fn main() {
    let cli = Cli::parse();
    let at = std::env::current_dir().expect("a working directory");
    let binding = match &cli.command {
        Command::Publish(_) => bindings::load(&at, For::Publish),
        Command::Verify(_) => bindings::load(&at, For::Verify),
        Command::Sync(_) => bindings::load(&at, For::Sync),
        Command::Push(_) => bindings::load(&at, For::Push),
        Command::Rebuild(_) => bindings::load(&at, For::Rebuild),
        Command::Promote(_) => bindings::load(&at, For::Promote),
        Command::Status(_) => bindings::load(&at, For::Status),
        Command::Policy(_) => bindings::load(&at, For::Policy),
    };
    let mut out = std::io::stdout().lock();
    let code = run(&cli, &binding, &at, &mut out);
    let _ = out.flush();
    std::process::exit(code);
}
