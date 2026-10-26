# command-line Specification

## Purpose
The command line as tailrace's first surface: `tailrace publish`,
`tailrace verify` and `tailrace status` each call the one core and
render its answer, as text for a person or as the answer itself for a
program.

## Requirements

### Requirement: Each command calls the one core and renders its answer

`tailrace publish`, `tailrace verify`, `tailrace status`, `tailrace
sync`, `tailrace push`, `tailrace rebuild`, `tailrace promote` and
`tailrace policy` SHALL each turn their arguments into one question to
the core and render the core's answer; no command SHALL read a seam, a
file of the record or the history itself, or compute a version, a
digest, a signer or a channel of its own (56, core.31).

#### Scenario: The same answer through the command line

- **WHEN** a version is published over the fakes through the command line with `--json`, and the same question is put to the core directly over an identical binding
- **THEN** the printed JSON is the core's answer, value for value

### Requirement: Every answer is readable by a program on request

Each command SHALL print, with `--json`, its answer as one JSON document
naming which of the four answers it is and, for an answer, the digests,
versions and claims in the statement's words (57).

#### Scenario: A refusal as JSON

- **WHEN** a publish of a revision not landed is asked with `--json`
- **THEN** the output is one JSON document saying refused, naming the branch, the revision and the landing as the source

### Requirement: Absent, refused and unreadable are three exits

Each command SHALL exit 0 on an answer, 1 on a refusal, 3 on an
unreadable answer and 4 on an absence, and 2 on arguments it cannot
read; a refusal or an unreadable answer SHALL print the source's own
words verbatim beside tailrace's (58, 59, core.19).

#### Scenario: Three answers, three exit codes

- **WHEN** a publish is asked of a revision not landed, a verify of a version never published, and a verify while the fake store's claims-of is unreadable with the words `connection refused`
- **THEN** they exit 1, 4 and 3 in turn, and the third prints `connection refused` verbatim

### Requirement: Publish and verify take what the statement gives them

`tailrace publish <revision>` SHALL take the revision, `--branch` (by
default `main`) and `--component` (by default every component the
landing touched); `tailrace verify <version>` SHALL take the version,
`--component` (needed only where more than one is declared), `--branch`
and `--policy <audience>` (by default acceptance). No command SHALL take
a version to publish (21, core.13).

#### Scenario: No version given to a publish

- **WHEN** `tailrace publish` is given a `--version` argument
- **THEN** it exits 2 naming the argument it cannot read, and the core is not asked

### Requirement: Status needs no seam

`tailrace status` SHALL answer in any checkout of a repository, bound to
the landing, the policy and the ledger alone, by the host's bindings
file or, where the host has none, as 118 binds a host with no file, and
SHALL ask no store, signer or target (60). It SHALL take `--branch` (by
default `main`) for the channels, `--component` to narrow both parts,
`--revision` to narrow the record's part, and `--json`. A binding that
cannot be made SHALL be the channels' answer, in its own form, and the
record's part SHALL still answer. Status SHALL exit 0 whenever it
answers, each part carrying its own answer, and exit with the code of
its answer only where it cannot read the repository at all.

#### Scenario: Status from the installed binary

- **WHEN** the built binary runs `tailrace status --json` in a repository whose host record holds one entry
- **THEN** it prints that entry's component, version, revision and index's digest in the record's part, the channels in theirs, the environments' part unreadable, and exits 0

#### Scenario: A broken bindings file

- **WHEN** the built binary runs `tailrace status --json` on a host whose bindings file is not well formed
- **THEN** the channels' part is unreadable naming the file with the parser's words, the record's part answers, and the exit is 0

#### Scenario: Status in no repository

- **WHEN** the built binary runs `tailrace status` in a directory that is no git repository
- **THEN** it exits 3 naming the directory

### Requirement: An unbound host answers absent

A host with no bindings file is bound by 118 as a host with no file;
of that binding, status is bound to the branch, or sluice where the
repository is set up for it, and the git ref ledger, while `publish`
and `verify` SHALL still answer absent, naming the bindings file's
path, and SHALL offer, read or write nothing, until the rest of 118's
binding is built. Where the file is there, `publish` and `verify` SHALL
be bound to the seams it names, and render a broken file's unreadable
answer in their own form, as text or with `--json` (58, 59).

