//! The request path (design D3, D4): a request sent as asked and its
//! answer read whole, a 4xx or 5xx answered as a response so the
//! registry's words reach the caller verbatim (59); a request the
//! registry challenges answered once with the host's Docker credential,
//! reached at that moment through the host's Docker configuration and
//! the helper it names, as `Basic` or as a `Bearer` token asked of the
//! challenge's realm for the scope it names. What authorizes the next
//! request to each repository is held in memory by the one handle and
//! written nowhere, so a token scoped to one repository is kept beside
//! another's (64, rule core.21; design D8).

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use base64::Engine as _;
use docker_credential::{CredentialRetrievalError, DockerCredential};
use serde::Deserialize;
use tailrace_seams::{Answer, Not, Source, Why};
use ureq::http::{HeaderMap, Method, Request, StatusCode};

/// What the registry answered: its status, headers and whole body.
#[derive(Debug)]
pub(super) struct Answered {
    pub status: StatusCode,
    headers: HeaderMap,
    pub body: Vec<u8>,
}

impl Answered {
    /// The header `name`, where the registry gave it as text.
    pub(super) fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }

    /// The registry's own words: its body, or its status line where the
    /// body is empty, as for a `HEAD`.
    pub(super) fn words(&self) -> String {
        if self.body.is_empty() {
            let reason = self.status.canonical_reason().unwrap_or("");
            format!("{} {reason}", self.status.as_u16())
                .trim_end()
                .to_string()
        } else {
            String::from_utf8_lossy(&self.body).into_owned()
        }
    }
}

/// A challenge a registry answers a request with (design D4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Challenge {
    Basic,
    Bearer {
        realm: String,
        service: Option<String>,
        scope: Option<String>,
    },
}

/// The challenge a `WWW-Authenticate` header holds: `Basic`, or `Bearer`
/// with its realm and the service and scope it names; none for another
/// scheme or a header that is not well formed.
pub(super) fn challenge(header: &str) -> Option<Challenge> {
    let header = header.trim();
    let (scheme, rest) = header.split_once(' ').unwrap_or((header, ""));
    let params = params(rest)?;
    let param = |key: &str| {
        params
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.clone())
    };
    if scheme.eq_ignore_ascii_case("basic") {
        Some(Challenge::Basic)
    } else if scheme.eq_ignore_ascii_case("bearer") {
        Some(Challenge::Bearer {
            realm: param("realm").filter(|r| !r.is_empty())?,
            service: param("service"),
            scope: param("scope"),
        })
    } else {
        None
    }
}

/// A challenge's parameters, `key=token` or `key="quoted"` separated by
/// commas, a comma inside quotes kept; none when they are not well
/// formed.
fn params(text: &str) -> Option<Vec<(String, String)>> {
    let mut params = Vec::new();
    let mut chars = text.chars().peekable();
    loop {
        while chars.peek().is_some_and(|c| *c == ',' || c.is_whitespace()) {
            chars.next();
        }
        if chars.peek().is_none() {
            return Some(params);
        }
        let key: String =
            std::iter::from_fn(|| chars.next_if(|c| *c != '=' && *c != ',')).collect();
        let key = key.trim().to_string();
        if key.is_empty() || key.contains(char::is_whitespace) || chars.next() != Some('=') {
            return None;
        }
        let mut value = String::new();
        if chars.peek() == Some(&'"') {
            chars.next();
            loop {
                match chars.next()? {
                    '"' => break,
                    '\\' => value.push(chars.next()?),
                    c => value.push(c),
                }
            }
        } else {
            value = std::iter::from_fn(|| chars.next_if(|c| *c != ',')).collect();
            value = value.trim().to_string();
        }
        params.push((key, value));
    }
}

