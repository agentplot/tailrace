//! The store's group: 66 to 73, and the clauses of Part A a store's
//! operations keep (B.1).

use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;
use tailrace_core::{follow, Followed};
use tailrace_fakes::FakeKeeper;
use tailrace_seams::{
    Answer, ArtifactMeta, ChannelId, ChannelName, Claim, ClaimKind, Digest, Ecosystem,
    EcosystemName, Envelope, Held, Keeper, Manifest, Name, Not, Revision, Seam, Signature,
    SignerId, Store, Version, Why, IN_TOTO,
};

use crate::seams::unreadable;
use crate::{not_applicable, ok, Rig, Scenario};

pub const GROUP: &str = "store";

/// The bytes the group puts, so a rig can stage what a scenario reads.
pub const BYTES: &[u8] = b"the bytes of an artifact";

/// Bytes the group never puts.
pub const NEVER_PUT: &[u8] = b"bytes no scenario puts";

/// A store, made new for each scenario.
pub trait StoreRig: Rig {
    fn store(&self) -> &dyn Store;

    /// The store made unreachable, every operation answering unreadable,
    /// and the words it answers with, which the profile carries
    /// unchanged.
    fn unreachable(&self) -> String;

    /// How long, after a move, the profile may still resolve the name to
    /// a place it was at, as the profile declares; none by default, for
    /// a store whose every read shows a move at once (C.1, 36).
    fn settles_within(&self) -> Duration {
        Duration::ZERO
    }

    /// Whether the profile's names are conditional, as the profile
    /// declares: a move refused by the store itself unless the name is at
    /// the expected digest; yes by default. A profile whose names take no
    /// condition, a registry's, refuses a move from a place the name has
    /// left only by best effort, and the group runs the repair from the
    /// ledger in place of that refusal (C.1, 109).
    fn names_conditional(&self) -> bool {
        true
    }
}

