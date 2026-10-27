//! `tailrace status`: where each channel the policy declares is and the
//! chain of promotions that put it there, read from the ledger, each
//! promotion with the notes of the version it moved to, the
//! environments, unreadable while no target is read, and the versions
//! this host published, from its own record, each with its notes; every
//! version's notes rendered from the repository's own history, on any
//! machine, one that never published included (31, 33, 36, 56 to 61, 98,
//! 134). It is bound to
//! the landing, the policy and the ledger alone, asks no store, signer or
//! target, and writes nothing (60, rule core.32). It reads the asker's
//! time from the clock and hands it to the core, which reads none.
//!
//! `--page` asks another question: where the repository's own page is.
//! It names the checkout's repository to the host, by its landing's
//! remote's owner and name, finds the host's serving process or starts
//! one where none answers, and answers the page's address (57, 114, 132;
//! design D7, D12).

use std::io::{self, Write};
use std::net::SocketAddr;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Args;
use tailrace_core::{
    overview, overview_unbound, Channel, ChannelStatus, Channels, EnvironmentCounts, History, Kept,
    Logged, Overview, OverviewAsk, RepositoryName, Status,
};
use tailrace_seams::{
    Answer, Branch, ComponentName, Digest, Moves, Not, Promotion, Read, Seam, Source,
};

use crate::bindings;
use crate::commands::serve::{self, host, host::Host};
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
    /// Print the address of the repository's own page on the host's
    /// serving process, starting the process where none answers.
    #[arg(long)]
    pub page: bool,
    /// Print the answer as one JSON document.
    #[arg(long)]
    pub json: bool,
}

/// Status asked over `binding` in the repository at `at`: exit 0
/// whenever an overview is made, each part carrying its own answer, and
/// the answer's own code only where the repository cannot be read at
/// all. With `--page`, the page's address, with its answer's code.
pub fn run(args: &StatusArgs, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    if args.page {
        let answer = Host::of_environment()
            .and_then(|host| page(at, &host, serve::address(), detached(&host)));
        let _ = match (&answer, args.json) {
            (_, true) => render::json(out, &answer),
            (Ok(address), false) => writeln!(out, "{address}"),
            (Err(not), false) => render::not(out, "", not),
        };
        return render::exit(&answer);
    }
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
        Err(not) => overview_unbound(not.clone(), &history, &kept, &ask),
    })
}

/// 114, 132: the address of the page of the repository whose checkout is
/// at `checkout`, on the host's serving process at `address`: the
/// repository named by its landing's remote's owner and name and named
/// to `host`, then the process found at `address`, or `start` run where
/// none answers. Absent where the landing has no remote or the remote no
/// address, and unreadable naming the remote and its address where it
/// ends in no owner and name, writing nothing.
pub fn page(
    checkout: &Path,
    host: &Host,
    address: SocketAddr,
    start: impl FnOnce() -> Answer<()>,
) -> Answer<String> {
    let remote = bindings::landing_remote(&host.config, host.home.as_deref(), checkout)?
        .ok_or_else(|| {
            Not::absent(
                &Source::history(checkout.display()),
                "remote of the landing, which neither [landing] nor the repository names",
            )
        })?;
    let history = History::open(checkout)?;
    let url = history.remote_url(&remote)?.ok_or_else(|| {
        Not::absent(
            &Source::history(&remote),
            format!("address of the remote {remote}"),
        )
    })?;
    let repository = RepositoryName::from_remote(&url).ok_or_else(|| {
        Not::unreadable(
            &Source::history(&remote),
            format!("an owner and a name in the address of the remote {remote}, {url}"),
            None,
        )
    })?;
    let root = history.workdir().unwrap_or_else(|| history.common_dir());
    host::name(host, &repository, &root)?;
    host::find_or_start(address, start)?;
    Ok(format!("http://{address}{}/status", repository.path()))
}

/// `tailrace serve` started from this binary, detached from the asker's
/// terminal and its signals, its working directory the state directory,
/// never a checkout, and not waited for (114).
fn detached(host: &Host) -> impl FnOnce() -> Answer<()> + '_ {
    move || {
        let unstarted = |said: String| {
            Not::unreadable(
                &Source::new(Seam::Host, "serve"),
                "tailrace serve, started from this binary",
                Some(said),
            )
        };
        let binary = std::env::current_exe().map_err(|e| unstarted(e.to_string()))?;
        let mut command = Command::new(binary);
        command
            .arg("serve")
            .current_dir(&host.state)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut command, 0);
        // The process runs on alone; the asker ends without waiting.
        let started = command.spawn().map(drop);
        started.map_err(|e| unstarted(e.to_string()))
    }
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
            landing(out, &channels.landing)?;
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

