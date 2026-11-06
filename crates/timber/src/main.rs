use std::io::Write;

use clap::Parser;
use timber::bindings::{self, For};
use timber::{run, Cli, Command};
use timber_seams::{Not, Seam, Source};

/// The command line over the host's bindings file (118): publish, verify,
/// sync, push, rebuild, promote, apply, status and policy are bound by it,
/// only publish and push making the layout it names, `push --left`
/// needing no `[landing]`, status and policy bound on a host with none to
/// what they ask (60, 139), and apply with every target profile built
/// beside its store (92); serve binds nothing itself, and each request the
/// checkout that named its repository (132).
fn main() {
    let cli = Cli::parse();
    let at = std::env::current_dir().expect("a working directory");
    let binding = match &cli.command {
        Command::Publish(_) => bindings::load(&at, For::Publish),
        Command::Verify(_) => bindings::load(&at, For::Verify),
        Command::Sync(_) => bindings::load(&at, For::Sync),
        Command::Push(args) if args.left => bindings::load(&at, For::Left),
        Command::Push(_) => bindings::load(&at, For::Push),
        Command::Rebuild(_) => bindings::load(&at, For::Rebuild),
        Command::Promote(_) => bindings::load(&at, For::Promote),
        Command::Apply(_) => bindings::load(&at, For::Apply),
        Command::Status(_) => bindings::load(&at, For::Status),
        Command::Policy(_) => bindings::load(&at, For::Policy),
        Command::Serve(_) => Err(Not::absent(
            &Source::new(Seam::Host, "bindings"),
            "binding, since serve binds each request's checkout itself",
        )),
    };
    let mut out = std::io::stdout().lock();
    let code = run(&cli, &binding, &at, &mut out);
    let _ = out.flush();
    std::process::exit(code);
}