pub fn scenarios() -> Vec<Scenario<dyn StoreRig>> {
    vec![
        Scenario {
            name: "the same bytes put twice",
            plays: None,
            cites: &[1, 5, 66],
            run: the_same_bytes_put_twice,
        },
        Scenario {
            name: "an artifact read back",
            plays: None,
            cites: &[4, 67, 72],
            run: an_artifact_read_back,
        },
        Scenario {
            name: "two attestations on one digest",
            plays: None,
            cites: &[8, 10, 68, 69],
            run: two_attestations_on_one_digest,
        },
        Scenario {
            name: "a name moved from where it is not",
            plays: None,
            cites: &[70, 71],
            run: a_name_moved_from_where_it_is_not,
        },
        Scenario {
            name: "a name that names nothing",
            plays: None,
            cites: &[71],
            run: a_name_that_names_nothing,
        },
        Scenario {
            name: "bytes read back",
            plays: None,
            cites: &[1, 72, 73],
            run: bytes_read_back,
        },
        Scenario {
            name: "a store that cannot be reached",
            plays: None,
            cites: &[58, 59, 72],
            run: a_store_that_cannot_be_reached,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
        Scenario {
            name: "a name moved from a stale place, repaired from the ledger",
            plays: None,
            cites: &[70, 109],
            run: a_name_moved_from_a_stale_place_repaired_from_the_ledger,
        },
    ]
}

/// Metadata in an ecosystem the store need not know.
fn meta(version: &str) -> ArtifactMeta {
    ArtifactMeta::new(
        Ecosystem::new("file"),
        EcosystemName::new("site"),
        Version::new(version),
    )
    .expect("the metadata names all three")
}

/// An artifact of `version` over `BYTES`, put, and its digest.
fn artifact(store: &dyn Store, version: &str) -> Digest {
    let bytes = ok(store.put_bytes(BYTES), "put-bytes").digest;
    ok(
        store.put_artifact(&Manifest::Artifact {
            meta: meta(version),
            layers: vec![bytes],
        }),
        "put-artifact",
    )
    .digest
}

/// An attestation about `subject`, as a signer would make it.
fn attestation(subject: &Digest, says: &str) -> Envelope {
    let claim = Claim {
        subject: subject.clone(),
        kind: ClaimKind::Test,
        revision: Revision::new("4f2a9c1"),
        build: None,
        body: json!({ "says": says }),
    };
    let payload = claim.payload();
    let sig = Digest::of(&payload).as_str().as_bytes().to_vec();
    Envelope::signed(
        IN_TOTO,
        payload,
        Signature {
            signer: SignerId::new("a tester"),
            keyid: String::new(),
            sig,
        },
    )
}

fn is_absent<T: std::fmt::Debug>(answer: &Answer<T>) -> bool {
    matches!(answer, Err(Not::Absent(a)) if a.source.seam == Seam::Store)
}

/// What `name`, moved to `to`, resolves to in `store` once it settles:
/// read again at growing intervals while it answers a place it was at,
/// one of `was` (nothing among them where it was made from nothing),
/// until `within`, the bound the profile declares, passes. Any other
/// answer is answered at once, so a digest the name was never at fails
/// without a wait (C.1, 36). A profile's own tests read a moved name
/// through it as the group does.
pub fn settled(
    store: &dyn Store,
    within: Duration,
    name: &Name,
    to: &Digest,
    was: &[Option<&Digest>],
) -> Answer<Digest> {
    let deadline = Instant::now() + within;
    let mut wait = Duration::from_millis(250);
    loop {
        let answer = store.resolve(name);
        let former = match &answer {
            Ok(at) => at != to && was.contains(&Some(at)),
            absent => is_absent(absent) && was.contains(&None),
        };
        let left = deadline.saturating_duration_since(Instant::now());
        if !former || left.is_zero() {
            return answer;
        }
        thread::sleep(wait.min(left));
        wait *= 2;
    }
}

fn the_same_bytes_put_twice(rig: &dyn StoreRig) {
    let store = rig.store();
    let digest = Digest::of(BYTES);
    let first = ok(store.put_bytes(BYTES), "the first put");
    let second = ok(store.put_bytes(BYTES), "the second put");
    assert_eq!(
        first.digest, digest,
        "the first put answered {first:?}, not the bytes' own digest"
    );
    assert_eq!(
        second.digest, digest,
        "the second put answered {second:?}, not the bytes' own digest"
    );
    assert!(
        second.held_already,
        "the second put answered {second:?}, not held already"
    );
    match ok(store.read(&digest), "read") {
        Held::Bytes(bytes) => assert!(
            bytes == BYTES,
            "read of the bytes put answered {:?}",
            String::from_utf8_lossy(&bytes)
        ),
        held => panic!("read of bytes put answered {held:?}"),
    }
}

fn an_artifact_read_back(rig: &dyn StoreRig) {
    let store = rig.store();
    let bytes = ok(store.put_bytes(BYTES), "put-bytes").digest;
    let manifest = Manifest::Artifact {
        meta: meta("1.0.0"),
        layers: vec![bytes.clone()],
    };
    let put = ok(store.put_artifact(&manifest), "put-artifact");
    match ok(store.read(&put.digest), "read of the artifact") {
        Held::Manifest(Manifest::Artifact { meta: held, layers }) => {
            assert_eq!(held.ecosystem(), &Ecosystem::new("file"), "{held:?}");
            assert_eq!(held.name(), &EcosystemName::new("site"), "{held:?}");
            assert_eq!(held.version(), &Version::new("1.0.0"), "{held:?}");
            assert_eq!(layers, [bytes], "the artifact names other bytes");
        }
        held => panic!("read of an artifact answered {held:?}"),
    }
    let again = ok(store.put_artifact(&manifest), "put-artifact again");
    assert_eq!(
        again.digest, put.digest,
        "the artifact is named by another digest the second time"
    );
}

fn two_attestations_on_one_digest(rig: &dyn StoreRig) {
    let store = rig.store();
    let subject = artifact(store, "1.0.0");
    let (first, second) = (
        attestation(&subject, "it passed"),
        attestation(&subject, "it passed again"),
    );
    let put = ok(store.attach(&subject, &first), "the first attach");
    assert_eq!(put.digest, first.digest(), "attach answered {put:?}");
    ok(store.attach(&subject, &second), "the second attach");
    let held = ok(store.claims_of(&subject), "claims-of");
    assert_eq!(held.len(), 2, "claims-of answered {held:?}");
    for attached in [&first, &second] {
        assert!(
            held.contains(attached),
            "claims-of answered {held:?}, without {attached:?} as attached"
        );
    }
    let elsewhere = Digest::of(NEVER_PUT);
    let answer = store.claims_of(&elsewhere);
    assert!(
        is_absent(&answer),
        "claims-of a digest not held answered {answer:?}"
    );
    let answer = store.attach(&elsewhere, &attestation(&elsewhere, "nothing"));
    assert!(
        is_absent(&answer),
        "an attestation was held where its digest is not: {answer:?}"
    );
    let held = ok(store.claims_of(&subject), "claims-of again");
    assert!(
        held.contains(&first) && held.contains(&second),
        "an attestation was not kept: {held:?}"
    );
}

fn a_name_moved_from_where_it_is_not(rig: &dyn StoreRig) {
    if !rig.names_conditional() {
        not_applicable(
            "the profile's names take no condition, so it refuses a move from a place the name \
             has left only by best effort, and the group repairs the name from the ledger in its \
             stead (C.1)",
        );
    }
    let store = rig.store();
    let (at, other, elsewhere) = (
        artifact(store, "1.0.0"),
        artifact(store, "1.1.0"),
        artifact(store, "1.2.0"),
    );
    let name = Name::new("site-v1");
    ok(store.name(&name, &at, None), "a name made from nothing");
    let refused = |answer: Answer<()>, asked: &str| match answer {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::NameAt {
                at: Some(at.clone())
            },
            "{asked} was refused naming another digest"
        ),
        other => panic!("{asked} answered {other:?}, not refused"),
    };
    refused(
        store.name(&name, &other, Some(&elsewhere)),
        "a move from where the name is not",
    );
    refused(
        store.name(&name, &other, None),
        "a move from nothing of a name made",
    );
    assert_eq!(
        ok(
            settled(store, rig.settles_within(), &name, &at, &[None]),
            "resolve"
        ),
        at,
        "a refused move moved the name"
    );
    ok(
        store.name(&name, &other, Some(&at)),
        "a move from where the name is",
    );
    assert_eq!(
        ok(
            settled(
                store,
                rig.settles_within(),
                &name,
                &other,
                &[None, Some(&at)]
            ),
            "resolve"
        ),
        other,
        "the name did not move"
    );
}

