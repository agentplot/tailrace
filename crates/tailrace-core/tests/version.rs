//! Spec component-versions: the version a landed revision takes per
//! component, from the history alone.

mod common;

use common::{landed, Fixture, Repo};
use semver::Version;
use tailrace_core::{
    claim, derive, touched, Declared, Derived, Kind, Landed, Rule, Versioned, POLICY_FILE,
};
use tailrace_fakes::Fakes;
use tailrace_seams::{Branch, Component, ComponentName};

fn component(name: &str, paths: &[&str], prefix: &str) -> Component {
    Component {
        name: ComponentName::new(name),
        outputs: vec![],
        paths: paths.iter().map(|p| p.to_string()).collect(),
        prefix: prefix.into(),
    }
}

fn lib() -> Component {
    component("lib", &["crates/lib/**"], "lib-v")
}

fn at_head(repo: &Repo, component: &Component) -> Derived {
    derive(
        &repo.history(),
        &landed(&repo.head()),
        &Rule::default(),
        component,
    )
    .unwrap()
}

fn versioned(derived: Derived) -> Versioned {
    match derived {
        Derived::Versioned(v) => v,
        Derived::Unchanged { component, .. } => panic!("{component} was not versioned"),
    }
}

/// A repository whose `lib` was released at 1.2.3.
fn released(fixture: &Fixture) -> Repo {
    let repo = fixture.repo("repo");
    let rev = repo.touch("feat: lib", &["crates/lib"]);
    repo.tag("lib-v1.2.3", &rev);
    repo
}

#[test]
fn the_same_revision_twice_derives_the_same_version() {
    let fixture = Fixture::new();
    let repo = released(&fixture);
    repo.touch("chore: tidy", &["crates/lib"]);
    repo.touch("feat: more", &["crates/lib"]);
    assert_eq!(at_head(&repo, &lib()), at_head(&repo, &lib()));
}

#[test]
fn a_version_names_its_component_and_carries_its_prefix() {
    let fixture = Fixture::new();
    let repo = released(&fixture);
    repo.touch("fix: lib", &["crates/lib"]);
    let v = versioned(at_head(&repo, &lib()));
    assert_eq!(v.component, ComponentName::new("lib"));
    assert_eq!(v.name, format!("lib-v{}", v.version));
    assert_eq!(v.last.unwrap().name, "lib-v1.2.3");
}

#[test]
fn a_tag_another_publisher_claimed_is_the_last_release() {
    let fixture = Fixture::new();
    let shared = fixture.bare("shared.git");
    let first = fixture.clone_of(&shared, "first");
    let rev = first.touch("feat: lib", &["crates/lib"]);
    first.git(&["push", "-q", "origin", "main"]);
    first.tag("lib-v1.2.3", &rev);
    first.git(&["push", "-q", "origin", "refs/tags/lib-v1.2.3"]);

    let second = fixture.clone_of(&shared, "second");
    second.git(&["tag", "-d", "lib-v1.2.3"]);
    second.touch("fix: lib", &["crates/lib"]);
    let v = versioned(at_head(&second, &lib()));
    assert_eq!(v.version.as_str(), "1.2.4");
    assert_eq!(v.last.unwrap().revision, rev);
}

#[test]
fn a_commit_elsewhere_in_the_repository_gives_no_new_version() {
    let fixture = Fixture::new();
    let repo = released(&fixture);
    repo.touch("feat: site", &["site"]);
    let Derived::Unchanged { component, last } = at_head(&repo, &lib()) else {
        panic!("lib was versioned by a commit under site")
    };
    assert_eq!(component.as_str(), "lib");
    assert_eq!(last.unwrap().name, "lib-v1.2.3");
}

#[test]
fn another_components_tags_are_not_read() {
    let fixture = Fixture::new();
    let repo = released(&fixture);
    let rev = repo.touch("feat: web", &["site"]);
    repo.tag("web-v4.0.0", &rev);
    repo.tag("v9.0.0", &rev);
    repo.touch("fix: lib", &["crates/lib"]);
    let v = versioned(at_head(&repo, &lib()));
    assert_eq!(v.version.as_str(), "1.2.4");
    assert_eq!(v.last.unwrap().name, "lib-v1.2.3");
}

#[test]
fn a_feature_since_the_last_release_makes_a_minor_release() {
    let fixture = Fixture::new();
    let repo = released(&fixture);
    repo.touch("chore: tidy", &["crates/lib"]);
    repo.touch("feat(lib): a new call", &["crates/lib"]);
    repo.touch("fix: lib", &["crates/lib"]);
    let v = versioned(at_head(&repo, &lib()));
    assert_eq!((v.version.as_str(), v.kind), ("1.3.0", Kind::Release));
    assert_eq!(v.name, "lib-v1.3.0");
}

#[test]
fn a_fix_since_the_last_release_makes_a_patch_release() {
    let fixture = Fixture::new();
    let repo = released(&fixture);
    repo.touch("fix: lib", &["crates/lib"]);
    let v = versioned(at_head(&repo, &lib()));
    assert_eq!((v.version.as_str(), v.kind), ("1.2.4", Kind::Release));
}

#[test]
fn a_breaking_change_makes_a_major_release() {
    let fixture = Fixture::new();
    let repo = released(&fixture);
    repo.touch("refactor!: lib", &["crates/lib"]);
    let v = versioned(at_head(&repo, &lib()));
    assert_eq!((v.version.as_str(), v.kind), ("2.0.0", Kind::Release));
}

