//! `tailrace promote`: a channel moved to an accepted version of its
//! component by a promotion claim the host's signer makes, judged under
//! the channel's policy and its binding, held by the ledger before
//! anything is written to the store, and undone by a promotion back; the
//! store rewritten from the ledger; and a channel read from the ledger
//! alone (9, 31 to 39, 58, 101, 105, 106, 109, rules core.4, core.6,
//! core.7, core.15, core.16, core.34; design D4 to D8).
//!
//! The chain is reached through the ledger seam alone, and nothing here
//! names how a ledger holds it, a ref, a table or the store's own claims
//! (rules core.1, core.2). The channel, how it moves and who may sign its
//! promotion are learned through the policy seam alone, its channels
//! answer and the policy for the channel (131). A chain the ledger
//! answers as last read is never moved from, and moves nothing in the
//! store (60, 61).

use serde::{Serialize, Serializer};
use tailrace_seams::{
    channel_name, Answer, Audience, Branch, Chain, ChannelId, ChannelName, Claim, ClaimKind,
    ComponentName, DeclaredChannel, Digest, Envelope, Keeper, Moved, Name, NamedSigner, Not, Read,
    Required, RequiredClaim, Satisfied, Seam, Seams, SignerId, Source, Store, Version, Why,
};

pub use tailrace_seams::Promotion;

use crate::{
    answer_json, channels_at, chosen, follow, merged, policy_at, tell, Declared, Followed, History,
    Landed, Provenance, BOUND, LOCAL,
};

/// Where a promotion moves its channel: to a version of its component,
/// or back to the version the channel's head moved from (34).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum To {
    Version(Version),
    Back,
}

/// What a promotion is asked: the channel and where it moves it, of the
/// one component named or the only one declared, the branch whose landed
/// head holds the policy, the version the asker expects the channel at
/// where it expects one (35), the identity of the party running it, which
/// signs and is recorded as who asked (33, 79; design D7), and the moment
/// of the act, in seconds since the Unix epoch, so the core reads no
/// clock.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PromoteAsk {
    pub branch: Branch,
    pub component: Option<ComponentName>,
    pub channel: ChannelName,
    pub to: To,
    pub from: Option<Version>,
    pub signer: SignerId,
    pub at: u64,
}

/// 33, 57: what a promotion answers, with how the store was brought to
/// the ledger beside it, which a store write that failed after the
/// ledger held the promotion does not undo.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "promoted", rename_all = "snake_case")]
pub enum Promoted {
    /// The ledger holds `claim`, the promotion of `channel` `from` the
    /// version its head was at, none for its first, `to` the version
    /// asked, after `predecessor`, under `policy`.
    Moved {
        channel: ChannelName,
        component: ComponentName,
        to: Moved,
        from: Option<Moved>,
        claim: Digest,
        predecessor: Option<Digest>,
        policy: Satisfied,
        #[serde(serialize_with = "one_answer")]
        store: Answer<Mirrored>,
    },
    /// The channel's head is at `at` already, and nothing was appended.
    There {
        channel: ChannelName,
        component: ComponentName,
        at: Moved,
        #[serde(serialize_with = "one_answer")]
        store: Answer<Mirrored>,
    },
}

/// 106, 109: the store brought to a channel's chain: the envelopes of
/// the claims it did not hold, attached to the indexes they are about,
/// and how the channel's name followed the head.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Mirrored {
    pub attached: Vec<Digest>,
    pub name: Followed,
}

/// 31, 101: a channel as its chain holds it: empty, which is an answer
/// and not an error, or at its head's version, with every promotion in
/// order by its envelope's digest and how the chain was read (60).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Channel {
    Empty,
    At {
        head: Promotion,
        chain: Vec<(Digest, Promotion)>,
        read: Read,
    },
}

/// 57: the store's answer in the one form every surface renders.
fn one_answer<S: Serializer>(answer: &Answer<Mirrored>, serializer: S) -> Result<S::Ok, S::Error> {
    answer_json(answer).serialize(serializer)
}

fn keeper() -> Source {
    Source::new(Seam::Keeper, BOUND)
}

