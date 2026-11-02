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
//! and, per pin and per version judged for a range, held with its index,
//! refused with the claim found wanting and the signers checked, and a
//! count of the pins the store does not name; then each range, what
//! required it and the version held for it or that none within it was
//! admitted (137); a layout that cannot be refreshed says what it holds
//! and when it was rendered, beside what could not be read; then where
//! each served layout is served and the process serving it. The exit is
//! the worst among the entries and pins not absent, a version judged for
//! a range counting only where its range holds none, since a range
//! holding an older admitted version builds (99), a layout that cannot
//! be refreshed counting as its cause, and a served layout's address and
//! the serving process each counting as a layout's answer (57; design D9;
//! cargos-registry design D7).

use std::io::{self, Write};
use std::net::SocketAddr;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Args;
use tailrace_core::render::{Kept, Refreshed};
use tailrace_core::{
    answer_json, sync_checkout, CheckoutAsk, CheckoutSynced, History, Layouts, PinAnswer, PolicyOf,
    RangeAnswer, Refresh, RepositoryName, Under, LOCAL,
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
        Ok(Refresh::Refreshed(refreshed)) => layout_code(refreshed),
    });
    let served = synced.served.values().map(render::exit);
    render::worst(entries.chain(layouts).chain(served))
}

/// A layout's code: the worst of its pins', of the versions judged for a
/// range whose range holds none, and of the ranges the projection would
/// not read (99; cargos-registry design D7).
fn layout_code(layout: &Refreshed) -> i32 {
    let holds_none = |pin: &PinAnswer| {
        layout
            .ranges
            .iter()
            .any(|r| r.range.name == pin.pin.name && matches!(r.answer, Ok(None)))
    };
    let pins = layout.pins.iter().map(pin_code);
    let asked = layout.asked.iter().filter(|a| holds_none(a)).map(pin_code);
    let ranges = layout.ranges.iter().map(|r| match &r.answer {
        Ok(_) => render::OK,
        Err(not) => render::code(not),
    });
    render::worst(pins.chain(asked).chain(ranges))
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
    for pin in layout.pins.iter().chain(&layout.asked) {
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
    for range in &layout.ranges {
        ranged(out, range)?;
    }
    Ok(())
}

/// One range's line: its name and range, what required it, and the
/// version held for it or that none within it was admitted (137).
fn ranged(out: &mut dyn Write, range: &RangeAnswer) -> io::Result<()> {
    let by = match &range.by {
        None => "the checkout's manifest".to_string(),
        Some(by) => format!("{} {}", by.name, by.version),
    };
    let line = format!(
        "  range {} {} from {by}",
        range.range.name, range.range.range
    );
    match &range.answer {
        Ok(Some(version)) => writeln!(out, "{line}: {version} held"),
        Ok(None) => writeln!(out, "{line}: none within it admitted"),
        Err(not) => render::not(out, &format!("{line}: "), not),
    }
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
    use serde_json::json;
    use tailrace_core::{artifact_name, Own, RenderAsk};
    use tailrace_fakes::{FakeSigner, Fakes};
    use tailrace_seams::{
        Artifact, ArtifactMeta, AskedOf, Claim, ClaimKind, ComponentName, EcosystemName, IndexMeta,
        Manifest, NamedSigner, Pin, Range, RequiredClaim, Requirement, Revision, Signer, SignerId,
        Store, Version,
    };
    use tempfile::TempDir;

    use super::*;

    const GATE: &str = "gate";
    const LANDED: &str = "4f2a9c1";

    fn eco() -> Ecosystem {
        Ecosystem::new("eco")
    }

    /// Acceptance by the gate's signer.
    fn acceptance() -> RequiredClaim {
        RequiredClaim {
            kind: ClaimKind::Acceptance,
            check: None,
            signers: vec![NamedSigner {
                id: SignerId::new(GATE),
                verified_by: format!("{GATE}'s key"),
                kinds: vec![ClaimKind::Acceptance],
                checks: vec![],
            }],
            asked_of: AskedOf::Every,
        }
    }

    /// `site` at `version` published to `fakes`' store and named, its
    /// index accepted by the gate where `accepted`.
    fn publish(fakes: &Fakes, version: &str, accepted: bool) {
        let (name, version) = (EcosystemName::new("site"), Version::new(version));
        let meta = ArtifactMeta::new(eco(), name.clone(), version.clone()).unwrap();
        let store = &fakes.store;
        let bytes = format!("site {version}");
        let layer = store.put_bytes(bytes.as_bytes()).unwrap().digest;
        let digest = store
            .put_artifact(&Manifest::Artifact {
                meta: meta.clone(),
                layers: vec![layer],
            })
            .unwrap()
            .digest;
        let index = IndexMeta::new(
            ComponentName::new("site"),
            version.clone(),
            vec![Artifact { digest, meta }],
        )
        .unwrap();
        let index = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
        store
            .name(&artifact_name(&eco(), &name, &version), &index, None)
            .unwrap();
        if accepted {
            let gate = SignerId::new(GATE);
            let signer = FakeSigner::new();
            signer.hold(&gate);
            let claim = Claim {
                subject: index.clone(),
                kind: ClaimKind::Acceptance,
                revision: Revision::new("p1b11s4"),
                build: None,
                body: json!(null),
            };
            store
                .attach(&index, &signer.sign(&claim, &gate).unwrap())
                .unwrap();
        }
    }

    /// A sync over the fakes of a checkout whose manifest declares `site`
    /// by the range `1`, the store naming each of `published` and the
    /// consumer's policy requiring acceptance by the gate, those accepted
    /// admitted: its answer, its text and its exit.
    fn synced(published: &[(&str, bool)]) -> (Answer<CheckoutSynced>, String, i32) {
        let mut fakes = Fakes::new();
        fakes.projection(&eco());
        for (version, accepted) in published {
            publish(&fakes, version, *accepted);
        }
        let dir = TempDir::new().unwrap();
        let refreshed = tailrace_core::render::render(
            &fakes.seams(),
            &Layouts::open(dir.path().join("record")),
            &RenderAsk {
                ecosystem: eco(),
                pins: vec![Pin::Range(Requirement {
                    name: EcosystemName::new("site"),
                    range: Range::new("1"),
                })],
                own: Some(Own {
                    revision: Revision::new(LANDED),
                    claims: vec![acceptance()],
                    given: false,
                    signers: vec![],
                }),
                trusted: vec![],
                tree: dir.path().join("checkout"),
                when: 1_767_225_600,
            },
        )
        .unwrap();
        let answer = Ok(CheckoutSynced {
            branch: Branch::new("main"),
            revision: Revision::new(LANDED),
            entries: vec![],
            layouts: vec![(eco(), Ok(Refresh::Refreshed(refreshed)))],
            served: std::collections::BTreeMap::new(),
        });
        let mut printed = Vec::new();
        text(&mut printed, &answer).unwrap();
        let code = exit(&answer);
        (answer, String::from_utf8(printed).unwrap(), code)
    }

    #[test]
    fn a_range_holding_an_older_admitted_version_beside_a_refused_newer_one_exits_0() {
        let (answer, printed, code) = synced(&[("1.0.0", true), ("1.1.0", true), ("1.2.0", false)]);
        let lines: Vec<&str> = printed.lines().collect();
        assert!(
            lines.iter().any(|l| l.starts_with("  site 1.2.0: ")),
            "{printed}"
        );
        assert!(
            lines
                .iter()
                .any(|l| l.starts_with("  site 1.1.0 held, sha256:")),
            "{printed}"
        );
        assert_eq!(
            lines.last(),
            Some(&"  range site 1 from the checkout's manifest: 1.1.0 held"),
            "{printed}"
        );
        assert_eq!(code, render::OK, "{printed}");
        let mut json = Vec::new();
        render::json(&mut json, &answer).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&json).unwrap();
        let layout = &value["value"]["layouts"][0]["value"]["refreshed"];
        assert_eq!(layout["asked"][0]["version"], "1.2.0", "{layout}");
        assert_eq!(layout["asked"][0]["answer"], "refused", "{layout}");
        assert_eq!(
            (
                &layout["ranges"][0]["name"],
                &layout["ranges"][0]["range"],
                &layout["ranges"][0]["by"],
                &layout["ranges"][0]["value"]
            ),
            (&json!("site"), &json!("1"), &json!(null), &json!("1.1.0")),
            "{layout}"
        );
    }

    #[test]
    fn a_range_holding_none_counts_the_refused_version_against_the_exit() {
        let (_, printed, code) = synced(&[("1.2.0", false)]);
        assert!(
            printed.lines().any(
                |l| l == "  range site 1 from the checkout's manifest: none within it admitted"
            ),
            "{printed}"
        );
        assert_eq!(code, render::REFUSED, "{printed}");
    }

    #[test]
    fn a_time_is_written_as_a_utc_date() {
        assert_eq!(utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(utc(1_767_225_600), "2026-01-01T00:00:00Z");
        assert_eq!(utc(1_709_210_096), "2024-02-29T12:34:56Z");
    }
}
