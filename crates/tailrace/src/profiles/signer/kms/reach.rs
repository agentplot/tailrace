//! How the profile reaches KMS (design D5, D6): the host's own AWS
//! chain, or for a test what it is given. Every call runs on a runtime
//! made for it, with the credentials resolved once at its start, and
//! drops the runtime, the client and the credentials before it answers,
//! so nothing outlives a call (64, rule core.21).

use std::error::Error;
use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;

use aws_config::retry::RetryConfig;
use aws_config::timeout::TimeoutConfig;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_kms::config::{ProvideCredentials, SharedCredentialsProvider, SharedHttpClient};
use aws_sdk_kms::error::{ProvideErrorMetadata, SdkError};
use aws_sdk_kms::Client;
use aws_smithy_http_client::tls::{self, rustls_provider::CryptoMode};
use tokio::runtime::Runtime;

/// How long a connection to KMS may take before the attempt fails, so a
/// service that cannot be reached is answered within seconds.
const CONNECT: Duration = Duration::from_secs(3);

/// How the profile reaches KMS.
#[derive(Clone, Debug, Default)]
pub enum Reach {
    /// The host's own: the AWS configuration and credentials the AWS
    /// tools find, the environment, the AWS profile and its SSO session,
    /// web identity, the container's or the instance's role, loaded at
    /// each call in the key's region.
    #[default]
    Host,
    /// What it is given, reading no AWS file: the credentials, the HTTP
    /// client, and an endpoint in place of the region's where one is
    /// given.
    Given {
        credentials: SharedCredentialsProvider,
        http: SharedHttpClient,
        endpoint: Option<String>,
    },
}

/// The HTTPS client the profile reaches KMS through: rustls on ring, the
/// crypto `ureq` already uses, so the build needs no other (design D6).
pub fn https() -> SharedHttpClient {
    aws_smithy_http_client::Builder::new()
        .tls_provider(tls::Provider::Rustls(CryptoMode::Ring))
        .build_https()
}

/// Why a call to KMS answered nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Failure {
    /// No credentials were found, in the SDK's words.
    NoCredentials(String),
    /// KMS answered an error of its own: its code, and its words.
    Kms { code: String, words: String },
    /// KMS could not be reached, or its answer read, in the SDK's words.
    Unreached(String),
}

impl Failure {
    /// What the SDK answered for a call that failed.
    pub(super) fn of<E, R>(error: SdkError<E, R>) -> Failure
    where
        E: ProvideErrorMetadata + Error + 'static,
        R: Debug + 'static,
    {
        match (&error, error.code()) {
            (SdkError::ServiceError(_), Some(code)) => Failure::Kms {
                code: code.to_string(),
                words: match error.message() {
                    Some(message) => format!("{code}: {message}"),
                    None => code.to_string(),
                },
            },
            _ => Failure::Unreached(words(&error)),
        }
    }
}

/// One call's reach: the runtime made for it, and a KMS client's
/// configuration holding the credentials it resolved, dropped when the
/// call answers.
pub(super) struct Session {
    runtime: Runtime,
    config: aws_sdk_kms::config::Builder,
}

impl Reach {
    /// A session for one call, in `region` for the host's chain: a
    /// current-thread runtime made now and the credentials resolved
    /// once, `Failure::NoCredentials` where none are found.
    pub(super) fn open(&self, region: &str) -> Result<Session, Failure> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| Failure::Unreached(words(&e)))?;
        let timeouts = TimeoutConfig::builder().connect_timeout(CONNECT).build();
        let config = match self {
            Reach::Host => {
                let loaded = runtime.block_on(
                    aws_config::defaults(BehaviorVersion::latest())
                        .region(Region::new(region.to_string()))
                        .http_client(https())
                        .timeout_config(timeouts)
                        .retry_config(RetryConfig::standard())
                        .load(),
                );
                let provider = loaded.credentials_provider().ok_or_else(|| {
                    Failure::NoCredentials("the AWS configuration names no credentials".into())
                })?;
                let credentials = runtime
                    .block_on(provider.provide_credentials())
                    .map_err(|e| Failure::NoCredentials(words(&e)))?;
                aws_sdk_kms::config::Builder::from(&loaded).credentials_provider(credentials)
            }
            Reach::Given {
                credentials,
                http,
                endpoint,
            } => {
                let credentials = runtime
                    .block_on(credentials.provide_credentials())
                    .map_err(|e| Failure::NoCredentials(words(&e)))?;
                let config = aws_sdk_kms::config::Builder::new()
                    .behavior_version(BehaviorVersion::latest())
                    .http_client(http.clone())
                    .credentials_provider(credentials)
                    .timeout_config(timeouts)
                    .retry_config(RetryConfig::standard());
                match endpoint {
                    Some(endpoint) => config.endpoint_url(endpoint),
                    None => config,
                }
            }
        };
        Ok(Session { runtime, config })
    }
}

impl Session {
    /// A KMS client for a key in `region`.
    pub(super) fn client(&self, region: &str) -> Client {
        Client::from_conf(
            self.config
                .clone()
                .region(Region::new(region.to_string()))
                .build(),
        )
    }

    /// `call` run to its answer on the session's runtime.
    pub(super) fn run<F: Future>(&self, call: F) -> F::Output {
        self.runtime.block_on(call)
    }
}

/// An error's words and its sources' after them, as the SDK chains them,
/// each once.
fn words(error: &(dyn Error + 'static)) -> String {
    let mut said: Vec<String> = Vec::new();
    let mut at = Some(error);
    while let Some(error) = at {
        let words = error.to_string();
        if said.last() != Some(&words) {
            said.push(words);
        }
        at = error.source();
    }
    said.join(": ")
}
