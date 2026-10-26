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

// The channels: helpers

/// A `[[component]]` table with every key the core requires.
fn component(name: &str) -> String {
    format!(
        "[[component]]\nname = \"{name}\"\noutputs = []\npaths = [\"{name}/**\"]\nprefix = \"{name}-v\"\n\n"
    )
}

/// The gate's signer, which may claim acceptance and, where `promotes`,
/// a promotion too.
fn gate_table(promotes: bool) -> String {
    let kinds = if promotes {
        "[\"acceptance\", \"promotion\"]"
    } else {
        "[\"acceptance\"]"
    };
    format!("[signers.gate]\nssh = \"ssh-ed25519 GGGG\"\nkinds = {kinds}\n\n")
}

fn gate(promotes: bool) -> NamedSigner {
    let mut kinds = vec![ClaimKind::Acceptance];
    if promotes {
        kinds.push(ClaimKind::Promotion);
    }
    NamedSigner {
        id: SignerId::new("gate"),
        verified_by: "ssh-ed25519 GGGG".into(),
        kinds,
        checks: vec![],
    }
}

/// The channel `name` of `component`, or of none.
fn id(component: Option<&str>, name: &str) -> ChannelId {
    ChannelId::new(component.map(ComponentName::new), ChannelName::new(name))
}

fn declared(channel: ChannelId, moves: Moves) -> DeclaredChannel {
    DeclaredChannel { channel, moves }
}

fn bound(to: &str) -> Moves {
    Moves::Bound {
        to: ChannelName::new(to),
    }
}

/// `beta` and `stable` of `component`, as every component is given them.
fn givens(component: Option<&str>) -> Vec<DeclaredChannel> {
    vec![
        declared(id(component, "beta"), Moves::EachPublish),
        declared(id(component, "stable"), bound("beta")),
    ]
}

/// `channels` sorted as the answer orders them, by name then component.
fn sorted(mut channels: Vec<DeclaredChannel>) -> Vec<DeclaredChannel> {
    channels.sort_by(|a, b| a.channel.cmp(&b.channel));
    channels
}

fn channel_policy(repo: &Repo, at: &Revision, channel: &ChannelId) -> Answer<Required> {
    repo.policy()
        .policy_for(at, &Audience::Channel(channel.clone()))
}

/// The gate's acceptance, by the signers that may claim it.
fn by_the_gate(signers: Vec<NamedSigner>) -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Acceptance,
        check: None,
        signers,
    }
}

/// The words of the channels' answer and of `channel`'s policy, which
/// are one.
fn both_unreadable(repo: &Repo, at: &Revision, channel: &ChannelId) -> String {
    let said = unreadable(repo.policy().channels(at), at);
    assert_eq!(unreadable(channel_policy(repo, at, channel), at), said);
    said
}

// Requirement: An audience with no table is absent

#[test]
fn a_given_channel_with_no_line_requires_the_gates_acceptance_and_is_promoted_by_it() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}{}[acceptance]\nrequire = [{{ kind = \"provenance\", signers = [\"laptop\"] }}]\n",
        gate_table(false),
        signers()
    ));
    assert_eq!(
        channel_policy(&repo, &at, &id(None, "stable")),
        Ok(Required::Channel {
            claims: vec![by_the_gate(vec![gate(false)])],
            bound_to: Some(ChannelName::new("beta")),
            promoted_by: vec![gate(false)],
        })
    );
}

#[test]
fn a_channel_neither_given_nor_declared_is_absent_naming_it() {
    let repo = Repo::new();
    let at = repo.at(&signers());
    absent(
        channel_policy(&repo, &at, &id(None, "edge")),
        "the channel edge",
    );
}

#[test]
fn with_no_file_a_given_channel_is_answered_as_given() {
    let repo = Repo::new();
    fs::write(repo.path().join("README"), "no policy").unwrap();
    let none = repo.commit();
    assert_eq!(
        channel_policy(&repo, &none, &id(None, "beta")),
        Ok(Required::Channel {
            claims: vec![by_the_gate(vec![])],
            bound_to: None,
            promoted_by: vec![],
        })
    );
    absent(
        repo.policy().policy_for(&none, &Audience::Acceptance),
        "acceptance",
    );
}

// Requirement: A channel's table names its component

#[test]
fn with_no_component_declared_every_channel_is_spelled_by_its_name() {
    let repo = Repo::new();
    let at = repo.at("[channels.edge]\nrequire = []\n\n[channels.next]\nbound_to = \"edge\"\n");
    let mut all = givens(None);
    all.push(declared(id(None, "edge"), Moves::Promotion));
    all.push(declared(id(None, "next"), bound("edge")));
    assert_eq!(repo.policy().channels(&at), Ok(sorted(all)));
    assert_eq!(id(None, "edge").spelled(), "edge");
}

