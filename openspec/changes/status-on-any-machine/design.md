# Design

## Context

`tailrace status` (`crates/tailrace/src/commands/status.rs`) answers
`tailrace_core::status(kept, ask)`: the versions this host published,
from its record, handed no binding (`main.rs` passes
`bindings::not_read()`). On a machine that never published it answers
absent, and it shows no channel.

The two units this one comes after give it what it reads:

- `the-ledger-and-promotion`: `tailrace_core::ledger::channel(keeper,
  channel) -> Answer<Channel>`, `Channel::Empty` or `Channel::At {
  head, chain: Vec<(Digest, Promotion)>, read }`, asking the ledger
  alone; `Promotion` (channel, component, to, from, predecessor,
  asked_by, policy, at); `Read::Source` or `Read::Kept { at, said }`;
  the git ref ledger fetching its remote and answering the refs as last
  fetched where the remote does not answer, and the store ledger as last
  read; `[ledger]` in the bindings file, `NoLedger` where it is not.
- `the-policy-seam-lists-the-channels`: `Policy::channels(revision) ->
  Answer<Vec<DeclaredChannel>>` and `landed::channels_at(policy,
  landed)`, each channel with its name, component and `bound_to`, and,
  as the plan's proposal 92 amends it for 131 at 67f95da, how it moves.
  This unit reads the type as that unit lands it and renders every
  field it carries.

