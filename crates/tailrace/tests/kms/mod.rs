//! A stand-in for KMS's JSON API, reached through the SDK as its HTTP
//! client (design D9): keys by ARN, each a P-256 signing key with a
//! state, a spec, a usage and an origin; `Sign`, `GetPublicKey`,
//! `DescribeKey`, `EnableKey` and `DisableKey` answered as KMS's API
//! reference has them, errors as a `__type` and a `message`; an error
//! told to answer an operation; a count of the requests it was sent; and
//! credentials that can be switched off. Beside it, what both runs of
//! the signer group share: a signer the rig swaps for another, and a
//! policy's listing of a key.

#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::error::Error;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use aws_credential_types::provider::error::CredentialsError;
use aws_credential_types::provider::{future, ProvideCredentials};
use aws_sdk_kms::config::retry::RetryConfig;
use aws_sdk_kms::config::{
    BehaviorVersion, Credentials, Region, SharedCredentialsProvider, SharedHttpClient,
};
use aws_smithy_http_client::test_util::infallible_client_fn;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use p256::ecdsa::signature::hazmat::PrehashSigner;
use p256::ecdsa::signature::Signer as _;
use p256::ecdsa::{Signature, SigningKey};
use p256::pkcs8::EncodePublicKey;
use serde_json::{json, Value};
use tailrace::profiles::signer::kms::{https, KeyArn, Kms, Reach};
use tailrace_seams::{Answer, Claim, ClaimKind, Envelope, Made, NamedSigner, Signer, SignerId};

/// The account and region every key of the stand-in is in.
pub const ACCOUNT: &str = "111122223333";
pub const REGION: &str = "us-west-2";

/// The access key and session token the stand-in's credentials hold.
pub const ACCESS_KEY: &str = "AKIASTANDIN0EXAMPLE1";
pub const SECRET_KEY: &str = "stand-in/secret/K7MDENG/bPxRfiCYEXAMPLEKEY";
pub const SESSION_TOKEN: &str = "IQoJb3JpZ2luX2VjEStandInSessionToken0123456789";

/// What the SDK's default chain says when no provider in it finds
/// credentials, which the stand-in's switched-off credentials say too.
pub const NO_CREDENTIALS: &str = "no providers in chain provided credentials";

/// A key the stand-in holds.
struct Key {
    signing: SigningKey,
    state: &'static str,
    spec: &'static str,
    usage: &'static str,
    origin: &'static str,
}

/// An error an operation is told to answer: its code and its message.
#[derive(Clone)]
struct Told {
    code: String,
    message: String,
}

#[derive(Default)]
struct State {
    keys: BTreeMap<String, Key>,
    made: u32,
    requests: usize,
    told: BTreeMap<String, Told>,
}

/// The stand-in for KMS, its state shared with the HTTP client it
/// answers through.
#[derive(Clone, Default)]
pub struct StandIn {
    state: Arc<Mutex<State>>,
}

impl StandIn {
    pub fn new() -> StandIn {
        StandIn::default()
    }

    /// A new key's ARN: an enabled `ECC_NIST_P256` signing key whose key
    /// material KMS made.
    pub fn make_key(&self) -> String {
        self.make_key_of("ECC_NIST_P256", "SIGN_VERIFY", "AWS_KMS")
    }

    /// A new enabled key's ARN, of `spec` and `usage`, from `origin`.
    pub fn make_key_of(
        &self,
        spec: &'static str,
        usage: &'static str,
        origin: &'static str,
    ) -> String {
        let mut state = self.state.lock().unwrap();
        state.made += 1;
        let arn = format!(
            "arn:aws:kms:{REGION}:{ACCOUNT}:key/{:08x}-12ab-34cd-56ef-{:012x}",
            state.made, state.made
        );
        state.keys.insert(
            arn.clone(),
            Key {
                signing: SigningKey::random(&mut p256::elliptic_curve::rand_core::OsRng),
                state: "Enabled",
                spec,
                usage,
                origin,
            },
        );
        arn
    }

    /// The key at `arn` put in `state`, as the console or a call would:
    /// `Enabled`, `Disabled` or `PendingDeletion`.
    pub fn set_state(&self, arn: &str, state: &'static str) {
        self.state.lock().unwrap().keys.get_mut(arn).unwrap().state = state;
    }

