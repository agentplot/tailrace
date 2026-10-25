//! Spec policy-file: the policy read from the file at a revision of a
//! scratch repository, each test with a repository of its own.

use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

use super::*;

/// A host key's listing, as a policy names one.
const LAPTOP_KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOv host@laptop";

/// A git repository in a temporary directory, committed to under a fixed
/// author and date with no global or system configuration.
struct Repo {
    dir: TempDir,
}

impl Repo {
    fn new() -> Repo {
        let repo = Repo {
            dir: TempDir::new().unwrap(),
        };
        repo.git(&["init", "-q", "-b", "main"]);
        repo
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn git(&self, args: &[&str]) -> String {
        let mut command = Command::new("git");
        for name in GIT_REPOSITORY_ENV {
            command.env_remove(name);
        }
        let out = command
            .arg("-C")
            .arg(self.path())
            .args(args)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_AUTHOR_NAME", "an author")
            .env("GIT_AUTHOR_EMAIL", "author@example.org")
            .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z")
            .env("GIT_COMMITTER_NAME", "an author")
            .env("GIT_COMMITTER_EMAIL", "author@example.org")
            .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

    /// The policy file written in the working tree, nothing committed.
    fn write(&self, text: &str) {
        let at = self.path().join(POLICY_FILE);
        fs::create_dir_all(at.parent().unwrap()).unwrap();
        fs::write(at, text).unwrap();
    }

    /// Everything written, committed; answers the commit.
    fn commit(&self) -> Revision {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "--allow-empty", "-m", "the policy"]);
        Revision::new(self.git(&["rev-parse", "HEAD"]))
    }

    /// `text` as the policy file of a new commit.
    fn at(&self, text: &str) -> Revision {
        self.write(text);
        self.commit()
    }

    fn policy(&self) -> PolicyFile {
        PolicyFile::new(self.path())
    }
}

fn laptop() -> NamedSigner {
    NamedSigner {
        id: SignerId::new("laptop"),
        verified_by: LAPTOP_KEY.into(),
        kinds: vec![ClaimKind::Provenance],
        checks: vec![],
    }
}

/// The two signers of design D3, the laptop's key and the gate's.
fn signers() -> String {
    format!(
        "[signers.laptop]\nssh = \"{LAPTOP_KEY}\"\nkinds = [\"provenance\"]\n\n\
         [signers.box]\nssh = \"ssh-ed25519 BBBB\"\nkinds = [\"verdict\"]\nchecks = [\"cli:test\", \"core:test\"]\n\n"
    )
}

/// The words of an unreadable answer, once it names the file and the
/// revision.
fn unreadable<T: std::fmt::Debug>(answer: Answer<T>, revision: &Revision) -> String {
    match answer {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, source());
            assert_eq!(u.what, format!("{POLICY_FILE} at {revision}"));
            u.said.expect("the words it was read with")
        }
        other => panic!("answered {other:?}, not unreadable"),
    }
}

/// That `answer` is absent from this profile, naming `audience`.
fn absent(answer: Answer<Required>, audience: &str) {
    match answer {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source, source());
            assert!(a.what.contains(audience), "{a:?} names no {audience}");
        }
        other => panic!("answered {other:?}, not absent"),
    }
}

#[test]
fn a_host_key_listed_is_a_signer_with_its_verifier_and_reach() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}[acceptance]\nrequire = [{{ kind = \"provenance\", signers = [\"laptop\"] }}]\n",
        signers()
    ));
    let required = repo
        .policy()
        .policy_for(&at, &Audience::Acceptance)
        .unwrap();
    assert_eq!(
        required,
        Required::Acceptance {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Provenance,
                check: None,
                signers: vec![laptop()],
            }],
        }
    );
}

