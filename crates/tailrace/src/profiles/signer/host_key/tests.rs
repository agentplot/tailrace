//! Spec host-key-signer: sign and verify over keys made fresh in a
//! temporary directory, and the fixed texts of keys this profile refuses.

use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use serde_json::json;
use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, HashAlg, LineEnding, PrivateKey, SshSig};
use tailrace_fakes::FakeSigner;
use tailrace_seams::{
    Answer, Claim, ClaimKind, Digest, Envelope, Made, NamedSigner, Not, Revision, Seam, Signature,
    Signer, SignerId, Why, IN_TOTO,
};
use tempfile::TempDir;

use super::*;

/// An Ed25519 key under the passphrase `a passphrase`, as ssh-keygen
/// wrote it.
const ENCRYPTED: &str = "-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAACmFlczI1Ni1jdHIAAAAGYmNyeXB0AAAAGAAAABBt0ZJMNb
MxEEUt5o1jM1tOAAAAGAAAAAEAAAAzAAAAC3NzaC1lZDI1NTE5AAAAINO3uylRL1FAKOU+
UAtGuMyYx2wlTWgapZ8fhBW5wCmZAAAAkKMuRupyQsLLAwkzmQV0QS9lOw9a9QwBuMguh4
oWoGv+2nhQPJDltI1rREOPDfuRw9mqWrIwGyUMruMBXDUrS+irTUixwk4FwUx3Zq4OnT6Z
2nzvif9Y8Ag56/2VWWWXB6gRz70mfKs445qm4yCFLkanC7Tqn72NoyldEJw6YSjhdjiU/I
4O0PsWfNxduD2dbA==
-----END OPENSSH PRIVATE KEY-----
";

/// A 1024-bit RSA key with no passphrase, as ssh-keygen wrote it.
const RSA: &str = "-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAlwAAAAdzc2gtcn
NhAAAAAwEAAQAAAIEAxOyEGUl380cc6CtekKPaYsIJKfb3xTPtB30BwXcS840sMhwaQB+s
F/cC605G/VoGGrLEkvqzUyoGFsnaVnpRVXJkboevwflvomBNEgYivvu6SwZC2eJ0WMT7Kg
sYwC2heT+I+YVi1bKOlr3VNLevQ686AMBl2cO/dLaFr+55lYMAAAH4ymNer8pjXq8AAAAH
c3NoLXJzYQAAAIEAxOyEGUl380cc6CtekKPaYsIJKfb3xTPtB30BwXcS840sMhwaQB+sF/
cC605G/VoGGrLEkvqzUyoGFsnaVnpRVXJkboevwflvomBNEgYivvu6SwZC2eJ0WMT7KgsY
wC2heT+I+YVi1bKOlr3VNLevQ686AMBl2cO/dLaFr+55lYMAAAADAQABAAAAgAsvC4mMHJ
3aag/ddq9sPtjhADg76HJwsoN8W/8qzg6688mcwvjlU5bA1o+akHmPBU263WMjjc+pUVM1
/4ptw/SmzrHD4ehV7Ic2HU7B+NcBI5BJ+h0dkmnUnpKYJIsT7HWL0bVH83zHmS21300CGZ
8FkJjCu9+wVArz3w0lZaexAAAAQGzi9VbB3kVYMZ5WBkgTwraeZtSzFO5aHgtiBiyzyFti
Etw34OzG9Xa3fk142o3kc6qWumqNtDqwnIhlrOEnSv8AAABBAO8jLJhIFvg5hsAvuK0XGP
4Gn5VVWEx1swz4rcEDXot8mndu4lUHrVlMInGklQ9bH8zvBZWfSrxl0F4862GtTxcAAABB
ANLPUXCYwxbi8XzdDF0P+7FRFATHDTs8umtcXdbd/4VJxd0zMD+Si1FIsSokmGh1E9hqL1
k6N80uOOqCv5ElEHUAAAADcnNh
-----END OPENSSH PRIVATE KEY-----
";

fn new_key() -> PrivateKey {
    PrivateKey::random(&mut OsRng, Algorithm::Ed25519).expect("an Ed25519 key")
}

fn mode(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("the key file's mode set");
}

