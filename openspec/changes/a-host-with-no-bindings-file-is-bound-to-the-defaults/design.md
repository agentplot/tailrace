# Design

## Context

ADR 0013 put the host's bindings in `$XDG_CONFIG_HOME/tailrace/config.toml`
and decided that no file is absent naming its path, that a file naming
no `[store]` or `[landing]` is absent naming the table, and that a host
naming no `[signing]` signs nothing. The ledger's unit (ADR 0016) added
`[ledger]`; the-publishs-host-tells-the-gate added `[gate]`, whose
`Teller::of` binds the host's own publish where a file holds no
`[gate]`; status-on-any-machine bound status on a host with no file by a
path of its own (`unfiled`), with `origin` as the remote of both its
landing and its ledger.

Clause 118, as the statement now reads (blueprints ab0dfea), binds a
host with no file to the layout under the user's data directory (C.2),
a host key made on first use (C.8), the branch or sluice where the
repository is set up for it (C.6, C.5), and the git ref (C.13). Its
landing remote is the checkout's `origin`, or its one remote where it
has no `origin`, and unreadable naming them where it has several and
the file names none. Its publish tells the gate as its next step, and
where the policy names no gate the host is the one 108 admits, its
acceptance claim saying so; so `beta` moves on a laptop with nothing
written. The file is for pointing a host at a registry, a key service
or a shared ledger.

What is there on this bolt:

- `bindings::read(file, home)` answers absent for a missing file and a
  `Bindings` of optional tables otherwise; `bind` answers absent for a
  missing `[store]` or `[landing]` and for publish and promote with no
  `[signing]`, and binds the given git ref for a missing `[ledger]`;
  `status` binds through `unfiled` where there is no file, and through
  `NoLedger` and `NoStore` where a file leaves those out.
- Each family gives its given binding already for the store
  (`profiles::store`), the landing (`profiles::landing::given`, with no
  remote) and the ledger (`profiles::ledger::given`); `Table::given`
  names a given table `[store] layout (given)`.
- The host key profile makes its key on first sign where it is bound
  as the given key (`HostKey::given`), and the signer family gives the
  given signer's name (`signing.pub`'s comment, else the host's name).
- The gate (`crates/tailrace-core/src/gate.rs`) refuses a version whose
  provenance its own identity made, `WhyWanting::OfferedByGate`, unless
  the requirement names that identity for both provenance and
  acceptance (`offers_and_accepts`); `count` sets `one_host` where the
  gate's identity made a provenance it counted; `gate_named` finds the
  gate's signer among the signers the acceptance policy and the
  channels' policies require, for `already` (94).

Rules this change keeps: core.1 (the bindings module names no tool, so
each default lives in its family), core.7 (a claim counts only through
a signer the policy names), core.14 (the one host is the only host that
both offers and accepts; this change adds a second way to be it),
core.19 (a broken file is still unreadable, never replaced by a
default), core.21 (the made key is the host's file, read at use, kept
nowhere of tailrace's), core.22 (sluice's file is read from a commit,
never the working tree), core.32 (verify, sync, rebuild and status make
nothing).

## Goals / Non-Goals

**Goals:**

- Every command that reads a bindings file works on a host with none,
  bound to the given bindings, by one path through the code.
- A file binds what it names and leaves the rest given.
- The given host key is made on first sign and its name is stable.
- On a host with nothing written, whose policy names no gate, one
  `tailrace publish` after a landing offers, accepts as one host and
  moves each channel that follows each publish.

**Non-Goals:**