#[test]
fn the_one_declared_component_holds_its_channels_in_the_per_component_form() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}[channels.lib.edge]\nrequire = []\n",
        component("lib")
    ));
    let mut all = givens(Some("lib"));
    all.push(declared(id(Some("lib"), "edge"), Moves::Promotion));
    assert_eq!(repo.policy().channels(&at), Ok(sorted(all)));
    assert_eq!(id(Some("lib"), "edge").spelled(), "lib/edge");
}

#[test]
fn the_flat_form_among_components_is_unreadable_naming_the_form() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}{}[channels.edge]\nrequire = []\n",
        component("lib"),
        component("web")
    ));
    let said = both_unreadable(&repo, &at, &id(Some("lib"), "beta"));
    assert_eq!(
        said,
        format!(
            "[channels.edge] names no component, and {POLICY_FILE} declares lib, web: a channel is declared as [channels.<component>.<name>]"
        )
    );
}

#[test]
fn a_channel_under_a_component_not_declared_is_unreadable_naming_it() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}[channels.app.edge]\nrequire = []\n",
        component("lib")
    ));
    let said = both_unreadable(&repo, &at, &id(Some("lib"), "beta"));
    assert_eq!(
        said,
        format!(
            "[channels.app.edge] names the component app, which {POLICY_FILE} does not declare; it declares lib"
        )
    );
}

#[test]
fn the_component_key_of_before_is_unreadable_naming_the_form_to_use() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}[channels.edge]\ncomponent = \"lib\"\nrequire = []\n",
        component("lib")
    ));
    let said = both_unreadable(&repo, &at, &id(Some("lib"), "beta"));
    assert_eq!(
        said,
        "[channels.edge] names its component with component = \"lib\", and a channel of a component is declared as [channels.lib.edge]"
    );
}

#[test]
fn a_per_component_table_where_no_component_is_declared_is_unreadable_naming_it() {
    let repo = Repo::new();
    let at = repo.at("[channels.lib.edge]\nrequire = []\n");
    let said = both_unreadable(&repo, &at, &id(None, "beta"));
    assert_eq!(
        said,
        format!(
            "[channels.lib.edge] names the component lib, and {POLICY_FILE} declares no component: a channel is declared as [channels.<name>]"
        )
    );
}

// Requirement: A channel is bound within its component

#[test]
fn a_bound_channel_names_the_channel_it_is_bound_to() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}[channels.edge]\nrequire = []\n\n[channels.next]\nbound_to = \"edge\"\nfollows = \"left to its reader\"\n\
         require = [{{ kind = \"provenance\", signers = [\"laptop\"] }}]\n",
        signers()
    ));
    assert_eq!(
        channel_policy(&repo, &at, &id(None, "next")),
        Ok(Required::Channel {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Provenance,
                check: None,
                signers: vec![laptop()],
            }],
            bound_to: Some(ChannelName::new("edge")),
            promoted_by: vec![],
        })
    );
}

#[test]
fn a_binding_across_components_is_unreadable_naming_the_channel_bound_to() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}{}[channels.lib.next]\nbound_to = \"edge\"\n\n[channels.web.edge]\nrequire = []\n",
        component("lib"),
        component("web")
    ));
    let said = both_unreadable(&repo, &at, &id(Some("lib"), "next"));
    assert_eq!(
        said,
        "[channels.lib.next] is bound to edge, which is no channel of lib, and a channel is bound only within its component (125)"
    );
}

#[test]
fn a_binding_to_a_channel_not_declared_is_unreadable_naming_it() {
    let repo = Repo::new();
    let at = repo.at("[channels.next]\nbound_to = \"edge\"\n");
    let said = both_unreadable(&repo, &at, &id(None, "next"));
    assert_eq!(
        said,
        "[channels.next] is bound to edge, which the file neither declares, gives nor makes"
    );
}

#[test]
fn a_binding_to_a_given_channel_is_read() {
    let repo = Repo::new();
    let at = repo.at("[channels.next]\nbound_to = \"beta\"\n");
    let mut all = givens(None);
    all.push(declared(id(None, "next"), bound("beta")));
    assert_eq!(repo.policy().channels(&at), Ok(sorted(all)));
}

// Requirement: The profile lists the channels the file declares

