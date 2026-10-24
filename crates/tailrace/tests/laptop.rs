//! Spec host-bindings: the laptop end to end, over a scratch repository
//! that sluice gates and whose builds moon runs, its host bound by a
//! bindings file naming a fresh layout, sluice and the laptop's key
//! (118; design D8). Needs moon, sluice and ssh-keygen, so it is in the
//! tools layer.

#![cfg(feature = "tools")]

mod fixtures;
mod scratch;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use scratch::gate::{Gate, SIGNER};
use scratch::recorder::Recorder;
use scratch::{moon_workspace, policy_file, ssh_keygen, Scratch, Task};

/// The identity the laptop signs as.
const LAPTOP: &str = "laptop";

/// The two outputs, the task that makes each, and the projection that
/// packs it.
const OUTPUTS: [(&str, &str, &str); 2] = [
    ("notes", "notes:build", "file"),
    ("image", "image:build", "image"),
];

/// A host as the binary sees it: a home and a configuration directory
/// of its own, in the scratch directory.
struct Host {
    home: PathBuf,
    config: PathBuf,
}

impl Host {
    fn new(scratch: &Scratch, name: &str) -> Host {
        let host = Host {
            home: scratch.path().join(format!("{name}-home")),
            config: scratch.path().join(format!("{name}-config")),
        };
        fs::create_dir_all(&host.home).unwrap();
        host
    }

    /// `text` as the host's bindings file.
    fn bind(&self, text: &str) {
        let file = self.config.join("tailrace/config.toml");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, text).unwrap();
    }
}

/// An SSH key made by ssh-keygen at `path`, mode 0600; answers its
/// public half as a policy lists it.
fn keygen(path: &Path, comment: &str) -> String {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let out = Command::new(ssh_keygen())
        .args(["-q", "-t", "ed25519", "-N", "", "-C", comment, "-f"])
        .arg(path)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::read_to_string(path.with_extension("pub"))
        .unwrap()
        .trim()
        .to_string()
}

/// The scratch world: a repository sluice gates, the laptop's host, its
/// key, and the revision sluice landed.
struct Laptop {
    scratch: Scratch,
    host: Host,
    layout: PathBuf,
    landed: String,
}

impl Laptop {
    /// The repository's moon tasks and policy file, committed and pushed;
    /// sluice gating `main` on both tasks lands a submission; and the
    /// laptop's bindings file names a fresh layout, sluice with the
    /// remote, and the laptop's key.
    fn new() -> Laptop {
        let scratch = Scratch::new();
        let repo = &scratch.repo;
        // The shared history the version's name is claimed in (22).
        repo.git(&["config", "branch.main.remote", "origin"]);
        repo.git(&["config", "branch.main.merge", "refs/heads/main"]);

        let host = Host::new(&scratch, LAPTOP);
        let key = host.home.join(".config/tailrace/signing");
        let laptop_key = keygen(&key, "host@laptop");
        let recorder = Recorder::new(repo, true, &scratch.path().join("keys"));

        moon_workspace(
            repo,
            &[
                Task::new("notes", "build", "notes.txt", true, 0),
                Task::new("image", "build", "image.tar", true, 0),
            ],
        );
        // The image task copies the archive the repository holds.
        repo.write(
            "image/moon.yml",
            "tasks:\n  build:\n    command: 'sh -c \"mkdir -p dist && cp image.tar dist/image.tar\"'\n    \
             inputs: ['moon.yml', 'image.tar']\n    outputs: ['dist/image.tar']\n    options: {shell: false}\n",
        );
        fs::write(
            repo.path.join("image/image.tar"),
            fixtures::image::archive(),
        )
        .unwrap();
        let projections: String = OUTPUTS
            .iter()
            .map(|(output, _, ecosystem)| format!("{output} = \"{ecosystem}\"\n"))
            .collect();
        let checks: Vec<String> = OUTPUTS.iter().map(|(_, t, _)| format!("\"{t}\"")).collect();
        policy_file(
            repo,
            &OUTPUTS.map(|(output, task, _)| (output, task)),
            &format!(
                "\n[projections]\n{projections}\n\
                 [signers.{LAPTOP}]\nssh = \"{laptop_key}\"\nkinds = [\"provenance\"]\n\n\
                 [signers.{SIGNER}]\nssh = \"{}\"\nkinds = [\"verdict\"]\nchecks = [{}]\n\n\
                 [acceptance]\nrequire = [{{ kind = \"provenance\", signers = [\"{LAPTOP}\"] }}]\n",
                recorder.public_key(SIGNER),
                checks.join(", ")
            ),
        );
        repo.commit("feat: the notes and the image");
        repo.push();

        let gate = Gate::new(&scratch, &recorder, &OUTPUTS.map(|(_, task, _)| task));
        let landed = gate.land("x", &[("notes/x.txt", "x")]);

        let layout = scratch.path().join("layouts/laptop");
        host.bind(&format!(
            "[store]\nlayout = {:?}\n\n[landing]\nsluice = {{ remote = \"origin\" }}\n\n\
             [signing]\nsigner = \"{LAPTOP}\"\nkey = \"~/.config/tailrace/signing\"\n",
            layout.display().to_string()
        ));
        Laptop {
            scratch,
            host,
            layout,
            landed,
        }
    }
}

/// sluice landed the revision on `main`, and the host's bindings file
/// names a layout no publish has made yet.
#[test]
fn the_laptop_is_bound_over_a_sluice_landing() {
    let laptop = Laptop::new();
    let repo = &laptop.scratch.repo;
    assert_eq!(repo.git(&["rev-parse", "origin/main"]), laptop.landed);
    assert!(laptop.host.config.join("tailrace/config.toml").is_file());
    assert!(
        !laptop.layout.exists(),
        "the layout was made before a publish"
    );
}
