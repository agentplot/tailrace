//! `tailrace policy`: the policy in force at the governed branch's landed
//! head, the one table the givens and the repository's file make, as the
//! policy seam answers it: what acceptance and a layout require, and
//! every channel, the givens beside the repository's own, with how each
//! moves, whether a following one takes prereleases, the claims it
//! requires, its promoters and who may ask for it; and beneath them each
//! given held off, with how it would move and the line that switches it
//! on, from the same channels answer (82, 131, 32, C.10; design D11,
//! D13). It is bound as status is, to the landing and the policy, binds no
//! store and no signer, and writes nothing (60, rule core.32).

use std::io::{self, Write};
use std::path::Path;

use clap::Args;
use tailrace_core::{in_force, ChannelInForce, History, InForce, InForceAsk};
use tailrace_seams::{
    Answer, Branch, HeldChannel, Moves, NamedSigner, Not, Required, RequiredClaim,
};

use crate::{render, Binding};

#[derive(Debug, Args)]
pub struct PolicyArgs {
    /// The branch whose landed head holds the policy.
    #[arg(long, default_value = "main")]
    pub branch: String,
    /// Print the answer as one JSON document.
    #[arg(long)]
    pub json: bool,
}

/// The policy in force asked over `binding` in the repository at `at`:
/// exit 0 whenever an answer is made, each part carrying its own, and the
/// answer's own code where the branch's head cannot be read or has not
/// landed.
pub fn run(args: &PolicyArgs, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    let answer = ask(args, binding, at);
    let _ = match (&answer, args.json) {
        (_, true) => render::json(out, &answer),
        (Ok(in_force), false) => text(out, in_force),
        (Err(not), false) => render::not(out, "", not),
    };
    render::exit(&answer)
}

fn ask(args: &PolicyArgs, binding: &Answer<Binding>, at: &Path) -> Answer<InForce> {
    let binding = binding.as_ref().map_err(Not::clone)?;
    let history = History::open(at)?;
    in_force(
        &binding.seams,
        &history,
        &InForceAsk {
            branch: Branch::new(&args.branch),
        },
    )
}

fn text(out: &mut dyn Write, in_force: &InForce) -> io::Result<()> {
    writeln!(out, "{} at {}", in_force.branch, in_force.revision)?;
    writeln!(out)?;
    writeln!(out, "acceptance")?;
    required(out, &in_force.acceptance)?;
    writeln!(out, "layout")?;
    required(out, &in_force.layout)?;
    writeln!(out)?;
    writeln!(out, "channels")?;
    match &in_force.channels {
        Ok(channels) => {
            for channel in &channels.in_force {
                self::channel(out, channel)?;
            }
            if channels.held.is_empty() {
                return Ok(());
            }
            writeln!(out)?;
            writeln!(out, "held off")?;
            for held in &channels.held {
                self::held(out, held)?;
            }
            Ok(())
        }
        Err(not) => render::not(out, "  ", not),
    }
}

/// How a channel moves, or would once switched on, in the words the
/// channels are shown with.
fn moves(moves: &Moves) -> String {
    match moves {
        Moves::EachPublish { prereleases: false } => "each publish".to_string(),
        Moves::EachPublish { prereleases: true } => "each publish, prereleases too".to_string(),
        Moves::Promotion => "promotion".to_string(),
        Moves::Bound { to } => format!("bound to {to}"),
    }
}

/// One given held off, spelled, how it would move, and the line that
/// switches it on, as the policy words it (C.10).
fn held(out: &mut dyn Write, held: &HeldChannel) -> io::Result<()> {
    writeln!(
        out,
        "  {}, {}, switch on with {}",
        held.channel.channel,
        moves(&held.channel.moves),
        held.switch
    )
}

/// The claims an audience requires, by kind, its check and its signers,
/// in the order the verdict reads them; or its own answer (58).
fn required(out: &mut dyn Write, answer: &Answer<Required>) -> io::Result<()> {
    match answer {
        Ok(required) if required.claims().is_empty() => writeln!(out, "  requires no claim"),
        Ok(required) => {
            for required in required.claims() {
                writeln!(out, "  {}", claim(required))?;
            }
            Ok(())
        }
        Err(not) => render::not(out, "  ", not),
    }
}

/// One channel, spelled, and how it moves; then the claims its policy
/// requires, its promoters and who may ask for it, any asker where it
/// names none, or its policy's own answer (58).
fn channel(out: &mut dyn Write, channel: &ChannelInForce) -> io::Result<()> {
    writeln!(
        out,
        "  {}, {}",
        channel.channel.channel,
        moves(&channel.channel.moves)
    )?;
    match &channel.policy {
        Ok(policy) => {
            let claims: Vec<String> = policy.claims().iter().map(claim).collect();
            if claims.is_empty() {
                writeln!(out, "    requires no claim")?;
            } else {
                writeln!(out, "    requires {}", claims.join(", then "))?;
            }
            writeln!(out, "    promoted by {}", signers(policy.promoted_by()))?;
            let askers = match policy.approvers() {
                [] => "any asker".to_string(),
                approvers => signers(approvers),
            };
            writeln!(out, "    may be asked for by {askers}")
        }
        Err(not) => render::not(out, "    ", not),
    }
}

/// One claim required: its kind, its check where it is a verdict, and
/// the signers that may make it.
fn claim(claim: &RequiredClaim) -> String {
    let check = claim
        .check
        .as_ref()
        .map_or_else(String::new, |c| format!(" of {c}"));
    format!(
        "{}{check} by {}",
        claim.kind.name(),
        signers(&claim.signers)
    )
}

/// Signers by their names, or no signer.
fn signers(signers: &[NamedSigner]) -> String {
    if signers.is_empty() {
        return "no signer".into();
    }
    signers
        .iter()
        .map(|s| s.id.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}
