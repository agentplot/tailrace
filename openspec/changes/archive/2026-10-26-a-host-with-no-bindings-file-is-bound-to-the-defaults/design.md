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

Clause 118, as the statement now reads (blueprints ab0dfea, 85bbf54,
b64d367), binds a host with no file to the layout under the user's data
directory (C.2), a host key made on first use (C.8), the branch or
sluice where the repository is set up for it (C.6, C.5), and the git
ref (C.13) on the landing's own remote, pushed to and fetched from it
as the branch is, so two machines of one repository read one ledger
with nothing written, a repository with no remote keeping its ledger
where it keeps its branch. Its
landing remote is the checkout's `origin`, or its one remote where it
has no `origin`, and unreadable naming them where it has several and
the file names none. Its publish tells the gate as its next step, and
where the policy names no gate the host is the one 108 admits, its
acceptance claim saying so; so `beta` moves on a laptop with nothing
written. The file is for pointing a host at a registry, a key service
or a shared ledger. C.10 (blueprints 061bfa8) places that admission in
the policy: where the file names no signer that may claim acceptance,
the policy answers the host's own signer, the one it names for
provenance, as the repository's gate for acceptance and for the given
channels' promotion, its acceptance claim saying one host, and the gate
reads this from the policy's answer and decides nothing of it (core.7).
C.10 at blueprints 53f6d44 adds that where the file names several
signers for provenance and none for acceptance, 108 admits no host,
nothing is accepted, and the refusal says so in those words: several
publishers and no gate, name an accepting signer (58). The design
agent's batch 18 rulings place the given key at
`$XDG_DATA_HOME/tailrace/host/signing.key`, its public half beside it,
and name its signer by the host's short name at first use, fixed in the
public half.

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
  given signer's name (the public half's comment, else the host's
  name); `signer::given` places the key at `signing` beside the
  bindings file, which D2 moves to the data directory.
- The gate (`crates/tailrace-core/src/gate.rs`) refuses a version whose
  provenance its own identity made, `WhyWanting::OfferedByGate`, unless
  the requirement names that identity for both provenance and
  acceptance (`offers_and_accepts`); `count` sets `one_host` where the
  gate's identity made a provenance it counted; `gate_named` finds the
  gate's signer among the signers the acceptance policy and the
  channels' policies require, for `already` (94). This unit's first
  build added `names_a_gate` and `signer_named` there, with tests in
  the core's gate tests and a scenario in the conformance suite's gate
  group; D7 takes them out.
- The policy file profile (`profiles/policy/file/`, as
  the-channels-answer-says-how-each-moves leaves it) reads `[signers]`
  once per revision into the signers every audience's claims name; a
  given or made channel's policy requires acceptance by, and is promoted
  by, every signer `may_claim` finds that may claim acceptance
  (`channel_policy`).

Rules this change keeps: core.1 (the bindings module names no tool, so
each default lives in its family), core.7 (a claim counts only through
a signer the policy names: the given gate is a signer the policy's
answer names), core.14 (the one
host is the only host that both offers and accepts),
core.19 (a broken file is still unreadable, never replaced by a
default; the several-publishers refusal carries the policy file's own
words), core.21 (the made key is the host's file, read at use, kept
nowhere of tailrace's), core.22 (sluice's file is read from a commit,
never the working tree), core.32 (verify, sync, rebuild and status make
nothing).

## Goals / Non-Goals

**Goals:**

- Every command that reads a bindings file works on a host with none,
  bound to the given bindings, by one path through the code.
- A file binds what it names and leaves the rest given.
- The given host key is made on first sign and its name is stable.
- On a host with nothing written, whose policy lists its key for
  provenance only, one `tailrace publish` after a landing offers,
  accepts as one host and moves the given `beta`.

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
- `profiles::ledger`: `given(repository, remote)`, the git ref,
  `GitRef::new(repository, remote)`, where `remote` is the landing's
  remote as the host binds it: a written `[landing]`'s `remote`, or
  D5's where the file names no `[landing]`. With a remote, the git ref
  profile does what it already does for `[ledger] git_ref = { remote }`:
  each read fetches the channel's chain into the repository's copy
  first, each append is a push that only fast-forwards, and a remote
  that does not answer leaves the chain answered from the copy as last
  fetched, saying when (C.13, 60). With none, the repository's own refs
  are the ledger and nothing is pushed, the ledger kept where the
  branch is. So `bind` reads the landing's remote before it binds a
  given ledger, and the given ledger never names a remote the landing
  does not. A written `[ledger]` binds what it names, its `remote` or
  none, exactly as written.
