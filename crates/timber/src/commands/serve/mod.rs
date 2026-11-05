//! `timber serve`: the host's one serving process (57, 114, 132; design
//! D5 to D8). It answers HTTP at the core's address, which no setting,
//! flag or variable moves, for as long as it holds the host's serving
//! lock, and ends when nothing has asked it for ten minutes; a second
//! that finds the lock held says where the first answers and ends.
//!
//! Each request is answered anew from the one core and written nothing
//! for (60, rule core.31): `/serving.json` names the process,
//! `/<owner>/<name>/status.json` is the status answer in the checkout
//! that last named the repository, as `timber status --json` prints it
//! there, and `/<owner>/<name>/status` its page; `/status.json` is every
//! repository the host's file names, one row each made by the core from
//! its status answer, and `/` their page. No row is kept between
//! requests and no timer renders one (132). Under
//! `/<owner>/<name>/<ecosystem>/` it answers that checkout's served
//! layout of the ecosystem: the request is handed, with the base it was
//! asked at, the address the process is bound at with that prefix, to the
//! projection the checkout binds for the ecosystem, which
//! answers it in its client's own protocol, a version the layout lacks
//! rendered into it by the core on the request, and the response is
//! answered as given, bytes and all (86, 114, 115; design D2). The process
//! holds no layout of its own. Its working directory is the state
//! directory, never a checkout (114).

