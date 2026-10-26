//! The ledger seam (B.8) held in the store (C.14): a channel's chain is
//! the promotion claims of that channel and component attached to the
//! indexes they move to, walked back by predecessor from the index the
//! channel's name in the store names, `channel/<channel>` where no
//! component is declared and `channel/<component>/<channel>` where one
//! is, through the store seam alone (105, 120 to 122, 125, rule core.2;
//! design D10).
//!
//! Each chain read from the store is kept, with when it was read, under
//! the repository's common git directory beside the record of published
//! versions, and is answered as last read where the store does not
//! answer, saying when, with the store's words; append is then
//! unreadable (60 to 62). What is kept is the chain alone: no
//! credential, and nothing else the store holds (64).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tailrace_seams::{
    channel_name, Answer, Chain, ChannelId, Claim, Digest, Envelope, Keeper, Not, Promotion, Read,
    Seam, Source, Store, Unreadable, Why,
};

/// The ledger seam held in a store, its chains kept in a directory of
/// the host's.
pub struct InStore {
    store: Box<dyn Store>,
    kept: PathBuf,
}

fn source() -> Source {
    Source::new(Seam::Keeper, "store")
}

/// A promotion claim of the chain, with its body.
#[derive(Clone)]
struct Link {
    envelope: Envelope,
    promotion: Promotion,
}

impl Link {
    fn digest(&self) -> Digest {
        self.envelope.digest()
    }
}

/// A chain as this host last read it from the store, and when, in
/// seconds since the Unix epoch.
#[derive(Serialize, Deserialize)]
struct Kept {
    at: u64,
    claims: Vec<Envelope>,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// 105: the fork of two walks neither of which holds the other: the claim
/// both follow, where they share one, and the first claim of each after
/// it.
fn fork(a: &[Link], b: &[Link]) -> Not {
    let shared = a
        .iter()
        .zip(b)
        .take_while(|(x, y)| x.digest() == y.digest())
        .count();
    Not::refused(
        &source(),
        Why::Fork {
            predecessor: shared.checked_sub(1).map(|i| a[i].digest()),
            claims: vec![a[shared].digest(), b[shared].digest()],
        },
        None,
    )
}

impl InStore {
    /// The chains held in `store`, kept on this host in `kept`.
    pub fn new(store: Box<dyn Store>, kept: impl Into<PathBuf>) -> InStore {
        InStore {
            store,
            kept: kept.into(),
        }
    }

    /// The promotion claims of `channel` attached to `index`, each naming
    /// its name and its component (125), in the order attached.
    fn on(&self, channel: &ChannelId, index: &Digest) -> Answer<Vec<Link>> {
        Ok(self
            .store
            .claims_of(index)?
            .into_iter()
            .filter_map(|envelope| {
                let promotion = Claim::from_envelope(&envelope)
                    .as_ref()
                    .and_then(Promotion::of)?;
                (promotion.channel == channel.name
                    && promotion.component == channel.component_name())
                .then_some(Link {
                    envelope,
                    promotion,
                })
            })
            .collect())
    }

    /// `head` walked back by the index each claim moved from and the
    /// digest of the claim it follows, until a claim that follows none:
    /// the chain to it, from the first. A predecessor not attached to its
    /// index is unreadable naming both digests; two claims met on one
    /// index naming one predecessor are a fork (105).
    fn walk_back(&self, channel: &ChannelId, head: Link) -> Answer<Vec<Link>> {
        let mut walk = vec![head];
        loop {
            let at = &walk[walk.len() - 1].promotion;
            let Some(predecessor) = at.predecessor.clone() else {
                break;
            };
            let missing = |index: &str| {
                Not::unreadable(
                    &source(),
                    format!("the claim {predecessor} on {index}"),
                    Some(format!(
                        "the chain of {channel} names {predecessor} as a predecessor on {index}, which does not hold it"
                    )),
                )
            };
            let Some(from) = at.from.clone() else {
                return Err(missing("no index"));
            };
            let on = match self.on(channel, &from.index) {
                Ok(on) => on,
                Err(Not::Absent(_)) => return Err(missing(from.index.as_str())),
                Err(not) => return Err(not),
            };
            let Some(found) = on.iter().find(|l| l.digest() == predecessor).cloned() else {
                return Err(missing(from.index.as_str()));
            };
            if let Some(other) = on.iter().find(|l| {
                l.digest() != predecessor && l.promotion.predecessor == found.promotion.predecessor
            }) {
                return Err(Not::refused(
                    &source(),
                    Why::Fork {
                        predecessor: found.promotion.predecessor.clone(),
                        claims: vec![found.digest(), other.digest()],
                    },
                    None,
                ));
            }
            walk.push(found);
        }
        walk.reverse();
        Ok(walk)
    }