/// 125, 131: the channel named `name` of `component` as the policy lists
/// it at `landed`; absent naming both where it lists none.
pub(crate) fn listed(
    seams: &Seams,
    landed: &Landed,
    name: &ChannelName,
    component: &ComponentName,
) -> Answer<DeclaredChannel> {
    channels_at(seams.policy.as_ref(), landed)?
        .into_iter()
        .find(|c| c.channel.name == *name && c.channel.component_name() == *component)
        .ok_or_else(|| {
            Not::absent(
                &Source::new(Seam::Policy, BOUND),
                format!("channel {name} of {component} at {}", landed.revision()),
            )
        })
}

/// The claims of `channel`'s chain as the ledger holds them now, none
/// for a channel never promoted, refused naming a fork as the ledger
/// answers it (105); unreadable naming the chain, with the ledger's
/// words, where the ledger answered it as last read, whose head may be
/// behind the one held (60, rule core.6).
pub(crate) fn held(ledger: &dyn Keeper, channel: &ChannelId) -> Answer<Vec<Envelope>> {
    match ledger.chain(channel) {
        Ok(Chain {
            read: Read::Kept { said, .. },
            ..
        }) => Err(Not::unreadable(
            &keeper(),
            format!("the chain of {channel}"),
            Some(said),
        )),
        Ok(chain) => Ok(chain.claims),
        Err(Not::Absent(_)) => Ok(Vec::new()),
        Err(not) => Err(not),
    }
}

/// The promotion `envelope` holds, unreadable naming its digest where it
/// holds no promotion claim of tailrace's form.
pub(crate) fn promotion_of(channel: &ChannelId, envelope: &Envelope) -> Answer<Promotion> {
    Claim::from_envelope(envelope)
        .as_ref()
        .and_then(Promotion::of)
        .ok_or_else(|| {
            let digest = envelope.digest();
            Not::unreadable(
                &keeper(),
                format!("the claim {digest} of the chain of {channel}"),
                Some(format!(
                    "{digest} holds no promotion claim of tailrace's form"
                )),
            )
        })
}

/// 31 to 39, 105: the promotion `ask` names, in the order of design D4:
/// the channel found among those the policy lists for the component
/// (125, 131), its policy read at the branch's landed head, the chain's
/// head read
/// from the ledger itself, the version's name resolved in the store
/// before anything of it is read (rule core.4), the channel moved only
/// from where it is (35) and, where bound, only to a version its bound
/// channel held (37), judged (D5), signed, and appended naming the head
/// as predecessor before anything is written to the store (33, rule
/// core.15). A refusal at any step is the answer, nothing after it is
/// done, and nothing is retried (rules core.6, core.34).
pub fn promote(seams: &Seams, history: &History, ask: &PromoteAsk) -> Answer<Promoted> {
    // 1: the branch's head, landed (65, 77), and the component it
    // declares.
    let head = history.head(&ask.branch)?;
    let at_head = Landed::check(seams.landing.as_ref(), &ask.branch, &head)?;
    let declared = Declared::read(history, &at_head)?;
    let component = chosen(&declared, ask.component.as_ref(), &head)?;

    // 2: the channel among those the policy lists for the component,
    // absent naming both where it lists none (125, 131), and its policy
    // at the head; absent where none is declared for it (82).
    let listed = listed(seams, &at_head, &ask.channel, &component.name)?;
    let channel = &listed.channel;
    let audience = Audience::Channel(channel.clone());
    let required =
        policy_at(seams.policy.as_ref(), &at_head, &audience).map_err(|not| match not {
            Not::Absent(a) => Not::absent(&a.source, format!("{} at {head}", a.what)),
            other => other,
        })?;

    // 3: the chain, read from the ledger itself, and its head (105, 121).
    let chain = held(seams.keeper.as_ref(), channel)?;
    let last = match chain.last() {
        Some(envelope) => Some((envelope.digest(), promotion_of(channel, envelope)?)),
        None => None,
    };
    let at = last.as_ref().map(|(_, p)| p.to.clone());

    // 4: the version, its name resolved in the store (rule core.4), and
    // its tag's revision, landed (14).
    let version = match &ask.to {
        To::Version(version) => version.clone(),
        To::Back => last
            .as_ref()
            .and_then(|(_, p)| p.from.as_ref())
            .map(|from| from.version.clone())
            .ok_or_else(|| Not::absent(&keeper(), format!("a version {channel} moved from")))?,
    };
    let tag = format!("{}{}", component.prefix, version);
    let index = seams.store.resolve(&Name::new(&tag))?;
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
    let to = Moved { version, index };

    // 5: the version the asker expects the channel at (35, S7).
    if let Some(from) = &ask.from {
        if at.as_ref().map(|m| &m.version) != Some(from) {
            return Err(Not::refused(
                &keeper(),
                Why::ChannelAt {
                    channel: channel.clone(),
                    at,
                },
                None,
            ));
        }
    }

    // 6: the channel there already: nothing appended, the store brought
    // to the ledger (106).
    if at.as_ref().is_some_and(|m| m.index == to.index) {
        return Ok(Promoted::There {
            channel: ask.channel.clone(),
            component: component.name.clone(),
            at: to,
            store: mirror(seams.store.as_ref(), seams.keeper.as_ref(), channel),
        });
    }

    // 7: a bound channel takes only what its bound channel, of its own
    // component, held (37, 83, 125).
    if let Some(bound_to) = listed.bound_to() {
        if !held_by(seams.keeper.as_ref(), &bound_to, &to.index)? {
            return Err(Not::refused(
                &keeper(),
                Why::NeverHeld {
                    channel: channel.clone(),
                    bound_to,
                },
                None,
            ));
        }
    }

    // 8: judged, and the host's signer one the channel's policy names
    // among its promoters (9, 31, 32).
    let claims = judge(seams, &required, &to.index)?;
    may_promote(&required, &ask.signer)?;

    // 9: the claim, naming the head as its predecessor (33, 39, 105),
    // signed as publish signs.
    let promotion = Promotion {
        channel: channel.name.clone(),
        component: component.name.clone(),
        to: to.clone(),
        from: at.clone(),
        predecessor: last.map(|(digest, _)| digest),
        asked_by: ask.signer.clone(),
        policy: Satisfied {
            revision: head.clone(),
            claims,
        },
        at: ask.at,
    };
    let signer = seams
        .signers
        .first()
        .ok_or_else(|| Not::absent(&Source::new(Seam::Signer, BOUND), "signer bound"))?;
    let envelope = signer.sign(&promotion.claim(revision), &ask.signer)?;

    // 10: held by the ledger before the store is written, never retried
    // (33, 120, rules core.6, core.34).
    let claim = seams
        .keeper
        .append(channel, &envelope, promotion.predecessor.as_ref())?;

    // 11: the store brought to the ledger, its answer beside (39, 106,
    // 109).
    let store = mirror(seams.store.as_ref(), seams.keeper.as_ref(), channel);
    Ok(Promoted::Moved {
        channel: promotion.channel,
        component: promotion.component,
        to: promotion.to,
        from: promotion.from,
        claim,
        predecessor: promotion.predecessor,
        policy: promotion.policy,
        store,
    })
}