- What runs a publish after a landing (118's last sentence).
- The given channels (`the-channels-answer-says-how-each-moves`).
- Changing a named table's behaviour or a profile's operations.
- A line in the publish's answer saying a key was made.

## Decisions

### D1. A missing file is an empty file

`read` answers a `Bindings` with every table `None` where no file is at
the path, in place of absent. `bind` then gives each `None` table its
given binding, so no file, an empty file and a file naming some tables
are one path through the code, for every command. `load`'s absent
answer where neither `XDG_CONFIG_HOME` nor `HOME` is set stays: with
neither, no given path can be placed.

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
- `profiles::landing`: sluice where `git -C <repository> rev-parse
  --verify --quiet HEAD:.config/sluice.toml` names an object, else a
  plain branch, either over the given remote (D5). An unborn `HEAD` or a
  missing path both mean a branch; a repository git cannot open is
  unreadable with git's words. The check runs through `profiles::run`'s
  `Tool`, as every git call of the landings does.
- `profiles::ledger`: `GitRef::new(repository, None)`. The given ledger
  reads and writes the repository's own refs and pushes nowhere: 118
  names a remote for the landing alone, and keeps a shared ledger for
  the file.
- `profiles::signer`: for publish and promote, the host key at
  `<config>/tailrace/signing`, `<config>` being the bindings file's own
  directory's parent (the file is `<config>/tailrace/config.toml`), bound
  as the given key (D3); for every other command, `plan(None)`, every
  signer profile verify-only.
- what tells the gate: `Teller::of` already binds `Teller::Publish`
  where the file holds no `[gate]`, so an empty `Bindings` binds it with
  no change there (D6).

`Table::given` carries the bindings file's path and is marked given, so
an answer about a given table names it as given: `<file>, [store]
layout (given)`. The marker is in `Table::at`, so no profile changes its
words.

`bind` keeps its order: every table, written or given, is read before
the layout is made, so a file that cannot bind leaves nothing behind.

### D3. The given host key

`HostKey` has a third shape beside a named key and verify-only: a given
key, `HostKey::given(signer, path)`. It differs from a named key in one
place: where sign finds no file at its path, it makes one.

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
others may read is refused as for any key, so a mistake is the person's
to see, not tailrace's to overwrite.

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
binding's `signer` is the identity a publish signs as and the gate signs
as. An empty name, a `gethostname` error, or a `signing.pub` with no key
line or no comment is unreadable, naming the host's name or
`signing.pub`.

`libc` is a direct dependency of `crates/tailrace`, already in
`Cargo.lock` through other crates; the call is one `unsafe` block over a
stack buffer. Rejected: running `hostname`, a tool not every host's
shell holds; a new crate such as `whoami`, a dependency for one call.

### D5. The given landing remote

`profiles::landing::given_remote(repository) -> Answer<Option<String>>`
lists the repository's remotes with `git -C <repository> remote`
through the landing's `Tool`, and answers:

- `origin` where it is among them;
- the one remote where there is exactly one and it is not `origin`;
- none where there are none, so the landing reads the repository's own
  refs, as a written `[landing]` with no `remote` does;
- unreadable where there are several and none is `origin`, from the
  landing's source, naming the repository and saying `the checkout has
  the remotes <a>, <b>, …, none of them origin; name one in [landing]
  remote of <file>`, the remotes in git's order (58).

It is read only where the file names no `[landing]`, the given landing
(D2). A `[landing]` the file names binds the remote it names or none,
exactly as written, so a written file means what it meant. The remote
is the checkout's: git reads it from the repository's own
configuration, which every worktree of it shares.

Rejected: the given remote for a written `[landing]` with no `remote`.
It would change what a written file binds, which 118 keeps for the
file, and leave a host no way to say "read my own refs" in a checkout
with an `origin`.

### D6. The given teller

A host with no file, or a file with no `[gate]`, binds `Teller::Publish`
through the-publishs-host-tells-the-gate's `Teller::of`, read from the
empty `Bindings` D1 gives. Nothing of the command changes: `tailrace
publish` tells the gate after the core's publish answers, signing as
the binding's signer, which on a host with no `[signing]` is the given
signer of D4. This change adds the tests that hold it for a host with
no file.

### D7. Where the policy names no gate

A policy names a gate where some signer it requires a claim of may
claim acceptance, among the signers of the acceptance policy at the
landed head and of each channel's policy the gate reads there: the same
walk `gate_named` makes, asking whether any signer, not only the gate's,
holds `acceptance` among its kinds. `names_a_gate(seams, history,
at_head, &required) -> Answer<bool>` answers it once per judgment,
before step 6, from what the gate has already read; a channel's policy
the gate cannot read answers as `gate_named` answers it.

Where the policy names no gate, the gate reads the signer it signs as,
as the policy names it for provenance, as the one host 108 admits, and
nothing else differently:

- `found`: a provenance the gate's own identity made counts, in place of
  `WhyWanting::OfferedByGate`, as `offers_and_accepts` would admit it.
- `count`: sets `one_host` as it does today, so the acceptance claim's
  body says `"offered": { "by": [<signer>], "one_host": true }` (108).
- `already`: the gate's own signer, as the policy names it, with
  `acceptance` added to what it may claim for this reading alone, so a
  second telling of the same version finds the acceptance the first made
  and answers it `already` (94), rather than attaching another.

Where the gate's own identity is not among the signers the policy names
for provenance, nothing counts its provenance, and the version is
refused naming the provenance found wanting, as today: 118 admits the
host the policy names, never one it does not.

Where the policy names a gate, nothing changes: a build host running
the gate signing as its own identity is refused as offered by the gate
(27, 96, S9), whether or not that identity is the named gate.

The admission is the gate's alone. `verify` and every other verdict
read the policy as it is written (core.7): an audience that requires an
acceptance names the signers that may make it, and so names a gate.

Rule core.14's details gain, under "Not ruled out", the host whose
policy names no gate, whose acceptance says one host (118, 108).

Rejected: a key in the policy file naming the host as gate. 118 asks
for nothing written, and the policy already names the host's signer
for provenance.

### D8. Status and the serving name through the same binding

`For::Status` binds through D1 and D2 like every other command, with
status's limits: the store given or named, opened and never made; no
signer but every profile verify-only; nothing made (60, core.32). The
status-only path for a host with no file goes: `unfiled`, `origin`,
`NoLedger`, `NoStore` where a file names no `[store]`, and the
families' `unfiled` functions. So status on a host with no file reads
the landing D5 gives and the ledger in the repository's own refs, the
chain the given gate appends to.

`landing_remote` (114), which names the repository for the serving
address, answers from the same rule: a written `[landing]`'s remote, or
D5's where the file names no `[landing]`, the file there or not.

### D9. The ADR

A new ADR, the next free number when it is written (the-publishs-host-
tells-the-gate and the-channels-answer-says-how-each-moves each claim
0019, so the coder reads `docs/adr/` on the branch first), titled "A
host with no bindings file is bound to the given profiles", records
D1 to D7: a missing file and a missing table bind the
given bindings, how the landing and its remote are chosen, where the
given key and its name come from, why `HEAD` and not the working tree,
why the given ledger has no remote, and the gate where the policy names
no gate. It amends ADR 0013's decisions for a missing file, a missing
table and a host with no `[signing]`; 0013's status line names the new
ADR.

## Risks / Trade-offs

- **A key made where the person did not expect one.** → The site's
  signer and self-hosting pages say where it is made and how to list it,
  and only publish and promote make it.
- **A build host with nothing written accepts its own work.** → Only
  where the repository's policy names no gate, which 118 admits; a team
  that separates its gate names it in the policy, and from then on the
  build host is refused (27, 96, S9). The acceptance claim says one host,
  so a channel's policy may require a separated one (108).
- **A repository whose `HEAD` is on a branch from before sluice was
  set up binds a plain branch.** → Every revision on such a branch was
  landed without sluice, so a plain branch is what it is; a file's
  `[landing]` says otherwise where the person wants it.
- **A checkout with several remotes and no `origin` is unreadable for
  every command with no file.** → 118 asks for that answer; it names the
  remotes and `[landing] remote`, the one line that fixes it.
- **The given channels.** → The-channels-answer-says-how-each-moves
  gives `beta` and `stable` a required acceptance by the `[signers]`
  that may claim acceptance; where the policy names no gate that list is
  empty. This change moves only channels whose policy it can count; the
  pairing is named to the design agent with the commit.
- **Two commands racing to make the layout.** → The same as for a named
  layout; `Layout::create` over a missing path is not changed here.
- **The overlapping units.** → the-publishs-host-tells-the-gate,
  status-on-any-machine and the-channels-answer-says-how-each-moves also
  edit `bindings/mod.rs`, its tests, `gate.rs` or the self-hosting page;
  this unit comes after the first, and merges the others by hand where
  they land first.

## Migration Plan

None for a host whose file names every table: it binds as before. A
host whose file left `[signing]` out now signs with a made key instead
of being refused, one whose file left `[store]` or `[landing]` out now
binds the given ones instead of being answered absent, and a host whose
file holds no `[gate]` tells its own gate; each is what 118 asks. A
repository whose policy names no gate and whose host publishes now has
its versions accepted as one host where before they were refused as
offered by the gate.

## Open Questions

- The given key's path and the given signer's name are this change's
  choices; the statement says only "a host key made on first use".
- The given ledger's remote: 118 names the landing's remote alone, and
  this change gives the ledger none.
- Where the policy names no gate, what a given channel's required
  acceptance names (the pairing with the-channels-answer-says-how-each-
  moves).

All three go to the design agent with the commit.