/// `key` written to `name` in `dir` with mode 0600.
fn written(dir: &Path, name: &str, key: &PrivateKey) -> PathBuf {
    let path = dir.join(name);
    key.write_openssh_file(&path, LineEnding::LF)
        .expect("the key file written");
    mode(&path, 0o600);
    path
}

/// `text` written to `name` in `dir` with mode 0600.
fn text(dir: &Path, name: &str, text: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, text).expect("the file written");
    mode(&path, 0o600);
    path
}

fn host(key: &Path) -> HostKey {
    HostKey::new(SignerId::new("host"), key)
}

fn claim() -> Claim {
    Claim {
        subject: Digest::of(b"an index"),
        kind: ClaimKind::Provenance,
        revision: Revision::new("4f2a9c1"),
        build: Some(tailrace_seams::BuildId::new("b7e1")),
        body: json!({ "outputs": ["site"] }),
    }
}

fn sign(host: &HostKey) -> Answer<Envelope> {
    host.sign(&claim(), &SignerId::new("host"))
}

/// The envelope's signature, parsed as the armored SSH signature it is.
fn ssh_sig(envelope: &Envelope) -> SshSig {
    let held = envelope.sole().expect("the envelope holds one signature");
    let pem = std::str::from_utf8(&held.sig).expect("the signature is text");
    SshSig::from_pem(pem).expect("the signature is an SSH signature")
}

fn absent(answer: Answer<Envelope>) -> String {
    match answer {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source, HostKey::source());
            a.what
        }
        other => panic!("answered {other:?}, not absent"),
    }
}

/// The words of a refusal that names nothing beyond them.
fn refused(answer: Answer<Envelope>) -> String {
    match answer {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source, HostKey::source());
            assert_eq!(r.why, Why::Said);
            r.said.expect("the refusal says why")
        }
        other => panic!("answered {other:?}, not refused"),
    }
}

/// What could not be read, and the words it was said in.
fn unreadable(answer: Answer<Envelope>) -> (String, String) {
    match answer {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, HostKey::source());
            (u.what, u.said.expect("the source's words"))
        }
        other => panic!("answered {other:?}, not unreadable"),
    }
}

#[test]
fn the_encoding_is_the_dsse_specifications() {
    assert_eq!(
        pae("http://example.com/HelloWorld", b"hello world"),
        b"DSSEv1 29 http://example.com/HelloWorld 11 hello world"
    );
    assert_eq!(pae("", b""), b"DSSEv1 0  0 ");
}

#[test]
fn the_payload_is_carried_byte_for_byte() {
    let payload = [0xff, 0x00, b' ', 0x80];
    let bytes = pae("t", &payload);
    assert_eq!(&bytes[..11], b"DSSEv1 1 t ");
    assert_eq!(&bytes[11..], b"4 \xff\x00 \x80");
}

#[test]
fn every_answer_names_the_signer_seam_and_the_host_key() {
    let source = HostKey::source();
    assert_eq!(source.seam, Seam::Signer);
    assert_eq!(source.to_string(), "the signer (host key)");
    let host = host(Path::new("/keys/host"));
    assert_eq!(host.signer(), Some(&SignerId::new("host")));
    assert_eq!(host.key(), Some(Path::new("/keys/host")));
    let verifying = HostKey::verifying();
    assert_eq!((verifying.signer(), verifying.key()), (None, None));
}

#[test]
fn signed_as_the_host() {
    let dir = TempDir::new().unwrap();
    let key = new_key();
    let envelope = sign(&host(&written(dir.path(), "host", &key))).unwrap();
    assert_eq!(Claim::from_envelope(&envelope), Some(claim()));
    assert_eq!(envelope.payload_type, IN_TOTO);
    assert_eq!(envelope.payload, claim().payload());
    let held = envelope.sole().expect("the envelope holds one signature");
    assert_eq!(held.signer, SignerId::new("host"));
    let pem = std::str::from_utf8(&held.sig).unwrap();
    assert!(pem.starts_with("-----BEGIN SSH SIGNATURE-----\n"), "{pem}");
    let sig = ssh_sig(&envelope);
    let signed = pae(&envelope.payload_type, &envelope.payload);
    let public = key.public_key();
    assert_eq!(public.verify(NAMESPACE, &signed, &sig), Ok(()));
    assert_eq!(
        public.verify("git", &signed, &sig),
        Err(ssh_key::Error::Namespace)
    );
    assert!(public
        .verify(NAMESPACE, &pae(&envelope.payload_type, b"another"), &sig)
        .is_err());
}

