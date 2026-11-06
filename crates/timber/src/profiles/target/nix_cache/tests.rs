use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use serde_json::{json, Value};
use tempfile::TempDir;
use timber_fakes::FakeStore;
use timber_seams::{
    Artifact, ArtifactMeta, ChannelId, ChannelName, ComponentName, Ecosystem, EcosystemName,
    EnvironmentName, IndexMeta, Manifest, Version,
};

use super::*;

const VERSION: &str = "0.4.0";
const COMPONENT: &str = "swancloud-srv";

/// A root of the machine's closure, under this host's store directory.
fn root() -> String {
    format!(
        "{}/0c7yzjm6jy3dxfyb8zd6x0fv1r6l1sbm-nixos-system-srv-25.05",
        nix_closure::store_dir()
    )
}

/// A second root, of another closure.
fn other_root() -> String {
    format!(
        "{}/9kn2wd8q1ycx0l6hzm4fjrb3gv5sap7i-hello-2.12.1",
        nix_closure::store_dir()
    )
}

fn cache(settings: Value) -> Environment {
    Environment {
        name: EnvironmentName::new("cache"),
        channel: ChannelId::new(
            Some(ComponentName::new(COMPONENT)),
            ChannelName::new("beta"),
        ),
        waits: false,
        settings,
    }
}

fn swancloud() -> Environment {
    cache(json!({ "attic": "fleet:swancloud", "closures": [COMPONENT] }))
}

/// A stand-in attic and nix-store, written by the test, each recording
/// the arguments it was given one to a line; nix-store prints the paths
/// of `invalid` as `--print-invalid` does, and each fails with the words
/// of its `fails` where the test writes one.
struct Programs {
    dir: TempDir,
}

impl Programs {
    fn new() -> Programs {
        let dir = TempDir::new().unwrap();
        let at = dir.path().display().to_string();
        let attic = format!(
            "#!/bin/sh\n\
             for arg in \"$@\"; do printf '%s\\n' \"$arg\"; done > '{at}/attic.args'\n\
             env > '{at}/attic.env'\n\
             if [ -f '{at}/attic.fails' ]; then cat '{at}/attic.fails' >&2; exit 1; fi\n"
        );
        let nix_store = format!(
            "#!/bin/sh\n\
             for arg in \"$@\"; do printf '%s\\n' \"$arg\"; done > '{at}/nix-store.args'\n\
             if [ -f '{at}/nix-store.fails' ]; then cat '{at}/nix-store.fails' >&2; exit 1; fi\n\
             if [ -f '{at}/invalid' ]; then cat '{at}/invalid'; fi\n"
        );
        for (name, script) in [("attic", attic), ("nix-store", nix_store)] {
            let program = dir.path().join(name);
            fs::write(&program, script).unwrap();
            fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
        }
        Programs { dir }
    }

    fn program(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }

    fn target(&self, store: &FakeStore) -> NixCache {
        NixCache::with_programs(
            Box::new(store.clone()),
            self.program("attic"),
            self.program("nix-store"),
        )
    }

    /// The paths this host's store does not hold.
    fn lacks(&self, paths: &[String]) {
        let mut text = paths.join("\n");
        text.push('\n');
        fs::write(self.dir.path().join("invalid"), text).unwrap();
    }

    /// `program` fails from now, with `words` on its stderr.
    fn fails(&self, program: &str, words: &str) {
        fs::write(self.dir.path().join(format!("{program}.fails")), words).unwrap();
    }

    /// The arguments `program` was given, or `None` where it never ran.
    fn args(&self, program: &str) -> Option<Vec<String>> {
        fs::read_to_string(self.dir.path().join(format!("{program}.args")))
            .ok()
            .map(|args| args.lines().map(str::to_string).collect())
    }

    /// The names of the variables attic ran with.
    fn attic_env(&self) -> BTreeSet<String> {
        let env = fs::read_to_string(self.dir.path().join("attic.env")).unwrap();
        env.lines()
            .filter_map(|line| line.split_once('=').map(|(name, _)| name))
            .filter(|name| {
                !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            })
            .map(str::to_string)
            .collect()
    }
}

/// A version of `COMPONENT` published into `store`, each of `files` an
/// artifact of the ecosystem it names, as the file projection packs one:
/// a manifest naming one layer of the file's bytes; and the version
/// verified, as the core makes it.
fn publish(store: &FakeStore, files: &[(&str, &str, &str)]) -> Verified {
    let mut artifacts = Vec::new();
    for (ecosystem, name, text) in files {
        let layer = store.put_bytes(text.as_bytes()).unwrap().digest;
        let meta = ArtifactMeta::new(
            Ecosystem::new(*ecosystem),
            EcosystemName::new(*name),
            Version::new(VERSION),
        )
        .unwrap();
        let manifest = Manifest::Artifact {
            meta: meta.clone(),
            layers: vec![layer],
        };
        let digest = store.put_artifact(&manifest).unwrap().digest;
        artifacts.push(Artifact { digest, meta });
    }
    let index = IndexMeta::new(
        ComponentName::new(COMPONENT),
        Version::new(VERSION),
        artifacts,
    )
    .unwrap();
    let digest = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
    Verified::new(digest, Version::new(VERSION))
}

