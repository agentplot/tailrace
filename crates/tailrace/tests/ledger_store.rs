//! Spec store-ledger: the ledger seam held in an OCI layout in a
//! temporary directory, its chains kept beside it, and the suite's keeper
//! group over it. Needs nothing beyond the file system, so it is in the
//! fast layer.

use std::fs;
use std::path::PathBuf;

use tailrace::profiles::ledger::store::InStore;
use tailrace::profiles::store::layout::Layout;
use tailrace_conformance::seams::keeper::{self, KeeperRig};
use tailrace_conformance::{run, Rig};
use tailrace_fakes::FakeSigner;
use tailrace_seams::{
    channel_name, Chain, ChannelId, ChannelName, ComponentName, Digest, Envelope, IndexMeta,
    Keeper, Manifest, Moved, Not, Promotion, Read, Revision, Satisfied, Signer, SignerId, Store,
    Version, Why,
};
use tempfile::TempDir;

const GATE: &str = "gate";

fn stable() -> ChannelId {
    ChannelId::new(None, ChannelName::new("stable"))
}

/// `beta` of `component`.
fn beta_of(component: &str) -> ChannelId {
    ChannelId::new(
        Some(ComponentName::new(component)),
        ChannelName::new("beta"),
    )
}

/// A layout, the store ledger over it, and the directory its chains are
/// kept in.
struct StoreRig {
    dir: TempDir,
    layout: Layout,
    keeper: InStore,
}

impl StoreRig {
    fn new() -> StoreRig {
        let dir = TempDir::new().unwrap();
        let layout = Layout::create(dir.path().join("layout")).unwrap();
        let keeper = InStore::new(Box::new(layout.clone()), dir.path().join("kept"));
        StoreRig {
            dir,
            layout,
            keeper,
        }
    }

    fn kept(&self) -> PathBuf {
        self.dir.path().join("kept")
    }

    /// The index of version `0.<n>.0` of `lib`, put in the layout.
    fn index(&self, n: u32) -> Moved {
        let version = Version::new(format!("0.{n}.0"));
        let meta = IndexMeta::new(ComponentName::new("lib"), version.clone(), vec![]).unwrap();
        let index = self
            .layout
            .put_artifact(&Manifest::Index(meta))
            .unwrap()
            .digest;
        Moved { version, index }
    }

    /// A promotion of `channel` to the index of `to`, from the index of
    /// `from`, after `predecessor`, at `at`, signed as the gate.
    fn moved(
        &self,
        channel: &ChannelId,
        to: u32,
        from: Option<u32>,
        predecessor: Option<Digest>,
        at: u64,
    ) -> Envelope {
        let body = Promotion {
            channel: channel.name.clone(),
            component: channel.component_name(),
            to: self.index(to),
            from: from.map(|n| self.index(n)),
            predecessor,
            asked_by: SignerId::new(GATE),
            policy: Satisfied {
                revision: Revision::new(format!("{to:040x}")),
                claims: vec![],
            },
            at,
        };
        let signer = FakeSigner::new();
        signer.hold(&SignerId::new(GATE));
        signer
            .sign(
                &body.claim(Revision::new(format!("{to:040x}"))),
                &SignerId::new(GATE),
            )
            .unwrap()
    }

    /// The `n`th promotion of `channel`: to version `0.<n>.0`, from the
    /// one before, after the `n - 1`th promotion.
    fn nth(&self, channel: &ChannelId, n: u32) -> Envelope {
        let predecessor = (n > 1).then(|| self.nth(channel, n - 1).digest());
        self.moved(
            channel,
            n,
            (n > 1).then_some(n - 1),
            predecessor,
            1_767_225_600 + u64::from(n),
        )
    }
}

impl Rig for StoreRig {
    fn profile(&self) -> String {
        "store".into()
    }
}

impl KeeperRig for StoreRig {
    fn keeper(&self) -> &dyn Keeper {
        &self.keeper
    }

    fn promotion(&self, channel: &ChannelId, n: u32) -> Envelope {
        self.nth(channel, n)
    }

    /// A first promotion appended, then two claims naming it, each moving
    /// the channel to 0.2.0, attached there, and the channel's name moved
    /// to it, as two gates racing on a store with no conditional names
    /// leave them.
    fn fork(&self, channel: &ChannelId) -> [Envelope; 2] {
        let first = self.nth(channel, 1);
        self.keeper.append(channel, &first, None).unwrap();
        let [a, b] = [10, 20].map(|at| self.moved(channel, 2, Some(1), Some(first.digest()), at));
        let (to, from) = (self.index(2).index, self.index(1).index);
        for claim in [&a, &b] {
            self.layout.attach(&to, claim).unwrap();
        }
        self.layout
            .name(&channel_name(channel), &to, Some(&from))
            .unwrap();
        [a, b]
    }

