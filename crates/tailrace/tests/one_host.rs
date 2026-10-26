//! Spec command-line: the one host end to end, the built binary bound by
//! a bindings file naming `[gate] publish = {}` over a scratch repository
//! on a plain branch whose builds moon runs. Two landings, each followed
//! by one `tailrace publish`, move `beta` twice in the ledger and the
//! layout, each acceptance saying one host offered and accepted; the same
//! host with no `[gate]` does exactly the same, its file binding the
//! host's publish as what tells its gate (95, 108, 118, 130, C.6; design
//! D7). Where the policy names two publishers and no gate, the publish
//! offers the version and the gate accepts nothing, refused in the
//! policy's words, and no channel moves (C.10; design D7a). The binary
//! runs with
//! its home and its configuration in the scratch directory, and none of
//! the `SLUICE_*` or `MOON_*` variables of a gate that may be running
//! these tests. Needs moon, so it is in the tools layer.

#![cfg(feature = "tools")]

mod scratch;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use scratch::{git_command, moon_workspace, policy_file, Repo, Scratch, Task};
use serde_json::Value;
use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey};
use tailrace::bindings::{self, For};
use tailrace::profiles::store::layout::Layout;
use tailrace_core::{channel, Channel};
use tailrace_seams::{
    channel_name, ChannelId, ChannelName, Claim, ClaimKind, Digest, Promotion, Store,
};

/// The identity the one host signs as, and the gate's.
const LAPTOP: &str = "laptop";

fn beta() -> ChannelId {
    ChannelId::new(None, ChannelName::new("beta"))
}

/// The one host: a scratch repository on a plain branch with moon's
/// task, a policy naming the host's key for provenance, acceptance and
/// promotion and `beta` following each publish, and a home and a
/// configuration directory of its own binding a layout, the branch, the
/// git ref ledger and the key, with `gate` as its `[gate]`.
struct OneHost {
    scratch: Scratch,
    home: PathBuf,
    config: PathBuf,
}

impl OneHost {
    fn new(gate: &str) -> OneHost {
        OneHost::under(gate, |laptop| {
            format!(
                "[signers.{LAPTOP}]\nssh = \"{laptop}\"\nkinds = [\"provenance\", \"acceptance\", \"promotion\"]\n\n\
                 [acceptance]\nrequire = [{{ kind = \"provenance\", signers = [\"{LAPTOP}\"] }}]\n\n\
                 [channels.beta]\non_each_publish = true\n\
                 require = [{{ kind = \"acceptance\", signers = [\"{LAPTOP}\"] }}]\n"
            )
        })
    }

    /// The one host as [`OneHost::new`] makes it, its policy's signers,
    /// audiences and channels the tables `policy` writes given the host
    /// key's public line.
    fn under(gate: &str, policy: impl Fn(&str) -> String) -> OneHost {
        let scratch = Scratch::new();
        let repo = &scratch.repo;
        // The shared history the version's name is claimed in (22).
        repo.git(&["config", "branch.main.remote", "origin"]);
        repo.git(&["config", "branch.main.merge", "refs/heads/main"]);

        let home = scratch.path().join("home");
        let config = scratch.path().join("config");
        let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).expect("an Ed25519 key");
        let file = home.join(".config/tailrace/signing");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        key.write_openssh_file(&file, LineEnding::LF).unwrap();
        fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();

        moon_workspace(repo, &[Task::new("web", "build", "site.txt", true, 0)]);
        policy_file(
            repo,
            &[("site", "web:build")],
            &format!(
                "\n[projections]\nsite = \"file\"\n\n{}",
                policy(&key.public_key().to_openssh().unwrap())
            ),
        );

