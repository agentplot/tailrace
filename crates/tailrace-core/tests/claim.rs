//! Spec component-versions: the name is claimed in the shared history
//! before anything is packed, and a name another revision holds refuses
//! the publish (design D6).

mod common;

use std::fs;

use common::{landed, Fixture, Repo};
use tailrace_core::{claim, derive, Claimed, Derived, Rule, Versioned};
use tailrace_seams::{Component, ComponentName, Not, Revision, Seam, Why};

fn lib() -> Component {
    Component {
        name: ComponentName::new("lib"),
        outputs: vec![],
        paths: vec!["crates/lib/**".into()],
        prefix: "lib-v".into(),
    }
}

fn derived(repo: &Repo, rev: &Revision) -> Versioned {
    match derive(&repo.history(), &landed(rev), &Rule::default(), &lib()).unwrap() {
        Derived::Versioned(v) => v,
        Derived::Unchanged { .. } => panic!("lib was not versioned"),
    }
}

fn claimed(repo: &Repo, rev: &Revision) -> Result<Claimed, Not> {
    claim(&repo.history(), &landed(rev), &derived(repo, rev))
}

/// A shared history holding `lib` released at 1.2.3, and two clones of
/// it, two publishers.
fn two_publishers(fixture: &Fixture) -> (Repo, Repo, Repo) {
    let shared = fixture.bare("shared.git");
    let first = fixture.clone_of(&shared, "first");
    let base = first.touch("feat: lib", &["crates/lib"]);
    first.tag("lib-v1.2.3", &base);
    first.git(&["push", "-q", "origin", "main", "refs/tags/lib-v1.2.3"]);
    let second = fixture.clone_of(&shared, "second");
    (shared, first, second)
}

fn refs(repo: &Repo) -> String {
    repo.git(&["for-each-ref", "--format=%(refname) %(objectname)"])
}

#[test]
fn a_new_name_lands_in_both_histories() {
    let fixture = Fixture::new();
    let (shared, first, _) = two_publishers(&fixture);
    let rev = first.touch("fix: lib", &["crates/lib"]);
    let claimed = claimed(&first, &rev).unwrap();
    assert_eq!(
        claimed,
        Claimed {
            name: "lib-v1.2.4".into(),
            revision: rev.clone(),
            shared: Some("origin".into()),
        }
    );
    assert_eq!(first.tag_at("lib-v1.2.4").as_deref(), Some(rev.as_str()));
    assert_eq!(shared.tag_at("lib-v1.2.4").as_deref(), Some(rev.as_str()));
}

#[test]
fn the_same_revision_claimed_again_changes_nothing() {
    let fixture = Fixture::new();
    let (shared, first, _) = two_publishers(&fixture);
    let rev = first.touch("fix: lib", &["crates/lib"]);
    let once = claimed(&first, &rev).unwrap();
    let before = (refs(&shared), refs(&first));
    let again = claimed(&first, &rev).unwrap();
    assert_eq!(again, once);
    assert_eq!((refs(&shared), refs(&first)), before);
}

#[test]
fn a_branch_with_no_remote_claims_in_its_own_history_alone() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let rev = repo.touch("feat: lib", &["crates/lib"]);
    let claimed = claimed(&repo, &rev).unwrap();
    assert_eq!(claimed.shared, None);
    assert_eq!(claimed.name, "lib-v0.1.0");
    assert_eq!(repo.tag_at("lib-v0.1.0").as_deref(), Some(rev.as_str()));
}

#[test]
fn a_name_another_publisher_claimed_at_another_revision_is_refused() {
    let fixture = Fixture::new();
    let (shared, first, second) = two_publishers(&fixture);
    let theirs = second.touch("fix: lib, theirs", &["crates/lib"]);
    let ours = first.touch("fix: lib, ours", &["crates/lib"]);
    let ours_derived = derived(&first, &ours);
    claimed(&second, &theirs).unwrap();
    assert_eq!(ours_derived.name, "lib-v1.2.4");

    let Err(Not::Refused(r)) = claim(&first.history(), &landed(&ours), &ours_derived) else {
        panic!("a name another revision holds was claimed")
    };
    assert_eq!(
        r.why,
        Why::NameHeld {
            name: "lib-v1.2.4".into(),
            revision: ours.clone(),
            other: theirs.clone(),
        }
    );
    assert_eq!(r.source.seam, Seam::History);
    assert_eq!(r.source.profile, "git, origin");
    let said = r.said.unwrap();
    assert!(
        said.contains("!\trefs/tags/lib-v1.2.4:refs/tags/lib-v1.2.4\t[rejected] (already exists)"),
        "{said}"
    );
    assert!(said.contains("error: failed to push some refs"), "{said}");
    assert_eq!(
        shared.tag_at("lib-v1.2.4").as_deref(),
        Some(theirs.as_str())
    );
    assert_eq!(first.tag_at("lib-v1.2.4"), None);
}

#[test]
fn a_name_the_local_history_holds_at_another_revision_is_refused_before_any_push() {
    let fixture = Fixture::new();
    let (shared, first, _) = two_publishers(&fixture);
    let other = first.touch("fix: lib", &["crates/lib"]);
    let rev = first.touch("fix: lib again", &["crates/lib"]);
    let v = derived(&first, &rev);
    first.tag(&v.name, &other);
    let before = refs(&shared);

    let Err(Not::Refused(r)) = claim(&first.history(), &landed(&rev), &v) else {
        panic!("a name the local history holds elsewhere was claimed")
    };
    assert_eq!(
        r.why,
        Why::NameHeld {
            name: v.name.clone(),
            revision: rev,
            other: other.clone(),
        }
    );
    assert_eq!(r.source.profile, "git, local");
    assert_eq!(refs(&shared), before);
    assert_eq!(first.tag_at(&v.name).as_deref(), Some(other.as_str()));
}

#[test]
fn a_shared_history_that_is_gone_is_unreadable_never_refused() {
    let fixture = Fixture::new();
    let (shared, first, _) = two_publishers(&fixture);
    let rev = first.touch("fix: lib", &["crates/lib"]);
    let v = derived(&first, &rev);
    fs::remove_dir_all(&shared.path).unwrap();

    let Err(Not::Unreadable(u)) = claim(&first.history(), &landed(&rev), &v) else {
        panic!("a claim with the shared history gone was not unreadable")
    };
    assert_eq!(u.source.profile, "git, origin");
    let said = u.said.unwrap();
    assert!(
        said.contains("does not appear to be a git repository"),
        "{said}"
    );
}

#[test]
fn a_derivation_after_a_claim_gives_the_claimed_version() {
    let fixture = Fixture::new();
    let (_, first, _) = two_publishers(&fixture);
    let rev = first.touch("chore: tidy", &["crates/lib"]);
    let v = derived(&first, &rev);
    claim(&first.history(), &landed(&rev), &v).unwrap();
    let again = derived(&first, &rev);
    assert_eq!((again.version.clone(), again.claimed), (v.version, true));
    assert_eq!(again.name, "lib-v1.2.4-pre.1");
}
