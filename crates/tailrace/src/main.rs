use clap::Parser;

/// Build once, sign everything, promote with proof, in your own account.
#[derive(Parser)]
#[command(name = "tailrace", version = tailrace::VERSION_LINE, about)]
struct Cli {}

fn main() {
    let Cli {} = Cli::parse();
    println!("tailrace {}", tailrace::VERSION_LINE);
    println!(
        "Nothing of the design is built yet: see https://agentplot.github.io/tailrace/roadmap/"
    );
}