/// A value written into a URL's query.
pub(super) fn encoded(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                char::from(b).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Whether `error` is the connection closing under a request, the far
/// side gone before the whole answer arrived.
fn closed(error: &ureq::Error) -> bool {
    use std::io::ErrorKind;
    matches!(error, ureq::Error::Io(e) if matches!(
        e.kind(),
        ErrorKind::UnexpectedEof
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::BrokenPipe
    ))
}

/// A token a realm answers (design D4).
#[derive(Deserialize)]
struct Token {
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    access_token: Option<String>,
}

/// The connection to one registry host: the agent, the host the
/// credential is the host's for, and what authorizes the next request
/// to each repository it has been answered for.
pub(super) struct Client {
    agent: ureq::Agent,
    host: String,
    source: Source,
    authorization: RefCell<BTreeMap<String, String>>,
}

impl Client {
    pub(super) fn new(host: &str, source: &Source) -> Client {
        let config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build();
        Client {
            agent: ureq::Agent::new_with_config(config),
            host: host.to_string(),
            source: source.clone(),
            authorization: RefCell::new(BTreeMap::new()),
        }
    }

    /// `method` of `url`, a request for `repository`, with `headers` and
    /// `body`, answered as the registry answered it; a challenge answered
    /// once with the host's credential and the request sent once more,
    /// what authorized it kept under `repository` as the caller names it,
    /// never as read back from `url`, whose path a repository's own may
    /// hold an operation's word in (design D8). Unreadable naming `url` in
    /// the client's words where no connection is made or the answer cannot
    /// be read.
    pub(super) fn send(
        &self,
        repository: &str,
        method: Method,
        url: &str,
        headers: &[(&str, &str)],
        body: Option<&[u8]>,
    ) -> Answer<Answered> {
        let held = self.authorization.borrow().get(repository).cloned();
        let first = self.once(&method, url, headers, body, held.as_deref())?;
        if first.status != StatusCode::UNAUTHORIZED {
            return Ok(first);
        }
        let Some(challenge) = first.header("www-authenticate").and_then(challenge) else {
            return Ok(first);
        };
        let Some(authorization) = self.authorize(&challenge)? else {
            return Ok(first);
        };
        self.authorization
            .borrow_mut()
            .insert(repository.to_string(), authorization.clone());
        self.once(&method, url, headers, body, Some(&authorization))
    }

    /// `method` of `url` sent and its answer read whole. A read, `GET` or
    /// `HEAD`, whose connection closed before its whole answer arrived,
    /// as a kept connection the far side closed while it idled does, is
    /// sent once more on a new connection, the redirect it follows too; a
    /// write is never sent again. Unreadable naming `url` in the client's
    /// words otherwise.
    fn once(
        &self,
        method: &Method,
        url: &str,
        headers: &[(&str, &str)],
        body: Option<&[u8]>,
        authorization: Option<&str>,
    ) -> Answer<Answered> {
        let read = [Method::GET, Method::HEAD].contains(method);
        match self.exchange(method, url, headers, body, authorization, false) {
            Err(e) if read && closed(&e) => {
                self.exchange(method, url, headers, body, authorization, true)
            }
            answered => answered,
        }
        .map_err(|e| Not::unreadable(&self.source, url, Some(e.to_string())))
    }

    /// `method` of `url` sent once and its answer read whole, on a new
    /// connection where `fresh` rather than one the agent keeps.
    fn exchange(
        &self,
        method: &Method,
        url: &str,
        headers: &[(&str, &str)],
        body: Option<&[u8]>,
        authorization: Option<&str>,
        fresh: bool,
    ) -> Result<Answered, ureq::Error> {
        let mut request = Request::builder().method(method.clone()).uri(url);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        if let Some(authorization) = authorization {
            request = request.header("authorization", authorization);
        }
        let mut response = match body {
            Some(body) => self.agent.run(self.connected(request.body(body)?, fresh))?,
            None => self.agent.run(self.connected(request.body(())?, fresh))?,
        };
        let body = response
            .body_mut()
            .with_config()
            .limit(u64::MAX)
            .read_to_vec()?;
        Ok(Answered {
            status: response.status(),
            headers: response.headers().clone(),
            body,
        })
    }

    /// `request`, sent on a new connection where `fresh`: no kept
    /// connection is young enough for it.
    fn connected<S: ureq::AsSendBody>(&self, request: Request<S>, fresh: bool) -> Request<S> {
        if fresh {
            self.agent
                .configure_request(request)
                .max_idle_age(Duration::ZERO)
                .build()
        } else {
            request
        }
    }

    /// What authorizes a request `challenge` was answered to: the host's
    /// credential as `Basic`, or a token the realm answers for it; none
    /// where the host holds no credential for the registry, so the
    /// challenge is the answer.
    fn authorize(&self, challenge: &Challenge) -> Answer<Option<String>> {
        let credential = self.credential()?;
        match challenge {
            Challenge::Basic => Ok(match credential {
                Some(DockerCredential::UsernamePassword(user, password)) => Some(format!(
                    "Basic {}",
                    base64::engine::general_purpose::STANDARD.encode(format!("{user}:{password}"))
                )),
                Some(DockerCredential::IdentityToken(_)) | None => None,
            }),
            Challenge::Bearer {
                realm,
                service,
                scope,
            } => {
                let query: Vec<String> = [("service", service), ("scope", scope)]
                    .into_iter()
                    .filter_map(|(k, v)| v.as_ref().map(|v| format!("{k}={}", encoded(v))))
                    .collect();
                let asked = match credential {
                    Some(DockerCredential::IdentityToken(token)) => {
                        let mut form = query.clone();
                        form.push("grant_type=refresh_token".into());
                        form.push("client_id=tailrace".into());
                        form.push(format!("refresh_token={}", encoded(&token)));
                        self.once(
                            &Method::POST,
                            realm,
                            &[("content-type", "application/x-www-form-urlencoded")],
                            Some(form.join("&").as_bytes()),
                            None,
                        )?
                    }
                    credential => {
                        let basic = match credential {
                            Some(DockerCredential::UsernamePassword(user, password)) => {
                                Some(format!(
                                    "Basic {}",
                                    base64::engine::general_purpose::STANDARD
                                        .encode(format!("{user}:{password}"))
                                ))
                            }
                            _ => None,
                        };
                        let sep = if realm.contains('?') { '&' } else { '?' };
                        let url = if query.is_empty() {
                            realm.clone()
                        } else {
                            format!("{realm}{sep}{}", query.join("&"))
                        };
                        self.once(&Method::GET, &url, &[], None, basic.as_deref())?
                    }
                };
                if !asked.status.is_success() {
                    return Err(Not::refused(&self.source, Why::Said, Some(asked.words())));
                }
                let token: Token = serde_json::from_slice(&asked.body)
                    .map_err(|e| Not::unreadable(&self.source, realm, Some(e.to_string())))?;
                match token.token.or(token.access_token) {
                    Some(token) if !token.is_empty() => Ok(Some(format!("Bearer {token}"))),
                    _ => Err(Not::unreadable(
                        &self.source,
                        realm,
                        Some(format!("{realm} answered no token: {}", asked.words())),
                    )),
                }
            }
        }
    }

    /// The host's credential for the registry, from its Docker
    /// configuration and the helper it names, read now and kept nowhere;
    /// none where the configuration names none for the registry. A
    /// configuration or helper that cannot be read is unreadable naming
    /// the credential, in the reader's words.
    fn credential(&self) -> Answer<Option<DockerCredential>> {
        let named = std::env::var_os("DOCKER_CONFIG").filter(|d| !d.is_empty());
        // A test that read the host's configuration would ask the helper
        // it names, the keychain among them, which no gate check may
        // (rule core.38): each names a configuration of its own.
        #[cfg(test)]
        assert!(
            named.is_some(),
            "a test reached the host's Docker configuration; run it under_docker_config"
        );
        let file = named
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".docker")))
            .map(|dir| dir.join("config.json"));
        if !file.as_ref().is_some_and(|f| f.exists()) {
            return Ok(None);
        }
        match docker_credential::get_credential(&self.host) {
            Ok(credential) => Ok(Some(credential)),
            Err(CredentialRetrievalError::NoCredentialConfigured) => Ok(None),
            Err(CredentialRetrievalError::HelperFailure { stdout, stderr, .. })
                if format!("{stdout}{stderr}").contains("credentials not found") =>
            {
                Ok(None)
            }
            Err(e) => Err(Not::unreadable(
                &self.source,
                format!("the credential for {}", self.host),
                Some(e.to_string()),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_basic_challenge_is_read() {
        assert_eq!(
            challenge(r#"Basic realm="Registry Realm""#),
            Some(Challenge::Basic)
        );
        assert_eq!(challenge("basic"), Some(Challenge::Basic));
    }

    #[test]
    fn a_bearer_challenge_names_its_realm_service_and_scope() {
        let header = r#"Bearer realm="https://auth.example.com/token",service="registry.example.com",scope="repository:acme/tailrace:pull,push""#;
        assert_eq!(
            challenge(header),
            Some(Challenge::Bearer {
                realm: "https://auth.example.com/token".into(),
                service: Some("registry.example.com".into()),
                scope: Some("repository:acme/tailrace:pull,push".into()),
            })
        );
        assert_eq!(
            challenge(r#"Bearer realm=https://auth.example.com/token, scope="a\"b""#),
            Some(Challenge::Bearer {
                realm: "https://auth.example.com/token".into(),
                service: None,
                scope: Some("a\"b".into()),
            })
        );
    }

    #[test]
    fn a_malformed_challenge_is_none() {
        for header in [
            r#"Bearer realm="https://auth.example.com/token"#,
            r#"Bearer service="registry.example.com""#,
            "Bearer realm",
            r#"Negotiate realm="x""#,
            "",
        ] {
            assert_eq!(challenge(header), None, "{header}");
        }
    }

    #[test]
    fn a_token_for_one_repository_is_kept_beside_anothers() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::{Arc, OnceLock};
        let at: Arc<OnceLock<u16>> = Arc::new(OnceLock::new());
        let asked = Arc::new(AtomicUsize::new(0));
        let (port_of, count) = (at.clone(), asked.clone());
        let port = stub(7, move |head| {
            count.fetch_add(1, Ordering::SeqCst);
            let first = head.lines().next().unwrap_or_default().to_string();
            let reply = |status: &str, headers: &str, body: &str| {
                format!(
                    "HTTP/1.1 {status}\r\n{headers}content-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                )
            };
            if first.contains("/token") {
                let scope = if first.contains("one") { "one" } else { "two" };
                return reply("200 OK", "", &format!(r#"{{"token":"{scope}"}}"#));
            }
            let repository = if first.contains("/v2/one/tags/") {
                "one"
            } else {
                "two"
            };
            let lower = head.to_lowercase();
            if lower.contains(&format!("authorization: bearer {repository}")) {
                return reply("200 OK", "", "{}");
            }
            let port = port_of.get().copied().unwrap();
            reply(
                "401 Unauthorized",
                &format!(
                    "www-authenticate: Bearer realm=\"http://127.0.0.1:{port}/token\",scope=\"repository:{repository}:pull\"\r\n"
                ),
                "",
            )
        });
        at.set(port).unwrap();
        let source = Source::new(tailrace_seams::Seam::Store, "OCI registry stub");
        let client = Client::new(&format!("127.0.0.1:{port}"), &source);
        let none = tempfile::TempDir::new().unwrap();
        let get = |repository: &str| {
            let url = format!("http://127.0.0.1:{port}/v2/{repository}/manifests/x");
            under_docker_config(none.path(), || {
                client.send(repository, Method::GET, &url, &[], None)
            })
            .unwrap()
            .status
        };
        assert_eq!(get("one/tags"), StatusCode::OK);
        assert_eq!(get("two"), StatusCode::OK);
        assert_eq!(asked.load(Ordering::SeqCst), 6);
        assert_eq!(get("one/tags"), StatusCode::OK);
        assert_eq!(
            asked.load(Ordering::SeqCst),
            7,
            "one's token was thrown away by the request to two"
        );
        let held = client.authorization.borrow().clone();
        assert_eq!(
            held.into_iter().collect::<Vec<_>>(),
            [
                ("one/tags".to_string(), "Bearer one".to_string()),
                ("two".to_string(), "Bearer two".to_string())
            ]
        );
    }

    #[test]
    fn a_query_value_is_encoded() {
        assert_eq!(
            encoded("repository:acme/tailrace:pull,push"),
            "repository%3Aacme%2Ftailrace%3Apull%2Cpush"
        );
        assert_eq!(encoded("sha256:00ff"), "sha256%3A00ff");
    }

    #[test]
    fn the_registrys_words_are_its_body_or_its_status() {
        let answered = |status: u16, body: &[u8]| Answered {
            status: StatusCode::from_u16(status).unwrap(),
            headers: HeaderMap::new(),
            body: body.to_vec(),
        };
        let words =
            "{\"errors\":[{\"code\":\"DENIED\",\"message\":\"tag \\\"v1\\\" is immutable\"}]}\n";
        assert_eq!(answered(400, words.as_bytes()).words(), words);
        assert_eq!(answered(401, b"").words(), "401 Unauthorized");
    }

    /// What `f` answers with DOCKER_CONFIG at `dir`, the client reading
    /// that configuration alone and never the host's or the helper it
    /// names (rule core.38); one test at a time, since the variable is the
    /// process's, and the variable as it was after.
    fn under_docker_config<T>(dir: &std::path::Path, f: impl FnOnce() -> T) -> T {
        static ENVIRONMENT: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _environment = ENVIRONMENT.lock().unwrap_or_else(|e| e.into_inner());
        let before = std::env::var_os("DOCKER_CONFIG");
        std::env::set_var("DOCKER_CONFIG", dir);
        let answer = f();
        match before {
            Some(before) => std::env::set_var("DOCKER_CONFIG", before),
            None => std::env::remove_var("DOCKER_CONFIG"),
        }
        answer
    }

    /// A server on a loopback port answering each request in turn with
    /// what `answer` makes of its head, closing each connection after.
    fn stub(answers: usize, answer: impl Fn(&str) -> String + Send + 'static) -> u16 {
        use std::io::{BufRead, BufReader, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for stream in listener.incoming().take(answers) {
                let mut stream = stream.unwrap();
                let mut head = String::new();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                    head.push_str(&line);
                }
                stream.write_all(answer(&head).as_bytes()).unwrap();
            }
        });
        port
    }

    #[test]
    fn a_basic_challenge_is_answered_once_from_the_docker_configuration_and_nothing_is_written() {
        let basic = base64::engine::general_purpose::STANDARD.encode("tester:secret");
        let expected = format!("authorization: Basic {basic}");
        let port = stub(2, move |head| {
            let body = if head.to_lowercase().contains(&expected.to_lowercase()) {
                "{}"
            } else {
                return "HTTP/1.1 401 Unauthorized\r\nwww-authenticate: Basic realm=\"stub\"\r\n\
                        content-length: 12\r\nconnection: close\r\n\r\nunauthorized"
                    .into();
            };
            format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            )
        });
        let dir = tempfile::TempDir::new().unwrap();
        let config = format!(r#"{{"auths":{{"127.0.0.1:{port}":{{"auth":"{basic}"}}}}}}"#);
        std::fs::write(dir.path().join("config.json"), &config).unwrap();

        let source = Source::new(tailrace_seams::Seam::Store, "OCI registry stub");
        let client = Client::new(&format!("127.0.0.1:{port}"), &source);
        let answered = under_docker_config(dir.path(), || {
            client.send(
                "",
                Method::GET,
                &format!("http://127.0.0.1:{port}/v2/"),
                &[],
                None,
            )
        })
        .unwrap();
        assert_eq!(answered.status, StatusCode::OK, "{}", answered.words());
        assert_eq!(answered.body, b"{}");
        assert_eq!(
            client.authorization.borrow().get("").map(String::as_str),
            Some(format!("Basic {basic}").as_str())
        );
        let files: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(files, ["config.json"]);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("config.json")).unwrap(),
            config
        );
    }

    /// A server on a loopback port that keeps each connection open after
    /// an answer, answering each request with the last segment of its
    /// path, and closing a connection without an answer at a request
    /// `drops` names by the connection's number, the request's number on
    /// it and its line, as a far side that closed a kept connection while
    /// the request was on its way does. Every request is kept, with the
    /// number of the connection it came on.
    #[allow(clippy::type_complexity)]
    fn kept(
        drops: impl Fn(usize, usize, &str) -> bool + Send + Sync + 'static,
    ) -> (u16, std::sync::Arc<std::sync::Mutex<Vec<(usize, String)>>>) {
        use std::io::{BufRead, BufReader, Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let asked = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let kept = asked.clone();
        let drops = std::sync::Arc::new(drops);
        std::thread::spawn(move || {
            for (connection, stream) in listener.incoming().enumerate() {
                let (asked, drops) = (kept.clone(), drops.clone());
                std::thread::spawn(move || {
                    let mut stream = stream.unwrap();
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    for at in 0.. {
                        let mut line = String::new();
                        if reader.read_line(&mut line).unwrap_or(0) == 0 {
                            return;
                        }
                        let line = line.trim_end().to_string();
                        let mut length = 0;
                        loop {
                            let mut header = String::new();
                            reader.read_line(&mut header).unwrap();
                            if header == "\r\n" || header.is_empty() {
                                break;
                            }
                            if let Some((name, value)) = header.split_once(':') {
                                if name.eq_ignore_ascii_case("content-length") {
                                    length = value.trim().parse().unwrap();
                                }
                            }
                        }
                        reader.read_exact(&mut vec![0; length]).unwrap();
                        asked.lock().unwrap().push((connection + 1, line.clone()));
                        if drops(connection + 1, at, &line) {
                            return;
                        }
                        let path = line.split(' ').nth(1).unwrap_or_default();
                        let body = path.rsplit('/').next().unwrap_or_default();
                        let head = if line.starts_with("HEAD ") { "" } else { body };
                        let answer = format!(
                            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{head}",
                            body.len()
                        );
                        stream.write_all(answer.as_bytes()).unwrap();
                    }
                });
            }
        });
        (port, asked)
    }

    #[test]
    fn a_read_on_a_kept_connection_the_far_side_closed_is_sent_once_more_on_a_new_one() {
        let (port, asked) = kept(|_, at, _| at > 0);
        let source = Source::new(tailrace_seams::Seam::Store, "OCI registry stub");
        let client = Client::new(&format!("127.0.0.1:{port}"), &source);
        let url = |blob: &str| format!("http://127.0.0.1:{port}/v2/acme/blobs/{blob}");
        for (method, blob) in [(Method::GET, "one"), (Method::GET, "two")] {
            let answered = client.send("acme", method, &url(blob), &[], None).unwrap();
            assert_eq!(answered.status, StatusCode::OK, "{}", answered.words());
            assert_eq!(answered.body, blob.as_bytes());
        }
        let answered = client
            .send("acme", Method::HEAD, &url("three"), &[], None)
            .unwrap();
        assert_eq!(answered.header("content-length"), Some("5"));
        assert_eq!(
            *asked.lock().unwrap(),
            [
                (1, "GET /v2/acme/blobs/one HTTP/1.1".to_string()),
                (1, "GET /v2/acme/blobs/two HTTP/1.1".to_string()),
                (2, "GET /v2/acme/blobs/two HTTP/1.1".to_string()),
                (2, "HEAD /v2/acme/blobs/three HTTP/1.1".to_string()),
                (3, "HEAD /v2/acme/blobs/three HTTP/1.1".to_string()),
            ]
        );
    }

    #[test]
    fn a_write_on_a_kept_connection_the_far_side_closed_is_unreadable_and_not_sent_again() {
        let (port, asked) = kept(|_, at, _| at > 0);
        let source = Source::new(tailrace_seams::Seam::Store, "OCI registry stub");
        let client = Client::new(&format!("127.0.0.1:{port}"), &source);
        let blob = format!("http://127.0.0.1:{port}/v2/acme/blobs/one");
        client.send("acme", Method::GET, &blob, &[], None).unwrap();
        let tag = format!("http://127.0.0.1:{port}/v2/acme/manifests/v1");
        match client.send("acme", Method::PUT, &tag, &[], Some(b"{}")) {
            Err(Not::Unreadable(u)) => assert_eq!(u.what, tag),
            other => panic!("a put whose connection closed answered {other:?}"),
        }
        assert_eq!(
            *asked.lock().unwrap(),
            [
                (1, "GET /v2/acme/blobs/one HTTP/1.1".to_string()),
                (1, "PUT /v2/acme/manifests/v1 HTTP/1.1".to_string()),
            ]
        );
    }

    #[test]
    fn a_read_whose_new_connection_closes_too_is_unreadable_in_the_clients_words() {
        let (port, asked) = kept(|_, _, line| line.contains("/gone"));
        let source = Source::new(tailrace_seams::Seam::Store, "OCI registry stub");
        let client = Client::new(&format!("127.0.0.1:{port}"), &source);
        let url = format!("http://127.0.0.1:{port}/v2/acme/blobs/gone");
        match client.send("acme", Method::GET, &url, &[], None) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, source);
                assert_eq!(u.what, url);
                assert!(u.said.is_some_and(|w| w.starts_with("io: ")));
            }
            other => panic!("a read the far side closed twice answered {other:?}"),
        }
        assert_eq!(
            *asked.lock().unwrap(),
            [
                (1, "GET /v2/acme/blobs/gone HTTP/1.1".to_string()),
                (2, "GET /v2/acme/blobs/gone HTTP/1.1".to_string()),
            ]
        );
    }

    #[test]
    fn a_connection_refused_is_unreadable_in_the_clients_words() {
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let source = Source::new(tailrace_seams::Seam::Store, "OCI registry test");
        let client = Client::new(&format!("127.0.0.1:{port}"), &source);
        let url = format!("http://127.0.0.1:{port}/v2/");
        match client.send("", Method::GET, &url, &[], None) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, source);
                assert_eq!(u.what, url);
                let words = u.said.unwrap();
                assert!(words.to_lowercase().contains("refused"), "{words}");
            }
            other => panic!("a port nothing listens on answered {other:?}"),
        }
        assert!(client.authorization.borrow().is_empty());
    }
}
