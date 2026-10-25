//! Spec git-ref-ledger: the ledger seam held as git refs, each test over
//! a bare remote and clones of it in a scratch directory, and the suite's
//! keeper group over it. Needs git alone, so it is in the fast layer.

mod scratch;

use std::fs;
use std::os::unix::fs::PermissionsExt;

use scratch::{git, git_fails, Repo, Scratch};
use tailrace::profiles::ledger::git_ref::GitRef;
use tailrace_conformance::seams::keeper::{self, KeeperRig};
use tailrace_conformance::{run, Rig};
use tailrace_fakes::FakeSigner;
use tailrace_seams::{
    Chain, ChannelName, ComponentName, Digest, Envelope, Keeper, Moved, Not, Promotion, Read,
    Revision, Satisfied, Signer, SignerId, Version, Why,
};

const GATE: &str = "gate";

fn stable() -> ChannelName {
    ChannelName::new("stable")
}

fn ledger(channel: &ChannelName) -> String {
    format!("refs/tailrace/ledger/{channel}")
}

/// The `n`th promotion of `channel`, signed as the gate, at a time of its
/// own.
fn promotion(channel: &ChannelName, n: u32) -> Envelope {
    let body = Promotion {
        channel: channel.clone(),
        component: ComponentName::new("lib"),
        to: Moved {
            version: Version::new(format!("0.{n}.0")),
            index: Digest::of(format!("{channel}'s index {n}").as_bytes()),
        },
        from: None,
        predecessor: None,
        asked_by: SignerId::new(GATE),
        policy: Satisfied {
            revision: Revision::new(format!("{n:040x}")),
            claims: vec![],
        },
        at: 1_767_225_600 + u64::from(n),
    };
    let signer = FakeSigner::new();
    signer.hold(&SignerId::new(GATE));
    signer
        .sign(
            &body.claim(Revision::new(format!("{n:040x}"))),
            &SignerId::new(GATE),
        )
        .unwrap()
}

/// A bare remote, the repository the keeper reads, and the keeper bound
/// to it with `origin`.
struct GitRefRig {
    scratch: Scratch,
    keeper: GitRef,
}

impl GitRefRig {
    fn new() -> GitRefRig {
        let scratch = Scratch::new();
        let keeper = GitRef::new(&scratch.repo.path, Some("origin".into()));
        GitRefRig { scratch, keeper }
    }
}

impl Rig for GitRefRig {
    fn profile(&self) -> String {
        "git_ref".into()
    }
}

impl KeeperRig for GitRefRig {
    fn keeper(&self) -> &dyn Keeper {
        &self.keeper
    }

    fn promotion(&self, channel: &ChannelName, n: u32) -> Envelope {
        promotion(channel, n)
    }

    /// Two promotions appended and read, then the remote's ref forced to
    /// a sibling of the second, naming the first, from another clone.
    fn fork(&self, channel: &ChannelName) -> [Envelope; 2] {
        let (first, ours, theirs) = (
            promotion(channel, 1),
            promotion(channel, 2),
            promotion(channel, 3),
        );
        self.keeper.append(channel, &first, None).unwrap();
        self.keeper
            .append(channel, &ours, Some(&first.digest()))
            .unwrap();
        self.keeper.chain(channel).unwrap();
        let name = ledger(channel);
        let other = self.scratch.clone_as("forker");
        let root = self.scratch.repo.git(&["rev-parse", &format!("{name}~1")]);
        other.git(&["fetch", "-q", "origin", &format!("{name}:{name}")]);
        other.git(&["update-ref", &name, &root]);
        GitRef::new(&other.path, None)
            .append(channel, &theirs, Some(&first.digest()))
            .unwrap();
        other.git(&["push", "-q", "--force", "origin", &name]);
        [ours, theirs]
    }

    /// The remote removed, and git's words for listing it.
    fn unreachable(&self) -> String {
        fs::remove_dir_all(&self.scratch.remote).unwrap();
        git_fails(
            &self.scratch.repo.path,
            &["ls-remote", "origin", &ledger(&stable())],
        )
    }
}