    /// 121, C.14: the chain of `channel` as the store holds it now,
    /// walked from the index the channel's name names; `None` where the
    /// name names nothing. Of the channel's claims on that index, the
    /// head is the one whose walk holds every other; two neither of which
    /// holds the other are a fork (105, 122).
    fn walk(&self, channel: &ChannelId) -> Answer<Option<Vec<Link>>> {
        let name = channel_name(channel);
        let index = match self.store.resolve(&name) {
            Ok(index) => index,
            Err(Not::Absent(_)) => return Ok(None),
            Err(not) => return Err(not),
        };
        let candidates = self.on(channel, &index)?;
        if candidates.is_empty() {
            return Err(Not::unreadable(
                &source(),
                format!("the chain of {channel} on {index}"),
                Some(format!(
                    "{name} names {index}, which holds no promotion of {channel}"
                )),
            ));
        }
        let mut walks = Vec::new();
        for candidate in candidates {
            walks.push(self.walk_back(channel, candidate)?);
        }
        let holds = |walk: &[Link], other: &[Link]| {
            walk.iter()
                .any(|l| l.digest() == other[other.len() - 1].digest())
        };
        if let Some(i) = walks
            .iter()
            .position(|walk| walks.iter().all(|other| holds(walk, other)))
        {
            return Ok(Some(walks.swap_remove(i)));
        }
        for (i, a) in walks.iter().enumerate() {
            for b in &walks[i + 1..] {
                if !holds(a, b) && !holds(b, a) {
                    return Err(fork(a, b));
                }
            }
        }
        unreachable!("walks of which none holds every other hold two that part")
    }

    /// Where the chain of `channel` is kept: a file of its spelling,
    /// under a directory of its component where it has one.
    fn path(&self, channel: &ChannelId) -> PathBuf {
        self.kept.join(format!("{}.json", channel.spelled()))
    }

    fn cannot(path: &Path, e: io::Error) -> Not {
        Not::unreadable(
            &Source::host(path),
            path.display().to_string(),
            Some(e.to_string()),
        )
    }

    /// 61: `claims` kept as the chain of `channel` last read, now, written
    /// beside the last and renamed over it, so a reader never sees half a
    /// chain.
    fn keep(&self, channel: &ChannelId, claims: &[Envelope]) -> Answer<()> {
        let path = self.path(channel);
        let dir = path.parent().unwrap_or(&self.kept);
        fs::create_dir_all(dir).map_err(|e| Self::cannot(dir, e))?;
        let mut text = serde_json::to_string_pretty(&Kept {
            at: now(),
            claims: claims.to_vec(),
        })
        .expect("a chain is plain data");
        text.push('\n');
        let beside = dir.join(format!(".{}.json.{}", channel.name, std::process::id()));
        fs::write(&beside, text).map_err(|e| Self::cannot(&beside, e))?;
        fs::rename(&beside, &path).map_err(|e| {
            let _ = fs::remove_file(&beside);
            Self::cannot(&path, e)
        })
    }