fn a_name_that_names_nothing(rig: &dyn StoreRig) {
    let answer = rig.store().resolve(&Name::new("never-made"));
    assert!(
        is_absent(&answer),
        "a name never made resolved to {answer:?}"
    );
}

fn bytes_read_back(rig: &dyn StoreRig) {
    let store = rig.store();
    let digest = ok(store.put_bytes(BYTES), "put-bytes").digest;
    for read in ["the first read", "the second read"] {
        match ok(store.read(&digest), read) {
            Held::Bytes(bytes) => {
                assert_eq!(
                    Digest::of(&bytes),
                    digest,
                    "{read} answered bytes whose digest is {}, under {digest}",
                    Digest::of(&bytes)
                );
                assert!(
                    bytes == BYTES,
                    "{read} answered {:?}",
                    String::from_utf8_lossy(&bytes)
                );
            }
            held => panic!("{read} of bytes put answered {held:?}"),
        }
    }
    let answer = store.read(&Digest::of(NEVER_PUT));
    assert!(
        is_absent(&answer),
        "read of a digest never put answered {answer:?}"
    );
}

fn a_store_that_cannot_be_reached(rig: &dyn StoreRig) {
    let digest = ok(rig.store().put_bytes(BYTES), "put-bytes").digest;
    let words = rig.unreachable();
    let store = rig.store();
    unreadable(store.read(&digest), Seam::Store, &words, "read");
    unreadable(store.claims_of(&digest), Seam::Store, &words, "claims-of");
    unreadable(
        store.resolve(&Name::new("site-v1")),
        Seam::Store,
        &words,
        "resolve",
    );
    unreadable(store.put_bytes(BYTES), Seam::Store, &words, "put-bytes");
}

