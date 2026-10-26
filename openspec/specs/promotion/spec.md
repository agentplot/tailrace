# promotion Specification

## Purpose
TBD - created by archiving change the-ledger-and-promotion. Update Purpose after archive.

## Requirements

### Requirement: A promotion moves a channel to an accepted version

A promotion SHALL name one channel, one version of one component and
the branch whose landed head holds the policy. It SHALL find the
channel among the channels the policy lists at that head for that
component, absent naming the channel and the component where none is
listed (125, 131); it SHALL resolve the version's name in the store to
its index before anything else is read of it, and SHALL read the
channel's policy at the branch's landed head (3, 65, core.4, core.22).
It SHALL move the channel only when the version's index carries an
acceptance claim that counts under the channel's policy (31, 9) and, in
the order the policy lists them, every claim the channel's policy
requires counts; otherwise it SHALL be refused naming the first claim
found wanting, and nothing SHALL be signed, appended or written (58,
core.7).

#### Scenario: A version with every claim the channel requires

- **WHEN** a version's index carries the gate's acceptance and a rebuild claim by a second signer, and the policy for `stable` requires both
- **THEN** the promotion moves `stable` to that version, and the answer names the claims that satisfied the policy

#### Scenario: A version not accepted

- **WHEN** a version's index carries no acceptance claim by a signer the channel's policy names for acceptance
- **THEN** the promotion is refused naming acceptance as the claim found wanting, and the ledger and the store are not written

#### Scenario: A claim the channel requires is wanting

- **WHEN** the policy for `stable` requires a rebuild claim and the index carries none that counts
- **THEN** the promotion is refused naming the rebuild claim and the signers that may make it

#### Scenario: A version never published

- **WHEN** the version's name resolves to nothing in the store
- **THEN** the answer is absent naming the version, and nothing is written

#### Scenario: A channel of another component

- **WHEN** the repository declares `lib` and `web`, only `web` has a channel `edge`, and `edge` is promoted for `lib`
- **THEN** the answer is absent naming `edge` and `lib`, and nothing is read of the version or written

### Requirement: Only the gate's signer makes a promotion

The promotion claim SHALL be signed by the host's signer, and only when
the channel's policy names that signer among its promoters: for a
given channel, or one an environment's words make, the gate's signers;
for one declared by hand, the signers it lets claim a promotion.
Otherwise the promotion SHALL be refused naming the promotion as the
claim found wanting and the promoters, before anything is signed (9,
32, core.8, core.14).

#### Scenario: A host the policy does not let promote

- **WHEN** the host's signer is named by the channel's policy for acceptance alone and not among its promoters
- **THEN** the promotion is refused naming the promotion claim and the signers that may make it, and nothing is signed or appended

#### Scenario: The gate promotes a given stable

- **WHEN** the policy writes no line under `stable`, names `gate` as a signer that may claim acceptance, and the host signing as `gate` promotes a version `beta` has held
- **THEN** `stable` moves, its claim made by `gate` and naming `gate` as who asked

### Requirement: The promotion claim records the move

The promotion claim SHALL be about the index of the version moved to,
name the revision that version's tag names and no build identity, and
record the channel, the component, the version and index moved to, the
version and index moved from or that the channel was empty, the digest
of the promotion it follows or that it is the channel's first, who
asked, the revision the policy was read at and the digests of the
claims that satisfied it, and when (33, 39, 105, core.9).

#### Scenario: The claim read back

- **WHEN** a promotion of `stable` from 0.1.0 to 0.2.0 has been made
- **THEN** the chain's head is a promotion claim about 0.2.0's index naming `stable`, 0.1.0 and its index, the digest of the claim before it, the asker, the policy's revision, the counted claims and the time

### Requirement: The ledger holds the move before the store

A promotion SHALL append its claim to the channel's chain, naming the
head it read as predecessor, before it attaches the claim to the index
or moves the channel's name in the store; a promotion the ledger does
not hold SHALL write nothing to the store (33, 39, core.15). The
channel's name in the store SHALL then be moved from the version the
channel moved from to the version it moved to, a race lost on it
reported and repaired from the ledger (109). A store write that fails
after the append SHALL be reported beside the promotion, which stands.

#### Scenario: The order of the writes

- **WHEN** a promotion is made over the fakes
- **THEN** the ledger's append comes before the store's attach, and the attach before the name's move

#### Scenario: The store cannot be written after the append

- **WHEN** the store's attach is unreadable after the ledger held the promotion
- **THEN** the answer is the promotion, with the store's unreadable answer beside it, and the chain holds the claim

### Requirement: A channel moves only from where it is

A promotion SHALL read the chain's head as the version the channel moves
from. Given the version it expects the channel to be at, a promotion
whose expectation is not the head's version SHALL be refused naming the
version and index the channel is at, or that it is empty (35). An
append the ledger refuses as not after the head SHALL be the answer,
naming the head, and SHALL never be retried (core.6, core.34). A chain
that holds a fork SHALL refuse the promotion naming the fork, and
nothing SHALL be appended or written (105).

#### Scenario: S7, a promotion from a version the channel has left

