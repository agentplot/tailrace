## ADDED Requirements

### Requirement: One process answers a host at one fixed address

`tailrace serve` SHALL answer HTTP at `127.0.0.1` port `7420`, the
core's name and port, which no setting, flag or variable moves, on
every host (114, core.36). It SHALL answer `GET /` with a JSON document
naming itself as tailrace's serving process, its revision and its
process id, so an asker tells it from anything else on the port.

#### Scenario: The address on every host

- **WHEN** `tailrace serve` runs on a host
- **THEN** it answers `GET http://127.0.0.1:7420/` with a document naming tailrace, its revision and its process id, and no flag or setting is read that would move the address

### Requirement: The repository is in the path, by its remote's owner and name

The process SHALL answer a repository at `/<owner>/<name>/`, its owner
and name the last two parts of the path of the address its landing's
remote has in the repository's git configuration, without a trailing
`.git`, never a directory or a name an operator chose (49, 114). A
remote whose address does not end in an owner and a name SHALL make the
repository unnamed, unreadable naming the remote and its address.

#### Scenario: An HTTPS and an SSH remote

- **WHEN** the landing's remote is `https://example.com/agentplot/tailrace.git`, and in another checkout `git@example.com:agentplot/tailrace`
- **THEN** both are named `agentplot/tailrace`

#### Scenario: A remote that names no owner

- **WHEN** the landing's remote is `/srv/tailrace.git`
- **THEN** the repository is unreadable naming the remote and that address, and no entry is written

### Requirement: The process answers the repositories a checkout named

The host SHALL keep one file of the repositories named to its serving
process, at `$XDG_STATE_HOME/tailrace/repositories`, or
`~/.local/state/tailrace/repositories` where that variable is unset or
empty, readable with any file reader, one entry per repository naming
its owner and name and the path of the checkout that last named it
(62, 132). The process SHALL read the file on each request, SHALL
answer a repository no entry names absent, saying no command run in a
checkout of it on this host has named it, and an entry whose checkout
is gone unreadable naming the path (58).

#### Scenario: A repository none named

- **WHEN** `GET /agentplot/tailrace/status.json` is asked and the file names no `agentplot/tailrace`
- **THEN** the answer is absent, status 404, naming the repository and saying no checkout on this host has named it

#### Scenario: The last checkout to name it

- **WHEN** two checkouts of one repository name it in turn
- **THEN** the file holds one entry for the repository, naming the second checkout

### Requirement: The page and its JSON are the status answer

The process SHALL answer `GET /<owner>/<name>/status.json` with the
status answer the core gives in the named checkout, as `tailrace
status --json` prints it there, and `GET /<owner>/<name>/status` with
a page rendering that answer: each channel with its component, how it
moves, its version or that it is empty, its chain, and the record of
what this host published, every refusal and unreadable answer shown
with the source's own words (57, 58, 59, 132). Each request SHALL
compute the answer anew, and SHALL write nothing and ask no store,
signer or target (60). A path the process does not answer SHALL be 404,
and a method other than `GET` 405.

#### Scenario: The page and the JSON agree

- **WHEN** the page and the JSON of one named repository are asked
- **THEN** the JSON is the status answer in that checkout, and the page names every channel, version and promotion the JSON names

#### Scenario: A promotion after the page was read

- **WHEN** a channel is promoted after the page was asked, and the page is asked again
- **THEN** the second page shows the new version, with no restart and no timer between

### Requirement: The first asker that finds none starts it, and one lock keeps it one

`tailrace status --page` SHALL name the checkout's repository in the
host's file, find the serving process at the core's address, start
`tailrace serve` detached from its terminal and from the checkout where
none answers, wait for it to answer, and print the page's address; a
port that answers as something else SHALL be unreadable naming the
address (114). The process SHALL hold the host's one lock,
`tailrace/serve.lock` beside the file of repositories, for as long as
it runs: a `tailrace serve` that finds the lock held SHALL exit 0
saying the host's process answers at the address, and no asker SHALL
start a second (114, as sluice 129). The process SHALL end when nothing
has asked it for ten minutes. A host may run it as a service of its
own, and nothing SHALL tell a person to start one.

#### Scenario: Two askers on one host

- **WHEN** two askers find no process at once and each starts one
- **THEN** one process holds the lock and answers, the other exits saying the first answers, and both askers print the same address answered by one process id

#### Scenario: A second serve

- **WHEN** `tailrace serve` runs while another holds the lock
- **THEN** it exits 0 naming the address the first answers at, and binds nothing

#### Scenario: Idle

- **WHEN** nothing asks the process for the idle time
- **THEN** it exits and lets the lock go, and the next asker starts one again

#### Scenario: Something else on the port

- **WHEN** the port answers `GET /` with something other than tailrace's document
- **THEN** `tailrace status --page` is unreadable naming the address, and starts nothing
