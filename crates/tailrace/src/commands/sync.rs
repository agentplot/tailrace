//! `tailrace sync [revision]`: the cache entries a landed revision's
//! passing verdicts name, rendered into the checkout it runs in where
//! its tools read them, and then each layout of a version the
//! checkout's lockfiles pin, rendered at `.tailrace/layouts/<ecosystem>/`
//! holding only what the policy for layouts at that revision admits; by
//! default for the newest revision of the branch the checkout descends
//! from (47 to 54, 99, C.3). Filling the cache reads no policy (53); the
//! layouts read the consumer's layout line, or, where it writes none, the
//! publisher's, else the given, each claim read through the signers the
//! consumer's file names for it, else through the host's own signer and
//! those its bindings trust (113, C.10). It writes nothing to the store.
//! Where a layout's projection serves it, the layout names the address
//! its clients read it at, the core's name and port with the checkout's
//! repository in the path, and sync names the repository to the host and
//! finds the host's serving process or starts it, as `tailrace status
//! --page` does (114, 132; design D10). Text names each layout's place
//! and, per pin, held with its index, refused with the claim found
//! wanting and the signers checked, and a count of the pins the store
//! does not name; a layout that cannot be refreshed says what it holds
//! and when it was rendered, beside what could not be read; then where
//! each served layout is served and the process serving it. The exit is
//! the worst among the entries and pins not absent, a layout that cannot
//! be refreshed counting as its cause, a served layout's address and the
//! serving process each counting as a layout's answer (57; design D9).

use std::io::{self, Write};
use std::net::SocketAddr;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Args;
use tailrace_core::render::{Kept, Refreshed};
use tailrace_core::{
    answer_json, sync_checkout, CheckoutAsk, CheckoutSynced, History, Layouts, PinAnswer, PolicyOf,
    Refresh, RepositoryName, Under, LOCAL,
};
use tailrace_seams::{Answer, Branch, Ecosystem, Not, Source};

use crate::commands::serve::{self, host, host::Host, host::Serving};
use crate::commands::status::detached;
use crate::{render, Binding};

#[derive(Debug, Args)]
pub struct SyncArgs {
    /// The landed revision whose verdicts name the entries and whose
    /// policy judges the layouts, any name git reads for a commit; by
    /// default the newest revision of the branch the checkout descends
    /// from.
    pub revision: Option<String>,
    /// The branch that landed it.
    #[arg(long, default_value = "main")]
    pub branch: String,
    /// Print the answer as one JSON document.
    #[arg(long)]
    pub json: bool,
}

/// Sync asked over `binding` in the repository at `at`, on the host the
/// environment places, its serving process at the core's address, started
/// from this binary where none answers.
pub fn run(args: &SyncArgs, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    let host = Host::of_environment();
    let start = || match &host {
        Ok(host) => detached(host)(),
        Err(not) => Err(not.clone()),
    };
    run_on(args, binding, at, out, &host, serve::address(), start)
}

/// Sync asked over `binding` in the repository at `at`, on `host`, its
/// serving process found at `address` or started by `start` where a
/// layout is served and none answers: what the command line prints, with
/// its exit code. A test gives a host, an address and a start of its own,
/// so it reads no file of the host running it and starts nothing real.
pub fn run_on(
    args: &SyncArgs,
    binding: &Answer<Binding>,
    at: &Path,
    out: &mut dyn Write,
    host: &Answer<Host>,
    address: SocketAddr,
    start: impl FnOnce() -> Answer<()>,
) -> i32 {
    let (answer, repository) = ask(args, binding, at, host);
    let serving = answer
        .as_ref()
        .ok()
        .and_then(|synced| serving(synced, &repository, host, at, address, start));
    let _ = if args.json {
        let mut value = answer_json(&answer);
        if let Some(serving) = &serving {
            value["serving"] = answer_json(serving);
        }
        serde_json::to_writer_pretty(&mut *out, &value)
            .map_err(io::Error::from)
            .and_then(|()| writeln!(out))
    } else {
        text(out, &answer).and_then(|()| match &serving {
            Some(serving) => serving_text(out, address, serving),
            None => Ok(()),
        })
    };
    render::worst([
        exit(&answer),
        serving.as_ref().map_or(render::OK, render::exit),
    ])
}

