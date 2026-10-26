## ADDED Requirements

### Requirement: Publish tells the host's gate where the host binds it to

Where the bindings file binds `[gate] publish = {}`, `tailrace publish`
SHALL, after the core's publish has answered, tell the gate of each
version the answer offered, in the same run: the component, the version
and the index digest the answer names, told by a publish, the gate
signing as the host's `[signing]` signer (95, 108, 130). A component the
publish left unchanged, or answered with absent, refused or unreadable,
SHALL tell the gate nothing. The core's publish SHALL reach nothing of
the gate: the command calls the gate after the publish has answered
(130, core.10). The gate's answer for each version SHALL be printed
beside that version's, and in the JSON answer SHALL be the version's
`gate`, in the core's one form (57). The exit code SHALL be the worst of
each component's publish answer and of the gate's, a version the gate
refused or a following channel it held counting as refused.

#### Scenario: One host, beta moves in the same run

- **WHEN** the policy lets the host's signer claim both provenance and acceptance, `beta` follows each publish and requires acceptance from that signer, the file binds `[gate] publish = {}` and a ledger, and `tailrace publish` offers version 0.2.0
- **THEN** the same run prints 0.2.0 accepted by the host, the acceptance saying the host accepted what it offered, and `beta` promoted to 0.2.0's index; the ledger's chain for `beta` holds the promotion; and the exit code is 0

#### Scenario: Two landings, beta follows both

- **WHEN** two revisions land one after the other and `tailrace publish` runs after each on that one host
- **THEN** `beta` is at the second version, its chain holds two promotions, the second naming the first as its predecessor, and no other process ran

#### Scenario: The gate refuses what the host offered

- **WHEN** the file binds `[gate] publish = {}` and the policy does not let the host's signer claim acceptance
- **THEN** the version is offered, the gate's answer refuses it naming the first artifact and the claim found wanting, nothing is attached, `beta` does not move, and the exit code is that of a refusal

#### Scenario: A channel held

- **WHEN** the gate accepts the version and `beta`'s own policy requires a claim the version does not carry
- **THEN** the version is accepted, the answer for `beta` is held naming the claim, and the exit code is that of a refusal

#### Scenario: Accepted with no ledger bound

- **WHEN** the file binds `[gate] publish = {}` and no `[ledger]`, and `beta` follows each publish
- **THEN** the version is accepted and the answer for `beta` is absent naming `[ledger]` in the file

#### Scenario: Unchanged is not told

- **WHEN** the publish answers a component unchanged since its last release
- **THEN** the gate is not asked, and that component's answer carries no gate answer

#### Scenario: The JSON answer carries the gate's

- **WHEN** `tailrace publish --json` tells the gate of 0.2.0 and the gate accepts it
- **THEN** the component's entry holds the publish's answer as before and a `gate` member holding the gate's answer, value for value as the core's gate answers it

### Requirement: Publish says when nothing on the host tells the gate

Where the bindings file holds no `[gate]`, `tailrace publish` SHALL
offer as it does today and SHALL say, for each version it offered, that
nothing on this host tells the gate, naming `[gate]` in the file; the
version stays offered (20, 130). Where the file binds `[gate] job`, it
SHALL say, for each version it offered, that the gate is told by the
job the file names, carrying its words as written, and SHALL ask the
gate nothing. Neither SHALL change the exit code.

#### Scenario: No gate table

- **WHEN** the file holds no `[gate]` and `tailrace publish` offers 0.2.0
- **THEN** the answer offers 0.2.0 as before and says nothing on this host tells the gate, naming `[gate]` in the file; no acceptance is attached, the ledger is not asked, and the exit code is 0

#### Scenario: A job tells the gate

- **WHEN** the file binds `[gate] job = "the release job a push to main wakes"` and `tailrace publish` offers 0.2.0
- **THEN** the answer says 0.2.0 is offered and the gate is told by "the release job a push to main wakes"; no acceptance is attached, the ledger is not asked, and the exit code is 0

#### Scenario: The JSON answer says so

- **WHEN** `tailrace publish --json` offers 0.2.0 on a host whose file holds no `[gate]`
- **THEN** the component's `gate` member is absent naming `[gate]` in the file, in the core's one form
