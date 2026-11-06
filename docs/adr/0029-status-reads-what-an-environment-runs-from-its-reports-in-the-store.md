---
number: 29
title: Status reads what an environment runs from its reports in the store
status: accepted
date: 2026-11-06
decision-makers:
- madswan-design
---

# Status reads what an environment runs from its reports in the store

Status: accepted on 2026-11-06 by madswan-design, the design agent, whose ruling of that day on the change's question it records; the statement carries the ruling at `agentplot/blueprints` 90cf023, and the construction's choices below are cheap to change, which are design's.

Deciders: madswan-design, ruling on 2026-11-06 on the question of the change `the-notes-between-what-an-environment-runs-and-should`, and taking that change's choices. Its number is 0029, the next after record 0028.
Sources: statement clauses 40, 41, 43, 56 to 61, 63, 88, 91, 97, 98, 116, 132, 134, 136, C.7, C.10, C.14, S10, S11 and S12, as the statement has them at `agentplot/blueprints` 90cf023 (`design/timber/requirements.md`), which amends 60, 61 and 98 on this ruling; ADRs 0009, 0016, 0017, 0021 and 0028; rules core.4, core.7, core.19, core.20, core.21, core.27, core.31, core.32, core.42, core.45, core.50, core.56 and core.71; the change's design, D2 to D13.

## Context and problem statement

Status and the host's page named every channel, where it was and the chain that put it there, and answered the environments unreadable, saying no target is read. 98 asks that an asker learn of every environment what it should run, what it runs and that the two differ; 132 asks the page for what every environment runs and waits on, and the root's rows for how many run what they should, are behind, wait on a person or cannot be read; 134 asks for the notes of every version between. The clan push profile (record 0028) made what a pushed environment runs a report claim the apply's host signs and attaches beside the version's index, so what runs is in the store. Clause 60, as it stood, had asking reach no store, no signer and no target, so status could read neither the reports nor a target.

The question was how status learns what an environment runs. Under the ruling, six construction questions followed: which report counts, where the reports are kept, how a key-service signer's report is told, what each environment's state is, which versions stand between, and how status binds a store it may not be able to open.

## Considered options

For how status learns what an environment runs (60, 61, 89, 98):

1. From its reports in the store: status fetches the reports attached to the versions its channel's chain moved to into the host's copy where the store answers, as it fetches a chain the store holds (C.14), and answers the newest report the policy counts, from the copy saying when where the store does not; it reaches no signer and no target and writes nothing but that copy, and an environment no counted report names is unread with the reason.
2. From every target, asking each its current (89).
3. Not at all: 60 left as it was, and environments in status left to phase 2.

For which report is answered and how it counts (43, 91, 116, rule core.42; design D3):

1. Read and counted apart: the report claims of the environment and component attached to each index the chain moved to, once each in the order the chain first moved to it and uncounted, then told latest `at` first, the later in chain order where two name one moment, until one counts, a report whose signer cannot be told before then making the answer unreadable naming it; counted through the signers the policy's answer for a layout lists with `report` among their kinds, the set apply and an `after` channel count through.
2. Counted through the requirement of the channel the environment follows.
3. Every report told, and the newest of those that count answered whatever the others answer.

For where the reports are kept (61, 63; design D4):

1. One copy per environment, `<common>/timber/reports/<environment>.json`, or `<component>/<environment>.json` where components are declared, beside the store ledger's `<common>/timber/ledger/`, holding `{ at, reports: [{ index, envelope }] }`, every report claim of that environment and component on the chain's indexes, uncounted, written beside and renamed, and only once every index's claims are read; kept by the core, read through the store seam.
2. One copy per index of every claim attached to it.
3. A copy kept by each store profile.

For how a key-service signer's report is told (60, C.7, rule core.7; design D5):

1. The key-service profile bound for status to reach no service, `Reach::None`: its verify does everything it does before its first call to KMS, so what it refuses unreached it refuses as ever, and where it would call KMS answers unreadable naming the key the policy lists, saying status reaches no key service and that a key-service signer's public half is read from the service until the policy carries it; its sign answers absent.
2. Status reaching KMS as verify does.
3. A report counted by the signer its envelope names.

