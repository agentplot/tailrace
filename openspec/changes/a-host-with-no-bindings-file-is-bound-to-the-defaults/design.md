# Design

## Context

ADR 0013 put the host's bindings in `$XDG_CONFIG_HOME/tailrace/config.toml`
and decided that no file is absent naming its path, that a file naming
no `[store]` or `[landing]` is absent naming the table, and that a host
naming no `[signing]` signs nothing. The ledger's unit (ADR 0016) added
`[ledger]` and an unbound ledger, `NoLedger`, that answers absent. Clause
118, as the statement now reads (blueprints 85bbf54), binds a host with
no file to the layout under the user's data directory (C.2), a host key
made on first use (C.8), the branch or sluice where the repository is
set up for it (C.6, C.5), and the git ref (C.13), and says the file is
for pointing a host at a registry, a key service or a shared ledger.

What is there today, on this bolt (`crates/tailrace/src/bindings/mod.rs`):

- `read(file, home)` answers absent for a missing file and a `Bindings`
  of four optional tables otherwise.
- `bind(file, home, repository, command)` answers absent for a missing
  `[store]` or `[landing]`, for publish and promote with no `[signing]`,
  and for promote with no `[ledger]`; it binds `NoLedger` for the rest.
- Each seam's family matches a `Table`'s key to its profile:
  `profiles::store::{bind, named}`, `profiles::landing::bind`,
  `profiles::ledger::bind`, `profiles::signer::{read, plan, bind}`. The
  bindings module names no tool (core.1).
- The host key profile reads its key at each sign and answers absent
  where no file is there (`profiles/signer/host_key/mod.rs`). Tests
  already make Ed25519 keys with `ssh_key::PrivateKey::random(&mut
  OsRng, Algorithm::Ed25519)`; the workspace's `ssh-key` 0.6.7 has its
  `ed25519` and `getrandom` features on, which enable `rand_core`.

Rules this change keeps: core.1 (the bindings module names no tool, so
each default lives in its family), core.19 (a broken file is still
unreadable, never replaced by a default), core.21 (the made key is the
host's file, read at use, kept nowhere of tailrace's), core.22 (sluice's
file is read from a commit, never the working tree), core.32 (verify,
sync and rebuild make nothing).

## Goals / Non-Goals

**Goals:**

- Every command that reads a bindings file works on a host with none,
  bound to the four given profiles.
- A file binds what it names and leaves the rest given.
- The given host key is made on first sign and its name is stable.

**Non-Goals:**

- `[gate]` (the-publishs-host-tells-the-gate unit) and anything of
  `status`.
- Changing a named table's behaviour, a profile's operations, or the
  conformance suite: no clause of Parts A or B changes, and every
  profile bound by default is one the suite already holds.
- A line in the publish's answer saying a key was made.

## Decisions

### D1. A missing file is an empty file

`read` answers a `Bindings` with every table `None` where no file is at
the path, in place of absent. `bind` then gives each `None` table its
default, so no file, an empty file and a file naming some tables are one
path through the code. `load`'s absent answer where neither
`XDG_CONFIG_HOME` nor `HOME` is set stays: with neither, no given path
can be placed.

Rejected: keeping `read`'s absent answer and catching it in `bind`. It
would leave two ways to be bound to the defaults and one answer of
`read` that nothing reads.

### D2. Each family gives its own default

The bindings module still names no tool (core.1). Each family exposes
the binding it gives where its table is missing, with the places it
needs passed in:

- `profiles::store`: the layout at `<data>/tailrace/layout`, where
  `<data>` is `$XDG_DATA_HOME` where it is set and not empty, else
  `<home>/.local/share`. It is a `Table` of key `layout` whose value is
  that path, so `bind`, `named` and the store ledger's binding read it
  exactly as a written one; a publish makes it with its parents
  (`Layout::create` over a missing path), and verify opens it and answers
  unreadable where it is not there, as a named one does.
- `profiles::landing`: sluice where `git -C <repository> cat-file -e
  HEAD:.config/sluice.toml` succeeds, else a plain branch, either with
  no remote. git's "not a valid object name" for an unborn `HEAD` or a
  missing path both mean a branch; a repository git cannot open is
  unreadable with git's words. The check runs through `profiles::run`'s
  `Tool`, as every git call of the landings does.
