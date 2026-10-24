# command-line Specification

## MODIFIED Requirements

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
