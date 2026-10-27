//! `tailrace publish <revision>`: a landed revision offered as a version
//! of each component it publishes (14 to 22). It takes no version: the
//! core derives it (21, rule core.13).
//!
//! After the core's publish has answered, the command tells the host's
//! gate of each version it offered where the bindings file names this
//! host's publish as what tells it, or names no `[gate]` (118), in the
//! same run, and prints the gate's answer beside the version's; where
//! the file names a job, it says so and asks the gate nothing (95, 108,
//! 130; design D2, D5). The core's publish reaches nothing of the gate:
//! this step, the host's binding of what tells it, calls one and then
//! the other. It reads the moment of the gate's act from the clock and
//! hands it to the core, which reads none.

use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Args;
use serde_json::{json, Value};
use tailrace_core::{
    accept, answer_json, publish, AcceptAsk, Ask, History, Judged, Kept, Moved, Publish, Published,
    ToldBy, WhyWanting,
};
use tailrace_seams::{Answer, Branch, ComponentName, Not, RequiredClaim};

use crate::bindings::Teller;
use crate::{render, unsigned, Binding};

#[derive(Debug, Args)]
pub struct PublishArgs {
    /// The revision to publish, any name git reads for a commit.
    pub revision: String,
    /// The branch that landed it.
    #[arg(long, default_value = "main")]
    pub branch: String,
    /// The one component to publish; by default every one the landing
    /// touched.
    #[arg(long)]
    pub component: Option<String>,
    /// Print the answer as one JSON document.
    #[arg(long)]
    pub json: bool,
}

/// The core's publish answer and, for each component in its order, what
/// tells the gate of the version it offered, none where it offered none.
struct Offered {
    publish: Publish,
    gate: Vec<Option<Gate>>,
}

/// What became of telling the gate of one version a publish offered
/// (design D5).
#[derive(Clone, Debug, PartialEq)]
enum Gate {
    /// This host's publish told its own gate, and this is its answer.
    Told(Answer<Judged>),
    /// A job outside tailrace tells the gate, in the file's words; the
    /// gate was not asked.
    Job(String),
}

impl Gate {
    /// 57: the version's `gate` member, in the core's one form.
    fn json(&self) -> Value {
        match self {
            Gate::Told(answer) => {
                let mut value = answer_json(answer);
                value["told_by"] = json!("publish");
                value
            }
            Gate::Job(words) => json!({ "told_by": "job", "job": words }),
        }
    }

    /// The exit code the gate's part adds: a refusal or a channel held
    /// counts as refused, and a channel's own answer by its kind; a job
    /// adds none, the publish having done what it was asked.
    fn code(&self) -> i32 {
        match self {
            Gate::Told(Err(not)) => render::code(not),
            Gate::Told(Ok(Judged::Refused { .. })) => render::REFUSED,
            Gate::Told(Ok(Judged::Accepted { channels, .. })) => {
                render::worst(channels.iter().map(|(_, moved)| match moved {
                    Ok(Moved::Held { .. }) => render::REFUSED,
                    other => render::exit(other),
                }))
            }
            Gate::Job(_) => render::OK,
        }
    }
}

pub fn run(args: &PublishArgs, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    let answer = ask(args, binding, at);
    let _ = if args.json {
        json(out, &answer)
    } else {
        text(out, &answer)
    };
    match &answer {
        Ok(offered) => render::worst(
            offered
                .publish
                .components
                .iter()
                .map(|(_, a)| render::exit(a))
                .chain(offered.gate.iter().flatten().map(Gate::code)),
        ),
        Err(not) => render::code(not),
    }
}

fn ask(args: &PublishArgs, binding: &Answer<Binding>, at: &Path) -> Answer<Offered> {
    let binding = binding.as_ref().map_err(Not::clone)?;
    let signer = binding.signer.clone().ok_or_else(unsigned)?;
    let history = History::open(at)?;
    let ask = Ask {
        branch: Branch::new(&args.branch),
        revision: history.resolve(&args.revision)?,
        component: args.component.as_ref().map(ComponentName::new),
        signer,
        store: binding.store.clone(),
    };
    let publish = publish(&binding.seams, &history, &Kept::at(&history), &ask)?;
    // Only once the publish has answered: the gate, told of each version
    // it offered, where this host's publish is what tells it (130).
    let gate = publish
        .components
        .iter()
        .map(|(component, answer)| {
            let Ok(Published::Version { version, index, .. }) = answer else {
                return None;
            };
            Some(match &binding.gate {
                Teller::Publish => Gate::Told(accept(
                    &binding.seams,
                    &history,
                    &AcceptAsk {
                        branch: ask.branch.clone(),
                        component: Some(component.clone()),
                        version: version.clone(),
                        told: ToldBy::Publish {
                            index: index.clone(),
                        },
                        gate: ask.signer.clone(),
                        when: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map_or(0, |d| d.as_secs()),
                    },
                )),
                Teller::Job(words) => Gate::Job(words.clone()),
            })
        })
        .collect();
    Ok(Offered { publish, gate })
}

