## RENAMED Requirements

- FROM: `### Requirement: An unbound host answers absent`
- TO: `### Requirement: A host with no bindings file is bound to the defaults`

## MODIFIED Requirements

### Requirement: A host with no bindings file is bound to the defaults

A host with no bindings file SHALL be bound as 118 binds it, to the
given store, landing and its remote, ledger, signer and teller
(host-bindings), for `publish`, `verify`, `sync`, `push`, `rebuild`,
`promote` and `status` alike. The installed binary SHALL answer absent
or unreadable for the binding only where a default cannot be made:
absent naming `XDG_CONFIG_HOME` and `HOME` where neither is set, and
unreadable naming the repository's remotes and `[landing]` in the
bindings file where the checkout has several and none is `origin`; it
SHALL then offer, read or write nothing. Where the file is there, each
command SHALL be bound to the seams it names and the given ones for the
tables it leaves out, and render a broken file's unreadable answer in
its own form, as text or with `--json` (58, 59).

#### Scenario: Publish with nothing bound

- **WHEN** the built binary runs `tailrace publish HEAD` in a repository on a host with no bindings file, whose branch has not landed `HEAD`
- **THEN** it exits 1 refusing `HEAD` as not landed, and no tag, layout, key, record or bindings file is written

#### Scenario: Verify with nothing written

- **WHEN** the built binary runs `tailrace verify 0.1.0 --json` on a host with no bindings file and no given layout yet
- **THEN** it exits 3 unreadable naming the given layout's path, and no directory is made

#### Scenario: Several remotes and no origin

- **WHEN** the built binary runs `tailrace publish HEAD` on a host with no bindings file, in a checkout whose remotes are `github` and `gitlab`
- **THEN** it exits 3 naming both remotes and `[landing]` in the bindings file, and nothing is written

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
bound as verify is, by the host's bindings file or the defaults where
it has none, never making the store it is bound to, and SHALL answer
each build identity as rendered, absent, or the refusal or unreadable
answer its render gave, exiting 0 when the only entries not rendered
are absent (54, 57, 118).

#### Scenario: A sync with one entry gone

- **WHEN** `tailrace sync --json` runs over the fakes for a landed revision whose verdicts name two build identities, one of which the store holds no entry for
- **THEN** the printed JSON names one as rendered and the other as absent, and the exit is 0

#### Scenario: A sync on an unbound host

- **WHEN** the built binary runs `tailrace sync --json` on a host with no bindings file and no given layout yet
- **THEN** it answers unreadable naming the given layout's path, exits 3, and writes no cache, layout or bindings file

### Requirement: Push takes a landed revision

`tailrace push [revision]` SHALL take the landed revision whose
archives it pushes, by default the local branch's head, `--branch` (by
default `main`) and `--json`; it SHALL be bound by the host's bindings
file or the defaults where it has none, needing no `[signing]` and
making no key, and SHALL answer each build identity as written, held
already, absent, or the refusal or unreadable answer its write gave,
exiting 0 when the only build identities not written or held are absent
(54, 57, 118).

#### Scenario: A push with one archive missing

- **WHEN** `tailrace push --json` runs over the fakes for a landed revision whose verdicts pass on two build identities, one of which has no archive on the host
- **THEN** the printed JSON names one as written with its digest and the other as absent, its JSON is the core's answer value for value, and the exit is 0

#### Scenario: A push on an unbound host

- **WHEN** the built binary runs `tailrace push --json` on a host with no bindings file, in a repository whose branch has not landed its head
- **THEN** it answers refused naming the head as not landed, exits 1, and writes no record, key or bindings file

### Requirement: Promote takes a version and a channel

`tailrace promote <version> --to <channel>` SHALL take the version,
bare of its component's prefix, the channel, `--from <version>` where
the asker expects the channel at a version, `--component` (needed only
where more than one is declared), `--branch` (by default `main`) and
`--json`; `tailrace promote --back --to <channel>` SHALL take no version
and promote the version the channel's head moved from. It SHALL be
bound by the host's bindings file, its `[signing]` and `[ledger]` or,
for each it leaves out or where there is no file, the given host key
and the git ref in the repository (118), never making the store it is
bound to. It SHALL print the channel, the version and index moved to
and from, the promotion claim's digest and its predecessor's, and how
the store followed, with the store's own answer beside where it was not
written (33, 57).

#### Scenario: A promotion as JSON

- **WHEN** `tailrace promote 0.2.0 --to stable --json` runs over the fakes for an accepted version
- **THEN** the printed JSON is the core's answer value for value, naming the claim's digest, and the exit is 0

#### Scenario: A stale start

- **WHEN** `tailrace promote 0.3.0 --to stable --from 0.1.0` runs while `stable` is at 0.2.0
- **THEN** it exits 1 naming 0.2.0 as where `stable` is

#### Scenario: A version and --back together

- **WHEN** `tailrace promote 0.2.0 --back --to stable` is run
- **THEN** it exits 2 naming the arguments it cannot take together, and the core is not asked

#### Scenario: A promote with no ledger or signing written

- **WHEN** `tailrace promote` runs on a host whose file names no `[signing]` and no `[ledger]`
- **THEN** the promotion is signed by the given host key as the host's name and appended to `refs/tailrace/ledger/<channel>` in the repository the command runs in, and nothing is pushed to a remote