#[test]
fn a_signer_with_two_verifiers_is_unreadable_naming_its_table() {
    let repo = Repo::new();
    let at = repo.at(
        "[signers.laptop]\nssh = \"ssh-ed25519 AAAA\"\nkms = \"a key's name\"\nkinds = [\"provenance\"]\n\n\
         [acceptance]\nrequire = [{ kind = \"provenance\", signers = [\"laptop\"] }]\n",
    );
    let said = unreadable(repo.policy().policy_for(&at, &Audience::Acceptance), &at);
    assert_eq!(
        said,
        "[signers.laptop] names kms, ssh, and a signer is verified by one key"
    );
}

#[test]
fn acceptance_requires_its_claims_in_the_order_written() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}[acceptance]\nrequire = [\n  {{ kind = \"provenance\", signers = [\"laptop\"] }},\n  \
         {{ kind = \"verdict\", check = \"cli:test\", signers = [\"box\"] }},\n]\n",
        signers()
    ));
    let required = repo
        .policy()
        .policy_for(&at, &Audience::Acceptance)
        .unwrap();
    let claims = required.claims();
    assert_eq!(claims.len(), 2, "{claims:?}");
    assert_eq!(
        (claims[0].kind, &claims[0].check),
        (ClaimKind::Provenance, &None)
    );
    assert_eq!(claims[0].signers, [laptop()]);
    assert_eq!(
        (claims[1].kind, &claims[1].check),
        (ClaimKind::Verdict, &Some(Check::new("cli:test")))
    );
    assert_eq!(
        claims[1].signers,
        [NamedSigner {
            id: SignerId::new("box"),
            verified_by: "ssh-ed25519 BBBB".into(),
            kinds: vec![ClaimKind::Verdict],
            checks: vec![Check::new("cli:test"), Check::new("core:test")],
        }]
    );
}

#[test]
fn a_bound_channel_names_the_channel_it_is_bound_to() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}[channels.beta]\nrequire = []\n\n\
         [channels.stable]\nbound_to = \"beta\"\nfollows = \"left to its reader\"\n\
         require = [{{ kind = \"provenance\", signers = [\"laptop\"] }}]\n",
        signers()
    ));
    let stable = repo
        .policy()
        .policy_for(&at, &Audience::Channel(ChannelName::new("stable")))
        .unwrap();
    assert_eq!(
        stable,
        Required::Channel {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Provenance,
                check: None,
                signers: vec![laptop()],
            }],
            bound_to: Some(ChannelName::new("beta")),
        }
    );
    absent(
        repo.policy()
            .policy_for(&at, &Audience::Channel(ChannelName::new("edge"))),
        "the channel edge",
    );
}

#[test]
fn acceptance_naming_a_channel_is_unreadable_naming_its_table() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}[acceptance]\nbound_to = \"beta\"\nrequire = []\n",
        signers()
    ));
    let said = unreadable(repo.policy().policy_for(&at, &Audience::Acceptance), &at);
    assert!(said.starts_with("[acceptance] names bound_to"), "{said}");
}

#[test]
fn a_signer_the_signers_table_lacks_is_unreadable_naming_it() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}[layout]\nrequire = [{{ kind = \"acceptance\", signers = [\"gate\"] }}]\n",
        signers()
    ));
    let said = unreadable(repo.policy().policy_for(&at, &Audience::Layout), &at);
    assert_eq!(
        said,
        "[layout] requires a claim of the signer gate, which [signers] does not name"
    );
}

#[test]
fn no_file_and_no_table_are_absent_naming_the_audience() {
    let repo = Repo::new();
    fs::write(repo.path().join("README"), "no policy").unwrap();
    let none = repo.commit();
    absent(
        repo.policy().policy_for(&none, &Audience::Acceptance),
        "acceptance",
    );
    let at = repo.at(&format!(
        "{}[acceptance]\nrequire = [{{ kind = \"provenance\", signers = [\"laptop\"] }}]\n",
        signers()
    ));
    absent(repo.policy().policy_for(&at, &Audience::Layout), "layout");
}

