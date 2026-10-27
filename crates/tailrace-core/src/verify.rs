//! `tailrace verify`: which signer the policy names made each
//! attestation attached to a version's index and artifacts, and each
//! verdict a provenance carries, and whether each counts (3, 9, 80, 81,
//! 116, 117, rules core.4, core.7; design D10), and the notes the index
//! names, read from the store (134). It reaches no verdict of the gate's,
//! and writes nothing.

use serde::{Serialize, Serializer};
use tailrace_seams::{
    Answer, Artifact, Audience, Branch, Check, Claim, ClaimKind, Component, ComponentName, Digest,
    Envelope, Held, Manifest, Name, NamedSigner, Not, Required, Revision, Seam, Seams, SignerId,
    Source, Version, Why,
};

use crate::ledger::listed;
use crate::{
    answer_json, policy_at, Declared, History, Landed, Provenance, BOUND, LOCAL, NOTES, POLICY_FILE,
};

/// What verify is asked: the version, of the one component named or the
/// only one declared, on the branch, under the policy for an audience; a
/// channel by its name, whose component is the version's (125).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifyAsk {
    pub branch: Branch,
    pub component: Option<ComponentName>,
    pub version: Version,
    pub audience: Audience,
}

/// 80: a version's attestations, each told by the signer that made it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Verified {
    pub component: ComponentName,
    pub version: Version,
    pub tag: String,
    /// The revision the version's tag names, the policy read at it (65).
    pub revision: Revision,
    /// The digest the version's name resolved to (3, rule core.4).
    pub index: Digest,
    pub artifacts: Vec<Artifact>,
    pub audience: Audience,
    pub claims: Vec<Told>,
    /// The notes the index names, in the one answer form (57); none for
    /// an index that names none (134).
    #[serde(serialize_with = "notes_json", skip_serializing_if = "Option::is_none")]
    pub notes: Option<Answer<VerifiedNotes>>,
}

/// 134: a version's notes as verify shows them: the digest of their
/// artifact, which the index names, and their text as the store holds it
/// (design D10).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct VerifiedNotes {
    pub digest: Digest,
    pub text: String,
}

fn notes_json<S: Serializer>(
    notes: &Option<Answer<VerifiedNotes>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    notes.as_ref().map(answer_json).serialize(serializer)
}

/// One attestation told: the digest it is attached to, its envelope's
/// digest, its kind and check where tailrace reads them, which named
/// signer made it and whether that identity has expired (81), whether it
/// counts (9, 116), the signers it was checked against, and the
/// provenance it was carried in, for a verdict carried (12).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Told {
    pub on: Digest,
    pub envelope: Digest,
    pub kind: Option<ClaimKind>,
    pub check: Option<Check>,
    pub made_by: Option<SignerId>,
    pub expired: bool,
    pub counts: bool,
    pub checked: Vec<SignerId>,
    pub carried_in: Option<Digest>,
}

fn local() -> Source {
    Source::history(LOCAL)
}

/// 3, 80, 81, 116: the version `ask` names, resolved to its index, each
/// attestation on it told by the signer that made it under the policy at
/// the version's own landed revision (65; design D10).
pub fn verify(seams: &Seams, history: &History, ask: &VerifyAsk) -> Answer<Verified> {
    // 1: the component, as the branch's head declares it.
    let head = history.head(&ask.branch)?;
    let at_head = Landed::check(seams.landing.as_ref(), &ask.branch, &head)?;
    let declared = Declared::read(history, &at_head)?;
    let component = chosen(&declared, ask.component.as_ref(), &head)?;
    let tag = format!("{}{}", component.prefix, ask.version);

    // 2: the name resolved in the store (rule core.4).
    let index = seams.store.resolve(&Name::new(&tag))?;

    // 3: the tag's revision, landed, and the policy at it (65).
    let revision = history
        .tags(&at_head, &component.prefix)?
        .into_iter()
        .find(|t| t.name == tag)
        .map(|t| t.revision)
        .ok_or_else(|| Not::absent(&local(), format!("tag {tag} reached from {head}")))?;
    let landed = Landed::check(seams.landing.as_ref(), &ask.branch, &revision)?;
    // A channel asked by its name alone is the version's component's, as
    // the policy at the revision lists it (125, 131).
    let audience = match &ask.audience {
        Audience::Channel(asked) => {
            Audience::Channel(listed(seams, &landed, &asked.name, &component.name)?.channel)
        }
        other => other.clone(),
    };
    let required =
        policy_at(seams.policy.as_ref(), &landed, &audience).map_err(|not| match not {
            Not::Absent(a) => Not::absent(&a.source, format!("{} at {revision}", a.what)),
            other => other,
        })?;

    // 4: the signers it names, one each, with all it lets each claim.
    let signers = merged(&required);

    // 5, 6: the index and each artifact, each attestation told.
    let Held::Manifest(Manifest::Index(meta)) = seams.store.read(&index)? else {
        return Err(Not::unreadable(
            &Source::new(Seam::Store, BOUND),
            format!("{index} as an index"),
            Some(format!("{index} holds no index")),
        ));
    };
    let mut claims = Vec::new();
    let subjects =
        std::iter::once(index.clone()).chain(meta.artifacts().iter().map(|a| a.digest.clone()));
    for subject in subjects {
        for envelope in seams.store.claims_of(&subject)? {
            let claim = Claim::from_envelope(&envelope);
            let told = tell(
                seams,
                &signers,
                &subject,
                &envelope,
                claim.as_ref().map(|c| c.kind),
                None,
                None,
            )?;
            let carried_in = told.envelope.clone();
            claims.push(told);
            let Some(provenance) = claim.as_ref().and_then(Provenance::of) else {
                continue;
            };
            for carried in &provenance.carried {
                claims.push(tell(
                    seams,
                    &signers,
                    &subject,
                    &carried.envelope,
                    Some(ClaimKind::Verdict),
                    Some(carried.check.clone()),
                    Some(carried_in.clone()),
                )?);
            }
        }
    }

    // 7: the notes the index names, their claims told with the rest.
    let notes = meta
        .artifacts()
        .iter()
        .find(|a| a.meta.ecosystem().as_str() == NOTES)
        .map(|a| notes_at(seams, &a.digest));

    Ok(Verified {
        component: component.name.clone(),
        version: ask.version.clone(),
        tag,
        revision,
        index,
        artifacts: meta.artifacts().to_vec(),
        audience,
        claims,
        notes,
    })
}

