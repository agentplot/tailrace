## ADDED Requirements

### Requirement: Sync makes a served layout reachable
After the layouts are rendered, `tailrace sync` SHALL answer, for each
layout whose projection serves it, the address its clients read it at,
and, where there is one, SHALL name the checkout's repository to the
host and find the host's serving process at the core's address or start
it where none answers, as `tailrace status --page` does, and answer that
it serves or why it cannot (114, 132). A layout whose projection serves
none SHALL name no address and start nothing. A repository whose landing
remote names no owner and name SHALL be answered unreadable naming the
remote and its address beside the layouts, and nothing named or started.
The exit SHALL count that answer as it counts a layout's (57).

#### Scenario: A sync of an npm layout
- **WHEN** `tailrace sync --json` runs in a checkout of `agentplot/consumer` whose npm projection rendered a layout, with no process answering at the address
- **THEN** the answer names the layout's address `http://127.0.0.1:7420/agentplot/consumer/npm/`, the host's file names the checkout for `agentplot/consumer`, one process was started, and a second sync starts none

#### Scenario: A sync with no served layout
- **WHEN** `tailrace sync` runs in a checkout whose only rendered layouts are of projections that serve none
- **THEN** the answer names no address, the host's file is unchanged, and no process is started

#### Scenario: A remote that names no owner
- **WHEN** `tailrace sync` renders an npm layout in a checkout whose landing remote is `/srv/consumer.git`
- **THEN** the layout is rendered, the answer is unreadable naming the remote and its address in place of the address, and the exit is 3
