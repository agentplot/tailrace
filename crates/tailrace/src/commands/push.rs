//! `tailrace push [revision]`: the step after a landing, run on the host
//! that ran its checks, that writes to the store the cache entry of each
//! build identity the landing's passing verdicts name, from what the
//! landing's build left on the host; by default for the local branch's
//! head, which on the landing host is the last landing (52, 77, C.3).
//! `tailrace push --left` is the push before a landing, run in the
//! checkout a check job's run left: it writes the cache entry of each
//! build identity that checkout's run left with a pass, taking no
//! revision and asking no landing (135, 139). Each offers, signs and
//! keeps nothing (53, 61, 100).

use std::io::{self, Write};
use std::path::Path;

use clap::Args;
use tailrace_core::cache::Written;
use tailrace_core::{push, push_left, History, LeftAsk, Push, PushAsk, PushedLeft, LOCAL};
use tailrace_seams::{Answer, Branch, BuildId, Not, Source};

use crate::{render, Binding};

#[derive(Debug, Args)]
pub struct PushArgs {
    /// The landed revision whose archives are pushed, any name git reads
    /// for a commit; by default the local branch's head.
    pub revision: Option<String>,
    /// The branch that landed it.
    #[arg(long, default_value = "main")]
    pub branch: String,
    /// Push what this checkout's own run of the build left, before the
    /// landing: no revision is taken and no landing is asked.
    #[arg(long, conflicts_with_all = ["revision", "branch"])]
    pub left: bool,
    /// Print the answer as one JSON document.
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &PushArgs, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    if args.left {
        let answer = ask_left(binding, at);
        let _ = if args.json {
            render::json(out, &answer)
        } else {
            text_left(out, &answer)
        };
        return match &answer {
            Err(not) => render::code(not),
            Ok(pushed) => worst(&pushed.entries),
        };
    }
    let answer = ask(args, binding, at);
    let _ = if args.json {
        render::json(out, &answer)
    } else {
        text(out, &answer)
    };
    match &answer {
        Err(not) => render::code(not),
        Ok(push) => worst(&push.entries),
    }
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

/// The push of what the working tree of the repository at `at` left, the
/// moon cache projection's root, so each entry is packed from the
/// archives the build named; absent naming the working tree where there
/// is none.
fn ask_left(binding: &Answer<Binding>, at: &Path) -> Answer<PushedLeft> {
    let binding = binding.as_ref().map_err(Not::clone)?;
    let history = History::open(at)?;
    let tree = history.workdir().ok_or_else(|| {
        Not::absent(
            &Source::history(LOCAL),
            format!("working tree at {}, whose run is pushed", at.display()),
        )
    })?;
    let ask = LeftAsk {
        at: tree.display().to_string(),
    };
    push_left(&binding.seams, &ask)
}

/// The worst of the entries' refused and unreadable answers, an archive
/// not on the host counting as an answer (54).
fn worst(entries: &[(BuildId, Answer<Written>)]) -> i32 {
    render::worst(entries.iter().map(|(_, entry)| match entry {
        Ok(_) | Err(Not::Absent(_)) => render::OK,
        Err(not) => render::code(not),
    }))
}

fn text(out: &mut dyn Write, answer: &Answer<Push>) -> io::Result<()> {
    let push = match answer {
        Ok(push) => push,
        Err(not) => return render::not(out, "", not),
    };
    if push.entries.is_empty() {
        writeln!(out, "no passing verdict of {} names a build", push.revision)?;
    }
    lines(out, &push.entries)
}

fn text_left(out: &mut dyn Write, answer: &Answer<PushedLeft>) -> io::Result<()> {
    let pushed = match answer {
        Ok(pushed) => pushed,
        Err(not) => return render::not(out, "", not),
    };
    if pushed.entries.is_empty() {
        writeln!(out, "the run in {} left no build that passed", pushed.at)?;
    }
    lines(out, &pushed.entries)
}

/// One line per build identity: pushed with its digest, held already,
/// absent, or why not.
fn lines(out: &mut dyn Write, entries: &[(BuildId, Answer<Written>)]) -> io::Result<()> {
    for (build, entry) in entries {
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