    /// The key at `arn`'s state.
    pub fn state_of(&self, arn: &str) -> &'static str {
        self.state.lock().unwrap().keys[arn].state
    }

    /// The operation `operation` (`Sign`, `GetPublicKey`, ...) answered
    /// from now with the error `code` and `message`.
    pub fn tell(&self, operation: &str, code: &str, message: &str) {
        self.state.lock().unwrap().told.insert(
            operation.to_string(),
            Told {
                code: code.into(),
                message: message.into(),
            },
        );
    }

    /// How many requests the stand-in has been sent.
    pub fn requests(&self) -> usize {
        self.state.lock().unwrap().requests
    }

    /// The SDK's HTTP client the stand-in answers through, which opens no
    /// connection.
    pub fn http(&self) -> SharedHttpClient {
        let stand_in = self.clone();
        infallible_client_fn(move |request: http::Request<_>| {
            let operation = request
                .headers()
                .get("x-amz-target")
                .and_then(|t| t.to_str().ok())
                .and_then(|t| t.strip_prefix("TrentService."))
                .unwrap_or("")
                .to_string();
            let body: Value = request
                .body()
                .bytes()
                .and_then(|b| serde_json::from_slice(b).ok())
                .unwrap_or(Value::Null);
            let (status, answer) = stand_in.answer(&operation, &body);
            http::Response::builder()
                .status(status)
                .header("content-type", "application/x-amz-json-1.1")
                .body(answer.to_string())
                .unwrap()
        })
    }

    /// The profile's reach to the stand-in, with `switch`'s credentials.
    pub fn reach(&self, switch: &Switch) -> Reach {
        Reach::Given {
            credentials: switch.provider(),
            http: self.http(),
            endpoint: Some("https://kms.stand-in.invalid".into()),
        }
    }

    /// The key at `arn`'s public half, as DER `SubjectPublicKeyInfo`.
    pub fn public_der(&self, arn: &str) -> Vec<u8> {
        let state = self.state.lock().unwrap();
        state.keys[arn]
            .signing
            .verifying_key()
            .to_public_key_der()
            .unwrap()
            .into_vec()
    }

    fn answer(&self, operation: &str, body: &Value) -> (u16, Value) {
        let mut state = self.state.lock().unwrap();
        state.requests += 1;
        if let Some(told) = state.told.get(operation) {
            return error(&told.code, &told.message);
        }
        let arn = body["KeyId"].as_str().unwrap_or("").to_string();
        let Some(key) = state.keys.get_mut(&arn) else {
            if matches!(
                operation,
                "Sign" | "GetPublicKey" | "DescribeKey" | "EnableKey" | "DisableKey"
            ) {
                return error("NotFoundException", &format!("Key '{arn}' does not exist"));
            }
            return error(
                "UnknownOperationException",
                &format!("the stand-in answers no {operation}"),
            );
        };
        let pending = || {
            error(
                "KMSInvalidStateException",
                &format!("{arn} is pending deletion."),
            )
        };
        match operation {
            "DescribeKey" => (
                200,
                json!({"KeyMetadata": {
                    "AWSAccountId": ACCOUNT,
                    "KeyId": arn.rsplit('/').next().unwrap(),
                    "Arn": arn,
                    "CreationDate": 1_790_000_000.0,
                    "Enabled": key.state == "Enabled",
                    "Description": "",
                    "KeyUsage": key.usage,
                    "KeyState": key.state,
                    "Origin": key.origin,
                    "KeyManager": "CUSTOMER",
                    "CustomerMasterKeySpec": key.spec,
                    "KeySpec": key.spec,
                    "SigningAlgorithms": algorithms(key.spec),
                    "MultiRegion": false,
                }}),
            ),
            "GetPublicKey" if key.state == "PendingDeletion" => pending(),
            "GetPublicKey" => (
                200,
                json!({
                    "KeyId": arn,
                    "PublicKey": STANDARD.encode(
                        key.signing.verifying_key().to_public_key_der().unwrap().as_bytes()
                    ),
                    "CustomerMasterKeySpec": key.spec,
                    "KeySpec": key.spec,
                    "KeyUsage": key.usage,
                    "SigningAlgorithms": algorithms(key.spec),
                }),
            ),
            "Sign" if key.state == "Disabled" => {
                error("DisabledException", &format!("{arn} is disabled."))
            }
            "Sign" if key.state == "PendingDeletion" => pending(),
            "Sign" if key.usage != "SIGN_VERIFY" => error(
                "InvalidKeyUsageException",
                &format!(
                    "{arn} key usage is {} which is not valid for Sign.",
                    key.usage
                ),
            ),
            "Sign"
                if key.spec != "ECC_NIST_P256" || body["SigningAlgorithm"] != "ECDSA_SHA_256" =>
            {
                error(
                    "InvalidKeyUsageException",
                    &format!(
                        "Algorithm {} is incompatible with key spec {}.",
                        body["SigningAlgorithm"], key.spec
                    ),
                )
            }
            "Sign" => {
                let message = STANDARD
                    .decode(body["Message"].as_str().unwrap_or(""))
                    .unwrap_or_default();
                let sig: Signature = match body["MessageType"].as_str() {
                    Some("DIGEST") if message.len() == 32 => {
                        key.signing.sign_prehash(&message).unwrap()
                    }
                    Some("DIGEST") => {
                        return error(
                            "ValidationException",
                            "Digest is invalid length for algorithm ECDSA_SHA_256.",
                        )
                    }
                    _ => key.signing.sign(&message),
                };
                (
                    200,
                    json!({
                        "KeyId": arn,
                        "Signature": STANDARD.encode(high_s(&sig).to_der().as_bytes()),
                        "SigningAlgorithm": "ECDSA_SHA_256",
                    }),
                )
            }
            "EnableKey" | "DisableKey" if key.state == "PendingDeletion" => pending(),
            "EnableKey" => {
                key.state = "Enabled";
                (200, json!({}))
            }
            "DisableKey" => {
                key.state = "Disabled";
                (200, json!({}))
            }
            _ => error(
                "UnknownOperationException",
                &format!("the stand-in answers no {operation}"),
            ),
        }
    }
}

