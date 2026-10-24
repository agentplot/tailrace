# Design

## Context

Three units come before this one in bolt 1, and their code is built on
their branches. `the-seams-and-their-fakes` gives the eight traits, the
four answers (`Answer<T>`, whose `Not` keeps absent, refused and
unreadable apart, each naming its `Source`), the `Seams` binding, and
in `tailrace-fakes` a `Fake` per seam over one `Script`: every clone a
view of the same state, `calls()` listing every operation in order, and
`fault(op, Fault::Refused | Unreadable(words), Times::Once |
UntilHealed)` and `heal(op)`. `Fakes` keeps a handle on each and binds
clones with `seams()`. Each fake's own scripting is what the rigs use:
`FakeStore::corrupt`, `FakeBuild::made`, `FakeLanding::land` and
`verdict`, `FakeSigner::hold` and `expire`, `FakePolicy::require_at`,
`FakeProjection::named` and `unsteady`, `FakeTarget::head`, `runs` and
`unreadable`, `FakeKeeper::fork`. `a-landed-revision-takes-its-version`
gives `Landed::check`, `History::open`, `Declared::read`, `derive`,
`touched` and `claim`, with the policy file (`POLICY_FILE`) declaring
each `[[component]]` by `name`, `outputs`, `paths`, `prefix` and
`projections`. `publish-status-and-verify-from-one-core` gives
`publish(seams, history, kept, ask)`, `verify(seams, history, ask)`,
`status(kept, ask)`, `Kept::at(history)`, `answer_json`, and
`tailrace::run(cli, Option<&Binding>, at, out)` with `Binding { seams,
signer }` for the command line. Where a name moves before this unit is
built, the coder takes the name as it stands and keeps the behaviour.

`tailrace-conformance` is the scaffold's crate: a doc comment, a moon
project with one `test` task tagged `check-fast`, depending on the
seams, the core, the fakes, `anyhow` and `serde_json`. The `tailrace`
crate already takes it as a dev-dependency, and its moon project
already depends on it.

The rule is core.33 as record 0012 amends it, accepted by the user on
2026-10-23: a clause no scenario cites is named in the register with the
unit that will cite it and why, and leaves the register with the
commit that cites it. Section 12 (blueprints b100cc8) says the same.
Both now let an entry name the phase of the plan, with the reason,
while that phase has no units carved; the planner names the unit when
it carves the phase (core.33 as tailrace ba83d81 has it).
The register's form is this unit's (record 0012, More information).
The model is sluice's conformance crate (scenarios generic over a rig,
run on the fakes in the crate and on the real profiles in the binary's
tests) and flywheel-next's `pending.yaml` for the register.

## Goals / Non-Goals

**Goals:**

- Citations as data the compiler checks and the coverage check reads,
  never parsed out of comments.
- One runner, one report, for a fake and a real profile alike.
- Groups that a later unit only adds to, so units of one bolt add
  scenarios side by side without touching each other's lines.
- Every clause of Parts A and B either proved by a scenario in this
  unit or named in the register, with nothing cited that the scenario
  does not assert.

**Non-Goals:**

- Running any real profile: each profile unit runs its seam's group in
  its own tests.
- Changing a trait, a fake or a core function. A scenario that cannot
  be written against what landed is a `crew needs`, and the change it
  needs is a unit of its own.
- Checking that a citation is honest. That is the reviewer's job under
  core.33. The check only finds what is missing or malformed.

## Decisions

### D1. A scenario is a value with its citations

```rust
pub struct Scenario<R: ?Sized> {
    pub name: &'static str,          // "the same bytes put twice"
    pub plays: Option<&'static str>, // Some("S1")
    pub cites: &'static [u16],       // clause numbers of Parts A and B
    pub run: fn(&R),                 // panics on a failed assertion
}
```

Each group is a module with `pub fn scenarios() -> Vec<Scenario<R>>`,
listing its scenarios in order. The last line of the list is the
marker:

```rust
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
```

A later unit adds entries below the marker, and the scenario functions
they name below the module's last function. `lib.rs` holds
`pub fn citations() -> Vec<Cited>` (group, name, plays, cites), built
from every group's list. A new group, from a later unit that builds a
new part of the core, adds its module and one line at the end of that
list.

Alternatives: sluice's `/// satisfies:` doc lines read by a parser,
where a typo or a missed line passes silently and the check has to
parse Rust; one `#[test]` per scenario, where a profile has to repeat
every test in its own crate and can leave one out.

