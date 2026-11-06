# clan-push-target Specification

## Purpose
TBD - created by archiving change the-clan-push-profile. Update Purpose after archive.

## Requirements

### Requirement: The clan push profile is a target profile of its own

The clan push profile SHALL implement the target seam for one machine
of a clan fleet, pushed to from a host that holds the machine's key
(B.7, C.12). It SHALL be bound under the profile name `clan`, among
every target profile the binding holds. It SHALL be reached through the
target seam alone, and it SHALL name clan, Nix and ssh only under
`profiles/target/` (rules core.1, core.2).

#### Scenario: An environment names the clan profile

- **WHEN** the environment `srv` writes `target.clan = { machine = "swancloud-srv", closure = "swancloud-srv" }`
- **THEN** apply reaches the clan push profile with `srv`'s settings

### Requirement: The target's table names the machine and its closure, and nothing else

The profile's table SHALL hold `machine`, the machine's name in the
fleet, and `closure`, the name of the component's output that names the
machine's closure (C.10). Both SHALL be required strings. Any other key
SHALL make desired, current, apply and report unreadable, naming the key
and the two the table reads. No credential, host address or key path
SHALL be read from the table: the machine is reached as the fleet
reaches it (45).

#### Scenario: A host address in the table

- **WHEN** the table names `machine`, `closure` and `host = "root@192.168.1.172"`
- **THEN** apply is unreadable naming `host`, `machine` and `closure`, and nothing runs

#### Scenario: No closure named

- **WHEN** the table names `machine` alone
- **THEN** apply is unreadable naming `closure` as required

### Requirement: A machine is named as clan and Nix name one

The table's `machine` SHALL be letters, digits, `-` and `_`, beginning
with a letter or a digit, since it is a part of a Nix attribute path
and an argument of clan's. Any other SHALL make every operation
unreadable naming `machine`, with nothing run.

#### Scenario: A machine named as an option

- **WHEN** the table names `machine = "-swancloud"`
- **THEN** apply is unreadable naming `machine`, and nothing runs

### Requirement: desired is the channel's head

The profile SHALL answer desired with the head the core hands it,
absent where the core's head is absent (31, 88).

#### Scenario: An empty channel

- **WHEN** the core's head is absent naming the channel
- **THEN** desired answers that absence

### Requirement: Apply switches the machine to the one root its closure artifact names

Given the verified version, apply SHALL read the version's index by its
digest through the profile's store binding, and from it the revision
the index names and the `file` artifact of the table's `closure` output,
read by its digest (42, rule core.4). The closure SHALL be read through
the family's closure reader, and it SHALL name exactly one root. A
closure naming no root or several SHALL refuse the apply naming the
artifact, with nothing run on the machine.

#### Scenario: A closure of two roots

- **WHEN** the version's `swancloud-srv` artifact names two store paths
- **THEN** apply is refused naming the artifact and its two roots, and clan is not run

### Requirement: A version with no closure of the output or no revision is not applied

An index naming no `file` artifact of the table's output SHALL answer
absent, naming the output and the index. An index naming no revision
SHALL refuse the apply naming the index, since no flake can be
evaluated where it was published. In either, nothing SHALL run on the
machine.

#### Scenario: No artifact of the output

- **WHEN** the version's index names no artifact of the output `swancloud-srv`
- **THEN** apply is absent naming the output and the index

#### Scenario: An index naming no revision

- **WHEN** the version's index was put before indexes named a revision
- **THEN** apply is refused naming the index, and nothing runs

### Requirement: Apply runs only what the version's closure names

Before it updates the machine, apply SHALL evaluate the machine's
toplevel from the fleet's flake at the revision the index names, in a
checkout of that revision. If the evaluated store path is not the
closure's root, apply SHALL refuse, naming both paths, with the machine
untouched. What runs SHALL then be what the landing's checks and the
version's claims named (42, I3).

#### Scenario: The flake evaluates to another closure

- **WHEN** the revision's flake evaluates `swancloud-srv`'s toplevel to a path other than the artifact's root
- **THEN** apply is refused naming the evaluated path and the root, and no update runs

### Requirement: Apply realises the root from substituters alone

Before it updates the machine, apply SHALL realise the closure's root
in this host's Nix store from substituters alone, building no
derivation (I3, 42). A store path no substituter holds SHALL refuse
the apply in Nix's words, naming the path, with the machine untouched
and nothing built in its place (59).

#### Scenario: A store path the cache does not hold

- **WHEN** the closure's root needs a store path no substituter of this host holds
- **THEN** apply is refused with Nix's words naming the path, no derivation is built, and clan is not run

### Requirement: Apply is one update of the machine

Apply SHALL converge the machine with one `clan machines update` of
that machine, against the checkout of the version's revision, with
this host as the build host, so the update finds the root realised and
only copies and switches it (90, C.12). A failed update SHALL answer
what stopped it in clan's own words verbatim (59).

#### Scenario: The update converges

- **WHEN** the root is realised and the switch succeeds
- **THEN** apply answers converged, and one `clan machines update` ran

