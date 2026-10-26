//! Spec policy-file: the conformance suite's policy group over the policy
//! file, each policy the group asks for written as a commit of a scratch
//! repository of its own (design D3, D7). Needs git alone, so it is in
//! the fast layer.

mod scratch;

use std::collections::BTreeMap;
use std::fs;

use scratch::{git_fails, Scratch, POLICY_FILE};
use tailrace::profiles::policy::file::PolicyFile;
use tailrace_conformance::seams::policy::{self, PolicyRig};
use tailrace_conformance::{run, Rig};
use tailrace_seams::{
    Audience, ChannelId, ChannelName, ComponentName, DeclaredChannel, Moves, NamedSigner, Policy,
    Required, Revision,
};

/// The given channels as tailrace ships them, beside the profile (C.10).
const GIVENS: &str = include_str!("../src/profiles/policy/file/givens.toml");

/// A scratch repository and the policy file over it.
struct PolicyFileRig {
    scratch: Scratch,
    policy: PolicyFile,
}

impl PolicyFileRig {
    fn new() -> PolicyFileRig {
        let scratch = Scratch::new();
        let policy = PolicyFile::new(&scratch.repo.path);
        PolicyFileRig { scratch, policy }
    }
}

/// `text` as a TOML string.
fn quoted(text: &str) -> String {
    toml::Value::from(text).to_string()
}

fn list<'a>(items: impl IntoIterator<Item = &'a str>) -> String {
    let items: Vec<String> = items.into_iter().map(quoted).collect();
    format!("[{}]", items.join(", "))
}

/// The policy file declaring what each of `policies` requires: every
/// signer they name under `[signers]`, verified by the text each is
/// named with, each audience's table, and a channel one of them is bound
/// to and `policies` names no policy for, requiring nothing, since the
/// file binds a channel only to one it declares (37).
fn policy_file(policies: &[(Audience, Required)]) -> String {
    let mut signers: BTreeMap<String, &NamedSigner> = BTreeMap::new();
    for named in policies
        .iter()
        .flat_map(|(_, required)| required.claims())
        .flat_map(|claim| &claim.signers)
    {
        let held = signers.entry(named.id.to_string()).or_insert(named);
        assert_eq!(
            *held, named,
            "one signer is named with two reaches, which one [signers] table cannot hold"
        );
    }
    let mut text = String::new();
    for (id, named) in &signers {
        text.push_str(&format!(
            "[signers.{id}]\nkey = {}\nkinds = {}\nchecks = {}\n\n",
            quoted(&named.verified_by),
            list(named.kinds.iter().map(|k| k.name())),
            list(named.checks.iter().map(|c| c.as_str())),
        ));
    }
    for (audience, required) in policies {
        let table = match audience {
            Audience::Acceptance => "acceptance".to_string(),
            Audience::Layout => "layout".to_string(),
            Audience::Channel(channel) => match &channel.component {
                Some(component) => format!("channels.{component}.{}", channel.name),
                None => format!("channels.{}", channel.name),
            },
        };
        text.push_str(&format!("[{table}]\n"));
        if let Some(bound_to) = required.bound_to() {
            text.push_str(&format!("bound_to = {}\n", quoted(bound_to.as_str())));
        }
        text.push_str("require = [\n");
        for claim in required.claims() {
            let check = claim
                .check
                .as_ref()
                .map_or_else(String::new, |c| format!(", check = {}", quoted(c.as_str())));
            text.push_str(&format!(
                "  {{ kind = {}{check}, signers = {} }},\n",
                quoted(claim.kind.name()),
                list(claim.signers.iter().map(|s| s.id.as_str())),
            ));
        }
        text.push_str("]\n\n");
    }
    let declared: Vec<&Audience> = policies.iter().map(|(audience, _)| audience).collect();
    let mut bound: Vec<&ChannelName> = policies
        .iter()
        .filter_map(|(_, required)| required.bound_to())
        .filter(|channel| {
            !declared.contains(&&Audience::Channel(ChannelId::new(
                None,
                (*channel).clone(),
            )))
        })
        .collect();
    bound.sort_unstable();
    bound.dedup();
    for channel in bound {
        text.push_str(&format!("[channels.{channel}]\nrequire = []\n\n"));
    }
    text
}

