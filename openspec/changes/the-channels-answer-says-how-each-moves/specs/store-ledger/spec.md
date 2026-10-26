# Spec Delta

## MODIFIED Requirements

### Requirement: The chain is walked from the channel's name

The store ledger SHALL hold each channel's chain as the promotion claims
attached to the indexes they move to, through the bound store's seam
alone (C.14, core.2). Chain SHALL resolve the channel's name in the
store, spelled with its component where the repository declares
components (125), take the promotion claims of that channel and
component attached to the index it names, and walk each back by the
index it moved from and the digest of the claim it follows, answering
the claims from the first in order (121). A channel whose name names
nothing SHALL be absent.

#### Scenario: Three promotions and a move back

- **WHEN** `stable` was promoted to 0.1.0, 0.2.0 and back to 0.1.0, so 0.1.0's index holds two of its claims
- **THEN** the chain answers all three in order, the last at the head

#### Scenario: A claim the store no longer holds

- **WHEN** a claim the walk names as a predecessor is not attached to the index it moved from
- **THEN** the chain is unreadable naming the missing claim's digest and the index it was looked for on

#### Scenario: Two components' beta

- **WHEN** `beta` of `lib` and `beta` of `web` are each promoted twice
- **THEN** the chain of `beta` of `lib` is walked from `channel/lib/beta` and answers its two claims alone, and the same holds of `web`
