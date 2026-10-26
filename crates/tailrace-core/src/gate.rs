//! The gate: an offered version judged under the policy for acceptance
//! at the governed branch's landed head, accepted whole with one claim of
//! the gate's own or refused whole naming the claim found wanting, and
//! each channel that follows each publish moved by the gate itself (25
//! to 30, 94 to 96, 107, 108, rules core.4, core.7, core.14; design D1).
//! Where the policy names no gate, the gate's own identity, as the policy
//! names it for provenance, is the one host 108 admits (118).
//! A publish's answer tells it of the index it offered, and the gate
//! judges that index or nothing (95, 130). It imports nothing of publish,
//! and publish nothing of it: what calls one and then the other is the
//! host's binding of what tells the gate, never the core (130).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize, Serializer};
use serde_json::{json, Value};
use tailrace_seams::{
    channel_name, Answer, Audience, Branch, ChannelName, Claim, ClaimKind, ComponentName, Digest,
    Envelope, Held, Manifest, Name, NamedSigner, Not, Outcome, Promotion, Required, RequiredClaim,
    Revision, Satisfied, Seam, Seams, SignerId, Source, Version,
};

use crate::ledger::{held, promotion_of};
use crate::{
    answer_json, chosen, follow, policy_at, tell, Declared, Followed, History, Landed, Provenance,
    BOUND, LOCAL, POLICY_FILE,
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
    // The channels that follow each publish, read before anything is
    // written and followed only when a publish told the gate (95).
    let followed = match ask.told {
        ToldBy::Publish { .. } => following(history, &at_head, &declared, &component.name)?,
        ToldBy::Asked => Vec::new(),
    };

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
    // Whether the policy names a gate, once for the judgment (118).
    let names_a_gate = names_a_gate(seams, history, &at_head, &required)?;
    let mut judging = Judging {
        seams,
        gate: &ask.gate,
        names_a_gate,
        component: &component.name,
        version: &ask.version,
        revision: &revision,
        index: &index,
        on_index,
        artifacts,
    };

    // 6: an acceptance the gate's identity made already answers the
    // version accepted, whatever the head's policy now requires (94); where
    // the policy names no gate, the gate's identity is read as the policy
    // names it, letting it claim acceptance for this reading alone (118).
    let mut found = None;
    if judging.holds_acceptance() {
        let named = if names_a_gate {
            gate_named(seams, history, &at_head, &required, &ask.gate)?
        } else {
            signer_named(seams, history, &at_head, &required, |s| s.id == ask.gate)?.map(
                |mut named| {
                    named.kinds.push(ClaimKind::Acceptance);
                    named
                },
            )
        };
        if let Some(named) = named {
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
        .map(|(channel, readable)| {
            let moved = readable.and_then(|()| judging.promote(&at_head, &channel, ask.when));
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

/// The keys of a `[channels.<name>]` table the gate reads; every other
/// key is left to its reader (design D6).
#[derive(Deserialize)]
struct ChannelKeys {
    #[serde(default)]
    on_each_publish: bool,
    component: Option<ComponentName>,
    bound_to: Option<ChannelName>,
}

/// The policy file's channel tables, each with where it begins.
#[derive(Deserialize)]
struct ChannelTables {
    #[serde(default)]
    channels: BTreeMap<ChannelName, toml::Spanned<toml::Value>>,
}

/// Every `[channels.<name>]` table of the policy file at the landed
/// revision, in the file's order, each read for the keys the gate reads
/// or not, in the parser's words; none where there is no file.
fn channel_tables(
    history: &History,
    landed: &Landed,
) -> Answer<Vec<(ChannelName, Result<ChannelKeys, String>)>> {
    let what = format!("{POLICY_FILE} at {}", landed.revision());
    let unreadable = |said: String| Not::unreadable(&Source::history(LOCAL), &what, Some(said));
    let bytes = match history.blob(landed, POLICY_FILE) {
        Ok(bytes) => bytes,
        Err(Not::Absent(_)) => return Ok(Vec::new()),
        Err(not) => return Err(not),
    };
    let text = String::from_utf8(bytes).map_err(|e| unreadable(e.to_string()))?;
    let file: ChannelTables = toml::from_str(&text).map_err(|e| unreadable(e.to_string()))?;
    let mut tables: Vec<_> = file.channels.into_iter().collect();
    tables.sort_by_key(|(_, table)| table.span().start);
    Ok(tables
        .into_iter()
        .map(|(name, table)| {
            let keys = table.into_inner().try_into().map_err(|e| e.to_string());
            (name, keys)
        })
        .collect())
}

/// 32, 37, 125: the channels of `component` that follow each publish,
/// in the policy file's order at the landed revision: each table with
/// `on_each_publish = true` whose `component` names it, or naming none
/// where it is the one declared. A following table that names no
/// component among several, names one not declared, names `bound_to`,
/// or cannot be read is unreadable naming the table (design D6).
pub fn following(
    history: &History,
    landed: &Landed,
    declared: &Declared,
    component: &ComponentName,
) -> Answer<Vec<(ChannelName, Answer<()>)>> {
    let mut following = Vec::new();
    for (name, keys) in channel_tables(history, landed)? {
        let what = format!(
            "[channels.{name}] in {POLICY_FILE} at {}",
            landed.revision()
        );
        let unreadable =
            |said: String| Err(Not::unreadable(&Source::history(LOCAL), &what, Some(said)));
        let keys = match keys {
            Ok(keys) => keys,
            Err(said) => {
                following.push((name, unreadable(said)));
                continue;
            }
        };
        if !keys.on_each_publish {
            continue;
        }
        let answer = match (&keys.component, declared.components.as_slice()) {
            (Some(named), _) if !declared.components.iter().any(|c| c.name == *named) => {
                unreadable(format!(
                    "it names the component {named}, which {POLICY_FILE} does not declare"
                ))
            }
            (Some(named), _) if named != component => continue,
            (Some(_), _) | (None, [_]) => match &keys.bound_to {
                Some(bound) => unreadable(format!(
                    "it follows each publish and is bound to {bound}, and a bound channel takes no version from a publish (37)"
                )),
                None => Ok(()),
            },
            (None, several) => unreadable(format!(
                "it follows each publish and names no component, and several are declared: {}",
                several
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        };
        following.push((name, answer));
    }
    Ok(following)
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
/// and lets claim acceptance; none where no policy names one (rule
/// core.7).
fn gate_named(
    seams: &Seams,
    history: &History,
    at_head: &Landed,
    acceptance: &Required,
    gate: &SignerId,
) -> Answer<Option<NamedSigner>> {
    signer_named(seams, history, at_head, acceptance, |s| {
        s.id == *gate && s.kinds.contains(&ClaimKind::Acceptance)
    })
}

/// 118: whether the policy at the head names a gate: some signer it
/// requires a claim of, for acceptance or for any channel, may claim
/// acceptance (design D7).
fn names_a_gate(
    seams: &Seams,
    history: &History,
    at_head: &Landed,
    acceptance: &Required,
) -> Answer<bool> {
    Ok(signer_named(seams, history, at_head, acceptance, |s| {
        s.kinds.contains(&ClaimKind::Acceptance)
    })?
    .is_some())
}

/// The first signer the policy at the head names that `fits`, looked for
/// in what acceptance requires and then in what each channel the policy
/// file declares requires, since a channel is what asks for the gate's
/// acceptance; none where no policy names one. A channel with no policy
/// is passed over, and one that cannot be read is answered.
fn signer_named(
    seams: &Seams,
    history: &History,
    at_head: &Landed,
    acceptance: &Required,
    fits: impl Fn(&NamedSigner) -> bool,
) -> Answer<Option<NamedSigner>> {
    let named = |required: &Required| {
        required
            .claims()
            .iter()
            .flat_map(|c| &c.signers)
            .find(|s| fits(s))
            .cloned()
    };
    if let Some(found) = named(acceptance) {
        return Ok(Some(found));
    }
    for (channel, _) in channel_tables(history, at_head)? {
        match required_at(seams, at_head, &Audience::Channel(channel)) {
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
/// provenance and acceptance, the one host that offers and accepts; the
/// other is the gate's own identity where the policy names no gate (118).
fn offers_and_accepts(wanted: &RequiredClaim, signer: &SignerId) -> bool {
    wanted.signers.iter().any(|s| {
        s.id == *signer
            && s.kinds.contains(&ClaimKind::Provenance)
            && s.kinds.contains(&ClaimKind::Acceptance)
    })
}

/// One version of `component` being judged by the gate signing as
/// `gate`, under a policy that names a gate or none: its revision, its
/// index, the claims attached to the index, and each artifact in the
/// index's order with the claims attached to it.
struct Judging<'a> {
    seams: &'a Seams,
    gate: &'a SignerId,
    names_a_gate: bool,
    component: &'a ComponentName,
    version: &'a Version,
    revision: &'a Revision,
    index: &'a Digest,
    on_index: Vec<Envelope>,
    artifacts: Vec<(Digest, Vec<Envelope>)>,
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
        for (artifact, own) in &self.artifacts {
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
    /// the policy lets that signer claim acceptance too (27, 96, 108) or
    /// names no gate (118), so every provenance is read before the first
    /// that counts is answered.
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
                if maker == *self.gate && self.names_a_gate && !offers_and_accepts(wanted, &maker) {
                    return Ok(Err(WhyWanting::OfferedByGate { signer: maker }));
                }
                first.get_or_insert((told.envelope, maker));
            }
        }
        Ok(first.ok_or_else(|| near.unwrap_or(WhyWanting::Missing)))
    }

    /// 33, 35, 39, 105: `channel` judged under its own policy at the head
    /// and, where satisfied, moved to the index: a promotion in the one
    /// body every promotion holds, signed as the gate naming the chain's
    /// head as its predecessor, appended to the chain, attached to the
    /// index, and only then the channel's name in the store moved from
    /// where the old head left it (rules core.6, core.15, core.34). A
    /// refused append, a fork or any other answer is the channel's, and
    /// nothing after it is done or retried.
    fn promote(&self, at_head: &Landed, channel: &ChannelName, when: u64) -> Answer<Moved> {
        // 1: the channel's own policy at the head (107).
        let audience = Audience::Channel(channel.clone());
        let required = required_at(self.seams, at_head, &audience)?;
        if let Some(bound) = required.bound_to() {
            return Err(Not::unreadable(
                &Source::new(Seam::Policy, BOUND),
                format!("the policy for channel {channel} at {}", at_head.revision()),
                Some(format!(
                    "{channel} is bound to {bound}, and a bound channel takes no version from a publish (37)"
                )),
            ));
        }
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

        // 3: the promotion, signed as the gate, asked by the rule that
        // each publish moves the channel (33, 39, 95).
        let promotion = Promotion {
            channel: channel.clone(),
            component: self.component.clone(),
            to: tailrace_seams::Moved {
                version: self.version.clone(),
                index: self.index.clone(),
            },
            from,
            predecessor: head.map(|(digest, _)| digest),
            asked_by: SignerId::new("each publish"),
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