#[test]
fn asked_to_sign_as_another_reads_no_key() {
    let dir = TempDir::new().unwrap();
    let path = written(dir.path(), "host", &new_key());
    // A read of the file would fail, so an answer that read it would say so.
    mode(&path, 0o000);
    let what = absent(host(&path).sign(&claim(), &SignerId::new("gate")));
    assert_eq!(what, "identity gate");
}

#[test]
fn a_missing_key_file_is_no_identity_at_hand() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("host");
    let what = absent(sign(&host(&path)));
    assert!(what.contains("identity host"), "{what}");
    assert!(what.contains(&path.display().to_string()), "{what}");
}

#[test]
fn a_key_others_may_read_is_refused() {
    let dir = TempDir::new().unwrap();
    let path = written(dir.path(), "host", &new_key());
    mode(&path, 0o644);
    let said = refused(sign(&host(&path)));
    let shown = path.display().to_string();
    assert_eq!(
        said,
        format!(
            "the key file {shown} has mode 0644, which lets others read it: \
             give it mode 0600 (chmod 600 {shown})"
        )
    );
    mode(&path, 0o640);
    assert!(refused(sign(&host(&path))).contains("has mode 0640, which lets others read it"));
}

#[test]
fn a_key_with_a_passphrase_is_refused() {
    let dir = TempDir::new().unwrap();
    let path = text(dir.path(), "host", ENCRYPTED);
    assert!(PrivateKey::from_openssh(ENCRYPTED).unwrap().is_encrypted());
    let said = refused(sign(&host(&path)));
    assert_eq!(
        said,
        format!(
            "the key file {} has a passphrase; tailrace signs with none to type",
            path.display()
        )
    );
}

#[test]
fn a_key_of_another_algorithm_is_refused_naming_it() {
    let dir = TempDir::new().unwrap();
    let path = text(dir.path(), "host", RSA);
    let said = refused(sign(&host(&path)));
    assert_eq!(
        said,
        format!(
            "the key file {} holds an ssh-rsa key; tailrace signs with ssh-ed25519 keys",
            path.display()
        )
    );
}

#[test]
fn a_key_file_that_is_a_directory_is_unreadable_in_the_systems_words() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("host");
    fs::create_dir(&path).unwrap();
    let words = fs::read(&path).unwrap_err().to_string();
    let (what, said) = unreadable(sign(&host(&path)));
    assert_eq!(what, format!("the key file {}", path.display()));
    assert_eq!(said, words);
}

#[test]
fn a_key_file_that_is_not_a_key_is_unreadable_in_the_parsers_words() {
    let dir = TempDir::new().unwrap();
    let path = text(dir.path(), "host", "no key here\n");
    let words = PrivateKey::from_openssh("no key here\n")
        .unwrap_err()
        .to_string();
    let (what, said) = unreadable(sign(&host(&path)));
    assert_eq!(what, format!("the key file {}", path.display()));
    assert_eq!(said, words);
}

#[test]
fn the_key_replaced_between_two_signatures_signs_the_second() {
    let dir = TempDir::new().unwrap();
    let (old, new) = (new_key(), new_key());
    let path = written(dir.path(), "host", &old);
    let host = host(&path);
    let first = sign(&host).unwrap();
    written(dir.path(), "host", &new);
    let second = sign(&host).unwrap();
    let signed = pae(&second.payload_type, &second.payload);
    assert_eq!(
        new.public_key()
            .verify(NAMESPACE, &signed, &ssh_sig(&second)),
        Ok(())
    );
    assert!(old
        .public_key()
        .verify(NAMESPACE, &signed, &ssh_sig(&second))
        .is_err());
    assert_eq!(
        old.public_key()
            .verify(NAMESPACE, &signed, &ssh_sig(&first)),
        Ok(())
    );
}