#[test]
fn two_components_channels_are_listed_by_name_then_component() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}{}[channels.lib.next]\nbound_to = \"beta\"\n\n[channels.web.edge]\non_each_publish = true\n",
        component("lib"),
        component("web")
    ));
    assert_eq!(
        repo.policy().channels(&at),
        Ok(vec![
            declared(id(Some("lib"), "beta"), Moves::EachPublish),
            declared(id(Some("web"), "beta"), Moves::EachPublish),
            declared(id(Some("web"), "edge"), Moves::EachPublish),
            declared(id(Some("lib"), "next"), bound("beta")),
            declared(id(Some("lib"), "stable"), bound("beta")),
            declared(id(Some("web"), "stable"), bound("beta")),
        ])
    );
    assert_eq!(
        channel_policy(&repo, &at, &id(Some("lib"), "next")).map(|r| r.bound_to().cloned()),
        Ok(Some(ChannelName::new("beta")))
    );
}

#[test]
fn no_channel_declared_and_no_file_answer_the_given_channels() {
    let repo = Repo::new();
    fs::write(repo.path().join("README"), "no policy").unwrap();
    let none = repo.commit();
    assert_eq!(repo.policy().channels(&none), Ok(givens(None)));
    let at = repo.at(&format!(
        "{}[acceptance]\nrequire = [{{ kind = \"provenance\", signers = [\"laptop\"] }}]\n",
        signers()
    ));
    assert_eq!(repo.policy().channels(&at), Ok(givens(None)));
}

#[test]
fn every_component_is_given_beta_and_stable() {
    let repo = Repo::new();
    let at = repo.at(&format!("{}{}", component("lib"), component("web")));
    assert_eq!(
        repo.policy().channels(&at),
        Ok(sorted([givens(Some("lib")), givens(Some("web"))].concat()))
    );
}

#[test]
fn a_working_trees_channel_is_not_listed() {
    let repo = Repo::new();
    let at = repo.at("[channels.edge]\nrequire = []\n");
    repo.write("[channels.edge]\nrequire = []\n\n[channels.next]\nrequire = []\n");
    let mut all = givens(None);
    all.push(declared(id(None, "edge"), Moves::Promotion));
    assert_eq!(repo.policy().channels(&at), Ok(sorted(all)));
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

// Requirement: How a declared channel moves is read from its table

#[test]
fn a_hand_declared_channel_moves_as_its_table_says() {
    let repo = Repo::new();
    let at = repo.at(
        "[channels.edge]\non_each_publish = true\n\n[channels.lts]\nrequire = []\n\n\
         [channels.next]\nbound_to = \"edge\"\non_each_publish = false\n",
    );
    let mut all = givens(None);
    all.push(declared(id(None, "edge"), Moves::EachPublish));
    all.push(declared(id(None, "lts"), Moves::Promotion));
    all.push(declared(id(None, "next"), bound("edge")));
    assert_eq!(repo.policy().channels(&at), Ok(sorted(all)));
}

#[test]
fn a_following_channel_that_is_bound_is_unreadable_naming_it() {
    let repo = Repo::new();
    let at = repo.at("[channels.edge]\non_each_publish = true\nbound_to = \"beta\"\n");
    let said = both_unreadable(&repo, &at, &id(None, "edge"));
    assert_eq!(
        said,
        "[channels.edge] follows each publish and is bound to beta, and a bound channel takes no version from a publish (37)"
    );
}

// Requirement: A line under a given channel never changes how it moves

#[test]
fn a_line_under_stable_adds_its_claims_after_the_gates_acceptance() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}[signers.verifier]\nssh = \"ssh-ed25519 VVVV\"\nkinds = [\"rebuild\"]\n\n\
         [channels.stable]\nbound_to = \"beta\"\nrequire = [{{ kind = \"rebuild\", signers = [\"verifier\"] }}]\n",
        gate_table(false)
    ));
    let verifier = NamedSigner {
        id: SignerId::new("verifier"),
        verified_by: "ssh-ed25519 VVVV".into(),
        kinds: vec![ClaimKind::Rebuild],
        checks: vec![],
    };
    assert_eq!(
        channel_policy(&repo, &at, &id(None, "stable")),
        Ok(Required::Channel {
            claims: vec![
                by_the_gate(vec![gate(false)]),
                RequiredClaim {
                    kind: ClaimKind::Rebuild,
                    check: None,
                    signers: vec![verifier],
                },
            ],
            bound_to: Some(ChannelName::new("beta")),
            promoted_by: vec![gate(false)],
        })
    );
    assert_eq!(repo.policy().channels(&at), Ok(givens(None)));
}

