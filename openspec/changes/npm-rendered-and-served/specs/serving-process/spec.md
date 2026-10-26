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
- **WHEN** `GET /agentplot/consumer/cargo/index` is asked and the checkout's cargo projection serves none
- **THEN** the answer is `404` naming the ecosystem, and nothing is rendered