/// The policy file declaring `components`, each with every key the core
/// requires, and each of `channels` requiring nothing, moving as it says,
/// in the per-component form where components are declared (C.10).
fn declaring_file(components: &[ComponentName], channels: &[DeclaredChannel]) -> String {
    let mut text = String::new();
    for name in components {
        text.push_str(&format!(
            "[[component]]\nname = {}\noutputs = []\npaths = {}\nprefix = {}\n\n",
            quoted(name.as_str()),
            list([format!("{name}/**").as_str()]),
            quoted(&format!("{name}-v")),
        ));
    }
    for channel in channels {
        let table = match &channel.channel.component {
            Some(component) => format!("{component}.{}", channel.channel.name),
            None => channel.channel.name.to_string(),
        };
        text.push_str(&format!("[channels.{table}]\nrequire = []\n"));
        match &channel.moves {
            Moves::EachPublish => text.push_str("on_each_publish = true\n"),
            Moves::Bound { to } => {
                text.push_str(&format!("bound_to = {}\n", quoted(to.as_str())));
            }
            Moves::Promotion => {}
        }
        text.push('\n');
    }
    text
}

impl Rig for PolicyFileRig {
    fn profile(&self) -> String {
        "policy file".into()
    }
}

impl PolicyRig for PolicyFileRig {
    fn policy(&self) -> &dyn Policy {
        &self.policy
    }

    /// The policy file declaring `policies`, committed.
    fn at(&self, policies: &[(Audience, Required)]) -> Revision {
        let repo = &self.scratch.repo;
        repo.write(POLICY_FILE, &policy_file(policies));
        Revision::new(repo.commit("chore: the policy"))
    }

    /// The policy file declaring `components` and `channels`, committed.
    fn declaring(&self, components: &[ComponentName], channels: &[DeclaredChannel]) -> Revision {
        let repo = &self.scratch.repo;
        repo.write(POLICY_FILE, &declaring_file(components, channels));
        Revision::new(repo.commit("chore: the channels"))
    }

    /// C.10: each channel the shipped fragment has on, of every
    /// component, or of the one where none is declared, moving as its
    /// table says, in the order the channels answer them.
    fn given(&self, components: &[ComponentName]) -> Vec<DeclaredChannel> {
        let fragment: toml::Table = toml::from_str(GIVENS).expect("the shipped fragment parses");
        let tables = fragment["channels"]
            .as_table()
            .expect("the fragment's channels");
        let scopes: Vec<Option<ComponentName>> = if components.is_empty() {
            vec![None]
        } else {
            components.iter().cloned().map(Some).collect()
        };
        let mut given = Vec::new();
        for (name, table) in tables {
            let key = |key: &str| table.get(key);
            if key("on").and_then(toml::Value::as_bool) == Some(false) {
                continue;
            }
            let moves = match (
                key("on_each_publish").and_then(toml::Value::as_bool),
                key("bound_to").and_then(toml::Value::as_str),
            ) {
                (_, Some(to)) => Moves::Bound {
                    to: ChannelName::new(to),
                },
                (Some(true), None) => Moves::EachPublish,
                _ => Moves::Promotion,
            };
            for component in &scopes {
                given.push(DeclaredChannel {
                    channel: ChannelId::new(component.clone(), ChannelName::new(name)),
                    moves: moves.clone(),
                });
            }
        }
        given.sort_by(|a, b| a.channel.cmp(&b.channel));
        given
    }

    /// The repository removed, and git's words for reading it.
    fn unreachable(&self) -> String {
        let repo = &self.scratch.repo.path;
        fs::remove_dir_all(repo).unwrap();
        git_fails(repo, &["rev-parse", "--git-common-dir"])
    }
}

#[test]
fn the_policy_group_over_the_policy_file() {
    let report = run(policy::GROUP, &policy::scenarios(), &|| {
        Box::new(PolicyFileRig::new())
    });
    assert_eq!(report.profile, "policy file");
    assert_eq!(report.not_applicable, [], "{report}");
    report.assert();
}