/// The signing algorithms KMS gives for a key spec.
fn algorithms(spec: &str) -> Value {
    match spec {
        "ECC_NIST_P256" => json!(["ECDSA_SHA_256"]),
        _ => json!(["RSASSA_PSS_SHA_256", "RSASSA_PKCS1_V1_5_SHA_256"]),
    }
}

/// An error as KMS answers one: a 500 for its own failures, else a 400,
/// with the `__type` and the `message`.
fn error(code: &str, message: &str) -> (u16, Value) {
    let status = match code {
        "KMSInternalException" | "KeyUnavailableException" | "DependencyTimeoutException" => 500,
        _ => 400,
    };
    (status, json!({"__type": code, "message": message}))
}

/// The signature with its S made high, as KMS may answer one: it does
/// not promise low-S, and a verifier that takes only low-S signatures
/// must normalize first.
fn high_s(sig: &Signature) -> Signature {
    let low = sig.normalize_s().unwrap_or(*sig);
    let (r, s) = low.split_scalars();
    Signature::from_scalars(r.to_bytes(), (-*s).to_bytes()).unwrap()
}

/// Credentials the stand-in's reach finds until they are switched off,
/// and finds none of after, in the SDK's chain's words.
#[derive(Clone, Debug)]
pub struct Switch {
    on: Arc<AtomicBool>,
    inner: SharedCredentialsProvider,
}

impl Switch {
    /// The stand-in's known access key and session token.
    pub fn new() -> Switch {
        Switch::over(SharedCredentialsProvider::new(Credentials::new(
            ACCESS_KEY,
            SECRET_KEY,
            Some(SESSION_TOKEN.into()),
            None,
            "stand-in",
        )))
    }

    /// The credentials `inner` finds, the host's chain among them.
    pub fn over(inner: SharedCredentialsProvider) -> Switch {
        Switch {
            on: Arc::new(AtomicBool::new(true)),
            inner,
        }
    }

    /// No credentials from now.
    pub fn off(&self) {
        self.on.store(false, Ordering::SeqCst);
    }

    pub fn provider(&self) -> SharedCredentialsProvider {
        SharedCredentialsProvider::new(self.clone())
    }
}

impl ProvideCredentials for Switch {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        if self.on.load(Ordering::SeqCst) {
            self.inner.provide_credentials()
        } else {
            future::ProvideCredentials::ready(Err(CredentialsError::not_loaded(NO_CREDENTIALS)))
        }
    }
}

/// The profile a rig hands the group, swapped for another when the rig
/// makes it unreachable or takes its credentials away.
pub struct Bound(RefCell<Kms>);

impl Bound {
    pub fn new(kms: Kms) -> Bound {
        Bound(RefCell::new(kms))
    }

    pub fn swap(&self, kms: Kms) {
        *self.0.borrow_mut() = kms;
    }
}

impl Signer for Bound {
    fn sign(&self, claim: &Claim, identity: &SignerId) -> Answer<Envelope> {
        self.0.borrow().sign(claim, identity)
    }

    fn verify(&self, attestation: &Envelope, signers: &[NamedSigner]) -> Answer<Made> {
        self.0.borrow().verify(attestation, signers)
    }
}

/// `id` as a policy lists it, verified by the key `verified_by` names,
/// for every kind of claim.
pub fn listed(id: &str, verified_by: &str) -> NamedSigner {
    NamedSigner {
        id: SignerId::new(id),
        verified_by: verified_by.into(),
        kinds: vec![
            ClaimKind::Provenance,
            ClaimKind::Verdict,
            ClaimKind::Test,
            ClaimKind::Review,
            ClaimKind::Rebuild,
            ClaimKind::Acceptance,
            ClaimKind::Promotion,
            ClaimKind::Carried,
        ],
        checks: vec![],
    }
}

/// The profile bound to the key at `arn`, reaching KMS at a loopback port
/// nothing listens on, and the words the SDK answers a call there with:
/// its chain of errors, read by a client of the SDK's own.
pub fn dead(arn: &str) -> (Kms, String) {
    let key = KeyArn::parse(arn).unwrap();
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
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(key.region().to_string()))
            .credentials_provider(switch.provider())
            .http_client(https())
            .endpoint_url(&endpoint)
            .retry_config(RetryConfig::disabled())
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
    let kms = Kms::new(key).reaching(Reach::Given {
        credentials: switch.provider(),
        http: https(),
        endpoint: Some(endpoint),
    });
    (kms, words.join(": "))
}

/// A port on the loopback interface nothing listens on once the listener
/// answered beside it is dropped.
pub fn free_port() -> (u16, TcpListener) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a free loopback port");
    (listener.local_addr().unwrap().port(), listener)
}