fn closure_of(roots: &[String]) -> String {
    let mut text = roots.join("\n");
    text.push('\n');
    text
}

fn refused_said(answer: Answer<()>) -> String {
    match answer {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source, NixCache::source());
            assert_eq!(r.why, Why::Said);
            r.said.expect("the target's words")
        }
        other => panic!("answered {other:?}, not refused"),
    }
}

fn unreadable(answer: Answer<impl std::fmt::Debug>) -> (String, String) {
    match answer {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, NixCache::source());
            (u.what, u.said.expect("the words"))
        }
        other => panic!("answered {other:?}, not unreadable"),
    }
}

fn index(n: u8) -> Digest {
    Digest::of(&[b'i', n])
}

// Requirement: The target's table names the cache and the closures, and
// nothing else

#[test]
fn a_token_in_the_table_is_unreadable_at_every_operation_and_attic_is_not_run() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
    let target = programs.target(&store);
    let environment = cache(json!({
        "attic": "fleet:swancloud",
        "closures": [COMPONENT],
        "token": "eyJhbGciOi",
    }));
    let named = "setting token of environment cache of swancloud-srv";
    let (what, said) = unreadable(target.apply(&environment, &version));
    assert_eq!(what, named);
    assert!(said.contains("no setting token"), "{said}");
    assert_eq!(
        unreadable(target.desired(&environment, &|| Ok(index(3)))).0,
        named
    );
    assert_eq!(unreadable(target.current(&environment)).0, named);
    assert_eq!(unreadable(target.report(&environment)).0, named);
    assert_eq!(programs.args("attic"), None, "attic ran");
    assert_eq!(programs.args("nix-store"), None, "nix-store ran");
}

#[test]
fn a_cache_with_no_server_is_unreadable_saying_it_is_server_and_cache() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
    let environment = cache(json!({ "attic": "swancloud", "closures": [COMPONENT] }));
    let (what, said) = unreadable(programs.target(&store).apply(&environment, &version));
    assert_eq!(what, "setting attic of environment cache of swancloud-srv");
    assert!(said.contains("<server>:<cache>"), "{said}");
    assert_eq!(programs.args("attic"), None, "attic ran");
}

#[test]
fn an_empty_closures_is_unreadable_naming_it() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
    let environment = cache(json!({ "attic": "fleet:swancloud", "closures": [] }));
    let (what, _) = unreadable(programs.target(&store).apply(&environment, &version));
    assert_eq!(
        what,
        "setting closures of environment cache of swancloud-srv"
    );
    assert_eq!(programs.args("attic"), None, "attic ran");
}

// Requirement: desired is the channel's head

#[test]
fn desired_answers_the_head_it_is_handed() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let target = programs.target(&store);
    assert_eq!(target.desired(&swancloud(), &|| Ok(index(3))), Ok(index(3)));
    let empty = || -> Answer<Digest> {
        Err(Not::absent(
            &Source::new(Seam::Keeper, "fake"),
            "the head of the chain of swancloud-srv/beta",
        ))
    };
    assert_eq!(target.desired(&swancloud(), &empty), empty());
}

// Requirement: current and report are unread, since a cache keeps no
// marker

#[test]
fn current_and_report_are_unread_naming_the_cache_even_after_an_apply() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
    let target = programs.target(&store);
    target.apply(&swancloud(), &version).unwrap();
    let applied = version.index().to_string();
    for (what, said) in [
        unreadable(target.current(&swancloud())),
        unreadable(target.report(&swancloud())),
    ] {
        assert_eq!(what, "what environment cache of swancloud-srv runs");
        assert_eq!(
            said,
            "the Nix binary cache fleet:swancloud holds store paths by their hash and keeps no \
             marker of what was pushed to it"
        );
        assert!(!said.contains(&applied) && !what.contains(&applied));
    }
}

// Requirement: Apply pushes each root with its closure through attic

#[test]
fn attic_is_given_the_cache_and_the_roots_and_nothing_else() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(
        &store,
        &[
            ("file", COMPONENT, &closure_of(&[root()])),
            ("file", "tools", &closure_of(&[other_root(), root()])),
        ],
    );
    let environment = cache(json!({
        "attic": "fleet:swancloud",
        "closures": [COMPONENT, "tools"],
    }));
    let before: BTreeSet<String> = std::env::vars_os()
        .map(|(name, _)| name.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        programs.target(&store).apply(&environment, &version),
        Ok(())
    );
    assert_eq!(
        programs.args("nix-store").unwrap(),
        [
            "--check-validity",
            "--print-invalid",
            &root(),
            &other_root()
        ]
    );
    assert_eq!(
        programs.args("attic").unwrap(),
        ["push", "fleet:swancloud", &root(), &other_root()]
    );
    let set: Vec<String> = programs
        .attic_env()
        .into_iter()
        .filter(|name| !before.contains(name))
        .filter(|name| !["PWD", "OLDPWD", "SHLVL", "_"].contains(&name.as_str()))
        .collect();
    assert!(set.is_empty(), "timber set {set:?} for attic");
}

