//! Spec host-key-signer: sign and verify over keys made fresh in a
//! temporary directory, and the fixed texts of keys this profile refuses.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use serde_json::json;
use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey, SshSig};
use tailrace_seams::{
    Answer, Claim, ClaimKind, Digest, Envelope, Not, Revision, Seam, Signer, SignerId, Why, IN_TOTO,
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
    let pem = std::str::from_utf8(&envelope.signature.sig).expect("the signature is text");
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
    assert_eq!(host.signer(), &SignerId::new("host"));
    assert_eq!(host.key(), Path::new("/keys/host"));
}

#[test]
fn signed_as_the_host() {
    let dir = TempDir::new().unwrap();
    let key = new_key();
    let envelope = sign(&host(&written(dir.path(), "host", &key))).unwrap();
    assert_eq!(Claim::from_envelope(&envelope), Some(claim()));
    assert_eq!(envelope.payload_type, IN_TOTO);
    assert_eq!(envelope.payload, claim().payload());
    assert_eq!(envelope.signature.signer, SignerId::new("host"));
    let pem = std::str::from_utf8(&envelope.signature.sig).unwrap();
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
    for bytes in [&serialized, &envelope.payload, &envelope.signature.sig] {
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
    let HostKey { signer, key } = bound;
    assert_eq!(signer, SignerId::new("host"));
    assert_eq!(key, path);
}
