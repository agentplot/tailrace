//! Spec key-service-signer: the conformance suite's signer group over a
//! KMS key in a stand-in for KMS's API reached through the SDK, a fresh
//! stand-in for each scenario, and the profile's own scenarios beside it
//! (design D9). Nothing here opens a connection but to a loopback port
//! nothing listens on.

mod kms;

use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

use aws_sdk_kms::primitives::Blob;
use aws_sdk_kms::types::{MessageType, SigningAlgorithmSpec};
use kms::{
    free_port, listed, Bound, StandIn, Switch, ACCESS_KEY, NO_CREDENTIALS, REGION, SECRET_KEY,
    SESSION_TOKEN,
};
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature as EcdsaSignature, VerifyingKey};
use p256::pkcs8::DecodePublicKey;
use serde_json::json;
use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey};
use tailrace::profiles::signer::host_key::HostKey;
use tailrace::profiles::signer::kms::{https, KeyArn, Kms, Reach};
use tailrace_conformance::seams::signer::{self, SignerRig};
use tailrace_conformance::{run, Rig};
use tailrace_seams::{
    Answer, Claim, ClaimKind, Digest, Envelope, Made, NamedSigner, Not, Revision, Signer, SignerId,
    Why,
};
use tempfile::TempDir;

/// A claim about one digest.
fn claim(kind: ClaimKind, says: &str) -> Claim {
    Claim {
        subject: Digest::of(b"an index"),
        kind,
        revision: Revision::new("4f2a9c1"),
        build: Some(tailrace_seams::BuildId::new("b7e1")),
        body: json!({ "says": says }),
    }
}

fn provenance() -> Claim {
    claim(ClaimKind::Provenance, "published")
}

/// The stand-in, credentials for it, and a key and a second key in it.
struct World {
    stand_in: StandIn,
    switch: Switch,
    key: String,
    second: String,
}

impl World {
    fn new() -> World {
        let stand_in = StandIn::new();
        let key = stand_in.make_key();
        let second = stand_in.make_key();
        World {
            stand_in,
            switch: Switch::new(),
            key,
            second,
        }
    }

    /// The profile bound to the key at `arn`, reaching the stand-in.
    fn bound(&self, arn: &str) -> Kms {
        Kms::new(KeyArn::parse(arn).unwrap()).reaching(self.stand_in.reach(&self.switch))
    }

    /// The profile bound to no key, reaching the stand-in.
    fn verifying(&self) -> Kms {
        Kms::verifying().reaching(self.stand_in.reach(&self.switch))
    }

    /// A claim signed with the key at `arn`.
    fn signed(&self, arn: &str, claim: &Claim) -> Envelope {
        self.bound(arn)
            .sign(claim, &SignerId::new(arn))
            .unwrap_or_else(|not| panic!("sign as {arn} answered {not:?}"))
    }
}

/// The profile bound to a key at a loopback port nothing listens on, and
/// the words the SDK answers a call there with, its chain of errors.
fn dead(arn: &str) -> (Kms, String) {
    let (port, held) = free_port();
    drop(held);
    let endpoint = format!("http://127.0.0.1:{port}");
    let switch = Switch::new();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let client = aws_sdk_kms::Client::from_conf(
        aws_sdk_kms::Config::builder()
            .behavior_version(aws_sdk_kms::config::BehaviorVersion::latest())
            .region(aws_sdk_kms::config::Region::new(REGION))
            .credentials_provider(switch.provider())
            .http_client(https())
            .endpoint_url(&endpoint)
            .retry_config(aws_sdk_kms::config::retry::RetryConfig::disabled())
            .build(),
    );
    let failed = runtime
        .block_on(client.describe_key().key_id(arn).send())
        .expect_err("nothing listens on the port");
    let mut words = vec![failed.to_string()];
    let mut at = failed.source();
    while let Some(error) = at {
        words.push(error.to_string());
        at = error.source();
    }
    let kms = Kms::new(KeyArn::parse(arn).unwrap()).reaching(Reach::Given {
        credentials: switch.provider(),
        http: https(),
        endpoint: Some(endpoint),
    });
    (kms, words.join(": "))
}

/// The signer group's rig over a fresh stand-in: a key it signs as, a
/// second key for the stranger.
struct StandInRig {
    world: World,
    signer: Bound,
}

impl StandInRig {
    fn new() -> StandInRig {
        let world = World::new();
        let signer = Bound::new(world.bound(&world.key));
        StandInRig { world, signer }
    }
}

impl Rig for StandInRig {
    fn profile(&self) -> String {
        Kms::source().profile
    }
}