#[test]
fn a_malformed_file_is_unreadable_with_the_parsers_words() {
    let repo = Repo::new();
    let text = "[signers.laptop\nssh = \"x\"\n";
    let at = repo.at(text);
    let said = unreadable(repo.policy().policy_for(&at, &Audience::Acceptance), &at);
    let words = toml::from_str::<toml::Table>(text).unwrap_err().to_string();
    assert_eq!(said, words);

    let at = repo.at("[signers.laptop]\nssh = \"x\"\nkinds = [\"blessing\"]\n");
    let said = unreadable(repo.policy().policy_for(&at, &Audience::Acceptance), &at);
    assert!(said.starts_with("[signers.laptop] kinds: "), "{said}");
    assert!(said.contains("blessing"), "{said}");
}

#[test]
fn a_working_trees_edit_is_not_read() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}[acceptance]\nrequire = [{{ kind = \"provenance\", signers = [\"laptop\"] }}]\n",
        signers()
    ));
    repo.write(&format!(
        "{}[signers.stranger]\nssh = \"ssh-ed25519 CCCC\"\nkinds = [\"provenance\"]\n\n\
         [acceptance]\nrequire = [{{ kind = \"provenance\", signers = [\"stranger\"] }}]\n",
        signers()
    ));
    let required = repo
        .policy()
        .policy_for(&at, &Audience::Acceptance)
        .unwrap();
    let named: Vec<&str> = required
        .claims()
        .iter()
        .flat_map(|c| &c.signers)
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(named, ["laptop"]);
}

#[test]
fn a_revision_the_repository_does_not_hold_is_absent() {
    let repo = Repo::new();
    repo.at("");
    let missing = Revision::new("0000000000000000000000000000000000000001");
    assert_eq!(
        repo.policy().policy_for(&missing, &Audience::Acceptance),
        Err(Not::absent(&source(), format!("revision {missing}")))
    );
}

// Requirement: A channel's table names its component

/// A `[[component]]` table with every key the core requires.
fn component(name: &str) -> String {
    format!(
        "[[component]]\nname = \"{name}\"\noutputs = []\npaths = [\"{name}/**\"]\nprefix = \"{name}-v\"\n\n"
    )
}

/// `[channels.<name>]` requiring nothing, in `component` and bound to
/// `bound_to` where each is named.
fn channel(name: &str, component: Option<&str>, bound_to: Option<&str>) -> String {
    let mut table = format!("[channels.{name}]\nrequire = []\n");
    if let Some(component) = component {
        table.push_str(&format!("component = \"{component}\"\n"));
    }
    if let Some(bound_to) = bound_to {
        table.push_str(&format!("bound_to = \"{bound_to}\"\n"));
    }
    table + "\n"
}

fn declared(name: &str, component: &str, bound_to: Option<&str>) -> DeclaredChannel {
    DeclaredChannel {
        name: ChannelName::new(name),
        component: ComponentName::new(component),
        bound_to: bound_to.map(ChannelName::new),
    }
}

fn channel_policy(repo: &Repo, at: &Revision, name: &str) -> Answer<Required> {
    repo.policy()
        .policy_for(at, &Audience::Channel(ChannelName::new(name)))
}

#[test]
fn with_no_component_declared_every_channel_is_the_repositorys() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}{}",
        channel("beta", None, None),
        channel("stable", None, Some("beta"))
    ));
    assert_eq!(
        repo.policy().channels(&at),
        Ok(vec![
            declared("beta", "repository", None),
            declared("stable", "repository", Some("beta")),
        ])
    );
}

#[test]
fn with_one_component_declared_a_channel_naming_none_is_its() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}{}",
        component("lib"),
        channel("beta", None, None)
    ));
    assert_eq!(
        repo.policy().channels(&at),
        Ok(vec![declared("beta", "lib", None)])
    );
}

#[test]
fn a_channel_naming_no_component_among_several_is_unreadable_naming_them() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}{}{}",
        component("lib"),
        component("web"),
        channel("beta", None, None)
    ));
    let said = unreadable(repo.policy().channels(&at), &at);
    assert_eq!(
        said,
        "[channels.beta] names no component, and several are declared: lib, web"
    );
    assert_eq!(unreadable(channel_policy(&repo, &at, "beta"), &at), said);
}