#[test]
fn attic_failing_is_unreadable_in_its_own_words() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
    let words = "Error: Unauthorized: You are not authorized to push to cache swancloud";
    programs.fails("attic", &format!("{words}\n"));
    let (what, said) = unreadable(programs.target(&store).apply(&swancloud(), &version));
    assert_eq!(said, words);
    assert!(
        what.ends_with(&format!("attic push fleet:swancloud {}", root())),
        "{what}"
    );
}

// Requirement: Apply pushes only from the host that holds the closure

#[test]
fn a_root_this_host_does_not_hold_is_refused_naming_it_and_the_host_and_attic_never_runs() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(
        &store,
        &[("file", COMPONENT, &closure_of(&[other_root(), root()]))],
    );
    programs.lacks(&[root()]);
    let said = refused_said(programs.target(&store).apply(&swancloud(), &version));
    assert!(
        said.starts_with(&format!("{} is not valid in the Nix store of ", root())),
        "{said}"
    );
    assert!(said.contains(&host()), "{said}");
    assert!(said.contains("building and fetching nothing"), "{said}");
    assert_eq!(programs.args("attic"), None, "attic ran");
}

#[test]
fn nix_store_failing_is_unreadable_in_its_own_words_and_attic_never_runs() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
    let words = "error: cannot connect to socket at '/nix/var/nix/daemon-socket/socket'";
    programs.fails("nix-store", words);
    let (_, said) = unreadable(programs.target(&store).apply(&swancloud(), &version));
    assert_eq!(said, words);
    assert_eq!(programs.args("attic"), None, "attic ran");
}

// Requirement: Apply reads each closure the version names by digest

#[test]
fn a_closure_output_the_version_lacks_is_refused_naming_it_and_the_version() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(&store, &[("eco", COMPONENT, "not a closure")]);
    let said = refused_said(programs.target(&store).apply(&swancloud(), &version));
    assert_eq!(
        said,
        "version 0.4.0 of swancloud-srv names no file artifact of the output swancloud-srv, so \
         it carries no closure of it"
    );
    assert_eq!(programs.args("nix-store"), None, "nix-store ran");
    assert_eq!(programs.args("attic"), None, "attic ran");
}

#[test]
fn a_line_that_is_no_store_path_is_refused_naming_it_and_nothing_runs() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(&store, &[("file", COMPONENT, "result\n")]);
    let said = refused_said(programs.target(&store).apply(&swancloud(), &version));
    assert!(said.contains("`result`"), "{said}");
    assert!(
        said.contains("the closure file swancloud-srv@0.4.0"),
        "{said}"
    );
    assert_eq!(programs.args("nix-store"), None, "nix-store ran");
    assert_eq!(programs.args("attic"), None, "attic ran");
}

#[test]
fn a_closure_file_whose_bytes_are_not_its_layers_is_refused_and_nothing_runs() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let text = closure_of(&[root()]);
    let version = publish(&store, &[("file", COMPONENT, &text)]);
    let layer = Digest::of(text.as_bytes());
    store.corrupt(&layer, closure_of(&[other_root()]).as_bytes());
    match programs.target(&store).apply(&swancloud(), &version) {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source, NixCache::source());
            assert!(
                matches!(&r.why, Why::OtherBytes { expected, .. } if expected == &layer),
                "{:?}",
                r.why
            );
        }
        other => panic!("answered {other:?}"),
    }
    assert_eq!(programs.args("nix-store"), None, "nix-store ran");
}

#[test]
fn an_index_the_store_does_not_hold_is_unreadable_naming_the_version() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = Verified::new(index(9), Version::new(VERSION));
    let (what, said) = unreadable(programs.target(&store).apply(&swancloud(), &version));
    assert_eq!(what, "the index of version 0.4.0");
    assert_eq!(said, format!("the store holds no {}", index(9)));
}

#[test]
fn an_index_of_another_version_than_the_one_verified_is_refused() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let published = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
    let version = Verified::new(published.index().clone(), Version::new("0.5.0"));
    let said = refused_said(programs.target(&store).apply(&swancloud(), &version));
    assert!(
        said.ends_with("names version 0.4.0, and the version verified is 0.5.0"),
        "{said}"
    );
    assert_eq!(programs.args("nix-store"), None, "nix-store ran");
}