### D2. The groups

`src/seams/`: `store.rs`, `build.rs`, `landing.rs`, `signer.rs`,
`policy.rs`, `projection.rs`, `target.rs`, `keeper.rs`, `binding.rs`.
`src/core/`: `version.rs`, `publish.rs`, `verify.rs`, `status.rs`,
`surfaces.rs`, and, holding no scenario yet, `gate.rs` (bolt 4),
`ledger.rs` (bolt 4), `cache.rs` (bolt 3), `record.rs` (bolt 3),
`layouts.rs` (bolt 5). Each is named for the core module the plan has
build it (phase-1-bolts.md), so a unit adds below the marker of the
group named for what it owns.

### D3. A seam's group runs over a rig, a fresh one per scenario

Each seam has a rig trait in its module: the seam itself, what a
scenario must set up, and how it makes the source unreachable. Every
rig extends `Rig { fn profile(&self) -> String; }`. In outline:

| rig | gives | sets up |
|---|---|---|
| `StoreRig` | `store()` | `unreachable()` |
| `BuildRig` | `build()`, `second_host()` | `revision(&[(output, Made::Pass \| Fail \| Unmade)])`, `unreachable()` |
| `LandingRig` | `landing()`, `branch()` | `landed(verdicts)`, `not_landed()`, `unreachable()` |
| `SignerRig` | `signer()`, `identity()`, `named(&SignerId)`, `stranger()` | `no_identity()`, `expire(&SignerId)`, `unreachable()` |
| `PolicyRig` | `policy()` | `at(revision, audience, Required)`, `unreachable()` |
| `ProjectionRig` | `projection()`, `output()`, `artifacts(n)` | `serves() -> bool`, `unreachable()` |
| `TargetRig` | `target()`, `verified(&Digest)` | `environment(head: Option<Digest>)`, `unreadable_environment()` |
| `KeeperRig` | `keeper()`, `promotion(channel, n)` | `fork(channel)`, `unreachable()` |

The rig that a profile's tests write is the profile's to make; this
unit writes the rigs over the fakes in `src/fakes.rs`, each a `Fakes`
with the fake's own scripting behind the setup methods (Context).
`unreachable()` over a fake faults every operation of the seam with
`Fault::Unreadable("connection refused")` and `Times::UntilHealed`.
`Rig::profile()` over a fake is `fake`, the profile the fakes' `Source`
names.

The runner:

```rust
pub fn run<R: ?Sized>(group: &str, scenarios: &[Scenario<R>], make: &dyn Fn() -> Box<R>) -> Report
```

It makes a fresh rig per scenario, runs it under `catch_unwind`, and
keeps every outcome. `Report` names the profile (from the rig), the
group, and per failure the scenario, what it plays, its clauses and the
panic's message. `Report::assert()` panics with the whole report when
anything failed. Every caller, the fakes' tests here and a profile's
tests later, calls `run(..).assert()`, so a fake is reported exactly as
any profile (93).

Alternatives: one rig shared by a group's scenarios, where one
scenario's state leaks into the next; stopping at the first failure,
which hides the rest of a profile's debt.

### D4. The core's groups run over one world of fakes and git

The core's groups (version, publish, verify, status) are about the
core, not a profile, and many of them script faults no real profile can
be told to make. So they take a concrete `FakeWorld` (in
`src/world.rs`), not a rig trait, as sluice's fault scenarios take its
`Harness`. The core's own tests have such a world (`tests/world/mod.rs`
over `tests/common/mod.rs`), which another crate cannot import; this
one is written after it, in the conformance crate's `src/`:

- `Fakes` bound as `Seams`, with two projections (ecosystems `file` and
  `image`), one target and one signer holding the host's identity;
- a git fixture (`src/fixture.rs`): a temporary directory (`tempfile`)
  holding a bare repository standing for the shared history and a
  clone, made with the git command as the core's fixture runs it (no
  global or system configuration, a fixed author and committer), and a
  fixed author and committer date as well, so revisions are the same
  every run; `commit(files, message)`, `tag`, `clone_again()` for a
  second publisher, and `policy_file(text)`, which commits
  `POLICY_FILE`;
- `History` and `Kept` opened over the clone;
- `land(revision, verdicts)`, which tells the fake landing the branch
  holds the revision and scripts its verdicts, and `outputs(revision,
  ...)`, which scripts the fake build's reports;
