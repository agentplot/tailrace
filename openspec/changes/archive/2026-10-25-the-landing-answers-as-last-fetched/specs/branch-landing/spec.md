## MODIFIED Requirements

### Requirement: A branch that cannot be read is unreadable

Where the repository cannot be read, landed and verdicts-of SHALL
answer unreadable naming the landing, carrying git's words verbatim
(58, 59, core.19). Where the bound remote does not answer, they SHALL
answer from the host's copy as last fetched (77, C.6), and SHALL answer
unreadable with the remote's words only where the copy holds nothing to
answer from, never not landed and never an empty set.

#### Scenario: The repository is gone

- **WHEN** the repository is removed and landed and verdicts-of are asked
- **THEN** both answer unreadable carrying git's words unchanged, and verdicts-of does not answer an empty set

#### Scenario: A remote never fetched

- **WHEN** the bound remote is removed before landed or verdicts-of was ever answered from it, and both are asked
- **THEN** both answer unreadable carrying git's words unchanged

### Requirement: The branch's landing group passes, and S13 against it

The conformance suite's landing group SHALL pass over the branch
landing twice, with no remote bound and with one, its scenario of
verdicts as the landing recorded them named not applicable in both,
since the branch records none, and its scenario of a landing read as
last fetched not applicable with no remote, since the repository is
then the landing itself (93, C.6). S13 SHALL play over the branch
landing and the moon build: a revision on a branch sluice does not run
is published with no verdict carried; verify under a policy that
requires a verdict counts none, and under one that requires provenance
alone counts the provenance (S13, 15, 111).

#### Scenario: The landing group over a branch

- **WHEN** the suite's landing group runs over a rig binding the branch landing with no remote
- **THEN** the report names the profile `branch`, names verdicts as the landing recorded them and a landing read as last fetched not applicable with their reasons, and every other scenario passed

#### Scenario: The landing group over a branch on a remote

- **WHEN** the suite's landing group runs over a rig binding the branch landing to a remote
- **THEN** the report names the profile `branch`, names verdicts as the landing recorded them not applicable with its reason, and every other scenario, a landing read as last fetched among them, passed

#### Scenario: S13 over a plain branch and moon

- **WHEN** a scratch repository's branch, which no landing gate runs, holds a revision whose output moon makes and passes, and the revision is published and then verified under each of two policies
- **THEN** the publish offers the version with no verdict carried; verify under the policy requiring a verdict reports no verdict counted, and under the policy requiring provenance alone reports the provenance counted

## ADDED Requirements

### Requirement: The branch is read on the remote, and from the copy where it does not answer

With a remote bound, every read of the branch from the remote SHALL
fetch it into the host's copy, which then says when it was fetched;
where the remote does not answer, landed SHALL answer from the copy of
the branch, saying when it was fetched and carrying git's words, and
verdicts-of SHALL answer an empty set for a revision the repository
holds, saying when the remote was last fetched (77, 78, C.6). With no
remote bound, the repository is the landing and every answer says so.

#### Scenario: Landed as last fetched

- **WHEN** landed is answered for a revision from the remote, the remote is removed, and landed is asked again
- **THEN** it answers landed from the copy, with a fetch time no earlier than the first answer and git's words unchanged

#### Scenario: Landed on the remote after the fetch

- **WHEN** a revision reaches the remote's branch after the host last fetched it, the remote is then removed, and landed is asked of the revision
- **THEN** it answers not landed from the copy, saying when the copy was fetched, never landed

#### Scenario: No remote bound

- **WHEN** landed and verdicts-of are answered with no remote bound
- **THEN** each says it was read from the landing itself
