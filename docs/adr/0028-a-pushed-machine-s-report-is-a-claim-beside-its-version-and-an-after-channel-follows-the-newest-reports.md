---
number: 28
title: A pushed machine's report is a claim beside its version and an after channel follows the newest reports
status: accepted
date: 2026-11-06
decision-makers:
- madswan-design
---

# A pushed machine's report is a claim beside its version and an after channel follows the newest reports

Status: accepted on 2026-11-06 by madswan-design, the design agent: the user's words it rests on are swancloud's record 0043, accepted on 2026-10-30, which S16 and Bolt 6 carry; the rest are choices on clauses already written, cheap to change, which are design's. Environments in status are not left to phase 2: the statement as amended at `agentplot/blueprints` 90cf023 reads them from the reports (60, 61, 98), as the consequences below say.

Deciders: madswan-design, taking the choices of the the-clan-push-profile unit of phase 1's bolt 6, the Nix fleet pushed (`agentplot/blueprints:design/timber/models/phase-1-bolts.md`, Bolt 6). Its number is 0028, the next after the Nix cache target's change, bolt 6's first unit, which takes 0027 after main's 0026. The user's own words it rests on are those of swancloud's record 0043, quoted below.
Sources: statement clauses 11, 37, 41 to 45, 83, 88 to 91, 97, 116, 119, 129, 130, 131, 141, C.10, C.12, C.16, section 9 and S16, as the statement has them at blueprints 2287669 (`design/timber/requirements.md`) with the amendment at `agentplot/blueprints` 33928bf; swancloud's record 0043 (afterthought/swancloud at 82a9065); ADRs 0009, 0016, 0019, 0020, 0022 and 0027; rules core.7, core.8, core.9, core.21, core.27, core.29, core.34, core.38, core.45, core.50, core.52, core.57 and core.60; the change's design, D2 to D9.

## Context and problem statement

After the Nix cache target (record 0027) the core could apply an environment: find it in the policy, read its channel's head from the chain, judge the head's claims and hand a target the verified version. No target converged a machine, and no channel an `after` makes ever moved: the policy file listed such a channel as bound to the channel its environments follow, and nothing moved it on what those environments ran. swancloud's machines are deployed by swancloud's own deployer, which timber cannot replace until it can reach a machine.

The amendment at 33928bf writes S16, one environment that waits on an approver and a second that comes after it, whose channel "is never moved to a version the first did not last report it runs" (37, 41, 43, 91, 97, C.10). It makes a pushed machine's report a claim the host that ran the apply signs (C.12), and counts the report among timber's own kinds (section 9, 141). Six questions were open:

- how one push converges one machine of a clan fleet on the closure its version names, and builds nothing (42, 90, C.12, I3);
- where a push leaves its marker of what it applied, which current and report read (89, 91, 119);
- what a report is, where it is kept and who signs it (11, 43, 91, 141);
- who may claim one, and how one is required (116, C.10);
- which of an environment's reports is its newest (43, 91);
- how a channel an `after` makes moves on them, and what tells the gate to move it (37, 83, 95, 129, 130, C.10).

swancloud's record 0043, accepted by the user on 2026-10-30, holds their words: "I don't think we need to have the Max go after the server." swancloud's policy therefore writes no `after` for a Mac, and S16 and the suite prove `after` on a fixture that names no machine of swancloud's.

## Considered options

For how a push converges a machine (42, 90, C.12, I3; design D3):

1. The version's root, checked and realised, then one update: the closure file of the table's output read by digest, naming exactly one root; the machine's toplevel evaluated from the fleet's flake at the revision the index names, which must be that root; the root realised in this host's Nix store from substituters alone; then one `clan machines update <machine> --flake <worktree> --build-host localhost --host-key-check strict`.
2. `--option max-jobs 0` passed to the update, keeping the fleet's build host.
3. Copying and switching by hand, with `nix copy` and `switch-to-configuration`.
4. The flake at the revision deployed without the check.

For where the marker is kept (119; design D4):

