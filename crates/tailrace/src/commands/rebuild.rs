//! `tailrace rebuild [revision]`: the host's record of the versions it
//! published, rebuilt from what the store and git hold, for the
//! components the policy file declares at a landed revision; by default
//! the local branch's head (61 to 63). It writes nothing but the record,
//! and never rewrites or deletes an entry the record holds.

use std::io::{self, Write};
use std::path::Path;

use clap::Args;
use tailrace_core::{rebuild, History, Kept, RebuildAsk, Rebuilt, Written};
use tailrace_seams::{Answer, Branch, Not};

use crate::{render, Binding};

#[derive(Debug, Args)]
pub struct RebuildArgs {
    /// The landed revision whose policy file declares the components, any
    /// name git reads for a commit; by default the local branch's head.
    pub revision: Option<String>,
    /// The branch that landed it.
    #[arg(long, default_value = "main")]
    pub branch: String,
    /// Print the answer as one JSON document.
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &RebuildArgs, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    let answer = ask(args, binding, at);
    let _ = if args.json {
        render::json(out, &answer)
    } else {
        text(out, &answer)
    };
    match &answer {
        Ok(rebuilt) => render::worst(rebuilt.entries.iter().map(|(_, a)| render::exit(a))),
        Err(not) => render::code(not),
    }
}

fn ask(args: &RebuildArgs, binding: &Answer<Binding>, at: &Path) -> Answer<Rebuilt> {
    let binding = binding.as_ref().map_err(Not::clone)?;
    let history = History::open(at)?;
    let branch = Branch::new(&args.branch);
    let revision = match &args.revision {
        Some(text) => history.resolve(text)?,
        None => history.head(&branch)?,
    };
    let ask = RebuildAsk {
        branch,
        revision,
        store: binding.store.clone(),
    };
    rebuild(&binding.seams, &history, &Kept::at(&history), &ask)
}

fn text(out: &mut dyn Write, answer: &Answer<Rebuilt>) -> io::Result<()> {
    let rebuilt = match answer {
        Ok(rebuilt) => rebuilt,
        Err(not) => return render::not(out, "", not),
    };
    if rebuilt.entries.is_empty() {
        writeln!(
            out,
            "the store and git hold no version for {}",
            rebuilt.record
        )?;
    }
    for (entry, written) in &rebuilt.entries {
        let line = format!(
            "{} {} ({}) from {}",
            entry.component, entry.version, entry.tag, entry.revision
        );
        match written {
            Ok(Written::Wrote) => writeln!(out, "{line}: written")?,
            Ok(Written::Held) => writeln!(out, "{line}: held")?,
            Err(not) => {
                writeln!(out, "{line}:")?;
                render::not(out, "  ", not)?;
            }
        }
    }
    Ok(())
}