- **WHEN** a promotion names `stable` as at 0.1.0 and `stable` has since moved to 0.2.0
- **THEN** it is refused naming 0.2.0 and its index, nothing is appended, and nothing is retried

#### Scenario: Another promotion got there first

- **WHEN** another promotion is appended to the chain between this promotion's read of the head and its append
- **THEN** the answer is the ledger's refusal naming the new head, the ledger is asked to append once, and the store is not written

#### Scenario: A forked chain

- **WHEN** the channel's chain holds two claims naming one predecessor
- **THEN** the promotion is refused naming both claims, and nothing is appended or written

#### Scenario: A promotion to where the channel is

- **WHEN** a promotion names the version the channel's head is already at
- **THEN** the answer says the channel is there, nothing is appended, and the store is brought to the ledger

### Requirement: A wrong move is undone by a promotion back

A promotion back SHALL be a promotion to the version the head moved
from, judged and recorded as any other; the ledger SHALL hold both
moves, and the version left SHALL stay accepted and readable by digest
with every claim attached to it (34, 38, 94).

#### Scenario: S8, stable moved back

- **WHEN** `stable` moved from 0.1.0 to 0.2.0, and is then promoted back
- **THEN** the chain holds three promotions in order, `stable`'s name is at 0.1.0's index, and 0.2.0's index and its promotion claim are still read by digest

#### Scenario: Back from an empty or first move

- **WHEN** a promotion back is asked of a channel whose chain is empty or whose head moved from nothing
- **THEN** it is absent naming the version the channel moved from, and nothing is written

### Requirement: A bound channel takes only versions its bound channel held

Where the channel's policy binds it to another channel, a promotion
SHALL move it only to a version that is the subject of a promotion in
the bound channel's chain, whether or not the bound channel is still at
it; any other version SHALL be refused naming the bound channel, and
nothing SHALL be written (37, 83). A bound channel's chain read as last
read SHALL serve, since a chain only grows.

#### Scenario: A version the bound channel never held

- **WHEN** `stable` is bound to `beta` and a version `beta` was never promoted to is promoted to `stable`
- **THEN** it is refused naming `beta`, and nothing is appended

#### Scenario: A version the bound channel has since left

- **WHEN** `beta` was promoted to 0.2.0 and then to 0.3.0, and 0.2.0 is promoted to `stable`
- **THEN** `stable` moves to 0.2.0

### Requirement: The store is rewritten from the ledger

After every promotion, and when the channel is already where it was
asked to go, the store SHALL be brought to the channel's chain: each
claim of the chain not attached to the index it is about SHALL be
attached, and the channel's name SHALL be moved to the head's index,
so a store that disagrees with the ledger is rewritten from it and
never the other way (101, 106). Nothing SHALL be written to the store
from a chain read as last read.

#### Scenario: A claim missing from the store

- **WHEN** the chain holds a promotion whose claim is not attached to its index, and another promotion of the channel is made
- **THEN** the missing claim is attached to its index, and the ledger is not written but for the new promotion

#### Scenario: The channel's name moved elsewhere

- **WHEN** another writer has moved the channel's name in the store to another index
- **THEN** the name is moved back to the head's index and the answer reports the lost race naming that index

### Requirement: A channel is read from the ledger alone

The channel's state SHALL be answered from its chain through the ledger
seam alone: its head's version and index, the version it moved from,
and every promotion in order with what its claim records; a head that
holds no promotion claim of tailrace's form SHALL be unreadable naming
it. Where the ledger answered the chain as last read, the answer SHALL
say when and carry the source's words, an empty channel as any other
(60 as amended at agentplot/blueprints b64d367). Reading a channel
SHALL ask the store nothing and write nothing (101, 60, core.32).

#### Scenario: A promotion read on another machine

- **WHEN** a promotion is made on one machine through a git ref ledger, and a second clone with an empty store reads the channel
- **THEN** the second clone answers the same head and the same chain, and its store is asked nothing

#### Scenario: A channel never promoted

- **WHEN** a channel whose chain is empty is read
- **THEN** the answer is that the channel is empty, which is an answer and not an error (31)

#### Scenario: An empty channel read as last read

- **WHEN** the ledger answers a channel's chain as last read, holding no claim, with the time of that read and its words
- **THEN** the answer is that the channel is empty, with that time and those words, and status shows it `empty, as last read` at that time with the words

### Requirement: Only an approver asks for a channel that names approvers

Where the channel's policy names signers that may ask for it, a
promotion SHALL be asked by one of them: the asker, the host's signer
while no identity provider stands (129), SHALL be among them, or the
promotion SHALL be refused naming the asker and the signers that may
ask, before anything is signed or appended (32, C.10). Where the policy
names none, any asker the gate verifies SHALL ask.

#### Scenario: An asker the policy does not name

- **WHEN** the policy for `stable` names `ci` as the one signer that may ask, and the host signing as `desk`, a promoter of `stable`, promotes a version `beta` has held
- **THEN** the promotion is refused naming `desk` and `ci`, and nothing is signed or appended

#### Scenario: A channel that names no approvers

- **WHEN** the policy for `stable` names no approvers, and the host signing as the gate promotes a version `beta` has held
- **THEN** `stable` moves, naming the gate as who asked