#[test]
fn the_keepers_group_over_a_git_ref_with_a_remote() {
    let report = run(keeper::GROUP, &keeper::scenarios(), &|| {
        Box::new(GitRefRig::new())
    });
    assert_eq!(report.profile, "git_ref");
    assert!(report.not_applicable.is_empty(), "{report:?}");
    report.assert();
}

// Requirement: One ref per channel, one claim per commit

#[test]
fn a_chain_read_from_the_ref_is_a_commit_per_claim_each_on_the_one_before() {
    let rig = GitRefRig::new();
    let (first, second) = (promotion(&stable(), 1), promotion(&stable(), 2));
    rig.keeper.append(&stable(), &first, None).unwrap();
    rig.keeper
        .append(&stable(), &second, Some(&first.digest()))
        .unwrap();
    for at in [&rig.scratch.repo.path, &rig.scratch.remote] {
        let head = git(at, &["rev-parse", &ledger(&stable())]);
        let parents = git(at, &["rev-list", "--parents", "-n", "1", &head]);
        let [_, parent] = parents.split(' ').collect::<Vec<_>>()[..] else {
            panic!("the head has other than one parent: {parents}")
        };
        let held = |commit: &str| -> Envelope {
            serde_json::from_str(&git(
                at,
                &["cat-file", "blob", &format!("{commit}:promotion.json")],
            ))
            .unwrap()
        };
        assert_eq!(held(&head), second);
        assert_eq!(held(parent), first);
        let roots = git(at, &["rev-list", "--max-parents=0", &head]);
        assert_eq!(roots, parent, "the first claim's commit has a parent");
        let message = git(at, &["log", "-1", "--format=%an <%ae> %at %s", &head]);
        assert_eq!(
            message,
            format!(
                "tailrace <tailrace@localhost> 1767225602 promotion {}",
                second.digest()
            )
        );
    }
    assert_eq!(
        rig.keeper.chain(&stable()),
        Ok(Chain {
            claims: vec![first, second],
            read: Read::Source,
        })
    );
    let status = rig.scratch.repo.git(&["status", "--porcelain"]);
    assert_eq!(status, "", "the working tree or index was touched");
}

/// A repository whose `stable` ref names `commit`, read with no remote.
fn bound_at(repo: &Repo, commit: &str) -> GitRef {
    repo.git(&["update-ref", &ledger(&stable()), commit]);
    GitRef::new(&repo.path, None)
}

fn unreadable_naming(answer: tailrace_seams::Answer<Chain>, commit: &str) {
    match answer {
        Err(Not::Unreadable(u)) => assert!(u.what.contains(commit), "{u:?} names no {commit}"),
        other => panic!("a ref holding no chain answered {other:?}"),
    }
}

#[test]
fn a_ref_naming_a_commit_with_two_parents_or_holding_no_claim_is_unreadable_naming_it() {
    let scratch = Scratch::new();
    let repo = &scratch.repo;
    let keeper = GitRef::new(&repo.path, None);
    keeper
        .append(&stable(), &promotion(&stable(), 1), None)
        .unwrap();
    let one = repo.git(&["rev-parse", &ledger(&stable())]);
    let other = GitRef::new(&repo.path, None);
    repo.git(&["update-ref", "-d", &ledger(&stable())]);
    other
        .append(&stable(), &promotion(&stable(), 2), None)
        .unwrap();
    let two = repo.git(&["rev-parse", &ledger(&stable())]);
    let tree = repo.git(&["rev-parse", &format!("{one}^{{tree}}")]);
    let merge = repo.git(&[
        "commit-tree",
        &tree,
        "-p",
        &one,
        "-p",
        &two,
        "-m",
        "two parents",
    ]);
    unreadable_naming(bound_at(repo, &merge).chain(&stable()), &merge);

    let empty = repo.git(&["hash-object", "-w", "-t", "tree", "/dev/null"]);
    let bare = repo.git(&["commit-tree", &empty, "-p", &one, "-m", "no claim"]);
    unreadable_naming(bound_at(repo, &bare).chain(&stable()), &bare);

    repo.write("promotion.json", "not an envelope");
    let blob = repo.git(&["hash-object", "-w", "promotion.json"]);
    let made = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "printf '100644 blob {blob}\\tpromotion.json\\n' | git -C '{}' mktree",
            repo.path.display()
        ))
        .output()
        .unwrap();
    let tree = String::from_utf8(made.stdout).unwrap().trim().to_string();
    let garbled = repo.git(&["commit-tree", &tree, "-p", &one, "-m", "garbled"]);
    unreadable_naming(bound_at(repo, &garbled).chain(&stable()), &garbled);
}

