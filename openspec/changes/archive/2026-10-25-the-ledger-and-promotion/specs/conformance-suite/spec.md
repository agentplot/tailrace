## MODIFIED Requirements

### Requirement: The keeper's contract

The keeper's group SHALL hold a keeper to 120 to 122, and to 34, 35 and
105 as a keeper keeps them: promotions only appended, each after the
claim it names as predecessor, a stale predecessor refused naming the
head, a fork reported and never chosen; and to 60 and 61 as C.13 and
C.14 keep them: a chain the keeper cannot reach answered as last read,
saying when, and never appended to.

#### Scenario: A chain appended in order

- **WHEN** a first promotion is appended with no predecessor and a second naming the first
- **THEN** the chain answers both, in that order, to two readers alike

#### Scenario: A stale predecessor

- **WHEN** a promotion names as its predecessor a claim that is no longer the head
- **THEN** the append is refused naming the claim at the head, and the chain is unchanged

#### Scenario: A fork

- **WHEN** the rig holds two promotions naming one predecessor and the chain is read
- **THEN** the chain is refused naming both claims, and neither is chosen

#### Scenario: A channel with no promotion

- **WHEN** the chain of a channel never appended to is read
- **THEN** the keeper answers that none is held, not an error

#### Scenario: A chain read as last read

- **WHEN** a chain is appended and read, the keeper is made unreachable, and the chain is read again
- **THEN** it answers the claims as read, a time no earlier than the first read, and the rig's words, and an append is unreadable with them

## ADDED Requirements

### Requirement: Promotion over the fakes

The ledger's group SHALL hold the core's promotion over the fakes: S6 as
far as the ledger goes, an approver promoting a version whose policy
for `stable` requires the gate's acceptance and a rebuild by a second
signer, the move in the ledger before the store, the claim attached to
the index and `stable`'s name at it (33, 39, 101); S7 (35, 105); S8 (34,
38); a bound channel taking a version its bound channel held and since
left, and refusing one it never held (37, 83); a version found wanting,
one not accepted, and a host the policy does not let promote, each
refused naming the claim and writing nothing (9, 31, 32, 58); a forked
chain refusing the promotion (105); the store rewritten from the ledger
(106); and a channel read from the ledger alone, the store asked
nothing (101, 60). Each scenario SHALL cite the clauses it exercises,
and 32, 33, 37, 38, 39, 101 and 106 SHALL leave the register in the
commit that cites them.

#### Scenario: S6 over the fakes

- **WHEN** an approver promotes an accepted version whose index carries a second signer's rebuild claim the policy for `stable` requires
- **THEN** the ledger's append comes before every store write, the promotion claim is attached to the index, and `stable`'s name resolves to it

#### Scenario: S7 over the fakes

- **WHEN** a promotion names `stable` at a version it has left
- **THEN** it is refused naming the version `stable` is at, and the ledger and the store are asked to write nothing

#### Scenario: S8 over the fakes

- **WHEN** `stable` is moved back to the version before
- **THEN** the ledger holds both moves, and the newer version's index and claims are still read by digest

#### Scenario: The register after this unit

- **WHEN** the coverage check runs
- **THEN** 32, 33, 37, 38, 39, 101 and 106 are cited and in no register entry, 41 and 97 are pending with the first target, and 125 is pending with `the-policy-seam-lists-the-channels`