        let bindings = config.join("tailrace/config.toml");
        fs::create_dir_all(bindings.parent().unwrap()).unwrap();
        fs::write(
            &bindings,
            format!(
                "[store]\nlayout = \"~/.local/share/tailrace/layout\"\n\n\
                 [landing]\nbranch = {{ remote = \"origin\" }}\n\n\
                 [ledger]\ngit_ref = {{ remote = \"origin\" }}\n\n\
                 [signing]\nsigner = \"{LAPTOP}\"\nkey = \"~/.config/tailrace/signing\"\n{gate}"
            ),
        )
        .unwrap();
        OneHost {
            scratch,
            home,
            config,
        }
    }

    fn repo(&self) -> &Repo {
        &self.scratch.repo
    }

    fn layout(&self) -> Layout {
        Layout::open(self.home.join(".local/share/tailrace/layout"))
            .expect("the publish made the layout")
    }

    /// Everything written committed with `message` and pushed: landed on
    /// the plain branch (C.6); the commit.
    fn land(&self, message: &str) -> String {
        let revision = self.repo().commit(message);
        self.repo().push();
        revision
    }

    /// `tailrace publish <revision> --json` by the built binary in the
    /// repository: its exit and the one JSON document it printed.
    fn publish(&self, revision: &str) -> (Option<i32>, Value) {
        let mut command = Command::new(env!("CARGO_BIN_EXE_tailrace"));
        for (name, _) in std::env::vars_os() {
            let name = name.to_string_lossy().to_string();
            if name.starts_with("SLUICE_") || name.starts_with("MOON_") {
                command.env_remove(&name);
            }
        }
        // git as the scratch repositories are made: none of the host's
        // configuration, a fixed author.
        for (name, value) in git_command().get_envs() {
            match value {
                Some(value) => command.env(name, value),
                None => command.env_remove(name),
            };
        }
        let out = command
            .args(["publish", revision, "--json"])
            .current_dir(&self.repo().path)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.config)
            .output()
            .expect("the binary runs");
        let json = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
            panic!(
                "tailrace publish printed no JSON document ({e}): {}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            )
        });
        (out.status.code(), json)
    }

    /// Every acceptance claim on `index` the layout holds.
    fn acceptances(&self, index: &Digest) -> Vec<Claim> {
        self.layout()
            .claims_of(index)
            .unwrap()
            .iter()
            .filter_map(Claim::from_envelope)
            .filter(|c| c.kind == ClaimKind::Acceptance)
            .collect()
    }

    /// The ledger refs the repository and its remote hold.
    fn ledger_refs(&self) -> (String, String) {
        let remote = Repo {
            path: self.scratch.remote.clone(),
        };
        (
            self.repo().git(&["for-each-ref", "refs/tailrace/ledger"]),
            remote.git(&["for-each-ref", "refs/tailrace/ledger"]),
        )
    }
}

/// The one component's offered version in publish's answer: the version,
/// its index and its `gate` member.
fn offered(json: &Value) -> (String, Digest, Value) {
    assert_eq!(json["answer"], "ok", "{json}");
    let components = json["value"]["components"].as_array().unwrap();
    assert_eq!(components.len(), 1, "{json}");
    let component = &components[0];
    assert_eq!(component["answer"], "ok", "{json}");
    assert_eq!(component["value"]["published"], "version", "{json}");
    (
        component["value"]["version"].as_str().unwrap().to_string(),
        Digest::parse(component["value"]["index"].as_str().unwrap()).unwrap(),
        component["gate"].clone(),
    )
}

