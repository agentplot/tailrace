## ADDED Requirements

### Requirement: A push before the landing binds with a store alone

`tailrace push --left` SHALL bind from the host's file with `[store]`
alone: a file naming no `[landing]` and no `[signing]` SHALL bind, the
landing answering absent naming its binding for every operation; it
SHALL make the layout the file names as push does. Every other command
SHALL still answer absent naming `[landing]` where the file names none.

#### Scenario: A check job's file

- **WHEN** the file names `[store] registry` and nothing else, and `tailrace push --left` runs
- **THEN** the push binds and writes the checkout's entries into that repository, and the file names no credential

#### Scenario: The same file for a publish

- **WHEN** the same file is read for `tailrace publish`
- **THEN** publish answers absent naming `[signing]`, and nothing is read or written
