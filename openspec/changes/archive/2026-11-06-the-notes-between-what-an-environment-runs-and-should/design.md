# Design

## Context

The statement this change rests on is `design/timber/requirements.md`
at agentplot/blueprints `90cf023`, which amends 60, 61 and 98 on
madswan-design's ruling of 2026-11-06. It sits on the host's local
blueprints `main` and is pushed when the user says so. The clauses the
change rests on are 40, 41, 43, 56 to 61, 63, 88, 91, 97, 98, 116, 132,
134, 136, C.7, C.10, C.14, S10, S11 and S12.

As amended, 98 says what an environment runs "is what its newest
counted report names, with when it converged (43, 91), read from the
store and never asked of its target (60)", and that one no counted
report names "is reported unread with the reason, never as converged".
60 says asking what an environment runs "fetches the reports attached
to the versions its channel's chain moved to into the host's copy where
the store answers, and answers from the copy saying when where it does
not. The answer is the newest report the policy counts (116, C.10). A
fetch writes the host's copy and nothing else ... No asking reaches a
signer or a target." 61 keeps "each environment's reports as the host
last read them".

The change is cut from `bolt/tailrace-the-nix-fleet-pushed`, which holds
the clan push profile. From there:

- `crates/timber-core/src/status.rs`: `overview(seams, history, kept,
  ask)` answers `Overview { branch, asked_at, channels, environments,
  published }`. `channels` reads the landed head, the policy's channels
  and each chain through `ledger::channel`, which takes a chain the
  ledger answers as last read, and renders each version's notes once
  through `Changelog`. `environments` answers `Err(no_target())`
  always, and `row` copies it into `Row.environments`.
- `crates/timber-core/src/after.rs`: `newest(seams, environment,
  component, channel, required)` reads the chain through `ledger::held`,
  which is unreadable on a chain as last read, asks `claims_of` of each
  index the chain moved to, keeps the reports of the environment and
  component that `verify::tell` counts, and answers the one with the
  latest `at`.
- `crates/timber-core/src/apply.rs`: `Reported` is a report claim's
  body, `{ environment, component, channel, version, index, runs,
  converged_at, at }`, and `reporters(seams, landed)` answers the
  signers the policy's answer for a layout lists with `report` among
  their kinds.
- `crates/timber/src/bindings/mod.rs`: for `For::Status` the store is
  opened only for a ledger held in it, else `Unopened` answers every
  operation absent; no signer signs, and the signer family is bound
  verify-only. The key-service profile's verify asks KMS for the key's
  public half and its state at every call.
- `crates/timber/src/profiles/ledger/store/mod.rs`: a chain read from
  the store is kept under `<common>/timber/ledger/`, written to a file
  beside and renamed, and answered as `Read::Kept { at, said }` where
  the store is unreadable.
- The register names 98, and the parts of 132 and 134 that show
  environments, under `phase 2 · environments in status`.
- Records 0026 and 0027 are the bolt's, which a fix on the bolt
  renumbers 0027 and 0028 as `main` holds a 0026 of its own. This change
  cites them by the new numbers: record 0027 for a Nix closure, record
  0028 for a pushed machine's report and the `after` move.

## Goals / Non-Goals

**Goals:**

- Status, the page, the root's rows and the pane show every environment
  the policy declares: what it should run, what it runs, its state, what
  it waits on, and, beside one that is behind, the notes of every
  version between (98, 132, 134).
- What runs is read from the store's reports, kept in the host's copy,
  and answered from the copy where the store does not answer (60, 61).
- Nothing status asks reaches a signer or a target (60).
- The suite plays S11, and S10 and S12 hold for environments.

**Non-Goals:**

- A target's roster (126) and the environments it holds.
- A key-service signer's public half carried in the policy (C.7).
- Anything the gate or apply does: the gate's newest report reads no
  copy and refuses a chain as last read, as before (core.45).

## Decisions

### D1. The build takes the renumbered bolt in first