// Requirement: Append is a push that fast-forwards

#[test]
fn two_hosts_appending_after_one_head_hold_the_first_and_refuse_the_second() {
    let scratch = Scratch::new();
    let a = GitRef::new(&scratch.repo.path, Some("origin".into()));
    let other = scratch.clone_as("other");
    let b = GitRef::new(&other.path, Some("origin".into()));
    let root = promotion(&stable(), 1);
    a.append(&stable(), &root, None).unwrap();
    assert_eq!(a.chain(&stable()).unwrap().claims, vec![root.clone()]);
    assert_eq!(b.chain(&stable()).unwrap().claims, vec![root.clone()]);

    let (first, second) = (promotion(&stable(), 2), promotion(&stable(), 3));
    a.append(&stable(), &first, Some(&root.digest())).unwrap();
    match b.append(&stable(), &second, Some(&root.digest())) {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::NotHead {
                head: Some(first.digest())
            }
        ),
        other => panic!("the second append answered {other:?}"),
    }
    let remote = GitRef::new(&scratch.remote, None);
    assert_eq!(remote.chain(&stable()).unwrap().claims, [root, first]);
}

#[test]
fn a_remote_refusing_for_its_own_reasons_is_refused_with_gits_words_verbatim() {
    let scratch = Scratch::new();
    let keeper = GitRef::new(&scratch.repo.path, Some("origin".into()));
    let root = promotion(&stable(), 1);
    keeper.append(&stable(), &root, None).unwrap();
    let hook = scratch.remote.join("hooks/pre-receive");
    fs::write(
        &hook,
        "#!/bin/sh\necho 'the ledger of stable is frozen by its owner' >&2\nexit 1\n",
    )
    .unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
    match keeper.append(&stable(), &promotion(&stable(), 2), Some(&root.digest())) {
        Err(Not::Refused(r)) => {
            assert_eq!(r.why, Why::Said);
            let said = r.said.expect("git's words");
            assert!(
                said.contains("remote: the ledger of stable is frozen by its owner"),
                "{said}"
            );
            assert!(said.contains("pre-receive hook declined"), "{said}");
        }
        other => panic!("a remote's refusal answered {other:?}"),
    }
    assert_eq!(keeper.chain(&stable()).unwrap().claims, [root]);
}

#[test]
fn with_no_remote_the_repositorys_own_ref_moves_only_from_where_it_was_read() {
    let scratch = Scratch::new();
    let repo = &scratch.repo;
    let keeper = GitRef::new(&repo.path, None);
    let root = promotion(&stable(), 1);
    keeper.append(&stable(), &root, None).unwrap();
    let elsewhere = GitRef::new(&repo.path, None);
    let theirs = promotion(&stable(), 2);
    elsewhere
        .append(&stable(), &theirs, Some(&root.digest()))
        .unwrap();
    let at = repo.git(&["rev-parse", &ledger(&stable())]);
    match keeper.append(&stable(), &promotion(&stable(), 3), Some(&root.digest())) {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::NotHead {
                head: Some(theirs.digest())
            }
        ),
        other => panic!("an append after a moved ref answered {other:?}"),
    }
    assert_eq!(repo.git(&["rev-parse", &ledger(&stable())]), at);
    assert_eq!(
        repo.git(&["ls-remote", "origin", &ledger(&stable())]),
        "",
        "a ledger with no remote pushed"
    );
}

// Requirement: A remote that no longer extends what was read is a fork

#[test]
fn a_remote_rewritten_under_a_reader_is_a_fork_and_the_readers_copy_keeps_what_it_read() {
    let rig = GitRefRig::new();
    let [ours, theirs] = rig.fork(&stable());
    let read = rig.scratch.repo.git(&["rev-parse", &ledger(&stable())]);
    match rig.keeper.chain(&stable()) {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::Fork {
                predecessor: Some(promotion(&stable(), 1).digest()),
                claims: vec![ours.digest(), theirs.digest()],
            }
        ),
        other => panic!("a forked remote answered {other:?}"),
    }
    assert_eq!(
        rig.scratch.repo.git(&["rev-parse", &ledger(&stable())]),
        read
    );
}

