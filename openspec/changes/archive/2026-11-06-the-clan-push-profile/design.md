# Design

## Context

The statement this change rests on is `design/timber/requirements.md`
at agentplot/blueprints `2287669`, with the amendment on
`design/timber-an-environment-after-another` at `c118a95`, which is
committed and not yet pushed. That amendment writes S16. It also makes a
pushed machine's report a claim the host that ran the apply signs (C.12)
and counts the report among timber's own kinds (section 9). The clauses
the change rests on are 11, 37, 41 to 45, 83, 88 to 91, 97, 116, 119,
129, 131, 141, C.10, C.12 and C.16.

This change builds on `openspec/changes/the-nix-cache-target`
(`change/the-nix-cache-target`, f211c86), Bolt 6's first unit, and its
code stage takes that branch's work first. That change gives:

- `Policy::environments` and `DeclaredEnvironment { name, channel,
  waits, approvers, target: Option<TargetTable { profile, settings }> }`.
- `Seams.targets` keyed by `TargetProfile`, and `Environment.settings`.
- `Target::desired(&self, environment, head: Head<'_>)`.
- `crates/timber/src/profiles/target/mod.rs`, whose
  `bound(store, checkout)` binds every target profile built, each with a
  store of its own, opened and never made, and the checkout's path.
- `crates/timber/src/profiles/target/nix_closure.rs`, which reads a
  closure artifact's bytes into its roots.
- `crates/timber-core/src/apply.rs`, `apply(seams, history, &ApplyAsk)
  -> Answer<Applied>`. It finds the landed head, the environment and its
  bound profile. It reads the chain head, asks desired and current, and
  judges the head under the channel's policy. Only then does it hand
  over `Verified` and read the report. Apply moves no name and makes no
  store.
- `timber apply <environment> [--component] [--branch] [--json]`, bound
  as status is, with the target family beside it.
- The register: 41 and 97 named `bolt 6 · the clan push profile`.
- Record 0027 and rule core.70: a Nix closure enters the store as one
  artifact naming its roots.

The two planners settled the split on 2026-11-05, and their messages
are in each session. From this tree, before that branch:

- `crates/timber-seams/src/policy.rs`: `Moves::{EachPublish,
  Promotion, Bound { to }}`. A channel an `after` makes is
  `Bound { to }`, as `crates/timber/src/profiles/policy/file/mod.rs`
  builds it. The rule that moves it on the reports is built nowhere.
- `crates/timber-core/src/ledger.rs`: `promote` reads the chain, checks
  a bound channel's version against `held_by`, then calls `may_ask`,
  `judge` and `may_promote`, signs, appends and mirrors. `judge` counts
  a required claim through the signers it names, and a verdict through
  its check.
- `crates/timber-seams/src/claim.rs`: `ClaimKind` has no report.
  `crates/timber/src/bindings/mod.rs`: `Teller::{Publish, Job}` is read
  from `[gate]`.

What swancloud has (afterthought/swancloud at `82a9065`):

- `clan.nix` gives `swancloud-srv` a target host of `root@192.168.1.172`
  and a build host of the same, and each Mac a target host of `chuck@…`
  with `machineClass = "darwin"`.
- Record 0043, accepted by the user on 2026-10-30, holds their words "I
  don't think we need to have the Max go after the server." swancloud's
  policy will therefore write no `after` for a Mac. S16 and the suite
  prove `after` on a fixture.

The vendor facts were read from the host's `clan` on 2026-11-05:

- `clan machines update [MACHINE] --flake PATH --build-host BUILD_HOST
  --target-host … --host-key-check {strict,ask,accept-new,tofu,none}
  --option name value` (`ask` is the default).
- `clan ssh MACHINE -c …` runs a command on the machine at its
  configured target host (`tofu` is the default).
- The in-toto predicate catalogue (`in-toto/attestation`,
  `spec/predicates/`, read 2026-11-05) holds CycloneDX, Link,
  Reference, Release, Runtime Traces, SCAI, SLSA Provenance, SLSA VSA,
  SPDX, Simple Verification Result, Test Result and Vulns. None of them
  says that an environment runs a digest.

## Goals / Non-Goals

**Goals:**

- One machine of a clan fleet converges on its channel's head, from the
  host that holds its key. What runs is the closure the version names,
  and nothing is built.
- What the machine answers after an apply is held beside the version,
  signed, so that every reader of the store reads it, and the gate can
  count it.
- A channel an `after` makes follows what its environments last
  reported, forward and back, and never moves ahead of it.

**Non-Goals:**

