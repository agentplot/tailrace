# after-channels Specification

## Purpose
TBD - created by archiving change the-clan-push-profile. Update Purpose after archive.

## Requirements

### Requirement: The apply's host tells its own gate of a report

After an apply signs and attaches a report, `timber apply` SHALL tell
the gate as the publish command does (ADR 0020, 130). Where the host's
bindings bind this host's own steps to tell its gate, `[gate] publish
= {}` or no `[gate]`, it SHALL ask the core's gate to move the channels
that come after the environment applied. The core's apply SHALL start
nothing itself (rule core.52).

#### Scenario: The host's own gate

- **WHEN** a host with no `[gate]` applies `first`, signing its report
- **THEN** the gate is asked for every channel that comes after `first`, and apply's answer carries each channel's answer

### Requirement: A job named to tell the gate moves nothing after an apply

Where the host's bindings name a job to tell the gate, apply's answer
SHALL carry the job's words as written, timber SHALL run nothing for
it, and no channel SHALL move (130, C.16).

#### Scenario: A job tells the gate

- **WHEN** the host's file writes `[gate] job = "the fleet's gate job"`, and an apply signs a report
- **THEN** apply's answer carries `the fleet's gate job`, and no channel moves

### Requirement: A channel an after makes moves to what its environments last reported

For each channel the channels answer lists as bound and coming after
the environment applied, the gate SHALL read the newest report of each
environment it comes after. Where every one names one index, it SHALL
move the channel to that index as its own act (37, 43, 91, C.10).

#### Scenario: second follows first

- **WHEN** `second` comes after `first`, and `first`'s newest report names `0.4.0`'s index
- **THEN** the gate moves `second`'s channel to `0.4.0`

### Requirement: A channel whose environments disagree is held

Where the newest reports of the environments a channel comes after
name different indexes, or one is absent or unreadable, the gate SHALL
not move the channel. Its answer SHALL name each environment and what
it last reported, or its absent or unreadable answer (58).

#### Scenario: Two environments that disagree

- **WHEN** `eu` comes after `us` and `asia`, both following `stable`, and `us` last reported `0.4.0` while `asia` last reported `0.3.0`
- **THEN** `eu`'s channel does not move, and its answer names `us` at `0.4.0` and `asia` at `0.3.0`

### Requirement: The channel is never moved ahead of what was last reported

The gate SHALL never move a channel an `after` makes to a version that
an environment it comes after did not last report it runs, even on a
report that counts but is not that environment's newest. It SHALL take
the version only from among those its bound channel has held (37, 83).

#### Scenario: An older report of a newer version

- **WHEN** `first` reported `0.4.0`, then was moved back and reported `0.3.0`
- **THEN** `second`'s channel is moved to `0.3.0`, never to `0.4.0`

### Requirement: An asked promotion of a channel an after makes is refused

A promotion asked of a channel an `after` makes SHALL be refused,
naming the channel and the environments it comes after, since the gate
alone moves it, on their reports (37, 43, C.10, rule core.72). It SHALL
be refused before the channel's chain is read, with nothing signed or
appended, whoever asks, among them the channel's own promoters.

#### Scenario: Asked ahead of what first last reported

- **WHEN** `first` reported `0.4.0`, then was moved back and reported `0.3.0`, the gate moved `second` to `0.3.0`, and `second`'s own promoter asks `timber promote 0.4.0 --to second`
- **THEN** the promotion is refused naming `second` and `first`, nothing is appended, and `second` stays at `0.3.0`

### Requirement: The channel follows its environments back

Where every environment a channel comes after last reported a version
older than the one the channel names, the gate SHALL move the channel
back to it as any move is made: a promotion naming its predecessor,
held by the ledger before the store is written (33, 34, 105).

#### Scenario: S16, back with first

- **WHEN** `first`'s channel is moved back from `0.4.0` to `0.3.0`, and `first` converges back and reports `0.3.0`
- **THEN** the gate moves `second`'s channel back to `0.3.0`, and its chain holds both moves

### Requirement: The move is judged under the channel's own policy

Before it moves the channel, the gate SHALL judge the version under the
channel's policy: the gate's acceptance, its line's claims, and a
counted report of each environment it comes after (83, 9). A claim
found wanting SHALL refuse that channel naming the claim, with nothing
signed or appended.

#### Scenario: The report is from a signer that may not report

- **WHEN** `first`'s only report of `0.4.0` is signed by a signer whose kinds lack `report`
- **THEN** `first`'s newest report is absent, and `second`'s channel does not move

### Requirement: The move is signed by one of the channel's promoters

The move SHALL be signed by the host's signer, which SHALL be one of
the channel's promoters (32, 108). Where it is not, that channel's
answer SHALL be refused naming the host's signer and the channel's
promoters, with nothing appended.

#### Scenario: A host that is not the gate

- **WHEN** the apply's host is not among `second`'s promoters
- **THEN** the answer for `second` is refused naming the host's signer and the promoters, and nothing is appended

### Requirement: The move is the gate's act, not an asking

The gate SHALL record its own signer as who asked for a move on
reports, whatever `approvers` the channel's policy names, as for a move
on a publish (95, 129, C.10). The reports SHALL be among the claims the
promotion names as counted.

#### Scenario: second's promotion claim

- **WHEN** the gate signing as `deployer` moves `second`'s channel on `first`'s report
- **THEN** the promotion names `deployer` as who asked, and `first`'s report among the counted claims

### Requirement: Each channel is answered on its own

Apply's answer SHALL carry one answer for each channel that comes after
the environment applied: moved, already there, held with its
environments' reports, or refused or unreadable naming why (57, 58). A
refused channel SHALL leave the others' moves standing, and nothing
SHALL be retried (35, rule core.6).

#### Scenario: One moves and one is refused

- **WHEN** two channels come after `first`, and the second's policy wants a review the version lacks
- **THEN** the first channel moves, and the second's answer is refused naming the review
