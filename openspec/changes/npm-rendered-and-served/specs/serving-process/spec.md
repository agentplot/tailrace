## ADDED Requirements

### Requirement: The process serves each served layout under its repository's path
The host's one serving process SHALL answer a request under
`/<owner>/<name>/<ecosystem>/` by handing it, with the base address it
was asked at, to the projection the checkout that last named the
repository binds for that ecosystem, over that checkout's layout of it,
and SHALL answer the projection's response as it is, bytes included
(86, 114, 132). A version the layout lacks SHALL be rendered by the core
on the request, in that checkout (115). The process SHALL hold no layout
of its own and start no second process (core.36). A repository none
named SHALL be answered absent as its status is; an ecosystem the
checkout binds no projection for, or one whose projection serves none,
SHALL be `404` naming the ecosystem; a checkout that cannot be bound
SHALL be its binding's answer (58).

#### Scenario: A served layout asked through the process
- **WHEN** a checkout of `agentplot/consumer` has synced its npm layout and named its repository, and `GET /agentplot/consumer/npm/@family%2fwidget` is asked of the process
- **THEN** the answer is the npm projection's response from that checkout's layout, with `application/json`

#### Scenario: A tarball's bytes through the process
- **WHEN** a tarball the layout holds is asked through the process
- **THEN** the response's body is the tarball's bytes exactly, and its length is theirs

#### Scenario: Two checkouts on one host
- **WHEN** two checkouts of one repository on one host each sync, and each one's `npm install` asks the host's address
- **THEN** both are answered by one process, of one process id, and neither checkout started one of its own

#### Scenario: An ecosystem nothing serves
- **WHEN** `GET /agentplot/consumer/file/notes.txt` is asked, the checkout's file projection serving none
- **THEN** the answer is `404` naming the ecosystem, and nothing is rendered

## MODIFIED Requirements

### Requirement: The first asker that finds none starts it, and one lock keeps it one

`tailrace status --page` SHALL name the checkout's repository in the
host's file, find the serving process at the core's address by its
`/serving.json`, start `tailrace serve` detached from its terminal and
from the checkout where none answers, wait for it to answer, and print
the address of the repository's own page; a port that answers as
something else SHALL be unreadable naming the address (114). The
process SHALL hold the host's one lock, `tailrace/serve.lock` beside
the file of repositories, for as long as it runs: a `tailrace serve`
that finds the lock held SHALL exit 0 saying the host's process answers
at the address, and no asker SHALL start a second (114, as sluice 129).
The process SHALL end when nothing has asked it for ten minutes and no
checkout the host's file names holds a rendered layout whose projection
serves it; while one does, the process SHALL stay, since the
ecosystem's client cannot start it, and SHALL end at its next idle time
once the layout or its checkout is gone (114, 115). A host may run it
as a service of its own, and nothing SHALL tell a person to start one.

#### Scenario: Two askers on one host

- **WHEN** two askers find no process at once and each starts one
- **THEN** one process holds the lock and answers, the other exits saying the first answers, and both askers print the same address answered by one process id

#### Scenario: A second serve

- **WHEN** `tailrace serve` runs while another holds the lock
- **THEN** it exits 0 naming the address the first answers at, and binds nothing

#### Scenario: Idle

- **WHEN** nothing asks the process for the idle time and no checkout the host's file names holds a served layout
- **THEN** it exits and lets the lock go, and the next asker starts one again

#### Scenario: Idle while a layout is served

- **WHEN** nothing asks the process for the idle time, and the checkout the host's file names for `agentplot/consumer` holds a rendered npm layout
- **THEN** the process still answers, and a request for that layout after the idle time is answered by the same process id

#### Scenario: The served layout removed

- **WHEN** the process stays for a served layout and that layout's directory is then removed, or the checkout holding it is
- **THEN** the process exits at its next idle time and lets the lock go

#### Scenario: A layout of a projection that serves none

- **WHEN** nothing asks the process for the idle time and the only rendered layouts of named checkouts are of projections that serve none
- **THEN** it exits as when no layout is held

#### Scenario: Something else on the port

- **WHEN** the port answers `GET /serving.json` with something other than tailrace's document
- **THEN** `tailrace status --page` is unreadable naming the address, and starts nothing
