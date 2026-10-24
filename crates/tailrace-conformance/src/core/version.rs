//! The version's group: a version per component from the history alone,
//! claimed as its name in the shared history (21 to 24, 65, 123, 124).

use tailrace_core::{
    touched, Declared, History, Landed, Publish, Published, Versioned, POLICY_FILE,
};
use tailrace_seams::{Answer, ComponentName, Held, Manifest, Not, Outcome, Revision, Seam, Why};

use crate::core::publish::version_of;
use crate::world::{FakeWorld, POLICY, TWO_COMPONENTS};
use crate::{ok, Scenario};

pub const GROUP: &str = "version";

/// The group runs over [`FakeWorld::two_components`]; a scenario that
/// needs a repository declaring no component makes its own world.
pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        Scenario {
            name: "a landing that touches one component",
            plays: Some("S15"),
            cites: &[123, 124],
            run: a_landing_that_touches_one_component,
        },
        Scenario {
            name: "a path both components name",
            plays: None,
            cites: &[124],
            run: a_path_both_components_name,
        },
        Scenario {
            name: "no component declared",
            plays: None,
            cites: &[123],
            run: no_component_declared,
        },
        Scenario {
            name: "the same revision, two publishers",
            plays: None,
            cites: &[21, 24],
            run: the_same_revision_two_publishers,
        },
        Scenario {
            name: "a name another revision holds",
            plays: None,
            cites: &[22],
            run: a_name_another_revision_holds,
        },
        Scenario {
            name: "versions ordered by their rules",
            plays: None,
            cites: &[23],
            run: versions_ordered_by_their_rules,
        },
        Scenario {
            name: "a checkout's edit is not read",
            plays: None,
            cites: &[65],
            run: a_checkouts_edit_is_not_read,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}

/// `revision` landed with no verdicts, and every output of the two
/// components built and passing there.
fn landed(world: &FakeWorld, revision: &Revision) {
    world.land(revision, &[]);
    world.outputs(
        revision,
        &[("library", Outcome::Pass), ("server", Outcome::Pass)],
    );
}

/// Each component a publish answered, by name, with its answer.
fn components(answer: Answer<Publish>) -> Vec<(String, Answer<Published>)> {
    ok(answer, "the publish")
        .components
        .into_iter()
        .map(|(c, a)| (c.to_string(), a))
        .collect()
}

/// The tags `prefix` begins in both histories.
fn tags(world: &FakeWorld, prefix: &str) -> (Vec<String>, Vec<String>) {
    let of = |tags: Vec<String>| tags.into_iter().filter(|t| t.starts_with(prefix)).collect();
    (of(world.repo().tags()), of(world.shared().tags()))
}

fn a_landing_that_touches_one_component(world: &FakeWorld) {
    landed(world, &world.head);
    let answered = components(world.publish(&world.ask(&world.head)));
    let names: Vec<&str> = answered.iter().map(|(c, _)| c.as_str()).collect();
    assert_eq!(names, ["lib"], "the publish answered other components");
    let published = ok(answered[0].1.clone(), "lib's publish");
    let Published::Version {
        version,
        tag,
        index,
        ..
    } = &published
    else {
        panic!("lib was not versioned: {published:?}")
    };

    // lib: derived from its own tags, claimed under its prefix, its index
    // offered and named.
    assert_eq!(
        (version.as_str(), tag.as_str()),
        ("0.2.0", "lib-v0.2.0"),
        "lib's version is not derived from lib-v0.1.0 and the feature since"
    );
    let lib = vec!["lib-v0.1.0".to_string(), "lib-v0.2.0".into()];
    assert_eq!(
        tags(world, "lib-v"),
        (lib.clone(), lib),
        "lib's name was not claimed"
    );
    let store = world.fakes.store.contents();
    assert_eq!(
        store.names.get(&tailrace_seams::Name::new("lib-v0.2.0")),
        Some(index)
    );

    // app: no version, no tag, no index and no name.
    let app = vec!["app-v0.1.0".to_string()];
    assert_eq!(
        tags(world, "app-v"),
        (app.clone(), app),
        "app's history moved"
    );
    for held in store.held.values() {
        if let Held::Manifest(Manifest::Index(meta)) = held {
            assert_eq!(
                meta.component(),
                &ComponentName::new("lib"),
                "an index of another component was offered"
            );
        }
    }
    assert!(
        !store.names.keys().any(|n| n.as_str().starts_with("app-v")),
        "app was named in the store"
    );

    // The build was told no version, and asked nothing of app.
    for call in world.build_calls() {
        let given = call.given.to_string();
        assert!(
            !given.contains("0.2.0"),
            "the build was told the version: {given}"
        );
        assert!(
            !given.contains("server"),
            "the build was asked of app: {given}"
        );
    }
}

