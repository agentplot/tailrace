## Context

The statement, blueprints `2287669`, `design/timber/requirements.md`:

- **B.7** (88 to 91): a target answers `desired`, the digest an
  environment's channel names or that it is empty; `current`, the digest
  it runs or that it cannot be read, with the reason; `apply`, given a
  version the reconciler has verified (42), the environment converged
  or what stopped it; `report`, what it runs and when it last converged,
  recorded where every asker reads it (43).
- **136**: the policy seam's third operation, `environments`: each
  environment the policy declares, with its target, the channel it
  follows, whether it waits and whom it waits on.
- **42, 45, 119**: a reconciler reads the version's bytes by digest and
  verifies the claims it is told to require before it applies anything;
  it acts with the environment's own credentials in the environment's
  own place, and timber never holds a credential that reaches a target;
  it leaves a marker of what it applied where the target can hold one,
  and reports unread with the reason where it can hold none.
- **C.10**: an environment names its channel, `policy` and `after`, and
  "a target's own settings, a cache's address among them, sit in the
  target's table and nowhere else"; **92**: the repository's authors
  bind the targets, any number at once.
- **C.12**, the Nix binary cache row: "push: the version's store paths
  pushed to the cache"; the Nix fleet's push profile is the clan push
  profile's, its pull profile phase 2's.
- **15, I3**: what is offered is what the landing's verdicts name, and
  nothing after the offer builds.
- **Section 10**: "How does a Nix closure, thousands of digests with
  references among them, map onto artifacts and the cache without a
  reference-aware removal (52, 54)?" A question the model answers, not a
  requirement; the snix and casita note (`models/snix-and-casita.md`,
  points 2 and 5) reads the answer the statement already implies: the
  Nix cache holds the bytes, timber's store the pointer and the claims.

The bolt, `models/phase-1-bolts.md`, Bolt 6: two units, this one and the
clan push profile, usable when swancloud's machines are deployed by
`timber apply` from the deployer's host. This unit owns
`profiles/target/nix_cache/` and builds "the version's store paths
pushed to the fleet's cache (C.12), the closure named as the version's
artifact for its machine", proved when "a closure built on one host is
realised on another from the cache alone".

What the tree holds:

- `crates/timber-seams/src/target.rs`: the `Target` trait, `Environment`
  (name, channel, waits), `Verified` (index and version, made by the
  core alone), `Running`. `Seams.targets` is a map from
  `EnvironmentName` to a target, and no binding fills it.
- `crates/timber-seams/src/policy.rs`: `Policy` has `policy_for` and
  `channels`. The policy file profile reads each environment's
  `channel`, `policy`, `after` and `approvers`, makes a channel for a
  manual or after environment, and skips `target` unread.
- `crates/timber-core/src/status.rs`: `environments` answers unreadable
  naming the environments, asking no target.
- `crates/timber-core/src/ledger.rs`: `channel(ledger, id)` reads a
  channel's chain and its head promotion; `judge(seams, required,
  index)` judges an index's claims against a policy, refused naming the
  first found wanting. `promote` reads the governed branch's landed
  head, the declared component, the channel listed and its policy, in
  that order; apply reads them the same way.
- The conformance target group holds three scenarios over the fake
  target; the register names 41, 42, 44, 45, 97, 98, 119, 126, 136 and
  two parts of 132 and 134 with "phase 2 · the first target", written
  when phase 1 left every target to phase 2, before the user put bolt 6
  in it (2026-10-26).
- swancloud today: its releaser builds each environment's top-level
  closure and pushes it to attic's `swancloud` cache on srv with
  `swancloud-cache-push`, the pushing machine's own token in a
  throwaway attic configuration (swancloud `docs/architecture/fleet.md`,
  "The fleet's cache"; `deploy.10`). Each environment is versioned under
  its own tag prefix, `<environment>-v`, which is a component (123).

## Goals / Non-Goals

**Goals:**

- The policy answers its environments, each with its target's profile
  and settings (136, C.10).
- One core path that applies one environment, and judges before it
  applies (42, core.29), which every target profile is reached through.
- A version's closures pushed to the fleet's attic cache from the host
  that holds them, with the environment's own credential (45, C.12).
- The suite holding a target that can keep no marker to 119.

**Non-Goals:**

- The clan push profile, what runs apply when a channel moves, the
  pull profile, the roster, environments in status (the proposal's out
  of scope, each with what it waits on).
- Building, substituting or copying a closure between Nix stores
  anywhere but by the cache's own push.

## Decisions

### D1. This change builds the target machinery once, and the clan push profile takes it

Neither unit of bolt 6 owns the environments answer, the binding of
targets or the core's apply, and both need all three. This change, the
bolt's first, builds them with the first profile that uses them, as
`cargos-registry` built the core's layout loop that npm's unit then
closed its layouts by. The clan push profile's change takes this
branch's work before its code reads any of it, and writes none of its
own.

Not taken: a core unit of its own before both (the bolt carves none, and
both units would wait on a third nobody planned); the clan push profile
owning it (it comes second, and the cache would have no way to be
applied until it landed).

### D2. The policy answers environments with their target's profile and settings

```rust
fn environments(&self, revision: &Revision) -> Answer<Vec<DeclaredEnvironment>>;