/// 114, 132: where any layout `synced` answers is served at an address,
/// the checkout's repository named to `host` and the host's serving
/// process found at `address`, or `start` run where none answers, as
/// `tailrace status --page` does; none where no layout is served, so
/// nothing is named or started.
fn serving(
    synced: &CheckoutSynced,
    repository: &Answer<RepositoryName>,
    host: &Answer<Host>,
    at: &Path,
    address: SocketAddr,
    start: impl FnOnce() -> Answer<()>,
) -> Option<Answer<Serving>> {
    if !synced.served.values().any(Result::is_ok) {
        return None;
    }
    let found = || {
        let host = host.as_ref().map_err(Not::clone)?;
        let repository = repository.as_ref().map_err(Not::clone)?;
        let history = History::open(at)?;
        let root = history.workdir().unwrap_or_else(|| history.common_dir());
        host::name(host, repository, &root)?;
        host::find_or_start(address, start)
    };
    Some(found())
}

/// Seconds since the Unix epoch by this host's clock, the time a layout
/// is rendered at; the core reads no clock.
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

/// The core's answer of the checkout at `at`, and the repository the
/// checkout is of, by its landing's remote as `host`'s bindings file
/// names it.
fn ask(
    args: &SyncArgs,
    binding: &Answer<Binding>,
    at: &Path,
    host: &Answer<Host>,
) -> (Answer<CheckoutSynced>, Answer<RepositoryName>) {
    let repository = host
        .as_ref()
        .map_err(Not::clone)
        .and_then(|host| host::repository_of(host, at));
    let synced = synced(args, binding, at, &repository);
    (synced, repository)
}

fn synced(
    args: &SyncArgs,
    binding: &Answer<Binding>,
    at: &Path,
    repository: &Answer<RepositoryName>,
) -> Answer<CheckoutSynced> {
    let binding = binding.as_ref().map_err(Not::clone)?;
    let history = History::open(at)?;
    let branch = Branch::new(&args.branch);
    let revision = match &args.revision {
        Some(text) => history.resolve(text)?,
        None => history.base(&branch)?,
    };
    let tree = history.workdir().ok_or_else(|| {
        Not::absent(
            &Source::history(LOCAL),
            format!("working tree at {}, to render into", at.display()),
        )
    })?;
    let ask = CheckoutAsk {
        branch,
        revision,
        tree,
        when: now(),
        trusted: binding.trust.clone(),
        repository: repository.clone(),
    };
    sync_checkout(&binding.seams, &Layouts::at(&history), &ask)
}

/// A pin's code: an answer, or a pin the store does not name, counts as
/// an answer (51).
fn pin_code(pin: &PinAnswer) -> i32 {
    match &pin.answer {
        Ok(_) | Err(Not::Absent(_)) => render::OK,
        Err(not) => render::code(not),
    }
}

/// The whole answer's code when it is no answer; otherwise the worst of
/// the entries' and the pins' refused and unreadable answers, an entry
/// gone or a pin the store does not name counting as an answer (51, 54),
/// and a layout that cannot be refreshed as what stopped it.
fn exit(answer: &Answer<CheckoutSynced>) -> i32 {
    let synced = match answer {
        Err(not) => return render::code(not),
        Ok(synced) => synced,
    };
    let entries = synced.entries.iter().map(|(_, entry)| match entry {
        Ok(_) | Err(Not::Absent(_)) => render::OK,
        Err(not) => render::code(not),
    });
    let layouts = synced.layouts.iter().map(|(_, layout)| match layout {
        Err(not) => render::code(not),
        Ok(Refresh::Kept(kept)) => render::code(&kept.because),
        Ok(Refresh::Refreshed(refreshed)) => render::worst(refreshed.pins.iter().map(pin_code)),
    });
    let served = synced.served.values().map(render::exit);
    render::worst(entries.chain(layouts).chain(served))
}