#[test]
fn a_remote_rolled_back_to_an_ancestor_is_unreadable_naming_the_claims_it_lost() {
    let rig = GitRefRig::new();
    let (first, second, third) = (
        promotion(&stable(), 1),
        promotion(&stable(), 2),
        promotion(&stable(), 3),
    );
    rig.keeper.append(&stable(), &first, None).unwrap();
    rig.keeper
        .append(&stable(), &second, Some(&first.digest()))
        .unwrap();
    rig.keeper
        .append(&stable(), &third, Some(&second.digest()))
        .unwrap();
    let name = ledger(&stable());
    let root = git(&rig.scratch.remote, &["rev-parse", &format!("{name}~2")]);
    git(&rig.scratch.remote, &["update-ref", &name, &root]);
    for answer in [
        rig.keeper.chain(&stable()).map(|_| ()),
        rig.keeper
            .append(&stable(), &promotion(&stable(), 4), Some(&third.digest()))
            .map(|_| ()),
    ] {
        match answer {
            Err(Not::Unreadable(u)) => {
                for lost in [&second, &third] {
                    assert!(u.what.contains(lost.digest().as_str()), "{u:?}");
                }
                assert!(!u.what.contains(first.digest().as_str()), "{u:?}");
            }
            other => panic!("a remote rolled back answered {other:?}"),
        }
    }
    git(&rig.scratch.remote, &["update-ref", "-d", &name]);
    assert!(matches!(
        rig.keeper.chain(&stable()),
        Err(Not::Unreadable(u)) if u.what.contains(first.digest().as_str())
    ));
}

// Requirement: A remote that does not answer is read as last fetched

#[test]
fn the_remote_gone_after_a_read_answers_the_chain_as_last_fetched_with_gits_words() {
    let rig = GitRefRig::new();
    let first = promotion(&stable(), 1);
    rig.keeper.append(&stable(), &first, None).unwrap();
    let before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    rig.keeper.chain(&stable()).unwrap();
    let words = rig.unreachable();
    assert!(
        words.contains("does not appear to be a git repository"),
        "{words}"
    );
    match rig.keeper.chain(&stable()) {
        Ok(Chain {
            claims,
            read: Read::Kept { at, said },
        }) => {
            assert_eq!(claims, vec![first.clone()]);
            assert!(at >= before, "kept at {at}, read at {before}");
            assert_eq!(said, words);
        }
        other => panic!("the remote gone answered {other:?}"),
    }
    match rig
        .keeper
        .append(&stable(), &promotion(&stable(), 2), Some(&first.digest()))
    {
        Err(Not::Unreadable(u)) => assert_eq!(u.said.as_deref(), Some(words.as_str())),
        other => panic!("an append with the remote gone answered {other:?}"),
    }
    assert_eq!(
        rig.scratch
            .repo
            .git(&["rev-list", "--count", &ledger(&stable())]),
        "1",
        "the copy alone was appended to"
    );
    let never = ChannelName::new("beta");
    match rig.keeper.chain(&never) {
        Err(Not::Unreadable(u)) => assert_eq!(u.said.as_deref(), Some(words.as_str())),
        other => panic!("a chain never fetched answered {other:?}"),
    }
}

#[test]
fn the_profile_names_no_credential_and_writes_only_its_own_refs() {
    let rig = GitRefRig::new();
    let first = promotion(&stable(), 1);
    rig.keeper.append(&stable(), &first, None).unwrap();
    rig.keeper.chain(&stable()).unwrap();
    let refs = rig
        .scratch
        .repo
        .git(&["for-each-ref", "--format=%(refname)", "refs/tailrace"]);
    assert_eq!(
        refs.lines().collect::<Vec<_>>(),
        [
            "refs/tailrace/fetched/stable",
            "refs/tailrace/ledger/stable"
        ]
    );
    let config = rig.scratch.repo.git(&["config", "--local", "--list"]);
    assert!(!config.contains("credential"), "{config}");
}
