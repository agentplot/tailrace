## ADDED Requirements

### Requirement: The fake landing answers as last fetched

The fake landing SHALL keep, as its copy, what it held when it last
answered, and once made unreachable SHALL answer every question from
that copy, saying so, with the time of that answer and the words it was
made unreachable with; made unreachable before it answered anything, it
SHALL answer unreadable with those words (77, C.5, C.6). While
reachable, every answer SHALL say it was read from the landing itself.

#### Scenario: Unreachable after a read

- **WHEN** the fake landing answers a landed revision, is made unreachable with the words `remote hung up`, and is asked again
- **THEN** it answers landed from the copy, with the time of the first answer and `remote hung up`

#### Scenario: A landing after the last answer

- **WHEN** the fake landing answers of `main`, lands another revision on it, is made unreachable, and is asked of that revision
- **THEN** it answers not landed from the copy, with the time of its last answer

#### Scenario: Unreachable before any answer

- **WHEN** the fake landing is made unreachable before it answered anything, and landed and verdicts-of are asked
- **THEN** both answer unreadable with the words it was made unreachable with