/// 77: how the landing read the branch's head, under the branch and the
/// revision: nothing where it read itself, and where it read the host's
/// copy, when that was fetched and the remote's words, unchanged, on
/// lines of their own beneath, as a chain read as last read is printed
/// (59; design D6).
fn landing(out: &mut dyn Write, read: &Read) -> io::Result<()> {
    match read {
        Read::Source => Ok(()),
        Read::Kept { at, said } => {
            writeln!(out, "  landing: as last fetched {}", render::utc(*at))?;
            words(out, said)
        }
    }
}

/// Each component's channels in a block of its own, in the order the
/// policy declares them.
fn channels(out: &mut dyn Write, channels: &Channels) -> io::Result<()> {
    let mut components: Vec<(ComponentName, Vec<&ChannelStatus>)> = Vec::new();
    for channel in &channels.channels {
        let component = channel.declared.channel.component_name();
        match components.iter_mut().find(|(c, _)| *c == component) {
            Some((_, of)) => of.push(channel),
            None => components.push((component, vec![channel])),
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

/// One channel's line, then its chain newest first, each promotion with
/// the notes of the version it moved to beneath it, or its own answer
/// with the source's words on a line of their own (58, 59, 134).
fn channel(out: &mut dyn Write, channel: &ChannelStatus, width: usize) -> io::Result<()> {
    write!(out, "  {:<width$}  ", channel.name().as_str())?;
    let bound = match &channel.declared.moves {
        Moves::Bound { to } => format!(", bound to {to}"),
        Moves::EachPublish { prereleases: true } => ", prereleases too".to_string(),
        Moves::EachPublish { prereleases: false } | Moves::Promotion => String::new(),
    };
    match &channel.at {
        Ok(Channel::Empty { read }) => {
            write!(out, "empty{bound}")?;
            if let Read::Kept { at, .. } = read {
                write!(out, ", as last read {}", render::utc(*at))?;
            }
            writeln!(out)?;
            if let Read::Kept { said, .. } = read {
                words(out, said)?;
            }
            Ok(())
        }
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
                logged(out, &format!("{indent}  "), &channel.notes, promotion)?;
            }
            Ok(())
        }
        Err(not) => render::not(out, "", not),
    }
}

/// 33, 105: one promotion of a chain: where it moved the channel from,
/// who asked, the policy revision it was judged under with the claims
/// that satisfied it, when, and its claim and its predecessor's.
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
    let after = promotion
        .predecessor
        .as_ref()
        .map(|p| format!(" after {p}"))
        .unwrap_or_default();
    writeln!(
        out,
        "{indent}{}{from}, asked by {}, under the policy at {} satisfied by {}, {}, claim \
         {claim}{after}",
        promotion.to.version,
        promotion.asked_by,
        promotion.policy.revision,
        satisfied(promotion.policy.claims.iter().map(Digest::to_string)),
        render::utc(promotion.at)
    )
}

/// 134: the notes of the version `promotion` moved its channel to, as
/// `notes` answers them, each line at `indent` with trailing spaces
/// trimmed, as a published version's notes are printed, or why they
/// cannot be read (58; design D4).
fn logged(
    out: &mut dyn Write,
    indent: &str,
    notes: &[Logged],
    promotion: &Promotion,
) -> io::Result<()> {
    let to = &promotion.to;
    let Some(logged) = notes
        .iter()
        .find(|l| l.version == to.version && l.index == to.index)
    else {
        return Ok(());
    };
    match &logged.notes {
        Ok(text) => {
            for line in text.lines() {
                writeln!(out, "{}", format!("{indent}{line}").trim_end())?;
            }
            Ok(())
        }
        Err(not) => render::not(out, indent, not),
    }
}

/// The claims that satisfied a promotion's policy, or `no claim` where
/// it required none.
pub(crate) fn satisfied(claims: impl Iterator<Item = String>) -> String {
    let claims: Vec<String> = claims.collect();
    if claims.is_empty() {
        "no claim".to_string()
    } else {
        claims.join(" and ")
    }
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
        for shown in &component.versions {
            let entry = &shown.entry;
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