For each environment's state (41, 97, 98, rule core.4; design D6):

1. In order: unread where what it should run or what it runs is not answered, naming the answer in its place; behind where its newest counted report names another index than its channel's head; waiting where it waits and the channel its own is bound to is at another index than its own; running what it should otherwise. One that comes after others is never waiting.
2. Waiting before behind, so an environment that waits is shown waiting on a person whatever it runs.
3. An environment whose channel is empty answered running what it should, or left out.

For the versions between what an environment runs and what it should (21, 23, 134; design D8):

1. The component's tags the landed head reaches above the lower of the two and at or below the higher by semantic precedence, releases only, with the higher where it is a prerelease, ascending, each by its version and its tag's revision with its notes rendered as the channels' changelog renders them, once per status; the direction `ahead` where what it should run is higher and `back` where it is lower.
2. Every tag between, prereleases among them.
3. The versions the channel's chain moved to.

For how status binds the store (60, 61, 118; design D11):

1. On every host, as `[store]` names it or the given layout where the file names none, opened and never made, for the reports and a ledger held in it; a store that cannot be opened bound as one answering every operation unreadable with the opening's words, the store ledger reading through it.
2. The binding failed where the store cannot be opened.

## Decision outcome

Status reads what an environment runs by option 1, as madswan-design ruled on 2026-11-06 and the statement says at 90cf023 (60, 61, 98). What an environment runs is what its newest counted report names, with when it converged (43, 91), read from the store and never asked of its target. Option 2 needs every target's credential on the reader's host (45, 64) and reaches every machine at every status and every page request. Option 3 leaves 98, 132 and 134 unmet, while the phase 1 herdr pane shows what runs and phase 2's list in the bolts model holds no environments in status. Status reaches the store as it already reached a ledger the store holds (C.14): the fetch writes the host's copy and nothing else, and a fetch is not a write in 60's sense.

The report is option 1 (design D3). `after::newest` is split into `reports`, which reads, and `newest_of`, which counts, and both readers use them: the gate and apply read the chain through `ledger::held`, strict on a chain as last read (core.45), and ask the store with no copy; status passes the indexes of the chain its channels part read, from the source or as last read, and reads through the copy. Telling newest first and stopping at the first that counts never tells a report older than one that counts; one that cannot be told before then may be the newest, so the answer names it and passes nothing over (core.42). `reporters` moves beside them from `apply.rs`, so apply's check of an environment already running its head, an `after` channel's requirement and status count through one set (record 0028). Option 2 names no report for a channel no `after` makes. Option 3 makes the answer turn on reports a counted newer one already outranks.

The copy is option 1 (61, 63, rule core.20; design D4). It holds every report claim of the environment uncounted, so a changed policy counts the copy again with no fetch, and nothing the store does not hold. It is the host's and the next status fetches it again, so deleting it loses nothing (S12). Where the store answers any index unreadable nothing is written and the reports are answered from the copy, `Read::Kept { at, said }`, with the store's words verbatim (59, S10); with no copy, what the environment runs is unreadable naming its reports with the store's words. An index the store answers absent has been answered, so what runs is unreadable naming the index and the copy is left as it was. A copy that cannot be read or written is unreadable naming its file with the system's words (58). The core keeps it as it keeps the record of published versions, and the copy names no profile, since the reports are read through the store seam alone (core.1, core.2). Option 2 keeps more than 61 lets. Option 3 writes one copy per store profile for an answer that does not differ between them.

A key-service signer's report is told by option 1 (60, C.7; design D5). The host key profile verifies against the key text the policy lists and reaches nothing. `Reach::None` is bound only for status. Option 2 is what 60 forbids. Option 3 is what core.7 forbids. Until the policy carries a key-service signer's public half (C.7, unit `a-policy-carries-a-key-service-signers-public-half`), a report only a key service could tell leaves its environment unread naming the key.