#[test]
fn the_key_file_removed_after_a_signature_is_no_identity_at_hand() {
    let dir = TempDir::new().unwrap();
    let path = written(dir.path(), "host", &new_key());
    let host = host(&path);
    sign(&host).unwrap();
    fs::remove_file(&path).unwrap();
    let what = absent(sign(&host));
    assert_eq!(
        what,
        format!("identity host: no key file at {}", path.display())
    );
}

/// Whether `bytes` holds `part` anywhere.
fn holds(bytes: &[u8], part: &[u8]) -> bool {
    bytes.windows(part.len()).any(|w| w == part)
}

#[test]
fn nothing_of_the_key_leaves_it() {
    let dir = TempDir::new().unwrap();
    let key = new_key();
    let path = written(dir.path(), "host", &key);
    let file = fs::read_to_string(&path).unwrap();
    let secret = key.key_data().ed25519().unwrap().private.to_bytes();
    let envelope = sign(&host(&path)).unwrap();
    let serialized = serde_json::to_vec(&envelope).unwrap();
    for bytes in [
        &serialized,
        &envelope.payload,
        &envelope.sole().unwrap().sig,
    ] {
        assert!(!holds(bytes, file.as_bytes()));
        assert!(!holds(bytes, &secret));
        for line in file.lines().filter(|l| !l.starts_with("-----")) {
            assert!(!holds(bytes, line.as_bytes()), "the envelope holds {line}");
        }
    }
    let listed: Vec<PathBuf> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(listed, [path]);
}

#[test]
fn the_profile_is_its_signer_and_its_path_alone() {
    let dir = TempDir::new().unwrap();
    let path = written(dir.path(), "host", &new_key());
    let bound = host(&path);
    sign(&bound).unwrap();
    // Every field named, so a field added to hold anything of the key
    // fails to compile here.
    let HostKey {
        signing: Some(Signing { signer, key, given }),
    } = bound
    else {
        panic!("a keyed host key holds no signer and path")
    };
    assert_eq!(signer, SignerId::new("host"));
    assert_eq!(key, path);
    assert!(!given, "a named key is taken for the given one");
    let HostKey { signing: None } = HostKey::verifying() else {
        panic!("the verify-only host key holds a signer")
    };
}

// Requirement: The given host key is made on first use

/// The given key at `tailrace/signing` under `dir`, signing as `laptop`.
fn given(dir: &Path) -> HostKey {
    HostKey::given(SignerId::new("laptop"), dir.join("tailrace/signing"))
}

fn sign_given(host: &HostKey) -> Answer<Envelope> {
    host.sign(&claim(), &SignerId::new("laptop"))
}

/// `signing.pub`'s one line as a policy lists it, under `id`.
fn listed_as(id: &str, public: &Path) -> NamedSigner {
    NamedSigner {
        id: SignerId::new(id),
        verified_by: fs::read_to_string(public).unwrap().trim().to_string(),
        kinds: vec![ClaimKind::Provenance],
        checks: vec![],
    }
}

/// Each file's mode, modification time and bytes.
fn state(paths: &[&Path]) -> Vec<(u32, std::time::SystemTime, Vec<u8>)> {
    paths
        .iter()
        .map(|path| {
            let meta = fs::metadata(path).unwrap();
            (
                meta.permissions().mode() & 0o777,
                meta.modified().unwrap(),
                fs::read(path).unwrap(),
            )
        })
        .collect()
}