    /// The layout's directory removed, and the words it then answers with.
    fn unreachable(&self) -> String {
        let root = self.layout.root().to_path_buf();
        fs::remove_dir_all(&root).unwrap();
        match self.layout.resolve(&channel_name(&stable())) {
            Err(Not::Unreadable(u)) => u.said.expect("the operating system's words"),
            other => panic!("a layout removed answered {other:?}"),
        }
    }
}

#[test]
fn the_keepers_group_over_the_store() {
    let report = run(keeper::GROUP, &keeper::scenarios(), &|| {
        Box::new(StoreRig::new())
    });
    assert_eq!(report.profile, "store");
    assert!(report.not_applicable.is_empty(), "{report:?}");
    report.assert();
}

// Requirement: The chain is walked from the channel's name

#[test]
fn three_promotions_and_a_move_back_are_answered_in_order() {
    let rig = StoreRig::new();
    let (first, second) = (rig.nth(&stable(), 1), rig.nth(&stable(), 2));
    rig.keeper.append(&stable(), &first, None).unwrap();
    rig.keeper
        .append(&stable(), &second, Some(&first.digest()))
        .unwrap();
    let back = rig.moved(&stable(), 1, Some(2), Some(second.digest()), 1_767_225_700);
    rig.keeper
        .append(&stable(), &back, Some(&second.digest()))
        .unwrap();
    let held = rig.layout.claims_of(&rig.index(1).index).unwrap();
    assert_eq!(
        held,
        [first.clone(), back.clone()],
        "0.1.0's index holds both"
    );
    assert_eq!(
        rig.layout.resolve(&channel_name(&stable())),
        Ok(rig.index(1).index)
    );
    assert_eq!(
        rig.keeper.chain(&stable()),
        Ok(Chain {
            claims: vec![first, second, back],
            read: Read::Source,
        })
    );
}

#[test]
fn a_channel_whose_name_names_nothing_is_absent() {
    let rig = StoreRig::new();
    assert!(matches!(
        rig.keeper.chain(&stable()),
        Err(Not::Absent(a)) if a.what == "chain of stable"
    ));
}

#[test]
fn a_claim_the_store_no_longer_holds_is_unreadable_naming_it_and_its_index() {
    let rig = StoreRig::new();
    let (first, second) = (rig.nth(&stable(), 1), rig.nth(&stable(), 2));
    let (to, from) = (rig.index(2).index, rig.index(1).index);
    rig.layout.attach(&to, &second).unwrap();
    rig.layout
        .name(&channel_name(&stable()), &to, None)
        .unwrap();
    match rig.keeper.chain(&stable()) {
        Err(Not::Unreadable(u)) => {
            assert!(u.what.contains(first.digest().as_str()), "{u:?}");
            assert!(u.what.contains(from.as_str()), "{u:?}");
        }
        other => panic!("a missing predecessor answered {other:?}"),
    }
}

#[test]
fn two_components_beta_walk_apart_from_their_own_names() {
    let rig = StoreRig::new();
    let (lib, web) = (beta_of("lib"), beta_of("web"));
    let mut held = Vec::new();
    for channel in [&lib, &web] {
        let (first, second) = (rig.nth(channel, 1), rig.nth(channel, 2));
        rig.keeper.append(channel, &first, None).unwrap();
        rig.keeper
            .append(channel, &second, Some(&first.digest()))
            .unwrap();
        held.push(vec![first, second]);
    }
    assert_eq!(
        rig.layout.resolve(&channel_name(&lib)),
        Ok(rig.index(2).index)
    );
    assert_eq!(
        channel_name(&lib),
        tailrace_seams::Name::new("channel/lib/beta")
    );
    for (channel, claims) in [&lib, &web].into_iter().zip(held) {
        assert_eq!(
            rig.keeper.chain(channel),
            Ok(Chain {
                claims,
                read: Read::Source,
            }),
            "{channel}'s chain holds another's claims"
        );
    }
    assert!(rig.kept().join("lib").join("beta.json").is_file());
    assert!(matches!(
        rig.keeper
            .chain(&ChannelId::new(None, ChannelName::new("beta"))),
        Err(Not::Absent(_))
    ));
}

// Requirement: Two claims naming one predecessor are a fork

#[test]
fn two_claims_after_one_head_are_refused_as_a_fork_naming_both() {
    let rig = StoreRig::new();
    let [a, b] = rig.fork(&stable());
    match rig.keeper.chain(&stable()) {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::Fork {
                predecessor: Some(rig.nth(&stable(), 1).digest()),
                claims: vec![a.digest(), b.digest()],
            }
        ),
        other => panic!("a fork answered {other:?}"),
    }
}