1. One JSON object, `{ index, version, closure, at }`, in the deploy user's state directory, `${XDG_STATE_HOME:-$HOME/.local/state}/timber/markers/<owner>/<name>/<component>/<environment>.json`, written through `clan ssh` to a temporary file renamed into place, its `<owner>/<name>` the repository the checkout is of.
2. A system-wide path, such as `/var/lib/timber`.
3. The closure's own `system.configurationRevision`.
4. A marker on the deploying host.
5. The machine's system profile.
6. The index's repository in the path at apply and the checkout's at current, or a search of the markers directory by component and environment.

For what a report is (11, 43, 91, 141; design D6):

1. A claim of timber's own kind `report`, which the apply's host signs once the target's report names the index applied, attached beside that index.
2. A git ref of reports per environment, read as the ledger is.
3. An unsigned record.

For whose reports count when apply asks whether an environment already running the head is reported (116; design D6):

1. Those of the signers the policy's answer for a layout lists with `report` among what each may claim.
2. Those of the signers the report requirements of the channels that come after the environment name.
3. A policy seam operation answering the signers that may claim a kind.

For who may claim a report (116, C.10; design D7):

1. The signers whose `[signers.<name>]` `kinds` hold `report`, and no other; the one host 108 admits gains no report.
2. 108's one host widened to reports.

For how a report is required (C.10; design D7, D9):

1. Only by the channel an `after` makes, one of each environment it names; a `require` entry of kind `report` unreadable, naming its table.
2. An `environment` key on `require` entries.

For which report is an environment's newest (43, 91; design D8):

1. Read across the chain of the channel the environment follows: the counted reports of that environment and component on every index the chain moved to, the one whose apply's moment, `at`, is latest.
2. The report on the newest index the chain holds.
3. The report the machine says converged last.

For how a channel an `after` makes moves (37, 83, 95, 129, 130, C.10; design D9):

1. `Moves::Bound { to, after }`: the gate, told by the apply's host as the publish's host tells it (record 0020), reads the newest report of each environment the channel comes after; it holds the channel where any is absent or unreadable or where they name different indexes, and otherwise moves it to the one index through the steps a promotion takes after its asking, as its own act, the host's signer recorded as who asked.
2. A fourth way a channel moves.
3. A move on the report just signed alone.
4. A new `[gate]` key for apply.

For a host that is not among the channel's promoters (32, 108; design D9):

1. Refused, `NotPromoter`, naming the host's signer and the channel's promoters, before any report is read.
2. `may_promote`'s refusal, a promotion `Wanting`, after the reports are read.

## Decision outcome

The push is option 1 (42, 90, C.12, I3; design D3; rule core.73). An environment names its machine with `target.clan = { machine, closure }`: the machine as the fleet's inventory names it, and the component's output whose closure file (record 0027) names the machine's closure. The apply reads the verified version's index by digest through the profile's own store binding, and from it the revision and the publishing repository it names (core.60), and the closure file by digest through the family's closure reader, which the Nix cache target reads it by too. It makes a detached worktree of the checkout at that revision under `<common>/timber/fleet/<revision>`, once, as the moon build profile makes `<common>/timber/builds/<commit>`. It evaluates `nixosConfigurations.<machine>`, or `darwinConfigurations.<machine>` where the flake holds no NixOS machine of that name, to its toplevel with `nix eval --raw`, and refuses naming both paths where that is not the root. It realises the root with `nix-store --realise <root> --option max-jobs 0`, a path no substituter holds refused in Nix's words. Only then does clan run, with this host as the build host, so the update finds the toplevel in this host's store and copies it, uploads the machine's vars and switches; `strict` asks no person, and a machine the host's known hosts do not hold fails in ssh's words. What runs is then what the landing's checks and the version's claims named. Option 2 leaves whether clan carries the option to a remote build host to clan's internals, and srv builds on itself, so a miss would quietly build there. Option 3 is not the fleet's own deploy tool, which C.12 names, and leaves the machine's vars and secrets unuploaded. Option 4 runs whatever the flake evaluates to, tied to nothing the checks named.

