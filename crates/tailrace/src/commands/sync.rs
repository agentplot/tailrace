//! `tailrace sync [revision]`: the cache entries a landed revision's
//! passing verdicts name, rendered into the checkout it runs in where
//! its tools read them; by default for the newest revision of the
//! branch the checkout descends from (52, 54, C.3). It writes nothing to
//! the store and reads no policy (53).

use std::io::{self, Write};
use std::path::Path;

use clap::Args;
use tailrace_core::{sync, History, SyncAsk, Synced, LOCAL};
use tailrace_seams::{Answer, Branch, Not, Source};

use crate::{render, Binding};

#[derive(Debug, Args)]
pub struct SyncArgs {
    /// The landed revision whose verdicts name the entries, any name git
    /// reads for a commit; by default the newest revision of the branch
    /// the checkout descends from.
    pub revision: Option<String>,
    /// The branch that landed it.
    #[arg(long, default_value = "main")]
    pub branch: String,
    /// Print the answer as one JSON document.
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &SyncArgs, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    let answer = ask(args, binding, at);
    let _ = if args.json {
        render::json(out, &answer)
    } else {
        text(out, &answer)
    };
    exit(&answer)
}

fn ask(args: &SyncArgs, binding: &Answer<Binding>, at: &Path) -> Answer<Synced> {
    let binding = binding.as_ref().map_err(Not::clone)?;
    let history = History::open(at)?;
    let branch = Branch::new(&args.branch);
    let revision = match &args.revision {
        Some(text) => history.resolve(text)?,
        None => history.base(&branch)?,
    };
    let at = history.workdir().ok_or_else(|| {
        Not::absent(
            &Source::history(LOCAL),
            format!("working tree at {}, to render into", at.display()),
        )
    })?;
    let ask = SyncAsk {
        branch,
        revision,
        at: at.display().to_string(),
    };
    sync(&binding.seams, &ask)
}

/// The whole answer's code when it is no answer; otherwise the worst of
/// the entries' refused and unreadable answers, an entry that is gone
/// counting as an answer (54).
fn exit(answer: &Answer<Synced>) -> i32 {
    match answer {
        Err(not) => render::code(not),
        Ok(synced) => render::worst(synced.entries.iter().map(|(_, entry)| match entry {
            Ok(_) | Err(Not::Absent(_)) => render::OK,
            Err(not) => render::code(not),
        })),
    }
}

fn text(out: &mut dyn Write, answer: &Answer<Synced>) -> io::Result<()> {
    let synced = match answer {
        Ok(synced) => synced,
        Err(not) => return render::not(out, "", not),
    };
    if synced.entries.is_empty() {
        writeln!(
            out,
            "no passing verdict of {} names a build",
            synced.revision
        )?;
    }
    for (build, entry) in &synced.entries {
        match entry {
            Ok(filled) => {
                let by: Vec<&str> = filled.rendered.iter().map(|e| e.as_str()).collect();
                writeln!(
                    out,
                    "{build} rendered by {}, {}",
                    by.join(" and "),
                    filled.digest
                )?;
            }
            Err(Not::Absent(a)) => writeln!(out, "{build} absent: {}", a.what)?,
            Err(not) => render::not(out, &format!("{build}: "), not)?,
        }
    }
    Ok(())
}
