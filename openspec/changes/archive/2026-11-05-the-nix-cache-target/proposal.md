## Why

No version timber holds can reach a target yet. The target seam (B.7)
has its trait, its fake and its group, and nothing more: the policy seam
answers no environment (136), the binding holds no target, no core path
verifies a version and hands it to a target (42, rule core.29), and no
target profile exists. Bolt 6 (`agentplot/blueprints:design/timber/models/phase-1-bolts.md`,
"the Nix fleet, pushed") starts with the Nix cache target: a version's
Nix store paths pushed to the fleet's binary cache as a target, the
closure named as the version's artifact for its machine, so a closure
built on one host is realised on another from the cache alone, and
timber holds no credential that reaches the cache (45, 88 to 91, 119,
B.7, C.12, as the statement has them at blueprints `2287669`,
`design/timber/requirements.md`).

It is the first target, so it is also the first change that needs an
environment read from the policy, a target bound by the repository, and
the core's apply. The bolt carves no unit for that machinery and its
second unit, the clan push profile, needs the same; this change builds
it once, and the clan push profile builds on it (design D1).

## What Changes

- **The policy answers its environments** (136). The policy seam gains
  `environments`: given a landed revision, every environment the policy
  declares, each with the channel it follows (its made channel where it
  waits or comes after another, 41, C.10), whether it waits, whom it
  waits on where it does, and its target: the profile the environment's
  `target` table names by its one key, and that key's table carried as
  written, the target's own settings (C.10). The policy file answers it;
  the fake policy answers what a test sets. 136 leaves the register.
- **A target is bound by its profile.** The binding holds targets by the
  profile each implements, as it holds projections by ecosystem, and
  the core hands a target the environment with its settings; every
  target profile built is bound, and the repository's environments pick
  among them (92). A target table naming a profile no binding holds is
  absent naming it.
- **desired is handed the channel's head.** A target's `desired` takes
  the head the core read from the channel's chain (core.50), as a
  projection's `serve` takes the core's render on a miss: a push target
  answers it, a target that pulls answers what it reads itself (88, 44).
- **The core applies one environment: `timber apply`.** At the governed
  branch's landed head, the core finds the environment, reads its
  channel's head from the ledger, asks the target what it should run and
  what it runs, judges the claims the channel's policy requires of the
  head's index, and only then hands the target the version, verified,
  to apply; it answers what it applied and the target's report, or the
  first claim found wanting with the target asked nothing (40, 42, 88 to
  91, rules core.4, core.16, core.29). It moves no name.
  `timber apply <environment>`, with `--component` where the repository
  declares components and `--json`, renders the answer.
- **A closure is named as an artifact.** A machine's build task leaves a
  file naming its closure's root store paths, one per line, declared as
  an output and packed by the file projection; the environment's target
  table names which outputs are closures. Their form is read by one
  module the Nix target profiles share, so the clan push profile reads
  the same artifact without calling this profile (rule core.2).