`main` holds record 0026 of its own (the shell carries bun, `cc8829c`),
and the bolt lands second, so a fix on `bolt/tailrace-the-nix-fleet-pushed`
takes `main` in and renumbers the bolt's 0026 and 0027 to 0027 and 0028
with every citation. This change's build takes that fix in before it
writes any record or rule, so its record is 0029 and `root:adrs` never
sees two records of one number. Not taken: writing the record on this
branch now, as 0029 beside the bolt's unrenumbered 0027, which a reader
would find citing a 0028 that is not there.

### D2. The environments part is built from the channels part's reads

Status reads the branch's landed head, the policy and each chain once.
The environments the policy declares at that head (136) are listed in
the policy seam's order, of the component asked where status names one.
Each one's channel is among the channels the policy lists (131, C.10),
so what it should run is that channel's `at` as the channels part
answers it: at a version and index, empty, or the chain's own answer
(40, 88). The two parts never read one chain twice or disagree on a
head (56, core.50). Where the landed head cannot be read, or the policy
cannot list its environments, the environments part is that answer, as
the channels part is.

A policy declaring no environment answers an empty list with four
counts of none: the policy's own answer, as an empty list of channels
is (58). Not taken: answering the part absent there, which would read as
"not asked".

### D3. What an environment runs is its newest counted report, read for status

`after::newest` is split in two, written once and used by both readers:

- `reports(store, indexes, environment, component)`: the report claims
  of that environment and component attached to each index, once each,
  in the order the chain first moved to it, read by body and not yet
  counted.
- `newest_of(seams, reporters, reports)`: of those, the newest that
  counts. Reports are taken latest `at` first, the later in chain order
  where two name one moment, and each is told through `verify::tell`
  until one counts. A report whose signer cannot be told before one
  counts makes the answer unreadable naming what could not be read,
  since it may be the newest (core.42). One older than a counted report
  is never told.

The gate's `newest` reads the chain through `ledger::held`, which stays
strict on a chain as last read (core.45), and asks the store with no
copy. Status passes the indexes of the chain the channels part read,
from the source or as last read, and reads through the copy (D4). The
claims of one index are asked once per status and shared by every
environment whose chain moved to it.

The signers a report counts through are `reporters`, moved from
`apply.rs` beside `newest`: the signers the policy's answer for a layout
lists with `report` among their kinds (116, C.10). Apply's check of an
environment already running its head, an `after` channel's requirement
and status count through one set (record 0028). Not taken: counting
through the requirement of the channel the environment follows, which
names no report for a channel no `after` makes.

What it runs is answered as `Runs { version, index, converged_at, at,
claim, signer, read }`: the report's version and index, the target's
time it converged (98's "when it converged"), the apply's moment, the
report claim's digest, the signer that made it, and how the reports
were read. Where no report counts it is absent naming the environment,
and where the policy lets no signer claim a report the words say so.

### D4. The reports' copy

Each environment's reports are kept, as status last fetched them, at
`<common>/timber/reports/<environment>.json` where the repository
declares no component and `<common>/timber/reports/<component>/<environment>.json`
where it does, beside the store ledger's `<common>/timber/ledger/`. The
file is `{ at, reports: [{ index, envelope }] }`, `at` the moment of the
fetch in seconds since the Unix epoch: every report claim of the
environment and component the store held on the chain's indexes,
uncounted, so a changed policy counts the copy again with no fetch. It
is written to a file beside and renamed into place, as the ledger's
copy is.

A fetch writes the copy only once every index's claims are read. Where
the store answers any of them unreadable, nothing is written, and the
reports are answered from the copy with `Read::Kept { at, said }`, the
store's words verbatim (59, 60, S10). Where there is no copy, what the
environment runs is unreadable naming its reports, with the store's
words. A store that answers an index absent has answered: what runs is
unreadable naming the index, as `after::claims_of` answers it, and the
copy is left as it was. A copy that cannot be written or read is
unreadable naming its file with the system's words, as the store
ledger's is (58).

The copy is the host's and is rebuilt by the next fetch (61, 63, rule
core.20). The core keeps it, as it keeps the record of published
versions: the reports are always the store's, read through the store
seam, so the copy names no profile (core.1, core.2). Not taken: one copy
per index of every claim attached, which keeps more than 61 lets; and a
copy kept by the store profiles, which would be written once per store
profile for an answer that does not differ between them.