- observation through the fakes' `calls()`: `writes()` (puts, attaches
  and name moves), `keeper_calls()`, `target_calls()` and
  `build_calls()`.

The two-component fixture for S15 is `FakeWorld::two_components()`: a
policy file declaring two `[[component]]`s, `lib` over `lib/**` with
prefix `lib-v` and `app` over `app/**` with prefix `app-v`, each with
one output and its projection, and a first release tag on each.

The world is fixed and deterministic, so the surfaces group (D6) can
ask the same question of a surface and of the core and compare.

### D5. Which scenario cites what

Each row is one scenario. A clause is cited only where the scenario
asserts what the clause requires of the case it sets up.

| group | scenario | plays | cites |
|---|---|---|---|
| store | the same bytes put twice | | 1, 5, 66 |
| store | an artifact read back | | 4, 67, 72 |
| store | two attestations on one digest | | 8, 10, 68, 69 |
| store | a name moved from where it is not | | 70, 71 |
| store | a name that names nothing | | 71 |
| store | bytes read back | | 1, 72, 73 |
| store | a store that cannot be reached | | 58, 59, 72 |
| build | outputs ensured and reported | | 74, 75 |
| build | outputs the build could not make | | 74 |
| build | the same revision on a second host | | 76 |
| build | a build that cannot be reached | | 58, 59 |
| landing | landed and not landed | | 77 |
| landing | verdicts as the landing recorded them | | 78 |
| landing | a landing that cannot be reached | | 58, 59, 78 |
| signer | signed and verified | | 7, 79, 80 |
| signer | no identity at hand | | 79 |
| signer | an envelope changed after signing | | 10, 80 |
| signer | a signer the policy does not name | | 80 |
| signer | an expired identity | | 81 |
| signer | a signer that cannot be reached | | 58, 59 |
| policy | the claims an audience requires | | 82, 116 |
| policy | acceptance names no channel | | 83 |
| policy | no policy for an audience | | 82 |
| policy | a policy that cannot be reached | | 58, 59 |
| projection | packed the same again | | 84, 87 |
| projection | a layout of the versions given | | 48, 50, 85 |
| projection | served where the clients read no files | | 86 |
| projection | a projection that cannot be reached | | 58, 59 |
| target | what an environment should run | | 31, 40, 88 |
| target | an environment that cannot be read | | 58, 89 |
| target | converged and reported | | 43, 90, 91 |
| keeper | a chain appended in order | | 34, 120, 121, 122 |
| keeper | a stale predecessor | | 35, 105, 120 |
| keeper | a fork | | 105, 122 |
| keeper | a channel with no promotion | | 31, 121 |
| keeper | a keeper that cannot be reached | | 58, 59 |
| binding | one of each, any number of projections and targets | | 92 |
| binding | a broken profile fails the suite | | 93 |
| publish | a landed revision published | S1 | 6, 7, 11, 12, 13, 17, 19, 20, 22, 110 |
| publish | the same revision again | S2 | 3, 5, 18 |
| publish | a revision not landed | | 14 |
| publish | an output that did not pass | | 16, 111 |
| publish | a build no verdict names | | 15 |
| publish | a publish that stops before its index | | 17 |
| publish | a store that names bytes wrongly | | 2 |
| publish | a repeat that packs differently | | 18, 87 |
| publish | channels declared or not | | 20, 102 |
| publish | attestations beside their digest | | 8 |
| verify | a verdict from a signer the policy does not name | S5 | 3, 9, 80 |
| verify | no landing gate | S13 | 9, 15, 111 |
| verify | the policy read at the landed revision | | 65 |
| verify | an expired signer | | 81, 117 |
| verify | a signer beyond its reach | | 9, 116 |
| verify | a changed attestation | | 10 |
| verify | claims that cannot be read | | 58, 59 |
| version | a landing that touches one component | S15 | 123, 124 |
| version | a path both components name | | 124 |
| version | no component declared | | 123 |
| version | the same revision, two publishers | | 21, 24 |
| version | a name another revision holds | | 22 |
| version | versions ordered by their rules | | 23 |
| version | a checkout's edit is not read | | 65 |
| status | status after a publish | | 60, 61 |
| status | the record read with a file reader | | 62 |
| status | nothing kept, and an entry that cannot be read | | 58 |
| surfaces | the command line answers what the core answers | | 56, 57 |

