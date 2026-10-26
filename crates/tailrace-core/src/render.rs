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
//!
//! A layout lives at `.tailrace/layouts/<ecosystem>/` under the
//! checkout's working tree, the same place on every host, and
//! `.tailrace/` ignores itself to git (49, rule core.18). Every read of a
//! layout comes before any write: the projection renders exactly the
//! versions held, so a version refused now leaves the layout (47), and a
//! read its source cannot answer, or a render that fails, leaves the
//! layout as it was, answered from the host's record of what it holds
//! (51; design D5, D6). The record is one file per layout under the
//! checkout's own git directory, rewritten whole by each render (61 to
//! 63, rule core.20; design D7).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize, Serializer};
use serde_json::{json, Value};
use tailrace_seams::{
    Answer, Artifact, ArtifactMeta, Audience, Claim, ClaimKind, Contents, Digest, Ecosystem,
    EcosystemName, Envelope, Held, Manifest, Name, NamedSigner, Not, Pinned, Policy, RequiredClaim,
    Revision, Seam, Seams, SignerId, Source, Store, Version, Why,
};

use crate::verify::{tell, Told};
use crate::{answer_json, policy_at, History, Landed, Provenance, BOUND};

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
/// its `[layout]` or the given, never the checkout's (C.10, rule
/// core.22); none where the policy answers absent, a repository with no
/// policy of its own, so each version is judged under its publisher's;
/// the policy's own answer where it cannot say.
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

/// The directory under a checkout's working tree that holds its layouts
/// and ignores itself to git (49; design D5).
pub const TAILRACE_DIR: &str = ".tailrace";

/// 49, rule core.18: where the layout of `ecosystem` is rendered in the
/// checkout whose working tree is `tree`: `.tailrace/layouts/<ecosystem>`
/// under it, a place that names no host.
pub fn place(tree: &Path, ecosystem: &Ecosystem) -> PathBuf {
    tree.join(TAILRACE_DIR)
        .join("layouts")
        .join(ecosystem.as_str())
}

/// 113: the policy a layout was rendered under: the consumer's own at the
/// landed revision, or, where it has none, each version's publisher's,
/// which each version's answer names.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Under {
    Own { revision: Revision },
    Publisher,
}

impl Under {
    fn of(own: Option<&Own>) -> Under {
        match own {
            Some(own) => Under::Own {
                revision: own.revision.clone(),
            },
            None => Under::Publisher,
        }
    }
}

/// What render is asked: the layout's ecosystem; the pins its projection
/// read in the checkout; the consumer's own policy for layouts, none
/// where it has none; the signers the host trusts, through whom alone a
/// publisher's policy counts (C.10, 118); the checkout's working tree;
/// and when, in seconds since the Unix epoch by the asker's clock, since
/// the core reads none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderAsk {
    pub ecosystem: Ecosystem,
    pub pins: Vec<Pinned>,
    pub own: Option<Own>,
    pub trusted: Vec<NamedSigner>,
    pub tree: PathBuf,
    pub when: u64,
}

/// A pin held: the digests of the index and the artifact the layout
/// holds (rule core.4), and the expired signers whose claims counted
/// (81, 117).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeldPin {
    pub index: Digest,
    pub artifact: Digest,
    pub expired: Vec<SignerId>,
}

/// 58, 99: one pin's answer, held, absent, refused naming the claim found
/// wanting, or unreadable, and the policy that judged it, none where it
/// was not judged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinAnswer {
    pub pin: Pinned,
    pub policy: Option<PolicyOf>,
    pub answer: Answer<HeldPin>,
}

impl Serialize for PinAnswer {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut value = answer_json(&self.answer);
        value["name"] = json!(self.pin.name);
        value["version"] = json!(self.pin.version);
        value["policy"] = json!(self.policy);
        value.serialize(serializer)
    }
}

