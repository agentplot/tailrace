//! Spec landed-policy: the components and the rule, read from the policy
//! file at the landed revision.

mod common;

use std::fs;

use common::{landed, Fixture, Repo};
use semver::Version;
use tailrace_core::{Declared, Rule, POLICY_FILE};
use tailrace_seams::{Component, ComponentName, Not, OutputName, Seam, Why};

const TWO: &str = r#"
[[component]]
name = "lib"
outputs = ["lib-pack"]
paths = ["crates/lib/**"]
prefix = "lib-v"

[[component]]
name = "web"
outputs = ["site", "cli"]
paths = ["site/**", "crates/web/**"]
prefix = "web-v"
"#;

fn component(name: &str, outputs: &[&str], paths: &[&str], prefix: &str) -> Component {
    Component {
        name: ComponentName::new(name),
        outputs: outputs.iter().map(|o| OutputName::new(*o)).collect(),
        paths: paths.iter().map(|p| p.to_string()).collect(),
        prefix: prefix.into(),
    }
}

fn read(repo: &Repo) -> Result<Declared, Not> {
    Declared::read(&repo.history(), &landed(&repo.head()))
}

#[test]
fn a_checkouts_edit_is_not_read() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit("chore: policy", &[(POLICY_FILE, TWO)]);
    fs::write(repo.path.join(POLICY_FILE), "[version]\nprefix = \"x\"\n").unwrap();

    let declared = read(&repo).unwrap();
    assert_eq!(declared.components.len(), 2);
    assert_eq!(declared.rule, Rule::default());
}

#[test]
fn a_file_committed_after_the_revision_is_not_read() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let earlier = repo.commit("chore: policy", &[(POLICY_FILE, TWO)]);
    repo.commit(
        "chore: one component",
        &[(POLICY_FILE, "[version]\nprefix = \"x\"\n")],
    );

    let declared = Declared::read(&repo.history(), &landed(&earlier)).unwrap();
    assert_eq!(
        declared
            .components
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["lib", "web"]
    );
}

#[test]
fn two_components_are_read_each_with_its_own_outputs_paths_and_prefix() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit("chore: policy", &[(POLICY_FILE, TWO)]);
    assert_eq!(
        read(&repo).unwrap().components,
        [
            component("lib", &["lib-pack"], &["crates/lib/**"], "lib-v"),
            component(
                "web",
                &["site", "cli"],
                &["site/**", "crates/web/**"],
                "web-v"
            ),
        ]
    );
}

#[test]
fn two_components_with_one_prefix_are_refused_naming_both() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit(
        "chore: policy",
        &[(POLICY_FILE, &TWO.replace("web-v", "lib-v"))],
    );
    let Err(Not::Refused(r)) = read(&repo) else {
        panic!("one prefix twice was read")
    };
    assert_eq!(
        r.why,
        Why::DeclaredTwice {
            first: ComponentName::new("lib"),
            second: ComponentName::new("web"),
            what: "prefix \"lib-v\"".into(),
        }
    );
    assert_eq!(r.source.seam, Seam::History);
}

#[test]
fn two_components_with_one_name_are_refused_naming_both() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit(
        "chore: policy",
        &[(POLICY_FILE, &TWO.replace("\"web\"", "\"lib\""))],
    );
    let Err(Not::Refused(r)) = read(&repo) else {
        panic!("one name twice was read")
    };
    assert_eq!(
        r.why,
        Why::DeclaredTwice {
            first: ComponentName::new("lib"),
            second: ComponentName::new("lib"),
            what: "name \"lib\"".into(),
        }
    );
}

#[test]
fn no_policy_file_is_one_component_over_every_path_under_the_default_rule() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.touch("feat: one", &["a"]);
    let declared = read(&repo).unwrap();
    assert_eq!(declared.rule, Rule::default());
    assert_eq!(
        declared.components,
        [component("repository", &[], &["**"], "v")]
    );
}

#[test]
fn a_file_with_no_component_is_one_component_under_its_rule() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let policy = r#"
outputs = ["site"]

[version]
prefix = "release-"
initial = "1.0.0"
prerelease = "dev"
features_always_bump_minor = false
breaking_always_bump_major = false

[projections.site]
ecosystem = "a later reader's"
"#;
    repo.commit("chore: policy", &[(POLICY_FILE, policy)]);
    let declared = read(&repo).unwrap();
    assert_eq!(
        declared.rule,
        Rule {
            prefix: "release-".into(),
            initial: Version::new(1, 0, 0),
            prerelease: "dev".into(),
            features_always_bump_minor: false,
            breaking_always_bump_major: false,
        }
    );
    assert_eq!(
        declared.components,
        [component("repository", &["site"], &["**"], "release-")]
    );
}

#[test]
fn a_malformed_file_is_unreadable_naming_the_file_and_revision_with_the_parsers_words() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let rev = repo.commit(
        "chore: policy",
        &[(POLICY_FILE, "[[component]\nname = 1\n")],
    );
    let Err(Not::Unreadable(u)) = read(&repo) else {
        panic!("a malformed file was read")
    };
    assert_eq!(u.what, format!("{POLICY_FILE} at {rev}"));
    let parser = toml::from_str::<toml::Table>("[[component]\nname = 1\n")
        .unwrap_err()
        .to_string();
    assert_eq!(u.said.as_deref(), Some(parser.as_str()));
}

#[test]
fn a_component_missing_a_key_is_unreadable_with_the_parsers_words() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit(
        "chore: policy",
        &[(
            POLICY_FILE,
            "[[component]]\nname = \"lib\"\noutputs = []\npaths = [\"a/**\"]\n",
        )],
    );
    let Err(Not::Unreadable(u)) = read(&repo) else {
        panic!("a component with no prefix was read")
    };
    let said = u.said.unwrap();
    assert!(said.contains("missing field `prefix`"), "{said}");
}
