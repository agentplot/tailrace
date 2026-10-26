## MODIFIED Requirements

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

## ADDED Requirements

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
