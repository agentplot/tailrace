# Spec Delta

## Purpose

How each channel of a repository moves, which channels every component
is given and which an environment's words make, and who signs a
promotion of each, as the policy seam answers them and the gate and the
ledger read them.

## ADDED Requirements

### Requirement: Each channel says how it moves

The policy SHALL answer, of every channel it lists, exactly one of
three ways it moves: on each accepted publish, as the gate's own act
(95); by a promotion alone; or bound, naming the channel of the same
component it is bound to (37, 83, 125, 131). A channel that moves on
each publish SHALL never be bound.

#### Scenario: Three channels, three ways

- **WHEN** a repository's policy lists `beta` moving on each publish, `edge` moving by promotion alone and `stable` bound to `beta`
- **THEN** the channels answer says so of each, and only `stable` names a channel it is bound to

#### Scenario: A following channel that is bound

- **WHEN** a policy file declares a channel both moving on each publish and bound to another
- **THEN** the channels answer is unreadable naming the channel's table, and no channel of it is listed

### Requirement: A channel is its component and its name

A channel SHALL be known everywhere by its component and its name
together: two components' channels of one name SHALL be two channels,
with two chains, two store names and two policies (125). In a
repository that declares no component a channel SHALL be spelled by its
name alone, and in one that declares components by its component, a
slash and its name.

#### Scenario: Two components' `beta`

- **WHEN** a repository declares `lib` and `web` and both components' `beta` are promoted
- **THEN** the ledger holds two chains and the store two names, `channel/lib/beta` and `channel/web/beta`, and neither chain holds the other's claims

#### Scenario: No component declared

- **WHEN** a repository declares no component and its `beta` is promoted
- **THEN** the chain and the store name are spelled `beta` and `channel/beta`, as before components were spelled

### Requirement: The givens are a fragment beneath the file

The given channels SHALL be a policy fragment in the policy file's own
form, shipped with tailrace as data in the policy profile's directory
and laid beneath the repository's file, the two read as one table in
force by every answer the profile gives (C.10). The fragment SHALL hold
`beta`, moving on each accepted publish, and `stable`, bound to `beta`,
both on, and `alpha`, moving on each accepted publish, and `lts`, bound
to `stable`, both off. Every component SHALL have each given that is
on with no line written, listed like a channel declared by hand, with
the policy for each answering like any channel's.

#### Scenario: A repository that writes no channel

- **WHEN** the policy file at a revision declares the components `lib` and `web` and no channel
- **THEN** the channels answer is `beta` of `lib` and of `web`, each moving on each publish, and `stable` of each, bound to its component's `beta`, and neither `alpha` nor `lts`

#### Scenario: A repository with no policy file

- **WHEN** the revision holds no policy file
- **THEN** the channels answer is `beta` and `stable` of the one component, `stable` bound to `beta`

#### Scenario: Another fragment

- **WHEN** the profile is given, in place of the shipped fragment, one holding only `edge` moving on each publish, and the revision holds no policy file
- **THEN** the channels answer is `edge` alone, and the gate moves `edge` on an accepted publish with nothing else changed

### Requirement: One line switches a held given on

A repository SHALL switch on a channel the fragment holds off by
writing that channel's table, `[channels.<name>]` or
`[channels.<component>.<name>]`, with no key needed, since its keys are
already written beneath (C.10). A channel the table in force has off
SHALL not be listed, SHALL have no policy, and SHALL be named by no
binding or environment.

#### Scenario: alpha switched on

- **WHEN** the file declares no component and holds `[channels.alpha]` and nothing under it
- **THEN** the channels answer lists `alpha` moving on each publish beside `beta` and `stable`, and the policy for `alpha` names the gate as its promoter

#### Scenario: lts of one component

- **WHEN** the file declares `lib` and `web` and holds `[channels.lib.lts]`
- **THEN** `lts` of `lib` is listed bound to `stable` of `lib`, and no `lts` of `web` is listed

#### Scenario: A channel held off

- **WHEN** the file writes no `alpha`, and an environment names `channel = "alpha"`
- **THEN** the channels are unreadable naming the environment's table and `alpha`, and the policy for `alpha` alone is absent naming the channel

### Requirement: The same line overrides a given

A key the repository writes in a given channel's table SHALL replace
the fragment's key: `on_each_publish` or `bound_to` SHALL replace how
the channel moves, both keys together, `on = false` SHALL switch it
off, and `require` SHALL add the claims its versions must carry after
the gate's acceptance (C.10). No line SHALL widen who signs a given's
promotion (32, C.10).

#### Scenario: A line that adds a claim

- **WHEN** `[channels.stable]` requires a rebuild claim from `verifier`
- **THEN** the policy for `stable` requires the gate's acceptance and then the rebuild, and `stable` is still bound to `beta`

#### Scenario: stable bound to alpha

- **WHEN** `[channels.alpha]` is written and `[channels.stable]` names `bound_to = "alpha"`
- **THEN** `stable` is listed bound to `alpha`, and its policy names `alpha` as the channel it is bound to

#### Scenario: beta made a promotion alone

- **WHEN** `[channels.beta]` names `on_each_publish = false`
- **THEN** `beta` is listed moving by a promotion alone, and no channel follows each publish