#[test]
fn the_first_sign_makes_the_key_and_its_public_half_which_a_policy_lists_as_it_stands() {
    let dir = TempDir::new().unwrap();
    let host = given(dir.path());
    let envelope = sign_given(&host).unwrap();
    let key = dir.path().join("tailrace/signing");
    let public = dir.path().join("tailrace/signing.pub");
    assert_eq!(public_half(&key), public);
    assert_eq!(
        fs::metadata(&key).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        fs::metadata(&public).unwrap().permissions().mode() & 0o777,
        0o644
    );
    assert_eq!(
        fs::metadata(dir.path().join("tailrace"))
            .unwrap()
            .permissions()
            .mode()
            & 0o077,
        0,
        "the key's directory lets others in"
    );
    let made = PrivateKey::read_openssh_file(&key).unwrap();
    assert!(!made.is_encrypted());
    assert_eq!(made.algorithm(), Algorithm::Ed25519);
    let line = fs::read_to_string(&public).unwrap();
    assert!(line.starts_with("ssh-ed25519 "), "{line}");
    assert!(line.ends_with(" laptop\n"), "{line}");
    let listing = listed_as("laptop", &public);
    let read = listed::read(&listing)
        .unwrap()
        .expect("a host key's listing");
    assert_eq!(read.key.key_data(), made.public_key().key_data());
    assert_eq!(read.valid_before, None);
    assert_eq!(
        HostKey::verifying().verify(&envelope, &[listing]),
        made_by("laptop")
    );
    let names: BTreeSet<_> = fs::read_dir(dir.path().join("tailrace"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(
        names,
        BTreeSet::from(["signing".into(), "signing.pub".into()]),
        "the first sign left a file of its own"
    );
}

#[test]
fn the_second_sign_signs_with_the_key_the_first_made_and_writes_neither_file() {
    let dir = TempDir::new().unwrap();
    let host = given(dir.path());
    let first = sign_given(&host).unwrap();
    let key = dir.path().join("tailrace/signing");
    let public = public_half(&key);
    let before = state(&[&key, &public]);
    std::thread::sleep(std::time::Duration::from_millis(20));
    let second = sign_given(&host).unwrap();
    assert_eq!(state(&[&key, &public]), before);
    let listing = listed_as("laptop", &public);
    for envelope in [&first, &second] {
        assert_eq!(
            HostKey::verifying().verify(envelope, std::slice::from_ref(&listing)),
            made_by("laptop")
        );
    }
}

#[test]
fn a_key_others_may_read_at_the_given_path_is_refused_and_kept() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("tailrace")).unwrap();
    let path = written(&dir.path().join("tailrace"), "signing", &new_key());
    mode(&path, 0o644);
    let bytes = fs::read(&path).unwrap();
    let said = refused(sign_given(&given(dir.path())));
    let shown = path.display().to_string();
    assert_eq!(
        said,
        format!(
            "the key file {shown} has mode 0644, which lets others read it: \
             give it mode 0600 (chmod 600 {shown})"
        )
    );
    assert_eq!(fs::read(&path).unwrap(), bytes, "the key file was replaced");
    assert!(!public_half(&path).exists(), "a public half was written");
}

#[test]
fn two_first_signs_at_once_make_one_key_and_both_sign_with_it() {
    let dir = TempDir::new().unwrap();
    let host = given(dir.path());
    let ready = std::sync::Barrier::new(2);
    let [first, second] = std::thread::scope(|scope| {
        let signs = [(); 2].map(|()| {
            scope.spawn(|| {
                ready.wait();
                sign_given(&host)
            })
        });
        signs.map(|sign| sign.join().unwrap().unwrap())
    });
    let tailrace = dir.path().join("tailrace");
    let names: BTreeSet<_> = fs::read_dir(&tailrace)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(
        names,
        BTreeSet::from(["signing".into(), "signing.pub".into()])
    );
    let made = PrivateKey::read_openssh_file(&tailrace.join("signing")).unwrap();
    let listing = listed_as("laptop", &tailrace.join("signing.pub"));
    assert_eq!(
        listed::read(&listing).unwrap().unwrap().key.key_data(),
        made.public_key().key_data()
    );
    for envelope in [&first, &second] {
        assert_eq!(
            HostKey::verifying().verify(envelope, std::slice::from_ref(&listing)),
            made_by("laptop")
        );
    }
}

#[test]
fn verifying_makes_no_key() {
    let dir = TempDir::new().unwrap();
    let key = new_key();
    let envelope = signed_by(&key);
    let listing = [listing("laptop", &key, None)];
    assert_eq!(
        HostKey::verifying().verify(&envelope, &listing),
        made_by("laptop")
    );
    assert_eq!(
        given(dir.path()).verify(&envelope, &listing),
        made_by("laptop")
    );
    assert_eq!(
        absent(given(dir.path()).sign(&claim(), &SignerId::new("gate"))),
        "identity gate"
    );
    assert_eq!(
        fs::read_dir(dir.path()).unwrap().count(),
        0,
        "a file was made"
    );
}

