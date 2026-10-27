//! The gate: an offered version judged under the policy for acceptance
//! at the governed branch's landed head, accepted whole with one claim of
//! the gate's own or refused whole naming the claim found wanting, and
//! each channel that follows each publish moved by the gate itself (25
//! to 30, 94 to 96, 107, 108, rules core.4, core.7, core.14; design D1).
//! A publish's answer tells it of the index it offered, and the gate
//! judges that index or nothing (95, 130). It imports nothing of publish,
//! and publish nothing of it: what calls one and then the other is the
//! host's binding of what tells the gate, never the core (130).
//!
//! The channels that follow each publish, and the signer the gate
//! accepts as, are learned through the policy seam alone: its channels
//! answer and the policy for each channel listed, never a key of the
//! policy file (131, rules core.1, core.2).

use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use tailrace_seams::{
    channel_name, Answer, Audience, Branch, ChannelId, Claim, ClaimKind, ComponentName, Digest,
    Envelope, Held, Manifest, Moves, Name, NamedSigner, Not, Outcome, Promotion, Required,
    RequiredClaim, Revision, Satisfied, Seam, Seams, SignerId, Source, Version,
};

use crate::ledger::{held, may_promote, promotion_of};
use crate::{
    answer_json, channels_at, chosen, follow, is_prerelease, policy_at, tell, Declared, Followed,
    History, Landed, Provenance, BOUND, LOCAL,
};

/// What told the gate of a version (95): a publish's answer, naming the
/// index it offered, or an asker or a rule the policy names, naming none.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToldBy {
    /// A publish's answer, and `index` the digest it named for the
    /// version: the gate judges that index, and answers unreadable where
    /// the version's name now resolves to another.
    Publish { index: Digest },
    /// Anything else, judged by what the version's name resolves to.
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
        channels: Vec<(ChannelId, Answer<Moved>)>,
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

/// 57: each channel's answer in the one form, naming its channel with
/// its component (125), as a publish names each component's.
fn each_channel<S: Serializer>(
    channels: &[(ChannelId, Answer<Moved>)],
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

    // 2: the name resolved in the store (rule core.4), and the index a
    // publish told the gate of, or nothing is read further (95).
    let index = seams.store.resolve(&Name::new(&tag))?;
    if let ToldBy::Publish { index: told } = &ask.told {
        if *told != index {
            return Err(Not::unreadable(
                &Source::new(Seam::Store, BOUND),
                format!("{tag} as the index a publish named"),
                Some(format!(
                    "{tag} resolves to {index}, and the publish named {told}"
                )),
            ));
        }
    }

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
    // The channels that follow each publish, read from the policy seam
    // before anything is written and followed only when a publish told
    // the gate (95, 131).
    let followed = match ask.told {
        ToldBy::Publish { .. } => following(seams, &at_head, &component.name, &ask.version)?,
        ToldBy::Asked => Vec::new(),
    };

    // 5: the index, the claims of it and of each artifact, and how each
    // artifact was made, told once under the acceptance policy (29, 134).
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
        let made = made(seams, &required, &revision, &artifact.digest, &claims)?;
        artifacts.push((artifact.digest.clone(), made, claims));
    }
    let mut judging = Judging {
        seams,
        gate: &ask.gate,
        component: &component.name,
        version: &ask.version,
        revision: &revision,
        index: &index,
        on_index,
        artifacts,
    };

    // 6: an acceptance the gate's identity made already answers the
    // version accepted, whatever the head's policy now requires (94).
    let mut found = None;
    if judging.holds_acceptance() {
        if let Some(named) = gate_named(seams, &at_head, &required, &ask.gate)? {
            found = judging.already(&named)?;
        }
    }
    let (acceptance, already, one_host) = match found {
        Some((acceptance, one_host)) => (acceptance, true, one_host),
        None => {
            // 7: every artifact against every required claim (28).
            let (claims, offered_by, one_host) = match judging.count(&required)? {
                Count::Satisfied {
                    claims,
                    offered_by,
                    one_host,
                } => (claims, offered_by, one_host),
                Count::Wanting {
                    artifact,
                    wanting,
                    why,
                } => {
                    return Ok(Judged::Refused {
                        component: component.name.clone(),
                        version: ask.version.clone(),
                        index: index.clone(),
                        revision: revision.clone(),
                        policy_at: head.clone(),
                        artifact,
                        wanting,
                        why,
                    })
                }
            };

            // 8: the acceptance signed before the one write, then
            // attached to the index alone (26, 7), saying who offered what
            // it accepts and whether that was the gate's own identity
            // (108).
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
                    "offered": { "by": offered_by, "one_host": one_host },
                }),
            };
            let envelope = sign(seams, &acceptance, &ask.gate)?;
            seams.store.attach(&index, &envelope)?;
            let digest = envelope.digest();
            judging.on_index.push(envelope);
            (digest, false, one_host)
        }
    };

    // 9: each following channel judged under its own policy and moved by
    // the gate, each answered on its own (95, design D7).
    let channels = followed
        .into_iter()
        .map(|channel| {
            let moved = judging.promote(&at_head, &channel, ask.when);
            (channel, moved)
        })
        .collect();

    Ok(Judged::Accepted {
        component: component.name.clone(),
        version: ask.version.clone(),
        index: index.clone(),
        revision: revision.clone(),
        policy_at: head.clone(),
        acceptance,
        already,
        one_host,
        channels,
    })
}