impl SignerRig for StandInRig {
    fn signer(&self) -> &dyn Signer {
        &self.signer
    }

    fn identity(&self) -> SignerId {
        SignerId::new(&self.world.key)
    }

    fn named(&self, identity: &SignerId) -> NamedSigner {
        listed(identity.as_str(), identity.as_str())
    }

    fn stranger(&self) -> NamedSigner {
        listed("a stranger", &self.world.second)
    }

    fn no_identity(&self) {
        self.world.switch.off();
    }

    fn expire(&self, identity: &SignerId) {
        self.world.stand_in.set_state(identity.as_str(), "Disabled");
    }

    fn unreachable(&self) -> String {
        let (kms, words) = dead(&self.world.key);
        self.signer.swap(kms);
        words
    }
}

#[test]
fn the_kms_key_keeps_the_signers_contract() {
    let report = run(signer::GROUP, &signer::scenarios(), &|| {
        Box::new(StandInRig::new())
    });
    report.assert();
    assert_eq!(report.profile, "KMS key");
    assert_eq!(report.ran, signer::scenarios().len());
    assert_eq!(report.not_applicable, [], "{report}");
}

#[test]
fn a_key_of_the_stand_in_signs_through_the_sdk_and_verifies_with_its_public_half() {
    let stand_in = StandIn::new();
    let arn = stand_in.make_key();
    let switch = Switch::new();
    let Reach::Given {
        credentials,
        http,
        endpoint,
    } = stand_in.reach(&switch)
    else {
        unreachable!("the stand-in's reach is given")
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let client = aws_sdk_kms::Client::from_conf(
        aws_sdk_kms::Config::builder()
            .behavior_version(aws_sdk_kms::config::BehaviorVersion::latest())
            .region(aws_sdk_kms::config::Region::new(REGION))
            .credentials_provider(credentials)
            .http_client(http)
            .endpoint_url(endpoint.unwrap())
            .build(),
    );
    let message = b"a claim's pre-authentication encoding";
    let signed = runtime
        .block_on(
            client
                .sign()
                .key_id(&arn)
                .message(Blob::new(message.to_vec()))
                .message_type(MessageType::Raw)
                .signing_algorithm(SigningAlgorithmSpec::EcdsaSha256)
                .send(),
        )
        .unwrap();
    assert_eq!(signed.key_id(), Some(arn.as_str()));
    let public = runtime
        .block_on(client.get_public_key().key_id(&arn).send())
        .unwrap();
    let der = public.public_key().unwrap().as_ref();
    assert_eq!(der, stand_in.public_der(&arn));
    let verifying = VerifyingKey::from_public_key_der(der).unwrap();
    let sig = EcdsaSignature::from_der(signed.signature().unwrap().as_ref()).unwrap();
    let low = sig
        .normalize_s()
        .expect("the stand-in answers S high, as KMS may");
    verifying
        .verify(message, &low)
        .expect("the signature verifies");
    assert_eq!(stand_in.requests(), 2);
}

/// That `answer` is absent from the signer, naming `what`, and its words.
fn absent<T: std::fmt::Debug>(answer: Answer<T>, what: &str) -> String {
    match answer {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source, Kms::source());
            assert!(a.what.contains(what), "{a:?} names no {what}");
            a.what
        }
        other => panic!("answered {other:?}, not absent"),
    }
}

/// That `answer` is unreadable from the signer, naming `what`, and its
/// words.
fn unreadable<T: std::fmt::Debug>(answer: Answer<T>, what: &str) -> String {
    match answer {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, Kms::source());
            assert_eq!(u.what, what);
            u.said.expect("the words it was read with")
        }
        other => panic!("answered {other:?}, not unreadable"),
    }
}

/// That `answer` is refused as made by none of `checked`, and its words.
fn no_named_signer(answer: Answer<Made>, checked: &[&str]) -> String {
    match answer {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source, Kms::source());
            assert_eq!(
                r.why,
                Why::NoNamedSigner {
                    checked: checked.iter().map(|c| SignerId::new(*c)).collect()
                }
            );
            r.said.expect("the words it was refused with")
        }
        other => panic!("answered {other:?}, not refused"),
    }
}

/// The words a policy's listing of `arn` for `id` is unreadable with in
/// a verify.
fn listing(arn: &str, id: &str) -> String {
    format!("the key {arn} the policy lists for the signer {id}")
}