/// 134: the notes whose artifact is `digest`, its one layer read from the
/// store as text; unreadable naming the digest when the store holds no
/// artifact of one layer there, or bytes that are not text (58).
fn notes_at(seams: &Seams, digest: &Digest) -> Answer<VerifiedNotes> {
    let store = Source::new(Seam::Store, BOUND);
    let what = format!("the notes {digest}");
    let layer = match seams.store.read(digest)? {
        Held::Manifest(Manifest::Artifact { layers, .. }) if layers.len() == 1 => layers[0].clone(),
        _ => {
            return Err(Not::unreadable(
                &store,
                what,
                Some(format!("{digest} holds no artifact of one layer")),
            ))
        }
    };
    let Held::Bytes(bytes) = seams.store.read(&layer)? else {
        return Err(Not::unreadable(
            &store,
            what,
            Some(format!("{layer} holds no bytes")),
        ));
    };
    let text = String::from_utf8(bytes)
        .map_err(|e| Not::unreadable(&store, what, Some(format!("{layer} is not text: {e}"))))?;
    Ok(VerifiedNotes {
        digest: digest.clone(),
        text,
    })
}

/// The signers `required` names, one each, with all it lets each claim
/// across the claims it requires (82, 116). A promotion's judgment tells
/// by them too.
pub(crate) fn merged(required: &Required) -> Vec<NamedSigner> {
    let mut signers: Vec<NamedSigner> = Vec::new();
    for named in required.claims().iter().flat_map(|c| &c.signers) {
        match signers.iter_mut().find(|s| s.id == named.id) {
            Some(held) => {
                for kind in &named.kinds {
                    if !held.kinds.contains(kind) {
                        held.kinds.push(*kind);
                    }
                }
                for check in &named.checks {
                    if !held.checks.contains(check) {
                        held.checks.push(check.clone());
                    }
                }
            }
            None => signers.push(named.clone()),
        }
    }
    signers
}

/// 123: the component `named`, or the only one `declared` holds at
/// `head`; absent naming it where none is so named, and refused
/// [`Why::WhichComponent`] where several are declared and none named.
pub(crate) fn chosen<'d>(
    declared: &'d Declared,
    named: Option<&ComponentName>,
    head: &Revision,
) -> Answer<&'d Component> {
    match named {
        Some(name) => declared
            .components
            .iter()
            .find(|c| c.name == *name)
            .ok_or_else(|| {
                Not::absent(
                    &local(),
                    format!("component {name} in {POLICY_FILE} at {head}"),
                )
            }),
        None => match declared.components.as_slice() {
            [one] => Ok(one),
            several => Err(Not::refused(
                &local(),
                Why::WhichComponent {
                    declared: several.iter().map(|c| c.name.clone()).collect(),
                },
                None,
            )),
        },
    }
}

/// 80: which of `signers` made `envelope`, asked of each bound signer in
/// turn until one says; none saying is no named signer, and one that
/// cannot be read ends the verify unreadable (58). It counts only when
/// its maker may claim its kind, and for a verdict its check (9, 116,
/// rule core.7); an expired maker is reported beside and still counts
/// (81, 117). The gate tells by it too, and adds the joins acceptance
/// needs, and a promotion's judgment tells by it unchanged.
pub(crate) fn tell(
    seams: &Seams,
    signers: &[NamedSigner],
    on: &Digest,
    envelope: &Envelope,
    kind: Option<ClaimKind>,
    check: Option<Check>,
    carried_in: Option<Digest>,
) -> Answer<Told> {
    let mut made = None;
    for signer in &seams.signers {
        match signer.verify(envelope, signers) {
            Ok(m) => {
                made = Some(m);
                break;
            }
            Err(Not::Refused(_)) => continue,
            Err(not) => return Err(not),
        }
    }
    let counts = made.as_ref().is_some_and(|m| {
        signers.iter().any(|s| {
            s.id == m.signer
                && kind.is_some_and(|k| s.kinds.contains(&k))
                && (kind != Some(ClaimKind::Verdict)
                    || check.as_ref().is_some_and(|c| s.checks.contains(c)))
        })
    });
    Ok(Told {
        on: on.clone(),
        envelope: envelope.digest(),
        kind,
        check,
        expired: made.as_ref().is_some_and(|m| m.expired),
        made_by: made.map(|m| m.signer),
        counts,
        checked: signers.iter().map(|s| s.id.clone()).collect(),
        carried_in,
    })
}
