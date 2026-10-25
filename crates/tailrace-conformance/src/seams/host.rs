//! The host's group: 118, the operator's bindings kept in one file
//! beside the landing's own (B.9; design D7).

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;
use tailrace_seams::{
    Answer, Branch, Claim, ClaimKind, Digest, NamedSigner, Revision, Seams, SignerId,
};

use crate::{ok, Rig, Scenario};

pub const GROUP: &str = "host";

/// A host's configuration directory, holding the landing's own file, and
/// what binds from a bindings file written there, made new for each
/// scenario.
pub trait HostRig: Rig {
    /// The directory the host keeps its tools' configuration in.
    fn config(&self) -> PathBuf;

    /// The landing's own configuration file, under that directory.
    fn landings_file(&self) -> PathBuf;

    /// A bindings file written where the host keeps it, binding a store,
    /// the landing whose own file [`landings_file`](HostRig::landings_file)
    /// is, and `signer` with a key of its own; answers the file's path.
    fn write(&self, signer: &SignerId) -> PathBuf;

    /// The binding the host's file makes for a publish: the seams, and
    /// the signer it signs as.
    fn bind(&self) -> Answer<(Seams, Option<SignerId>)>;

    /// Whether the store the file names holds `digest`, read apart from
    /// the binding.
    fn stored(&self, digest: &Digest) -> bool;

    /// A revision the landing the file names has landed, and its branch.
    fn landed(&self) -> (Branch, Revision);

    /// `signer` as a policy lists it, verified by the key the file names.
    fn listed(&self, signer: &SignerId) -> NamedSigner;
}

pub fn scenarios() -> Vec<Scenario<dyn HostRig>> {
    vec![
        Scenario {
            name: "the host's bindings beside the landing's",
            plays: None,
            cites: &[118],
            run: the_hosts_bindings_beside_the_landings,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}

/// The file's directory under the configuration directory, and its
/// name.
fn placed(file: &Path, config: &Path) -> (PathBuf, String) {
    let dir = file
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("{} is not in a directory of its own", file.display()));
    assert_eq!(
        dir,
        config,
        "{} is not under the configuration directory {}",
        file.display(),
        config.display()
    );
    let name = file
        .file_name()
        .unwrap_or_else(|| panic!("{} names no file", file.display()))
        .to_string_lossy()
        .into_owned();
    (dir.to_path_buf(), name)
}

fn the_hosts_bindings_beside_the_landings(rig: &dyn HostRig) {
    let signer = SignerId::new("laptop");
    let file = rig.write(&signer);
    let (config, landings) = (rig.config(), rig.landings_file());
    assert_eq!(
        file,
        config.join("tailrace").join("config.toml"),
        "the bindings file is elsewhere than tailrace/config.toml under the configuration directory"
    );
    let (_, ours) = placed(&file, &config);
    let (_, theirs) = placed(&landings, &config);
    assert_eq!(
        ours, theirs,
        "the bindings file is not named as the landing's own is"
    );
    for read in [&file, &landings] {
        let text = fs::read_to_string(read)
            .unwrap_or_else(|e| panic!("{} does not read as text: {e}", read.display()));
        assert!(!text.is_empty(), "{} is empty", read.display());
    }

    let (seams, bound) = ok(rig.bind(), "the binding of the host's file");
    assert_eq!(
        bound.as_ref(),
        Some(&signer),
        "the binding signs as another"
    );

    let put = ok(seams.store.put_bytes(b"bound"), "a put to the bound store");
    assert!(
        rig.stored(&put.digest),
        "the bound store is not the store the file names"
    );

    let (branch, revision) = rig.landed();
    assert_eq!(
        seams.landing.landed(&branch, &revision),
        Ok(true),
        "the bound landing is not the landing the file names"
    );

    let claim = Claim {
        subject: put.digest,
        kind: ClaimKind::Provenance,
        revision,
        build: None,
        body: json!({}),
    };
    let Some(host) = seams.signers.first() else {
        panic!("no signer is bound for the host")
    };
    let envelope = ok(host.sign(&claim, &signer), "a sign by the bound signer");
    let made = ok(
        host.verify(&envelope, &[rig.listed(&signer)]),
        "a verify by the key the file names",
    );
    assert_eq!(
        made.signer, signer,
        "the bound signer signs with another key"
    );
}