### D5. A report counts without reaching a signer

Status binds the signer family verify-only, as before, with the key
service profile bound to reach no service. Its verify does everything
it does before its first call to KMS, so an envelope with no P-256
signature, or a policy that lists no signer by a key ARN, is refused
unreached as ever. Where it would call KMS, it answers unreadable
naming the key the policy lists for the signer, saying status reaches
no key service (60) and that a key-service signer's public half is read
from the service until the policy carries it (C.7). The host key
profile verifies against the key text the policy lists and reaches
nothing. The profile's mode is `Reach::None` in
`crates/timber/src/profiles/signer/kms/reach.rs`, bound only for
`For::Status`. Not taken: letting status reach KMS, which 60 forbids;
and counting a report by the signer its envelope names, which core.7
forbids.

### D6. Each environment's state

An environment's state is decided in this order:

1. **Unread** where what it should run or what it runs is not answered,
   naming the answer in its place: the chain's, the reports', or the
   newest report's absence, which 98 answers as unread with the reason.
   An environment whose channel is empty has no report, so it is unread,
   saying no counted report names it and its channel is empty.
2. **Behind** where its newest counted report names another index than
   its channel's head (core.4).
3. **Waiting** where it waits (41, 97) and the channel its own is bound
   to is at another index than its own: nothing moves it without a
   person's yes.
4. **Running what it should** otherwise.

A waiting environment that is also behind is behind, since its converge
is due and needs no person. One that comes after others (37, C.10) is
never waiting on a person: its channel moves on reports. State is
answered in the one answer form: `converged`, `behind` or `waiting` as a
value, and unread as the unreadable answer carrying the reason (58).

### D7. What an environment waits on

Beside its state, an environment names what its channel moves on: for
one that waits, the approvers its policy names and the channel its own
is bound to with that channel's head; for one that comes after others,
the environments its channel comes after; for one that moves on its own,
nothing (41, 97, 132). These are read from the channels part's
`Moves::Bound { to, after }` and the policy's environment, and compute
nothing a surface could not find in the answer.

### D8. The notes of every version between

Beside an environment that is behind, status answers the versions
between what it runs and what it should, `Between { direction,
versions }`, `direction` `ahead` where what it should run is higher and
`back` where it is lower. The versions are the component's tags the
landed head reaches whose versions lie above the lower of the two and at
or below the higher by semantic precedence (23), releases only, with the
higher itself where it is a prerelease, in ascending order. A version's
notes cover the commits since the release below it (21, 134), so the
releases between and the higher end cover every commit between with no
commit twice. Each version is shown by its version and its tag's
revision, since status resolves no name in the store (60), with its
notes rendered by `Changelog::notes`, once per ask and shared with the
channels' changelogs (134). A version whose notes cannot
be rendered is its own unreadable answer (core.19); tags that cannot be
read, or a version that does not parse as a semantic version, make the
whole between unreadable naming it. An environment behind on the same
version at another index answers an empty list.

Not taken: every tag between, prereleases among them, each of which
repeats the commits since the release below; and the versions the
channel's chain moved to, which miss the commits of a version the
channel skipped.

### D9. The answer's form

The environments part answers `Environments { converged, behind,
waiting, unreadable, environments }`: the four counts flattened beside
the list, so the part and a repository's row read the same keys (57),
and the row's `environments` is the part's four counts or the part's own
answer. Each environment is:

```
{ environment, component, channel, waits, approvers,
  moves_on,                       // D7, absent where it moves on its own
  should: <answer of HeadAt>,     // D2
  runs: <answer of Runs>,         // D3
  state: <answer of State>,       // D6
  between: <answer of Between> }  // D8, only where behind
```

A count is of the environments listed, each in exactly one count.

### D10. Every surface renders the one answer

- `timber status` prints the counts under `environments`, then one line
  per environment with its state, what it should run and what it runs
  with when it converged, the reports' read where it was the copy, and
  the notes between indented beneath, as the changelog's are.