/// 47, 85: a layout rendered now: where, when, under which policy, and
/// every pin's answer in the order pinned.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Refreshed {
    pub at: String,
    pub rendered_at: u64,
    pub policy: Under,
    pub pins: Vec<PinAnswer>,
}

/// 51, S10: a layout that could not be refreshed, left as it was: where,
/// what it holds, when it was last rendered and under which policy, from
/// the host's record, none of them where it was never rendered; and
/// `because`, what could not be read, with its source's own words (58,
/// 59).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Kept {
    pub at: String,
    pub holds: Vec<HeldEntry>,
    pub rendered_at: Option<u64>,
    pub policy: Option<Under>,
    pub because: Not,
}

/// 47, 51: one layout's answer: refreshed, or kept as it was.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Refresh {
    Refreshed(Refreshed),
    Kept(Kept),
}

/// 61: one version a layout holds, as the record keeps it: its name and
/// version, its index's and its artifact's digests, and the policy that
/// judged it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeldEntry {
    pub name: EcosystemName,
    pub version: Version,
    pub index: Digest,
    pub artifact: Digest,
    pub policy: PolicyOf,
}

/// 61, 99: one version a layout's policy refused, as the record keeps it:
/// its name and version, its index's digest, the claim found wanting, and
/// the policy that refused it, none where none was found.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefusedEntry {
    pub name: EcosystemName,
    pub version: Version,
    pub index: Digest,
    pub wanting: RequiredClaim,
    pub policy: Option<PolicyOf>,
}

/// 61: the host's record of one layout: its ecosystem and place, when it
/// was rendered and under which policy, each version it holds and each
/// its policy refused. A pin the store does not name is not kept: the
/// lockfile and the store answer it again.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutRecord {
    pub ecosystem: Ecosystem,
    pub at: String,
    pub rendered_at: u64,
    pub policy: Under,
    pub held: Vec<HeldEntry>,
    pub refused: Vec<RefusedEntry>,
}

/// 61 to 63: what the host keeps of a checkout's layouts, one file per
/// layout, readable with a file reader and no tailrace process, holding
/// no credential (64), and rebuilt by the next sync (rule core.20).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layouts {
    dir: PathBuf,
}

impl Layouts {
    /// The record of the checkout `history` opens, under its own git
    /// directory, `tailrace/layouts/` there, since two worktrees pin
    /// apart and no push carries it (50, 62; design D7).
    pub fn at(history: &History) -> Layouts {
        Layouts::open(history.git_dir().join("tailrace").join("layouts"))
    }

    /// The record kept in `dir`.
    pub fn open(dir: impl Into<PathBuf>) -> Layouts {
        Layouts { dir: dir.into() }
    }

    /// The directory the record is kept in.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The file the layout of `ecosystem` is kept in.
    pub fn path(&self, ecosystem: &Ecosystem) -> PathBuf {
        self.dir.join(format!("{ecosystem}.json"))
    }

    fn source(&self) -> Source {
        Source::host(&self.dir)
    }

    fn cannot(&self, path: &Path, e: io::Error) -> Not {
        Not::unreadable(
            &self.source(),
            path.display().to_string(),
            Some(e.to_string()),
        )
    }

    /// `record` kept as its layout's file, written beside it first and
    /// renamed over what was there, so a reader never sees half a record.
    pub fn write(&self, record: &LayoutRecord) -> Answer<()> {
        let path = self.path(&record.ecosystem);
        fs::create_dir_all(&self.dir).map_err(|e| self.cannot(&self.dir, e))?;
        let mut text = serde_json::to_string_pretty(record).expect("a record is plain data");
        text.push('\n');
        let beside = self
            .dir
            .join(format!(".{}.json.{}", record.ecosystem, std::process::id()));
        fs::write(&beside, text).map_err(|e| self.cannot(&beside, e))?;
        fs::rename(&beside, &path).map_err(|e| {
            let _ = fs::remove_file(&beside);
            self.cannot(&path, e)
        })
    }