#[test]
fn signed_as_the_key_the_envelope_names_its_arn_and_verifies_over_its_encoding() {
    let world = World::new();
    let envelope = world.signed(&world.key, &provenance());
    let [signature] = envelope.signatures.as_slice() else {
        panic!("{} signatures", envelope.signatures.len())
    };
    assert_eq!(signature.signer.as_str(), world.key);
    assert_eq!(signature.keyid, world.key);
    assert_eq!(Claim::from_envelope(&envelope), Some(provenance()));
    let verifying =
        VerifyingKey::from_public_key_der(&world.stand_in.public_der(&world.key)).unwrap();
    let sig = EcdsaSignature::from_der(&signature.sig).unwrap();
    let sig = sig.normalize_s().unwrap_or(sig);
    let encoding =
        tailrace::profiles::signer::host_key::pae(&envelope.payload_type, &envelope.payload);
    verifying
        .verify(&encoding, &sig)
        .expect("the signature is over the envelope's pre-authentication encoding");
}

#[test]
fn asked_to_sign_as_another_it_is_absent_and_reaches_nothing() {
    let world = World::new();
    absent(
        world
            .bound(&world.key)
            .sign(&provenance(), &SignerId::new("gate")),
        "identity gate",
    );
    assert_eq!(world.stand_in.requests(), 0);
}

#[test]
fn the_verify_only_profile_signs_as_nothing_and_reaches_nothing() {
    let world = World::new();
    for identity in ["gate", world.key.as_str()] {
        let what = absent(
            world
                .verifying()
                .sign(&provenance(), &SignerId::new(identity)),
            identity,
        );
        assert!(what.contains("no identity is at hand"), "{what}");
    }
    assert_eq!(world.stand_in.requests(), 0);
}

#[test]
fn the_envelope_holds_nothing_of_the_credentials() {
    let world = World::new();
    let envelope = world.signed(&world.key, &provenance());
    let mut bytes = serde_json::to_vec(&envelope).unwrap();
    bytes.extend_from_slice(&envelope.payload);
    for signature in &envelope.signatures {
        bytes.extend_from_slice(&signature.sig);
    }
    let text = String::from_utf8_lossy(&bytes);
    for secret in [ACCESS_KEY, SECRET_KEY, SESSION_TOKEN] {
        assert!(!text.contains(secret), "the envelope holds {secret}");
    }
}

#[test]
fn the_credentials_taken_away_after_a_signature_are_absent_in_the_sdks_words() {
    let world = World::new();
    let kms = world.bound(&world.key);
    world.signed(&world.key, &provenance());
    world.switch.off();
    let what = absent(
        kms.sign(&provenance(), &SignerId::new(&world.key)),
        &world.key,
    );
    assert!(what.contains(NO_CREDENTIALS), "{what}");
}

#[test]
fn a_role_the_key_does_not_let_sign_is_absent_carrying_kmss_message() {
    let world = World::new();
    let message = format!(
        "User: arn:aws:sts::111122223333:assumed-role/runner/i-0 is not authorized to perform: \
         kms:Sign on resource: {} because no identity-based policy allows the kms:Sign action",
        world.key
    );
    world
        .stand_in
        .tell("Sign", "AccessDeniedException", &message);
    let what = absent(
        world
            .bound(&world.key)
            .sign(&provenance(), &SignerId::new(&world.key)),
        &world.key,
    );
    assert!(what.contains(&message), "{what}");
}

#[test]
fn a_disabled_key_a_key_pending_deletion_and_no_key_are_absent_in_kmss_words() {
    let world = World::new();
    world.stand_in.set_state(&world.key, "Disabled");
    let what = absent(
        world
            .bound(&world.key)
            .sign(&provenance(), &SignerId::new(&world.key)),
        &world.key,
    );
    assert!(
        what.contains(&format!("DisabledException: {} is disabled.", world.key)),
        "{what}"
    );
    world.stand_in.set_state(&world.key, "PendingDeletion");
    let what = absent(
        world
            .bound(&world.key)
            .sign(&provenance(), &SignerId::new(&world.key)),
        &world.key,
    );
    assert!(what.contains("KMSInvalidStateException"), "{what}");
    let gone =
        format!("arn:aws:kms:{REGION}:111122223333:key/00000000-0000-0000-0000-000000000000");
    let what = absent(
        world
            .bound(&gone)
            .sign(&provenance(), &SignerId::new(&gone)),
        &gone,
    );
    assert!(what.contains("NotFoundException"), "{what}");
}

#[test]
fn kmss_own_failure_at_sign_is_unreadable_naming_the_key() {
    let world = World::new();
    world.stand_in.tell(
        "Sign",
        "KMSInternalException",
        "The service is unavailable.",
    );
    let said = unreadable(
        world
            .bound(&world.key)
            .sign(&provenance(), &SignerId::new(&world.key)),
        &format!("the key {}", world.key),
    );
    assert_eq!(said, "KMSInternalException: The service is unavailable.");
}

