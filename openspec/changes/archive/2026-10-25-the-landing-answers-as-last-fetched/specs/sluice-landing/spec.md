## MODIFIED Requirements

### Requirement: A landing that cannot be reached is unreadable

Where the repository cannot be read, landed and verdicts-of SHALL
answer unreadable naming the landing, carrying git's or the operating
system's words verbatim (58, 59, core.19). Where the remote holding
sluice's records does not answer, they SHALL answer from the host's
copy as last fetched (77, C.5), and SHALL answer unreadable with the
remote's words only where the copy holds nothing to answer from, never
not landed or an empty set.

#### Scenario: The remote is gone

- **WHEN** the remote's repository is removed before landed or verdicts-of was ever answered from it, and both are asked
- **THEN** both answer unreadable carrying git's words unchanged

### Requirement: sluice's landing group passes

The conformance suite's landing group SHALL pass over the sluice landing
in a scratch repository with a remote, every scenario on a repository
of its own, each verdict staged as a run record sluice's format holds,
signed with an SSH key as sluice's signers sign, a landing read as last
fetched among them (93, core.33).

#### Scenario: The landing group over sluice

- **WHEN** the suite's landing group runs over a rig binding the sluice landing
- **THEN** the report names the profile `sluice` and every scenario passed

#### Scenario: A real landing

- **WHEN** sluice itself lands a revision of a scratch repository whose gate requires one moon task, signing as `box`
- **THEN** landed answers landed, and verdicts-of answers that task's verdict by `box` with moon's hash, carrying the envelope sluice wrote under `refs/sluice/runs/box`

## ADDED Requirements

### Requirement: sluice's records are read on the remote, and from the copy where it does not answer

With a remote bound, each of sluice's refs the landing reads, the
branch's log and each signer's run records, SHALL be fetched into the
host's copy, which says when it was fetched; where the remote does not
answer, landed and verdicts-of SHALL answer from the copies as from the
remote, saying the oldest fetch time the answer rests on, with git's
words (77, 78, C.5). With no remote bound, the host's own records are
the landing, and every answer says so.

#### Scenario: Landed and verdicts as last fetched

- **WHEN** landed and verdicts-of are answered for a landed revision from the remote, the remote is removed, and both are asked again
- **THEN** they answer the same, landed and the same verdicts carried whole, from the copy, with a fetch time no earlier than the first answers and git's words unchanged

#### Scenario: A landing after the fetch

- **WHEN** sluice lands a revision after the host last fetched the branch's log, the remote is then removed, and landed is asked of the revision
- **THEN** it answers not landed from the copy, saying when the log was fetched, never landed
