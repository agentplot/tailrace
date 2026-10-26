//! Spec version-notes: a version's notes rendered from the commits it is
//! derived from, by the rule's own tool under the repository's own
//! template for it, the same bytes on every host (134 as amended at
//! blueprints a3efbd8).

mod common;

use std::fs;
use std::thread;
use std::time::Duration;

use common::{landed, Fixture, Repo, CLIFF_TOML};
use tailrace_core::{derive, notes, Derived, Kind, Noted, Rule, Versioned};
use tailrace_seams::{Component, ComponentName, Not, Revision, Version};

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

/// The notes of `v` at `repo`'s head, rendered as a publish renders them.
fn notes_of(repo: &Repo, v: &Versioned) -> String {
    rendered(repo, v).unwrap()
}

fn rendered(repo: &Repo, v: &Versioned) -> Result<String, Not> {
    notes::render(
        &repo.history(),
        &v.component,
        &v.version,
        &v.name,
        &repo.head(),
        &Noted::of(v),
    )
    .map(|r| r.text)
}

fn short(revision: &Revision) -> &str {
    &revision.as_str()[..7]
}

/// A repository holding its own `cliff.toml`.
fn templated(fixture: &Fixture) -> Repo {
    let repo = fixture.repo("repo");
    repo.commit("chore: the notes' template", &[("cliff.toml", CLIFF_TOML)]);
    repo
}

/// A repository holding its own `cliff.toml`, whose `lib` was released
/// at `v1.2.3`.
fn released(fixture: &Fixture) -> Repo {
    let repo = templated(fixture);
    let rev = repo.touch("feat: lib", &["crates/lib"]);
    repo.tag("v1.2.3", &rev);
    repo
}

// Requirement: The notes are rendered from the commits the version is
// derived from

#[test]
fn a_releases_notes_hold_its_features_and_fixes_and_nothing_of_another_component() {
    let fixture = Fixture::new();
    let repo = released(&fixture);
    let feat = repo.touch("feat: a new call", &["crates/lib"]);
    let fix = repo.touch("fix: a bug", &["crates/lib"]);
    let docs = repo.touch("docs: the site", &["site"]);
    let v = versioned(&repo);
    assert_eq!(v.version.as_str(), "1.3.0");

    let text = notes_of(&repo, &v);
    assert!(text.starts_with("## v1.3.0\n"), "{text}");
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

    let text = notes_of(&repo, &v);
    assert!(text.starts_with("## v1.2.4-pre.2\n"), "{text}");
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

    let text = notes_of(&repo, &v);
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
fn a_feature_a_fix_a_breaking_change_and_a_plain_commit_render_under_the_repositorys_template() {
    let fixture = Fixture::new();
    let repo = released(&fixture);
    let feat = repo.touch("feat(api): a new call", &["crates/lib"]);
    let plain = repo.touch("Tidy the library", &["crates/lib"]);
    let fix = repo.touch("fix: a bug", &["crates/lib"]);
    let breaking = repo.touch("refactor!: drop the old call", &["crates/lib"]);
    let v = versioned(&repo);
    assert_eq!(v.version.as_str(), "2.0.0");

    // The template is handed the tag, the last release tag, and each
    // commit oldest first with its author, as git-cliff hands them.
    let expected = format!(
        "## v2.0.0\n\
         \n\
         Since v1.2.3.\n\
         \n\
         ### Features\n\
         \n\
         - a new call ({feat}) by A. Uthor\n\
         \n\
         ### Fixes\n\
         \n\
         - a bug ({fix}) by A. Uthor\n\
         \n\
         ### Other\n\
         \n\
         - Tidy the library ({plain}) by A. Uthor\n\
         - drop the old call (breaking) ({breaking}) by A. Uthor\n",
        feat = short(&feat),
        fix = short(&fix),
        breaking = short(&breaking),
        plain = short(&plain),
    );
    assert_eq!(notes_of(&repo, &v), expected);
}

#[test]
fn a_first_release_names_no_last_tag() {
    let fixture = Fixture::new();
    let repo = templated(&fixture);
    let feat = repo.touch("feat: lib", &["crates/lib"]);
    let v = versioned(&repo);
    assert_eq!(v.last, None);
    assert_eq!(
        notes_of(&repo, &v),
        format!(
            "## v0.1.0\n\n### Features\n\n- lib ({}) by A. Uthor\n",
            short(&feat)
        )
    );
}

#[test]
fn no_commits_render_the_heading_alone() {
    let fixture = Fixture::new();
    let repo = released(&fixture);
    let rendered = notes::render(
        &repo.history(),
        &ComponentName::new("lib"),
        &Version::new("1.2.4"),
        "v1.2.4",
        &repo.head(),
        &Noted {
            last: Some("v1.2.3".into()),
            commits: vec![],
        },
    )
    .unwrap();
    assert_eq!(rendered.text, "## v1.2.4\n\nSince v1.2.3.\n");
    assert_eq!(rendered.bytes(), rendered.text.as_bytes());
}

// Requirement: The notes render under the repository's own template, and
// tailrace imposes none

#[test]
fn a_repository_with_no_template_renders_under_git_cliffs_own_default() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let rev = repo.touch("feat: lib", &["crates/lib"]);
    repo.tag("v1.2.3", &rev);
    repo.touch("feat(api): a new call", &["crates/lib"]);
    repo.touch("fix: a bug", &["crates/lib"]);
    let v = versioned(&repo);

    // git-cliff's default: the tag bare of its `v`, the day of the tagged
    // revision, never the day it is rendered on.
    let day = repo.day(&repo.head());
    assert_eq!(
        notes_of(&repo, &v),
        format!(
            "## [1.3.0] - {day}\n\
             \n\
             ### 🚀 Features\n\
             \n\
             - *(api)* A new call\n\
             \n\
             ### 🐛 Bug Fixes\n\
             \n\
             - A bug\n"
        )
    );
}

