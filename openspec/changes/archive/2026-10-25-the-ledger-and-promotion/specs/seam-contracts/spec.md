## MODIFIED Requirements

### Requirement: The keeper's contract

The keeper SHALL answer append, given a promotion claim and the digest
of the promotion it names as its predecessor, with the claim held at
the head of its channel's chain, or refused naming the claim at the
head (120); and chain with every promotion claim of a channel in order
from the first, or absent when none is held (121). A chain that holds
two claims naming one predecessor SHALL be refused as a fork, naming
both, never answered by choosing one (105, 122). A chain answered from
the copy the host last read, because the source holding the ledger did
not answer, SHALL say when that copy was read and carry the source's
words of why it did not answer; a chain read from the source itself
SHALL say so. An append SHALL never be held by such a copy alone (60,
61, C.13, C.14).

#### Scenario: An append whose predecessor is not the head

- **WHEN** append is given a predecessor that is not the head of the channel's chain
- **THEN** it is refused naming the claim at the head, and the chain is unchanged

#### Scenario: A chain read as last read

- **WHEN** the source holding a chain the host has read cannot be reached, and the chain is asked
- **THEN** the answer is the claims as last read, the time of that read and the source's words, and an append is unreadable

## ADDED Requirements

### Requirement: The promotion claim's body

A promotion claim's body SHALL be one type defined with the ledger seam,
readable by a program and read back the same (57): the channel, the
component, the version and index it moves to, the version and index it
moves from or none for the channel's first, the digest of the promotion
it follows or none, who asked, the revision the channel's policy was
read at and the digests of the claims that satisfied it, and when, in
seconds since the Unix epoch (33, 39, 105). Reading a claim's body SHALL
answer none for a claim that is not a promotion of tailrace's form.

#### Scenario: A promotion's body read back

- **WHEN** a promotion claim is made with a body and its envelope is read back
- **THEN** the body read is the body made, value for value, and an acceptance claim's envelope reads as no promotion

### Requirement: A channel's name in the store

A channel's name in the store SHALL be `channel/<channel>`, the same for
the core and every ledger profile that walks from it, and never the
name of a version or a cache entry (3, C.14).

#### Scenario: Stable's name

- **WHEN** the store name of the channel `stable` is asked
- **THEN** it is `channel/stable`

### Requirement: The refusals a promotion gives

A promotion's refusal SHALL name, as Part A says each names: the first
claim found wanting, by its kind, its check where it is a verdict, and
the signers that may make it (58); the version and index a channel is
at, or that it is empty, where a promotion expected it elsewhere (35);
and the channel a bound channel is bound to, where the version was
never a subject of its chain (37).

#### Scenario: Each refusal read back

- **WHEN** each of the three refusals is written in its machine-readable form and read back
- **THEN** it is the same refusal, naming what it names