#[test]
fn a_key_directory_that_cannot_be_made_is_unreadable_naming_the_path() {
    let dir = TempDir::new().unwrap();
    let locked = dir.path().join("locked");
    fs::create_dir(&locked).unwrap();
    mode(&locked, 0o555);
    let host = given(&locked);
    let answer = sign_given(&host);
    let words = fs::create_dir(locked.join("tailrace"))
        .unwrap_err()
        .to_string();
    mode(&locked, 0o755);
    let (what, said) = unreadable(answer);
    assert_eq!(
        what,
        format!("the key file {}", locked.join("tailrace/signing").display())
    );
    assert_eq!(said, words);
    assert_eq!(fs::read_dir(&locked).unwrap().count(), 0);
}

/// `id` as a policy lists it, verified by `key`'s public half and, where
/// given, valid before `before`, for every kind of claim.
fn listing(id: &str, key: &PrivateKey, before: Option<&str>) -> NamedSigner {
    let public = key.public_key().to_openssh().unwrap();
    NamedSigner {
        id: SignerId::new(id),
        verified_by: match before {
            Some(time) => format!("valid-before=\"{time}\" {public} {id}@a-host"),
            None => format!("{public} {id}@a-host"),
        },
        kinds: vec![ClaimKind::Provenance, ClaimKind::Verdict],
        checks: vec![],
    }
}

/// A host's envelope, signed with a key in a directory of its own.
fn signed_by(key: &PrivateKey) -> Envelope {
    let dir = TempDir::new().unwrap();
    sign(&host(&written(dir.path(), "host", key))).unwrap()
}

/// A profile with no key file, as a machine that only verifies has.
fn verifier() -> HostKey {
    HostKey::new(SignerId::new("a verifier"), "/no/key/file/here")
}

fn made_by(id: &str) -> Answer<Made> {
    made(id, false)
}

fn made(id: &str, expired: bool) -> Answer<Made> {
    Ok(Made {
        signer: SignerId::new(id),
        expired,
    })
}

/// The words of a refusal that none of `checked` made the envelope.
fn none_of(answer: Answer<Made>, checked: &[&str]) -> String {
    match answer {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source, HostKey::source());
            assert_eq!(
                r.why,
                Why::NoNamedSigner {
                    checked: checked.iter().map(|id| SignerId::new(*id)).collect()
                }
            );
            r.said.expect("the refusal says why")
        }
        other => panic!("answered {other:?}, not refused naming the signers checked"),
    }
}

#[test]
fn verified_with_no_key_file_names_the_host() {
    let dir = TempDir::new().unwrap();
    let key = new_key();
    let path = written(dir.path(), "host", &key);
    let envelope = sign(&host(&path)).unwrap();
    fs::remove_file(&path).unwrap();
    let signers = [listing("host", &key, None)];
    assert_eq!(host(&path).verify(&envelope, &signers), made("host", false));
    assert_eq!(verifier().verify(&envelope, &signers), made("host", false));
}

#[test]
fn the_name_the_envelope_gives_itself_decides_nothing() {
    let (key, other) = (new_key(), new_key());
    let mut envelope = signed_by(&key);
    envelope.signatures[0].signer = SignerId::new("gate");
    let signers = [listing("gate", &other, None), listing("host", &key, None)];
    assert_eq!(verifier().verify(&envelope, &signers), made("host", false));
}

#[test]
fn an_envelope_changed_after_signing_is_refused() {
    let key = new_key();
    let envelope = signed_by(&key);
    let signers = [listing("host", &key, None)];
    let payload = Envelope {
        payload: Claim {
            subject: Digest::of(b"another index"),
            ..claim()
        }
        .payload(),
        ..envelope.clone()
    };
    let payload_type = Envelope {
        payload_type: "application/json".into(),
        ..envelope.clone()
    };
    let fingerprint = key.public_key().fingerprint(HashAlg::Sha256).to_string();
    for changed in [payload, payload_type] {
        let said = none_of(verifier().verify(&changed, &signers), &["host"]);
        assert!(said.contains(&fingerprint), "{said}");
    }
    assert_eq!(verifier().verify(&envelope, &signers), made("host", false));
}