use std::io::{self, BufRead as _, BufReader, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use clap::Args;
use serde::Serialize;
use timber_core::{
    answer_json, miss, place, row, History, Layouts, MissAsk, Overview, Repositories,
    RepositoryName, SERVING_HOST, SERVING_PORT,
};
use timber_seams::{
    no_serving, Answer, Branch, Ecosystem, Layout, Not, Pinned, Request, Seam, Source,
};

use crate::bindings::For;
use crate::commands::status::{now, overview_in};
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

/// `timber serve` takes no argument: nothing moves its address (114).
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
                "the host's timber serve answers at http://{}",
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

/// 114, 115: the serving process at `address` over `host`, each checkout
/// bound as the host binds it ([`Host::bind_for`]), ending once nothing
/// has asked for `idle` and no checkout the host's file names holds a
/// rendered layout its projection serves ([`host::serves_a_layout`]);
/// while one does, the idle time starts again, since the ecosystem's
/// client cannot start the process (design D11). It takes the host's
/// serving lock before it binds the address, and answers
/// [`Served::Held`] at once where another holds it; an address that
/// cannot be bound is unreadable naming it with the system's words (59).
/// The lock is let go as it returns.
pub fn serve_on(address: SocketAddr, host: &Host, idle: Duration) -> Answer<Served> {
    let bound = host.clone();
    serve_with(
        address,
        host,
        idle,
        Arc::new(move |checkout: &Path, command| bound.bind_for(checkout, command)),
    )
}

/// How the process binds a checkout on any of its threads.
pub type Binder = Arc<dyn Fn(&Path, For) -> Answer<Binding> + Send + Sync>;

/// [`serve_on`] with each checkout bound by `bind`, for its requests and
/// for whether a layout is served: a test binds over fakes.
pub fn serve_with(
    address: SocketAddr,
    host: &Host,
    idle: Duration,
    bind: Binder,
) -> Answer<Served> {
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
                let bind = Arc::clone(&bind);
                thread::spawn(move || {
                    connection(stream, address, &host, &*bind);
                    *asked.lock().expect("the moment asked") = Instant::now();
                });
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                if asked.lock().expect("the moment asked").elapsed() >= idle {
                    if host::serves_a_layout(host, &*bind) {
                        *asked.lock().expect("the moment asked") = Instant::now();
                        continue;
                    }
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

/// One request on `stream`, asked of the process bound at `address`,
/// answered and the connection closed.
fn connection(stream: TcpStream, address: SocketAddr, host: &Host, bind: Bind<'_>) {
    let mut stream = stream;
    let _ = match request(&stream) {
        Ok((method, target)) if method == "HEAD" => {
            respond_with(address, host, bind, &method, &target).write_head(&mut stream)
        }
        Ok((method, target)) => {
            respond_with(address, host, bind, &method, &target).write(&mut stream)
        }
        Err(said) => {
            Response::text(400, format!("a request it cannot read: {said}\n")).write(&mut stream)
        }
    };
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

/// What the process answers a request with: its status, the type of its
/// body, and the body's bytes, a projection's as it answered them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub content_type: String,
    pub body: Vec<u8>,
}

impl Response {
    fn text(status: u16, body: String) -> Response {
        Response {
            status,
            content_type: "text/plain; charset=utf-8".into(),
            body: body.into_bytes(),
        }
    }

    fn json<T: Serialize>(status: u16, value: &T) -> Response {
        let mut body = serde_json::to_string_pretty(value).expect("an answer is plain data");
        body.push('\n');
        Response {
            status,
            content_type: "application/json".into(),
            body: body.into_bytes(),
        }
    }

    fn html(status: u16, body: String) -> Response {
        Response {
            status,
            content_type: "text/html; charset=utf-8".into(),
            body: body.into_bytes(),
        }
    }

    fn reason(&self) -> &'static str {
        match self.status {
            200 => "OK",
            400 => "Bad Request",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            502 => "Bad Gateway",
            _ => "Internal Server Error",
        }
    }

    /// The response as HTTP/1.1, its `Content-Length` its body's length
    /// in bytes, the connection closed after it.
    pub fn write(&self, out: &mut dyn Write) -> io::Result<()> {
        self.head(out)?;
        out.write_all(&self.body)?;
        out.flush()
    }

    /// The response to a `HEAD`: its head alone, its `Content-Length` the
    /// length of the body it leaves out.
    pub fn write_head(&self, out: &mut dyn Write) -> io::Result<()> {
        self.head(out)?;
        out.flush()
    }

    fn head(&self, out: &mut dyn Write) -> io::Result<()> {
        write!(
            out,
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.status,
            self.reason(),
            self.content_type,
            self.body.len(),
        )
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

/// How the process binds a checkout for a command: as the host binds it,
/// or, in a test, over fakes.
pub type Bind<'a> = &'a dyn Fn(&Path, For) -> Answer<Binding>;

/// `method` of `target` asked of `host`'s process at the core's address,
/// each checkout bound as `host` binds it.
pub fn respond(host: &Host, method: &str, target: &str) -> Response {
    respond_with(
        address(),
        host,
        &|checkout, command| host.bind_for(checkout, command),
        method,
        target,
    )
}

/// `method` of `target` asked of `host`'s process bound at `address`,
/// each checkout bound by `bind`, status's binding for its routes and
/// sync's for a served layout: the routes of design D6, `GET` alone, and
/// the served layout's of design D2, whose method its projection answers.
pub fn respond_with(
    address: SocketAddr,
    host: &Host,
    bind: Bind<'_>,
    method: &str,
    target: &str,
) -> Response {
    let path = target.split_once('?').map_or(target, |(path, _)| path);
    if let Some((repository, ecosystem, rest)) = served_layout(path) {
        return layout(address, host, bind, &repository, &ecosystem, method, rest);
    }
    if method != "GET" {
        return Response::text(405, format!("{method} is not answered; only GET is\n"));
    }
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
    if path == "/status.json" {
        let answer = repositories(host, bind);
        return Response::json(status_of(&answer), &answer_json(&answer));
    }
    if path == "/" {
        let answer = repositories(host, bind);
        return Response::html(status_of(&answer), page::root(&answer));
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

/// The repository, ecosystem and the rest of `path` where it names a
/// served layout: `/<owner>/<name>/<ecosystem>/<rest>`, or exactly
/// `/<owner>/<name>/<ecosystem>` with `rest` empty, `<ecosystem>` any
/// name but the status routes' own three-part paths (design D2).
fn served_layout(path: &str) -> Option<(RepositoryName, Ecosystem, &str)> {
    let parts: Vec<&str> = path.strip_prefix('/')?.splitn(4, '/').collect();
    let (owner, name, ecosystem, rest) = match parts[..] {
        [_, _, "status" | "status.json"] => return None,
        [owner, name, ecosystem] => (owner, name, ecosystem, ""),
        [owner, name, ecosystem, rest] => (owner, name, ecosystem, rest),
        _ => return None,
    };
    if owner.is_empty() || name.is_empty() || ecosystem.is_empty() {
        return None;
    }
    let repository = RepositoryName {
        owner: owner.to_string(),
        name: name.to_string(),
    };
    Some((repository, Ecosystem::new(ecosystem), rest))
}

/// 86, 114, 115: `method` of `rest` below the served layout of
/// `ecosystem` in the checkout that last named `repository`, bound for
/// sync: handed to the projection bound for the ecosystem with the base it
/// was asked at, `http://<address>/<owner>/<name>/<ecosystem>`, `address`
/// the one the process is bound at, which on a host is the core's, since
/// nothing else binds it (114, rule core.36), and in a test the test's
/// own. The projection answers the method as its client's protocol does,
/// and its response is answered as given (design D2). A version the
/// layout lacks is rendered into it by the core on the request, under the
/// policy a sync with no argument would use there (design D4). A
/// repository none named is its absent answer and a checkout that cannot
/// be bound its binding's, each in text; an ecosystem the checkout binds
/// no projection for, or whose projection serves none, `404` naming it.
fn layout(
    address: SocketAddr,
    host: &Host,
    bind: Bind<'_>,
    repository: &RepositoryName,
    ecosystem: &Ecosystem,
    method: &str,
    rest: &str,
) -> Response {
    let words =
        |not: Not| Response::text(status_of(&Err::<(), _>(not.clone())), format!("{not}\n"));
    let checkout = match host::checkout_of(host, repository) {
        Ok(checkout) => checkout,
        Err(not) => return words(not),
    };
    let binding = match bind(&checkout, For::Sync) {
        Ok(binding) => binding,
        Err(not) => return words(not),
    };
    let serves_none = || {
        Response::text(
            404,
            format!("the checkout of {repository} serves no {ecosystem} layout\n"),
        )
    };
    let Some(projection) = binding.seams.projection(ecosystem) else {
        return serves_none();
    };
    let history = match History::open(&checkout) {
        Ok(history) => history,
        Err(not) => return words(not),
    };
    let tree = history.workdir().unwrap_or_else(|| checkout.clone());
    let main = Branch::new("main");
    let on_miss = |pin: &Pinned| {
        let ask = MissAsk {
            branch: main.clone(),
            revision: history.base(&main)?,
            ecosystem: ecosystem.clone(),
            tree: tree.clone(),
            pin: pin.clone(),
            trusted: binding.trust.clone(),
            when: now(),
        };
        miss(&binding.seams, &Layouts::at(&history), &ask).map(|_| ())
    };
    let request = Request {
        method: method.to_string(),
        path: rest.to_string(),
        base: format!(
            "http://{address}/{}/{}/{ecosystem}",
            repository.owner, repository.name
        ),
    };
    let at = Layout {
        at: place(&tree, ecosystem).display().to_string(),
    };
    match projection.serve(&at, &request, &on_miss) {
        Ok(response) => Response {
            status: response.status,
            content_type: response.content_type,
            body: response.body,
        },
        Err(not) if not == no_serving(not.source()) => serves_none(),
        Err(not) => words(not),
    }
}

fn not_found(path: &str) -> Response {
    Response::text(404, format!("{path} is not answered here\n"))
}

/// 57, 132: the status answer in the checkout that last named
/// `repository`, as `timber status --json` makes it there.
fn overview_of(host: &Host, bind: Bind<'_>, repository: &RepositoryName) -> Answer<Overview> {
    let checkout = host::checkout_of(host, repository)?;
    overview_in(&bind(&checkout, For::Status), &checkout, "main", None, None)
}

/// 132: every repository the host's file names, in turn, each row made
/// by the core from its checkout's status answer, or that row's own
/// answer where its checkout is gone or its status cannot be asked; none
/// where the file is not there, and unreadable with the parser's words
/// where it cannot be read.
fn repositories(host: &Host, bind: Bind<'_>) -> Answer<Repositories> {
    let asked_at = now();
    let repositories = host::named(host)?
        .into_iter()
        .map(|(repository, checkout)| {
            let checkout = checkout?;
            let overview =
                overview_in(&bind(&checkout, For::Status), &checkout, "main", None, None)?;
            Ok(row(&repository, &overview))
        })
        .collect();
    Ok(Repositories {
        asked_at,
        repositories,
    })
}
