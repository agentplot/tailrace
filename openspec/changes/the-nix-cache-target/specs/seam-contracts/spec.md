## MODIFIED Requirements

### Requirement: One interface per seam
Each seam of Part B SHALL be exactly one interface holding that seam's
operations as the statement names them and no other: the store (66 to
72), the build (74, 75), the landing (77, 78), the signer (79, 80), the
policy (82, 131, 136), a projection (84 to 86, 137, 140), a target (88 to
91) and the keeper (120, 121). A profile implements one interface and nothing of
the core (core.2).

#### Scenario: Each seam's operations
- **WHEN** a reader lists the operations of each seam's interface
- **THEN** the store has put-bytes, put-artifact, attach, claims-of, name, resolve and read; the build has ensure and report; the landing has landed and verdicts-of; the signer has sign and verify; the policy has policy-for, channels and environments; a projection has pack, render, serve, pinned, requires and satisfying; a target has desired, current, apply and report; the keeper has append and chain; and no interface has any other

#### Scenario: The seams name no tool
- **WHEN** the seams' code is searched for the names core.1 rules out
- **THEN** none appears, and the code imports nothing of any profile

### Requirement: A target's contract

A target SHALL answer desired with the digest an environment's channel
names, or absent when the channel is empty (88), given the head the core
read from the channel's chain, which a target that is pushed to answers
and a target that pulls answers by its own reading (44, rule core.50);
current with the digest the environment runs, or unreadable with the
reason (89); apply, given a version the reconciler has verified, with
the environment converged or what stopped it (90); and report with what
it runs and when it last converged (91). A target that can hold no
marker of the digest it applied SHALL answer current and report
unreadable with the reason, never a digest and never converged (119).
The environment a target is handed SHALL carry the target's own settings
as the policy answered them, and a target SHALL read no setting but its
own, answering unreadable naming a key it does not read (45, C.10).

#### Scenario: An environment that cannot be read

- **WHEN** current is asked of an environment whose state cannot be read
- **THEN** the answer is unreadable with the reason, never a digest

#### Scenario: A push target answers the head it is handed

- **WHEN** desired is asked of a target that is pushed to, with a head that answers a digest
- **THEN** it answers that digest, and with a head that answers the channel empty it answers absent

#### Scenario: A target that holds no marker

- **WHEN** a version is applied to an environment whose target can hold no marker, and current and report are asked
- **THEN** each answers unreadable with the reason, and neither names the digest applied

### Requirement: The core is handed one binding

The core SHALL be handed the seams as one binding holding exactly one
store, one build, one landing, one policy and one keeper, the signers
bound for the host, and any number of projections by ecosystem and
targets by the profile each implements (92), and SHALL reach no seam
except through it (core.2). An environment SHALL name the profile of its
target, and the core SHALL find that target by it.

#### Scenario: A binding with two projections

- **WHEN** a binding is made with two projections and one target
- **THEN** each projection is found by its ecosystem and the target by its profile, and the binding holds one store, one landing and one keeper

#### Scenario: Two environments of one profile

- **WHEN** two environments of two components both name the profile `nix_cache`, each with settings of its own
- **THEN** both reach the one target bound for `nix_cache`, each handed its own settings

## ADDED Requirements

### Requirement: The policy's environments

The policy SHALL answer environments, given a landed revision, with
every environment declared there, ordered by component and then name,
each with the channel it follows, the one made for it where it waits or
comes after another (41, C.10), whether it waits, whom it waits on, and
its target where it names one: the target's profile and that profile's
own settings, as written (136, C.10). Where none is declared, the answer
SHALL be an empty list, never absent.

#### Scenario: A manual environment and its target

- **WHEN** the policy at a revision declares `prod` of `web`, manual, following `stable`, approved by `box`, its target naming the profile `nix_cache` with settings
- **THEN** environments answers `prod` of `web` following the channel `prod` made for it, waiting on `box`, with its target's profile `nix_cache` and those settings

#### Scenario: No environment declared

- **WHEN** the policy at a revision declares no environment
- **THEN** environments answers an empty list

#### Scenario: Written and read back

- **WHEN** an environments answer with a target's settings is written in its machine-readable form and read back
- **THEN** it is the same, the settings included
