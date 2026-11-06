## ADDED Requirements

### Requirement: Apply finds the environment at the landed head

The core's apply SHALL read the governed branch's head, landed, and the
component it is asked for, as a promotion reads them (65, 77, 123), and
SHALL find the environment of that name in that component among the
environments the policy answers at that revision (136, rule core.22).

#### Scenario: An environment the policy does not declare

- **WHEN** apply is asked for `cache` of `web`, and the policy at the landed head declares no environment `cache` in `web`
- **THEN** the answer is absent naming `cache` and `web`, and no target is asked anything

### Requirement: Apply reaches the target the environment names

An environment that names no target SHALL be absent naming the
environment's `target`, and one whose target names a profile no binding
holds SHALL be absent naming the profile (92); in each case no target
SHALL be asked anything.

#### Scenario: A target profile no binding holds

- **WHEN** the environment's target names the profile `bucket` and the binding holds no target of that profile
- **THEN** the answer is absent naming `bucket`, and no target is asked anything

#### Scenario: An environment with no target

- **WHEN** the environment names no target
- **THEN** the answer is absent naming the environment's `target`, and no target is asked anything

### Requirement: Apply reads what the environment should run from its channel's chain

The core SHALL hand the target, as the head `desired` is given, the
digest of the index the head promotion of the environment's channel
moved to, read from the ledger's chain, never from resolving the
channel's store name (88, rules core.4, core.50). An empty chain SHALL
answer absent naming the channel, and no target SHALL be asked to apply
(31).

#### Scenario: An empty channel

- **WHEN** the environment follows `beta` of `web`, and the chain of `beta` of `web` holds no promotion
- **THEN** the answer is absent naming `beta` of `web`, and the target is asked to apply nothing

### Requirement: A target's desired must be the chain's head

A `desired` the target answers other than the chain's head SHALL be
refused naming both, and nothing SHALL be applied (40, rule core.16).

#### Scenario: A target that answers another digest than the head

- **WHEN** the chain's head moved `beta` to index `i2` and the target's desired answers `i1`
- **THEN** the apply is refused naming `i1` and `i2`, and the target is asked to apply nothing

### Requirement: Apply judges the head's claims before it hands over the version

The core SHALL judge the claims the policy for the environment's channel
requires, at the landed head, of the head's index before the target is
handed anything; a claim found wanting SHALL refuse the apply naming
that claim, the target asked to apply nothing (42, 58, rule core.29). No
apply path SHALL skip the judgment.

#### Scenario: A head whose claim is wanting

- **WHEN** the policy for `stable` requires a rebuild by `box`, and the index `stable`'s head names carries none
- **THEN** the apply is refused naming the rebuild and `box`, and the target was asked to apply nothing

### Requirement: Apply hands the target the verified version

Only once every claim is judged satisfied SHALL the core make the
verified version, the head's index and the version its promotion names,
and hand it to the target's apply (90).

#### Scenario: The channel's head applied

- **WHEN** the chain of `beta` of `web` has its head at `1.2.0`, every claim its policy requires is held, and the environment runs another version
- **THEN** the target is asked to apply `1.2.0` at its index, after the claims are judged, and the answer names the environment, `beta` of `web`, `1.2.0`, its index and the target's report

### Requirement: Apply does not apply what the environment already runs

Where the target's `current` answers the digest the chain's head names,
the core SHALL ask no apply, and the answer SHALL say the environment
runs that version already (89, 90).

#### Scenario: Already converged

- **WHEN** the target's current answers the index `beta`'s head names
- **THEN** no apply is asked, and the answer says the environment runs that version

### Requirement: Apply's answer

Apply's answer SHALL name the environment, its component, the channel
it follows, the version and index applied, what `current` answered
before, as a digest or its unread answer with the reason, and the
target's report after, or its answer where it gives none, in the one
form every surface renders (43, 57, 91, rule core.19). What stops the
target's apply SHALL be the answer, in the target's words verbatim (59).

#### Scenario: A target that cannot apply

- **WHEN** every claim is held and the target's apply answers unreadable with words of its own
- **THEN** the apply's answer is unreadable carrying those words verbatim

#### Scenario: A target that keeps no marker

- **WHEN** the head is applied to an environment whose target answers current and report unreadable, saying it holds no marker
- **THEN** the apply is asked, and the answer names the version applied with current before and the report after as those unreadable answers, never as converged

### Requirement: Apply moves no name and makes no store

Apply SHALL move no name (rules core.6, core.16), and SHALL make no
store, reading through a store binding that is opened and never made.

#### Scenario: No name moved

- **WHEN** an environment is applied
- **THEN** every name the store holds is at the digest it was at before

#### Scenario: No store made

- **WHEN** an environment is applied on a host whose bound layout does not exist
- **THEN** no layout is made
