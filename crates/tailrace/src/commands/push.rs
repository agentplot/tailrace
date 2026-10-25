//! `tailrace push [revision]`: the step after a landing, run on the host
//! that ran its checks, that writes to the store the cache entry of each
//! build identity the landing's passing verdicts name, from what the
//! landing's build left on the host; by default for the local branch's
//! head, which on the landing host is the last landing (52, 77, C.3). It
//! offers, signs and keeps nothing (53, 61, 100).

use std::io::{self, Write};
use std::path::Path;

use clap::Args;
use tailrace_core::{push, History, Push, PushAsk};
use tailrace_seams::{Answer, Branch, Not};

use crate::{render, Binding};

#[derive(Debug, Args)]
pub struct PushArgs {
    /// The landed revision whose archives are pushed, any name git reads
    /// for a commit; by default the local branch's head.
    pub revision: Option<String>,
    /// The branch that landed it.
    #[arg(long, default_value = "main")]
    pub branch: String,
    /// Print the answer as one JSON document.
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &PushArgs, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    let answer = ask(args, binding, at);
    let _ = if args.json {
        render::json(out, &answer)
    } else {
        text(out, &answer)
    };
    exit(&answer)
}

fn ask(args: &PushArgs, binding: &Answer<Binding>, at: &Path) -> Answer<Push> {
    let binding = binding.as_ref().map_err(Not::clone)?;
    let history = History::open(at)?;
    let branch = Branch::new(&args.branch);
    let revision = match &args.revision {
        Some(text) => history.resolve(text)?,
        None => history.head(&branch)?,
    };
    push(&binding.seams, &PushAsk { branch, revision })
}

/// The whole answer's code when it is no answer; otherwise the worst of
/// the entries' refused and unreadable answers, an archive not on the
/// host counting as an answer (54).
fn exit(answer: &Answer<Push>) -> i32 {
    match answer {
        Err(not) => render::code(not),
        Ok(push) => render::worst(push.entries.iter().map(|(_, entry)| match entry {
            Ok(_) | Err(Not::Absent(_)) => render::OK,
            Err(not) => render::code(not),
        })),
    }
}

fn text(out: &mut dyn Write, answer: &Answer<Push>) -> io::Result<()> {
    let push = match answer {
        Ok(push) => push,
        Err(not) => return render::not(out, "", not),
    };
    if push.entries.is_empty() {
        writeln!(out, "no passing verdict of {} names a build", push.revision)?;
    }
    for (build, entry) in &push.entries {
        match entry {
            Ok(written) => match &written.held_at {
                None => writeln!(out, "{build} pushed, {}", written.entry.digest)?,
                Some(at) => writeln!(out, "{build} held already at {at}")?,
            },
            Err(Not::Absent(a)) => writeln!(out, "{build} absent: {}", a.what)?,
            Err(not) => render::not(out, &format!("{build}: "), not)?,
        }
    }
    Ok(())
}