The marker is option 1 (119; design D4). After the switch the profile writes the index applied, its version, the closure's root and the machine's own clock at the write, so `at` is when the machine converged as the machine says it; the rename means a reader never reads part of a marker. current and report read `readlink /run/current-system` and the marker in one `clan ssh`, and answer the marker's index where the machine runs the closure the marker names; otherwise they answer unread naming what runs and what the marker says, and never guess (89, 91). The deploy user's directory is used because the Macs deploy as `chuck` and srv as `root`, and both can write their own state with no privilege beyond the deploy's; the path names the repository and the component, since one machine can be an environment of two repositories. Option 2 needs `sudo` on a Mac, which the deploy user may not hold without a password. Option 3 cannot name the digest, since the closure is built before its index exists. Option 4 is the host's state and not the target's (119), and is lost when the deploy role moves to another host. Option 5 is the pull profile's marker, whose agent learns the digest from the tag it reads; a push leaves no tag on the machine. The checkout's repository names the marker because current and report are asked of an environment alone, with no index to read a repository from; an index naming another publishing repository is refused before anything runs, since its marker would be one current never reads. Option 6's first half leaves a marker no read finds, and its second is ambiguous where two repositories deploy one machine.

A report is option 1 (11, 43, 91, 141; design D6; rule core.71). The in-toto catalogue (`in-toto/attestation`, `spec/predicates/`, read 2026-11-05) carries no claim that an environment runs a digest, so `report`, under `https://agentplot.github.io/timber/claim/report/v1`, is a kind of timber's own (core.57, record 0022). Once the target's report names the index applied, the core signs a report as the host's bound signer, `timber apply` binding it as publish and promote do, with the given host key made at the first sign where `[signing]` is absent. The claim is about the index and names the revision the index names, or, for an index put before indexes named one, the revision its version's tag names (core.9). Its body is `{ environment, component, channel, version, index, runs, converged_at, at }`: `runs` and `converged_at` are the target's own report, and `at` is the moment of the act, from the asker, so the core reads no clock. It is attached beside the index (core.27), where every asker that reads the version reads it, and `timber verify` lists it with its signer and its environment. A report that is unread, or names another digest, signs nothing, and apply's answer carries the target's answer in its place (core.8). Where current already answers the head, apply asks the target's report and reads the environment's newest report, signing only where that names another index or none, so a report the store refused to hold is made by the next apply and never doubled. Option 2 is a record clause 61 does not list; environments in status read the reports where this record keeps them (60, 61 as amended at 90cf023). Option 3 is one the gate could not count (core.7).

The reports apply counts for that check are option 1 (116, C.10; design D6). The policy's answer for a layout lists every signer the file names with what each may claim, so it holds whether or not any channel comes after the environment. Option 2 leaves an environment no channel comes after, such as each of swancloud's machines under its record 0043, with no report that counts, and every apply that found it running the head would sign another. Option 3 adds a fourth operation to the policy seam for an answer the layout's already carries.

Who may report is option 1 (116, C.10; design D7). `report` joins the kinds a `[signers.<name>]` table may list, and swancloud's file will list its deployer with `kinds = ["provenance", "report"]`. Option 2 contradicts C.10, which gives 108's one host acceptance and the given channels' promotion alone, and 116, under which the givens name no signer.

A report is required by option 1 (C.10; design D7, D9). The policy for a channel an `after` makes requires, after the gate's acceptance and its line's claims, one report of each environment the channel comes after, in `after`'s order, by every signer the file lets claim a report; where none may, each requirement names no signer and `judge` refuses it naming it, so the channel never moves on a report nobody may make. `RequiredClaim` carries the environment a report requirement is of, written only where it is set, as `check` is for a verdict, and `judge` counts it only through a report claim on the index naming that environment. Option 2 adds a key C.10 does not write, for a requirement no channel but an `after`'s has a use for.

The newest report is option 1 (43, 91, 40; design D8). Every version an environment ran came to it through its channel (core.16), so the chain bounds the search (core.50); a rollback reports an older index, and the newest index the chain holds can carry a stale report, which rules out option 2. Option 3 orders by a clock the machine keeps, and two machines' clocks order nothing between them. An unreadable chain or `claims_of` answers unreadable naming it, and no counted report answers absent naming the environment (58).