#### Scenario: beta switched off

- **WHEN** `[channels.beta]` names `on = false`, and `stable` is left as given
- **THEN** the channels are unreadable naming `stable`'s table and `beta`, since `stable` is bound to a channel that is off

#### Scenario: A cycle of bindings

- **WHEN** `[channels.beta]` names `bound_to = "stable"` while `stable` is bound to `beta`
- **THEN** the channels are unreadable naming a table of the cycle

### Requirement: The gate's signer promotes a given channel

The policy for a channel the givens hold and the table in force has on
SHALL require the gate's acceptance and SHALL name as the signers that
may sign its promotion the signers the policy lets claim acceptance,
the gate (32), whatever a line under the channel names; any other signer's promotion of it SHALL be refused
naming the promotion and those signers (9, 116).

#### Scenario: The gate promotes stable

- **WHEN** the policy file names `gate` with kinds `acceptance` alone, writes no channel, and the host signing as `gate` promotes an accepted version `beta` has held to `stable`
- **THEN** `stable` moves, and the promotion claim is made by `gate`

#### Scenario: A line naming another promoter

- **WHEN** `[channels.stable]` requires an acceptance from `gate` and names `laptop`, kinds `promotion`, among its signers, and the host signing as `laptop` promotes to `stable`
- **THEN** the promotion is refused naming the promotion and `gate`, and nothing is appended

### Requirement: A manual environment has a channel of its own

An environment whose `policy` is `manual` SHALL give it a channel of
its own, named for the environment, in the component of the channel it
names and bound to that channel (41, 97, C.10). An environment whose
`policy` is `auto`, or unwritten, SHALL make no channel.

#### Scenario: prod waits on stable

- **WHEN** the environment `prod` names the channel `stable` and `policy = "manual"`
- **THEN** the channels answer lists `prod`, bound to `stable`, beside `beta` and `stable`

#### Scenario: An environment that moves on its own

- **WHEN** the environment `site` names the channel `stable` and writes no `policy`
- **THEN** no channel named `site` is listed

### Requirement: An after environment has a channel bound to theirs

An environment that names `after` SHALL be given a channel of its own,
named for it and bound to the channel the environments it names follow:
the channel of its own where one has one, and the channel it names
otherwise (37, C.10). Environments named that follow different
channels, an environment not declared, or one that names itself again
through `after` SHALL make the channels unreadable naming the table.

#### Scenario: eu after us

- **WHEN** `us` names `stable` with `policy = "manual"`, and `eu` names `stable` with `after = ["us"]`
- **THEN** the channels answer lists `us` bound to `stable` and `eu` bound to `us`

#### Scenario: Two environments named that follow two channels

- **WHEN** `eu` names `after = ["us", "asia"]`, and `us` follows `stable` while `asia` follows `beta`
- **THEN** the channels are unreadable naming `eu`'s table, `us` and `asia`

### Requirement: An environment's channel is read as any channel

A channel an environment's words make SHALL be listed, judged and
promoted like a given channel: its policy requires the gate's
acceptance and its promotion is signed by the gate's signer (131,
C.10). A line under it SHALL add claims after the gate's acceptance,
and one naming another way it moves SHALL be unreadable naming its
table, since the environment's words say how it moves.

#### Scenario: Promoting prod

- **WHEN** `prod` is made by a manual environment bound to `stable`, and the gate's signer promotes to `prod` a version `stable` has held
- **THEN** `prod` moves, and its chain holds the promotion

#### Scenario: A line under prod that rebinds it

- **WHEN** `[channels.prod]` names `bound_to = "beta"` while the manual environment `prod` names `stable`
- **THEN** the channels are unreadable naming `[channels.prod]`

### Requirement: The core knows no channel's name

The core, the seams and the fakes SHALL treat no channel apart by its
name, and the policy file profile SHALL take every given's name and
how it moves from the fragment alone, no Rust source of it naming one;
the core SHALL know only the three ways a channel moves (C.10, 131).

#### Scenario: Channels of no given name

- **WHEN** the fake policy lists only `edge`, moving on each publish, and `next`, bound to `edge`, both promoted by the gate's signer, and the gate accepts a version a publish told it of
- **THEN** `edge` moves, and the gate's signer's promotion moves `next` to that version, as `beta` and `stable` would move

#### Scenario: A fragment of other names

- **WHEN** the policy file profile is given a fragment holding `nightly` moving on each publish and `release` bound to it, and a file holding `[channels.release]` with a `require`
- **THEN** the channels answer `nightly` and `release` as the fragment and the line say, and the policy for `release` names the gate as its promoter

### Requirement: The gate and the ledger learn a channel through the answer

The gate SHALL find the channels that follow each publish, and the
ledger's promotion the channel, its binding and who may sign it, only
through the policy seam's channels answer and policy for each channel,
reading no key of the policy file themselves (131, core.1, core.2).

#### Scenario: A gate over a fake policy

- **WHEN** the gate accepts a version it was told of by a publish, over a fake policy that lists `edge` moving on each publish and a repository whose history holds no policy file
- **THEN** `edge` moves, and nothing of the repository's history but its landed head and its tags is read