/// A promotion of `subject` to the channel `stable`, as the gate signs
/// one.
fn promotion(subject: &Digest) -> Envelope {
    let claim = Claim {
        subject: subject.clone(),
        kind: ClaimKind::Promotion,
        revision: Revision::new("4f2a9c1"),
        build: None,
        body: json!({ "channel": "stable" }),
    };
    let payload = claim.payload();
    let sig = Digest::of(&payload).as_str().as_bytes().to_vec();
    Envelope::signed(
        IN_TOTO,
        payload,
        Signature {
            signer: SignerId::new("a gate"),
            keyid: String::new(),
            sig,
        },
    )
}

/// On a profile whose names take no condition, a move from a place the
/// name has left is refused only when the check before the put reads
/// where the name is, so the scenario takes either answer and asks the
/// name's safety of the ledger: written before the name moves (33), and
/// the name repaired from it, wherever the stale move left it (109, C.1).
fn a_name_moved_from_a_stale_place_repaired_from_the_ledger(rig: &dyn StoreRig) {
    if rig.names_conditional() {
        not_applicable(
            "the profile's names are conditional, so it refuses a move from a place the name has \
             left, which `a name moved from where it is not` proves in place of the repair (C.1)",
        );
    }
    let (store, within) = (rig.store(), rig.settles_within());
    let (left, head, stale) = (
        artifact(store, "1.0.0"),
        artifact(store, "1.1.0"),
        artifact(store, "1.2.0"),
    );
    let (channel, name) = (
        ChannelId::new(None, ChannelName::new("stable")),
        Name::new("stable"),
    );
    ok(store.name(&name, &left, None), "a name made from nothing");
    assert_eq!(
        ok(settled(store, within, &name, &left, &[None]), "resolve"),
        left,
        "the name was not made"
    );
    let keeper = FakeKeeper::new();
    ok(
        keeper.append(&channel, &promotion(&head), None),
        "the promotion's append",
    );
    ok(
        store.name(&name, &head, Some(&left)),
        "a move from where the name is",
    );
    assert_eq!(
        ok(
            settled(store, within, &name, &head, &[None, Some(&left)]),
            "resolve"
        ),
        head,
        "the name did not move"
    );

    let was = [None, Some(&left), Some(&head)];
    let now = match store.name(&name, &stale, Some(&left)) {
        Ok(()) => &stale,
        Err(Not::Refused(r)) => match &r.why {
            Why::NameAt { at } if was.contains(&at.as_ref()) => &head,
            _ => panic!("a move from a place the name has left was refused naming {r:?}"),
        },
        answer => panic!("a move from a place the name has left answered {answer:?}"),
    };
    // What the registry shows of the stale move, read until it settles
    // before the repair reads it; where it settles is the repair's to find.
    let _ = settled(
        store,
        within,
        &name,
        now,
        &[None, Some(&left), Some(&head), Some(&stale)],
    );

    let followed = ok(
        follow(store, &keeper, &channel, &name, Some(&head)),
        "the repair from the ledger",
    );
    let to = match &followed {
        Followed::Moved { to } | Followed::LostRace { to, .. } => to,
        Followed::There { at } => at,
    };
    assert_eq!(
        to, &head,
        "the repair answered {followed:?}, not the digest the ledger's head is about"
    );
    assert_eq!(
        ok(
            settled(
                store,
                within,
                &name,
                &head,
                &[None, Some(&left), Some(&stale)]
            ),
            "resolve"
        ),
        head,
        "the repair did not put the name where the ledger's head says"
    );
}