#### Scenario: Publish with nothing bound

- **WHEN** the built binary runs `tailrace publish HEAD` in a repository on a host with no bindings file
- **THEN** it exits 4 naming the bindings file's path, and no tag, file or record is written

#### Scenario: A broken file as JSON

- **WHEN** the built binary runs `tailrace verify 0.1.0 --json` on a host whose bindings file is not well formed
- **THEN** it prints one JSON document saying unreadable, naming the file and carrying the parser's words, and exits 3

#### Scenario: Status with no file

- **WHEN** the built binary runs `tailrace status --json` on a host with no bindings file, in a repository whose host record holds one entry and whose ledger refs hold a promotion
- **THEN** it prints that entry and the promoted channel, and exits 0

### Requirement: Sync fills the checkout's cache from a landed revision

`tailrace sync [revision]` SHALL take the revision whose verdicts name
the entries, by default the newest revision of the branch the checkout
descends from, `--branch` (by default `main`) and `--json`; it SHALL be
bound by the host's bindings file as verify is, never making the store
it names, and SHALL answer each build identity as rendered, absent, or
the refusal or unreadable answer its render gave, exiting 0 when the
only entries not rendered are absent (54, 57).

#### Scenario: A sync with one entry gone

- **WHEN** `tailrace sync --json` runs over the fakes for a landed revision whose verdicts name two build identities, one of which the store holds no entry for
- **THEN** the printed JSON names one as rendered and the other as absent, and the exit is 0

#### Scenario: A sync on an unbound host

- **WHEN** the installed binary runs `tailrace sync` on a host with no bindings file
- **THEN** it answers absent naming the bindings file's path, exits 4, and writes nothing

### Requirement: Push takes a landed revision

`tailrace push [revision]` SHALL take the landed revision whose
archives it pushes, by default the local branch's head, `--branch` (by
default `main`) and `--json`; it SHALL be bound by the host's bindings
file, needing no `[signing]`, and SHALL answer each build identity as
written, held already, absent, or the refusal or unreadable answer its
write gave, exiting 0 when the only build identities not written or
held are absent (54, 57).

#### Scenario: A push with one archive missing

- **WHEN** `tailrace push --json` runs over the fakes for a landed revision whose verdicts pass on two build identities, one of which has no archive on the host
- **THEN** the printed JSON names one as written with its digest and the other as absent, its JSON is the core's answer value for value, and the exit is 0

#### Scenario: A push on an unbound host

- **WHEN** the built binary runs `tailrace push` on a host with no bindings file
- **THEN** it answers absent naming the bindings file's path, exits 4, and writes nothing

### Requirement: Rebuild takes a landed revision

`tailrace rebuild [revision]` SHALL take the landed revision whose
policy file declares the components, by default the local branch's
head, `--branch` (by default `main`) and `--json`; it SHALL be bound by
the host's bindings file as verify is, never making the store it names,
and SHALL answer each entry it rebuilt as written or held, or the
refusal its write gave (57, 63).

#### Scenario: Rebuild after the record is deleted

- **WHEN** a version is published over the fakes, the host's record is deleted, and `tailrace rebuild --json` runs
- **THEN** it prints the version's entry as written, exits 0, and `tailrace status --json` then prints what it printed before the deletion

### Requirement: Promote takes a version and a channel

`tailrace promote <version> --to <channel>` SHALL take the version,
bare of its component's prefix, the channel, `--from <version>` where
the asker expects the channel at a version, `--component` (needed only
where more than one is declared), `--branch` (by default `main`) and
`--json`; `tailrace promote --back --to <channel>` SHALL take no version
and promote the version the channel's head moved from. It SHALL be
bound by the host's bindings file, needing `[signing]` and `[ledger]`,
and never making the store it names. It SHALL print the channel, the
version and index moved to and from, the promotion claim's digest and
its predecessor's, and how the store followed, with the store's own
answer beside where it was not written (33, 57).

#### Scenario: A promotion as JSON

- **WHEN** `tailrace promote 0.2.0 --to stable --json` runs over the fakes for an accepted version
- **THEN** the printed JSON is the core's answer value for value, naming the claim's digest, and the exit is 0

