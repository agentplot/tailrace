//! `tailrace serve`: the host's one serving process (57, 114, 132; design
//! D5 to D8). It answers HTTP at the core's address, which no setting,
//! flag or variable moves, for as long as it holds the host's serving
//! lock, and ends when nothing has asked it for ten minutes; a second
//! that finds the lock held says where the first answers and ends.
//!
//! Each request is answered anew from the one core and written nothing
//! for (60, rule core.31): `/serving.json` names the process,
//! `/<owner>/<name>/status.json` is the status answer in the checkout
//! that last named the repository, as `tailrace status --json` prints it
//! there, and `/<owner>/<name>/status` its page. Its working directory
//! is the state directory, never a checkout (114).

use std::io::{self, BufRead as _, BufReader, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use clap::Args;
use serde::Serialize;
use tailrace_core::{answer_json, Overview, RepositoryName, SERVING_HOST, SERVING_PORT};
use tailrace_seams::{Answer, Not, Seam, Source};

use crate::commands::status::overview_in;
use crate::{render, Binding, REVISION};

pub mod host;
pub mod page;

use host::{Host, Serving, SERVING_NAME};

/// How long the process waits with nothing asked before it ends.
pub const IDLE: Duration = Duration::from_secs(600);

/// How often the process looks for a connection while none waits, or
/// asks the lock again.
const POLL: Duration = Duration::from_millis(25);

/// How long the lock is asked for before another is taken to hold it.
const HELD: Duration = Duration::from_millis(250);

/// The most of a request's head the process reads.
const HEAD: u64 = 64 * 1024;

/// `tailrace serve` takes no argument: nothing moves its address (114).
#[derive(Debug, Args)]
pub struct ServeArgs {}

/// The core's address, where the host's one process answers.
pub fn address() -> SocketAddr {
    let ip: IpAddr = SERVING_HOST
        .parse()
        .expect("the core's address is an address");
    SocketAddr::new(ip, SERVING_PORT)
}

/// How a serving process ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Served {
    /// Nothing asked it for the idle time.
    Idle,
    /// Another process holds the host's serving lock and answers.
    Held,
}

/// The host's process at the core's address, over the host the
/// environment places; exits 0 once idle or where another answers, and
/// with the answer's code where it cannot serve.
pub fn run(_: &ServeArgs, out: &mut dyn Write) -> i32 {
    let answer = Host::of_environment().and_then(|host| serve_on(address(), &host, IDLE));
    match answer {
        Ok(Served::Idle) => render::OK,
        Ok(Served::Held) => {
            let _ = writeln!(
                out,
                "the host's tailrace serve answers at http://{}",
                address()
            );
            render::OK
        }
        Err(not) => {
            let _ = render::not(out, "", &not);
            render::code(&not)
        }
    }
}

fn serving(address: SocketAddr) -> Source {
    Source::new(Seam::Host, format!("serve at http://{address}"))
}

/// 114: the serving process at `address` over `host`, ending once
/// nothing has asked for `idle`. It takes the host's serving lock before
/// it binds the address, and answers [`Served::Held`] at once where
/// another holds it; an address that cannot be bound is unreadable
/// naming it with the system's words (59). The lock is let go as it
/// returns.
pub fn serve_on(address: SocketAddr, host: &Host, idle: Duration) -> Answer<Served> {
    let lock = host::serving_lock(host)?;
    let lock_path = host.state.join(host::SERVING);
    // A process spawned at the moment the last holder let go may hold a
    // copy of the lock's descriptor until it runs; the lock is asked
    // again for a moment before another is taken to hold it.
    let asked = Instant::now();
    loop {
        match lock.try_lock() {
            Ok(()) => break,
            Err(std::fs::TryLockError::WouldBlock) if asked.elapsed() >= HELD => {
                return Ok(Served::Held)
            }
            Err(std::fs::TryLockError::WouldBlock) => thread::sleep(POLL),
            Err(std::fs::TryLockError::Error(e)) => {
                return Err(Not::unreadable(
                    &Source::host(&lock_path),
                    lock_path.display().to_string(),
                    Some(e.to_string()),
                ))
            }
        }
    }
    let unbound = |e: io::Error| {
        Not::unreadable(
            &serving(address),
            format!("the address http://{address}"),
            Some(e.to_string()),
        )
    };
    let listener = TcpListener::bind(address).map_err(unbound)?;
    listener.set_nonblocking(true).map_err(unbound)?;
    let asked = Arc::new(Mutex::new(Instant::now()));
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                *asked.lock().expect("the moment asked") = Instant::now();
                let host = host.clone();
                let asked = Arc::clone(&asked);
                thread::spawn(move || {
                    connection(stream, &host);
                    *asked.lock().expect("the moment asked") = Instant::now();
                });
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                if asked.lock().expect("the moment asked").elapsed() >= idle {
                    drop(listener);
                    drop(lock);
                    return Ok(Served::Idle);
                }
                thread::sleep(POLL);
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(unbound(e)),
        }
    }
}

