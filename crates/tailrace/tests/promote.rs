//! Spec promotion, a channel read from the ledger alone, and spec
//! git-ref-ledger, over real profiles on two machines: a bare remote, and
//! two clones of it in a scratch directory, each with a bindings file of
//! its own. Machine A lands on a plain branch, holds an OCI layout and a
//! host key the policy file names as the gate, and promotes through
//! `tailrace::run`; machine B, bound to an empty layout, reads the
//! channel from the git ref ledger alone. Needs git alone, so it is in
//! the fast layer.

mod scratch;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use clap::Parser;
use scratch::{Repo, Scratch, MAIN, POLICY_FILE};
use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey};
use tailrace::bindings::{self, For};
use tailrace::profiles::store::layout::Layout;
use tailrace::{run, Binding, Cli};
use tailrace_core::{channel, Channel};
use tailrace_seams::{
    channel_name, Answer, ChannelName, Claim, ClaimKind, ComponentName, Digest, IndexMeta,
    Manifest, Name, Not, Read, Revision, SignerId, Store, Version,
};

const GATE: &str = "gate";

fn stable() -> ChannelName {
    ChannelName::new("stable")
}

/// A machine: its clone of the remote, its layout, and its bindings file
/// and home under a directory of its own.
struct Machine {
    repo: Repo,
    dir: PathBuf,
}

impl Machine {
    /// `repo`, bound to a layout made empty in `dir`, a plain branch
    /// landing and the git ref ledger on `origin`, and the gate's key,
    /// held where `key` is given.
    fn new(repo: Repo, dir: PathBuf, key: Option<&PrivateKey>) -> Machine {
        let machine = Machine { repo, dir };
        Layout::create(machine.layout()).unwrap();
        let signing = match key {
            Some(key) => {
                let file = machine.home().join(".config/tailrace/signing");
                fs::create_dir_all(file.parent().unwrap()).unwrap();
                key.write_openssh_file(&file, LineEnding::LF).unwrap();
                fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
                format!("\n[signing]\nsigner = \"{GATE}\"\nkey = \"~/.config/tailrace/signing\"\n")
            }
            None => String::new(),
        };
        let file = machine.bindings_file();
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(
            &file,
            format!(
                "[store]\nlayout = {:?}\n\n[landing]\nbranch = {{ remote = \"origin\" }}\n\n\
                 [ledger]\ngit_ref = {{ remote = \"origin\" }}\n{signing}",
                machine.layout().display().to_string()
            ),
        )
        .unwrap();
        machine
    }

    fn home(&self) -> PathBuf {
        self.dir.join("home")
    }

    fn layout(&self) -> PathBuf {
        self.dir.join("layout")
    }

    fn bindings_file(&self) -> PathBuf {
        bindings::path(Some(self.dir.join("config").as_os_str()), None).unwrap()
    }

    fn bind(&self, command: For) -> Answer<Binding> {
        bindings::bind(
            &self.bindings_file(),
            Some(&self.home()),
            &self.repo.path,
            command,
        )
    }

    /// `args` asked through `tailrace::run` over `binding`: the exit and
    /// what it printed.
    fn run(&self, binding: &Answer<Binding>, args: &[&str]) -> (i32, String) {
        let cli = Cli::try_parse_from(std::iter::once("tailrace").chain(args.iter().copied()))
            .expect("the arguments parse");
        let mut out = Vec::new();
        let code = run(&cli, binding, &self.repo.path, &mut out);
        (code, String::from_utf8(out).unwrap())
    }
}

/// Every file under `dir` with its contents, to tell a layout untouched.
fn files(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut all = Vec::new();
    let mut dirs = vec![dir.to_path_buf()];
    while let Some(at) = dirs.pop() {
        for entry in fs::read_dir(&at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
            } else {
                all.push((path.clone(), fs::read(&path).unwrap()));
            }
        }
    }
    all.sort();
    all
}

/// The scratch remote, machine A with two versions of the repository's
/// one component landed, tagged and put in its layout, each index
/// carrying the gate's acceptance, and machine B, a second clone with an
/// empty layout and no key.
struct Two {
    scratch: Scratch,
    a: Machine,
    b: Machine,
    indexes: [Digest; 2],
}