The state is option 1 (98; design D6). An environment that waits and is behind has a converge due that needs no person, so option 2 would show a person's wait where none is asked. One whose channel is empty has no report, so it is unread, saying no counted report names it and its channel is empty; option 3 would show it converged on nothing, or not at all, both of which 98 forbids. State is answered in the one answer form: `converged`, `behind` or `waiting` as a value, and unread as the unreadable answer carrying the reason (58). Beside it, what the environment's channel moves on is read from the channels part's `Moves::Bound { to, after }` and the policy's environment (design D7).

The versions between are option 1 (134; design D8). A version's notes cover the commits since the release below it (21), so the releases between and the higher end cover every commit between, none twice. Each version is shown by its tag's revision, since status resolves no name in the store (60). Option 2 repeats, in each prerelease, the commits since the release below. Option 3 misses the commits of a version the channel skipped. A version whose notes cannot be rendered is its own unreadable answer (core.19), and tags that cannot be read, or a version that is no semantic version, make the whole between unreadable naming it.

The store is bound by option 1 (60, 61; design D11). Status reads the channels whether or not the store opens, and each environment is answered from its copy where it does not. Option 2 takes the channels down with the store, and with a store ledger loses the chain the host kept.

### The construction's own choices

Choices made in building it that the design did not fix at first, each cheap to change, with the path not taken; each is written into the change's design, its delta specs or the code's documents:

- A fetch reads the indexes in chain order and stops at the first the store does not answer: absent, unreadable naming that index with the copy left as it was; unreadable, answered from the copy. Not taken: asking every index and answering by the worst.
- A report whose signer cannot be told is unreadable with the signer profile's source and words, its `what` naming the report's digest, its environment and its index. Not taken: the signer's words alone, which say nothing of which report.
- `Reach::None` answers `Failure::NoService` at the profile's opening of a client, before any request. Its verify names the first key the policy lists; its sign answers absent, "no identity is at hand, since status reaches no key service". Not taken: an unreadable sign, since status signs nothing.
- A waiting environment whose bound channel's head cannot be read is unread naming that channel's answer. Not taken: answering it running what it should, which would guess that nothing waits.
- An environment behind on the same version at another index answers `ahead` with no versions between. Not taken: a direction of its own for a move that changes no version.
- A commit the local history no longer holds makes the whole between unreadable naming the tags the head reaches, with git's words naming the commit, since the history's walk of the tags crosses every commit above the lowest and fails before any version's notes are rendered. Not taken: that version's notes alone unreadable, which the walk cannot reach. A version whose notes' template the version rule's tool cannot read at its revision is that version's unreadable answer, the others shown.
- A store ledger reads through the same unopened store, so a chain it kept is answered from its copy and one it never kept is unreadable naming the chain with the opening's words; the opening's answer is carried as given. Not taken: failing a store ledger's binding, which loses the kept chain.

### Consequences

- Status, the repository's page, the root's rows and the herdr pane show every environment the policy declares at the landed head, of the component asked: what it should run, what it runs and since when, its state, what it waits on and, beside one that is behind, the notes of every version between, each from the core's one answer (56, 57, 132, core.31, core.56). The environments part and each row carry the four counts, each environment in exactly one.
- Each page request fetches the reports and writes the copy, and writes nothing else (60, 132).
- The reports' copy is kept under the repository's common git directory at `timber/reports/`, readable with a file reader (62), and rebuilt by the next status where it is deleted (63, S12).
- A report signed by a key-service signer leaves its environment unread in status until the policy carries the signer's public half (C.7); the open item in `docs/architecture/_open.md` says so.
- The gate's and apply's newest report read no copy and refuse a chain as last read, as before (core.45).
- Rule core.32 lets status reach a store only to fetch a chain it holds or an environment's reports into the host's copy, and the details of core.27 and core.71 say the copy is no keeping of a report apart from its index.
- Record 0028's line leaving environments in status to phase 2 is withdrawn; environments in status are this record's.
- Environments a target's roster holds (126) are phase 2's, since reading a roster reaches a target (60).