#[test]
fn a_template_under_git_cliffs_other_names_is_the_repositorys_own() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit(
        "chore: the notes' template",
        &[(".config/cliff.toml", CLIFF_TOML)],
    );
    let feat = repo.touch("feat: lib", &["crates/lib"]);
    let v = versioned(&repo);
    assert_eq!(
        notes_of(&repo, &v),
        format!(
            "## v0.1.0\n\n### Features\n\n- lib ({}) by A. Uthor\n",
            short(&feat)
        )
    );
}

#[test]
fn the_template_is_the_one_the_versions_revision_holds_never_the_checkouts() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.touch("feat: lib", &["crates/lib"]);
    let v = versioned(&repo);
    let given = notes_of(&repo, &v);

    // A template in the working tree alone is no template of the revision.
    fs::write(repo.path.join("cliff.toml"), CLIFF_TOML).unwrap();
    assert_eq!(notes_of(&repo, &v), given);
    assert!(given.starts_with("## [0.1.0] - "), "{given}");

    // A template committed after the revision is not the revision's.
    let at = repo.head();
    repo.commit("chore: the notes' template", &[("cliff.toml", CLIFF_TOML)]);
    let later = notes::render(
        &repo.history(),
        &v.component,
        &v.version,
        &v.name,
        &at,
        &Noted::of(&v),
    )
    .unwrap();
    assert_eq!(later.text, given);
}

#[test]
fn a_template_that_does_not_parse_is_unreadable_naming_it_at_the_revision() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit(
        "chore: the notes' template",
        &[("cliff.toml", "[changelog\nbody = 1")],
    );
    repo.touch("feat: lib", &["crates/lib"]);
    let v = versioned(&repo);
    let Err(Not::Unreadable(u)) = rendered(&repo, &v) else {
        panic!("a template that does not parse rendered notes")
    };
    assert_eq!(u.what, format!("cliff.toml at {}", repo.head()));
    assert!(u.said.is_some(), "{u:?}");
}

#[test]
fn a_template_git_cliff_cannot_render_is_unreadable_naming_the_notes() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit(
        "chore: the notes' template",
        &[("cliff.toml", "[changelog]\nbody = \"{{ version \"\n")],
    );
    repo.touch("feat: lib", &["crates/lib"]);
    let v = versioned(&repo);
    let Err(Not::Unreadable(u)) = rendered(&repo, &v) else {
        panic!("a template git-cliff cannot render rendered notes")
    };
    assert_eq!(u.what, "the notes of lib 0.1.0");
    assert!(u.said.is_some(), "{u:?}");
}

// Requirement: The same commits render the same notes

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

    // git-cliff's default prints the release's day, the tagged revision's.
    let here = notes_of(&first, &versioned(&first));
    thread::sleep(Duration::from_millis(1100));
    let second = fixture.clone_of(&shared, "elsewhere/second");
    let there = notes_of(&second, &versioned(&second));
    assert_eq!(here.as_bytes(), there.as_bytes());
}