- The repository's page renders the same in a table of environments,
  each version's notes in an escaped `<pre>`, no script (core.56); the
  root's rows render the four counts.
- The herdr pane's `words.jq` renders the counts and each environment's
  lines from the answer, computing nothing (core.31).
- The page and the root compute the answer anew on each request, so a
  request fetches the reports and writes the copy, and writes nothing
  else (60, 132).

### D11. Status binds the store, never made

For `For::Status` the store is bound on every host as `[store]` names
it, or the given layout where the file names none, opened and never
made, for the reports and for a ledger held in it. A store that cannot
be opened is bound as `Unopened` answering every operation unreadable
with the opening's words, so the channels are read as before and each
environment's reports are answered from its copy (60). Not taken:
failing the binding, which would take the channels down with the store.

### D12. The suite

- The status group holds environments in status over the fakes: an
  environment running what it should, one behind with the notes between,
  one going back, one waiting, one after another, one no counted report
  names, one reported by a signer that may not report, the store
  unreachable answered from the copy saying when (S10), the copy deleted
  and fetched again (S12), the channel's chain as last read, and the
  target and signer asked nothing. It plays S11: the overview names each
  environment with what it should run, what it runs and whether they
  differ, the row's counts are the part's, and the surfaces group holds
  the command line and the page to the core's answer with environments
  in it.
- 98 leaves the register, and the parts of 132 and 134 leave its parts,
  in the commit whose scenarios first cite them; the constant naming
  `phase 2 · environments in status` goes with them.
- Once `origin`'s `main` holds `90cf023`, `statement.rs` reads it: its
  clauses hold 1 to 143, 141, 142 and 143 in A.2 after 13, and the
  register names 141 with `every-claim-takes-the-catalogued-kind-that-carries-it`
  and 142 and 143 with `phase 2 · keyless signing`. The open item for the
  suite's clauses leaves `docs/architecture/_open.md`.

### D13. The records

- Record 0029, `status-reads-what-an-environment-runs-from-its-reports-in-the-store`,
  MADR, decided and accepted by madswan-design on 2026-11-06. It holds
  the ruling with its two options not taken, as the context gives them,
  and the construction's choices: D3's counting newest first, D4's copy,
  D5's key-service mode, D6's states, D8's versions between and D11's
  store binding, each with the path not taken.
- Record 0028 gains an amendment line: environments in status are
  record 0029's, and its phase 2 line is withdrawn.
- Rule core.32 reads: "A read of what timber holds opens no connection
  to a signer or a target, reaches a store only to fetch a chain it holds
  or an environment's reports into the host's copy, and writes nothing
  but that copy." Its details add: Not ruled out: status fetching a
  chain a store ledger holds, and the reports attached to the indexes an
  environment's chain moved to, into the host's copy where the store
  answers (60, 61, C.14; record 0029). Rules out as well: a status that
  verifies a report through a key service (60, C.7).
- The details of core.27 and core.71 add: Not ruled out: the host's copy
  of an environment's reports as status last fetched them, each naming
  the index it is attached to, rebuilt from the store (60, 61; record
  0029).

## Risks / Trade-offs

- [A page request asks the store for the claims of every index each
  environment's chain moved to] → each index is asked once per request,
  and the chain bounds the indexes (core.50). A registry with a long
  chain answers more slowly; nothing is retried.
- [Reports signed by a key-service signer are unread in status until the
  policy carries the public half] → the answer names the key and says
  why, and the open item names the unit that ends it.
- [The copy is written by a read] → 60 says a fetch is not a write; the
  copy holds nothing the store does not, and deleting it loses nothing
  (S12).
- [An apply's report not yet attached reads as behind] → it is what the
  store holds; record 0028's next apply makes the report again.

## Migration Plan

- The environments part's JSON changes from an unreadable answer to a
  value; a reader of `environments.answer == "ok"` with four counts reads
  the same keys. The herdr pane's documents and the page change in this
  change.
- The copy's directory is made at the first status that fetches a
  report; nothing is migrated.
- Rollback: reverting the change leaves the copy's directory unread,
  and the next status answers the environments unread as before.
