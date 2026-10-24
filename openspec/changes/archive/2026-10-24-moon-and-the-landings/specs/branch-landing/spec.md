## Purpose

The landing seam bound to a plain git branch (C.6), for a repository
that runs no landing gate: a revision is landed when the branch holds
it, and no landing records verdicts.

## ADDED Requirements

### Requirement: A revision is landed when the branch's history holds it

The branch landing SHALL answer that a branch has landed a revision
when the revision is the branch's head or one of its ancestors, the
branch read from the remote where one is bound and from the repository
otherwise (77, C.6). A revision the branch does not hold SHALL be
answered not landed, as an answer and not a refusal.

#### Scenario: Landed and not

- **WHEN** landed is asked of the branch's head, of an ancestor of it, and of a commit on another branch only
- **THEN** it answers landed, landed and not landed

#### Scenario: The remote's branch

- **WHEN** the branch is bound to a remote whose branch holds a revision the local branch does not
- **THEN** landed answers landed

### Requirement: There are no verdicts

Verdicts-of SHALL answer an empty set for every revision the repository
holds, which is neither absent nor unreadable (78, C.6).

#### Scenario: A landed revision's verdicts

- **WHEN** verdicts-of is asked of a landed revision
- **THEN** it answers an empty set

### Requirement: A branch that cannot be read is unreadable

Where the repository or the bound remote cannot be read, landed and
verdicts-of SHALL answer unreadable naming the landing, carrying git's
words verbatim (58, 59, core.19).

#### Scenario: The repository is gone

- **WHEN** the repository is removed and landed and verdicts-of are asked
- **THEN** both answer unreadable carrying git's words unchanged, and verdicts-of does not answer an empty set

### Requirement: The branch's landing group passes, and S13 against it

The conformance suite's landing group SHALL pass over the branch
landing, its scenario of verdicts as the landing recorded them named
not applicable, since the branch records none (93, C.6). S13 SHALL play
over the branch landing and the moon build: a revision on a branch
sluice does not run is published with no verdict carried; verify under
a policy that requires a verdict counts none, and under one that
requires provenance alone counts the provenance (S13, 15, 111).

#### Scenario: The landing group over a branch

- **WHEN** the suite's landing group runs over a rig binding the branch landing
- **THEN** the report names the profile `branch`, names verdicts as the landing recorded them not applicable with its reason, and every other scenario passed

#### Scenario: S13 over a plain branch and moon

- **WHEN** a scratch repository's branch, which no landing gate runs, holds a revision whose output moon makes and passes, and the revision is published and then verified under each of two policies
- **THEN** the publish offers the version with no verdict carried; verify under the policy requiring a verdict reports no verdict counted, and under the policy requiring provenance alone reports the provenance counted
