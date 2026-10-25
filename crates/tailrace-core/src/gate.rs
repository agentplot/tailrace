//! The gate: an offered version judged under the policy for acceptance
//! at the governed branch's landed head, accepted whole with one claim of
//! the gate's own or refused whole naming the claim found wanting, and
//! each channel that follows each publish moved by the gate itself (25
//! to 30, 94 to 96, 107, 108, rules core.4, core.7, core.14; design D1).
//! It imports nothing of publish, and publish nothing of it.

use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use tailrace_seams::{
    Answer, Audience, Branch, ChannelName, Claim, ClaimKind, ComponentName, Digest, Envelope, Held,
    Manifest, Name, Not, Required, RequiredClaim, Revision, Seam, Seams, SignerId, Source, Version,
};

use crate::{
    answer_json, chosen, policy_at, tell, Declared, Followed, History, Landed, BOUND, LOCAL,
};

/// What told the gate of a version (95): a publish's answer, or an asker
/// or a rule the policy names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToldBy {
    Publish,
    Asked,
}

/// What the gate is asked: the version, of the one component named or
/// the only one declared, on the branch; what told it; the identity it
/// signs as, the host's own (79, 95); and the moment of the act, in
/// seconds since the Unix epoch, so the core reads no clock.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AcceptAsk {
    pub branch: Branch,
    pub component: Option<ComponentName>,
    pub version: Version,
    pub told: ToldBy,
    pub gate: SignerId,
    pub when: u64,
}

/// 28, 58: the gate's one verdict on a version, each naming the index
/// its name resolved to (rule core.4), the version's revision, and the
/// branch's landed head the policy was read at (107).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "judged", rename_all = "snake_case")]
pub enum Judged {
    /// Every artifact satisfies every required claim: `acceptance` is the
    /// envelope of the gate's claim on the index, `already` when it was
    /// there before this judgment (94), `one_host` when the gate accepted
    /// what its own identity offered (108), and each channel that follows
    /// each publish answered on its own (95).
    Accepted {
        component: ComponentName,
        version: Version,
        index: Digest,
        revision: Revision,
        policy_at: Revision,
        acceptance: Digest,
        already: bool,
        one_host: bool,
        #[serde(serialize_with = "each_channel")]
        channels: Vec<(ChannelName, Answer<Moved>)>,
    },
    /// `artifact`, the first found wanting in the index's order, lacks
    /// `wanting`, the first required claim in the policy's order it does
    /// not satisfy, for `why`; nothing was written.
    Refused {
        component: ComponentName,
        version: Version,
        index: Digest,
        revision: Revision,
        policy_at: Revision,
        artifact: Digest,
        wanting: RequiredClaim,
        why: WhyWanting,
    },
}

/// What a channel that follows each publish came to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "moved", rename_all = "snake_case")]
pub enum Moved {
    /// The channel moved `from` the version its chain's head was about,
    /// none for an empty chain, `to` the index, by the promotion whose
    /// envelope is `promotion`; `name` is how the store's name followed
    /// (39, 109).
    Promoted {
        from: Option<Digest>,
        to: Digest,
        promotion: Digest,
        name: Followed,
    },
    /// The chain's head was about the index already, and nothing was
    /// written.
    There { at: Digest },
    /// The channel's own policy refused the version: `artifact` lacks
    /// `wanting` for `why`, and the channel did not move.
    Held {
        artifact: Digest,
        wanting: RequiredClaim,
        why: WhyWanting,
    },
}

/// Why a required claim found no claim that counts for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WhyWanting {
    /// No claim attached counts for it (9, 13).
    Missing,
    /// The one claim that would count was made by `signer`, whose
    /// identity has expired (81).
    Expired { signer: SignerId },
    /// The provenance that counts was made by `signer`, the gate's own
    /// identity, which the policy does not let claim acceptance too (27,
    /// 96, 108).
    OfferedByGate { signer: SignerId },
    /// The rebuild that would count was made by `signer`, the maker of
    /// the artifact's provenance, or said the digests did not match (30).
    Unmatched { signer: SignerId },
}

/// 57: each channel's answer in the one form, naming its channel, as a
/// publish names each component's.
fn each_channel<S: Serializer>(
    channels: &[(ChannelName, Answer<Moved>)],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let answers: Vec<Value> = channels
        .iter()
        .map(|(channel, answer)| {
            let mut value = answer_json(answer);
            value["channel"] = json!(channel);
            value
        })
        .collect();
    answers.serialize(serializer)
}