    /// The record of the layout of `ecosystem`: absent naming its file
    /// where none is kept; unreadable naming the file, with the parser's
    /// words where it is no record (58, 59).
    pub fn read(&self, ecosystem: &Ecosystem) -> Answer<LayoutRecord> {
        let path = self.path(ecosystem);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(Not::absent(&self.source(), path.display().to_string()));
            }
            Err(e) => return Err(self.cannot(&path, e)),
        };
        serde_json::from_str(&text).map_err(|e| {
            Not::unreadable(
                &self.source(),
                path.display().to_string(),
                Some(e.to_string()),
            )
        })
    }
}

/// The attestations attached to `digest`, as the pin's own unreadable
/// answer where the store holds no such digest.
fn claims_of(store: &dyn Store, digest: &Digest) -> Pinning<Vec<Envelope>> {
    match store.claims_of(digest) {
        Ok(envelopes) => Ok(Ok(envelopes)),
        Err(Not::Absent(_)) => Ok(Err(Not::unreadable(
            &Source::new(Seam::Store, BOUND),
            format!("the claims of {digest}"),
            Some(format!("{digest} is not held")),
        ))),
        Err(not) => Err(not),
    }
}

/// `pin` read and judged, writing nothing: its answer, and the artifact
/// the layout takes where it is held.
fn judged(seams: &Seams, ask: &RenderAsk, pin: &Pinned) -> Answer<(PinAnswer, Option<Artifact>)> {
    let answered = |policy: Option<PolicyOf>, not: Not| {
        let answer = PinAnswer {
            pin: pin.clone(),
            policy,
            answer: Err(not),
        };
        Ok((answer, None))
    };
    let store = seams.store.as_ref();
    let located = match locate(store, &ask.ecosystem, pin)? {
        Ok(located) => located,
        Err(not) => return answered(None, not),
    };
    let on_index = match claims_of(store, &located.index)? {
        Ok(envelopes) => envelopes,
        Err(not) => return answered(None, not),
    };
    let on_artifact = match claims_of(store, &located.artifact.digest)? {
        Ok(envelopes) => envelopes,
        Err(not) => return answered(None, not),
    };
    let meta = &located.artifact.meta;
    let own = ask.own.as_ref();
    let (policy, required) =
        match policy_for(seams, own, &ask.trusted, meta, &located.index, &on_index)? {
            Ok(found) => found,
            Err(not) => return answered(None, not),
        };
    match judge(seams, &required, &located, &on_index, &on_artifact)? {
        Ok(expired) => {
            let held = HeldPin {
                index: located.index.clone(),
                artifact: located.artifact.digest.clone(),
                expired,
            };
            let answer = PinAnswer {
                pin: pin.clone(),
                policy: Some(policy),
                answer: Ok(held),
            };
            Ok((answer, Some(located.artifact)))
        }
        Err(not) => answered(Some(policy), not),
    }
}

fn written(path: &Path, e: io::Error) -> Not {
    Not::unreadable(
        &Source::host(path),
        path.display().to_string(),
        Some(e.to_string()),
    )
}

/// `.tailrace/` under `tree`, with `.gitignore` holding `*` written where
/// it holds none, so the layouts ignore themselves to git and the
/// consumer's repository needs no line of its own (design D5).
fn ignored(tree: &Path) -> Answer<()> {
    let dir = tree.join(TAILRACE_DIR);
    let ignore = dir.join(".gitignore");
    if ignore.is_file() {
        return Ok(());
    }
    fs::create_dir_all(&dir).map_err(|e| written(&dir, e))?;
    fs::write(&ignore, "*\n").map_err(|e| written(&ignore, e))
}