/// `seconds` since the Unix epoch as a UTC date and time, as a person
/// reads it.
fn utc(seconds: u64) -> String {
    let (days, of_day) = (seconds / 86_400, seconds % 86_400);
    // The civil date of a day count, after Howard Hinnant's
    // `civil_from_days`, for days on or after 1970-01-01.
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + u64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        of_day / 3_600,
        of_day % 3_600 / 60,
        of_day % 60
    )
}

/// The policy a layout was rendered under, as a person reads it.
fn under(policy: &Under) -> String {
    match policy {
        Under::Own { revision } => format!("under its own policy at {revision}"),
        Under::Publisher => "under each version's publisher's policy, else the given".into(),
    }
}

/// The policy that judged one version, where it is not the layout's own.
fn judged_by(policy: &Option<PolicyOf>) -> String {
    match policy {
        Some(PolicyOf::Publisher { revision }) => {
            format!(" under its publisher's policy at {revision}")
        }
        Some(PolicyOf::Given {
            revision: Some(revision),
        }) => format!(" under the given policy at {revision}"),
        Some(PolicyOf::Given { revision: None }) => {
            " under the given policy, through the signers the host trusts".into()
        }
        _ => String::new(),
    }
}

fn refreshed(out: &mut dyn Write, ecosystem: &Ecosystem, layout: &Refreshed) -> io::Result<()> {
    writeln!(
        out,
        "layout {ecosystem} at {}, rendered {}",
        layout.at,
        under(&layout.policy)
    )?;
    let mut absent = 0;
    for pin in &layout.pins {
        let (name, version) = (&pin.pin.name, &pin.pin.version);
        match &pin.answer {
            Ok(held) => writeln!(
                out,
                "  {name} {version} held, {}{}",
                held.index,
                judged_by(&pin.policy)
            )?,
            Err(Not::Absent(_)) => absent += 1,
            Err(not) => render::not(out, &format!("  {name} {version}: "), not)?,
        }
    }
    if absent > 0 {
        let pins = if absent == 1 { "pin" } else { "pins" };
        writeln!(out, "  {absent} {pins} the store does not name")?;
    }
    Ok(())
}

fn kept(out: &mut dyn Write, ecosystem: &Ecosystem, layout: &Kept) -> io::Result<()> {
    match (layout.rendered_at, &layout.policy) {
        (Some(at), Some(policy)) => writeln!(
            out,
            "layout {ecosystem} at {} left as it was, rendered {} {}",
            layout.at,
            utc(at),
            under(policy)
        )?,
        _ => writeln!(
            out,
            "layout {ecosystem} at {} holds nothing and was never rendered",
            layout.at
        )?,
    }
    for held in &layout.holds {
        writeln!(out, "  {} {} held, {}", held.name, held.version, held.index)?;
    }
    render::not(out, "  ", &layout.because)
}

fn text(out: &mut dyn Write, answer: &Answer<CheckoutSynced>) -> io::Result<()> {
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
    for (ecosystem, layout) in &synced.layouts {
        match layout {
            Ok(Refresh::Refreshed(layout)) => refreshed(out, ecosystem, layout)?,
            Ok(Refresh::Kept(layout)) => kept(out, ecosystem, layout)?,
            Err(not) => render::not(out, &format!("layout {ecosystem}: "), not)?,
        }
        match synced.served(ecosystem) {
            Some(Ok(address)) => writeln!(out, "  served at {address}")?,
            Some(Err(not)) => render::not(out, "  not served: ", not)?,
            None => {}
        }
    }
    Ok(())
}

/// The host's serving process as sync found or started it, or why it
/// could not.
fn serving_text(
    out: &mut dyn Write,
    address: SocketAddr,
    serving: &Answer<Serving>,
) -> io::Result<()> {
    match serving {
        Ok(serving) => writeln!(
            out,
            "the host's tailrace serve answers at http://{address}, process {}",
            serving.pid
        ),
        Err(not) => render::not(out, "serving: ", not),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_time_is_written_as_a_utc_date() {
        assert_eq!(utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(utc(1_767_225_600), "2026-01-01T00:00:00Z");
        assert_eq!(utc(1_709_210_096), "2024-02-29T12:34:56Z");
    }
}
