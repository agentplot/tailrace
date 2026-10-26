//! `tailrace status`: where each channel the policy declares is and the
//! chain of promotions that put it there, read from the ledger, the
//! environments, unreadable while no target is read, and the versions
//! this host published, from its own record; on any machine, one that
//! never published included (31, 36, 56 to 61, 98). It is bound to the
//! landing, the policy and the ledger alone, asks no store, signer or
//! target, and writes nothing (60, rule core.32). It reads the asker's
//! time from the clock and hands it to the core, which reads none.

use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Args;
use tailrace_core::{
    overview, overview_unbound, Channel, ChannelStatus, Channels, EnvironmentCounts, History, Kept,
    Overview, OverviewAsk, Status,
};
use tailrace_seams::{Answer, Branch, ComponentName, Digest, Promotion, Read};

use crate::{render, Binding};

#[derive(Debug, Args)]
pub struct StatusArgs {
    /// The branch whose landed head holds the policy that declares the
    /// channels.
    #[arg(long, default_value = "main")]
    pub branch: String,
    /// Only the channels and versions of this component.
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

/// Status asked over `binding` in the repository at `at`: exit 0
/// whenever an overview is made, each part carrying its own answer, and
/// the answer's own code only where the repository cannot be read at
/// all.
pub fn run(args: &StatusArgs, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    let answer = ask(args, binding, at);
    let _ = match (&answer, args.json) {
        (_, true) => render::json(out, &answer),
        (Ok(overview), false) => text(out, overview),
        (Err(not), false) => render::not(out, "", not),
    };
    render::exit(&answer)
}

/// The moment asked, in seconds since the Unix epoch.
pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn ask(args: &StatusArgs, binding: &Answer<Binding>, at: &Path) -> Answer<Overview> {
    overview_in(
        binding,
        at,
        &args.branch,
        args.component.as_deref(),
        args.revision.as_deref(),
    )
}

/// 36, 60: the whole of status in the repository at `at` over `binding`,
/// or over none where the binding failed, for `branch`, of `component`
/// and `revision` where they are named, as of now: what the command line
/// prints and the serving process serves (57, rule core.31).
pub(crate) fn overview_in(
    binding: &Answer<Binding>,
    at: &Path,
    branch: &str,
    component: Option<&str>,
    revision: Option<&str>,
) -> Answer<Overview> {
    let history = History::open(at)?;
    let ask = OverviewAsk {
        branch: Branch::new(branch),
        component: component.map(ComponentName::new),
        revision: match revision {
            Some(text) => Some(history.resolve(text)?),
            None => None,
        },
        at: now(),
    };
    let kept = Kept::at(&history);
    Ok(match binding {
        Ok(binding) => overview(&binding.seams, &history, &kept, &ask),
        Err(not) => overview_unbound(not.clone(), &kept, &ask),
    })
}

fn text(out: &mut dyn Write, overview: &Overview) -> io::Result<()> {
    let asked = render::utc(overview.asked_at);
    match &overview.channels {
        Ok(channels) => {
            writeln!(
                out,
                "{} at {}, as of {asked}",
                overview.branch, channels.revision
            )?;
            self::channels(out, channels)?;
        }
        Err(not) => {
            writeln!(out, "{}, as of {asked}", overview.branch)?;
            writeln!(out)?;
            render::not(out, "", not)?;
        }
    }
    writeln!(out)?;
    writeln!(out, "environments")?;
    environments(out, &overview.environments)?;
    writeln!(out)?;
    writeln!(out, "published from this host")?;
    published(out, &overview.published)
}

/// Each component's channels in a block of its own, in the order the
/// policy declares them.
fn channels(out: &mut dyn Write, channels: &Channels) -> io::Result<()> {
    let mut components: Vec<(&ComponentName, Vec<&ChannelStatus>)> = Vec::new();
    for channel in &channels.channels {
        match components
            .iter_mut()
            .find(|(c, _)| **c == channel.declared.component)
        {
            Some((_, of)) => of.push(channel),
            None => components.push((&channel.declared.component, vec![channel])),
        }
    }
    for (component, of) in components {
        writeln!(out)?;
        writeln!(out, "{component}")?;
        let width = of
            .iter()
            .map(|c| c.name().as_str().len())
            .max()
            .unwrap_or(0);
        for channel in of {
            self::channel(out, channel, width)?;
        }
    }
    Ok(())
}

/// One channel's line, then its chain newest first, or its own answer
/// with the source's words on a line of their own (58, 59).
fn channel(out: &mut dyn Write, channel: &ChannelStatus, width: usize) -> io::Result<()> {
    write!(out, "  {:<width$}  ", channel.name().as_str())?;
    let bound = channel
        .declared
        .bound_to
        .as_ref()
        .map(|b| format!(", bound to {b}"))
        .unwrap_or_default();
    match &channel.at {
        Ok(Channel::Empty) => writeln!(out, "empty{bound}"),
        Ok(Channel::At { head, chain, read }) => {
            write!(out, "{} ({}){bound}", head.to.version, head.to.index)?;
            if let Read::Kept { at, .. } = read {
                write!(out, ", as last read {}", render::utc(*at))?;
            }
            writeln!(out)?;
            if let Read::Kept { said, .. } = read {
                words(out, said)?;
            }
            let indent = " ".repeat(width + 6);
            for (claim, promotion) in chain.iter().rev() {
                promoted(out, &indent, claim, promotion)?;
            }
            Ok(())
        }
        Err(not) => render::not(out, "", not),
    }
}

/// 33: one promotion of a chain: where it moved the channel from, who
/// asked, when, and its claim.
fn promoted(
    out: &mut dyn Write,
    indent: &str,
    claim: &Digest,
    promotion: &Promotion,
) -> io::Result<()> {
    let from = promotion.from.as_ref().map_or_else(
        || ", the first".to_string(),
        |f| format!(" from {}", f.version),
    );
    writeln!(
        out,
        "{indent}{}{from}, asked by {}, {}, claim {claim}",
        promotion.to.version,
        promotion.asked_by,
        render::utc(promotion.at)
    )
}

/// A source's own words, unchanged, on a line of their own (59).
fn words(out: &mut dyn Write, said: &str) -> io::Result<()> {
    out.write_all(said.as_bytes())?;
    if !said.ends_with('\n') {
        writeln!(out)?;
    }
    Ok(())
}

/// 98: how many environments run what they should, are behind, wait or
/// cannot be read, or the part's own answer.
fn environments(out: &mut dyn Write, answer: &Answer<EnvironmentCounts>) -> io::Result<()> {
    match answer {
        Ok(counts) => writeln!(
            out,
            "  {} running what they should, {} behind, {} waiting, {} unreadable",
            counts.converged, counts.behind, counts.waiting, counts.unreadable
        ),
        Err(not) => render::not(out, "  ", not),
    }
}

fn published(out: &mut dyn Write, answer: &Answer<Status>) -> io::Result<()> {
    let status = match answer {
        Ok(status) => status,
        Err(not) => return render::not(out, "  ", not),
    };
    for component in &status.components {
        for entry in &component.versions {
            writeln!(
                out,
                "  {} {} ({}) from {} on {}: index {} in {}",
                component.component,
                entry.version,
                entry.tag,
                entry.revision,
                entry.branch,
                entry.index,
                entry.store
            )?;
        }
    }
    Ok(())
}
