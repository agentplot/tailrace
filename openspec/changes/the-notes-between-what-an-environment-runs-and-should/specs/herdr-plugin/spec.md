## ADDED Requirements

### Requirement: A row shows every channel's head and the environments' counts as counted

Each row SHALL name the repository by its owner and name. It SHALL show
each channel's component, name, and the version it is at or that it is
empty. It SHALL show the environments' four counts as the process
answers them: running what they should, behind, waiting on a person,
and cannot be read (132).

#### Scenario: A repository with two channels

- **WHEN** a row's heads name `beta` at version `1.2.0` and `stable` empty, and its environments answer counts of 3, 1, 1 and 0
- **THEN** the row shows `beta 1.2.0`, `stable` empty, and the four counts under their four words

#### Scenario: Environments whose part cannot be read

- **WHEN** a row's environments are answered unreadable because the landing cannot be read
- **THEN** the row shows that they cannot be read, with the process's words, and shows no count, zero included

#### Scenario: A repository that declares no environment

- **WHEN** a row's environments answer counts of 0, 0, 0 and 0
- **THEN** the row shows the four counts as answered

## MODIFIED Requirements

### Requirement: Opening a row shows that repository whole

Opening a row SHALL show the repository's own answer as the process
gives it at its path. That means the branch and the landed revision the
policy was read at, and how the landing was read. It means each channel
with its version or that it is empty, and the chain of promotions that
put it there, newest first. It means the environments' four counts and
every environment with its state, what it should run, what it runs and
since when or why that cannot be read, what it waits on, and beside one
that is behind the notes of every version between. It also means what
this host published. A key SHALL return to every repository.

#### Scenario: A row opened and closed

- **WHEN** the person opens the row of `o/r`
- **THEN** the pane shows `o/r`'s branch, landed revision, each channel with its version and its chain newest first (the version moved from, who asked, when, and the claim), each environment and what this host published, and the back key returns to every repository

#### Scenario: A repository whose checkout is gone

- **WHEN** the person opens the row of a repository the process answers absent at its own path
- **THEN** the pane shows the absence with the process's words, and the back key returns to every repository

#### Scenario: An environment behind

- **WHEN** the person opens a repository whose answer holds `srv` behind, running 0.2.0 where its channel is at 0.4.0, with the notes of 0.3.0 and 0.4.0
- **THEN** the pane shows `srv` behind, what it runs and since when, what it should run, and both versions' notes beneath it, as the answer gives them

#### Scenario: An environment that cannot be read

- **WHEN** the person opens a repository whose answer holds `desk` unread, saying no counted report names it
- **THEN** the pane shows `desk` as cannot be read with those words, and neither as running what it should nor as behind

## REMOVED Requirements

### Requirement: A row shows every channel's head and the environments' counts

**Reason**: The environments are read in status (98, 132 as amended at
agentplot/blueprints `90cf023`; record 0029), so no row answers them
unreadable for want of a target.

**Migration**: "A row shows every channel's head and the environments'
counts as counted" holds the row.
