//! Consumer layouts: each version a checkout's lockfile pins, found in
//! the store by its ecosystem, name and version alone, judged under the
//! layout's policy, and rendered into the checkout when its claims
//! satisfy it (47 to 51, 113, 138; design D2). A pin is resolved
//! through the name `artifact/<ecosystem>/<name>@<version>` to the
//! version's index, and the artifact of that ecosystem and name found in
//! the index by its metadata; the digests resolved are the ones acted on
//! (rule core.4). A name the store does not hold is absent, as most of a
//! lockfile's pins are, and never a refusal (51, 58).
//!
//! A layout is judged under the consuming repository's own policy for
//! layouts, its `[layout]` or the given, read once at the landed revision
//! sync runs for and never from the checkout (113, 65, C.10, rule
//! core.22); where it has no policy of its own, each version under its
//! publisher's, read from the one layout-policy claim on the version's
//! index that a signer the host trusts made, each claim it requires
//! counted only through a signer the host trusts too (C.10, 118). A
//! version with no policy anywhere is refused, so no layout holds a
//! version judged under none (rule core.17; design D3). A version is held only when each claim the policy
//! requires, in its order, is satisfied by a claim attached to its index
//! or its artifact that a signer the policy names made and may make (47,
//! 9, 116, rule core.7); the first that none satisfies is the refusal,
//! naming it (99; design D4).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tailrace_seams::{
    Answer, Artifact, ArtifactMeta, Audience, Claim, ClaimKind, Digest, Ecosystem, EcosystemName,
    Envelope, Held, Manifest, Name, NamedSigner, Not, Pinned, Policy, RequiredClaim, Revision,
    Seam, Seams, SignerId, Source, Store, Version, Why,
};

use crate::verify::{tell, Told};
use crate::{policy_at, Landed, Provenance, BOUND};

/// What every name of a version's index by ecosystem, name and version
/// begins with (138).
pub const ARTIFACT: &str = "artifact/";

/// 138: the store's name for the index of `version` of `name` in
/// `ecosystem`.
pub fn artifact_name(ecosystem: &Ecosystem, name: &EcosystemName, version: &Version) -> Name {
    Name::new(format!("{ARTIFACT}{ecosystem}/{name}@{version}"))
}

/// A pin's own answer inside the answer of the reads it took. A read its
/// source answered unreadable or refused is the outer answer, and leaves
/// the layout as it was (51; design D6). Absent, refused under the
/// policy, and unreadable for what the core found in what the store held
/// are the pin's own, inner answer, and every other pin is judged and
/// rendered all the same.
pub type Pinning<T> = Answer<Answer<T>>;

/// 138, 4: a pin found: the index its name resolved to, and the one
/// artifact of the pin's ecosystem, name and version the index names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Located {
    pub index: Digest,
    pub artifact: Artifact,
}

/// 138, 4: `pin` of `ecosystem` resolved to its index and the artifact in
/// it (rule core.4). A name the store does not hold is absent naming the
/// name (51); a name at a digest holding no index, or at an index naming
/// no one artifact of that ecosystem, name and version, is unreadable
/// naming the name and the digest. A resolve or read its source answers
/// unreadable or refused is the outer answer.
pub fn locate(store: &dyn Store, ecosystem: &Ecosystem, pin: &Pinned) -> Pinning<Located> {
    let name = artifact_name(ecosystem, &pin.name, &pin.version);
    let index = match store.resolve(&name) {
        Ok(index) => index,
        Err(Not::Absent(a)) => return Ok(Err(Not::absent(&a.source, format!("name {name}")))),
        Err(not) => return Err(not),
    };
    let not_found = |said: String| {
        Not::unreadable(
            &Source::new(Seam::Store, BOUND),
            format!("{name} at {index}"),
            Some(said),
        )
    };
    let meta = match store.read(&index) {
        Ok(Held::Manifest(Manifest::Index(meta))) => meta,
        Ok(_) => return Ok(Err(not_found(format!("{index} holds no index")))),
        Err(Not::Absent(_)) => return Ok(Err(not_found(format!("{index} is not held")))),
        Err(not) => return Err(not),
    };
    let named: Vec<&Artifact> = meta
        .artifacts()
        .iter()
        .filter(|a| {
            a.meta.ecosystem() == ecosystem
                && a.meta.name() == &pin.name
                && a.meta.version() == &pin.version
        })
        .collect();
    match named.as_slice() {
        [artifact] => Ok(Ok(Located {
            index,
            artifact: (*artifact).clone(),
        })),
        none_or_several => Ok(Err(not_found(format!(
            "{index} names {} artifacts of {ecosystem} {} {}, and a pin takes one",
            none_or_several.len(),
            pin.name,
            pin.version
        )))),
    }
}

