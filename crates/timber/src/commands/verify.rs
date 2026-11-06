//! `timber verify <version>`: which signer the policy names made each
//! attestation on a version, and whether each counts (80, 81, 116), and
//! the notes it carries (134).

use std::io::{self, Write};
use std::path::Path;

use clap::Args;
use timber_core::{verify, History, Verified, VerifyAsk};
use timber_seams::{Answer, Audience, Branch, ChannelId, ChannelName, ComponentName, Not, Version};

use crate::{render, Binding};

#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// The version to verify, bare of its component's prefix.
    pub version: String,
    /// The component it is a version of; needed only where more than one
    /// is declared.
    #[arg(long)]
    pub component: Option<String>,
    /// The branch whose landed revisions hold the policy.
    #[arg(long, default_value = "main")]
    pub branch: String,
    /// The audience whose policy names the signers: `acceptance`,
    /// `layout` or `channel:<name>`.
    #[arg(long, default_value = "acceptance", value_parser = audience)]
    pub policy: Audience,
    /// Print the answer as one JSON document.
    #[arg(long)]
    pub json: bool,
}

/// An audience as `--policy` names it.
fn audience(text: &str) -> Result<Audience, String> {
    match text {
        "acceptance" => Ok(Audience::Acceptance),
        "layout" => Ok(Audience::Layout),
        _ => match text.strip_prefix("channel:") {
            Some(channel) if !channel.is_empty() => Ok(Audience::Channel(ChannelId::new(
                None,
                ChannelName::new(channel),
            ))),
            _ => Err(format!(
                "{text} is no audience: acceptance, layout or channel:<name>"
            )),
        },
    }
}

pub fn run(args: &VerifyArgs, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    let answer = ask(args, binding, at);
    let _ = if args.json {
        render::json(out, &answer)
    } else {
        text(out, &answer)
    };
    render::exit(&answer)
}

fn ask(args: &VerifyArgs, binding: &Answer<Binding>, at: &Path) -> Answer<Verified> {
    let binding = binding.as_ref().map_err(Not::clone)?;
    let history = History::open(at)?;
    let ask = VerifyAsk {
        branch: Branch::new(&args.branch),
        component: args.component.as_ref().map(ComponentName::new),
        version: Version::new(&args.version),
        audience: args.policy.clone(),
    };
    verify(&binding.seams, &history, &ask)
}

fn text(out: &mut dyn Write, answer: &Answer<Verified>) -> io::Result<()> {
    let verified = match answer {
        Ok(verified) => verified,
        Err(not) => return render::not(out, "", not),
    };
    writeln!(
        out,
        "{} {} ({}) from {}: index {}, under the policy for {}",
        verified.component,
        verified.version,
        verified.tag,
        verified.revision,
        verified.index,
        match &verified.audience {
            Audience::Acceptance => "acceptance".to_string(),
            Audience::Layout => "a layout".to_string(),
            Audience::Channel(channel) => format!("channel {channel}"),
        }
    )?;
    for told in &verified.claims {
        writeln!(out, "  {}", render::told(told))?;
    }
    match &verified.notes {
        Some(Ok(notes)) => {
            writeln!(out, "notes {}", notes.digest)?;
            for line in notes.text.lines() {
                writeln!(out, "{}", format!("  {line}").trim_end())?;
            }
        }
        Some(Err(not)) => {
            writeln!(out, "notes:")?;
            render::not(out, "  ", not)?;
        }
        None => {}
    }
    Ok(())
}