The `after` move is option 1 (37, 83, 95, 129, 130, C.10, S16; design D9; rule core.72). `Moves::Bound` gains `after`, skipped in JSON where empty, so every bound channel written before reads the same (57). The channel moves to the version whose tag resolves to the one index its environments last reported, through the same steps `ledger::promote` takes after its asking, shared by the two and written once: the chain's head, the bound channel's having held the version (37), `judge` under the channel's policy, the host's signer among the promoters, sign, append naming the predecessor, and mirror (33, 105, core.34, core.45). It moves back the same way. The gate records its own signer as who asked, as acceptance's on-publish move records it (95, 129), and `may_ask` is not applied. The trigger is the host's binding of what tells its gate, record 0020's arrangement again (130, core.52): after the core's apply answers a report's digest, `timber apply` reads the bound `[gate]`; for the host's own steps it asks the gate's `follow` and carries each channel's answer, moved, already there, held naming each environment's newest report, or refused or unreadable; for a job it carries the job's words, runs nothing and moves nothing. A refused channel leaves the others' moves standing, and nothing is retried (core.6). Option 2 contradicts C.10, which counts three ways a channel moves, and an `after` channel is bound, with a rule. Option 3 can carry the channel ahead of an environment whose newest report is older, S16's last sentence. Option 4 is a second table saying what tells one host's gate.

A host that could never move the channel is refused by option 1 (32, 108; design D9), so its answer names the host's signer as well as the promoters, and no store is read for a move the host cannot make. Option 2 names the promoters and not the host's signer, after reading every report.

Rules core.71, core.72 and core.73 hold the report, the move and the push.

### The construction's own choices

Choices made in building it that the design did not fix at first, each cheap to change, with the path not taken; each is written into the change's design and delta specs:

- A `machine` is letters, digits, `-` and `_`, beginning with a letter or a digit, since it is a part of a Nix attribute path and an argument of clan's; any other is unreadable naming `machine`, with nothing run. Not taken: any string handed to clan and Nix as given.
- An index naming no `file` artifact of the table's output answers absent naming the output and the index, since what the apply asks for is not there. Not taken: a refusal. An index naming no revision is refused, since no flake can be evaluated where it was published.
- `clan ssh` hands the command to the machine's login shell and answers no exit of it, so the read and mark scripts each end by printing a line of their own, and a run whose output lacks it answers unread with clan's and ssh's words verbatim. Every part of the marker's path is single-quoted, and no script holds a backslash pair, so a login shell of any kind parses it the same. Not taken: reading clan's exit.
- After the mark the machine is read back: it must run the root and its marker name the index, or apply answers unread naming what it runs. A marker that cannot be written makes the apply unreadable naming the marker and the machine's words, saying the switch was made.
- `timber apply`'s exit is the worst of the apply's answer, the report's and each channel's, a channel held for reports that disagree counting as refused. An unread report therefore exits as unreadable, and a Nix cache apply, whose report is always unread (record 0027), exits as unreadable after every push. Not taken: the apply's own answer alone as the exit, which says nothing of whether a channel after the environment can move.

### Consequences

- An environment names one machine of a clan fleet with `target.clan = { machine, closure }`, and its machine's build task leaves the closure file record 0027 describes.
- `timber apply <environment>` runs on the host that holds the machine's key, with the fleet's `clan`, `nix` and `nix-store` on its path and the machine's host key in its known hosts; timber reads, writes and passes no key (45, core.21).
- A converged apply attaches one report beside the version's index, read by every reader of the store; a signer whose `kinds` hold no `report` makes reports that are kept and never counted (core.7).
- A channel an `after` makes follows what its environments last reported, forward and back, told by the apply's host; a file whose signers hold no `report` leaves every such channel where it is.
- Environments in status and on the page (98) are read from the reports: status fetches each environment's reports into the host's copy as it fetches a chain the store holds, and answers the newest counted one, from the copy saying when where the store does not, reaching no signer and no target (60, 61, 98 as amended at `agentplot/blueprints` 90cf023). The change `the-notes-between-what-an-environment-runs-and-should` builds it, amending core.32.
- What runs `timber apply` when a channel moves stays the host's, a watcher, a timer, the driver or a person (C.16, core.52).
- Until the open unit `a-promotion-says-its-asker-names-a-host` gives a promotion its cause (129), a move on reports names the gate's signer as who asked, and the reports among its counted claims.
- The proof against a machine runs by hand under `check-fleet`, against a throwaway machine the operator names in `TIMBER_TOOLS_FLEET`, since the machine's key is a key file no gate task reads (core.38).
- The Nix fleet's pull profile and its roster (44, 126) are phase 2's, and keep the system profile as their marker.