/// 95, 125, 131: the channels of `component` that the policy's channels
/// answer at the landed head says move on each publish, each as its
/// identity, in the answer's order, leaving out those that take releases
/// alone where `version` is a prerelease (21, C.10); the answer's own
/// where it cannot be read, so nothing is accepted or moved.
pub fn following(
    seams: &Seams,
    landed: &Landed,
    component: &ComponentName,
    version: &Version,
) -> Answer<Vec<ChannelId>> {
    let prerelease = is_prerelease(version);
    Ok(channels_at(seams.policy.as_ref(), landed)?
        .into_iter()
        .filter(|c| match c.moves {
            Moves::EachPublish { prereleases } => {
                c.channel.component_name() == *component && (prereleases || !prerelease)
            }
            Moves::Promotion | Moves::Bound { .. } => false,
        })
        .map(|c| c.channel)
        .collect())
}

/// 82, 107: what `audience` requires at the landed head; absent naming
/// the head where no policy is declared for it.
fn required_at(seams: &Seams, at_head: &Landed, audience: &Audience) -> Answer<Required> {
    policy_at(seams.policy.as_ref(), at_head, audience).map_err(|not| match not {
        Not::Absent(a) => Not::absent(&a.source, format!("{} at {}", a.what, at_head.revision())),
        other => other,
    })
}

/// 94: the signer the policy at the head names by the gate's identity
/// and lets claim acceptance, looked for in what acceptance requires and
/// then in what each channel the policy lists requires, since a channel
/// is what asks for the gate's acceptance; none where no policy names
/// one (131, rule core.7).
fn gate_named(
    seams: &Seams,
    at_head: &Landed,
    acceptance: &Required,
    gate: &SignerId,
) -> Answer<Option<NamedSigner>> {
    let named = |required: &Required| {
        required
            .claims()
            .iter()
            .flat_map(|c| &c.signers)
            .find(|s| s.id == *gate && s.kinds.contains(&ClaimKind::Acceptance))
            .cloned()
    };
    if let Some(found) = named(acceptance) {
        return Ok(Some(found));
    }
    for listed in channels_at(seams.policy.as_ref(), at_head)? {
        match required_at(seams, at_head, &Audience::Channel(listed.channel)) {
            Ok(required) => {
                if let Some(found) = named(&required) {
                    return Ok(Some(found));
                }
            }
            Err(Not::Absent(_)) => continue,
            Err(not) => return Err(not),
        }
    }
    Ok(None)
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

/// 134: how an artifact of a version was made, as the gate tells it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Made {
    /// A build made it: a provenance that counts names its build, or none
    /// says it was rendered.
    Built,
    /// Rendered with no build behind it, as a version's notes are: a
    /// provenance that counts names no build, and none names one.
    Rendered,
}

