## ADDED Requirements

### Requirement: The file names what tells the gate

The bindings file SHALL name what tells the host's gate of an offered
version in a `[gate]` table, by one key, in the form every other table
of the file takes (118, 130, C.16): `publish = {}`, the host's own
publish step tells the host's own gate; or `job = "<words>"`, a job
outside tailrace, woken by a store's or a forge's event, tells a gate
elsewhere, and the words say which. The core SHALL name neither (130).
tailrace SHALL run nothing for `job`, and SHALL carry its words as
written. A `[gate]` naming no key, two keys, a key other than these, a
`publish` that is not an empty table, or a `job` that is not a string
with words in it SHALL be unreadable naming the file and the table, and
SHALL leave nothing written for every command that reads the file (58,
59, core.19).

#### Scenario: The host's publish tells its own gate

- **WHEN** the bindings file holds `[gate]` with `publish = {}`
- **THEN** the binding says the host's publish step tells the host's gate

#### Scenario: A job tells a separated gate

- **WHEN** the bindings file holds `[gate]` with `job = "the release job a push to main wakes"`
- **THEN** the binding says a job tells the gate, carrying those words as written, and nothing of tailrace starts or reaches that job

#### Scenario: Two tellers

- **WHEN** `[gate]` holds both `publish = {}` and `job = "…"`
- **THEN** publish and verify answer unreadable naming `[gate]` and both keys, and the layout the file names is not made

#### Scenario: A teller the file does not know

- **WHEN** `[gate]` holds `webhook = "…"`
- **THEN** the answer is unreadable naming `[gate]`, `webhook`, and the keys the table reads, `publish` and `job`

#### Scenario: A publish teller with options

- **WHEN** `[gate]` holds `publish = { now = true }`
- **THEN** the answer is unreadable naming `[gate] publish`, which takes no options

#### Scenario: A job with no words

- **WHEN** `[gate]` holds `job = ""`
- **THEN** the answer is unreadable naming `[gate] job`

### Requirement: A file naming nothing to tell the gate binds no teller

A bindings file with no `[gate]` table SHALL bind no teller: the binding
SHALL say that nothing on this host tells the gate, naming `[gate]` in
the file, and every command SHALL bind otherwise as it does without one
(130).

#### Scenario: No gate table

- **WHEN** the bindings file names a store, a landing and a signer and holds no `[gate]`
- **THEN** the binding binds them as before and says nothing tells the gate, naming `[gate]` in the file's path
