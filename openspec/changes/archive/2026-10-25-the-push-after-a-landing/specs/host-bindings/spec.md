# Spec Delta

## MODIFIED Requirements

### Requirement: Only publish makes the layout

publish and push SHALL make the layout the file names when its path is
missing or an empty directory, and write into it thereafter; push SHALL
need no `[signing]`, since it signs nothing. verify, status, sync and
rebuild SHALL never make or write a layout (60, core.32): verify over a
layout that is not there SHALL answer unreadable with the operating
system's words.

#### Scenario: The first publish on a laptop

- **WHEN** the file names a layout path that does not exist and a landed revision is published
- **THEN** the layout is made there and holds the version's artifacts and index

#### Scenario: verify over no layout

- **WHEN** verify runs while the layout path the file names does not exist
- **THEN** it exits 3 naming the layout, and no directory is made

#### Scenario: The first push on a landing host

- **WHEN** the file names a layout path that does not exist and no `[signing]`, and a landed revision is pushed
- **THEN** the layout is made there and holds the landing's cache entries

#### Scenario: rebuild over no layout

- **WHEN** rebuild runs while the layout path the file names does not exist
- **THEN** it exits 3 naming the layout, and no directory is made
