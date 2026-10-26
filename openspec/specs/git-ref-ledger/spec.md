# git-ref-ledger Specification

## Purpose
TBD - created by archiving change the-ledger-and-promotion. Update Purpose after archive.

## Requirements

### Requirement: One ref per channel, one claim per commit

The git ref ledger SHALL hold each channel's chain in the repository at
`refs/tailrace/ledger/<channel>`, each commit holding exactly one
promotion claim and naming as its one parent the commit of the claim
before it, the channel's first claim a commit with no parent (C.13,
105). The chain SHALL be the claims from the first commit to the ref's,
in order (121).

#### Scenario: A chain read from the ref

- **WHEN** two promotions are appended to `stable`
- **THEN** `refs/tailrace/ledger/stable` names a commit holding the second claim whose parent holds the first, and the chain answers both in order

#### Scenario: A ref that holds what is not a chain

- **WHEN** the ref names a commit with two parents, or one holding no promotion claim
- **THEN** the chain is unreadable naming the commit, never answered in part

### Requirement: Append is a push that fast-forwards

Append SHALL hold the claim at the head only when the predecessor given
is the claim at the head as the remote holds it, or none and the remote
holds no ref; it SHALL write the claim's commit on the head's and push
it so that the remote takes only a fast-forward, never a forced update
(120, 122). A push the remote refuses SHALL be refused naming the claim
at the remote's head, read again after the refusal; where the head is
still the predecessor given, the refusal SHALL carry git's words
verbatim (59). Where no remote is bound, the repository's own ref SHALL
be the ledger, moved only from the commit it was read at.

#### Scenario: Two hosts append after one head

- **WHEN** two clones each read `stable` at one head and each append a promotion naming it
- **THEN** the first push is held, the second is refused naming the first's claim, and the remote's chain holds the first alone after the head

#### Scenario: A remote that refuses for its own reasons

- **WHEN** the remote refuses the push while its head is still the predecessor given
- **THEN** the append is refused carrying git's words verbatim, and the chain is unchanged

### Requirement: A remote that no longer extends what was read is a fork

The ledger SHALL fetch the remote's ref before every append and chain.
A fetched ref that does not descend from the commit this repository
last read SHALL be refused as a fork naming the claim both sides follow
and the first claim of each side after it, and the repository's copy
SHALL keep what it read (105, 122). A fetched ref that is an ancestor of
what was read SHALL be unreadable naming the claims the remote no longer
holds.

#### Scenario: A remote rewritten under a reader

- **WHEN** a reader has read `stable` at a head, and the remote's ref is forced to a sibling commit naming the same predecessor
- **THEN** the chain is refused as a fork naming both claims, and the reader's copy still names what it read

### Requirement: A remote that does not answer is read as last fetched

Where the remote cannot be reached, chain SHALL answer the claims of
the repository's own copy of the ref as last fetched or appended, saying
when it was last written and carrying git's words of why the remote did
not answer (60, C.13); with no copy, the chain SHALL be unreadable with
git's words. Append SHALL be unreadable with git's words, and SHALL
never append to the copy alone where a remote is bound.

#### Scenario: The remote gone after a read

- **WHEN** a clone has read `stable`, and its remote's path is then removed
- **THEN** the chain answers the claims it read, the time of that read and git's words, and an append is unreadable

#### Scenario: The keeper's group

- **WHEN** the keeper's conformance group runs over the git ref ledger with a bare remote in a temporary directory
- **THEN** every scenario passes