#[test]
fn only_housekeeping_makes_a_prerelease_of_the_next_patch() {
    let fixture = Fixture::new();
    let repo = released(&fixture);
    repo.touch("chore: one", &["crates/lib"]);
    repo.touch("docs: two", &["crates/lib"]);
    repo.touch("not conventional at all", &["crates/lib"]);
    let v = versioned(at_head(&repo, &lib()));
    assert_eq!(
        (v.version.as_str(), v.kind),
        ("1.2.4-pre.3", Kind::Prerelease)
    );
    let version = Version::parse(v.version.as_str()).unwrap();
    assert!(Version::new(1, 2, 3) < version && version < Version::new(1, 2, 4));
}

#[test]
fn a_component_never_released_takes_the_initial_version() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.touch("feat: lib", &["crates/lib"]);
    let v = versioned(at_head(&repo, &lib()));
    assert_eq!((v.version.as_str(), v.kind), ("0.1.0", Kind::Release));
    assert_eq!(v.last, None);

    repo.touch("chore: tidy", &["crates/lib"]);
    let rule = Rule {
        initial: Version::new(1, 0, 0),
        ..Rule::default()
    };
    let rev = repo.head();
    let unreleased = derive(&repo.history(), &landed(&rev), &rule, &lib()).unwrap();
    // No release tag yet, so both commits since the start count.
    assert_eq!(versioned(unreleased).version.as_str(), "1.0.0");

    let docs = fixture.repo("docs");
    docs.touch("docs: one", &["crates/lib"]);
    docs.touch("docs: two", &["crates/lib"]);
    let v = versioned(derive(&docs.history(), &landed(&docs.head()), &rule, &lib()).unwrap());
    assert_eq!(
        (v.version.as_str(), v.kind),
        ("1.0.0-pre.2", Kind::Prerelease)
    );
}

#[test]
fn a_later_tag_of_a_lower_version_is_not_the_last_release() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let ten = repo.touch("feat: ten", &["crates/lib"]);
    repo.tag("lib-v1.10.0", &ten);
    let nine = repo.touch("feat: nine", &["crates/lib"]);
    repo.tag("lib-v1.9.0", &nine);
    repo.touch("fix: lib", &["crates/lib"]);
    let v = versioned(at_head(&repo, &lib()));
    assert_eq!(v.last.unwrap().name, "lib-v1.10.0");
    assert_eq!(v.version.as_str(), "1.11.0");
}

#[test]
fn a_revision_already_claimed_keeps_its_version() {
    let fixture = Fixture::new();
    let repo = released(&fixture);
    let rev = repo.touch("chore: tidy", &["crates/lib"]);
    repo.tag("lib-v1.3.0", &rev);
    let v = versioned(at_head(&repo, &lib()));
    assert_eq!((v.version.as_str(), v.claimed), ("1.3.0", true));
    assert_eq!(v.name, "lib-v1.3.0");
    assert_eq!(v.last.unwrap().name, "lib-v1.2.3");
}

const TWO: &str = r#"
[[component]]
name = "lib"
outputs = ["lib-pack"]
paths = ["crates/lib/**", "shared/**"]
prefix = "lib-v"

[[component]]
name = "web"
outputs = ["site"]
paths = ["site/**", "shared/**"]
prefix = "web-v"
"#;

fn two_released(fixture: &Fixture) -> Repo {
    let repo = fixture.repo("repo");
    let rev = repo.commit("chore: policy", &[(POLICY_FILE, TWO)]);
    repo.tag("lib-v1.0.0", &rev);
    repo.tag("web-v2.0.0", &rev);
    repo
}

fn touched_at_head(repo: &Repo) -> Vec<Versioned> {
    let history = repo.history();
    let at = landed(&repo.head());
    let declared = Declared::read(&history, &at).unwrap();
    touched(&history, &at, &declared).unwrap()
}

#[test]
fn a_landing_that_touches_one_component_versions_that_one() {
    let fixture = Fixture::new();
    let repo = two_released(&fixture);
    repo.touch("feat: lib", &["crates/lib"]);
    let touched = touched_at_head(&repo);
    assert_eq!(touched.len(), 1);
    assert_eq!(touched[0].component.as_str(), "lib");
    assert_eq!(touched[0].name, "lib-v1.1.0");
}

#[test]
fn a_path_two_components_name_versions_both() {
    let fixture = Fixture::new();
    let repo = two_released(&fixture);
    repo.touch("fix: shared", &["shared"]);
    let names: Vec<_> = touched_at_head(&repo).into_iter().map(|v| v.name).collect();
    assert_eq!(names, ["lib-v1.0.1", "web-v2.0.1"]);
}

#[test]
fn deriving_and_claiming_ask_nothing_of_the_build() {
    let fixture = Fixture::new();
    let repo = two_released(&fixture);
    repo.touch("feat: shared", &["shared"]);
    let fakes = Fakes::new();
    let seams = fakes.seams();
    let (main, rev) = (Branch::new("main"), repo.head());
    fakes.landing.land(&main, &rev);

    let history = repo.history();
    let at = Landed::check(seams.landing.as_ref(), &main, &rev).unwrap();
    let declared = Declared::read(&history, &at).unwrap();
    let touched = touched(&history, &at, &declared).unwrap();
    assert_eq!(touched.len(), 2);
    for v in &touched {
        claim(&history, &at, v).unwrap();
    }
    assert!(fakes.build.calls().is_empty(), "the build was asked");
}