/// 37, 83: whether `index` is the subject of a promotion in `bound_to`'s
/// chain, whether or not the channel is still at it. A chain the ledger
/// answers as last read serves, since a chain only grows (122).
fn held_by(ledger: &dyn Keeper, bound_to: &ChannelId, index: &Digest) -> Answer<bool> {
    let claims = match ledger.chain(bound_to) {
        Ok(chain) => chain.claims,
        Err(Not::Absent(_)) => Vec::new(),
        Err(not) => return Err(not),
    };
    Ok(claims
        .iter()
        .filter_map(Claim::from_envelope)
        .filter_map(|claim| Promotion::of(&claim))
        .any(|promotion| promotion.to.index == *index))
}

fn wanting(wanted: &RequiredClaim) -> Not {
    Not::refused(
        &Source::new(Seam::Policy, BOUND),
        Why::Wanting {
            kind: wanted.kind,
            check: wanted.check.clone(),
            signers: wanted.signers.iter().map(|s| s.id.clone()).collect(),
        },
        None,
    )
}

/// The signers of `signers` that may claim `kind`, as a claim required.
fn of_kind(signers: &[NamedSigner], kind: ClaimKind) -> RequiredClaim {
    RequiredClaim {
        kind,
        check: None,
        signers: signers
            .iter()
            .filter(|s| s.kinds.contains(&kind))
            .cloned()
            .collect(),
    }
}