- **The Nix cache target**, `crates/timber/src/profiles/target/nix_cache/`
  (C.12's Nix binary cache row), pushing with attic, the fleet's cache:
  - its table names the cache as attic names it and the closure outputs,
    and nothing else; any other key is unreadable naming it, so no
    credential sits in the repository's file (45, rule core.21);
  - `desired` answers the channel's head the core hands it;
  - `apply` reads the version's index and each closure artifact through
    the store by digest, refuses a root this host's Nix store does not
    hold naming the root and the host, never building or fetching it
    (15, I3), and pushes the roots with `attic push`, which reads the
    environment's own token from attic's own configuration at the push
    and pushes each root's closure (45);
  - `current` and `report` answer unread, saying a Nix binary cache
    holds store paths by their hash and keeps no name a marker could be
    held under (119, 89, 98).
- **The suite holds targets to 45 and 119.** The target group's rig
  declares whether its target holds a marker: "converged and reported"
  runs where it does, a new scenario where it does not; a new scenario
  holds a target to refusing a table key it does not read; `desired` is
  asked with the core's head. A core group, apply, runs over the fakes:
  the channel's head applied once its claims are judged, a head whose
  claim is wanting refused with the target asked nothing, an empty
  channel and an unbound profile absent. 42, 45, 119 and 136 leave the
  register; the entries naming "the first target" are named again by
  the unit or phase that will cite them (design D11).
- **The proof**, in the tools layer: an attic server started for the
  test, a closure of two store paths built by the test's host, published
  as a version whose closure artifact names its root, applied to a cache
  environment through the Nix cache target with the token in attic's own
  configuration alone; then a fresh Nix store realises the root, its
  leaf with it, from that cache alone.
- The site's targets page says the Nix binary cache is built, how an
  environment names it and its closures, and how `timber apply` runs;
  record 0027 proposes the decisions of design D2 to D9, and rule
  core.70 holds the closure's form (design D6).

## Capabilities

### New Capabilities

- `environment-apply`: the core's apply of one environment: the
  environment found at the landed head, the channel's head read from
  the ledger, the claims its policy requires judged before the target
  is handed the version, and the answer `timber apply` renders.
- `nix-cache-target`: the Nix cache target profile: its table, the
  closure artifact's form, apply through attic with the environment's
  own credential, and current and report answered unread.

### Modified Capabilities

- `seam-contracts`: the policy has environments; a target's desired is
  handed the core's head, its environment carries its settings, and a
  target that holds no marker answers current and report unread; the
  binding holds targets by profile.
- `policy-file`: an environment's `target` is read as one key naming
  its profile and that key's table, and the profile answers the
  environments it declares.
- `host-bindings`: the repository binds the targets as it binds the
  projections, and the binding holds every target profile built.
- `command-line`: `timber apply` converges one environment.
- `seam-fakes`: the fake policy answers the environments a test sets;
  the fake target answers the head handed it, holds no marker where a
  test says so, and refuses a settings key a test did not let it read.
- `conformance-suite`: the target group's marker declaration and its
  two new scenarios, the policy group's environments, the apply group,
  the binding's targets by profile, and the register after this unit.

## Out of scope, and what each waits on

- The clan push profile, `profiles/target/nix_fleet/push/`: a machine
  applied by one `clan machines update` from the host that holds its
  key, what it runs read from the closure the machine runs; a report
  recorded where every asker reads it, as a claim the apply's host signs
  beside the index, and the apply's host telling its own gate so a
  channel made by `after` moves on the reports (37, 43, 83, 91, C.10);
  the bolt's S16; and 41 and 97 leaving the register: bolt 6's second
  unit, which takes this change's environments, binding, apply and
  closure reading, adds its fields to apply's answer, and writes none of
  them again.
- What runs `timber apply` when a channel an environment follows moves:
  a watcher, the driver, a job, a person (C.16). The core starts nothing
  (rule core.52); the host names it, and no unit of bolt 6 carves it.
- The Nix fleet's pull profile and the roster (44, 126, C.12): phase 2.
- What every environment should run and runs, shown by status and the
  page (98, and the parts of 132 and 134 the register names): phase 2;
  status's environments part stays unreadable naming the environments.
- Another pusher for a Nix binary cache, `nix copy` to a store address:
  a later key of the same table, when a fleet binds a cache attic is not.
- swancloud's own work: its machine tasks leaving a closure file as a
  declared output, its `.config/timber.toml` declaring its components,
  their closures and their cache environments, and retiring its
  releaser's push: swancloud's, after this unit and the clan push
  profile land.
- What the fleet's cache keeps and collects: attic's own, by age; a
  closure it has collected is built again by the machine that misses it
  (54, deploy.10 in swancloud).

## Impact

- New: `crates/timber/src/profiles/target/` (the family's `mod.rs`, the
  shared closure reading in `nix_closure.rs`, the profile in
  `nix_cache/`), `crates/timber-core/src/apply.rs`,
  `crates/timber/src/commands/apply.rs`,
  `crates/timber-conformance/src/core/apply.rs`,
  `crates/timber/tests/nix_cache.rs`.
- The seams: the policy trait gains `environments` and the types it
  answers; the target trait's `desired` takes `Head`; `Environment`
  carries the target's settings; `Seams.targets` is keyed by target
  profile. Every implementation and caller follows: the policy file,
  the fakes, `Nowhere`, the conformance rigs, the core's tests.
- The policy file profile reads `target`; the bindings bind the target
  family and give the Nix cache a store of its own, opened and never
  made.
- `devenv.nix` gains attic's client and server for the tools layer
  (rule core.26); the `tools` task of `crates/timber/moon.yml` gains
  `--test nix_cache`. No new crate: store paths are read with the
  profile's own code, linking no copyleft crate (the snix note, point 4).
- The register: 42, 45, 119 and 136 leave; 41 and 97 name the clan push
  profile; 44, 98, 126 and the parts of 132 and 134 name phase 2.
- The site's targets page and integrations index; record 0027 in
  `docs/adr/` and rule core.70 in `docs/architecture/core.md`.
- Rules kept: core.1 (attic, Nix and store paths named only under
  `profiles/target/`), core.2 (the closure's form shared through the
  family's module, never one profile calling another; the store reached
  through the store seam), core.4 and core.50 (the head read from the
  chain and resolved to the index's digest before anything acts),
  core.16 (an environment is handed a version only through its
  channel), core.19 (absent, refused and unreadable kept apart, attic's
  and nix's words carried verbatim), core.21 (the token is attic's own,
  reached by attic at the push; timber reads, writes and passes none),
  core.22 (the policy read at the landed revision), core.29 (no apply
  path skips the judge), core.31 (the command renders the core's
  answer), core.33 (each clause leaves the register in the commit that
  cites it), core.43 (the core names no channel; the policy profile
  alone reads the file's keys, a target its own settings as the policy
  answers them), core.52 (the core starts nothing after a move).

## Touches

```
devenv.nix
crates/timber/moon.yml
crates/timber/src/profiles/mod.rs
crates/timber/src/profiles/target/**
crates/timber/src/profiles/policy/file/**
crates/timber/src/bindings/mod.rs
crates/timber/src/bindings/tests.rs
crates/timber/src/commands/mod.rs
crates/timber/src/commands/apply.rs
crates/timber/src/main.rs
crates/timber/src/lib.rs
crates/timber/tests/nix_cache.rs
crates/timber/tests/**
crates/timber-seams/src/policy.rs
crates/timber-seams/src/target.rs
crates/timber-seams/src/bind.rs
crates/timber-seams/src/lib.rs
crates/timber-core/src/apply.rs
crates/timber-core/src/lib.rs
crates/timber-core/src/status.rs
crates/timber-core/tests/**
crates/timber-fakes/src/policy.rs
crates/timber-fakes/src/target.rs
crates/timber-fakes/src/lib.rs
crates/timber-conformance/src/**
crates/timber-conformance/tests/**
site/src/content/docs/guides/integrations/targets.mdx
site/src/content/docs/guides/integrations/index.mdx
docs/adr/0027-*.md
docs/architecture/core.md
```
