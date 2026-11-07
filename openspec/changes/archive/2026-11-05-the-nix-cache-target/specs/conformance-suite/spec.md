## MODIFIED Requirements

### Requirement: A target's contract

The target's group SHALL hold a target to 88 to 91, to 45 and 119, and
to the parts of 31, 40 and 43 a target keeps: what an environment should
run is its channel's head as the core hands it, an empty channel is an
answer, what it runs is reported by digest with when it last converged
where the target holds a marker and is unread with the reason where it
holds none, and a target reads no setting but its own. The group's rig
SHALL declare whether its target holds a marker of what it applied, as a
store's rig declares whether its names take a condition (rule core.49);
"converged and reported" SHALL run where it does, and "applied where no
marker is held" where it does not, each not applicable to the other.

#### Scenario: What an environment should run

- **WHEN** desired is asked of one environment with a head that names a digest, and of another with a head that answers the channel empty
- **THEN** the first answers that digest and the second answers empty, not an error

#### Scenario: An environment that cannot be read

- **WHEN** what an environment runs cannot be read
- **THEN** the answer is unreadable with the reason, never a digest

#### Scenario: Converged and reported

- **WHEN** a verified version is applied to an environment whose target holds a marker
- **THEN** the environment's current digest and its report name that version's digest and when it converged

#### Scenario: Applied where no marker is held

- **WHEN** a verified version is applied to an environment whose target holds no marker
- **THEN** current and report each answer unreadable with the reason, and neither names the digest applied

#### Scenario: A table with a key the target does not read

- **WHEN** desired, current and apply are asked of an environment whose settings hold a key the target does not read
- **THEN** each answers unreadable naming that key, and nothing is applied

### Requirement: The binding

The binding's group SHALL hold the binding to 92, and SHALL hold the
suite itself to 93 by running a seam's group over a profile that breaks
its contract and finding it reported as failed.

#### Scenario: One of each, and any number of projections and targets

- **WHEN** one store, build, landing, policy and keeper, two signers, two projections and two targets are bound
- **THEN** each projection is found by its ecosystem and each target by its profile

#### Scenario: A broken profile fails the suite

- **WHEN** the store's group runs over a store that holds other bytes under a digest
- **THEN** the run fails, reported as that profile's failure

## ADDED Requirements

### Requirement: The policy's environments

The policy's group SHALL hold a policy to 136, its rig declaring at a
landed revision a waiting environment and one that moves on its own,
each with a target's profile and settings; its scenarios SHALL cite 136,
and 136 SHALL leave the register in the commit whose scenario first
cites it (rule core.33).

#### Scenario: Environments declared

- **WHEN** the rig declares `cache`, following `beta` with a target, and `prod`, manual and following `stable` and approved by one signer, at a landed revision
- **THEN** environments answers both, `prod` following the channel made for it and waiting on that signer, each with its target as declared

#### Scenario: None declared

- **WHEN** environments is asked at a revision that declares none
- **THEN** it answers an empty list, not absent

### Requirement: Apply over the fakes

A core group, apply, SHALL run the core's apply over the fakes, a
scenario for each requirement of `environment-apply`: the head applied
once its claims are judged, a wanting claim, an empty channel, an
unbound profile, an environment already at the head. They SHALL cite
31, 40, 42, 88, 90 and 136 as each exercises them, and 42 SHALL leave
the register in the commit whose scenario first cites it (rules
core.29, core.33).

#### Scenario: The head applied after its claims are judged

- **WHEN** an environment follows `beta`, whose head's index carries every claim the policy for `beta` requires, and the fake target runs another digest
- **THEN** the fake target records one apply of that index and version, after the claims were read, and the answer names them

#### Scenario: A wanting claim stops the apply

- **WHEN** the policy for the channel requires a claim the head's index lacks
- **THEN** the apply is refused naming that claim, and the fake target records no apply

### Requirement: The register after the Nix cache target

When this unit's scenarios are in, 42, 45, 119 and 136 SHALL be cited and
in no register entry. The entries that named the first target SHALL name
the unit or phase that will cite them: 41 and 97 the clan push profile of
bolt 6; 44 and 126 phase 2; 98, and the parts of 132 and 134 that show
environments, phase 2's environments in status; each with its reason
(rule core.33, section 12).

#### Scenario: The register after this unit

- **WHEN** the coverage check runs after this unit's last commit
- **THEN** 42, 45, 119 and 136 are cited and not registered, 41 and 97 are pending with the clan push profile, and 44, 98, 126 and the parts of 132 and 134 are pending with phase 2, and the check passes
