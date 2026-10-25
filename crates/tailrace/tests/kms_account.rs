//! Spec key-service-signer, its run against a test account: the
//! conformance suite's signer group over the KMS key the host names in
//! `env.TAILRACE_TOOLS_KMS_KEY` of its `devenv.local.nix`, with the key
//! `env.TAILRACE_TOOLS_KMS_STRANGER` names as the stranger, reached
//! through the host's own AWS chain (design D9). Each scenario's key is
//! enabled and waited on until it signs, an expired identity is the key
//! disabled, and the key is enabled again when the group ends, whether
//! it passed or not. It refuses to run, naming both settings, on a host
//! that names neither, rather than pass without running.

#![cfg(feature = "tools")]

mod kms;

use std::rc::Rc;
use std::time::{Duration, Instant};

use aws_sdk_kms::types::KeyState;
use kms::{dead, listed, Bound, Switch};
use serde_json::json;
use tailrace::profiles::signer::kms::{https, KeyArn, Kms, Reach};
use tailrace_conformance::seams::signer::{self, SignerRig};
use tailrace_conformance::{run, Rig};
use tailrace_seams::{Claim, ClaimKind, Digest, NamedSigner, Revision, Signer, SignerId};

/// The settings a host names its two keys by.
const KEY: &str = "TAILRACE_TOOLS_KMS_KEY";
const STRANGER: &str = "TAILRACE_TOOLS_KMS_STRANGER";

/// How long KMS is given to show a key's new state, and a key enabled to
/// sign.
const SETTLES: Duration = Duration::from_secs(90);

/// The two keys the host names; a panic naming both settings, and what
/// the session needs of them, where it names either not.
fn keys() -> (KeyArn, KeyArn) {
    let read = |setting: &str| {
        std::env::var(setting)
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    };
    let (Some(key), Some(stranger)) = (read(KEY), read(STRANGER)) else {
        panic!(
            "the KMS run names no keys: set env.{KEY} and env.{STRANGER} in devenv.local.nix \
             to the key ARNs of two asymmetric ECC_NIST_P256 SIGN_VERIFY keys in a test \
             account, and run with a live AWS session whose role may kms:Sign, \
             kms:GetPublicKey, kms:DescribeKey, kms:EnableKey and kms:DisableKey on both"
        )
    };
    let parse = |setting: &str, text: &str| {
        KeyArn::parse(text).unwrap_or_else(|not| panic!("env.{setting}: {}", not.words()))
    };
    (parse(KEY, &key), parse(STRANGER, &stranger))
}

/// The host's own reach to KMS for the rig's own calls: enabling and
/// disabling the key, and reading its state.
struct Account {
    runtime: tokio::runtime::Runtime,
    client: aws_sdk_kms::Client,
    credentials: aws_sdk_kms::config::SharedCredentialsProvider,
}

impl Account {
    fn new(key: &KeyArn) -> Account {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let config = runtime.block_on(
            aws_config::defaults(aws_config::BehaviorVersion::latest())
                .region(aws_config::Region::new(key.region().to_string()))
                .http_client(https())
                .load(),
        );
        let credentials = config
            .credentials_provider()
            .expect("the host's AWS chain names credentials");
        Account {
            client: aws_sdk_kms::Client::new(&config),
            runtime,
            credentials,
        }
    }

    fn state(&self, key: &KeyArn) -> Option<KeyState> {
        let described = self
            .runtime
            .block_on(self.client.describe_key().key_id(key.as_str()).send())
            .unwrap_or_else(|e| panic!("describe {key}: {e:?}"));
        described
            .key_metadata()
            .and_then(|m| m.key_state())
            .cloned()
    }