fn two() -> Two {
    let scratch = Scratch::new();
    let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).expect("an Ed25519 key");
    let repo = scratch.repo.clone();
    repo.write(
        POLICY_FILE,
        &format!(
            "outputs = [\"site\"]\n\n[projections]\nsite = \"file\"\n\n\
             [signers.{GATE}]\nssh = \"{}\"\nkinds = [\"acceptance\", \"promotion\"]\n\n\
             [channels.stable]\nrequire = [{{ kind = \"acceptance\", signers = [\"{GATE}\"] }}]\n",
            key.public_key().to_openssh().unwrap()
        ),
    );
    repo.write("site/index.html", "<p>hello</p>");
    let first = Revision::new(repo.commit("feat: the site"));
    repo.write("site/index.html", "<p>hello again</p>");
    let second = Revision::new(repo.commit("feat: the site grows"));
    repo.git(&["tag", "v0.1.0", first.as_str()]);
    repo.git(&["tag", "v0.2.0", second.as_str()]);
    repo.git(&["push", "-q", "origin", MAIN, "v0.1.0", "v0.2.0"]);

    let a = Machine::new(repo, scratch.path().join("a"), Some(&key));
    let binding = a.bind(For::Promote).expect("machine A binds");
    let mut indexes = Vec::new();
    for (version, revision) in [("0.1.0", &first), ("0.2.0", &second)] {
        let meta = IndexMeta::new(
            ComponentName::new("repository"),
            Version::new(version),
            vec![],
        )
        .unwrap();
        let store = &binding.seams.store;
        let index = store.put_artifact(&Manifest::Index(meta)).unwrap().digest;
        store
            .name(&Name::new(format!("v{version}")), &index, None)
            .unwrap();
        let acceptance = Claim {
            subject: index.clone(),
            kind: ClaimKind::Acceptance,
            revision: revision.clone(),
            build: None,
            body: serde_json::json!({ "version": version }),
        };
        let envelope = binding.seams.signers[0]
            .sign(&acceptance, &SignerId::new(GATE))
            .expect("the gate's key signs");
        store.attach(&index, &envelope).unwrap();
        indexes.push(index);
    }
    let b = Machine::new(scratch.clone_as("b"), scratch.path().join("b"), None);
    Two {
        scratch,
        a,
        b,
        indexes: indexes.try_into().unwrap(),
    }
}

fn at_of(channel: Answer<Channel>) -> (tailrace_seams::Promotion, Vec<Digest>, Read) {
    match channel {
        Ok(Channel::At { head, chain, read }) => {
            (head, chain.into_iter().map(|(d, _)| d).collect(), read)
        }
        other => panic!("the channel answered {other:?}"),
    }
}

#[test]
fn a_promotion_on_one_machine_is_read_on_another_from_the_ledger_alone() {
    let Two {
        scratch,
        a,
        b,
        indexes: [first, second],
    } = two();
    let binding = a.bind(For::Promote);
    for args in [
        &["promote", "0.1.0", "--to", "stable"][..],
        &["promote", "0.2.0", "--to", "stable", "--from", "0.1.0"][..],
        &["promote", "--back", "--to", "stable"][..],
    ] {
        let (code, printed) = a.run(&binding, args);
        assert_eq!(code, 0, "{args:?}: {printed}");
        assert!(printed.contains("the store followed"), "{printed}");
    }
    let layout = Layout::open(a.layout()).unwrap();
    assert_eq!(layout.resolve(&channel_name(&stable())), Ok(first.clone()));
    let on = |index: &Digest| {
        layout
            .claims_of(index)
            .unwrap()
            .iter()
            .filter(|e| Claim::from_envelope(e).is_some_and(|c| c.kind == ClaimKind::Promotion))
            .count()
    };
    assert_eq!((on(&first), on(&second)), (2, 1));

    let ours = at_of(channel(
        binding.as_ref().unwrap().seams.keeper.as_ref(),
        &stable(),
    ));
    assert_eq!(ours.0.to.index, first);
    assert_eq!(ours.0.from.as_ref().map(|f| &f.index), Some(&second));
    assert_eq!(ours.1.len(), 3);

    let untouched = files(&b.layout());
    let theirs = b.bind(For::Verify).expect("machine B binds");
    let before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let read = at_of(channel(theirs.seams.keeper.as_ref(), &stable()));
    assert_eq!(read.0, ours.0, "machine B read another head");
    assert_eq!(read.1, ours.1, "machine B read another chain");
    assert_eq!(read.2, Read::Source);
    assert_eq!(
        files(&b.layout()),
        untouched,
        "machine B's layout was touched"
    );

    fs::remove_dir_all(&scratch.remote).unwrap();
    let kept = at_of(channel(theirs.seams.keeper.as_ref(), &stable()));
    assert_eq!((kept.0, kept.1), (ours.0, ours.1));
    match kept.2 {
        Read::Kept { at, said } => {
            assert!(at >= before, "kept at {at}, read at {before}");
            assert!(
                said.contains("does not appear to be a git repository"),
                "{said}"
            );
        }
        Read::Source => panic!("the chain was read from a remote that is gone"),
    }
    assert_eq!(
        files(&b.layout()),
        untouched,
        "machine B's layout was touched"
    );
}

#[test]
fn a_promotion_found_wanting_on_real_profiles_writes_nothing() {
    // The scratch directory is kept for as long as the machines are.
    let Two {
        scratch: _kept, a, ..
    } = two();
    let binding = a.bind(For::Promote);
    let (code, printed) = a.run(&binding, &["promote", "0.3.0", "--to", "stable"]);
    assert_eq!(code, 4, "{printed}");
    assert!(printed.contains("v0.3.0"), "{printed}");
    let (code, printed) = a.run(&binding, &["promote", "0.2.0", "--to", "beta"]);
    assert_eq!(code, 4, "{printed}");
    assert!(printed.contains("channel beta"), "{printed}");
    assert_eq!(
        a.repo.git(&["for-each-ref", "refs/tailrace/ledger"]),
        "",
        "a ledger ref was written"
    );
    let layout = Layout::open(a.layout()).unwrap();
    assert!(matches!(
        layout.resolve(&channel_name(&stable())),
        Err(Not::Absent(_))
    ));
}