#[test]
fn a_line_that_would_unbind_stable_is_unreadable_naming_it() {
    let repo = Repo::new();
    let at = repo.at("[channels.stable]\non_each_publish = true\n");
    let said = both_unreadable(&repo, &at, &id(None, "stable"));
    assert_eq!(
        said,
        "[channels.stable] would change how stable moves, and stable is bound to beta: a line under a channel given or made never changes how it moves (C.10)"
    );
    let at = repo.at("[channels.beta]\nbound_to = \"stable\"\n");
    let said = both_unreadable(&repo, &at, &id(None, "beta"));
    assert!(
        said.starts_with("[channels.beta] would change how beta moves"),
        "{said}"
    );
}

// Requirement: The given channels' policy names the gate

#[test]
fn the_gates_signers_promote_beta_whatever_a_line_names() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}{}[channels.stable]\nrequire = [{{ kind = \"acceptance\", signers = [\"gate\", \"laptop\"] }}]\n",
        gate_table(true),
        "[signers.laptop]\nssh = \"ssh-ed25519 LLLL\"\nkinds = [\"promotion\"]\n\n"
    ));
    let beta = channel_policy(&repo, &at, &id(None, "beta")).unwrap();
    assert_eq!(beta.claims(), &[by_the_gate(vec![gate(true)])]);
    assert_eq!(beta.promoted_by(), &[gate(true)]);
    let stable = channel_policy(&repo, &at, &id(None, "stable")).unwrap();
    assert_eq!(
        stable.promoted_by(),
        &[gate(true)],
        "a line widened who signs"
    );
}

#[test]
fn a_hand_declared_channels_promoters_are_the_signers_it_names_that_may_promote() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}[signers.release]\nssh = \"ssh-ed25519 RRRR\"\nkinds = [\"promotion\"]\n\n\
         [channels.lts]\nrequire = [{{ kind = \"acceptance\", signers = [\"gate\", \"release\"] }}]\n",
        gate_table(false)
    ));
    let lts = channel_policy(&repo, &at, &id(None, "lts")).unwrap();
    assert_eq!(
        lts.promoted_by()
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        ["release"]
    );
}

// Requirement: The environments' words are read

#[test]
fn an_environment_that_moves_on_its_own_makes_no_channel_and_its_targets_keys_are_not_read() {
    let repo = Repo::new();
    let at = repo
        .at("[environments.site]\nchannel = \"stable\"\nbucket = \"example-site\"\nregion = 3\n");
    assert_eq!(repo.policy().channels(&at), Ok(givens(None)));
}

#[test]
fn an_unknown_policy_word_is_unreadable_naming_it() {
    let repo = Repo::new();
    let at = repo.at("[environments.prod]\nchannel = \"stable\"\npolicy = \"sometimes\"\n");
    let said = both_unreadable(&repo, &at, &id(None, "beta"));
    assert_eq!(
        said,
        "[environments.prod] names the policy sometimes, and an environment's policy is auto or manual"
    );
}

#[test]
fn an_environment_following_a_channel_not_known_is_unreadable_naming_it() {
    let repo = Repo::new();
    let at = repo.at("[environments.prod]\nchannel = \"lts\"\n");
    let said = unreadable(repo.policy().channels(&at), &at);
    assert_eq!(
        said,
        "[environments.prod] follows lts, which the file neither declares, gives nor makes"
    );
}

#[test]
fn environments_among_components_are_declared_per_component() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}{}[environments.web.prod]\nchannel = \"stable\"\npolicy = \"manual\"\n",
        component("lib"),
        component("web")
    ));
    let mut all = [givens(Some("lib")), givens(Some("web"))].concat();
    all.push(declared(id(Some("web"), "prod"), bound("stable")));
    assert_eq!(repo.policy().channels(&at), Ok(sorted(all)));

    let at = repo.at(&format!(
        "{}{}[environments.prod]\nchannel = \"stable\"\n",
        component("lib"),
        component("web")
    ));
    let said = unreadable(repo.policy().channels(&at), &at);
    assert_eq!(
        said,
        format!(
            "[environments.prod] names no component, and {POLICY_FILE} declares lib, web: an environment is declared as [environments.<component>.<name>]"
        )
    );
}

// Requirement: A manual environment has a channel of its own