/// 47 to 50, 85, 113: the layout `ask` names, rendered from the pins its
/// policy admits and no other (design D5, D6). Every pin is resolved,
/// its claims read and judged before anything is written; a pin is
/// asked of the store by its own name alone (50, S4). A read its source
/// answers unreadable or refused is the answer, and nothing is written.
/// Then `.tailrace/` is made to ignore itself, the bound projection
/// renders exactly the held versions' artifacts at [`place`], so a
/// version refused now leaves the layout (47), and the layout's record
/// is written; a render or a record that fails is the answer.
pub fn render(seams: &Seams, layouts: &Layouts, ask: &RenderAsk) -> Answer<Refreshed> {
    let projection = seams.projection(&ask.ecosystem).ok_or_else(|| {
        Not::absent(
            &Source::new(Seam::Host, BOUND),
            format!("projection for {}", ask.ecosystem),
        )
    })?;

    // 1: every pin read and judged, writing nothing.
    let mut pins: Vec<PinAnswer> = Vec::new();
    let mut held: Vec<Artifact> = Vec::new();
    for pin in &ask.pins {
        if pins.iter().any(|p| p.pin == *pin) {
            continue;
        }
        let (answer, artifact) = judged(seams, ask, pin)?;
        pins.push(answer);
        held.extend(artifact);
    }

    // 2: the place, the render of exactly what is held, then the record.
    let at = place(&ask.tree, &ask.ecosystem).display().to_string();
    ignored(&ask.tree)?;
    projection.render(Contents::Artifacts(&held), &at)?;
    let policy = Under::of(ask.own.as_ref());
    layouts.write(&LayoutRecord {
        ecosystem: ask.ecosystem.clone(),
        at: at.clone(),
        rendered_at: ask.when,
        policy: policy.clone(),
        held: pins.iter().filter_map(held_entry).collect(),
        refused: pins.iter().filter_map(refused_entry).collect(),
    })?;
    Ok(Refreshed {
        at,
        rendered_at: ask.when,
        policy,
        pins,
    })
}

fn held_entry(pin: &PinAnswer) -> Option<HeldEntry> {
    let held = pin.answer.as_ref().ok()?;
    Some(HeldEntry {
        name: pin.pin.name.clone(),
        version: pin.pin.version.clone(),
        index: held.index.clone(),
        artifact: held.artifact.clone(),
        policy: pin.policy.clone()?,
    })
}

fn refused_entry(pin: &PinAnswer) -> Option<RefusedEntry> {
    let Err(Not::Refused(refusal)) = &pin.answer else {
        return None;
    };
    let Why::Wanting { index, wanting, .. } = &refusal.why else {
        return None;
    };
    Some(RefusedEntry {
        name: pin.pin.name.clone(),
        version: pin.pin.version.clone(),
        index: index.clone(),
        wanting: wanting.clone(),
        policy: pin.policy.clone(),
    })
}

/// 51, S10: the layout of `ecosystem` at `at` left as it was beside
/// `because`, what could not be read: what it holds, when it was last
/// rendered and under which policy, from the host's record; holding
/// nothing and never rendered where no record is kept. Where the record
/// cannot be read either, `because` is the answer.
pub fn kept(layouts: &Layouts, ecosystem: &Ecosystem, at: String, because: Not) -> Answer<Refresh> {
    let (holds, rendered_at, policy) = match layouts.read(ecosystem) {
        Ok(record) => (record.held, Some(record.rendered_at), Some(record.policy)),
        Err(Not::Absent(_)) => (Vec::new(), None, None),
        Err(_) => return Err(because),
    };
    Ok(Refresh::Kept(Kept {
        at,
        holds,
        rendered_at,
        policy,
        because,
    }))
}

/// 47, 51: the layout `ask` names rendered, or, where a read, the render
/// or the record could not be made, left as it was and answered from the
/// host's record (design D6).
pub fn refresh(seams: &Seams, layouts: &Layouts, ask: &RenderAsk) -> Answer<Refresh> {
    match render(seams, layouts, ask) {
        Ok(refreshed) => Ok(Refresh::Refreshed(refreshed)),
        Err(because) => {
            let at = place(&ask.tree, &ask.ecosystem).display().to_string();
            kept(layouts, &ask.ecosystem, at, because)
        }
    }
}
