## ADDED Requirements

### Requirement: The process serves each served layout under its repository's path
The host's one serving process SHALL answer a request under
`/<owner>/<name>/<ecosystem>/` by handing it, with the base address it
was asked at, to the projection the checkout that last named the
repository binds for that ecosystem, over that checkout's layout of it,
and SHALL answer the projection's response as it is, bytes included
(86, 114, 132). A version the layout lacks SHALL be rendered by the core
on the request, in that checkout (115). The route SHALL name no
ecosystem: which ecosystems are served is which projections the checkout
binds (core.1). The process SHALL hold no layout of its own and start no
second process (core.36). A repository none named SHALL be answered
absent as its status is; an ecosystem the checkout binds no projection
for, or one whose projection serves none, SHALL be `404` naming the
ecosystem; a checkout that cannot be bound SHALL be its binding's answer
(58).

#### Scenario: A served layout asked through the process
- **WHEN** a checkout of `agentplot/consumer` has synced its Cargo layout and named its repository, and `GET /agentplot/consumer/cargo/config.json` is asked of the process
- **THEN** the answer is the Cargo projection's response from that checkout's layout, with `application/json`, naming downloads under `http://127.0.0.1:7420/agentplot/consumer/cargo`

#### Scenario: A crate's bytes through the process
- **WHEN** a crate the layout holds is asked through the process
- **THEN** the response's body is the crate's bytes exactly, and its length is theirs

#### Scenario: A refused version through the process
- **WHEN** a version the layout lacks and the policy refuses is asked through the process
- **THEN** the answer is `403` carrying the version and the claim found wanting, and the layout is unchanged

#### Scenario: An ecosystem nothing serves
- **WHEN** `GET /agentplot/consumer/file/notes.txt` is asked, the checkout's file projection serving none
- **THEN** the answer is `404` naming the ecosystem, and nothing is rendered

#### Scenario: The status routes kept
- **WHEN** `GET /agentplot/consumer/status.json` is asked of a process that also serves layouts
- **THEN** the answer is the repository's status, as before
