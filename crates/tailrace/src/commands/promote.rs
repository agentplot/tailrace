//! `tailrace promote <version> --to <channel>`: a channel moved to a
//! version of its component by a promotion the host's signer makes, held
//! by the ledger before the store follows; `--back` moves it to the
//! version its head moved from (31 to 39, 105). It reads the asker's time
//! from the clock and hands it to the core, which reads none.

use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{ArgGroup, Args};
use tailrace_core::{promote, Followed, History, Mirrored, PromoteAsk, Promoted, To};
use tailrace_seams::{Answer, Branch, ChannelName, ComponentName, Moved, Not, Version};

use crate::{render, unsigned, Binding};

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("where").required(true).args(["version", "back"])))]
pub struct PromoteArgs {
    /// The version to promote, bare of its component's prefix.
    #[arg(conflicts_with = "back")]
    pub version: Option<String>,
    /// The channel to move.
    #[arg(long)]
    pub to: String,
    /// Move the channel back to the version its head moved from.
    #[arg(long)]
    pub back: bool,
    /// The version the channel is expected at; refused where it is
    /// elsewhere.
    #[arg(long)]
    pub from: Option<String>,
    /// The component the version is of; needed only where more than one
    /// is declared.
    #[arg(long)]
    pub component: Option<String>,
    /// The branch whose landed head holds the policy.
    #[arg(long, default_value = "main")]
    pub branch: String,
    /// Print the answer as one JSON document.
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &PromoteArgs, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    let answer = ask(args, binding, at);
    let _ = if args.json {
        render::json(out, &answer)
    } else {
        text(out, &answer)
    };
    render::exit(&answer)
}

fn ask(args: &PromoteArgs, binding: &Answer<Binding>, at: &Path) -> Answer<Promoted> {
    let binding = binding.as_ref().map_err(Not::clone)?;
    let signer = binding.signer.clone().ok_or_else(unsigned)?;
    let history = History::open(at)?;
    let ask = PromoteAsk {
        branch: Branch::new(&args.branch),
        component: args.component.as_ref().map(ComponentName::new),
        channel: ChannelName::new(&args.to),
        to: args
            .version
            .as_ref()
            .map_or(To::Back, |v| To::Version(Version::new(v))),
        from: args.from.as_ref().map(Version::new),
        signer,
        at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
    };
    promote(&binding.seams, &history, &ask)
}

fn moved(moved: &Moved) -> String {
    format!("{} ({})", moved.version, moved.index)
}

/// How the store followed the ledger, or its own answer where it did not.
fn store(out: &mut dyn Write, store: &Answer<Mirrored>) -> io::Result<()> {
    let mirrored = match store {
        Ok(mirrored) => mirrored,
        Err(not) => {
            writeln!(
                out,
                "the ledger holds the move, and the store did not follow:"
            )?;
            return render::not(out, "  ", not);
        }
    };
    let name = match &mirrored.name {
        Followed::Moved { .. } => "the store followed".to_string(),
        Followed::There { .. } => "the store held it already".to_string(),
        Followed::LostRace { found, .. } => format!(
            "the store followed, moving the channel's name back from {}, where another writer had moved it",
            found
                .as_ref()
                .map_or_else(|| "nothing".to_string(), ToString::to_string)
        ),
    };
    write!(out, "{name}")?;
    if !mirrored.attached.is_empty() {
        let attached: Vec<String> = mirrored.attached.iter().map(ToString::to_string).collect();
        write!(out, ", attaching {}", attached.join(", "))?;
    }
    writeln!(out)
}

fn text(out: &mut dyn Write, answer: &Answer<Promoted>) -> io::Result<()> {
    match answer {
        Err(not) => render::not(out, "", not),
        Ok(Promoted::Moved {
            channel,
            to,
            from,
            claim,
            predecessor,
            store: mirrored,
            ..
        }) => {
            match (from, predecessor) {
                (Some(from), Some(predecessor)) => write!(
                    out,
                    "{channel} moved from {} to {}: claim {claim} after {predecessor}; ",
                    self::moved(from),
                    self::moved(to)
                )?,
                _ => write!(
                    out,
                    "{channel} moved to {}, its first promotion: claim {claim}; ",
                    self::moved(to)
                )?,
            }
            store(out, mirrored)
        }
        Ok(Promoted::There {
            channel,
            at,
            store: mirrored,
            ..
        }) => {
            write!(out, "{channel} is at {} already; ", self::moved(at))?;
            store(out, mirrored)
        }
    }
}
