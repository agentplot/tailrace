# Spec Delta

## Purpose

The herdr plugin's pane: beside a person's agents, every repository the
host knows and each one whole, read from the host's one serving process
and rendered as the core answered it (section 9, 56, 114, 132).

## ADDED Requirements

### Requirement: The pane opens beside the person's work on every repository

The plugin's action SHALL open the pane as a split beside the focused
herdr pane. The pane SHALL first show every repository the host's
serving process knows, one row each, in the order the process answers
them.

#### Scenario: The pane opens on every repository

- **WHEN** the action is invoked over a focused pane and the process knows two repositories
- **THEN** a split opens beside the focused pane showing two rows, one for each repository, in the process's order

#### Scenario: A host that knows no repository

- **WHEN** the process answers an empty list of repositories
- **THEN** the pane says the host knows no repository yet, and that a repository appears once a tailrace command run in a checkout of it names it

### Requirement: A row shows every channel's head and the environments' counts

Each row SHALL name the repository by its owner and name. It SHALL show
each channel's component, name, and the version it is at or that it is
empty. It SHALL show the environments' four counts as the process
answers them: running what they should, behind, waiting on a person,
and cannot be read (132).

#### Scenario: A repository with two channels

- **WHEN** a row's heads name `beta` at version `1.2.0` and `stable` empty, and its environments answer counts of 3, 1, 1 and 0
- **THEN** the row shows `beta 1.2.0`, `stable` empty, and the four counts under their four words

#### Scenario: Environments not read in phase 1

- **WHEN** a row's environments are answered unreadable because no target is read
- **THEN** the row shows that they cannot be read, with the process's words, and shows no count, zero included

### Requirement: Opening a row shows that repository whole

Opening a row SHALL show the repository's own answer as the process
gives it at its path. That means the branch and the landed revision the
policy was read at, and how the landing was read. It means each channel
with its version or that it is empty, and the chain of promotions that
put it there, newest first. It also means the environments' part and
what this host published. A key SHALL return to every repository.

#### Scenario: A row opened and closed

- **WHEN** the person opens the row of `o/r`
- **THEN** the pane shows `o/r`'s branch, landed revision, each channel with its version and its chain newest first (the version moved from, who asked, when, and the claim), the environments' part and what this host published, and the back key returns to every repository

#### Scenario: A repository whose checkout is gone

- **WHEN** the person opens the row of a repository the process answers absent at its own path
- **THEN** the pane shows the absence with the process's words, and the back key returns to every repository

### Requirement: The four answers are never confused

The pane SHALL render a value, an absence, a refusal and an unreadable
answer each in its own words, wherever one stands. That includes the
whole document, a row, a channel's head or chain, the environments, and
what was published. Each SHALL carry the source's own words verbatim
(58, 59).

#### Scenario: One channel's chain refused

- **WHEN** a repository's answer holds `beta` at a version and `stable`'s chain refused with the ledger's words
- **THEN** the pane shows `beta` and its chain, and `stable` as refused with the ledger's words verbatim, and neither as empty or absent

#### Scenario: Nothing published on this host

- **WHEN** what this host published is answered absent
- **THEN** the pane shows it as absent with the process's words, never as an empty list

### Requirement: The pane reads the host's one process at the core's address

The pane SHALL read only the host's one serving process, at the core's
fixed name and port: the root answer for every repository, and
`/<owner>/<name>/status.json` for one. No setting, flag or variable of
the plugin SHALL move that address (114, core.36). The pane SHALL ask
again on a key, and on its own every few seconds while it is open.

#### Scenario: The address is the core's

- **WHEN** the plugin's scripts are searched for the address they ask
- **THEN** they name the core's one name and port in one place, and read no variable or file for it

#### Scenario: The answer changes while the pane is open

- **WHEN** a channel moves while the pane is open
- **THEN** the pane shows the new version within a few seconds without a key pressed

### Requirement: The pane starts the process where none answers, as any asker does

Before reading, the pane SHALL ask the process to name itself. Where
nothing answers, the pane SHALL start `tailrace serve` once. The new
process is detached from the pane's terminal and signals, and its
working directory is never a checkout. The pane then asks again until
the process answers, up to a limit. Where something other than tailrace
answers, the pane SHALL say so with what answered, and start nothing
(114).

#### Scenario: No process answers

- **WHEN** the pane opens and nothing answers at the core's address
- **THEN** the pane starts `tailrace serve` exactly once, and shows every repository once the process answers

#### Scenario: A process answers

- **WHEN** the pane opens and tailrace's process answers
- **THEN** the pane starts nothing

#### Scenario: Something else holds the address

- **WHEN** something that is not tailrace's process answers at the core's address
- **THEN** the pane shows that something else answers there, with what it answered, and starts nothing

#### Scenario: The process never answers

- **WHEN** the pane started `tailrace serve` and nothing answers within the limit
- **THEN** the pane says the process it started did not answer, and starts no second one

#### Scenario: Two panes on one host

- **WHEN** two panes open at once on a host where nothing answers at the core's address
- **THEN** one process answers both, each pane names the same process, and both show the same answer

### Requirement: The plugin computes no answer of its own

The plugin SHALL render the documents the process answers and nothing
else. It SHALL NOT count environments, read a channel's head from its
chain, compare versions, or turn one answer into another. It SHALL NOT
read a store, a ledger, a policy or git, or run any tailrace command but
`tailrace serve` (56, core.31, core.32).

#### Scenario: Counts come from the answer

- **WHEN** a row's environment counts in the process's answer disagree with the repository's own environments' part
- **THEN** the row shows the row's counts as answered, and the opened repository shows its own part as answered

#### Scenario: Nothing but the process is asked

- **WHEN** the pane draws every repository and one repository with the process answering
- **THEN** it has run no `git`, no tailrace command and no store client, and has asked only the core's address

### Requirement: The plugin is kept outside the core

The plugin SHALL live in `plugins/herdr/`, a moon project of its own
whose check-fast tasks check the manifest and test the scripts. No crate
SHALL import, name or start anything of the plugin (section 9, core.1,
core.25).

#### Scenario: The gate checks the plugin

- **WHEN** `moon run ':#check-fast'` runs
- **THEN** the plugin's manifest check and its tests under sh and dash run as tasks of `plugin-herdr`, and its scripts are formatted and linted by the tree's formatter

### Requirement: The pane shows what status prints on a machine that never published

On a machine that never published, a repository's view SHALL show every
part of the answer the process gives at its path. That answer is what
`tailrace status --json` prints in its checkout. The root answer the
pane renders SHALL be the one the process makes for the same host (57,
132).

#### Scenario: The binary's own documents

- **WHEN** the root and a repository's answers are made for a scratch host whose repository declares two channels with promotions in the git ref ledger and has no record of a publish
- **THEN** they equal the documents the plugin's tests render, apart from times, revisions and digests, and the pane's frames of them show every channel, version, promotion, the environments' answer and the absent record