#[test]
fn prod_waits_on_stable_with_a_channel_of_its_own() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}[environments.prod]\nchannel = \"stable\"\npolicy = \"manual\"\n",
        gate_table(false)
    ));
    let mut all = givens(None);
    all.push(declared(id(None, "prod"), bound("stable")));
    assert_eq!(repo.policy().channels(&at), Ok(sorted(all)));
    assert_eq!(
        channel_policy(&repo, &at, &id(None, "prod")),
        Ok(Required::Channel {
            claims: vec![by_the_gate(vec![gate(false)])],
            bound_to: Some(ChannelName::new("stable")),
            promoted_by: vec![gate(false)],
        })
    );
}

// Requirement: An after environment has a channel bound to theirs

#[test]
fn eu_after_us_is_bound_to_us() {
    let repo = Repo::new();
    let at = repo.at(
        "[environments.us]\nchannel = \"stable\"\npolicy = \"manual\"\n\n\
         [environments.eu]\nchannel = \"stable\"\nafter = [\"us\"]\n",
    );
    let mut all = givens(None);
    all.push(declared(id(None, "us"), bound("stable")));
    all.push(declared(id(None, "eu"), bound("us")));
    assert_eq!(repo.policy().channels(&at), Ok(sorted(all)));
}

#[test]
fn after_two_environments_that_follow_two_channels_is_unreadable_naming_them() {
    let repo = Repo::new();
    let at = repo.at(
        "[environments.us]\nchannel = \"stable\"\n\n[environments.asia]\nchannel = \"beta\"\n\n\
         [environments.eu]\nchannel = \"stable\"\nafter = [\"us\", \"asia\"]\n",
    );
    let said = unreadable(repo.policy().channels(&at), &at);
    assert_eq!(
        said,
        "[environments.eu] is after us and asia, which follow stable and beta, and the environments one comes after follow one channel"
    );
}

#[test]
fn a_cycle_through_after_is_unreadable_naming_the_table() {
    let repo = Repo::new();
    let at = repo.at(
        "[environments.eu]\nchannel = \"stable\"\nafter = [\"us\"]\n\n\
         [environments.us]\nchannel = \"stable\"\nafter = [\"eu\"]\n",
    );
    let said = unreadable(repo.policy().channels(&at), &at);
    assert_eq!(said, "[environments.eu] comes after itself through after");
}

#[test]
fn after_an_environment_not_declared_is_unreadable_naming_it() {
    let repo = Repo::new();
    let at = repo.at("[environments.eu]\nchannel = \"stable\"\nafter = [\"mars\"]\n");
    let said = unreadable(repo.policy().channels(&at), &at);
    assert_eq!(
        said,
        "[environments.eu] is after mars, which [environments] does not declare"
    );
}

// Requirement: An environment's channel is named for it

#[test]
fn a_manual_environment_named_beta_is_unreadable_naming_it() {
    let repo = Repo::new();
    let at = repo.at("[environments.beta]\nchannel = \"stable\"\npolicy = \"manual\"\n");
    let said = unreadable(repo.policy().channels(&at), &at);
    assert_eq!(
        said,
        "[environments.beta] makes a channel named beta, which every component is given (C.10)"
    );
}

#[test]
fn a_line_that_restates_prods_binding_adds_its_claims() {
    let repo = Repo::new();
    let at = repo.at(&format!(
        "{}{}[environments.prod]\nchannel = \"stable\"\npolicy = \"manual\"\n\n\
         [channels.prod]\nbound_to = \"stable\"\nrequire = [{{ kind = \"provenance\", signers = [\"laptop\"] }}]\n",
        gate_table(false),
        signers()
    ));
    let mut all = givens(None);
    all.push(declared(id(None, "prod"), bound("stable")));
    assert_eq!(repo.policy().channels(&at), Ok(sorted(all)));
    assert_eq!(
        channel_policy(&repo, &at, &id(None, "prod")).map(|r| r.claims().to_vec()),
        Ok(vec![
            by_the_gate(vec![gate(false)]),
            RequiredClaim {
                kind: ClaimKind::Provenance,
                check: None,
                signers: vec![laptop()],
            },
        ])
    );
}

#[test]
fn a_line_under_prod_that_rebinds_it_is_unreadable_naming_it() {
    let repo = Repo::new();
    let at = repo.at(
        "[environments.prod]\nchannel = \"stable\"\npolicy = \"manual\"\n\n\
         [channels.prod]\nbound_to = \"beta\"\n",
    );
    let said = both_unreadable(&repo, &at, &id(None, "prod"));
    assert_eq!(
        said,
        "[channels.prod] would change how prod moves, and prod is bound to stable: a line under a channel given or made never changes how it moves (C.10)"
    );
}