/// One request on `stream`, answered and the connection closed.
fn connection(stream: TcpStream, host: &Host) {
    let response = match request(&stream) {
        Ok((method, target)) => respond(host, &method, &target),
        Err(said) => Response::text(400, format!("a request it cannot read: {said}\n")),
    };
    let mut stream = stream;
    let _ = response.write(&mut stream);
}

/// The method and target of the request on `stream`, its head read to
/// the blank line; requests carry no body.
fn request(stream: &TcpStream) -> Result<(String, String), String> {
    stream.set_nonblocking(false).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(io::Read::take(stream, HEAD));
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;
    let mut parts = line.split_whitespace();
    let (Some(method), Some(target), Some(_)) = (parts.next(), parts.next(), parts.next()) else {
        return Err(format!("{:?} is no request line", line.trim_end()));
    };
    let (method, target) = (method.to_string(), target.to_string());
    loop {
        let mut header = String::new();
        let read = reader.read_line(&mut header).map_err(|e| e.to_string())?;
        if read == 0 || header == "\r\n" || header == "\n" {
            return Ok((method, target));
        }
    }
}

/// What the process answers a request with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub content_type: &'static str,
    pub body: String,
}

impl Response {
    fn text(status: u16, body: String) -> Response {
        Response {
            status,
            content_type: "text/plain; charset=utf-8",
            body,
        }
    }

    fn json<T: Serialize>(status: u16, value: &T) -> Response {
        let mut body = serde_json::to_string_pretty(value).expect("an answer is plain data");
        body.push('\n');
        Response {
            status,
            content_type: "application/json",
            body,
        }
    }

    fn html(status: u16, body: String) -> Response {
        Response {
            status,
            content_type: "text/html; charset=utf-8",
            body,
        }
    }

    fn reason(&self) -> &'static str {
        match self.status {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            405 => "Method Not Allowed",
            _ => "Internal Server Error",
        }
    }

    /// The response as HTTP/1.1, the connection closed after it.
    pub fn write(&self, out: &mut dyn Write) -> io::Result<()> {
        write!(
            out,
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.status,
            self.reason(),
            self.content_type,
            self.body.len(),
            self.body
        )?;
        out.flush()
    }
}

/// The status a page or its JSON is served with: 200 for an answer,
/// 404 for an absence, 500 for anything else.
fn status_of<T>(answer: &Answer<T>) -> u16 {
    match answer {
        Ok(_) => 200,
        Err(Not::Absent(_)) => 404,
        Err(_) => 500,
    }
}

/// `method` of `target` asked of `host`'s process, each checkout bound
/// as `host` binds it.
pub fn respond(host: &Host, method: &str, target: &str) -> Response {
    respond_with(host, &|checkout| host.bind(checkout), method, target)
}

/// `method` of `target` asked of `host`'s process, each checkout bound by
/// `bind`: the routes of design D6, `GET` alone.
pub fn respond_with(
    host: &Host,
    bind: &dyn Fn(&Path) -> Answer<Binding>,
    method: &str,
    target: &str,
) -> Response {
    if method != "GET" {
        return Response::text(405, format!("{method} is not answered; only GET is\n"));
    }
    let path = target.split_once('?').map_or(target, |(path, _)| path);
    if path == "/serving.json" {
        return Response::json(
            200,
            &Serving {
                serving: SERVING_NAME.to_string(),
                revision: REVISION.to_string(),
                pid: std::process::id(),
            },
        );
    }
    let parts: Vec<&str> = path.strip_prefix('/').unwrap_or(path).split('/').collect();
    match parts[..] {
        [owner, name, last] if !owner.is_empty() && !name.is_empty() => {
            let repository = RepositoryName {
                owner: owner.to_string(),
                name: name.to_string(),
            };
            match last {
                "status.json" => {
                    let answer = overview_of(host, bind, &repository);
                    Response::json(status_of(&answer), &answer_json(&answer))
                }
                "status" => {
                    let answer = overview_of(host, bind, &repository);
                    Response::html(status_of(&answer), page::repository(&repository, &answer))
                }
                _ => not_found(path),
            }
        }
        _ => not_found(path),
    }
}

fn not_found(path: &str) -> Response {
    Response::text(404, format!("{path} is not answered here\n"))
}

/// 57, 132: the status answer in the checkout that last named
/// `repository`, as `tailrace status --json` makes it there.
fn overview_of(
    host: &Host,
    bind: &dyn Fn(&Path) -> Answer<Binding>,
    repository: &RepositoryName,
) -> Answer<Overview> {
    let checkout = host::checkout_of(host, repository)?;
    overview_in(&bind(&checkout), &checkout, "main", None, None)
}
