## ADDED Requirements

### Requirement: Reports over the fakes

A core group, `after`, SHALL hold the report scenarios over the fakes.
In the first, an apply converges and the fake target reports the index
applied, and one report claim about it, signed by the host's signer,
is attached beside it. In the second, the report is unread and no
claim is signed. In the third, an environment's newest report is read
across two versions of its channel. Each scenario SHALL cite 43 and 91,
and the first SHALL also cite 11 (rule core.33).

#### Scenario: The report group passes over the fakes

- **WHEN** `moon run conformance:test` runs the `after` group over the fakes
- **THEN** the three report scenarios pass, each citing its clauses

### Requirement: S16 over the fakes

The `after` group SHALL play S16 over the fakes, with `first` a manual
environment on `stable` whose `approvers` name `desk`, and `second`
after `first`. An asking by `desk` moves `first`'s channel and not
`second`'s; `first` is applied and reports, and `second`'s channel
follows; `first` is moved back, applied and reports, and `second`'s
channel follows back. It SHALL cite 37, 41, 43, 83, 91 and 97.

#### Scenario: S16 passes over the fakes

- **WHEN** the `after` group runs over the fakes
- **THEN** S16 passes, `second`'s chain holding two moves, each after a report of `first`

### Requirement: S16 checks that second is never ahead

After each step of S16, the scenario SHALL assert that `second`'s
channel is empty or names the index of `first`'s newest report (37,
C.10).

#### Scenario: A move ahead fails the scenario

- **WHEN** the fakes are scripted so the gate would move `second` to a version `first` has not reported
- **THEN** S16 fails naming the step and the two indexes

### Requirement: The clan push profile runs the target group

The target group SHALL run over the clan push profile, in its own tests
against a scripted fleet, declaring that its target holds a marker. It
SHALL also run by hand against one machine, under `check-fleet` (93,
rule core.38).

#### Scenario: The scripted fleet

- **WHEN** `moon run cli:test` runs
- **THEN** the target group passes over the clan push profile with the scripted fleet

### Requirement: The register after the clan push profile

41 and 97 SHALL leave the register in the commit whose S16 scenario
first cites them, and no entry SHALL name `bolt 6 · the clan push
profile` after this change (rule core.33, ADR 0012). 44, 98, 126 and
the parts of 132 and 134 SHALL stay named by the phase the Nix cache
target's change names them by.

#### Scenario: The coverage check after this change

- **WHEN** `moon run conformance:coverage` runs
- **THEN** 41 and 97 are cited by S16, appear in no register entry, and no entry names the clan push profile