fn a_path_both_components_name(world: &FakeWorld) {
    let repo = world.repo();
    let shared = TWO_COMPONENTS
        .replace(
            r#"paths = ["lib/**"]"#,
            r#"paths = ["lib/**", "shared/**"]"#,
        )
        .replace(
            r#"paths = ["app/**"]"#,
            r#"paths = ["app/**", "shared/**"]"#,
        );
    assert_ne!(shared, TWO_COMPONENTS);
    repo.policy_file(&shared);
    let both = repo.commit(&[("shared/code.txt", "shared")], "fix: the shared code");
    landed(world, &both);
    let answered = components(world.publish(&world.ask(&both)));
    let tags: Vec<String> = answered
        .iter()
        .map(|(c, a)| match a {
            Ok(Published::Version { tag, .. }) => tag.clone(),
            other => panic!("{c} was not versioned: {other:?}"),
        })
        .collect();
    assert_eq!(
        tags,
        ["lib-v0.2.0", "app-v0.1.1"],
        "a commit under a path both name did not version both"
    );
}

fn no_component_declared(_: &FakeWorld) {
    let world = FakeWorld::new();
    let repo = world.repo();
    repo.tag("v0.2.0", &world.head);
    let elsewhere = repo.commit(
        &[("elsewhere/notes.txt", "a note")],
        "fix: a note elsewhere",
    );
    world.land(&elsewhere, &[]);
    world.outputs(
        &elsewhere,
        &[("site", Outcome::Pass), ("server", Outcome::Pass)],
    );
    let answered = components(world.publish(&world.ask(&elsewhere)));
    let names: Vec<&str> = answered.iter().map(|(c, _)| c.as_str()).collect();
    assert_eq!(
        names,
        ["repository"],
        "the repository is not its one component"
    );
    match &answered[0].1 {
        Ok(Published::Version { tag, .. }) => assert_eq!(
            tag, "v0.2.1",
            "a commit anywhere did not version the one component under the rule's prefix"
        ),
        other => panic!("the repository was not versioned: {other:?}"),
    }
}

/// What the rule derives for every component `revision` touched, read
/// from the repository at `path`.
fn derived(world: &FakeWorld, path: &std::path::Path, revision: &Revision) -> Vec<Versioned> {
    let history = ok(History::open(path), "the clone");
    let landed = ok(
        Landed::check(&world.fakes.landing, &world.main, revision),
        "the landing",
    );
    let declared = ok(Declared::read(&history, &landed), "the policy file");
    ok(touched(&history, &landed, &declared), "the versions")
}

fn the_same_revision_two_publishers(world: &FakeWorld) {
    landed(world, &world.head);
    let second = world.fixture.clone_again();
    let here = derived(world, &world.repo().path, &world.head);
    let there = derived(world, &second.path, &world.head);
    assert_eq!(here, there, "two publishers derived two versions");
    assert_eq!(
        derived(world, &world.repo().path, &world.head),
        here,
        "the same revision derived another version"
    );
    assert_eq!(here.len(), 1);
    assert_eq!(here[0].version.as_str(), "0.2.0");
    assert_eq!(world.build_calls(), [], "deriving asked the build");

    ok(world.publish(&world.ask(&world.head)), "the publish");
    for call in world.build_calls() {
        let given = call.given.to_string();
        assert!(
            !given.contains("0.2.0"),
            "the build was told the version: {given}"
        );
    }
}

fn a_name_another_revision_holds(world: &FakeWorld) {
    landed(world, &world.head);
    let second = world.fixture.clone_again();
    let other = second.commit(&[("lib/other.txt", "another lib")], "feat: another lib");
    second.tag("lib-v0.2.0", &other);
    second.git(&["push", "-q", "origin", "refs/tags/lib-v0.2.0"]);

    let answered = components(world.publish(&world.ask(&world.head)));
    assert_eq!(answered.len(), 1);
    match &answered[0].1 {
        Err(Not::Refused(r)) => {
            assert_eq!(
                r.why,
                Why::NameHeld {
                    name: "lib-v0.2.0".into(),
                    revision: world.head.clone(),
                    other: other.clone(),
                },
                "the refusal names other revisions"
            );
            assert_eq!(r.source.seam, Seam::History);
        }
        other => panic!("a name another revision holds answered {other:?}"),
    }
    assert_eq!(world.packs(), [], "an output was packed before the refusal");
    assert_eq!(world.writes(), [], "the store was offered something");
    assert_eq!(
        world.repo().tag_at("lib-v0.2.0"),
        None,
        "the refused claim left a tag"
    );
}

fn versions_ordered_by_their_rules(world: &FakeWorld) {
    let repo = world.repo();
    let ten = repo.commit(&[("lib/lib.txt", "ten")], "feat: ten");
    repo.tag("lib-v1.10.0", &ten);
    let nine = repo.commit(&[("lib/lib.txt", "nine")], "feat: nine");
    repo.tag("lib-v1.9.0", &nine);
    let feature = repo.commit(&[("lib/lib.txt", "a feature")], "feat: a feature lands");
    landed(world, &feature);
    let answered = components(world.publish(&world.ask(&feature)));
    let published = ok(answered[0].1.clone(), "lib's publish");
    let (version, _) = version_of(&published);
    assert_eq!(
        version.as_str(),
        "1.11.0",
        "versions were ordered by time or spelling, not their rules"
    );
}

fn a_checkouts_edit_is_not_read(world: &FakeWorld) {
    landed(world, &world.head);
    world.repo().write(POLICY_FILE, POLICY);
    let answered = components(world.publish(&world.ask(&world.head)));
    let names: Vec<&str> = answered.iter().map(|(c, _)| c.as_str()).collect();
    assert_eq!(
        names,
        ["lib"],
        "the components were read from the checkout, not the landed revision"
    );
}