- `profiles::signer`: for publish and promote, the host key at
  `<data>/tailrace/host/signing.key`, `<data>` being the same data
  directory the given layout is under, passed in, bound as the given key
  (D3); for every other command, `plan(None)`, every signer profile
  verify-only. The key is the host's and not the configuration's, so it
  sits beside the host's other made state and not beside the bindings
  file (the design agent's batch 18 ruling).
- what tells the gate: `Teller::of` already binds `Teller::Publish`
  where the file holds no `[gate]`, so an empty `Bindings` binds it with
  no change there (D6).

Rejected for the ledger: the git ref with no remote. Two machines of
one repository would each keep a chain of their own, a second
machine's status would show none of the first's promotions, and 118
asks for one ledger read by both with nothing written.

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

1. Make the directory, `<data>/tailrace/host`, with its parents, mode
   0700 where it makes it.
2. Make the key, `PrivateKey::random(&mut OsRng, Algorithm::Ed25519)`,
   its comment the signer's name.
3. Write it to a fresh temporary file in the same directory, mode 0600,
   and hard-link it into place: `fs::hard_link` fails where a file is
   already there, so a second first use finds the first's key, drops
   its own, and reads that one. Remove the temporary file either way.
4. Write `signing.key.pub` the same way, the public half's OpenSSH
   line, `ssh-ed25519 <base64> <signer>`, mode 0644: the key's path with
   `.pub` added, as ssh-keygen names it and `host_key::public_half`
   already does. A `signing.key.pub` already there is left as it is.
5. Read the key back through `read_key`, the same checks as any key, and
   sign.

A key file that is there is never replaced, whatever it holds: a key
others may read is refused as for any key, so a mistake is the person's
to see, not tailrace's to overwrite.

`signing.key.pub` is the line a policy lists under `[signers.<name>]`
(`listed.rs` reads `keytype base64 [comment]`), so the site tells the
person to copy it as it stands.

Rejected: making the key at bind time. 118 says "made on first use",
and a bind that makes a key would make one for a publish that then
answers that nothing is landed.

### D4. The given signer's name

The signer's name is fixed when the key is made and never moves after:
where `signing.key.pub` is there, its comment, the last word of its one
line; otherwise the host's short name. The host's name is read with `gethostname`
through the `libc` crate, cut at its first `.` and lower-cased, so
`Laptop.local` is `laptop`. A macOS host's name can change with the
network it is on; reading it once and keeping it in `signing.key.pub` keeps
the name a policy lists true.

The name is resolved at bind, for publish and promote only, since the
binding's `signer` is the identity a publish signs as and the gate signs
as. An empty name, a `gethostname` error, or a `signing.key.pub` with no
key line or no comment is unreadable, naming the host's name or
`signing.key.pub`.

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

### D7. Where the policy file names no gate

The given gate is the policy file profile's, 118's default placed at
the policy seam (C.10), and never the gate's. When the profile reads
`[signers]` at a revision, and no signer it names holds `acceptance`
among its kinds while exactly one holds `provenance`, it adds
`acceptance` to that one signer's kinds in what it answers. Nothing
else of the answer changes, and the file is never rewritten.

Since every audience's claims name signers from that one reading, every
answer follows from it with no other code:

- `[acceptance]`'s required provenance names the signer with
  `provenance` and `acceptance`, so the gate's `offers_and_accepts`
  admits a provenance the gate's identity made under that name, and
  `count` sets `one_host`: the acceptance claim's body says
  `"offered": { "by": [<signer>], "one_host": true }` (108).
- A given or made channel's policy requires acceptance by, and is
  promoted by, the signers that may claim acceptance (`channel_policy`),
  so `beta` and `stable` with no line written name the same signer as
  their gate and their promoter (32, C.10).
- `gate_named` finds that signer for `already`, so a second telling of
  the same version answers the acceptance the first made (94).
- `verify` reads the same answer (core.31).

Where `[signers]` names a signer that may claim acceptance, the answer
is the file as written: a build host running the gate signing as its
own identity is refused as offered by the gate (27, 96, S9). Where it
names none that may claim provenance, or the revision holds no file,
there is no signer to name, and the answer is the file as written. Where
it names several that may claim provenance, D7a answers.

The gate's code is as the units before this one leave it: this unit
takes out `names_a_gate`, `signer_named` and the `names_a_gate` field
its first build added to `gate.rs`, their tests in
`crates/tailrace-core/tests/gate.rs`, and the gate group's "the policy
names no gate" scenario, which judged over a fake policy the default
does not reach. The fake policy answers what it is scripted, so a test
of the publish over the fakes scripts the host's signer with
`provenance` and `acceptance`, as the file profile answers it.

Rule core.14's details gain, under "Not ruled out", the host the policy
file answers as the gate where `[signers]` names no signer that may
claim acceptance, its acceptance claim saying one host (118, C.10).

Rejected: the gate reading a policy that names no gate as naming its
own identity. C.10 places the default in the policy's answer so the
gate decides nothing of whom the policy names (core.7).

### D7a. Several publishers and no gate

Where `[signers]` at the revision names no signer that may claim
`acceptance` and two or more that may claim `provenance`, 108 admits no
host (C.10). The profile answers `policy_for` for `Audience::Acceptance`,
and for each given or made channel, whose gate is the acceptance's, as

```rust
Not::refused(
    self.source(),
    Why::Wanting { kind: ClaimKind::Acceptance, check: None, signers: vec![] },
    Some("several publishers and no gate, name an accepting signer".into()),
)
```

`Why::Wanting` already says "no attestation of `kind` counts under the
policy, which lets `signers` make it", and here the policy lets no
signer make one; the words are the profile's own, carried verbatim
(58, 59, core.19). A channel declared by hand, and `[layout]`, answer as
written: they name their own signers. `channels` is unchanged, so
`status` still lists every channel.

Every reader follows with no other code: the gate's `required_at` for
acceptance answers the refusal before any claim is read, so nothing is
accepted and no channel moves, and the publish's `gate` member carries
the refusal with its words; `verify` under acceptance shows the same
answer (core.31).

Rejected: a new `Why` variant, which would change the seam's shared type
for one sentence the existing variant already carries. Rejected: the
gate telling several publishers apart, which C.10 keeps out of the
gate's code (core.7).

### D8. Status and the serving name through the same binding

`For::Status` binds through D1 and D2 like every other command, with
status's limits: the store given or named, opened and never made; no
signer but every profile verify-only; nothing made (60, core.32). The
status-only path for a host with no file goes: `unfiled`, `origin`,
`NoLedger`, `NoStore` where a file names no `[store]`, and the
families' `unfiled` functions. So status on a host with no file reads
the landing D5 gives and the ledger on that same remote, fetched into
the repository's copy, the chain the given gate appends to; the fetch
writes the copy and nothing else (131). status-on-any-machine's
two-machine test, whose second machine has no file and one remote
`origin`, binds through this path to the ledger on `origin`, as it
did through the path this one replaces, and stands as written.

`landing_remote` (114), which names the repository for the serving
address, answers from the same rule: a written `[landing]`'s remote, or
D5's where the file names no `[landing]`, the file there or not.

### D9. The ADR

ADR 0021, "A host with no bindings file is bound to the given
profiles" (0018 is status-on-any-machine's, 0019 and 0020 the units'
before this one), records D1 to D7: a missing file and a missing table
bind the given bindings, how the landing and its remote are chosen,
where the given key and its name come from, why `HEAD` and not the
working tree, why the given ledger is on the landing's remote, the given
gate in the policy file's answer and the refusal where several publish
and none accepts, citing C.10. It amends ADR 0013's decisions
for a missing file, a missing table and a host with no `[signing]`;
0013's status line names it.

