//! The repository's history as the core reads it (design D3, D6).

mod common;

use std::collections::BTreeSet;
use std::fs;

use common::{landed, Fixture};
use semver::Version;
use tailrace_core::History;
use tailrace_seams::{Branch, Not, Revision, Seam};

#[test]
fn a_path_that_is_not_a_repository_is_unreadable_with_gits_words() {
    let fixture = Fixture::new();
    let nowhere = fixture.path("nowhere");
    fs::create_dir_all(&nowhere).unwrap();
    let Err(Not::Unreadable(u)) = History::open(&nowhere) else {
        panic!("a directory with no repository opened")
    };
    assert_eq!(u.source.seam, Seam::History);
    assert_eq!(u.source.profile, "git, local");
    assert!(u.what.contains(nowhere.to_str().unwrap()), "{}", u.what);
    let said = u.said.unwrap();
    assert!(said.contains("could not find repository"), "{said}");
}

#[test]
fn a_file_is_read_from_the_revision_while_the_working_tree_holds_another() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let rev = repo.commit("chore: start", &[("notes.txt", "at the revision")]);
    fs::write(repo.path.join("notes.txt"), "in the working tree").unwrap();

    let history = repo.history();
    let at = landed(&rev);
    assert_eq!(history.blob(&at, "notes.txt").unwrap(), b"at the revision");
    assert!(matches!(
        history.blob(&at, "missing.txt"),
        Err(Not::Absent(a)) if a.what.contains("missing.txt")
    ));
}

#[test]
fn a_commit_outside_the_paths_is_not_listed() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let first = repo.touch("feat: lib", &["crates/lib"]);
    let site = repo.touch("feat: site", &["site"]);
    let lib = repo.touch("fix: lib", &["crates/lib"]);

    let history = repo.history();
    let at = landed(&lib);
    let revisions = |since: Option<&Revision>, paths: &[&str]| {
        let paths: Vec<String> = paths.iter().map(|p| p.to_string()).collect();
        history
            .commits(&at, since, &paths)
            .unwrap()
            .into_iter()
            .map(|c| (c.revision, c.message))
            .collect::<BTreeSet<_>>()
    };
    let commit = |rev: &Revision, message: &str| (rev.clone(), format!("{message}\n"));
    assert_eq!(
        revisions(None, &["crates/lib/**"]),
        BTreeSet::from([commit(&first, "feat: lib"), commit(&lib, "fix: lib")])
    );
    assert_eq!(
        revisions(Some(&first), &["crates/lib/**"]),
        BTreeSet::from([commit(&lib, "fix: lib")])
    );
    assert_eq!(
        revisions(Some(&first), &["**"]),
        BTreeSet::from([commit(&site, "feat: site"), commit(&lib, "fix: lib")])
    );
}

#[test]
fn an_annotated_and_a_lightweight_tag_both_peel_to_their_commit() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let one = repo.touch("feat: one", &["a"]);
    let two = repo.touch("feat: two", &["a"]);
    repo.annotated("v1.0.0", &one);
    repo.tag("v1.1.0", &two);
    repo.tag("v-not-a-version", &two);
    repo.tag("lib-v9.0.0", &two);

    let tags = repo.history().tags(&landed(&two), "v").unwrap();
    let named: Vec<_> = tags
        .iter()
        .map(|t| (t.name.as_str(), t.version.clone(), t.revision.clone()))
        .collect();
    assert_eq!(
        named,
        [
            ("v1.0.0", Version::new(1, 0, 0), one),
            ("v1.1.0", Version::new(1, 1, 0), two)
        ]
    );
}

#[test]
fn a_tag_on_a_revision_the_landed_one_does_not_reach_is_not_kept() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let base = repo.touch("feat: base", &["a"]);
    repo.tag("v1.0.0", &base);
    repo.git(&["checkout", "-q", "-b", "side"]);
    let side = repo.touch("feat: side", &["a"]);
    repo.tag("v2.0.0", &side);
    repo.git(&["checkout", "-q", "main"]);
    let main = repo.touch("fix: main", &["a"]);

    let tags = repo.history().tags(&landed(&main), "v").unwrap();
    assert_eq!(
        tags.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
        ["v1.0.0"]
    );
}

