# command-line Specification

## Purpose
The command line as tailrace's first surface: `tailrace publish`,
`tailrace verify` and `tailrace status` each call the one core and
render its answer, as text for a person or as the answer itself for a
program.

## Requirements

### Requirement: Each command calls the one core and renders its answer

`tailrace publish`, `tailrace verify`, `tailrace status`, `tailrace
sync`, `tailrace push` and `tailrace rebuild` SHALL each turn their
arguments into one question to the core and render the core's answer; no
command SHALL read a seam, a file of the record or the history itself,
or compute a version, a digest or a signer of its own (56, core.31).

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

`tailrace status` SHALL answer in any checkout of a repository with no
seam bound, from the host's record alone, taking `--component` and
`--revision` to narrow it (60).

#### Scenario: Status from the installed binary

- **WHEN** the built binary runs `tailrace status --json` in a repository whose host record holds one entry
- **THEN** it prints that entry's component, version, revision and index's digest and exits 0

### Requirement: An unbound host answers absent

A host with no bindings file binds nothing (118): the installed binary
SHALL answer `publish` and `verify` absent, naming the bindings file's
path, and SHALL offer, read or write nothing. Where the file is there,
`publish` and `verify` SHALL be bound to the seams it names, and render
a broken file's unreadable answer in their own form, as text or with
`--json` (58, 59).

#### Scenario: Publish with nothing bound

- **WHEN** the built binary runs `tailrace publish HEAD` in a repository on a host with no bindings file
- **THEN** it exits 4 naming the bindings file's path, and no tag, file or record is written

#### Scenario: A broken file as JSON

- **WHEN** the built binary runs `tailrace verify 0.1.0 --json` on a host whose bindings file is not well formed
- **THEN** it prints one JSON document saying unreadable, naming the file and carrying the parser's words, and exits 3

#### Scenario: Status with no file

- **WHEN** the built binary runs `tailrace status --json` on a host with no bindings file, in a repository whose host record holds one entry
- **THEN** it prints that entry and exits 0

### Requirement: Sync fills the checkout's cache from a landed revision
`tailrace sync [revision]` SHALL take the revision whose verdicts name
the entries and whose policy judges the layouts, by default the newest
revision of the branch the checkout descends from, `--branch` (by
default `main`) and `--json`; it SHALL be bound by the host's bindings
file as verify is, trusting the host's own signer and the signers the
file trusts, never making the
store it names, and SHALL answer,
as one answer of the core, each build identity as rendered, absent, or
the refusal or unreadable answer its render gave, and each layout as
refreshed, naming every pin held, refused with its claim found wanting,
or absent, or as what it still holds and when it was rendered beside
what could not be read. It SHALL exit 0 when the only entries and pins
not rendered or held are absent, and otherwise with the worst code
among them (51, 54, 57, 99).

#### Scenario: A sync with one entry gone
- **WHEN** `tailrace sync --json` runs over the fakes for a landed revision whose verdicts name two build identities, one of which the store holds no entry for
- **THEN** the printed JSON names one as rendered and the other as absent, and the exit is 0

#### Scenario: A sync on an unbound host
- **WHEN** the installed binary runs `tailrace sync` on a host with no bindings file
- **THEN** it answers absent naming the bindings file's path, exits 4, and writes nothing

#### Scenario: A sync with a version refused
- **WHEN** `tailrace sync --json` runs over the fakes in a checkout whose lockfile pins one version the policy admits, one it refuses and one the store does not name
- **THEN** the printed JSON is the core's answer value for value, naming the first held, the second refused with its version and the claim found wanting, and the third absent, and the exit is 1

#### Scenario: A sync whose layout cannot be refreshed
- **WHEN** `tailrace sync` runs over the fakes while the store's resolve is unreadable with the words `connection refused`, in a checkout whose layout was rendered before
- **THEN** it prints what the layout holds and when it was rendered, prints `connection refused` verbatim, and exits 3

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
