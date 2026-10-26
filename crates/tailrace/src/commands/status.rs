//! `tailrace status`: the versions this host published, from its own
//! record alone, each with its notes rendered from the repository's own
//! history; it needs no seam and writes nothing (60, 61, 134, rule
//! core.32).

use std::io::{self, Write};
use std::path::Path;

use clap::Args;
use tailrace_core::{status, History, Kept, Status, StatusAsk};
use tailrace_seams::{Answer, ComponentName};

use crate::render;

#[derive(Debug, Args)]
pub struct StatusArgs {
    /// Only the versions of this component.
    #[arg(long)]
    pub component: Option<String>,
    /// Only the versions this revision became, any name git reads for a
    /// commit.
    #[arg(long)]
    pub revision: Option<String>,
    /// Print the answer as one JSON document.
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &StatusArgs, at: &Path, out: &mut dyn Write) -> i32 {
    let answer = ask(args, at);
    let _ = if args.json {
        render::json(out, &answer)
    } else {
        text(out, &answer)
    };
    render::exit(&answer)
}

fn ask(args: &StatusArgs, at: &Path) -> Answer<Status> {
    let history = History::open(at)?;
    let ask = StatusAsk {
        component: args.component.as_ref().map(ComponentName::new),
        revision: match &args.revision {
            Some(text) => Some(history.resolve(text)?),
            None => None,
        },
    };
    status(&history, &Kept::at(&history), &ask)
}

fn text(out: &mut dyn Write, answer: &Answer<Status>) -> io::Result<()> {
    let status = match answer {
        Ok(status) => status,
        Err(not) => return render::not(out, "", not),
    };
    for component in &status.components {
        writeln!(out, "{}", component.component)?;
        for shown in &component.versions {
            let entry = &shown.entry;
            writeln!(
                out,
                "  {} ({}) from {} on {}: index {} in {}",
                entry.version, entry.tag, entry.revision, entry.branch, entry.index, entry.store
            )?;
            match &shown.notes {
                Some(Ok(notes)) => {
                    for line in notes.lines() {
                        writeln!(out, "{}", format!("    {line}").trim_end())?;
                    }
                }
                Some(Err(not)) => render::not(out, "    ", not)?,
                None => {}
            }
        }
    }
    Ok(())
}
