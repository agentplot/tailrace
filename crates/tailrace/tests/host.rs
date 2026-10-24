//! Spec host-bindings: the conformance suite's host group over the
//! bindings file, a configuration directory in a temporary directory
//! holding sluice's own `sluice/config.toml` beside it, sluice's records
//! of a scratch repository's landing kept in the host's own files, and a
//! host key made fresh (118; design D7). Needs git alone, so it is in the
//! fast layer.

mod scratch;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use scratch::recorder::Recorder;
use scratch::{Scratch, MAIN};
use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey};
use tailrace::bindings::{self, For};
use tailrace::profiles::store::layout::Layout;
use tailrace_conformance::seams::host::{self, HostRig};
use tailrace_conformance::{run, Rig};
use tailrace_seams::{
    Answer, Branch, ClaimKind, Digest, Held, NamedSigner, Revision, Seams, SignerId, Store,
};

/// A host: its configuration directory and home, a repository sluice
/// landed a revision of, and the key its bindings name.
struct BindingsRig {
    scratch: Scratch,
    key: PrivateKey,
    landed: Revision,
}

impl BindingsRig {
    fn new() -> BindingsRig {
        let scratch = Scratch::new();
        let recorder = Recorder::new(&scratch.repo, false, &scratch.path().join("records"));
        let landed = recorder.land(1, true).commits.remove(0);
        let rig = BindingsRig {
            scratch,
            key: PrivateKey::random(&mut OsRng, Algorithm::Ed25519).expect("an Ed25519 key"),
            landed: Revision::new(landed),
        };
        let landings = rig.landings_file();
        fs::create_dir_all(landings.parent().unwrap()).unwrap();
        fs::write(
            &landings,
            "[signing]\nsigner = \"box\"\nkey = \"~/.ssh/box\"\n",
        )
        .unwrap();
        rig
    }

    fn home(&self) -> PathBuf {
        self.scratch.path().join("home")
    }

    fn layout(&self) -> PathBuf {
        self.scratch.path().join("layout")
    }

    fn key_file(&self) -> PathBuf {
        self.home().join(".config/tailrace/signing")
    }
}

impl Rig for BindingsRig {
    fn profile(&self) -> String {
        "the bindings file".into()
    }
}

impl HostRig for BindingsRig {
    fn config(&self) -> PathBuf {
        self.scratch.path().join("config")
    }

    fn landings_file(&self) -> PathBuf {
        self.config().join("sluice/config.toml")
    }

    /// The layout in the scratch directory, sluice over the repository's
    /// own files, and `signer` with the key under the home.
    fn write(&self, signer: &SignerId) -> PathBuf {
        let key = self.key_file();
        fs::create_dir_all(key.parent().unwrap()).unwrap();
        self.key
            .write_openssh_file(&key, LineEnding::LF)
            .expect("the key file written");
        fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).unwrap();
        let file = bindings::path(Some(self.config().as_os_str()), None).unwrap();
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(
            &file,
            format!(
                "[store]\nlayout = {:?}\n\n[landing]\nsluice = {{}}\n\n\
                 [signing]\nsigner = \"{signer}\"\nkey = \"~/.config/tailrace/signing\"\n",
                self.layout().display().to_string()
            ),
        )
        .unwrap();
        file
    }

    fn bind(&self) -> Answer<(Seams, Option<SignerId>)> {
        let file = bindings::path(Some(self.config().as_os_str()), None).unwrap();
        let binding = bindings::bind(
            &file,
            Some(&self.home()),
            &self.scratch.repo.path,
            For::Publish,
        )?;
        Ok((binding.seams, binding.signer))
    }

    fn stored(&self, digest: &Digest) -> bool {
        matches!(
            Layout::open(self.layout()).and_then(|layout| layout.read(digest)),
            Ok(Held::Bytes(_))
        )
    }

    fn landed(&self) -> (Branch, Revision) {
        (Branch::new(MAIN), self.landed.clone())
    }

    fn listed(&self, signer: &SignerId) -> NamedSigner {
        NamedSigner {
            id: signer.clone(),
            verified_by: self.key.public_key().to_openssh().unwrap(),
            kinds: vec![ClaimKind::Provenance],
            checks: vec![],
        }
    }
}

#[test]
fn the_host_group_over_the_bindings_file() {
    let report = run(host::GROUP, &host::scenarios(), &|| {
        Box::new(BindingsRig::new())
    });
    assert_eq!(report.profile, "the bindings file");
    assert_eq!(report.not_applicable, [], "{report}");
    report.assert();
}

/// Every file the rig writes is the scratch host's own, and the layout is
/// made by the publish's binding and not before.
#[test]
fn the_files_are_the_scratch_hosts_own() {
    let rig = BindingsRig::new();
    let file = rig.write(&SignerId::new("laptop"));
    assert!(file.starts_with(rig.scratch.path()));
    assert!(rig.key_file().starts_with(rig.home()));
    assert!(
        !rig.layout().exists(),
        "the layout was made before a publish"
    );
    rig.bind().map(|_| ()).unwrap();
    assert!(Layout::open(rig.layout()).is_ok());
}
