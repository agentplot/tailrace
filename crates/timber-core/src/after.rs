//! An environment's newest report, read across its channel's chain (43,
//! 91, C.12, rules core.7, core.50; design D8).
//!
//! Every version an environment ran came to it through its channel (40,
//! rule core.16), so the claims attached to the indexes the channel's
//! chain moved to hold every report of it. A report counts only through
//! a signer the requirement names that may claim one (rule core.7), and
//! the newest is the one whose apply came last by the moment it names,
//! never by the time the target says it converged, which is the
//! machine's clock (design D8). Nothing is written.

use serde::Serialize;
use timber_seams::{
    Answer, ChannelId, Claim, ClaimKind, ComponentName, Digest, Envelope, EnvironmentName, Not,
    RequiredClaim, Seam, Seams, Source,
};

use crate::ledger::{held, promotion_of};
use crate::{tell, Reported, BOUND};

/// 43, 91: an environment's newest report: the digest of the claim's
/// envelope and what it reports.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Newest {
    pub claim: Digest,
    pub report: Reported,
}

/// 40, 43, 91, rule core.50: the newest report of `environment` of
/// `component`, which follows `channel`: the claims attached to each
/// index `channel`'s chain moved to, once each in the order the chain
/// first moved to it, the reports of that environment and component
/// among them that count through the signers `required` names (rule
/// core.7), and of those the one with the latest `at`, the later in that
/// order where two name one moment. Absent naming the environment where
/// none counts; unreadable naming the chain where the ledger answers it
/// as last read or cannot read it (60), and naming the index whose claims
/// the store could not read (58).
pub fn newest(
    seams: &Seams,
    environment: &EnvironmentName,
    component: &ComponentName,
    channel: &ChannelId,
    required: &RequiredClaim,
) -> Answer<Newest> {
    // 1: the chain of the channel the environment follows (rule
    // core.50), and each index it moved to, once.
    let mut indexes: Vec<Digest> = Vec::new();
    for envelope in held(seams.keeper.as_ref(), channel)? {
        let index = promotion_of(channel, &envelope)?.to.index;
        if !indexes.contains(&index) {
            indexes.push(index);
        }
    }

    // 2, 3: the reports of the environment and component attached to
    // each, kept where they count through the signers the requirement
    // names (rule core.7).
    let mut counted: Vec<Newest> = Vec::new();
    for index in &indexes {
        for envelope in claims_of(seams, channel, index)? {
            let Some(report) = Claim::from_envelope(&envelope)
                .as_ref()
                .and_then(Reported::of)
                .filter(|r| r.environment == *environment && r.component == *component)
            else {
                continue;
            };
            let told = tell(
                seams,
                &required.signers,
                index,
                &envelope,
                Some(ClaimKind::Report),
                None,
                None,
            )?;
            if told.counts {
                counted.push(Newest {
                    claim: told.envelope,
                    report,
                });
            }
        }
    }

    // 4: the one whose apply came last.
    counted
        .into_iter()
        .max_by_key(|n| n.report.at)
        .ok_or_else(|| {
            Not::absent(
                &Source::new(Seam::Store, BOUND),
                format!("a counted report of environment {environment} of {component}"),
            )
        })
}

/// The claims attached to `index`, which `channel`'s chain moved to: an
/// index the store does not hold is unreadable naming it, since the chain
/// says a version was there and its reports cannot be read (58).
fn claims_of(seams: &Seams, channel: &ChannelId, index: &Digest) -> Answer<Vec<Envelope>> {
    seams.store.claims_of(index).map_err(|not| match not {
        Not::Absent(a) => Not::unreadable(
            &a.source,
            format!(
                "the claims of {index}, which the chain of {channel} moved to and the store \
                 holds none of"
            ),
            None,
        ),
        other => other,
    })
}
