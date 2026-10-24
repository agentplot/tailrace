//! The request path (design D3, D4): a request sent as asked and its
//! answer read whole, a 4xx or 5xx answered as a response so the
//! registry's words reach the caller verbatim (59); a request the
//! registry challenges answered once with the host's Docker credential,
//! reached at that moment through the host's Docker configuration and
//! the helper it names, as `Basic` or as a `Bearer` token asked of the
//! challenge's realm for the scope it names. What authorizes the next
//! request is held in memory by the one handle and written nowhere (64,
//! rule core.21).

use std::cell::RefCell;
use std::path::PathBuf;

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

/// A token a realm answers (design D4).
#[derive(Deserialize)]
struct Token {
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    access_token: Option<String>,
}

/// The connection to one registry host: the agent, the host the
/// credential is the host's for, and what authorizes the next request.
pub(super) struct Client {
    agent: ureq::Agent,
    host: String,
    source: Source,
    authorization: RefCell<Option<String>>,
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
            authorization: RefCell::new(None),
        }
    }

    /// `method` of `url`, with `headers` and `body`, answered as the
    /// registry answered it; a challenge answered once with the host's
    /// credential and the request sent once more. Unreadable naming `url`
    /// in the client's words where no connection is made or the answer
    /// cannot be read.
    pub(super) fn send(
        &self,
        method: Method,
        url: &str,
        headers: &[(&str, &str)],
        body: Option<&[u8]>,
    ) -> Answer<Answered> {
        let held = self.authorization.borrow().clone();
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
        *self.authorization.borrow_mut() = Some(authorization.clone());
        self.once(&method, url, headers, body, Some(&authorization))
    }

    fn once(
        &self,
        method: &Method,
        url: &str,
        headers: &[(&str, &str)],
        body: Option<&[u8]>,
        authorization: Option<&str>,
    ) -> Answer<Answered> {
        let unreadable = |words: String| Not::unreadable(&self.source, url, Some(words));
        let mut request = Request::builder().method(method.clone()).uri(url);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        if let Some(authorization) = authorization {
            request = request.header("authorization", authorization);
        }
        let answered = match body {
            Some(body) => request
                .body(body)
                .map_err(|e| unreadable(e.to_string()))
                .and_then(|r| self.agent.run(r).map_err(|e| unreadable(e.to_string()))),
            None => request
                .body(())
                .map_err(|e| unreadable(e.to_string()))
                .and_then(|r| self.agent.run(r).map_err(|e| unreadable(e.to_string()))),
        };
        let mut response = answered?;
        let body = response
            .body_mut()
            .with_config()
            .limit(u64::MAX)
            .read_to_vec()
            .map_err(|e| unreadable(e.to_string()))?;
        Ok(Answered {
            status: response.status(),
            headers: response.headers().clone(),
            body,
        })
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
        let file = std::env::var_os("DOCKER_CONFIG")
            .filter(|d| !d.is_empty())
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
        std::env::set_var("DOCKER_CONFIG", dir.path());

        let source = Source::new(tailrace_seams::Seam::Store, "OCI registry stub");
        let client = Client::new(&format!("127.0.0.1:{port}"), &source);
        let answered = client
            .send(
                Method::GET,
                &format!("http://127.0.0.1:{port}/v2/"),
                &[],
                None,
            )
            .unwrap();
        std::env::remove_var("DOCKER_CONFIG");
        assert_eq!(answered.status, StatusCode::OK, "{}", answered.words());
        assert_eq!(answered.body, b"{}");
        assert_eq!(
            client.authorization.borrow().as_deref(),
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
        match client.send(Method::GET, &url, &[], None) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, source);
                assert_eq!(u.what, url);
                let words = u.said.unwrap();
                assert!(words.to_lowercase().contains("refused"), "{words}");
            }
            other => panic!("a port nothing listens on answered {other:?}"),
        }
        assert!(client.authorization.borrow().is_none());
    }
}