pub struct DeclaredEnvironment {
    pub name: EnvironmentName,
    pub channel: ChannelId,         // its made channel where it waits or comes after (41, C.10)
    pub waits: bool,
    pub approvers: Vec<SignerId>,   // whom it waits on, where it waits (97)
    pub target: Option<TargetTable>,
}

pub struct TargetTable {
    pub profile: TargetProfile,     // the table's one key
    pub settings: serde_json::Value, // that key's table, as written
}
```

The policy file reads `target` as a table of exactly one key, the
target's profile, whose value is a table: `target.nix_cache = { … }`,
or `[environments.<component>.<name>.target.nix_cache]`. None or two
keys, or a value that is no table, is unreadable naming the
environment's table, as the bindings file reads a seam's table by its
one key. An environment that names no `target` is listed with none,
since its channel stands whatever runs it; apply answers it absent
naming `target`. The settings are carried as written and read by the
target alone: the policy profile is the only reader of the file's keys
(core.43), and a target reads its own settings from the policy's
answer, which is where C.10 puts them. The answer is ordered by
component and name; an empty list, never absent, where the file
declares none.

Not taken: the target's settings flattened beside `channel` (C.10 puts
them in the target's table, and the profile already refuses them
there); the core reading `target` itself (core.43).

### D3. Targets are bound by profile, as projections are by ecosystem

`Seams.targets` becomes `BTreeMap<TargetProfile, Box<dyn Target>>`, and
`Environment`, the value a target is handed, gains `settings`. The
binding binds every target profile built, each handed a store binding
of its own, opened and never made, and the checkout's path, as Cargo's
projection is bound; the environment's table picks among them. An environment's name is unique
only within its component (125), and swancloud declares a cache
environment in each machine's component, so a map by name cannot hold
them; and a profile bound once serves every environment that names it,
each with its own settings.

Not taken: binding each environment's target before the core runs (the
bindings would read the policy at the landed revision themselves,
which only the core does, core.22); a key of component and name (it
binds a profile per environment for no gain).

### D4. desired is handed the channel's head

```rust
pub type Head<'a> = &'a dyn Fn() -> Answer<Digest>;
fn desired(&self, environment: &Environment, head: Head<'_>) -> Answer<Digest>;
```

`Head` answers the digest of the index the head promotion of the
environment's channel moved to, read from the chain (core.50), and
absent naming the channel where the chain holds none (31). A push
target answers it; a pulling target, the Nix fleet's pull profile in
phase 2, answers what its own reading of the channel gives, which is
why the operation stays the target's (88, 44). It mirrors `Miss`, the
core's render a projection's serve is handed.

Not taken: the core answering desired and never asking the target (88
is a target's operation); the profile reading the ledger itself (a
profile reaching a second seam and parsing promotion claims, the core's
work); the head carried as a field of `Environment` (read for every
environment made, asked or not).

### D5. Apply judges, then hands over

`crates/timber-core/src/apply.rs`, `apply(seams, history, ask)`, with
`ApplyAsk { branch, component, environment }`, in this order:

1. The governed branch's head, landed (`Landed::check`), the declared
   component (`Declared::read`, `chosen`), as promote reads them.
2. `environments` at that revision; the environment of that name in that
   component, absent naming both where there is none; its target table,
   absent naming the environment's `target` where it names none; the
   bound profile, absent naming it where none is bound.
3. The channel's chain (`ledger::channel`); an empty channel answers
   absent naming the channel, the target asked nothing (31, 88). `Head`
   answers the head's index.
4. `desired` and `current` of the target. A desired other than the
   chain's head is refused naming both and nothing is applied (40,
   core.16). Where current answers that digest, apply is not asked and
   the answer says the environment runs it already.
5. The policy for the channel at the landed revision, and `judge` of the
   head's index under it: refused naming the first claim found wanting,
   the target asked nothing (42, core.29). Then `Verified::new(index,
   version)`, the version the head promotion names, and `apply`.
6. `report` after the apply. The answer, `Applied`, names the
   environment, its component, its channel, the version and index
   applied, what current answered before, and the report or its unread
   answer, each in the one form every surface renders (57).

Apply moves no name, and makes no store: the store is opened and never
made, the ledger read. Here it signs nothing and is bound as status is,
with the targets beside; the clan push profile makes it sign one report
claim as the host's signer once an apply converges and tell the host's
own gate, so its binding takes the host's signer there, and `Applied`
is a struct its change adds fields to (91, core.8, core.27; ADR 0020).
An apply that stops at the target is that target's answer, its words
verbatim (core.19).

### D6. A closure is one artifact naming its roots

A machine's build task leaves one file naming its closure's roots: each
line one absolute store path, `<store dir>/<32 characters of Nix's
base32>-<name>`, as `nix build --print-out-paths` prints them. The
component declares it as an output and the file projection packs it as
it packs any file, so its digest, its provenance and the landing's
verdicts name it as they name any output. The environment's target
table names which of the component's outputs are closures. Their form
is read by `profiles/target/nix_closure.rs`, beside the target
family's `mod.rs`: it reads an artifact's bytes into its roots, refusing
a file with no line, an empty or duplicated line, or a line that is no
store path, naming the artifact and the line. The clan push profile
reads what a machine should run through it too.

This is the model's answer to section 10's question. A closure is one
artifact, small and named by its digest; its thousands of store paths,
and the references among them, are the Nix binary cache's, whose
collection is its own. The store holds none of them, so its removal of
untagged objects (54) never leaves half a closure, and nothing timber
names depends on a store path the cache may collect: a machine that
misses one builds it, as swancloud's `fallback` does today. The bytes
are trusted as the snix note reads it: timber's claims say which
version, the cache's signature on each path says the bytes (12). Rule
core.70 holds it for every Nix profile, this one, the clan push
profile's and the pull profile's to come.

Not taken: each store path's NAR as an artifact or a cache entry
(thousands of objects per version, removed one at a time by a store that
knows nothing of references, so a closure could be left part held); a
projection of a Nix ecosystem (C.11 has no Nix row, and there is
nothing to pack but the roots); the roots' NAR hashes in the artifact (a
closure evaluated on one host and built on another, as swancloud builds
srv's, has no NAR hash where the task runs, and the cache signs the
bytes it serves).

### D7. The Nix cache target pushes with attic, and its table holds no credential

```toml
[environments.swancloud-srv.cache]
channel = "beta"
target.nix_cache = { attic = "fleet:swancloud", closures = ["swancloud-srv"] }
```

`attic` is the cache as attic's own command names it,
`<server>:<cache>`; `closures`, one or more output names. Any other key,
an empty `closures`, or an `attic` that is not `<server>:<cache>` is
unreadable naming the environment and the key, read at every operation
that reads the settings, so a token can be written nowhere timber reads
(45, core.21). The push is `attic push <server>:<cache> <roots…>`: attic
reads its servers and the environment's token from its own
configuration, `$XDG_CONFIG_HOME/attic/config.toml` as the host keeps it,
at the push, and pushes each root with its closure, skipping what the
cache or an upstream it names already holds. timber passes no token on
the command line or in the environment, and reads nothing of attic's
configuration. attic's own words on a failure are the apply's answer,
unreadable, verbatim.

Not taken: `nix copy --to <store address>` (any binary cache, but the
fleet's is attic, which takes an upload through its own client); a
second key for it now (a later key of the same table, when a fleet binds
a cache attic is not); a `token` or `token_file` setting (a credential
in the repository's file, or one timber reaches for the target, 45).

### D8. Apply pushes only what this host's store holds

Apply reads the version's index through the store by digest, finds each
output `closures` names among its artifacts, the file projection's
artifact of that name, and reads its bytes by digest, checked (42,
core.4). An output named that the index carries no artifact of is
refused naming the output and the version. Before attic runs, every root
is checked valid in this host's Nix store (`nix-store --check-validity`);
a root it does not hold refuses the apply naming the root and the host,
and nothing is pushed. Apply never builds, substitutes or copies a root
into this host's store (15, I3): the host that applies is a host that
built, the deployer's host for the Macs' closures, srv for srv's.

Not taken: realising a missing root from another cache first (a fetch
the apply would push straight back, and on a fresh host a build); a
partial push of the roots this host holds (an environment half
converged, with no marker to say so).

### D9. A Nix binary cache keeps no marker: current and report are unread

A Nix binary cache holds store paths by their hash and their narinfo,
and no name that can be moved: there is nowhere to leave a marker of
the digest applied (119). `current` and `report` answer unreadable, the
profile's words saying so and naming the cache, never the digest last
pushed and never converged (89, 98). Apply runs whenever asked; attic
skips what the cache holds, so an apply of a version already pushed
uploads nothing.

So the target group's rig declares whether its target holds a marker,
as a store's rig declares whether its names take a condition (core.49).
"Converged and reported" runs where it does; where it does not, the
new scenario "applied where no marker is held" asks that current and
report after an apply answer unreadable naming why, never the digest
applied.

Not taken: answering current with the head where the cache holds every
root of its closures (it says what the cache holds, not what was
applied, and current is handed no head); a marker kept as a store path
of its own (a store path is named by its content and cannot be found by
a name).

### D10. A target refuses a settings key it does not read

The target group gains "a table with a key the target does not read":
the rig hands an environment whose settings hold a key its profile does
not read, and apply, current and desired answer unreadable naming it,
with nothing applied (45, 58). The fake target reads the keys a test
lets it read and refuses the first other. It holds every target to
reading its own table and no more, which is how 45's "the environment's
own place" is kept: the credential is never in the table.

### D11. The register after this unit

In the commit whose scenario first cites each: 136 leaves with the
policy group's environments scenario, 42 with the apply group's, 45 and
119 with the target group's. "phase 2 · the first target" is no longer
true of the rest, so they are named again: 41 and 97, S6's environment
half and the bolt's S16, by `bolt 6 · the clan push profile`; 44 and
126, the pull
profile and the roster, by `phase 2`; 98 and the parts of 132 and 134,
environments in status and the page, by `phase 2 · environments in
status`, which no unit of bolt 6 carves.

### D12. The proof

`crates/timber/tests/nix_cache.rs`, behind the `tools` feature:

- An attic server started for the test, monolithic, its database and
  storage in the test's directory, its token signing secret made by the
  test; a cache made in it; a token minted with `atticadm make-token`
  for push and pull on that cache; an attic configuration under a
  `XDG_CONFIG_HOME` of the test's own naming the server and holding the
  token, the environment's own place, which the test hands attic's
  process and nothing of timber's.
- A closure of two store paths, a root whose output names a leaf, built
  by the test with the host's Nix, the build host's part.
- A version published into a layout store, the closure file packed by
  the file projection, its index naming it; apply through the profile
  with `Verified` as the core makes it; the run's command line and the
  profile's own state hold no token.
- Another host's part: a fresh Nix store in the test's directory
  realises the root from the cache alone, `nix copy --from` the cache's
  address with the cache's public key as the only key trusted, and holds
  the leaf after.
- The target group runs over the profile's rig: no marker, its unread
  words, a settings key it does not read.

attic's client and server come from `devenv.nix` (core.26), and Nix is
every host's own, which `devenv` itself runs on. The test needs no
credential a person signs in for (core.38).

## Risks / Trade-offs

- **[Trait changes cross both units]** `Head`, `environments` and the
  binding by profile change seams the clan push profile codes against →
  this change lands them before that unit's code; its construct reads
  them from this change's specs.
- **[A cache environment's status is always unread]** → it is the
  truth 119 asks for; apply is idempotent through attic's own skip, and
  whether a machine can realise its closure is the clan push profile's
  and the machine's report.
- **[The host that applies may not hold the closure]** → refused naming
  the root and the host; the operator applies from the host that built,
  as swancloud's releaser pushes from it today.
- **[attic's configuration is the host's]** → timber writes none; the
  self-hosting guide and the targets page say where attic reads it.
- **[The fleet's cache collects by age]** → a released closure it has
  collected is built again by the machine that misses it (54); the
  fleet's retention is attic's and swancloud's.

## Migration Plan

swancloud adopts it after this unit and the clan push profile land: its
machine tasks leave a closure file as a declared output, its policy file
declares each machine's component with its cache environment, and its
releaser's push retires once `timber apply` pushes from the same host.
Nothing in timber migrates: no environment is declared anywhere yet.
