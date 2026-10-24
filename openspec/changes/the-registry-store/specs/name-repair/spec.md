# Spec Delta

## Purpose

A channel's name moved in the store to the version the ledger's head
promotion is about, and a race lost on that name reported and repaired
from the ledger, never retried blind (109, 101, 106).

## ADDED Requirements

### Requirement: A channel's name follows the ledger's head

Moving a channel's name SHALL read the channel's chain through the
ledger seam and move the name to the digest its head promotion claim
is about, from the digest the caller expects it at (39, 109). An empty
chain SHALL be absent and a forked one refused naming the fork, and
the name SHALL not move (105, 121).

#### Scenario: The name moved to the head

- **WHEN** the chain's head is a promotion about an index, and the channel's name is at the version before
- **THEN** the name is moved to that index from the version before, and the answer says it moved

#### Scenario: A forked chain

- **WHEN** the channel's chain holds two promotions naming one predecessor
- **THEN** the answer is refused naming the fork, and the store is not written

### Requirement: A lost race is reported and repaired from the ledger

When the store refuses the move naming a digest the ledger's head is
not about, the answer SHALL report a lost race naming that digest, and
the name SHALL be moved once from that digest to the head's; the
caller's move SHALL never be retried (109, core.6).

#### Scenario: Another writer moved the name

- **WHEN** another writer moves the channel's name to another index between the caller's read and its move
- **THEN** the answer reports a lost race naming that index, the name is moved from it to the head's index, and the store is asked to move the name twice in all

#### Scenario: The name already where the ledger says

- **WHEN** the store refuses the move naming the digest the head is about
- **THEN** the answer says the name is where the ledger says, and nothing more is written

#### Scenario: The repair refused

- **WHEN** the repair's move is refused in turn, the name having moved again
- **THEN** the answer is that refusal, naming where the name is now, and nothing is retried

### Requirement: A race lost on a registry is repaired

A race lost on a registry's tag SHALL be reported and repaired from
the ledger as on any store (109, C.1).

#### Scenario: A race on zot

- **WHEN** another writer moves a channel's tag in zot between the store's put and its read back
- **THEN** the answer reports the lost race naming the other writer's index, and the tag ends at the index the ledger's head is about