#[test]
fn a_tag_only_the_shared_history_holds_is_read() {
    let fixture = Fixture::new();
    let shared = fixture.bare("shared.git");
    let first = fixture.clone_of(&shared, "first");
    let rev = first.touch("feat: one", &["a"]);
    first.git(&["push", "-q", "origin", "main"]);
    let second = fixture.clone_of(&shared, "second");
    first.annotated("v1.0.0", &rev);
    first.git(&["push", "-q", "origin", "refs/tags/v1.0.0"]);
    first.git(&["tag", "-d", "v1.0.0"]);
    assert_eq!(second.tag_at("v1.0.0"), None);

    let history = second.history();
    assert_eq!(
        history.remote(&Branch::new("main")).unwrap().as_deref(),
        Some("origin")
    );
    let tags = history.tags(&landed(&rev), "v").unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "v1.0.0");
    assert_eq!(tags[0].revision, rev);
}

#[test]
fn a_shared_history_that_cannot_be_reached_makes_the_tags_unreadable() {
    let fixture = Fixture::new();
    let shared = fixture.bare("shared.git");
    let clone = fixture.clone_of(&shared, "clone");
    let rev = clone.touch("feat: one", &["a"]);
    clone.git(&["push", "-q", "origin", "main"]);
    fs::remove_dir_all(&shared.path).unwrap();

    let Err(Not::Unreadable(u)) = clone.history().tags(&landed(&rev), "v") else {
        panic!("tags read with the shared history gone")
    };
    assert_eq!(u.source.profile, "git, origin");
    let said = u.said.unwrap();
    assert!(
        said.contains("does not appear to be a git repository"),
        "{said}"
    );
}

#[test]
fn a_branch_with_no_remote_shares_no_history() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.touch("feat: one", &["a"]);
    assert_eq!(repo.history().remote(&Branch::new("main")).unwrap(), None);
}

#[test]
fn a_short_id_and_head_resolve_to_the_full_id() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let first = repo.commit("chore: start", &[("a", "a")]);
    let second = repo.commit("feat: more", &[("b", "b")]);
    repo.tag("v0.1.0", &first);

    let history = repo.history();
    assert_eq!(history.resolve(&second.as_str()[..7]).unwrap(), second);
    assert_eq!(history.resolve("HEAD").unwrap(), second);
    assert_eq!(history.resolve("HEAD~1").unwrap(), first);
    assert_eq!(history.resolve("v0.1.0").unwrap(), first);
    assert_eq!(history.resolve(second.as_str()).unwrap(), second);
    assert_eq!(history.head(&Branch::new("main")).unwrap(), second);
}

#[test]
fn a_name_that_is_no_commit_is_absent_with_gits_words() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit("chore: start", &[("a", "a")]);

    let history = repo.history();
    let Err(Not::Absent(a)) = history.resolve("no-such-thing") else {
        panic!("a name that is no commit resolved")
    };
    assert_eq!(a.source.seam, Seam::History);
    let git = git2::Repository::open(&repo.path)
        .unwrap()
        .revparse_single("no-such-thing")
        .unwrap_err();
    assert_eq!(
        a.what,
        format!("commit named no-such-thing: {}", git.message())
    );
    let tree = repo.git(&["rev-parse", "HEAD^{tree}"]);
    assert!(matches!(history.resolve(&tree), Err(Not::Absent(_))));
    assert!(matches!(
        history.head(&Branch::new("elsewhere")),
        Err(Not::Absent(a)) if a.what == "branch elsewhere"
    ));
}

#[test]
fn two_worktrees_of_one_repository_answer_one_common_directory() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit("chore: start", &[("a", "a")]);
    let other = fixture.path("other");
    repo.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "other",
        other.to_str().unwrap(),
    ]);

    let here = repo.history().common_dir();
    let there = History::open(&other).unwrap().common_dir();
    assert_eq!(here, there);
    assert_eq!(here, repo.path.join(".git").canonicalize().unwrap());

    let elsewhere = fixture.repo("elsewhere");
    elsewhere.commit("chore: start", &[("a", "a")]);
    assert_ne!(elsewhere.history().common_dir(), here);
}