#[test]
fn another_profiles_envelope_is_refused_never_unreadable() {
    let fake = FakeSigner::new();
    fake.hold(&SignerId::new("host"));
    let envelope = fake.sign(&claim(), &SignerId::new("host")).unwrap();
    let signers = [listing("host", &new_key(), None)];
    let said = none_of(verifier().verify(&envelope, &signers), &["host"]);
    assert_eq!(said, "the signature is not an SSH signature");
    let binary = Envelope {
        signatures: vec![Signature {
            signer: SignerId::new("host"),
            keyid: String::new(),
            sig: vec![0xff, 0xfe, 0x00],
        }],
        ..envelope
    };
    let said = none_of(verifier().verify(&binary, &signers), &["host"]);
    assert_eq!(said, "the signature is not an SSH signature");
}

#[test]
fn a_signature_in_another_namespace_is_refused_naming_it() {
    let key = new_key();
    let payload = claim().payload();
    let sig = key
        .sign("git", HashAlg::Sha512, &pae(IN_TOTO, &payload))
        .unwrap()
        .to_pem(LineEnding::LF)
        .unwrap();
    let envelope = Envelope::signed(
        IN_TOTO,
        payload,
        Signature {
            signer: SignerId::new("host"),
            keyid: String::new(),
            sig: sig.into_bytes(),
        },
    );
    let said = none_of(
        verifier().verify(&envelope, &[listing("host", &key, None)]),
        &["host"],
    );
    assert_eq!(
        said,
        "the signature was made in the namespace git, not tailrace-claim"
    );
}

#[test]
fn whichever_of_an_envelopes_signatures_the_listed_key_made_verifies() {
    let (key, other) = (new_key(), new_key());
    let host = signed_by(&key);
    let fake = FakeSigner::new();
    fake.hold(&SignerId::new("gate"));
    let theirs = fake.sign(&claim(), &SignerId::new("gate")).unwrap();
    let both = Envelope {
        signatures: [theirs.signatures, host.signatures.clone()].concat(),
        ..host.clone()
    };
    let signers = [listing("host", &key, None)];
    assert_eq!(verifier().verify(&both, &signers), made("host", false));

    let elsewhere = Envelope {
        signatures: [both.signatures.clone(), signed_by(&other).signatures].concat(),
        ..host.clone()
    };
    let said = none_of(
        verifier().verify(&elsewhere, &[listing("gate", &new_key(), None)]),
        &["gate"],
    );
    let fingerprint = |k: &PrivateKey| k.public_key().fingerprint(HashAlg::Sha256).to_string();
    assert!(
        said.starts_with("the signature is not an SSH signature; signed by the key "),
        "{said}"
    );
    assert!(said.contains(&fingerprint(&key)), "{said}");
    assert!(said.contains(&fingerprint(&other)), "{said}");

    let none = Envelope {
        signatures: vec![],
        ..host
    };
    let said = none_of(verifier().verify(&none, &signers), &["host"]);
    assert_eq!(said, "the envelope holds no signature");
}

#[test]
fn a_signer_verified_by_a_keys_name_is_passed_over_and_checked() {
    let key = new_key();
    let by_name = NamedSigner {
        id: SignerId::new("landing"),
        verified_by: "a key's name".into(),
        kinds: vec![ClaimKind::Verdict],
        checks: vec![],
    };
    let signers = [by_name.clone(), listing("host", &key, None)];
    assert_eq!(
        verifier().verify(&signed_by(&key), &signers),
        made("host", false)
    );
    none_of(
        verifier().verify(&signed_by(&new_key()), &signers),
        &["landing", "host"],
    );
}

#[test]
fn a_listing_past_its_valid_before_is_told_as_expired() {
    let key = new_key();
    let envelope = signed_by(&key);
    let signers = [listing("host", &key, Some("20000101Z"))];
    assert_eq!(verifier().verify(&envelope, &signers), made("host", true));
}

/// The date `days` after 1970-01-01, as `YYYYMMDD`.
fn date(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let of_era = z - era * 146_097;
    let year_of_era = (of_era - of_era / 1_460 + of_era / 36_524 - of_era / 146_096) / 365;
    let of_year = of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted = (5 * of_year + 2) / 153;
    let day = of_year - (153 * shifted + 2) / 5 + 1;
    let month = if shifted < 10 {
        shifted + 3
    } else {
        shifted - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}{month:02}{day:02}")
}