/// 13, 29, 134: how `artifact` was made, told from the provenance of
/// `revision` attached to it alone (never to the index), counting only
/// through a signer `required` names for a provenance, within its reach,
/// and not expired (rule core.7): rendered where such a provenance counts
/// and none that counts names a build, built otherwise, so a policy that
/// asks no provenance judges every artifact as built.
fn made(
    seams: &Seams,
    required: &Required,
    revision: &Revision,
    artifact: &Digest,
    own: &[Envelope],
) -> Answer<Made> {
    let mut signers: Vec<NamedSigner> = Vec::new();
    for signer in required
        .claims()
        .iter()
        .filter(|c| c.kind == ClaimKind::Provenance)
        .flat_map(|c| &c.signers)
    {
        if !signers.iter().any(|s| s.id == signer.id) {
            signers.push(signer.clone());
        }
    }
    if signers.is_empty() {
        return Ok(Made::Built);
    }
    let mut rendered = false;
    for envelope in own {
        let Some(claim) = Claim::from_envelope(envelope) else {
            continue;
        };
        if claim.kind != ClaimKind::Provenance || claim.revision != *revision {
            continue;
        }
        let told = tell(
            seams,
            &signers,
            artifact,
            envelope,
            Some(claim.kind),
            None,
            None,
        )?;
        if !told.counts || told.expired {
            continue;
        }
        if claim.build.is_some() {
            return Ok(Made::Built);
        }
        rendered = true;
    }
    Ok(if rendered {
        Made::Rendered
    } else {
        Made::Built
    })
}

/// Every signer `required` names, once each, as it first names it.
fn named_in(required: &Required) -> Vec<NamedSigner> {
    let mut named: Vec<NamedSigner> = Vec::new();
    for signer in required.claims().iter().flat_map(|c| &c.signers) {
        if !named.iter().any(|n| n.id == signer.id) {
            named.push(signer.clone());
        }
    }
    named
}

/// 108: whether `wanted` names `signer` as one that may claim both
/// provenance and acceptance, the one host that offers and accepts.
fn offers_and_accepts(wanted: &RequiredClaim, signer: &SignerId) -> bool {
    wanted.signers.iter().any(|s| {
        s.id == *signer
            && s.kinds.contains(&ClaimKind::Provenance)
            && s.kinds.contains(&ClaimKind::Acceptance)
    })
}

/// One version of `component` being judged by the gate signing as
/// `gate`: its revision, its index, the claims attached to the index,
/// and each artifact in the index's order with how it was made and the
/// claims attached to it.
struct Judging<'a> {
    seams: &'a Seams,
    gate: &'a SignerId,
    component: &'a ComponentName,
    version: &'a Version,
    revision: &'a Revision,
    index: &'a Digest,
    on_index: Vec<Envelope>,
    artifacts: Vec<(Digest, Made, Vec<Envelope>)>,
}

/// What one policy's judgment of every artifact came to.
enum Count {
    /// Every required claim found, for every artifact: the envelopes that
    /// satisfied each, in order; the makers of the provenance counted;
    /// and whether the gate's own identity made one (108).
    Satisfied {
        claims: Vec<Digest>,
        offered_by: Vec<SignerId>,
        one_host: bool,
    },
    /// The first artifact and the first required claim found wanting.
    Wanting {
        artifact: Digest,
        wanting: RequiredClaim,
        why: WhyWanting,
    },
}