#### Scenario: clan refuses the switch

- **WHEN** the machine refuses the switch
- **THEN** apply answers what stopped it with clan's words verbatim

### Requirement: The machine is reached with the host's own key

Every command the profile runs against the machine SHALL reach it as
the fleet's own tool does, through clan's view of the machine and the
host's own ssh configuration and keys, with host keys checked against
the host's known hosts and no person asked (45, 64). The profile SHALL
read, write and pass no key, and it SHALL keep nothing of the session
(rule core.21).

#### Scenario: The host holds no key for the machine

- **WHEN** ssh to the machine is refused for want of a key
- **THEN** apply, current and report answer unreadable with ssh's words verbatim, and timber asks no one for a key

### Requirement: Apply leaves a marker of what it applied

After a converged update, apply SHALL write a marker on the machine
naming the index digest applied, the version, the closure's root and
the time of the switch (119). It SHALL write it in the state directory
of the user the fleet deploys as, under a path naming the publishing
repository, the component and the environment, replacing an earlier
marker whole.

#### Scenario: The marker after a switch

- **WHEN** apply converges `srv` on version `0.4.0` at index `sha256:aa…`
- **THEN** the marker for `srv` names `sha256:aa…`, `0.4.0`, the closure's root and the time of the switch

### Requirement: The marker is kept under the checkout's repository

The marker's path SHALL name the repository the checkout is of, by the
owner and name its landing's remote ends in, at apply, current and
report alike (114). Where the checkout names none, each SHALL answer
why, with nothing run. An index naming another publishing repository
SHALL refuse the apply naming both, with nothing run on the machine
(core.60).

#### Scenario: A version published from another repository

- **WHEN** the index names the publishing repository `someone/else` and the checkout is of `afterthought/swancloud`
- **THEN** apply is refused naming both, and nothing runs

### Requirement: A marker that cannot be written is said

A marker that cannot be written SHALL make the apply's answer
unreadable naming the marker and what refused it, with the switch
reported as made (58, 59).

#### Scenario: The state directory refuses the write

- **WHEN** the update converges but the marker's write fails
- **THEN** apply is unreadable naming the marker and the machine's words, saying the switch was made

### Requirement: current is what the machine runs, read through the marker

current SHALL read the closure the machine runs and the marker for the
environment, and answer the marker's digest where the machine runs the
closure the marker names (89, 119, C.12). It SHALL never guess a
digest.

#### Scenario: Running what timber applied

- **WHEN** the marker names root `…-a` at `sha256:aa…`, and the machine runs `…-a`
- **THEN** current answers `sha256:aa…`

### Requirement: current is unread where the marker does not say what runs

Where no marker is held, current SHALL answer unread naming the closure
the machine runs. Where the machine runs another closure than the
marker names, it SHALL answer unread naming both. Where the machine
cannot be reached, it SHALL answer unread with the words of what could
not reach it (89, 119, 58).

#### Scenario: Deployed by hand after timber

- **WHEN** the marker names root `…-a` and the machine runs `…-b`
- **THEN** current is unread naming `…-b` as run and `…-a` as the marker's

#### Scenario: A machine timber never applied

- **WHEN** the machine holds no marker for the environment
- **THEN** current is unread naming the closure it runs and saying no marker of timber's is held

### Requirement: report is what the machine answers after

report SHALL read the machine as current does, and answer the marker's
digest and the switch's time where the machine runs the closure the
marker names. Otherwise it SHALL answer unread as current does (91,
43).

#### Scenario: Reported after a switch

- **WHEN** apply has converged `srv` on index `sha256:aa…`
- **THEN** report answers `sha256:aa…` and the time the marker holds

### Requirement: The profile passes the target group

The clan push profile SHALL pass the conformance suite's target group,
declaring that its target holds a marker. The group SHALL run in the
profile's own tests against a scripted fleet that answers the commands
the profile runs as clan, Nix and the machine would, in `check-fast`
(93, rule core.33).

#### Scenario: The target group over a scripted fleet

- **WHEN** `moon run cli:test` runs the target group over the clan push profile with the scripted fleet
- **THEN** every scenario of the group that applies to a target holding a marker passes

### Requirement: Against one machine, by hand

A test under its own tag, `check-fleet`, SHALL run the target group and
an apply of two versions and a move back against one machine the
operator names in `TIMBER_TOOLS_FLEET`, through the installed `clan`
and the host's own key. It SHALL refuse to run without the setting,
naming it. No gate task SHALL run it, since the machine's key is a key
file (rule core.38).

#### Scenario: Run without the setting

- **WHEN** `moon run cli:fleet` runs with `TIMBER_TOOLS_FLEET` unset
- **THEN** it fails naming `TIMBER_TOOLS_FLEET`, and nothing is run against any machine

#### Scenario: Two versions and back

- **WHEN** the run applies version A and then version B to the machine, and then A again
- **THEN** after each apply current and report name that version's index, and the machine runs that version's root
