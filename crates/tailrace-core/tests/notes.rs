//! Spec version-notes: a version's notes rendered from the commits it is
//! derived from, by the rule's own tool, the same bytes on every host.

mod common;

use std::thread;
use std::time::Duration;

use common::{landed, Fixture, Repo};
use tailrace_core::{derive, notes, Derived, Kind, Rule, Versioned};
use tailrace_seams::{Component, ComponentName, Revision, Version};

fn lib() -> Component {
    Component {
        name: ComponentName::new("lib"),
        outputs: vec![],
        paths: vec!["crates/lib/**".into()],
        prefix: "v".into(),
    }
}

fn versioned(repo: &Repo) -> Versioned {
    match derive(
        &repo.history(),
        &landed(&repo.head()),
        &Rule::default(),
        &lib(),
    )
    .unwrap()
    {
        Derived::Versioned(v) => v,
        Derived::Unchanged { .. } => panic!("lib was not versioned"),
    }
}

/// The notes of `v`, rendered as a publish renders them.
fn notes_of(v: &Versioned) -> String {
    notes::render(
        &v.component,
        &v.version,
        v.last.as_ref().map(|t| t.name.as_str()),
        &v.since,
    )
    .unwrap()
    .text
}

fn short(revision: &Revision) -> &str {
    &revision.as_str()[..7]
}

/// A repository whose `lib` was released at `v1.2.3`.
fn released(fixture: &Fixture) -> Repo {
    let repo = fixture.repo("repo");
    let rev = repo.touch("feat: lib", &["crates/lib"]);
    repo.tag("v1.2.3", &rev);
    repo
}

#[test]
fn a_releases_notes_hold_its_features_and_fixes_and_nothing_of_another_component() {
    let fixture = Fixture::new();
    let repo = released(&fixture);
    let feat = repo.touch("feat: a new call", &["crates/lib"]);
    let fix = repo.touch("fix: a bug", &["crates/lib"]);
    let docs = repo.touch("docs: the site", &["site"]);
    let v = versioned(&repo);
    assert_eq!(v.version.as_str(), "1.3.0");

    let text = notes_of(&v);
    assert!(text.starts_with("## 1.3.0\n"), "{text}");
    let features = text.find("### Features").expect(&text);
    let fixes = text.find("### Fixes").expect(&text);
    let call = text
        .find(&format!("- a new call ({})", short(&feat)))
        .expect(&text);
    let bug = text
        .find(&format!("- a bug ({})", short(&fix)))
        .expect(&text);
    assert!(features < call && call < fixes && fixes < bug, "{text}");
    assert!(!text.contains("the site"), "{text}");
    assert!(!text.contains(short(&docs)), "{text}");
}

#[test]
fn a_prereleases_notes_hold_every_commit_under_the_rest() {
    let fixture = Fixture::new();
    let repo = released(&fixture);
    let one = repo.touch("chore: one", &["crates/lib"]);
    let two = repo.touch("chore: two", &["crates/lib"]);
    let v = versioned(&repo);
    assert_eq!(
        (v.version.as_str(), v.kind),
        ("1.2.4-pre.2", Kind::Prerelease)
    );

    let text = notes_of(&v);
    assert!(text.starts_with("## 1.2.4-pre.2\n"), "{text}");
    let other = text.find("### Other").expect(&text);
    for (subject, rev) in [("one", &one), ("two", &two)] {
        let line = text
            .find(&format!("- {subject} ({})", short(rev)))
            .expect(&text);
        assert!(other < line, "{text}");
    }
    assert!(!text.contains("### Features") && !text.contains("### Fixes"));
}

#[test]
fn a_commit_that_is_not_conventional_is_held_under_the_rest_by_its_subject() {
    let fixture = Fixture::new();
    let repo = released(&fixture);
    let plain = repo.touch(
        "Tidy the library\n\nA body that is not the subject.",
        &["crates/lib"],
    );
    let fix = repo.touch("fix: a bug", &["crates/lib"]);
    let v = versioned(&repo);
    assert_eq!(v.since.len(), 2);

    let text = notes_of(&v);
    let other = text.find("### Other").expect(&text);
    let line = text
        .find(&format!("- Tidy the library ({})", short(&plain)))
        .expect(&text);
    assert!(other < line, "{text}");
    assert!(!text.contains("A body"), "{text}");
    assert!(
        text.contains(&format!("- a bug ({})", short(&fix))),
        "{text}"
    );
}

#[test]
fn two_clones_at_different_paths_and_times_render_the_same_bytes() {
    let fixture = Fixture::new();
    let shared = fixture.bare("shared.git");
    let first = fixture.clone_of(&shared, "first");
    let rev = first.touch("feat: lib", &["crates/lib"]);
    first.tag("v1.2.3", &rev);
    first.touch("feat: a new call", &["crates/lib"]);
    first.touch("fix!: a bug that breaks", &["crates/lib"]);
    first.git(&["push", "-q", "origin", "main", "v1.2.3"]);

    let here = notes_of(&versioned(&first));
    thread::sleep(Duration::from_millis(1100));
    let second = fixture.clone_of(&shared, "elsewhere/second");
    let there = notes_of(&versioned(&second));
    assert_eq!(here.as_bytes(), there.as_bytes());
}

#[test]
fn a_feature_a_fix_a_breaking_change_and_a_plain_commit_render_one_text() {
    let fixture = Fixture::new();
    let repo = released(&fixture);
    let feat = repo.touch("feat(api): a new call", &["crates/lib"]);
    let plain = repo.touch("Tidy the library", &["crates/lib"]);
    let fix = repo.touch("fix: a bug", &["crates/lib"]);
    let breaking = repo.touch("refactor!: drop the old call", &["crates/lib"]);
    let v = versioned(&repo);
    assert_eq!(v.version.as_str(), "2.0.0");

    let expected = format!(
        "## 2.0.0\n\
         \n\
         Since v1.2.3.\n\
         \n\
         ### Features\n\
         \n\
         - a new call ({feat})\n\
         \n\
         ### Fixes\n\
         \n\
         - a bug ({fix})\n\
         \n\
         ### Other\n\
         \n\
         - drop the old call (breaking) ({breaking})\n\
         - Tidy the library ({plain})\n",
        feat = short(&feat),
        fix = short(&fix),
        breaking = short(&breaking),
        plain = short(&plain),
    );
    assert_eq!(notes_of(&v), expected);
}

#[test]
fn a_first_release_names_no_last_tag() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let feat = repo.touch("feat: lib", &["crates/lib"]);
    let v = versioned(&repo);
    assert_eq!(v.last, None);
    assert_eq!(
        notes_of(&v),
        format!("## 0.1.0\n\n### Features\n\n- lib ({})\n", short(&feat))
    );
}

#[test]
fn no_commits_render_the_heading_alone() {
    let rendered = notes::render(
        &ComponentName::new("lib"),
        &Version::new("1.2.4"),
        Some("v1.2.3"),
        &[],
    )
    .unwrap();
    assert_eq!(rendered.text, "## 1.2.4\n\nSince v1.2.3.\n");
    assert_eq!(rendered.bytes(), rendered.text.as_bytes());
}