/// 113: the consuming repository's own policy for layouts: the landed
/// revision it was read at, and the claims it requires in its order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Own {
    pub revision: Revision,
    pub claims: Vec<RequiredClaim>,
}

/// 113: which policy judged a version, as the answer and the record name
/// it: the consumer's own at the landed revision, or its publisher's at
/// the revision the publisher's claim names.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyOf {
    Own { revision: Revision },
    Publisher { revision: Revision },
}

/// 113, 65: the consuming repository's policy for layouts at `landed`,
/// never the checkout's (rule core.22); none where it declares none, so
/// each version is judged under its publisher's; the policy's own answer
/// where it cannot say.
pub fn own_policy(policy: &dyn Policy, landed: &Landed) -> Answer<Option<Own>> {
    match policy_at(policy, landed, &Audience::Layout) {
        Ok(required) => Ok(Some(Own {
            revision: landed.revision().clone(),
            claims: required.claims().to_vec(),
        })),
        Err(Not::Absent(_)) => Ok(None),
        Err(not) => Err(not),
    }
}

/// 113: the body of the publisher's layout-policy claim: the claims a
/// layout must require, in the policy's order (design D8).
pub fn layout_policy_body(claims: &[RequiredClaim]) -> Value {
    json!({ "claims": claims })
}

#[derive(Deserialize)]
struct LayoutPolicyBody {
    claims: Vec<RequiredClaim>,
}

/// C.10, rule core.7: `claims`, a publisher's policy, each narrowed to
/// the signers it names that the host trusts, each verified by the key
/// the host trusts it by and allowed what the policy lets it claim. A
/// claim none of whose signers the host trusts names none, and so is
/// found wanting.
fn through_trusted(trusted: &[NamedSigner], claims: Vec<RequiredClaim>) -> Vec<RequiredClaim> {
    claims
        .into_iter()
        .map(|claim| RequiredClaim {
            signers: claim
                .signers
                .into_iter()
                .filter_map(|named| {
                    let trust = trusted.iter().find(|t| t.id == named.id)?;
                    Some(NamedSigner {
                        verified_by: trust.verified_by.clone(),
                        ..named
                    })
                })
                .collect(),
            ..claim
        })
        .collect()
}

/// 113, rule core.17: the policy `version`, whose index is `index`, is
/// judged under, and the claims it requires: the consumer's `own` where it
/// has one; otherwise its publisher's, the one layout-policy claim about
/// the index among `on_index`, the index's attestations, that verify's
/// telling says one of `trusted`, the signers the host trusts, made, its
/// claims each counted only through a signer the host trusts too (C.10,
/// 118, rule core.7; design D3). A layout-policy claim by any other
/// signer stays attached and does not count (9). None counting is refused
/// naming the publisher's layout policy as the claim found wanting,
/// checked against the trusted signers; two of different content are
/// unreadable naming both, and neither is chosen, as a fork is never
/// chosen between (rule core.34). A signer that cannot say is the outer
/// answer.
pub fn policy_for(
    seams: &Seams,
    own: Option<&Own>,
    trusted: &[NamedSigner],
    version: &ArtifactMeta,
    index: &Digest,
    on_index: &[Envelope],
) -> Pinning<(PolicyOf, Vec<RequiredClaim>)> {
    if let Some(own) = own {
        let policy = PolicyOf::Own {
            revision: own.revision.clone(),
        };
        return Ok(Ok((policy, own.claims.clone())));
    }
    let mut found: Vec<(Digest, Claim)> = Vec::new();
    for envelope in on_index {
        let Some(claim) = Claim::from_envelope(envelope) else {
            continue;
        };
        if claim.kind != ClaimKind::LayoutPolicy || claim.subject != *index {
            continue;
        }
        let told = tell(
            seams,
            trusted,
            index,
            envelope,
            Some(claim.kind),
            None,
            None,
        )?;
        if !told.counts {
            continue;
        }
        if !found.iter().any(|(_, held)| held.body == claim.body) {
            found.push((envelope.digest(), claim));
        }
    }
    let store = Source::new(Seam::Store, BOUND);
    match found.as_slice() {
        [] => Ok(Err(Not::refused(
            &Source::new(Seam::Policy, BOUND),
            Why::Wanting {
                version: version.clone(),
                index: index.clone(),
                wanting: RequiredClaim {
                    kind: ClaimKind::LayoutPolicy,
                    check: None,
                    signers: trusted.to_vec(),
                },
                checked: trusted.iter().map(|t| t.id.clone()).collect(),
            },
            None,
        ))),
        [(envelope, claim)] => {
            let body: LayoutPolicyBody = match serde_json::from_value(claim.body.clone()) {
                Ok(body) => body,
                Err(e) => {
                    return Ok(Err(Not::unreadable(
                        &store,
                        format!("the layout policy {envelope} on {index}"),
                        Some(e.to_string()),
                    )))
                }
            };
            let policy = PolicyOf::Publisher {
                revision: claim.revision.clone(),
            };
            Ok(Ok((policy, through_trusted(trusted, body.claims))))
        }
        several => {
            let claims: Vec<String> = several.iter().map(|(d, _)| d.to_string()).collect();
            Ok(Err(Not::unreadable(
                &store,
                format!("the layout policies {} on {index}", claims.join(" and ")),
                Some(format!(
                    "{} layout-policy claims of different content on one index, and none is chosen",
                    several.len()
                )),
            )))
        }
    }
}