    /// The key put in `state`, enabled or disabled, and waited on until
    /// KMS says it is.
    fn put(&self, key: &KeyArn, state: KeyState) {
        let call = match state {
            KeyState::Enabled => self
                .runtime
                .block_on(self.client.enable_key().key_id(key.as_str()).send())
                .map(drop)
                .map_err(|e| format!("{e:?}")),
            _ => self
                .runtime
                .block_on(self.client.disable_key().key_id(key.as_str()).send())
                .map(drop)
                .map_err(|e| format!("{e:?}")),
        };
        call.unwrap_or_else(|e| panic!("put {key} {}: {e}", state.as_str()));
        let started = Instant::now();
        while self.state(key).as_ref() != Some(&state) {
            assert!(
                started.elapsed() < SETTLES,
                "{key} is not {} after {SETTLES:?}",
                state.as_str()
            );
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    /// The key enabled, and waited on until a sign with it passes, since
    /// KMS may answer a sign just after EnableKey as disabled.
    fn ready(&self, key: &KeyArn) {
        self.put(key, KeyState::Enabled);
        let started = Instant::now();
        let claim = Claim {
            subject: Digest::of(b"ready"),
            kind: ClaimKind::Provenance,
            revision: Revision::new("4f2a9c1"),
            build: None,
            body: json!({}),
        };
        while let Err(not) = Kms::new(key.clone()).sign(&claim, &SignerId::new(key.as_str())) {
            assert!(
                started.elapsed() < SETTLES,
                "{key} does not sign after {SETTLES:?}: {not}"
            );
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}

/// The key enabled again when the run ends, however it ends.
struct Restore {
    account: Rc<Account>,
    key: KeyArn,
}

impl Drop for Restore {
    fn drop(&mut self) {
        self.account.put(&self.key, KeyState::Enabled);
    }
}

/// The signer group's rig over the host's key, reached through the
/// host's chain.
struct AccountRig {
    account: Rc<Account>,
    key: KeyArn,
    stranger: KeyArn,
    signer: Bound,
}

impl AccountRig {
    fn new(account: Rc<Account>, key: &KeyArn, stranger: &KeyArn) -> AccountRig {
        account.ready(key);
        AccountRig {
            account,
            key: key.clone(),
            stranger: stranger.clone(),
            signer: Bound::new(Kms::new(key.clone())),
        }
    }
}

impl Rig for AccountRig {
    fn profile(&self) -> String {
        Kms::source().profile
    }
}

impl SignerRig for AccountRig {
    fn signer(&self) -> &dyn Signer {
        &self.signer
    }

    fn identity(&self) -> SignerId {
        SignerId::new(self.key.as_str())
    }

    fn named(&self, identity: &SignerId) -> NamedSigner {
        listed(identity.as_str(), identity.as_str())
    }

    fn stranger(&self) -> NamedSigner {
        listed("a stranger", self.stranger.as_str())
    }

    /// The host's chain switched off.
    fn no_identity(&self) {
        let switch = Switch::over(self.account.credentials.clone());
        switch.off();
        self.signer
            .swap(Kms::new(self.key.clone()).reaching(Reach::Given {
                credentials: switch.provider(),
                http: https(),
                endpoint: None,
            }));
    }

    fn expire(&self, identity: &SignerId) {
        assert_eq!(identity.as_str(), self.key.as_str());
        self.account.put(&self.key, KeyState::Disabled);
    }

    fn unreachable(&self) -> String {
        let (kms, words) = dead(self.key.as_str());
        self.signer.swap(kms);
        words
    }
}

#[test]
fn a_kms_key_in_the_test_account_keeps_the_signers_contract() {
    let (key, stranger) = keys();
    let account = Rc::new(Account::new(&key));
    let _restore = Restore {
        account: account.clone(),
        key: key.clone(),
    };
    let report = run(signer::GROUP, &signer::scenarios(), &|| {
        Box::new(AccountRig::new(account.clone(), &key, &stranger))
    });
    report.assert();
    assert_eq!(report.profile, "KMS key");
    assert_eq!(report.ran, signer::scenarios().len());
    assert_eq!(report.not_applicable, [], "{report}");
}