#[test]
fn a_key_of_the_wrong_kind_is_refused_naming_it() {
    let world = World::new();
    let rsa = world
        .stand_in
        .make_key_of("RSA_2048", "SIGN_VERIFY", "AWS_KMS");
    let imported = world
        .stand_in
        .make_key_of("ECC_NIST_P256", "SIGN_VERIFY", "EXTERNAL");
    let encrypting = world
        .stand_in
        .make_key_of("ECC_NIST_P256", "ENCRYPT_DECRYPT", "AWS_KMS");
    for (arn, says) in [
        (&rsa, "RSA_2048"),
        (&imported, "its key material was imported"),
        (&encrypting, "ENCRYPT_DECRYPT"),
    ] {
        let before = world.stand_in.requests();
        match world.bound(arn).sign(&provenance(), &SignerId::new(arn)) {
            Err(Not::Refused(r)) => {
                assert_eq!(r.source, Kms::source());
                assert_eq!(r.why, Why::Said);
                let said = r.said.unwrap();
                assert!(said.contains(arn.as_str()), "{said}");
                assert!(said.contains(says), "{said}");
            }
            other => panic!("{arn} answered {other:?}, not refused"),
        }
        assert_eq!(
            world.stand_in.requests(),
            before + 1,
            "{arn} was asked to sign"
        );
    }
}

#[test]
fn verified_on_a_host_bound_to_no_key() {
    let world = World::new();
    let envelope = world.signed(&world.key, &provenance());
    assert_eq!(
        world
            .verifying()
            .verify(&envelope, &[listed("fleet", &world.key)]),
        Ok(Made {
            signer: SignerId::new("fleet"),
            expired: false
        })
    );
}

#[test]
fn the_name_the_envelope_gives_decides_nothing() {
    let world = World::new();
    let mut envelope = world.signed(&world.key, &provenance());
    envelope.signatures[0].signer = SignerId::new("gate");
    envelope.signatures[0].keyid = "gate".into();
    let signers = [listed("gate", &world.second), listed("fleet", &world.key)];
    assert_eq!(
        world.verifying().verify(&envelope, &signers),
        Ok(Made {
            signer: SignerId::new("fleet"),
            expired: false
        })
    );
}

#[test]
fn a_payload_changed_after_signing_is_refused() {
    let world = World::new();
    let envelope = Envelope {
        payload: claim(ClaimKind::Provenance, "published twice").payload(),
        ..world.signed(&world.key, &provenance())
    };
    let said = no_named_signer(
        world
            .verifying()
            .verify(&envelope, &[listed("fleet", &world.key)]),
        &["fleet"],
    );
    assert!(said.contains(&world.key), "{said}");
}

#[test]
fn a_host_keys_envelope_is_refused_reaching_nothing() {
    let world = World::new();
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("host");
    let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
    key.write_openssh_file(&path, LineEnding::LF).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let envelope = HostKey::new(SignerId::new("laptop"), &path)
        .sign(&provenance(), &SignerId::new("laptop"))
        .unwrap();
    let signers = [
        listed("fleet", &world.key),
        listed("laptop", &key.public_key().to_openssh().unwrap()),
    ];
    let said = no_named_signer(
        world.verifying().verify(&envelope, &signers),
        &["fleet", "laptop"],
    );
    assert!(said.contains("not a DER-encoded ECDSA signature"), "{said}");
    assert_eq!(world.stand_in.requests(), 0);
    let said = no_named_signer(
        world
            .verifying()
            .verify(&world.signed(&world.key, &provenance()), &signers[1..]),
        &["laptop"],
    );
    assert!(
        said.contains("no signer checked is listed by a KMS key"),
        "{said}"
    );
    assert_eq!(
        world.stand_in.requests(),
        2,
        "a verify against signers no KMS key lists reached KMS; only the sign did"
    );
}

#[test]
fn a_verdict_from_a_key_no_signer_lists_is_refused_naming_the_key_it_names() {
    let world = World::new();
    let verdict = world.signed(&world.second, &claim(ClaimKind::Verdict, "checks passed"));
    let said = no_named_signer(
        world
            .verifying()
            .verify(&verdict, &[listed("fleet", &world.key)]),
        &["fleet"],
    );
    assert!(said.contains(&world.second), "{said}");
}