Between them these cite every clause the register does not name (D7),
and none that it does.

S5 and S13 stop at verify's answer, as record 0012 has it. In bolt 1 no
gate refuses or accepts, so S5's "nothing moves" is checked as no name
moved but the version's own and no keeper append. S13's "refuses" is
checked as verify finding no verdict claim that counts.

The policy group's "no policy for an audience" asks for a layout's
policy where only acceptance's is declared, and holds the answer to
absent naming the audience, never refused. It does not ask for the
revision: the policy seam's absence names the audience alone, and
verify, which chose the revision from the version's tag, names it
beside the seam's absence (82 as blueprints b100cc8 has it; the landed
seam-contracts and attestation-verification specs divide it the same
way).

The target group's "converged and reported" hands the target a
`Verified` (the seams' type, not the core's verify answer of the same
name). The seams make it with the public `Verified::new(index,
version)`, for the core to call after its verify; a profile is handed
one and never makes one. The rig's `verified` calls it as the core
would, since the suite stands in the core's place here, and no profile
code makes one.

### D6. The surfaces group runs over the command line

`src/core/surfaces.rs` defines

```rust
pub trait Surface { fn name(&self) -> String; fn world(&self) -> &FakeWorld;
                    fn ask(&self, question: &Question) -> serde_json::Value; }
```

with `Question::{Publish, Verify, Status}`. The one scenario asks each
question through the surface and of the core (`answer_json` of
`publish`, `verify`, `status`), and compares the values. Publish is
asked through the surface first, then of the core. The core's answer is
then a repeat, which the publish design makes equal to the first. Verify
and status write nothing, so the order does not matter.
`crates/tailrace/tests/conformance.rs` implements `Surface` as the
binary's own `tests/cli.rs` asks the command line: the arguments with
`--json` through `Cli::try_parse_from`, then
`tailrace::run(&cli, Some(&Binding { seams: world.fakes.seams(), signer
}), clone, &mut out)`, then parses the printed JSON, and runs the
group. The conformance crate does not depend
on the binary's crate, so there is no cycle.

### D7. The clauses and the register are Rust constants

`src/statement.rs`: `pub const READ_AT: &str = "agentplot/blueprints
5b00e15"` and `pub const PARTS: &[(&str, &[u16])]`, one row per section
(A.1 to A.11, B.1 to B.9) with its clause numbers as the statement
orders them. This list changes only when the statement does.

`src/register.rs`:

```rust
pub struct Pending { pub clause: u16, pub unit: &'static str, pub why: &'static str }
pub const PENDING: &[Pending] = &[ ... ];
```

One entry per line, grouped by unit and in the plan's order, so the
units of one bolt each remove a block of their own and two removals do
not touch one hunk. The entries bolt 1 leaves:

| unit | clauses | why it waits |
|---|---|---|
| bolt 2 · the build and the landings | 104 | a verdict's signer named by the identity the landing gives it; the fake landing has no identities of its own |
| bolt 2 · the host-key signer | 64 | no credential kept; bolt 1 has no signer that holds a key |
| bolt 2 · the laptop | 118 | the host's bindings file |
| bolt 3 · the registry store | 103, 109 | a version copied to another store with its attestations; a name moved where the store cannot move by expected digest |
| bolt 3 · the cache | 52, 53, 54, 100 | cache entries, never artifacts, removable |
| bolt 3 · the push after a landing | 63 | the record rebuilt from the store and git |
| bolt 4 · the gate | 25, 26, 27, 28, 29, 30, 94, 95, 96, 107, 108 | acceptance; bolt 4's gate unit adds the gate's half of S5 and S13 and takes 25 to 30 out |
| bolt 4 · the keeper and promotion | 32, 33, 37, 38, 39, 41, 97, 101, 106, 125 | promotion, the ledger, bound channels, an approval as a promotion, a channel belonging to one component |
| bolt 4 · status across machines | 36, 98 | a channel's one version for every reader; what each environment should run and runs |
| bolt 5 · render under a policy | 47, 51, 99, 113 | consumer layouts under a policy |
| bolt 5 · the first served projection | 46, 49, 114, 115 | an ecosystem's clients and a served layout |
| phase 2 · the first target | 42, 44, 45, 119 | a reconciler; phase 1 leaves every target to phase 2 |
| phase 2 · the verifier's rebuild | 55, 112 | the rebuild by a second signer; left to phase 2 |