impl Judging<'_> {
    /// Whether the index holds any claim of acceptance of tailrace's form,
    /// whoever made it.
    fn holds_acceptance(&self) -> bool {
        self.on_index
            .iter()
            .filter_map(Claim::from_envelope)
            .any(|c| c.kind == ClaimKind::Acceptance)
    }

    /// 94: the envelope of an acceptance of the index that `named`, the
    /// gate's identity, made and may make, and whether it said one host;
    /// none where the gate made none, a forged one by another signer
    /// among them.
    fn already(&self, named: &NamedSigner) -> Answer<Option<(Digest, bool)>> {
        for envelope in &self.on_index {
            let Some(claim) = Claim::from_envelope(envelope) else {
                continue;
            };
            if claim.kind != ClaimKind::Acceptance || claim.subject != *self.index {
                continue;
            }
            let told = tell(
                self.seams,
                std::slice::from_ref(named),
                self.index,
                envelope,
                Some(claim.kind),
                None,
                None,
            )?;
            if told.counts && told.made_by.as_ref() == Some(self.gate) {
                let one_host = claim.body["offered"]["one_host"].as_bool();
                return Ok(Some((told.envelope, one_host.unwrap_or(false))));
            }
        }
        Ok(None)
    }

    /// 28: each artifact, in the index's order, against each required
    /// claim, in the policy's order; the first wanting refuses the whole.
    fn count(&self, required: &Required) -> Answer<Count> {
        let named = named_in(required);
        let (mut claims, mut offered_by, mut one_host) = (Vec::new(), Vec::new(), false);
        for (artifact, _, own) in &self.artifacts {
            for wanted in required.claims() {
                match self.found(wanted, artifact, own, &named)? {
                    Ok((envelope, maker)) => {
                        claims.push(envelope);
                        if wanted.kind == ClaimKind::Provenance {
                            one_host |= maker == *self.gate;
                            if !offered_by.contains(&maker) {
                                offered_by.push(maker);
                            }
                        }
                    }
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
        Ok(Count::Satisfied {
            claims,
            offered_by,
            one_host,
        })
    }

    /// The envelope of the first claim that counts for `wanted`, among
    /// those attached to `artifact` and then to the index, which a claim
    /// about the version is (7), or why none does: a claim counts only
    /// for the revision it names (13), through a signer `wanted` names
    /// within its reach (9, 116, rule core.7), and never from an identity
    /// that has expired (81). A rebuild counts only from a signer other
    /// than the provenance's maker, saying the digests matched (30). A
    /// provenance the gate's own identity made refuses the version unless
    /// the policy lets that signer claim acceptance too (27, 96, 108), so
    /// every provenance is read before the first that counts is answered.
    fn found(
        &self,
        wanted: &RequiredClaim,
        artifact: &Digest,
        own: &[Envelope],
        named: &[NamedSigner],
    ) -> Answer<Result<(Digest, SignerId), WhyWanting>> {
        let attached = own
            .iter()
            .map(|e| (artifact, e))
            .chain(self.on_index.iter().map(|e| (self.index, e)));
        let (mut near, mut first) = (None, None);
        for (on, envelope) in attached {
            let Some(claim) = Claim::from_envelope(envelope) else {
                continue;
            };
            if claim.revision != *self.revision {
                continue;
            }
            let told = match wanted.kind {
                // 13, 111: a verdict counts carried in the provenance, for
                // the check required and the provenance's build, passing.
                ClaimKind::Verdict => {
                    let Some(provenance) = Provenance::of(&claim) else {
                        continue;
                    };
                    let mut told = Vec::new();
                    for carried in &provenance.carried {
                        if wanted.check.as_ref() != Some(&carried.check)
                            || claim.build.as_ref() != Some(&carried.build)
                            || carried.outcome != Outcome::Pass
                        {
                            continue;
                        }
                        told.push(tell(
                            self.seams,
                            &wanted.signers,
                            on,
                            &carried.envelope,
                            Some(ClaimKind::Verdict),
                            Some(carried.check.clone()),
                            Some(envelope.digest()),
                        )?);
                    }
                    told
                }
                kind if kind == claim.kind => vec![tell(
                    self.seams,
                    &wanted.signers,
                    on,
                    envelope,
                    Some(kind),
                    None,
                    None,
                )?],
                _ => continue,
            };
            for told in told {
                let (true, Some(maker)) = (told.counts, told.made_by) else {
                    continue;
                };
                if told.expired {
                    near.get_or_insert(WhyWanting::Expired { signer: maker });
                    continue;
                }
                if wanted.kind == ClaimKind::Rebuild
                    && (self.offerers(artifact, own, named)?.contains(&maker)
                        || claim.body.get("matched") != Some(&Value::Bool(true)))
                {
                    near.get_or_insert(WhyWanting::Unmatched { signer: maker });
                    continue;
                }
                if wanted.kind != ClaimKind::Provenance {
                    return Ok(Ok((told.envelope, maker)));
                }
                if maker == *self.gate && !offers_and_accepts(wanted, &maker) {
                    return Ok(Err(WhyWanting::OfferedByGate { signer: maker }));
                }
                first.get_or_insert((told.envelope, maker));
            }
        }
        Ok(first.ok_or_else(|| near.unwrap_or(WhyWanting::Missing)))
    }

    /// 33, 35, 39, 105: `channel` judged under its own policy at the head
    /// and, where the gate is among its promoters and the policy is
    /// satisfied, moved to the index: a promotion in the one body every
    /// promotion holds, signed as the gate naming the chain's head as its
    /// predecessor, appended to the chain, attached to the index, and only
    /// then the channel's name in the store moved from where the old head
    /// left it (rules core.6, core.15, core.34). A refused append, a fork
    /// or any other answer is the channel's, and nothing after it is done
    /// or retried.
    fn promote(&self, at_head: &Landed, channel: &ChannelId, when: u64) -> Answer<Moved> {
        // 1: the channel's own policy at the head (107), and the gate one
        // of the signers whose promotion of it counts (32, 9, rule
        // core.7).
        let audience = Audience::Channel(channel.clone());
        let required = required_at(self.seams, at_head, &audience)?;
        may_promote(&required, self.gate)?;
        let claims = match self.count(&required)? {
            Count::Satisfied { claims, .. } => claims,
            Count::Wanting {
                artifact,
                wanting,
                why,
            } => {
                return Ok(Moved::Held {
                    artifact,
                    wanting,
                    why,
                })
            }
        };

        // 2: the chain's head, read from the ledger itself: what the
        // channel moves from, and the promotion the next one follows (60,
        // 105).
        let chain = held(self.seams.keeper.as_ref(), channel)?;
        let head = match chain.last() {
            Some(envelope) => Some((envelope.digest(), promotion_of(channel, envelope)?)),
            None => None,
        };
        let from = head.as_ref().map(|(_, p)| p.to.clone());
        if from.as_ref().is_some_and(|m| m.index == *self.index) {
            return Ok(Moved::There {
                at: self.index.clone(),
            });
        }

        // 3: the promotion, signed as the gate and asked by it, its own
        // act on the publish that told it, whatever approvers the channel
        // names (33, 39, 95, 129, C.10).
        let promotion = Promotion {
            channel: channel.name.clone(),
            component: self.component.clone(),
            to: tailrace_seams::Moved {
                version: self.version.clone(),
                index: self.index.clone(),
            },
            from,
            predecessor: head.map(|(digest, _)| digest),
            asked_by: self.gate.clone(),
            policy: Satisfied {
                revision: at_head.revision().clone(),
                claims,
            },
            at: when,
        };
        let envelope = sign(
            self.seams,
            &promotion.claim(self.revision.clone()),
            self.gate,
        )?;

        // 4 to 6: the ledger first, then the index, then the name.
        self.seams
            .keeper
            .append(channel, &envelope, promotion.predecessor.as_ref())?;
        self.seams.store.attach(self.index, &envelope)?;
        let from = promotion.from.map(|from| from.index);
        let name = follow(
            self.seams.store.as_ref(),
            self.seams.keeper.as_ref(),
            channel,
            &channel_name(channel),
            from.as_ref(),
        )?;
        Ok(Moved::Promoted {
            from,
            to: self.index.clone(),
            promotion: envelope.digest(),
            name,
        })
    }

    /// Which of `named` made each provenance of the version's revision
    /// attached to `artifact`.
    fn offerers(
        &self,
        artifact: &Digest,
        own: &[Envelope],
        named: &[NamedSigner],
    ) -> Answer<Vec<SignerId>> {
        let mut makers = Vec::new();
        for envelope in own {
            let Some(claim) = Claim::from_envelope(envelope) else {
                continue;
            };
            if claim.kind != ClaimKind::Provenance || claim.revision != *self.revision {
                continue;
            }
            let told = tell(
                self.seams,
                named,
                artifact,
                envelope,
                Some(claim.kind),
                None,
                None,
            )?;
            makers.extend(told.made_by);
        }
        Ok(makers)
    }
}