/// 9, 31, 58: the digests of the envelopes on `index` that satisfy, in
/// order, an acceptance that counts under the channel's policy, whether
/// or not the policy lists one, then each claim the policy requires, its
/// check where it is a verdict; refused [`Why::Wanting`] naming the first
/// found wanting and the signers that may make it. Each attestation is
/// told as verify tells it, under the signers the policy names merged as
/// verify merges them (rule core.7), and counts for a claim required only
/// when made by a signer that claim names. Only the index's claims are
/// judged: a claim about a version is a claim about its index (7).
pub fn judge(seams: &Seams, required: &Required, index: &Digest) -> Answer<Vec<Digest>> {
    let signers = merged(required);
    let mut told = Vec::new();
    for envelope in seams.store.claims_of(index)? {
        let claim = Claim::from_envelope(&envelope);
        let one = tell(
            seams,
            &signers,
            index,
            &envelope,
            claim.as_ref().map(|c| c.kind),
            None,
            None,
        )?;
        let carried_in = one.envelope.clone();
        told.push(one);
        let Some(provenance) = claim.as_ref().and_then(Provenance::of) else {
            continue;
        };
        for carried in &provenance.carried {
            told.push(tell(
                seams,
                &signers,
                index,
                &carried.envelope,
                Some(ClaimKind::Verdict),
                Some(carried.check.clone()),
                Some(carried_in.clone()),
            )?);
        }
    }

    let acceptance = of_kind(&signers, ClaimKind::Acceptance);
    let mut satisfied: Vec<Digest> = Vec::new();
    for wanted in std::iter::once(&acceptance).chain(required.claims()) {
        let found = told.iter().find(|t| {
            t.counts
                && t.kind == Some(wanted.kind)
                && (wanted.kind != ClaimKind::Verdict || t.check == wanted.check)
                && t.made_by
                    .as_ref()
                    .is_some_and(|maker| wanted.signers.iter().any(|s| s.id == *maker))
        });
        match found {
            Some(t) if satisfied.contains(&t.envelope) => {}
            Some(t) => satisfied.push(t.envelope.clone()),
            None => return Err(wanting(wanted)),
        }
    }
    Ok(satisfied)
}

/// 9, 32: `signer` is among the signers the channel's policy names as
/// its promoters (rule core.7); refused [`Why::Wanting`] a promotion,
/// naming them, otherwise.
pub(crate) fn may_promote(required: &Required, signer: &SignerId) -> Answer<()> {
    if required.promoted_by().iter().any(|s| s.id == *signer) {
        Ok(())
    } else {
        Err(wanting(&RequiredClaim {
            kind: ClaimKind::Promotion,
            check: None,
            signers: required.promoted_by().to_vec(),
        }))
    }
}

/// 106, 109: the store brought to `channel`'s chain as the ledger holds
/// it: each claim of the chain not attached to the index it is about
/// attached, then the channel's name moved to the head's index from the
/// index the head moved from, a race lost on it reported and repaired
/// (rule core.6). The ledger is written nothing, and a chain it answers
/// as last read writes nothing to the store (60).
pub fn mirror(store: &dyn Store, ledger: &dyn Keeper, channel: &ChannelId) -> Answer<Mirrored> {
    let mut attached = Vec::new();
    let mut head = None;
    for envelope in held(ledger, channel)? {
        let promotion = promotion_of(channel, &envelope)?;
        let digest = envelope.digest();
        let on = store.claims_of(&promotion.to.index)?;
        if !on.iter().any(|e| e.digest() == digest) {
            store.attach(&promotion.to.index, &envelope)?;
            attached.push(digest);
        }
        head = Some(promotion);
    }
    let Some(head) = head else {
        return Err(Not::absent(&keeper(), format!("chain of {channel}")));
    };
    let expected = head.from.as_ref().map(|from| &from.index);
    let name = follow(store, ledger, channel, &channel_name(channel), expected)?;
    Ok(Mirrored { attached, name })
}

/// 31, 60, 101: `channel` as its chain holds it, asked of the ledger
/// alone: nothing of the store is read, and nothing is written (rule
/// core.32). A chain the ledger answers as last read is answered so,
/// saying when and with its words; a claim of it holding no promotion of
/// tailrace's form is unreadable naming its digest.
pub fn channel(ledger: &dyn Keeper, channel: &ChannelId) -> Answer<Channel> {
    let chain = match ledger.chain(channel) {
        Ok(chain) => chain,
        Err(Not::Absent(_)) => return Ok(Channel::Empty),
        Err(not) => return Err(not),
    };
    let mut read = Vec::new();
    for envelope in &chain.claims {
        read.push((envelope.digest(), promotion_of(channel, envelope)?));
    }
    Ok(match read.last() {
        None => Channel::Empty,
        Some((_, head)) => Channel::At {
            head: head.clone(),
            chain: read,
            read: chain.read,
        },
    })
}