#[test]
fn a_disabled_key_is_told_expired_and_enabled_again_not() {
    let world = World::new();
    let envelope = world.signed(&world.key, &provenance());
    let signers = [listed("fleet", &world.key)];
    world.stand_in.set_state(&world.key, "Disabled");
    assert_eq!(
        world.verifying().verify(&envelope, &signers),
        Ok(Made {
            signer: SignerId::new("fleet"),
            expired: true
        })
    );
    world.stand_in.set_state(&world.key, "Enabled");
    assert_eq!(
        world.verifying().verify(&envelope, &signers),
        Ok(Made {
            signer: SignerId::new("fleet"),
            expired: false
        })
    );
}

#[test]
fn a_key_pending_deletion_is_unreadable_carrying_kmss_words() {
    let world = World::new();
    let envelope = world.signed(&world.key, &provenance());
    world.stand_in.set_state(&world.key, "PendingDeletion");
    let said = unreadable(
        world
            .verifying()
            .verify(&envelope, &[listed("fleet", &world.key)]),
        &listing(&world.key, "fleet"),
    );
    assert_eq!(
        said,
        format!(
            "KMSInvalidStateException: {} is pending deletion.",
            world.key
        )
    );
}

#[test]
fn a_key_the_host_may_not_read_is_unreadable_carrying_kmss_message() {
    let world = World::new();
    let envelope = world.signed(&world.key, &provenance());
    let message = "User: arn:aws:sts::444455556666:assumed-role/reader/laptop is not authorized \
                   to perform: kms:GetPublicKey";
    world
        .stand_in
        .tell("GetPublicKey", "AccessDeniedException", message);
    let said = unreadable(
        world
            .verifying()
            .verify(&envelope, &[listed("fleet", &world.key)]),
        &listing(&world.key, "fleet"),
    );
    assert_eq!(said, format!("AccessDeniedException: {message}"));
}

#[test]
fn a_listed_key_of_another_spec_is_unreadable_naming_it() {
    let world = World::new();
    let envelope = world.signed(&world.key, &provenance());
    let rsa = world
        .stand_in
        .make_key_of("RSA_2048", "SIGN_VERIFY", "AWS_KMS");
    let said = unreadable(
        world
            .verifying()
            .verify(&envelope, &[listed("fleet", &rsa)]),
        &listing(&rsa, "fleet"),
    );
    assert!(said.contains("RSA_2048"), "{said}");
}

#[test]
fn a_listing_naming_an_alias_is_unreadable_naming_the_signer() {
    let world = World::new();
    let envelope = world.signed(&world.key, &provenance());
    let alias = format!("arn:aws:kms:{REGION}:111122223333:alias/fleet");
    let said = unreadable(
        world
            .verifying()
            .verify(&envelope, &[listed("fleet", &alias)]),
        "the key the policy lists for the signer fleet",
    );
    assert!(said.contains("names the alias fleet"), "{said}");
}

#[test]
fn no_service_at_the_endpoint_is_unreadable_within_ten_seconds() {
    let world = World::new();
    let envelope = world.signed(&world.key, &provenance());
    let (kms, words) = dead(&world.key);
    let started = Instant::now();
    let said = unreadable(
        kms.sign(&provenance(), &SignerId::new(&world.key)),
        &format!("the key {}", world.key),
    );
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(said, words);
    let started = Instant::now();
    let said = unreadable(
        kms.verify(&envelope, &[listed("fleet", &world.key)]),
        &listing(&world.key, "fleet"),
    );
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(said, words);
}

#[test]
fn bound_by_an_alias_the_binding_is_unreadable_naming_the_value() {
    let alias = "arn:aws:kms:us-west-2:111122223333:alias/fleet";
    let file = std::path::Path::new("/config/tailrace/config.toml");
    let table: toml::Table = toml::from_str(&format!("kms = \"{alias}\"")).unwrap();
    match tailrace::profiles::signer::read(file, None, &table) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.what, format!("{}, [signing] kms", file.display()));
            let said = u.said.unwrap();
            assert!(said.starts_with(alias), "{said}");
            assert!(said.contains("a key is named by its key ARN"), "{said}");
        }
        other => panic!("{alias} bound as {other:?}, not unreadable"),
    }
}

#[test]
fn bound_by_its_key_arn_it_signs_as_the_arn_in_its_region() {
    let arn = "arn:aws:kms:us-west-2:111122223333:key/1234abcd-12ab-34cd-56ef-1234567890ab";
    let kms = Kms::new(KeyArn::parse(arn).unwrap());
    assert_eq!(kms.key().map(KeyArn::as_str), Some(arn));
    assert_eq!(kms.key().map(KeyArn::region), Some("us-west-2"));
}