- Running `timber apply` when a channel moves (C.16, core.52).
- Showing environments in status or on the page (98). Status reaches
  no store (60), and reading a report needs the store.
- A cause on a promotion (129); that is the open unit
  `a-promotion-says-its-asker-names-a-host`.
- Any other fleet tool than clan, and the pull profile.

## Decisions

### D1. This change takes the Nix cache target's machinery and adds to it

It writes none of `environments`, the target binding, `Head`, `apply`,
`timber apply` or `nix_closure` again. It adds its profile to the
family's `bound`, adds fields to `Applied`, and gives `For::Apply` the
host's signer. Not taken: building the machinery here. The cache target
comes first in the bolt, and both of its proofs need apply.

### D2. The profile's table is `clan = { machine, closure }`

`[environments.<component>.<name>.target.clan]` holds `machine`, the
machine's name in the fleet's inventory, and `closure`, the
component's output whose `file` artifact names the machine's closure,
as D6 of the Nix cache target makes it. Both are required strings, and
any other key is unreadable naming it (that change's D10). The
profile's key is `clan`, since it drives clan's own commands. A
colmena or deploy-rs push profile would be a key of its own. The code
lives in `profiles/target/nix_fleet/push/`, the bolt's directory, which
keeps room for the pull profile beside it.

A `machine` is letters, digits, `-` and `_`, beginning with a letter
or a digit, since it is a part of a Nix attribute path and an argument
of clan's; any other is unreadable naming `machine`.

Not taken: the host or user in the table. clan holds them in the
fleet's own inventory (`deploy.targetHost`), and the table holds no
credential and no address (45, C.10).

### D3. Apply: the version's root, checked, realised, then one update

The profile holds a store binding of its own and the checkout's path
from the family's `bound`. Its apply runs these steps:

1. **The version's root.** Read the index by `Verified::index()` and
   take from it the revision and the publishing repository it names
   (core.60). Find the `file` artifact whose name is the table's
   `closure` and read its bytes by digest. Pass the bytes to
   `nix_closure`; anything other than exactly one root is refused,
   naming the artifact. An index naming no `file` artifact of the
   output is absent, naming the output and the index, since what the
   apply asks for is not there. An index naming no revision is refused,
   since no flake can be evaluated where it was published. The index
   and the closure file are read by the functions of `nix_closure` the
   Nix cache target reads them by, so neither profile keeps a copy.
2. **The revision's checkout.** Make a detached worktree of the
   checkout at the revision under `<common>/timber/fleet/<revision>`,
   or reuse it, as the moon build profile makes
   `<common>/timber/builds/<commit>`.
3. **The check.** Evaluate the machine's toplevel at that worktree.
   The profile reads `nixosConfigurations.<machine>`, and where the
   flake has none of that name, `darwinConfigurations.<machine>`, taking
   `.config.system.build.toplevel.outPath` with `nix eval --raw`. A path
   other than the root is refused, naming both (I3).
4. **The realise.** Realise the root in this host's store with
   substitution alone, `nix-store --realise <root> --option max-jobs 0`.
   A miss is refused with Nix's words, which name the path (42, I3).
5. **The update.** Run `clan machines update <machine> --flake
   <worktree> --build-host localhost --host-key-check strict`. The
   toplevel is already in this host's store, so the update copies the
   closure to the machine, uploads its vars and switches. `strict`
   never asks a person: a machine the host's known hosts do not hold
   fails, with ssh's words carried.
6. **The marker** (D4), then a read-back: the machine must run the
   root, or apply answers unread naming what it runs.

Not taken:

- Passing `--option max-jobs 0` to the update and keeping the fleet's
  build host. Whether clan carries the option to a remote build host is
  clan's internals. srv builds on itself, so a miss would quietly build
  there.
- Copying and switching by hand with `nix copy` and
  `switch-to-configuration`. C.12 says the fleet's own deploy tool, and
  clan also uploads the machine's vars and secrets.
- Deploying the flake at the revision without the check. That would
  run whatever the flake evaluates to, and nothing would tie it to what
  the checks named.

### D4. The marker is a file in the deploy user's state directory

After the switch, the profile writes one JSON object, `{ "index":
"sha256:…", "version": "…", "closure": "/nix/store/…", "at": "<RFC 3339>"
}`, to `${XDG_STATE_HOME:-$HOME/.local/state}/timber/markers/<owner>/<name>/<component>/<environment>.json`
on the machine. It writes through `clan ssh <machine> -c`, to a
temporary file it then renames, so that a reader never sees part of
one (119). `at` is the machine's own clock at the write, so it is the
time the machine converged as the machine says it. `current` and
`report` read `readlink /run/current-system` and the marker in one
`clan ssh`, and answer as the spec says.

The `<owner>/<name>` of the path is the repository the checkout is of,
by the owner and name its landing's remote ends in, as the serving
process names it (114); the family's `bound` is handed it, and an
operation that needs the marker answers why it is unread where the
checkout has none. current and report are asked of an environment
alone, with no index to read a repository from, so the checkout's is
the one both apply and they can name. An index naming another
publishing repository is refused before anything runs, since the
marker apply would write is one current never reads. Not taken: the
index's repository at apply and the checkout's at current, which would
leave a marker no read finds; and a search of the markers directory by
component and environment, which two repositories deploying one
machine would make ambiguous.

`clan ssh` hands the command to the machine's login shell as `sh -c
<script>` and answers no exit of it (its interactive ssh ignores the
remote command's exit), so each script ends by printing a line of its
own, and a run whose stdout lacks that line answers unread with clan's
and ssh's stderr verbatim. Every part of the path is single-quoted in
the script, and the script holds no backslash pair, so a login shell of
any kind parses it the same.

The deploy user's directory is used because the Macs deploy as
`chuck`, srv as `root`, and both can write their own state with no
privilege beyond the deploy's. The path names the repository and the
component, since one machine could be an environment of two
repositories.

Not taken:

- A system-wide path such as `/var/lib/timber`. It needs `sudo` on a
  Mac, which the deploy user may not hold without a password.
- The marker in the closure, `system.configurationRevision`. The
  closure is built before its index exists, so it can name a revision,
  never the digest.
- A marker on the deploying host. That would be the host's state, not
  the target's (119), and lost when the role moves to another host.
- The system profile as the marker. That is the pull profile's marker,
  since the pull agent learns the digest from the tag it reads. A push
  has no tag on the machine to read it from.

### D5. The fleet is reached through the profile's own commands, scripted in the gate

The profile's operations against clan, Nix and the machine are
`eval_toplevel`, `realise`, `update`, `read` and `mark`, behind a trait
private to the profile. `Clan` runs `clan`, `nix` and `nix-store` from
the host's `PATH`, which on a deployer is the fleet's own shell, so the
tool is the version the fleet pins. Each command's stderr is carried
verbatim in an unread or refused answer (59). A `Scripted` fleet
answers as a test sets, writing and reading markers in memory, so the
target group and the profile's refusals run in `check-fast`. The trait
is no seam: the target seam stays the one interface (core.2).

Not taken: a machine in the tools layer. No host that runs the gate can
start a NixOS or nix-darwin machine in under a minute, and the key that
reaches a real one is a key file (core.38). The proof against a machine
is by hand (D11).

### D6. A report is a claim the apply's host signs, beside the index

`ClaimKind::Report` is a kind of timber's own, under
`https://agentplot.github.io/timber/claim/report/v1`, since the
catalogue carries no claim that an environment runs a digest (141,
core.57). `core::apply` uses it like this:

- After `report` answers the applied index, apply signs a report claim
  as `seams.signers.first()` with the asker's identity, as promote
  signs.
- The claim's subject is the index, and it names the revision the
  index names (core.9, core.60), or, for an index published before
  indexes named one, the revision its version's tag names, as promote
  finds it. Its body is `{ environment, component, channel, version,
  index, runs, converged_at, at }`. `runs` and `converged_at` are the
  target's `Running`; `at` is the moment of the act, from `ApplyAsk`,
  so the core reads no clock.
- Apply attaches the claim with `store.attach` and answers it in
  `Applied.report: Answer<Digest>`, the claim's digest or the answer
  given in its place. `Applied.running` carries the target's own
  report, the `Running` it answered.
- Where current already answers the head, apply asks `report`, reads
  the environment's newest report (D8), and signs one only where that
  newest report names another index or none, answering the held
  report's digest where it names the head. A refused attach is
  therefore made good by the next apply, never doubled. A newest report
  that cannot be read signs nothing and is the report's answer.
- The newest report apply reads counts through the signers the policy
  lets claim a report: those the policy's answer for a layout lists,
  each with what it may claim, filtered to `report` (116, C.10). That
  answer is the one the policy seam gives that lists every signer the
  file names, and it holds whether or not a channel comes after the
  environment.
- `judge`'s refusal of a report names its environment: `Why::Wanting`
  carries `environment`, written only where it is set (57), as it
  carries a verdict's `check`.

Where the report sits: the store holds the version's claims, and a
report beside its index is read by anyone who reads the version
(91, 39's form). Not taken:

- A git ref of reports per environment, read by status as the ledger
  is. That is a record 61 does not list. It is left until
  environments in status are designed, which also has to square 60.
- An unsigned record, which the gate could not count (core.7).
- Counting the newest report apply reads through the report
  requirements of the channels that come after the environment. An
  environment no channel comes after, such as each of swancloud's
  machines under its record 0043, would then have no report that
  counts, and every apply that finds it running the head would sign
  another.
- A seam operation answering the signers that may claim a kind. The
  layout's answer already carries them, and the policy seam keeps three
  operations.

### D7. Who may report is the file's alone

`report` joins the kinds a `[signers.<name>]` table may list. The one
host 108 admits gains acceptance and the given channels' promotion, and
no report. swancloud's file lists its deployer with `kinds =
["provenance", "report"]`.

Not taken: widening 108's one host to reports. C.10 gives that host
acceptance and promotion alone, and "the givens name no signer" (116).

### D8. An environment's newest report is read across its channel's chain

`core::after::newest(seams, environment, component, channel, required)`
works in four steps:

1. Read the chain of the channel the environment follows (core.50).
2. For each distinct index the chain has moved to, ask `claims_of` and
   keep the envelopes whose claim is a `Report` of that environment and
   component.
3. Verify each against the signers the requirement names (core.7).
4. Answer the one with the latest `at`.

Every version an environment ran came through its channel (40,
core.16), so the chain bounds the search. An unreadable chain or
`claims_of` answers unreadable naming it; no counted report answers
absent naming the environment.

Not taken: the report on the newest index the chain holds. A rollback
reports an older index, and the newest index can carry a stale report.

### D9. A channel an `after` makes moves on its environments' newest reports

These are the seam and policy changes:

- `Moves::Bound { to, after: Vec<EnvironmentName> }`, with `after`
  skipped in JSON when empty, so the answers written before read the
  same (57).
- `RequiredClaim` gains `environment: Option<EnvironmentName>`, set for
  a report, as `check` is for a verdict.
- `judge` counts a report requirement through a report claim on the
  index that names that environment.
- The policy file lists the `after` environments in `after`'s order.
  The after channel's policy requires, after acceptance and its line's
  claims, a report of each, by the signers whose kinds hold `report`. A
  requirement naming no signer is refused at judge, naming it.

The move, `core::after::follow(seams, history, &FollowAsk { branch,
component, environment, signer, at })`, works in four steps:

1. At the landed head, read `channels`. Take each channel in force
   whose `Moves::Bound { after }` holds the environment, and handle each
   on its own.
2. Read each named environment's newest report (D8). Where any is
   absent or unreadable, or where they name different indexes, answer
   `Held { reports }`, naming each environment's answer.
3. Otherwise take the one index. Promote the channel to the version
   whose tag resolves to that index, through the same steps
   `ledger::promote` takes after its asking: chain head, `held_by` (37),
   `judge` under the channel's policy, `may_promote`, sign, append and
   mirror (33, 105, core.34, core.45).
4. Record the asker as the host's signer, the gate's own act, as
   acceptance's on-publish move records it (95, 129). `may_ask` is not
   applied.

Before it reads a channel's reports, `follow` checks that the host's
signer is one of the channel's promoters (32, 108). Where it is not, the
channel is refused with `Why::NotPromoter { channel, signer, promoters }`,
naming both, and none of its reports is read, so a host that could
never move the channel is refused rather than held. Not taken:
`may_promote`'s refusal, `Wanting` a promotion, which names the
promoters and not the host's signer; and checking after the reports,
which reads the store for a move the host cannot make.

`ledger::promote` is refactored so its post-asking half is one function
the two callers share, with no second copy of the move.

The trigger lives in `commands/apply.rs`. After the core's apply
answers a report digest, it reads the bound `Teller`. For `Publish` it
calls `follow` and puts each channel's answer in `Applied.after`. For
`Job` it puts the words there. This is ADR 0020's arrangement again:
the step that calls one and then the other is the host's binding of
what tells the gate (130, core.52).

Not taken:

- A fourth `Moves`. C.10 counts three ways a channel moves, and an
  `after` channel is bound, with a rule.
- Moving on the report just signed alone. With several environments
  named, an older report of another environment's could carry it ahead
  (S16's last sentence).
- A new `[gate]` key for apply. One table says what tells this host's
  gate, whatever it is told of.

### D10. S16 and the register

`crates/timber-conformance/src/core/after.rs` is a new core group,
`after`, over the fakes. The world declares `first`, manual on `stable`
with `approvers = ["desk"]`, and `second`, `after = ["first"]`. The fake
policy scripts their channels and policies, and a fake target runs per
environment. The group holds:

- "a converged apply reports" (11, 43, 91)
- "an unread report signs nothing" (43, 91)
- "the newest report across a rollback" (43, 91)
- "S16", playing `S16` and citing 37, 41, 43, 83, 91 and 97

S16 takes three steps:

1. `desk` asks for `first` to move to 0.2.0, and `second` stays
   empty.
2. `first` is applied and reports, then `follow` moves `second` to
   0.2.0.
3. `first` is moved back, applied, reports 0.1.0 (it held 0.1.0
   before), and `second` follows back.

After each step the scenario checks that `second`'s head is `first`'s
newest report or empty. 41 and 97 leave the register in the commit that
adds S16, and the register's `bolt 6 · the clan push profile` constant
goes with them.

### D11. The proof against a machine, by hand

`crates/timber/tests/nix_fleet_push.rs` sits behind the `tools`
feature. The `fleet` task of `crates/timber/moon.yml` runs it under the
tag `check-fleet` (core.38), with `$TIMBER_TOOLS_FLEET` among its
inputs, set in `devenv.local.nix` as the ECR and KMS settings are.
The setting is `<checkout>#<machine>`:

- `<checkout>` is a clan fleet's git checkout whose `HEAD~1` and
  `HEAD` configure `<machine>` differently.
- `<machine>` is a throwaway machine the host holds the key to and may
  switch at will. Never srv, and never a Mac in use.

A run without the setting fails naming it. With it, the run takes these
steps:

1. Build both revisions' toplevels with the host's `nix build
   --print-out-paths`, which leaves them in the host's store.
2. Publish them as two versions into a layout in the test's directory,
   each with its closure file packed by the file projection, through the
   fakes' build answering those files.
3. Run the target group over the profile with `Clan`.
4. Apply A, then B, then A again, checking `current`, `report` and the
   marker after each. Then delete the marker, and check that `current`
   answers unread.

### D12. The records

- Record `docs/adr/0028-a-pushed-machine-s-report-is-a-claim-beside-its-version-and-an-after-channel-follows-the-newest-reports.md`,
  MADR, proposed, covers D3, D4, D6, D7, D8 and D9, citing the
  blueprints amendment `c118a95`.
- `docs/architecture/core.md` gains three rules:
  - **core.71**: a report is signed only by the host that ran the apply,
    only after the target's report names the index applied, and only
    beside that index.
  - **core.72**: a channel an `after` makes is moved only to the index
    every environment it names last reported, read from counted reports
    across each one's chain, and by no asking but the gate's own.
  - **core.73**: a push to a Nix machine runs only the root its
    version's closure names, checked against the revision's evaluation
    and realised from substituters alone before the fleet's tool runs,
    and builds nothing.
- The site's targets page
  (`site/src/content/docs/guides/integrations/targets.mdx`) gains the
  clan push profile: its table,
  what apply runs, the marker, the report and its signer's `kinds`,
  `after` with an example, and that `timber apply` runs on the host
  holding the machine's key.

## Risks / Trade-offs

- **The deployer realises a whole closure.** A Mac deploying srv
  substitutes srv's Linux closure into its own store before copying it
  back. → Correct and bounded by what the cache holds, and Nix skips
  the paths already present. A later key could let the fleet's build
  host realise instead, once clan is shown to carry `max-jobs` there.
- **A machine deployed by hand outside timber.** current answers unread
  until the next apply, and an `after` channel may have moved on the
  last timber report. → The report is what the host observed when it
  made it. A break-glass deploy is outside every reconciler, and
  current says so on the next read.
- **The machine's clock.** `converged_at` comes from the machine. → It
  orders nothing: D8 orders by the apply's `at`, the asker's moment.
- **Two hosts applying one environment at once.** The markers race and
  both reports are signed. → The core starts nothing, and one host
  holds a fleet's deploy role, as swancloud's handover keeps one.
  Ordering by `at` keeps the newest report well defined.
- **A report the store refuses to hold.** The `after` channel waits. →
  D6: the next apply that finds the head running signs the report
  again.

## Migration Plan

Nothing published changes. A bound channel's JSON with no `after` reads
and writes as before (D9). swancloud adopts this after both changes
land: its tasks leave closure files, and its policy file declares
components, environments, `target.clan` tables and a deployer signer
that may report. Its deployer's `timber apply` loop replaces
`clan machines update` machine by machine. Rollback is a promotion
back and an apply (34).