/// The answer as one JSON document: the publish's, in the core's one
/// form, each offered version's entry with its `gate` member (57).
fn json(out: &mut dyn Write, answer: &Answer<Offered>) -> io::Result<()> {
    let value = match answer {
        Ok(offered) => {
            let mut value = answer_json::<&Publish>(&Ok(&offered.publish));
            for (i, gate) in offered.gate.iter().enumerate() {
                if let Some(gate) = gate {
                    value["value"]["components"][i]["gate"] = gate.json();
                }
            }
            value
        }
        Err(not) => answer_json::<()>(&Err(not.clone())),
    };
    serde_json::to_writer_pretty(&mut *out, &value)?;
    writeln!(out)
}

/// A required claim as a person names it: its kind and its check.
fn wanting(wanting: &RequiredClaim, why: &WhyWanting) -> String {
    let claim = match &wanting.check {
        Some(check) => format!("{} of {check}", wanting.kind),
        None => wanting.kind.to_string(),
    };
    let why = match why {
        WhyWanting::Missing => "no claim attached counts for it".to_string(),
        WhyWanting::Expired { signer } => format!("{signer}'s identity has expired"),
        WhyWanting::OfferedByGate { signer } => format!(
            "its provenance is {signer}'s, the gate's own identity, which the policy does not let accept too"
        ),
        WhyWanting::Unmatched { signer } => format!(
            "the rebuild by {signer} is the offerer's or did not match"
        ),
    };
    format!("{claim}: {why}")
}

/// The gate's part of one offered version, below the version's lines.
fn gate(out: &mut dyn Write, version: &str, gate: &Gate) -> io::Result<()> {
    match gate {
        Gate::Told(Err(not)) => {
            writeln!(out, "  gate:")?;
            render::not(out, "    ", not)
        }
        Gate::Told(Ok(Judged::Refused {
            artifact,
            wanting: claim,
            why,
            ..
        })) => writeln!(
            out,
            "  gate: refused, {version} stays offered: {artifact} lacks {}",
            wanting(claim, why)
        ),
        Gate::Told(Ok(Judged::Accepted {
            acceptance,
            already,
            one_host,
            channels,
            ..
        })) => {
            write!(out, "  gate: accepted, acceptance {acceptance}")?;
            if *one_host {
                write!(out, ", one host offered and accepted")?;
            }
            if *already {
                write!(out, ", accepted already")?;
            }
            writeln!(out)?;
            for (channel, moved) in channels {
                match moved {
                    Ok(Moved::Promoted {
                        from,
                        to,
                        promotion,
                        ..
                    }) => writeln!(
                        out,
                        "    {channel} promoted from {} to {to}: claim {promotion}",
                        from.as_ref()
                            .map_or_else(|| "nothing".to_string(), ToString::to_string)
                    )?,
                    Ok(Moved::There { at }) => writeln!(out, "    {channel} at {at} already")?,
                    Ok(Moved::Held {
                        artifact,
                        wanting: claim,
                        why,
                    }) => writeln!(
                        out,
                        "    {channel} held: {artifact} lacks {}",
                        wanting(claim, why)
                    )?,
                    Err(not) => {
                        writeln!(out, "    {channel}:")?;
                        render::not(out, "      ", not)?;
                    }
                }
            }
            Ok(())
        }
        Gate::Job(words) => writeln!(
            out,
            "  gate: {version} is offered, and the gate is told by \"{words}\""
        ),
    }
}

fn text(out: &mut dyn Write, answer: &Answer<Offered>) -> io::Result<()> {
    let offered = match answer {
        Ok(offered) => offered,
        Err(not) => return render::not(out, "", not),
    };
    let publish = &offered.publish;
    if publish.components.is_empty() {
        writeln!(out, "no component was touched since its last release")?;
    }
    for ((component, answer), told) in publish.components.iter().zip(&offered.gate) {
        match answer {
            Ok(Published::Version {
                version,
                tag,
                revision,
                index,
                artifacts,
                notes,
                ..
            }) => {
                writeln!(
                    out,
                    "{component} {version} ({tag}) from {revision}: index {index}"
                )?;
                for artifact in artifacts {
                    writeln!(
                        out,
                        "  {} {} {}, from {} by build {}",
                        artifact.ecosystem,
                        artifact.name,
                        artifact.digest,
                        artifact.output,
                        artifact.build
                    )?;
                    for claim in &artifact.claims {
                        writeln!(
                            out,
                            "    {} by {}, envelope {}",
                            claim.kind, claim.signer, claim.envelope
                        )?;
                    }
                }
                if let Some(notes) = notes {
                    writeln!(out, "  notes {}", notes.digest)?;
                    for claim in &notes.claims {
                        writeln!(
                            out,
                            "    {} by {}, envelope {}",
                            claim.kind, claim.signer, claim.envelope
                        )?;
                    }
                    for line in notes.text.lines() {
                        writeln!(out, "{}", format!("    {line}").trim_end())?;
                    }
                }
                if let Some(told) = told {
                    gate(out, version.as_str(), told)?;
                }
            }
            Ok(Published::Unchanged { last, .. }) => match last {
                Some(last) => writeln!(out, "{component} unchanged since {last}")?,
                None => writeln!(out, "{component} unchanged, and never released")?,
            },
            Err(not) => {
                writeln!(out, "{component}:")?;
                render::not(out, "  ", not)?;
            }
        }
    }
    Ok(())
}
