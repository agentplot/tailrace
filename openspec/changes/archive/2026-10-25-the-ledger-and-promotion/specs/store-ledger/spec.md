## ADDED Requirements

### Requirement: The chain is walked from the channel's name

The store ledger SHALL hold each channel's chain as the promotion claims
attached to the indexes they move to, through the bound store's seam
alone (C.14, core.2). Chain SHALL resolve the channel's name in the
store, take the promotion claims of that channel attached to the index
it names, and walk each back by the index it moved from and the digest
of the claim it follows, answering the claims from the first in order
(121). A channel whose name names nothing SHALL be absent.

#### Scenario: Three promotions and a move back

- **WHEN** `stable` was promoted to 0.1.0, 0.2.0 and back to 0.1.0, so 0.1.0's index holds two of its claims
- **THEN** the chain answers all three in order, the last at the head

#### Scenario: A claim the store no longer holds

- **WHEN** a claim the walk names as a predecessor is not attached to the index it moved from
- **THEN** the chain is unreadable naming the missing claim's digest and the index it was looked for on

### Requirement: Two claims naming one predecessor are a fork

A walk that meets two claims of the channel naming one predecessor, or
two claims on the head's index neither of which follows the other,
SHALL refuse the chain as a fork naming both, never choosing (105, 122).

#### Scenario: Two claims after one head

- **WHEN** two promotions of `stable` naming the same predecessor are attached to their indexes
- **THEN** the chain is refused as a fork naming both claims

### Requirement: Append attaches the claim and moves the name

Append SHALL hold the claim only when the predecessor given is the head
of the chain as walked now and is the predecessor the claim itself
names; otherwise it SHALL be refused naming the head (120). It SHALL
attach the claim to the index it is about, then move the channel's name
from the head's index to it; a move the store refuses SHALL be refused
naming the head as walked after the refusal (35, 109).

#### Scenario: An append after a stale head

- **WHEN** append names a predecessor that is no longer the head
- **THEN** it is refused naming the head, and nothing is attached or moved

### Requirement: A store that does not answer is read as last read

Each chain this host reads SHALL be kept, with when it was read, under
the repository's common git directory beside the record of published
versions (61, 62). Where the store cannot be reached, chain SHALL
answer the chain as kept, saying when it was read and carrying the
store's words (60, C.14); with nothing kept, the chain SHALL be
unreadable with the store's words. Append SHALL be unreadable with the
store's words. What is kept SHALL hold no credential and nothing the
store holds that is not the chain (64).

#### Scenario: The store gone after a read

- **WHEN** a host has read `stable`'s chain from the layout, and the layout is then made unreadable
- **THEN** the chain answers the claims read, the time of the read and the store's words

#### Scenario: The keeper's group

- **WHEN** the keeper's conformance group runs over the store ledger bound to an OCI layout in a temporary directory
- **THEN** every scenario passes
