//! `tailrace publish <revision>`: a landed revision offered as a version
//! of each component it publishes (14 to 22). It takes no version: the
//! core derives it (21, rule core.13).

use std::io::{self, Write};
use std::path::Path;

use clap::Args;
use tailrace_core::{publish, Ask, History, Kept, Publish, Published};
use tailrace_seams::{Answer, Branch, ComponentName, Not};

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

pub fn run(args: &PublishArgs, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    let answer = ask(args, binding, at);
    let _ = if args.json {
        render::json(out, &answer)
    } else {
        text(out, &answer)
    };
    match &answer {
        Ok(publish) => render::worst(publish.components.iter().map(|(_, a)| render::exit(a))),
        Err(not) => render::code(not),
    }
}

fn ask(args: &PublishArgs, binding: &Answer<Binding>, at: &Path) -> Answer<Publish> {
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
    publish(&binding.seams, &history, &Kept::at(&history), &ask)
}

fn text(out: &mut dyn Write, answer: &Answer<Publish>) -> io::Result<()> {
    let publish = match answer {
        Ok(publish) => publish,
        Err(not) => return render::not(out, "", not),
    };
    if publish.components.is_empty() {
        writeln!(out, "no component was touched since its last release")?;
    }
    for (component, answer) in &publish.components {
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