#[test]
fn two_claims_met_on_one_index_naming_one_predecessor_are_a_fork() {
    let rig = StoreRig::new();
    let first = rig.nth(&stable(), 1);
    rig.keeper.append(&stable(), &first, None).unwrap();
    let ours = rig.nth(&stable(), 2);
    let theirs = rig.moved(&stable(), 2, Some(1), Some(first.digest()), 99);
    rig.keeper
        .append(&stable(), &ours, Some(&first.digest()))
        .unwrap();
    rig.layout.attach(&rig.index(2).index, &theirs).unwrap();
    let onward = rig.nth(&stable(), 3);
    rig.layout.attach(&rig.index(3).index, &onward).unwrap();
    rig.layout
        .name(
            &channel_name(&stable()),
            &rig.index(3).index,
            Some(&rig.index(2).index),
        )
        .unwrap();
    match rig.keeper.chain(&stable()) {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::Fork {
                predecessor: Some(first.digest()),
                claims: vec![ours.digest(), theirs.digest()],
            }
        ),
        other => panic!("a fork met on the walk answered {other:?}"),
    }
}

// Requirement: Append attaches the claim and moves the name

#[test]
fn an_append_after_a_stale_head_is_refused_and_nothing_is_attached_or_moved() {
    let rig = StoreRig::new();
    let (first, second) = (rig.nth(&stable(), 1), rig.nth(&stable(), 2));
    rig.keeper.append(&stable(), &first, None).unwrap();
    rig.keeper
        .append(&stable(), &second, Some(&first.digest()))
        .unwrap();
    let stale = rig.moved(&stable(), 3, Some(1), Some(first.digest()), 7);
    let held = rig.layout.claims_of(&rig.index(3).index).unwrap();
    match rig.keeper.append(&stable(), &stale, Some(&first.digest())) {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::NotHead {
                head: Some(second.digest())
            }
        ),
        other => panic!("a stale append answered {other:?}"),
    }
    assert_eq!(rig.layout.claims_of(&rig.index(3).index).unwrap(), held);
    assert_eq!(
        rig.layout.resolve(&channel_name(&stable())),
        Ok(rig.index(2).index)
    );
}

#[test]
fn a_claim_naming_another_predecessor_than_the_one_given_is_refused() {
    let rig = StoreRig::new();
    let first = rig.nth(&stable(), 1);
    rig.keeper.append(&stable(), &first, None).unwrap();
    let orphan = rig.moved(&stable(), 2, Some(1), None, 5);
    assert!(matches!(
        rig.keeper.append(&stable(), &orphan, Some(&first.digest())),
        Err(Not::Refused(r)) if r.why == Why::NotHead { head: Some(first.digest()) }
    ));
}

// Requirement: A store that does not answer is read as last read

#[test]
fn the_store_gone_after_a_read_answers_the_chain_as_kept_with_its_time_and_words() {
    let rig = StoreRig::new();
    let first = rig.nth(&stable(), 1);
    rig.keeper.append(&stable(), &first, None).unwrap();
    let before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    rig.keeper.chain(&stable()).unwrap();

    let file = rig.kept().join("stable.json");
    let kept: serde_json::Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    let mut keys: Vec<&String> = kept.as_object().unwrap().keys().collect();
    keys.sort();
    assert_eq!(
        keys,
        ["at", "claims"],
        "the kept file holds more than the chain"
    );
    assert_eq!(
        serde_json::from_value::<Vec<Envelope>>(kept["claims"].clone()).unwrap(),
        vec![first.clone()]
    );
    let text = fs::read_to_string(&file).unwrap();
    assert!(!text.contains("PRIVATE KEY") && !text.contains("credential"));

    let next = rig.nth(&stable(), 2);
    let words = rig.unreachable();
    assert_eq!(words, "No such file or directory (os error 2)");
    match rig.keeper.chain(&stable()) {
        Ok(Chain {
            claims,
            read: Read::Kept { at, said },
        }) => {
            assert_eq!(claims, vec![first.clone()]);
            assert!(at >= before, "kept at {at}, read at {before}");
            assert_eq!(said, words);
        }
        other => panic!("the store gone answered {other:?}"),
    }
    match rig.keeper.append(&stable(), &next, Some(&first.digest())) {
        Err(Not::Unreadable(u)) => assert_eq!(u.said.as_deref(), Some(words.as_str())),
        other => panic!("an append with the store gone answered {other:?}"),
    }
    match rig
        .keeper
        .chain(&ChannelId::new(None, ChannelName::new("beta")))
    {
        Err(Not::Unreadable(u)) => assert_eq!(u.said.as_deref(), Some(words.as_str())),
        other => panic!("a chain never read answered {other:?}"),
    }
}