/// Two landings on the one host whose file holds `gate`, each followed
/// by one `tailrace publish`: each version accepted by the one host and
/// `beta` moved to it in the same run, its chain holding both
/// promotions, the second naming the first, and the layout's name for
/// `beta` at the second.
fn beta_follows_two_landings(gate: &str) {
    let host = OneHost::new(gate);
    let mut indexes = Vec::new();
    for (message, grows, version) in [
        ("feat: the site", "", "0.1.0"),
        ("feat: the site grows", "# it grows\n", "0.2.0"),
    ] {
        if !grows.is_empty() {
            let moon = host.repo().path.join("web/moon.yml");
            let mut text = fs::read_to_string(&moon).unwrap();
            text.push_str(grows);
            fs::write(&moon, text).unwrap();
        }
        let landed = host.land(message);
        let (code, json) = host.publish(&landed);
        assert_eq!(code, Some(0), "{json}");
        let (offered_as, index, gate) = offered(&json);
        assert_eq!(offered_as, version, "{json}");
        assert_eq!(gate["told_by"], "publish", "{json}");
        assert_eq!(gate["answer"], "ok", "{json}");
        assert_eq!(gate["value"]["judged"], "accepted", "{json}");
        assert_eq!(gate["value"]["one_host"], true, "{json}");
        assert_eq!(gate["value"]["index"], index.as_str(), "{json}");
        let channels = gate["value"]["channels"].as_array().unwrap();
        assert_eq!(channels.len(), 1, "{json}");
        assert_eq!(
            channels[0]["channel"],
            serde_json::json!({ "component": null, "name": "beta" }),
            "{json}"
        );
        assert_eq!(channels[0]["value"]["moved"], "promoted", "{json}");
        assert_eq!(channels[0]["value"]["to"], index.as_str(), "{json}");

        let acceptances = host.acceptances(&index);
        assert_eq!(acceptances.len(), 1, "{index}");
        assert_eq!(acceptances[0].body["offered"]["one_host"], true);
        assert_eq!(acceptances[0].revision.as_str(), landed);
        indexes.push(index);
    }

    // beta's chain holds both promotions, the second naming the first,
    // read from the git ref ledger the file binds.
    let binding = bindings::bind(
        &host.config.join("tailrace/config.toml"),
        Some(&host.home),
        None,
        &host.repo().path,
        For::Verify,
    )
    .unwrap_or_else(|_| panic!("the host binds"));
    let Ok(Channel::At { head, chain, .. }) = channel(binding.seams.keeper.as_ref(), &beta())
    else {
        panic!("beta's chain does not read")
    };
    let [(first, made), (_, then)] = chain.as_slice() else {
        panic!("beta's chain holds {} promotions", chain.len())
    };
    assert_eq!(made.to.index, indexes[0]);
    assert_eq!(made.predecessor, None);
    assert_eq!(then.to.index, indexes[1]);
    assert_eq!(then.predecessor.as_ref(), Some(first));
    assert_eq!(then.from.as_ref().map(|f| &f.index), Some(&indexes[0]));
    assert_eq!(head, *then);
    let (ours, theirs) = host.ledger_refs();
    assert!(ours.contains("refs/tailrace/ledger/"), "{ours}");
    assert!(theirs.contains("refs/tailrace/ledger/"), "{theirs}");

    // The layout's name for beta is at the second index, and the
    // promotion on it is the chain's head.
    let layout = host.layout();
    assert_eq!(
        layout.resolve(&channel_name(&beta())),
        Ok(indexes[1].clone())
    );
    let promoted: Vec<Promotion> = layout
        .claims_of(&indexes[1])
        .unwrap()
        .iter()
        .filter_map(Claim::from_envelope)
        .filter_map(|c| Promotion::of(&c))
        .collect();
    assert_eq!(promoted, std::slice::from_ref(then));
}

#[test]
fn two_landings_on_the_one_host_each_move_beta_in_the_same_run() {
    beta_follows_two_landings("\n[gate]\npublish = {}\n");
}

#[test]
fn the_one_host_with_no_gate_table_moves_beta_on_each_landing_as_publish_does() {
    beta_follows_two_landings("");
}

// Spec policy-file, several publishers and no gate is refused in words:
// the gate accepts nothing.

#[test]
fn where_the_policy_names_two_publishers_and_no_gate_the_gate_accepts_nothing() {
    let desk = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).expect("an Ed25519 key");
    let desk = desk.public_key().to_openssh().unwrap();
    let host = OneHost::under("", |laptop| {
        format!(
            "[signers.{LAPTOP}]\nssh = \"{laptop}\"\nkinds = [\"provenance\"]\n\n\
             [signers.desk]\nssh = \"{desk}\"\nkinds = [\"provenance\"]\n\n\
             [acceptance]\nrequire = [{{ kind = \"provenance\", signers = [\"{LAPTOP}\"] }}]\n"
        )
    });
    let landed = host.land("feat: the site");
    let (code, json) = host.publish(&landed);
    let (version, index, gate) = offered(&json);
    assert_eq!(version, "0.1.0", "{json}");
    assert_eq!(gate["told_by"], "publish", "{json}");
    assert_eq!(gate["answer"], "refused", "{json}");
    assert_eq!(
        gate["refused"]["said"], "several publishers and no gate, name an accepting signer",
        "{json}"
    );
    assert_eq!(
        gate["refused"]["why"]["wanting"],
        serde_json::json!({ "kind": "acceptance", "check": null, "signers": [] }),
        "{json}"
    );
    assert_eq!(code, Some(1), "{json}");
    assert_eq!(host.acceptances(&index), [], "an acceptance was made");
    assert_eq!(
        host.ledger_refs(),
        (String::new(), String::new()),
        "the ledger was written"
    );
}