The last six name a phase, not a unit, since phase 2 has no units
carved (core.33, section 12). The planner names their unit when it
carves phase 2, in the commit that carves it.

That is 47 clauses: the 43 record 0012 names, and 42, 53, 64 and 100,
which bolt 1 has nothing to prove. Each `why` in the code is the row's
reason in one sentence. A unit is named by its bolt and what it builds,
in words core.1 allows in the core's crates: the plan's "moon and the
landings", "moon's cache" and "npm, rendered and served" are named for
the build, the cache and the first served projection.

Alternatives: a TOML or YAML file, as flywheel-next's, which needs a
parser and a schema check where the compiler gives both; the list of
clauses read from the statement at test time, which is in another
repository the gate does not check out.

### D8. The coverage check is a binary and a moon task of its own

`src/coverage.rs`: `check(parts, citations, pending) -> Coverage`, a
pure function. It returns the findings: a clause neither cited nor
registered; a clause cited and registered, with the scenarios that cite
it; a cited or registered number that is no clause; a scenario citing
nothing; an entry with an empty unit or reason; a clause registered
twice. It also returns the pending entries and the counts.
`src/bin/coverage.rs` calls it over `statement::PARTS`, `citations()`
and `register::PENDING`. It prints one line per pending clause
(`pending 25  bolt 4 · the gate  acceptance; ...`), the counts, and one
line per finding. It exits 1 on any finding.

The conformance crate's `moon.yml` gains

```yaml
  coverage:
    command: 'cargo run -q -p tailrace-conformance --bin coverage'
    inputs: [the test task's inputs]
    tags: [check-fast]
    options: {runFromWorkspaceRoot: true, outputStyle: 'buffer'}
```

`buffer` is moon's default for a task's output style, printed when the
task finishes, pass or fail. It is set explicitly so a later default
cannot hide the pending lines. One check per task (core.25): `test` runs
the scenarios and the check's own unit tests, and `coverage` runs the
check over the tree.

Alternatives: the check as a `#[test]`, whose output cargo swallows on
a pass, so pending would be reported only on a failure, against
section 12's "on every run".

### D9. What runs where

- `crates/tailrace-conformance/tests/fakes.rs`: one `#[test]` per
  group, each `run(group, &scenarios(), &|| Box::new(FakeXRig::new())).assert()`,
  and the core groups over `FakeWorld`.
- `crates/tailrace-conformance/tests/broken.rs`: the runner over
  deliberately broken stores (a wrapper over `FakeStore` that always
  answers "not held already", and one scripted to hold other bytes
  under a digest), asserting the report's text names the profile, the
  group, the scenarios, their clauses and the reason. This is how the
  binding group's "a broken profile fails the suite" is proved, and the
  scenario itself runs the same wrapper.
- `crates/tailrace-conformance/src/coverage.rs` unit tests: each finding
  over hand-made inputs.
- `crates/tailrace/tests/conformance.rs`: the surfaces group over the
  command line.

Nothing under `crates/tailrace-conformance/src/` names a word core.1
lists. The fixture's ecosystems are `file` and `image`, and its outputs
are named for what they hold.

## Risks / Trade-offs

- [A scenario cites a clause it only half proves, such as 31, 48 or 50,
  whose larger part a later bolt builds] → each such citation asserts
  the part the seam keeps (D5). The later unit adds the rest below its
  own marker and cites the clause again; a clause may be cited by many
  scenarios.
- [The register names units by the plan's words, and the plan moves] →
  the unit is a note (record 0012); an entry whose unit moved is
  corrected by hand in the commit that moves the plan.
- [Four entries beyond record 0012's forty-three] → named for the
  design agent and the planner. The check accepts any non-empty unit,
  so the suite passes either way.
- [The rigs' setup methods are this unit's guess at what a real profile
  can stage] → a profile unit that cannot stage one (expiring a real
  key, say) says so with `crew needs`, and the rig's method becomes a
  unit of its own. It is never skipped quietly.
- [Every scenario's panic prints through the default hook while the
  runner catches it] → the report is the record; the noise is in the
  test's captured output only.
- [`cargo run` in the coverage task compiles the crate a second time
  in another profile] → it shares the `test` task's dependencies and
  target directory, so the cost is one link.