#### Scenario: A stale start

- **WHEN** `tailrace promote 0.3.0 --to stable --from 0.1.0` runs while `stable` is at 0.2.0
- **THEN** it exits 1 naming 0.2.0 as where `stable` is

#### Scenario: A version and --back together

- **WHEN** `tailrace promote 0.2.0 --back --to stable` is run
- **THEN** it exits 2 naming the arguments it cannot take together, and the core is not asked

### Requirement: Status names the page

`tailrace status --page` SHALL answer the address of the checkout's
repository's own page, `http://127.0.0.1:7420/<owner>/<name>/status`, after naming the
repository to the host and finding or starting the host's serving
process; it SHALL answer unreadable where the repository has no remote
that names an owner and a name, or the port answers as something else,
and SHALL exit with its answer's code (57, 114, 132).

#### Scenario: The page's address

- **WHEN** `tailrace status --page --json` runs in a checkout whose landing's remote is `https://example.com/o/r.git`
- **THEN** it prints the address ending `/o/r/status` and exits 0

### Requirement: Serve is a command

`tailrace serve` SHALL take no argument that moves its address, and
SHALL run as the host's serving process until it is idle, exiting 0
where the host's process already holds the lock, and 3 naming the
address and the system's words where the port cannot be bound (114).

#### Scenario: An argument that would move it

- **WHEN** `tailrace serve --port 8080` is run
- **THEN** it exits 2 naming the argument it cannot read, and binds nothing

### Requirement: Policy shows the policy in force

`tailrace policy` SHALL take `--branch` (by default `main`) and
`--json`, and SHALL print, at the branch's landed head, the policy for
acceptance and for a layout and every channel the channels answer
lists, the givens beside the repository's own, each with its component,
how it moves, whether a following channel takes prereleases, the claims
its policy requires, its promoters and who may ask for it (C.10, 131,
82); and beneath that table, each given held off, with how it would
move and the line that would switch it on, as the same channels answer
carries them apart from the channels in force, so a reader sees what
one line would give them (C.10 as amended at agentplot/blueprints
b1267d7, 131 as amended at 1ae1cc5). Each part SHALL carry its own
answer, so an absent layout, a channel whose policy cannot be read, or
an unreadable channels answer leaves the rest shown, the last shown
once in place of the channels and the givens held off (58). It
SHALL be bound as status is, asking the landing and the policy alone,
and SHALL write nothing (60, core.32). It SHALL exit 0 whenever an
answer is made, and with the answer's own code where the branch's head
cannot be resolved or has not landed.

#### Scenario: The givens beside a line

- **WHEN** `tailrace policy --json` runs in a repository whose landed head holds a policy file with `[acceptance]` and `[channels.stable]` requiring a rebuild
- **THEN** the printed JSON is the core's answer value for value, listing `beta` moving on each publish with releases alone and `stable` bound to `beta` requiring the gate's acceptance and then the rebuild, and the exit is 0

#### Scenario: The givens held off, beneath the table

- **WHEN** `tailrace policy` runs at a head whose file writes no channel and declares no component
- **THEN** beneath `beta` and `stable` it shows `alpha`, each publish with prereleases too, switched on by `[channels.alpha]`, and `lts`, bound to `stable`, switched on by `[channels.lts]`, and with `--json` the same channels answer, held givens and all, value for value with the core's

#### Scenario: A given switched on leaves the held part

- **WHEN** `tailrace policy` runs at a head whose file holds `[channels.alpha]`
- **THEN** `alpha` is shown among the channels in force and not beneath them, and `lts` alone is shown held off

#### Scenario: alpha and its approvers shown

- **WHEN** `tailrace policy` runs at a head whose file holds `[channels.alpha]` and `[channels.stable]` naming `approvers = ["ci"]`
- **THEN** it shows `alpha` moving on each publish with prereleases, and `stable` with `ci` as who may ask

#### Scenario: No layout policy

- **WHEN** `tailrace policy` runs at a head whose file holds no `[layout]`
- **THEN** the layout is shown absent naming the audience, the channels are shown, and the exit is 0

#### Scenario: A head not landed

- **WHEN** the landing answers that the branch has not landed its head
- **THEN** it exits 1 naming the branch and the revision, and the policy is not asked