#[test]
fn a_channel_naming_a_component_not_declared_is_unreadable_naming_it() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}{}",
        component("lib"),
        channel("beta", Some("app"), None)
    ));
    let said = unreadable(repo.policy().channels(&at), &at);
    assert_eq!(
        said,
        format!(
            "[channels.beta] names the component app, which {POLICY_FILE} does not declare; it declares lib"
        )
    );
    assert_eq!(unreadable(channel_policy(&repo, &at, "beta"), &at), said);
}

// Requirement: A channel is bound within its component

#[test]
fn a_binding_across_components_is_unreadable_naming_the_channel_bound_to() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}{}{}{}",
        component("lib"),
        component("web"),
        channel("stable", Some("lib"), Some("edge")),
        channel("edge", Some("web"), None)
    ));
    let said = unreadable(repo.policy().channels(&at), &at);
    assert_eq!(
        said,
        "[channels.stable] is bound to edge, a channel of web, and a channel is bound only within its component, lib (125)"
    );
    assert_eq!(unreadable(channel_policy(&repo, &at, "stable"), &at), said);
    assert_eq!(
        channel_policy(&repo, &at, "edge").map(|r| r.bound_to().cloned()),
        Ok(None),
        "a channel bound to none is read whatever another's binding"
    );
}

#[test]
fn a_binding_to_a_channel_not_declared_is_unreadable_naming_it() {
    let repo = Repo::new();
    let at = repo.at(&channel("stable", None, Some("beta")));
    let said = unreadable(repo.policy().channels(&at), &at);
    assert_eq!(
        said,
        "[channels.stable] is bound to beta, which [channels] does not declare"
    );
    assert_eq!(unreadable(channel_policy(&repo, &at, "stable"), &at), said);
}

// Requirement: The profile lists the channels the file declares

#[test]
fn two_components_channels_are_listed_by_name_and_agree_with_policy_for() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}{}{}{}{}",
        component("lib"),
        component("web"),
        channel("stable", Some("lib"), Some("beta")),
        channel("edge", Some("web"), None),
        channel("beta", Some("lib"), None)
    ));
    assert_eq!(
        repo.policy().channels(&at),
        Ok(vec![
            declared("beta", "lib", None),
            declared("edge", "web", None),
            declared("stable", "lib", Some("beta")),
        ])
    );
    assert_eq!(
        channel_policy(&repo, &at, "stable").map(|r| r.bound_to().cloned()),
        Ok(Some(ChannelName::new("beta")))
    );
}

#[test]
fn no_channel_and_no_file_answer_an_empty_list() {
    let repo = Repo::new();
    fs::write(repo.path().join("README"), "no policy").unwrap();
    let none = repo.commit();
    assert_eq!(repo.policy().channels(&none), Ok(vec![]));
    let at = repo.at(&format!(
        "{}[acceptance]\nrequire = [{{ kind = \"provenance\", signers = [\"laptop\"] }}]\n",
        signers()
    ));
    assert_eq!(repo.policy().channels(&at), Ok(vec![]));
}

#[test]
fn a_working_trees_channel_is_not_listed() {
    let repo = Repo::new();
    let at = repo.at(&channel("beta", None, None));
    repo.write(&format!(
        "{}{}",
        channel("beta", None, None),
        channel("edge", None, None)
    ));
    assert_eq!(
        repo.policy().channels(&at),
        Ok(vec![declared("beta", "repository", None)])
    );
}

#[test]
fn the_channels_of_a_revision_not_held_or_a_file_not_read_answer_as_policy_for() {
    let repo = Repo::new();
    let text = "[channels.beta\nrequire = []\n";
    let at = repo.at(text);
    let said = unreadable(repo.policy().channels(&at), &at);
    assert_eq!(
        said,
        toml::from_str::<toml::Table>(text).unwrap_err().to_string()
    );
    let missing = Revision::new("0000000000000000000000000000000000000001");
    assert_eq!(
        repo.policy().channels(&missing),
        Err(Not::absent(&source(), format!("revision {missing}")))
    );
}
