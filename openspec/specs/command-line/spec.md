# command-line Specification

## Purpose
The command line as tailrace's first surface: `tailrace publish`,
`tailrace verify` and `tailrace status` each call the one core and
render its answer, as text for a person or as the answer itself for a
program.

## Requirements

### Requirement: Each command calls the one core and renders its answer

`tailrace publish`, `tailrace verify` and `tailrace status` SHALL each
turn their arguments into one question to the core and render the
core's answer; no command SHALL read a seam, a file of the record or the
history itself, or compute a version, a digest or a signer of its own
(56, core.31).

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
