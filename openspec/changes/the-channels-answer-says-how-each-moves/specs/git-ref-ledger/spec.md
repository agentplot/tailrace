# Spec Delta

## MODIFIED Requirements

### Requirement: One ref per channel, one claim per commit

The git ref ledger SHALL hold each channel's chain in the repository at
`refs/tailrace/ledger/<channel>` where the repository declares no
component, and at `refs/tailrace/ledger/<component>/<channel>` where it
declares components, each commit holding exactly one promotion claim
and naming as its one parent the commit of the claim before it, the
channel's first claim a commit with no parent (C.13, 105, 125). The
chain SHALL be the claims from the first commit to the ref's, in order
(121).

#### Scenario: A chain read from the ref

- **WHEN** two promotions are appended to `stable`
- **THEN** `refs/tailrace/ledger/stable` names a commit holding the second claim whose parent holds the first, and the chain answers both in order

#### Scenario: A ref that holds what is not a chain

- **WHEN** the ref names a commit with two parents, or one holding no promotion claim
- **THEN** the chain is unreadable naming the commit, never answered in part

#### Scenario: A component's chain

- **WHEN** a promotion is appended to `beta` of `lib` in a repository that declares `lib` and `web`
- **THEN** `refs/tailrace/ledger/lib/beta` names a commit holding it, and no `refs/tailrace/ledger/beta` or `refs/tailrace/ledger/web/beta` is written

### Requirement: A remote that does not answer is read as last fetched

Where the remote cannot be reached, chain SHALL answer the claims of
the repository's own copy of the ref as last fetched or appended, saying
when it was last written and carrying git's words of why the remote did
not answer (60, C.13); where the copy last read the channel holding no
claim, chain SHALL answer it empty, saying when it was read and
carrying git's words, as any other chain as last fetched (60, as
amended at agentplot/blueprints b64d367); with no copy of the channel
at all, the chain SHALL be unreadable with git's words. Append SHALL be
unreadable with git's words, and SHALL never append to the copy alone
where a remote is bound.

#### Scenario: The remote gone after a read

- **WHEN** a clone has read `stable`, and its remote's path is then removed
- **THEN** the chain answers the claims it read, the time of that read and git's words, and an append is unreadable

#### Scenario: An empty channel as last fetched

- **WHEN** a clone has read `beta` while its remote held no promotion of it, and its remote's path is then removed
- **THEN** the chain answers no claims, the time of that read and git's words, and not absent

#### Scenario: The keeper's group

- **WHEN** the keeper's conformance group runs over the git ref ledger with a bare remote in a temporary directory
- **THEN** every scenario passes
