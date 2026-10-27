# Spec Delta

## MODIFIED Requirements

### Requirement: The policy's contract

The policy's group SHALL hold a policy to 82, 83 and 131: for each
audience, the claims required with the signers that may make each and
the kinds and checks each may claim, binding no signer by output or
ecosystem (116); where the repository declares no policy for an
audience, an absence naming the audience, the revision beside it being
verify's to name, not the policy's (82); and the channels the policy
declares, each with the one component it belongs to and how it moves,
on each publish with releases alone or prereleases too, by promotion
alone, or bound within its component, one
component holding every channel where the repository declares one or
none (125, 37), listed beside the channels the profile gives with
nothing declared and like them (131). The group's rig SHALL declare
components and channels, each with how it moves, at a landed revision
as well as policies for audiences, and SHALL answer the channels its
profile gives a set of components with nothing declared, and the
channels it holds off. The group SHALL hold the givens a profile holds
off to the channels answer, carried apart from the channels in force,
each naming how it would move and the line that would switch it on
(131, C.10). The group
SHALL hold a profile to the three ways a channel moves and SHALL name
no channel a profile gives, its declared channels bearing names no
profile gives, so a profile whose givens bear other names passes alike
(C.10).

#### Scenario: The claims an audience requires

- **WHEN** the policy for acceptance is asked at a revision whose policy requires a provenance from one signer and a verdict for one check from another
- **THEN** it answers both requirements, each with its signers and what each may claim

#### Scenario: Acceptance names no channel

- **WHEN** the policy for acceptance and for a channel bound to another are asked
- **THEN** acceptance answers no channel, and the bound channel's answer names the channel it is bound to

#### Scenario: No policy for an audience

- **WHEN** the policy for a layout is asked where none is declared
- **THEN** the answer is absent naming the audience, never refused

#### Scenario: The channels a policy declares

- **WHEN** the channels are asked at a revision declaring the components `lib` and `web`, `edge` of `lib` moving by promotion alone, `nightly` of `web` moving on each publish and `next` of `lib` bound to `edge`
- **THEN** the answer is those three channels and the ones the profile gives `lib` and `web`, once each by name and then component, each with its component and how it moves, and the policy for `next` names `edge` as the channel it is bound to

#### Scenario: One component has every channel

- **WHEN** the channels `edge` and `next` are declared, at one revision declaring no component and at another declaring the one component `lib`
- **THEN** both are of no declared component, spelled by name alone, at the first, and of `lib` at the second, and so is every channel the profile gives

#### Scenario: A policy declaring no channel

- **WHEN** the channels are asked at a revision whose policy declares acceptance and no channel
- **THEN** the answer is the channels the profile gives the one component, an empty list where it gives none, and not an absence

#### Scenario: Each way a channel moves

- **WHEN** the channels are asked at a revision declaring one channel of each way it moves, and a second channel moving on each publish with prereleases
- **THEN** the answer says each as declared, prereleases of the second alone, and policy-for of each agrees on the channel it is bound to

#### Scenario: The givens held off

- **WHEN** the channels are asked at a revision declaring the components `lib` and `web` and the channel `edge` of `lib`
- **THEN** the answer's held givens are what the rig says its profile holds off for `lib` and `web`, an empty list where it holds none, each with how it would move and a line, and none of them is among the answer's channels in force

#### Scenario: The channels of a policy that cannot be reached

- **WHEN** the rig makes the policy unreachable and the channels are asked
- **THEN** the answer is unreadable naming the policy, with its words unchanged, and not an empty list

### Requirement: The keeper's contract

The keeper's group SHALL hold a keeper to 120 to 122, and to 34, 35 and
105 as a keeper keeps them: promotions only appended, each after the
claim it names as predecessor, a stale predecessor refused naming the
head, a fork reported and never chosen; to 125: two components'
channels of one name kept as two chains; and to 60 and 61 as C.13 and
C.14 keep them: a chain the keeper cannot reach answered as last read,
saying when, an empty chain as any other, and never appended to.

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

#### Scenario: Two components' channels of one name

- **WHEN** a promotion of `beta` of `lib` is appended with no predecessor, then one of `beta` of `web` with no predecessor
- **THEN** both appends are held, and each chain answers its own claim alone

#### Scenario: A chain read as last read

- **WHEN** a chain is appended and read, the keeper is made unreachable, and the chain is read again
- **THEN** it answers the claims as read, a time no earlier than the first read, and the rig's words, and an append is unreadable with them

#### Scenario: An empty chain read as last read

- **WHEN** the chain of a channel never appended to is read, the keeper is made unreachable, and the chain is read again
- **THEN** it answers no claims, a time no earlier than the first read, and the rig's words, and not that none is held

## ADDED Requirements

### Requirement: The given channels over the fakes

The gate's and the ledger's groups SHALL hold the core to the channels
the policy gives and makes: the gate moving every channel the channels
answer says moves on each publish, of the component published and no
other, and no channel it reads from the policy file (95, 125, 131), a
prerelease moving only those that take prereleases (95, 131); a
promotion of a channel whose policy names the gate's signer as its
promoter, refusing any other signer (32, 9); and a promotion asked by
a signer the channel's approvers do not name refused (32, 33). Each
scenario SHALL cite the clauses it exercises.

#### Scenario: beta of one component

- **WHEN** the fake policy lists `beta` of `lib` and of `web`, both moving on each publish, and the gate accepts a version of `lib` a publish told it of
- **THEN** `beta` of `lib` moves and `beta` of `web` does not, and the ledger holds one chain

#### Scenario: stable promoted by the gate's signer

- **WHEN** the policy for `stable` names `gate` as the signer of its promotion, and in one world a host signing as `gate` and in another one signing as `laptop` promotes a version `beta` has held
- **THEN** the first moves `stable`, and the second is refused naming the promotion and `gate` with nothing appended

#### Scenario: A prerelease and the channels that follow

- **WHEN** the fake policy lists `edge` moving on each publish with prereleases and `nightly` moving on each publish with releases alone, and the gate accepts a prerelease a publish told it of
- **THEN** `edge` moves and `nightly` does not, and the version is accepted

#### Scenario: An asker the approvers do not name

- **WHEN** the policy for `next` names `gate` as its promoter and `ci` as the one signer that may ask, and a host signing as `gate` promotes a version
- **THEN** the promotion is refused naming `gate` as the asker and `ci`, with nothing signed or appended
