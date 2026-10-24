# sluice-landing Specification

## Purpose
The landing seam bound to sluice's records (C.5): whether a branch
landed a revision, read from sluice's log for the branch, and the signed
run records that landed it, carried whole as sluice wrote them.

## Requirements

### Requirement: A revision is landed when sluice's log holds its landing

The sluice landing SHALL answer that a branch has landed a revision
only when sluice's log for the branch holds a landing recorded as done
whose commits include the revision: those on the first-parent line
from the revision it moved the branch to, back to the one it moved it
from (77, C.5; sluice 23). Any other revision, though the branch's
history holds it, SHALL be answered not landed.

#### Scenario: Each commit of a landing of two entries

- **WHEN** sluice lands two entries together, writing one landing commit for each, and landed is asked of each commit
- **THEN** both answer landed

#### Scenario: A commit sluice did not land

- **WHEN** landed is asked of a commit of the branch's history from before sluice ran the branch
- **THEN** it answers not landed, as an answer and not a refusal

#### Scenario: A landing begun and not finished

- **WHEN** the log holds a landing begun and no record that it was done
- **THEN** landed answers not landed for its commits

### Requirement: The log is read where sluice keeps it

The log SHALL be read from the remote's records of the branch where the
remote holds them, and otherwise from the host's own records of the
branch, with no sluice process running (sluice C.6).

#### Scenario: A branch on one host

- **WHEN** sluice keeps a branch's records in the host's own files and lands a revision on it
- **THEN** landed answers landed, read from those files with no sluice process running

### Requirement: Verdicts are the signed results of the combination that landed it

Verdicts-of SHALL read the run records under every signer's
`refs/sluice/runs/<signer>` on the remote for the combination its
landing commit names, and SHALL answer one verdict per result of each:
its check the result's task, its build identity the result's key,
moon's hash, its outcome, its signer the name of the ref it was found
under, and the envelope it was found in (78, 104, C.5). A record whose
signed statement names another combination SHALL be left out.

#### Scenario: Two checks on one record

- **WHEN** a revision is landed on a run record by the signer `box` holding a pass of `app:test` and a fail of `app:lint`
- **THEN** verdicts-of answers two verdicts, `app:test` passed and `app:lint` failed, each with its key as the build identity, each by `box`, each carrying that record's envelope

#### Scenario: Two signers

- **WHEN** the signers `box` and `forge` each recorded a run of the combination that landed a revision
- **THEN** verdicts-of answers the verdicts of each under its own name

### Requirement: Where sluice signed nothing there are no verdicts

Where sluice signed no record for the revision, because the branch is
kept on one host or no signer ran it, verdicts-of SHALL answer an empty
set, neither absent nor unreadable (78; sluice ADR 0032).

#### Scenario: A branch sluice signs nothing on

- **WHEN** verdicts-of is asked of a revision landed on a branch sluice keeps in one host's files
- **THEN** it answers an empty set

### Requirement: A replayed result is carried as the pass that ran

For a result its record marks replayed, verdicts-of SHALL answer not
the replay but every signed pass recorded for the same key and the same
system by a run that was not a replay, from the combination that ran
it, under whichever signer's ref it was found, each as made and naming
where it was found (C.5; sluice 108). Which of those counts is the
policy's, never the landing's (9, 104, core.7). A replay with no such
pass SHALL give no verdict.

#### Scenario: A replay of an earlier landing's pass

- **WHEN** a revision's combination replays `app:test` under key K on `linux-x64`, and an earlier combination's record by `box` ran `app:test` under K on `linux-x64` and passed
- **THEN** verdicts-of answers a verdict of `app:test` on K by `box`, carrying the earlier record's envelope and naming where it was found, and no verdict carries the replay's envelope

#### Scenario: A pass on another system does not stand in

- **WHEN** the only earlier pass under the replayed key was run on another system
- **THEN** verdicts-of answers no verdict for the replayed result

### Requirement: Every envelope is carried whole

Each verdict SHALL carry the envelope it was found in whole: its payload
type, its payload byte for byte, and every signature with its key id,
as sluice wrote them, and SHALL say where it was found: the remote, the
signer's ref and the record's path (12, C.5, core.8). The landing SHALL
verify no signature and sign nothing.

#### Scenario: The envelope as sluice wrote it

- **WHEN** a verdict's envelope is written back in DSSE's own form
- **THEN** it holds the same payload type, payload, key ids and signatures as the record sluice wrote

### Requirement: A landing that cannot be reached is unreadable

Where the remote holding sluice's records, or the repository, cannot be
read, landed and verdicts-of SHALL answer unreadable naming the
landing, carrying git's or the operating system's words verbatim, and
never not landed or an empty set (58, 59, core.19).

#### Scenario: The remote is gone

- **WHEN** the remote's repository is removed and landed and verdicts-of are asked
- **THEN** both answer unreadable carrying git's words unchanged

### Requirement: sluice's landing group passes

The conformance suite's landing group SHALL pass over the sluice landing
in a scratch repository with a remote, every scenario on a repository
of its own, each verdict staged as a run record sluice's format holds,
signed with an SSH key as sluice's signers sign (93, core.33).

#### Scenario: The landing group over sluice

- **WHEN** the suite's landing group runs over a rig binding the sluice landing
- **THEN** the report names the profile `sluice` and every scenario passed

#### Scenario: A real landing

- **WHEN** sluice itself lands a revision of a scratch repository whose gate requires one moon task, signing as `box`
- **THEN** landed answers landed, and verdicts-of answers that task's verdict by `box` with moon's hash, carrying the envelope sluice wrote under `refs/sluice/runs/box`