/// 25 to 29, 107: the version `ask` names, resolved to its index, judged
/// under the policy for acceptance at the branch's landed head, in the
/// order of design D2: nothing of the version is read before its name
/// resolves, and nothing is written before the verdict.
pub fn accept(seams: &Seams, history: &History, ask: &AcceptAsk) -> Answer<Judged> {
    // 1: the branch's head, landed (107, 77), and the component it
    // declares.
    let head = history.head(&ask.branch)?;
    let at_head = Landed::check(seams.landing.as_ref(), &ask.branch, &head)?;
    let declared = Declared::read(history, &at_head)?;
    let component = chosen(&declared, ask.component.as_ref(), &head)?;
    let tag = format!("{}{}", component.prefix, ask.version);

    // 2: the name resolved in the store (rule core.4).
    let index = seams.store.resolve(&Name::new(&tag))?;

    // 3: the tag's revision, landed (29, 14).
    let revision = history
        .tags(&at_head, &component.prefix)?
        .into_iter()
        .find(|t| t.name == tag)
        .map(|t| t.revision)
        .ok_or_else(|| {
            Not::absent(
                &Source::history(LOCAL),
                format!("tag {tag} reached from {head}"),
            )
        })?;
    Landed::check(seams.landing.as_ref(), &ask.branch, &revision)?;

    // 4: the policy for acceptance at the head, never at the version's
    // revision (107).
    let required = required_at(seams, &at_head, &Audience::Acceptance)?;

    // 5: the index, and the claims of it and of each artifact (29).
    let Held::Manifest(Manifest::Index(meta)) = seams.store.read(&index)? else {
        return Err(Not::unreadable(
            &Source::new(Seam::Store, BOUND),
            format!("{index} as an index"),
            Some(format!("{index} holds no index")),
        ));
    };
    let on_index = seams.store.claims_of(&index)?;
    let mut artifacts = Vec::new();
    for artifact in meta.artifacts() {
        let claims = seams.store.claims_of(&artifact.digest)?;
        artifacts.push((artifact.digest.clone(), claims));
    }
    let judging = Judging {
        seams,
        revision: &revision,
        index: &index,
        on_index,
        artifacts,
    };

    // 7: every artifact against every required claim (28).
    let refused = |artifact, wanting, why| Judged::Refused {
        component: component.name.clone(),
        version: ask.version.clone(),
        index: index.clone(),
        revision: revision.clone(),
        policy_at: head.clone(),
        artifact,
        wanting,
        why,
    };
    let claims = match judging.count(&required)? {
        Count::Satisfied { claims } => claims,
        Count::Wanting {
            artifact,
            wanting,
            why,
        } => return Ok(refused(artifact, wanting, why)),
    };

    // 8: the acceptance signed before the one write, then attached to
    // the index alone (26, 7).
    let acceptance = Claim {
        subject: index.clone(),
        kind: ClaimKind::Acceptance,
        revision: revision.clone(),
        build: None,
        body: json!({
            "component": component.name,
            "version": ask.version,
            "policy": { "audience": Audience::Acceptance, "revision": head },
            "claims": claims,
        }),
    };
    let envelope = sign(seams, &acceptance, &ask.gate)?;
    seams.store.attach(&index, &envelope)?;

    Ok(Judged::Accepted {
        component: component.name.clone(),
        version: ask.version.clone(),
        index: index.clone(),
        revision: revision.clone(),
        policy_at: head.clone(),
        acceptance: envelope.digest(),
        already: false,
        one_host: false,
        channels: Vec::new(),
    })
}

/// 82, 107: what `audience` requires at the landed head; absent naming
/// the head where no policy is declared for it.
fn required_at(seams: &Seams, at_head: &Landed, audience: &Audience) -> Answer<Required> {
    policy_at(seams.policy.as_ref(), at_head, audience).map_err(|not| match not {
        Not::Absent(a) => Not::absent(&a.source, format!("{} at {}", a.what, at_head.revision())),
        other => other,
    })
}

/// 79, 95: `claim` signed as `gate` by the first bound signer that holds
/// it at hand; absent naming the identity where none does.
fn sign(seams: &Seams, claim: &Claim, gate: &SignerId) -> Answer<Envelope> {
    for signer in &seams.signers {
        match signer.sign(claim, gate) {
            Ok(envelope) => return Ok(envelope),
            Err(Not::Absent(_)) => continue,
            Err(not) => return Err(not),
        }
    }
    Err(Not::absent(
        &Source::new(Seam::Signer, BOUND),
        format!("identity {gate}"),
    ))
}

/// One version being judged: its revision, its index, the claims
/// attached to the index, and each artifact in the index's order with
/// the claims attached to it.
struct Judging<'a> {
    seams: &'a Seams,
    revision: &'a Revision,
    index: &'a Digest,
    on_index: Vec<Envelope>,
    artifacts: Vec<(Digest, Vec<Envelope>)>,
}

/// What one policy's judgment of every artifact came to.
enum Count {
    /// Every required claim found, for every artifact: the envelopes that
    /// satisfied each, in order.
    Satisfied { claims: Vec<Digest> },
    /// The first artifact and the first required claim found wanting.
    Wanting {
        artifact: Digest,
        wanting: RequiredClaim,
        why: WhyWanting,
    },
}

impl Judging<'_> {
    /// 28: each artifact, in the index's order, against each required
    /// claim, in the policy's order; the first wanting refuses the whole.
    fn count(&self, required: &Required) -> Answer<Count> {
        let mut claims = Vec::new();
        for (artifact, own) in &self.artifacts {
            for wanted in required.claims() {
                match self.found(wanted, artifact, own)? {
                    Ok(envelope) => claims.push(envelope),
                    Err(why) => {
                        return Ok(Count::Wanting {
                            artifact: artifact.clone(),
                            wanting: wanted.clone(),
                            why,
                        })
                    }
                }
            }
        }
        Ok(Count::Satisfied { claims })
    }

    /// The envelope of the first claim that counts for `wanted`, among
    /// those attached to `artifact` and then to the index, which a claim
    /// about the version is (7), or why none does (9, 116, rule core.7).
    fn found(
        &self,
        wanted: &RequiredClaim,
        artifact: &Digest,
        own: &[Envelope],
    ) -> Answer<Result<Digest, WhyWanting>> {
        let attached = own
            .iter()
            .map(|e| (artifact, e))
            .chain(self.on_index.iter().map(|e| (self.index, e)));
        for (on, envelope) in attached {
            // 13: a claim counts only for the revision it names.
            let Some(claim) = Claim::from_envelope(envelope) else {
                continue;
            };
            if claim.kind != wanted.kind || claim.revision != *self.revision {
                continue;
            }
            let told = tell(
                self.seams,
                &wanted.signers,
                on,
                envelope,
                Some(claim.kind),
                None,
                None,
            )?;
            if told.counts {
                return Ok(Ok(told.envelope));
            }
        }
        Ok(Err(WhyWanting::Missing))
    }
}
