## ADDED Requirements

### Requirement: A landing read as last fetched

The landing's group SHALL hold every landing that reads a remote to 77
read from a copy: read once, made unreachable and asked again, it
answers as before from the copy, saying when and with the remote's
words, and a revision landed after the copy was fetched is not landed
from the copy (58, 59, 61, 77, 78). For a rig whose landing reads no
remote, the scenario SHALL be not applicable.

#### Scenario: Read again from the copy

- **WHEN** a landed revision's landed and verdicts-of are read, the landing is made unreachable, and both are read again
- **THEN** both answer as first read, each saying it was read from the copy at a time no earlier than the first read, with the words the rig made it unreachable with

#### Scenario: A revision landed after the copy

- **WHEN** a revision is landed after the landing was last read, the landing is then made unreachable, and landed is asked of it
- **THEN** it answers not landed from the copy, never landed and never refused

#### Scenario: A landing that reads no remote

- **WHEN** the group runs over a rig that says its landing reads no remote
- **THEN** the report names a landing read as last fetched not applicable, with the rig's reason

### Requirement: Not landed as last fetched over the fakes

The publish group SHALL hold publish to 77's refusal over the fakes: a
revision the landing answers not landed from its copy is refused naming
the copy's time and the remote's words before any seam but the landing
is asked, and one it answers landed from its copy is published as one
landed on its remote (14, 59, 77, core.11).

#### Scenario: Refused from the copy

- **WHEN** the fake landing answers not landed from its copy, and the revision is published
- **THEN** the publish is refused naming the branch, the revision, the copy's time and the remote's words, saying to ask again, and no seam but the landing is asked

#### Scenario: Published from the copy

- **WHEN** the fake landing answers landed from its copy, and the revision is published in two worlds alike but that one answers from its remote
- **THEN** the two publishes answer the same

### Requirement: Status's landing as last fetched over the fakes

The status group SHALL hold status to a landing read from its copy over
the fakes: every channel shown as for a landing read from its remote,
the channels' part saying the landing was read as last fetched, when,
and with the remote's words (36, 60, 77).

#### Scenario: Channels from a landing's copy

- **WHEN** the fake landing answers the branch's head landed from its copy, and status is asked
- **THEN** every channel the policy declares is shown as in a world alike whose landing answers from itself, and the channels' part names the copy's time and the remote's words verbatim
