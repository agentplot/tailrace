## Why

swancloud's machines are deployed by its own deployer, and timber can
reach no machine. After the Nix cache target
(`openspec/changes/the-nix-cache-target`, Bolt 6's first unit) the core
can apply an environment: it finds the environment in the policy, reads
its channel's head from the chain, judges the head's claims and hands a
target the verified version. What is missing is a target that converges
one machine of a Nix fleet, a report that every asker can read, and the
rule that moves a channel an `after` makes. Bolt 6's second unit, the
clan push profile (`agentplot/blueprints:design/timber/models/phase-1-bolts.md`),
adds all three. The statement this change rests on is
`design/timber/requirements.md` at blueprints `2287669`, plus the
amendment on `design/timber-an-environment-after-another` (`c118a95`),
which writes S16 and makes a pushed machine's report a claim its host
signs (37, 41 to 45, 83, 88 to 91, 97, 119, C.10, C.12, section 9, S16).

## What Changes

- **The clan push profile**, `crates/timber/src/profiles/target/nix_fleet/push/`,
  implements C.12's push row for one machine of a clan fleet. The
  environment's target table is `target.clan = { machine, closure }`:
  the machine's name in the fleet, and the component's output that names
  the machine's closure. Any other key is unreadable, naming that key.
  - `desired` answers the channel's head the core hands it (88).
  - `apply` reads the version's index and the machine's closure
    artifact by digest. The artifact must name exactly one root. The
    profile evaluates the machine's toplevel from the fleet's flake at
    the revision the index names, and refuses to go on unless it is that
    root. It realises that root on this host from substituters alone,
    so a store path no substituter holds refuses the apply, naming the
    path, and nothing is built (42, I3). Then it runs one
    `clan machines update` of that machine with this host as the build
    host, which copies and switches the root. After the switch it leaves a marker on the
    machine naming the index digest and the closure it applied (119).
    It acts with the host's own key to the machine, which clan reaches
    through the host's own ssh, and timber holds no credential (45, 64).
  - `current` and `report` read what the machine answers: the closure it
    runs, and the marker. They answer the marker's digest where the
    machine runs the closure the marker names. Otherwise they answer
    unread, naming what the machine runs and what the marker says, and
    never guess (89, 91, 119).
- **A converged apply records a report.** The target's report must name
  the index that was applied. When it does, the core signs a report
  claim as the host's signer: what the host observed the machine
  answer, a claim of timber's own kind `report` (11, 141). The claim
  names the environment, its component and channel, the version and
  index, and when the machine converged. It is attached beside the
  index, where every asker that reads the store reads it (43, 91,
  core.27). A signer may claim a report only where the policy file's
  `kinds` lets it (116). `timber apply` binds the host's signer for this
  report alone. `timber verify` lists report claims as it lists every
  claim.
- **A channel an `after` makes moves on the reports.** The channels
  answer names the environments such a channel comes after (131, C.10).
  The channel's policy requires a report from each of those
  environments, after the gate's acceptance (83). After a converged
  apply, the apply's host tells its own gate under the `[gate]` binding
  the publish uses (ADR 0020). The gate then moves every channel that
  comes after that environment, as its own act, to the version each
  named environment last reported it runs. It moves the channel back the
  same way. It never moves the channel to a version a named environment
  did not last report, nor to one its bound channel never held (37). A
  `[gate] job` carries the job's words and moves nothing. Each channel
  is answered on its own in the apply's answer.
- **The suite plays S16 over the fakes.** One environment waits on an
  approver and a second comes after it. A yes moves the first's
  channel. The first converges and reports, and the second's channel
  then follows it, forward and back. The target group also runs over the
  profile with a scripted fleet, with a marker. 41 and 97 leave the
  register.
- **The proof against a machine is run by hand.** The tag `check-fleet`
  runs the target group and two applies and a move back against one
  machine. The machine is one the operator names in
  `TIMBER_TOOLS_FLEET`, and the run refuses without it, naming the
  setting. A machine's key is a key file, which no gate task reads
  (core.38).
- The site's targets page documents the clan push profile, its table,
  the report and `after`. Record 0028 proposes the decisions in design
  D2 to D9. Rules core.71 to core.73 hold the report, the `after` move
  and the update that builds nothing.

## Capabilities

### New Capabilities

- `clan-push-target`: the clan push profile: its table, apply as one
  update of the machine that builds nothing, the marker, current and
  report read from what the machine answers, and its proofs.
- `environment-reports`: the report claim a converged apply signs and
  attaches beside the index, who may claim one, and how a reader finds
  an environment's newest report.
- `after-channels`: the gate's move of a channel an `after` makes, on
  the newest reports of the environments it names, told by the apply's
  host.

### Modified Capabilities

- `seam-contracts`: a bound channel names the environments it comes
  after, and a report is a kind of claim whose requirement names its
  environment.
- `policy-file`: `report` is a kind a signer may claim; a channel an
  `after` makes is listed with its environments, and its policy requires
  a report of each.
- `host-bindings`: apply binds the host's signer to sign its report, and
  `[gate]` names what tells the gate after an apply as after a publish.
- `command-line`: apply's answer carries the report claim and each
  `after` channel's move.
- `seam-fakes`: the fake policy lists a channel bound after the
  environments a test names.
- `conformance-suite`: S16 and the `after` group over the fakes, the
  clan push profile in the target group, and the register after this
  unit.

## Out of scope, and what each waits on

- What runs `timber apply` when a channel moves: a watcher, a timer,
  the driver, a person (C.16). The core starts nothing (core.52).
  swancloud names its own when it retires its deployer.
- swancloud's own work. Its machines' build tasks must leave the
  closure file. Its `.config/timber.toml` must declare its components,
  srv's manual environment, its signers with `report` among their kinds,
  and no `after` for a Mac (swancloud record 0043). And it must retire
  its deployer. All of that is swancloud's, after this change lands.
- Environments in status and on the page (98, the parts of 132 and
  134): `phase 2 · environments in status`. Status reaches no store
  (60), and the reports are in the store.
- The Nix fleet's pull profile and the roster (44, 126): phase 2.
- A promotion's cause, the publish or the report it follows (129): unit
  `a-promotion-says-its-asker-names-a-host` in `docs/architecture/_open.md`.
  Until it lands, a move on reports names the gate's signer as its asker,
  and the reports among its counted claims.

## Impact

- New: `crates/timber/src/profiles/target/nix_fleet/` (the push profile,
  its fleet commands and their scripted stand-in),
  `crates/timber-core/src/after.rs`,
  `crates/timber-conformance/src/core/after.rs`,
  `crates/timber/tests/nix_fleet_push.rs` (by hand).
- The seams: `Moves::Bound` gains `after`, `ClaimKind` gains `Report`,
  and `RequiredClaim` gains the environment a report is of. The policy
  file, the fakes and the conformance rigs follow.
- The core: `apply.rs` signs and attaches the report and adds fields to
  `Applied`; `ledger.rs` judges a report requirement; `after.rs` moves
  the `after` channels.
- `crates/timber/src/profiles/target/mod.rs` binds the push profile.
  `bindings/mod.rs` binds the host's signer for `For::Apply`.
  `commands/apply.rs` tells the gate.
- `crates/timber/moon.yml` gains the `fleet` task, tagged `check-fleet`.
  No crate, dependency, language or file type is added; `clan` and `nix`
  are the host's own, run by hand.
- The site's targets page; record 0028 in `docs/adr/`; rules core.71 to
  core.73 in `docs/architecture/core.md`.
- Rules kept: core.1, since clan, Nix and ssh are named only under
  `profiles/target/`. core.2, since the closure is read through the
  family's `nix_closure` and the store through the store seam. core.7,
  since a report counts only through a signer the policy lets claim
  one. core.8, since the host signs only what it observed the machine
  answer. core.14, since only the gate's path promotes. core.21, since
  the machine's key is the host's, reached by clan's ssh. core.27,
  since a report sits beside its index. core.29, since nothing is
  applied before the judge. core.34, core.45 and core.50, since the
  `after` move reads each chain first. core.52, since the apply's host
  tells its own gate as the publish's host does. core.57, since report
  is timber's own kind and the catalogue carries none.

## Touches

```
crates/timber-seams/src/policy.rs
crates/timber-seams/src/claim.rs
crates/timber-seams/src/lib.rs
crates/timber-core/src/apply.rs
crates/timber-core/src/after.rs
crates/timber-core/src/ledger.rs
crates/timber-core/src/lib.rs
crates/timber-core/tests/**
crates/timber-fakes/src/policy.rs
crates/timber-conformance/src/**
crates/timber/src/profiles/target/mod.rs
crates/timber/src/profiles/target/nix_fleet/**
crates/timber/src/profiles/policy/file/**
crates/timber/src/bindings/mod.rs
crates/timber/src/bindings/tests.rs
crates/timber/src/commands/apply.rs
crates/timber/src/render.rs
crates/timber/tests/nix_fleet_push.rs
crates/timber/tests/**
crates/timber/moon.yml
site/src/content/docs/guides/integrations/targets.mdx
docs/adr/0028-*.md
docs/architecture/core.md
```