#[test]
fn a_listing_still_standing_is_not_expired() {
    assert_eq!(date(0), "19700101");
    assert_eq!(date(10_957), "20000101");
    assert_eq!(date(19_782), "20240229");
    let ahead = format!("{}Z", date(now() / 86_400 + 365));
    let key = new_key();
    let signers = [listing("host", &key, Some(&ahead))];
    assert_eq!(
        verifier().verify(&signed_by(&key), &signers),
        made("host", false)
    );
}

#[test]
fn a_listing_that_cannot_be_read_makes_verify_unreadable_naming_its_signer() {
    let key = new_key();
    let envelope = signed_by(&key);
    for (verified_by, words) in [
        (
            "ssh-ed25519 notakey".to_string(),
            ssh_key::PublicKey::from_openssh("ssh-ed25519 notakey")
                .unwrap_err()
                .to_string(),
        ),
        (
            format!(
                "valid-before=\"20991231\" {}",
                key.public_key().to_openssh().unwrap()
            ),
            "the valid-before time 20991231 must end in Z, so every host reads it in UTC".into(),
        ),
    ] {
        let signers = [
            listing("host", &key, None),
            NamedSigner {
                id: SignerId::new("gate"),
                verified_by,
                kinds: vec![ClaimKind::Acceptance],
                checks: vec![],
            },
        ];
        match verifier().verify(&envelope, &signers) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, HostKey::source());
                assert_eq!(u.what, "the key the policy lists for the signer gate");
                assert_eq!(u.said.as_deref(), Some(words.as_str()));
            }
            other => panic!("answered {other:?}, not unreadable"),
        }
    }
}

#[test]
fn s5_a_verdict_from_another_host_is_refused_naming_its_key() {
    let dir = TempDir::new().unwrap();
    let (first, second) = (new_key(), new_key());
    let other = HostKey::new(
        SignerId::new("another host"),
        written(dir.path(), "another", &second),
    );
    let verdict = Claim {
        kind: ClaimKind::Verdict,
        body: json!({ "check": "test", "outcome": "pass" }),
        ..claim()
    };
    let envelope = other
        .sign(&verdict, &SignerId::new("another host"))
        .unwrap();
    let said = none_of(
        verifier().verify(&envelope, &[listing("host", &first, None)]),
        &["host"],
    );
    let fingerprint = second.public_key().fingerprint(HashAlg::Sha256).to_string();
    assert!(fingerprint.starts_with("SHA256:"), "{fingerprint}");
    assert_eq!(
        said,
        format!(
            "signed by the key {fingerprint}, which the envelope names another host; \
             no signer checked lists that key with a signature over this envelope"
        )
    );
}

// Requirement: The profile can be bound to verify only

#[test]
fn a_host_with_no_key_tells_the_laptops_envelope_by_the_listed_key() {
    let dir = TempDir::new().unwrap();
    let key = new_key();
    let laptop = HostKey::new(SignerId::new("laptop"), written(dir.path(), "laptop", &key));
    let envelope = laptop.sign(&claim(), &SignerId::new("laptop")).unwrap();
    let signers = [
        listing("gate", &new_key(), None),
        listing("laptop", &key, None),
    ];
    assert_eq!(
        HostKey::verifying().verify(&envelope, &signers),
        made("laptop", false)
    );
}

#[test]
fn a_host_with_no_key_signs_nothing_and_reads_no_file() {
    // HOME an empty directory for this test alone, so a profile that
    // looked for a key under it would read nothing there and say so.
    let home = TempDir::new().unwrap();
    let before = std::env::var_os("HOME");
    std::env::set_var("HOME", home.path());
    let answers = [
        HostKey::verifying().sign(&claim(), &SignerId::new("laptop")),
        HostKey::verifying().sign(&claim(), &SignerId::new("host")),
    ];
    match before {
        Some(home) => std::env::set_var("HOME", home),
        None => std::env::remove_var("HOME"),
    }
    for (answer, identity) in answers.into_iter().zip(["laptop", "host"]) {
        assert_eq!(
            absent(answer),
            format!("identity {identity}: no identity is at hand, since this host binds no key")
        );
    }
    assert_eq!(fs::read_dir(home.path()).unwrap().count(), 0);
}