/// Whether `told` satisfies `wanted`: it counts, by a signer the claim
/// names and may make it (rule core.7), and is of the claim's kind and,
/// for a verdict, its check (116).
fn satisfies(told: &Told, wanted: &RequiredClaim) -> bool {
    told.counts
        && told.kind == Some(wanted.kind)
        && (wanted.kind != ClaimKind::Verdict || told.check == wanted.check)
}

/// 47, 9, 116, rule core.7: whether the version at `located` satisfies
/// `required`, the claims its policy requires, by `on_index` and
/// `on_artifact`, the attestations attached to its index and its
/// artifact (design D4). Each required claim, in the policy's order, is
/// satisfied by an attestation, or a verdict carried in a provenance
/// among them, that verify's telling says a signer the claim names made
/// and may make. The first that none satisfies is refused
/// [`Why::Wanting`], naming it and the signers checked (99). A claim by an
/// expired signer counts, and the answer names the expired signers whose
/// claims counted (81, 117). A signer that cannot say is the outer
/// answer.
pub fn judge(
    seams: &Seams,
    required: &[RequiredClaim],
    located: &Located,
    on_index: &[Envelope],
    on_artifact: &[Envelope],
) -> Pinning<Vec<SignerId>> {
    let attached = [
        (&located.index, on_index),
        (&located.artifact.digest, on_artifact),
    ];
    let mut expired: Vec<SignerId> = Vec::new();
    for wanted in required {
        let mut counted: Option<Told> = None;
        'attached: for (on, envelopes) in attached {
            for envelope in envelopes {
                let claim = Claim::from_envelope(envelope);
                let kind = claim.as_ref().map(|c| c.kind);
                // A verdict counts only carried with its check, as verify
                // tells it.
                if kind == Some(wanted.kind) && wanted.kind != ClaimKind::Verdict {
                    let told = tell(seams, &wanted.signers, on, envelope, kind, None, None)?;
                    if satisfies(&told, wanted) {
                        counted = Some(told);
                        break 'attached;
                    }
                }
                if wanted.kind != ClaimKind::Verdict {
                    continue;
                }
                let Some(provenance) = claim.as_ref().and_then(Provenance::of) else {
                    continue;
                };
                for carried in &provenance.carried {
                    if wanted.check.as_ref() != Some(&carried.check) {
                        continue;
                    }
                    let told = tell(
                        seams,
                        &wanted.signers,
                        on,
                        &carried.envelope,
                        Some(ClaimKind::Verdict),
                        Some(carried.check.clone()),
                        Some(envelope.digest()),
                    )?;
                    if satisfies(&told, wanted) {
                        counted = Some(told);
                        break 'attached;
                    }
                }
            }
        }
        match counted {
            Some(told) => {
                if let Some(signer) = told.made_by.filter(|_| told.expired) {
                    if !expired.contains(&signer) {
                        expired.push(signer);
                    }
                }
            }
            None => {
                return Ok(Err(Not::refused(
                    &Source::new(Seam::Policy, BOUND),
                    Why::Wanting {
                        version: located.artifact.meta.clone(),
                        index: located.index.clone(),
                        wanting: wanted.clone(),
                        checked: wanted.signers.iter().map(|s| s.id.clone()).collect(),
                    },
                    None,
                )))
            }
        }
    }
    Ok(Ok(expired))
}