The design rulings this change takes, from `madswan-design`:
`agentplot/blueprints` dbe7ca0 (the address is the core's, 127.0.0.1
port 7420, and no setting moves it; the repository in the path is its
landing's remote's owner and name; the process learns a repository from
a host-state file a command in a checkout writes, and answers absent
for one none named; 132's notes half waits on the notes unit) and
85bbf54, on the user's word, which reverses dbe7ca0's third ruling: the
process is started by the first asker on the host that finds none, held
by the host's one lock as sluice's one reader per host is (sluice 129),
ends when nothing has asked for a while, may be run as a host's own
service, and no one is told to start one; a host with no bindings file
is bound to the branch (or sluice), the git ref and the rest (118); the
policy file gives `beta` and `stable` to every component (C.10), read
through 131 like any channel.

The workspace has no HTTP server crate, and the bolt's units add no
dependency (phase-1-bolts.md, How the units stay apart). Rust 1.98
(`.prototools`) has `File::try_lock`, which sluice uses for its locks.

## Goals / Non-Goals

**Goals:**

- One status answer, computed by one core function, printed by the
  command line and served as a page and its JSON.
- Channels read on a machine that never published, each channel's
  answer its own.
- One serving process per host, started by whoever first asks for the
  page, with no service file and nothing to configure.

**Non-Goals:**

- Environments and versions' notes on the page (registered, D11).
- 118's layout and host key for a host with no file: status needs
  neither, and publish keeps its absent answer.
- A served layout: bolt 5 shares the address and the process.

## Decisions

### D1. The status answer has two parts, each its own answer

`tailrace-core/src/status.rs` keeps `status(kept, ask)` and `Status`
unchanged, the record's part, which the status group's scenarios above
its marker call. It gains:

```rust
pub struct OverviewAsk {
    pub branch: Branch,
    pub component: Option<ComponentName>,
    pub revision: Option<Revision>,   // narrows the record's part
    pub at: u64,                      // the asker's moment; the core reads no clock
}

pub struct Overview {
    pub branch: Branch,
    pub asked_at: u64,
    pub channels: Answer<Channels>,   // serialized through answer_json (57)
    pub published: Answer<Status>,    // serialized through answer_json
}

pub struct Channels {
    pub revision: Revision,           // the branch's landed head, where the policy was read
    pub channels: Vec<ChannelStatus>, // in the policy seam's order
}

pub struct ChannelStatus {
    pub declared: DeclaredChannel,    // flattened: name, component, how it moves, bound_to
    pub at: Answer<Channel>,          // serialized through answer_json
}

pub fn overview(seams: &Seams, history: &History, kept: &Kept, ask: &OverviewAsk) -> Overview;
pub fn channels(seams: &Seams, history: &History, ask: &OverviewAsk) -> Answer<Channels>;
```

`channels` reads `history.head(branch)`, `Landed::check`,
`landed::channels_at`, keeps the channels of `ask.component` where one
is named, and asks `ledger::channel` of each. A channel's ledger
answer, absent, refused or unreadable, is that channel's `at`, never
the whole answer's (58). `overview` always answers: each part carries
its own `Answer`. Serde writes each inner answer with `answer_json`, as
`Promoted::Moved.store` does, so one JSON form names the four answers
at every depth.

The chain is shown newest first by the surfaces; the core answers it
in the ledger's order, from the first (121), and the text and the page
reverse it. JSON keeps the ledger's order.

Rejected: changing `status(kept, ask)` to the whole answer, which would
change the status group's scenarios above their marker; one `Answer`
for all channels, which would hide every channel behind one unreadable
chain.

### D2. Status is bound, and asks only three seams

`bindings::For` gains `Status`. `main.rs` loads it for `Command::Status`
in place of `not_read()`, and `lib.rs`'s `run` hands status the binding
as it hands the others. `bind` for `Status`:

- the file there: `[signing]` not needed; `[landing]` needed; `[ledger]`
  bound where named, else `NoLedger`; `[store]` opened and never made
  where named, else an unbound store answering absent naming `[store]`
  (`NoStore`, beside `NoLedger`), since only a store ledger reads it.
- no file (118 as amended at 85bbf54, as far as status asks): the
  landing is sluice where the checkout's tree holds
  `.config/sluice.toml`, the plain branch otherwise; the ledger the git
  ref; each with the remote `origin` where the repository's git
  configuration names one, none otherwise; no store, no signer
  (`NoStore`, and `profiles::signer::bind(None)`). Nothing is made.

A binding that fails is the channels' part of the answer: the command
computes `Overview { channels: Err(not), published: status(..), .. }`
through a core helper, `overview_unbound(not, kept, ask)`, so the
command computes nothing itself (core.31).

The fake world's `seams()` binds every fake; the status group asserts
the store, signer, target and projection fakes record no call, as
`status_after_a_publish` asserts today (60, core.32).

Rejected: binding every seam as verify does, which would make a reader
host name a store and a signer it never uses; reading the ledger's refs
directly in the command, which would bypass the seam (core.2).

### D3. The command line

`StatusArgs` gains `--branch` (`main`) and `--page`. `--component` and
`--revision` stay. The moment asked is the process's clock, passed in
the ask, as promote passes it.

Text, one block per component, the chain newest first:

```
main at 4f2a9c1, as of 2026-10-25T23:50:00Z

lib
  beta     0.3.0 (sha256:…), moves on each accepted publish
             0.3.0 from 0.2.0, asked by gate, 2026-10-25T21:02:11Z, claim sha256:…
             0.2.0, the first, asked by gate, 2026-10-24T18:40:03Z, claim sha256:…
  stable   empty, moves by promotion, bound to beta
  edge     unreadable: the chain of edge in ledger bound could not be read
           fatal: unable to access 'https://…': Could not resolve host

published from this host
  lib 0.3.0 (lib-v0.3.0) from 77b0e3d on main: index sha256:… in layout …
```

A chain read as kept adds `, as last read 2026-10-25T20:11:00Z` to the
channel's line, the source's words on the next line (59). `render.rs`
gains `utc(seconds) -> String`, RFC 3339 in UTC by the civil-from-days
rule, so no date crate is added. `--json` is `render::json(&Ok(overview))`.
The exit is 0 whenever an overview is made; `History::open` failing
exits with its code (the spec's "Status in no repository").

`--page` asks a different question and prints only its answer: D7.

### D4. The address is the core's

`tailrace-core/src/served.rs` (new):

```rust
pub const SERVING_HOST: &str = "127.0.0.1";
pub const SERVING_PORT: u16 = 7420;
pub struct RepositoryName { pub owner: String, pub name: String } // Display "owner/name"
impl RepositoryName {
    pub fn from_remote(url: &str) -> Option<RepositoryName>;
    pub fn path(&self) -> String;   // "/owner/name"
}
```

`from_remote` takes the path of `scheme://[user@]host[:port]/<path>` or
`[user@]host:<path>`, trims a trailing `/` and `.git`, and answers its
last two segments, `None` where the path has fewer, a segment is empty,
or the address is a local path (no scheme and no `host:`). It names no
forge (core.1): the owner and the name are the last two segments
whatever the host. Bolt 5's layouts take the same constants.

`History` gains `remote_url(remote) -> Answer<Option<String>>`, the raw
`remote.<name>.url` from git's configuration (no `insteadOf` applied,
so a test can point the remote's fetches elsewhere and keep its name).
The landing's remote is the one D2 binds: `[landing]`'s `remote` where
the file names one, `origin` with no file. `bindings` gains
`landing_remote(repository) -> Answer<Option<String>>`, reading only
`[landing]`, so the command learns the remote without binding seams.

### D5. The host's file of repositories

`commands/serve/host.rs`: the state directory is
`$XDG_STATE_HOME/tailrace`, else `~/.local/state/tailrace`, as the
bindings file reads `XDG_CONFIG_HOME`. `repositories` there is TOML, a
table per repository keyed by `owner/name`:

```toml
["agentplot/tailrace"]
checkout = "/Users/chuck/Code/github_agentplot/tailrace/main"
```

`name(repository, checkout)` takes `repositories.lock` beside it with
`File::lock` (blocking, held for the read and the write), reads the
file, sets the entry, writes a temporary file and renames it over, as
`published.rs` writes. `checkout_of(repository) -> Answer<PathBuf>` reads
it without a lock: absent naming the repository and that no command run
in a checkout of it on this host has named it; unreadable naming the
file with the parser's words; an entry whose path is no repository
unreadable naming the path. The checkout recorded is the working tree's
root, `History::workdir`, else the common git directory.

Only `tailrace status --page` names a repository, since it is the asker
that wants the page; plain `status` writes nothing (60). D12 says why.

### D6. The process

`commands/serve/mod.rs`, `tailrace serve`, no arguments:

1. Take `serve.lock` in the state directory with `File::try_lock`;
   `WouldBlock` prints `the host's tailrace serve answers at
   http://127.0.0.1:7420` and exits 0. The lock is held by the open file
   until the process ends.
2. Bind `TcpListener` at the core's address; a failure exits 3 naming
   the address with the system's words (59).
3. Accept in a loop, one thread per connection; record the moment of
   each request. A watcher thread exits the process 0 once nothing has
   been asked for `IDLE`, ten minutes.
4. Each request: read the request line and headers to the blank line
   (requests have no body), `GET` only, else 405; answer HTTP/1.1 with
   `Content-Length` and `Connection: close`.

Routes:

| path | answer |
|---|---|
| `/` | `{"serving": "tailrace", "revision": REVISION, "pid": <id>}` |
| `/<owner>/<name>/status.json` | `render::json` of the overview in the named checkout |
| `/<owner>/<name>/status` | the page (D8) of the same overview |
| anything else | 404 |

The repository's answer: `checkout_of` (D5), then `bindings::load(
checkout, For::Status)`, `History::open`, `Kept::at`, and `overview`, as
`tailrace status` makes it, the moment the request's. A named
repository answers 200 with the overview whatever its parts hold; an
absent repository 404 and an unreadable file 500, each with its answer
as JSON for `.json` and as a page for the page. The server is a
function over an address, the host and an idle time,
`serve_on(address, host, idle)`, taking the lock (step 1) before it
binds the address (step 2), so tests give it a free port the system
picked; `host` being a `Host { state, config,
home }`: the state directory, the bindings file's path and the home
directory, which `tailrace serve` reads from the environment once and
a test gives as temporary directories, so no test reads the real
host's files or environment. `tailrace serve` calls it with the core's
address and `IDLE`, and nothing else can move them. The request's
binding is `bindings::bind(&host.config, host.home, checkout,
For::Status)`.

The process's working directory is the state directory, never a
checkout (132: no checkout starts a server of its own beside it).

### D7. The asker that finds none starts one

`tailrace status --page`, in `commands/status.rs` over
`commands/serve/host.rs`, as `page(checkout, host, start) ->
Answer<String>`, which the command calls with the `Host` of the
environment and the `start` below:

1. The repository: `landing_remote`, `remote_url`, `from_remote`; no
   remote, absent naming the landing's remote; an address with no owner
   and name, unreadable naming the remote and its address.
2. `name(repository, checkout)` (D5).
3. `find_or_start(address, start)`: `GET /` with a one-second timeout;
   tailrace's document answers it. A connection refused runs `start`,
   then asks again every 100 ms for up to five seconds. Anything else
   answering is unreadable naming the address and what it answered. A
   process that has not answered in five seconds is unreadable naming
   the address and saying it was started and did not answer.
4. The answer: `Ok("http://127.0.0.1:7420/<owner>/<name>/status")`.

`start` runs `std::env::current_exe()` with `serve`, `stdin`, `stdout`
and `stderr` null, `process_group(0)` so the asker's terminal and its
signals do not reach it, working directory the state directory, and
does not wait. Two askers that both find none both start one; the lock
lets one serve and the other exit 0 (D6), so both find one process.
`find_or_start` takes `start` as a closure so the tests start
`serve_on` in a thread on a port the system picks.

### D8. The page

`commands/serve/page.rs`: one HTML document, styles inline, no script,
rendered from the `Overview` value and nothing else (core.31): the
repository and branch, the landed revision and the moment asked; per
component a table of its channels, each row the channel, how it moves,
its binding, its version and index or "empty", or its refusal or
unreadable answer with the source's words in a `<pre>` (59); below each
channel its chain newest first; then the record's part. Every string is
escaped (`&`, `<`, `>`, `"`, `'`). No environment row and no notes are
shown (D11).

### D9. The conformance groups

The status group, below its marker, over `FakeWorld` with the fake
policy declaring `beta` and `stable` in the world's component and the
fake keeper holding promotions made with `Promotion::claim`, sealed by
the fake signer before the scenario's asking begins:

| scenario | cites | plays |
|---|---|---|
| every channel at its version or empty | 31, 101, 125, 131 | |
| a channel read as last read, the store unreachable | 36, 60, 61 | S10 |
| one channel's answer beside another's | 58, 59, 105 | |
| everything kept deleted, the channels the same | 63 | S12 |

`world.rs` gains what they need: `promote_fake(channel, version)`, which
appends a promotion of a published version to the fake keeper, and
`declare_channels(&[...])` over the fake policy's `belongs`. Each
scenario counts the store's, signer's, targets' and projections' calls
before and after, as `status_after_a_publish` does.

The surfaces group gains `Question::Channels { branch }` and, below its
marker, "the command line and the page answer the channels" (36, 56,
57, 132), comparing each surface's `overview` with the core's, the
`asked_at` of each checked between the moments before and after.
`crates/tailrace/tests/conformance.rs` gains a second rig, the served
page: it names the world's repository in a state directory of its own
and answers through the process's request handling, without a socket;
asked `Publish` or `Verify`, which the page does not answer, it ends the
scenario with the suite's `not_applicable`, naming the page as a reader.
Each rig's `Question::Status` answers the record's part of its status
JSON, `["value"]["published"]`, which is the question's own doc ("status, of
every version the host published") and keeps the scenario above the
marker as it is.

### D10. The statement at 85bbf54

`statement.rs` reads `agentplot/blueprints 85bbf54`, and A.10 holds 132
after 60, before 129, as the statement places it; its test asserts 1 to
132 once each. Of the statement's text since 673d334, only 132 is a new
number; 114, 118, 129, 131 and C.10 changed words, not numbers.

### D11. The register and its parts

`register.rs` gains:

```rust
pub struct Part { pub clause: u16, pub part: &'static str, pub unit: &'static str, pub why: &'static str }
pub const PARTS: &[Part] = &[
    Part { clause: 132, part: "what every environment should run, runs and waits on", unit: TARGET, why: "the page shows environments once a target reports them, and phase 1 leaves every target to phase 2" },
    Part { clause: 132, part: "each version's notes", unit: NOTES, why: "no version carries notes yet, and the notes unit waits on the user" },
];
```

`NOTES` names "the notes unit, which the user has not said yes to".
`coverage::check` takes `parts` beside `pending`: each part printed as
pending, `Finding::PartOfUncited { clause }` where no scenario cites its
clause, and `Finding::Unexplained` for a part with no unit or reason,
as for an entry. `bin/coverage.rs` passes `register::PARTS`.

The register loses 36 (cited, D9). 98 moves from this unit's block to
the first target's, its reason "what each environment should run and
runs is reported by a target, and phase 1 leaves every target to phase
2". The `ACROSS` constant goes. 114 stays in `SERVED`, its reason
"a served layout is reached at the core's address, which bolt 5's
first served projection serves and cites; the process is
status-on-any-machine's, proven in the command line's tests".

### D12. Who names a repository

132 says the process answers a repository "once a command run in a
checkout of it on that host has named it", and 60 says asking what
tailrace holds writes nothing. Plain `tailrace status` asks what
tailrace holds, so it names nothing. `tailrace status --page` asks
where the page is, and is the first asker 114 names, so it names the
repository and starts the process. No other command names one in this
unit; the plugins ask `--page`.

### D13. ADR 0018

"The host's one serving process is started by the first asker that
finds none and answers repositories a checkout named": D4 to D8 and
D12, with ADR 0006 (serve only where a client reads no files) and the
rulings of dbe7ca0 and 85bbf54 as its context; the std sockets and lock
over an HTTP crate as its first alternative; a service unit shipped as
its second, rejected by 85bbf54. Proposed, for the user to accept with
the change.

## Risks / Trade-offs

- [A store ledger's chain is read from the store while C.14 lets
  asking read it there, and 60 and core.32 say asking reaches no
  store] → status asks the ledger seam alone, and the store ledger's
  profile reads the store as C.14 says; status's own code asks no store
  seam. The gap goes to the design agent.
- [A git ref ledger's read fetches into the repository's own refs, and
  a store ledger's writes its kept chain] → the 673d334 ruling counts a
  fetch as a read; status writes nothing of its own.
- [The file of repositories is kept by the host and is not rebuilt from
  the store and git (63)] → it is rebuilt by the next `--page` in a
  checkout; until then the process answers absent naming the
  repository, never a wrong answer. The gap goes to the design agent.
- [An empty channel answered from a kept read loses when it was read:
  `ledger::channel` answers `Channel::Empty` without `read`] → the
  moment status asked is on every answer; carrying `read` on `Empty` is
  the ledger unit's type to change.
- [A plain HTTP server on the standard library] → it answers one
  request per connection on loopback, `GET` only, no body; anything
  more is bolt 5's to add when a client needs it.
- [`origin` as the remote of a host with no bindings file] → 118 names
  the profiles, not the remote; `origin` is git's own default name, and
  a host that uses another names it in `[landing]`. The gap goes to the
  design agent.
- [The policy unit's amendment for how a channel moves has not landed]
  → this unit renders the field as that unit lands it, and its text
  form follows that unit's names.