- `profiles::ledger`: `GitRef::new(repository, None)`.
- `profiles::signer`: for publish and promote, the host key at
  `<config>/tailrace/signing`, `<config>` being the bindings file's own
  directory's parent (the file is `<config>/tailrace/config.toml`), bound
  as the given key (D3); for every other command, `plan(None)` as today.

`Table` gains a way to be made in code, carrying the bindings file's
path for its answers and marked given, so an answer about a given table
names it as given: `<file>, [store] layout (given)`. The marker is in
`Table::at`, so no profile changes its words.

`bind` keeps its order: every table, written or given, is read before
the layout is made, so a file that cannot bind leaves nothing behind.

### D3. The given host key

`HostKey` gains a third shape beside a named key and verify-only: a
given key, `HostKey::given(signer, path)`. It differs from a named key
in one place: where sign finds no file at its path, it makes one. The
making:

1. Make the directory with its parents, mode 0700 where it makes it.
2. Make the key, `PrivateKey::random(&mut OsRng, Algorithm::Ed25519)`,
   its comment the signer's name.
3. Write it to a fresh temporary file in the same directory, mode 0600,
   and hard-link it into place: `fs::hard_link` fails where a file is
   already there, so a second first use finds the first's key, drops
   its own, and reads that one. Remove the temporary file either way.
4. Write `signing.pub` the same way, the public half's OpenSSH line,
   `ssh-ed25519 <base64> <signer>`, mode 0644. A `signing.pub` already
   there is left as it is.
5. Read the key back through `read_key`, the same checks as any key, and
   sign.

A key file that is there is never replaced, whatever it holds: a key
others may read is refused as today, so a mistake is the person's to
see, not tailrace's to overwrite.

`signing.pub` is the line a policy lists under `[signers.<name>]`
(`listed.rs` reads `keytype base64 [comment]`), so the site tells the
person to copy it as it stands.

Rejected: making the key at bind time. 118 says "made on first use",
and a bind that makes a key would make one for a publish that then
answers that nothing is landed.

### D4. The given signer's name

The signer's name is fixed when the key is made and never moves after:
where `signing.pub` is there, its comment, the last word of its one line;
otherwise the host's name. The host's name is read with `gethostname`
through the `libc` crate, cut at its first `.` and lower-cased, so
`Laptop.local` is `laptop`. A macOS host's name can change with the
network it is on; reading it once and keeping it in `signing.pub` keeps
the name a policy lists true.

The name is resolved at bind, for publish and promote only, since the
binding's `signer` is the identity a publish signs as. An empty name, a
`gethostname` error, or a `signing.pub` with no key line or no comment
is unreadable, naming the host's name or `signing.pub`.

`libc` is a new direct dependency of `crates/tailrace`, already in
`Cargo.lock` through other crates; the call is one `unsafe` block over a
stack buffer. Rejected: running `hostname`, a tool not every host's
shell holds; a new crate such as `whoami`, a dependency for one call.

### D5. The ADR

A new ADR, the next free number when it is written (0018 at the time of
writing; two units ahead of this one also claim 0018, so the coder reads
`docs/adr/` on the branch first), records that a missing file and a
missing table bind the given profiles, how the landing is chosen, where
the given key and its name come from, and why `HEAD` and not the
working tree. It amends ADR 0013's decisions for a missing file, a
missing table and a host with no `[signing]`; 0013's status line names
the new ADR.

## Risks / Trade-offs

- **A key made where the person did not expect one.** → The site's
  signer and self-hosting pages say where it is made and how to list it,
  and only publish and promote make it.
- **A repository whose `HEAD` is on a branch from before sluice was
  set up binds a plain branch.** → Every revision on such a branch was
  landed without sluice, so a plain branch is what it is; a file's
  `[landing]` says otherwise where the person wants it.
- **Two commands racing to make the layout.** → The same as today for
  a named layout; `Layout::create` over a missing path is not changed
  here.
- **The overlapping units.** → the-publishs-host-tells-the-gate and
  status-on-any-machine also edit `bindings/mod.rs`, its tests and the
  self-hosting page; whichever lands after the other merges by hand.

## Migration Plan

None: a host with a file that names all four tables binds as before. A
host whose file left `[ledger]` out now promotes to the git ref instead
of being refused, and one whose file left `[signing]` out now signs with
a made key instead of being refused; both are what 118 asks.

## Open Questions

- The given key's path and the given signer's name are this change's
  choices; the statement says only "a host key made on first use". They
  go to the design agent with the commit.