## Risks / Trade-offs

- **A key made where the person did not expect one.** → The site's
  signer and self-hosting pages say where it is made and how to list it,
  and only publish and promote make it.
- **A build host with nothing written accepts its own work.** → Only
  where the repository's policy file names no signer that may claim
  acceptance and one for provenance, which 118 and C.10 admit; a team
  that separates its gate names it in `[signers]`, and from then on the
  build host is refused (27, 96, S9). The acceptance claim says one host,
  so a channel's policy may require a separated one (108).
- **A repository whose `HEAD` is on a branch from before sluice was
  set up binds a plain branch.** → Every revision on such a branch was
  landed without sluice, so a plain branch is what it is; a file's
  `[landing]` says otherwise where the person wants it.
- **A host with nothing written pushes its ledger to the landing's
  remote.** → 118 asks for it. The push carries only
  `refs/tailrace/ledger/...`, fast-forward only, with the host's own git
  credentials at the moment of use, as the branch's push does
  (core.21); a remote that refuses it is answered with git's words and
  never retried (core.6). A team that wants the chain elsewhere names
  `[ledger]`.
- **The given ledger on a host that only reads.** → Status and verify
  fetch the chain into the repository's copy and write nothing else
  (131, core.32); a remote that does not answer leaves the chain as last
  fetched, saying when.
- **A checkout with several remotes and no `origin` is unreadable for
  every command with no file.** → 118 asks for that answer; it names the
  remotes and `[landing] remote`, the one line that fixes it.
- **A repository with several publishers and no gate.** → Nothing is
  accepted, and every answer says so in the policy's words, naming the
  one fix: an accepting signer (D7a).
- **Two commands racing to make the layout.** → The same as for a named
  layout; `Layout::create` over a missing path is not changed here.
- **The overlapping units.** → the-publishs-host-tells-the-gate,
  status-on-any-machine and the-channels-answer-says-how-each-moves also
  edit `bindings/mod.rs`, its tests, the policy file profile, the
  command-line spec or the self-hosting page; this unit comes after all
  three, and takes in each one's branch, or the bolt once it has landed
  there, before building on it.

## Migration Plan

None for a host whose file names every table: it binds as before. A
host whose file left `[signing]` out now signs with a made key instead
of being refused, one whose file left `[store]` or `[landing]` out now
binds the given ones instead of being answered absent, one whose file
names a `[landing]` with a `remote` and no `[ledger]` now keeps its
ledger on that remote, its first append pushing the chain the
repository holds, and a host whose file holds no `[gate]` tells its own
gate; each is what 118 asks. A
repository whose policy names no gate and whose host publishes now has
its versions accepted as one host where before they were refused as
offered by the gate, where its policy file names one signer for
provenance and none for acceptance; one whose file names several for
provenance and none for acceptance is refused in words where before it
was refused as offered by the gate or wanting acceptance.

## Open Questions

None.