    /// 60, C.14: the store not answering, for `failed`, its own words:
    /// the chain of `channel` as kept, saying when; unreadable with the
    /// store's words where nothing is kept.
    fn kept(&self, channel: &ChannelId, failed: Unreadable) -> Answer<Chain> {
        let path = self.path(channel);
        let text = match fs::read(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(Not::unreadable(
                    &source(),
                    format!("the chain of {channel}"),
                    failed.said,
                ));
            }
            Err(e) => return Err(Self::cannot(&path, e)),
        };
        let kept: Kept = serde_json::from_slice(&text).map_err(|e| {
            Not::unreadable(
                &Source::host(&path),
                path.display().to_string(),
                Some(e.to_string()),
            )
        })?;
        Ok(Chain {
            claims: kept.claims,
            read: Read::Kept {
                at: kept.at,
                said: failed.said.unwrap_or_default(),
            },
        })
    }

    /// The chain of `channel` from the store, kept as read; from what is
    /// kept where the store does not answer. Empty where the channel's
    /// name names nothing.
    fn read(&self, channel: &ChannelId) -> Answer<Chain> {
        match self.walk(channel) {
            Ok(walked) => {
                let claims: Vec<Envelope> =
                    walked.into_iter().flatten().map(|l| l.envelope).collect();
                if !claims.is_empty() {
                    self.keep(channel, &claims)?;
                }
                Ok(Chain {
                    claims,
                    read: Read::Source,
                })
            }
            Err(Not::Unreadable(u)) if u.source.seam == Seam::Store => self.kept(channel, u),
            Err(not) => Err(not),
        }
    }
}

impl Keeper for InStore {
    /// 120: held when `predecessor` is the head of the chain as walked now
    /// and the claim's own, and the claim moves from the head's index:
    /// the claim attached to the index it is about, then the channel's
    /// name moved there from the head's index (39, 70). A move the store
    /// refuses is refused naming the head as walked again; the claim
    /// attached before it stays, and a walk that meets it reports the
    /// fork it is (105, 109).
    fn append(
        &self,
        channel: &ChannelId,
        promotion: &Envelope,
        predecessor: Option<&Digest>,
    ) -> Answer<Digest> {
        let chain = self.read(channel)?;
        if let Read::Kept { said, .. } = chain.read {
            return Err(Not::unreadable(
                &source(),
                format!("the chain of {channel}"),
                Some(said),
            ));
        }
        let head = chain.claims.last().map(Envelope::digest);
        let head_index = chain
            .claims
            .last()
            .and_then(Claim::from_envelope)
            .map(|c| c.subject);
        let Some(body) = Claim::from_envelope(promotion)
            .as_ref()
            .and_then(Promotion::of)
        else {
            return Err(Not::unreadable(
                &source(),
                format!("the promotion {}", promotion.digest()),
                Some(format!(
                    "{} holds no promotion claim of tailrace's form",
                    promotion.digest()
                )),
            ));
        };
        let from = body.from.as_ref().map(|f| f.index.clone());
        if head.as_ref() != predecessor || body.predecessor != head || from != head_index {
            return Err(Not::refused(&source(), Why::NotHead { head }, None));
        }
        self.store.attach(&body.to.index, promotion)?;
        match self
            .store
            .name(&channel_name(channel), &body.to.index, head_index.as_ref())
        {
            Ok(()) => Ok(promotion.digest()),
            Err(Not::Refused(r)) if matches!(r.why, Why::NameAt { .. }) => {
                let head = self.read(channel)?.claims.last().map(Envelope::digest);
                Err(Not::refused(&source(), Why::NotHead { head }, r.said))
            }
            Err(not) => Err(not),
        }
    }

    /// 121: every claim of `channel` from the first, walked in the store,
    /// or as last read where the store does not answer (60); absent where
    /// the channel's name names nothing.
    fn chain(&self, channel: &ChannelId) -> Answer<Chain> {
        let chain = self.read(channel)?;
        if chain.claims.is_empty() {
            return Err(Not::absent(&source(), format!("chain of {channel}")));
        }
        Ok(chain)
    }
}
