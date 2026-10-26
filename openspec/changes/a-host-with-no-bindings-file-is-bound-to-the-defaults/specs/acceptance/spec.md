## MODIFIED Requirements

### Requirement: The gate never accepts what it offered, but on the one host

Where the provenance an artifact's acceptance counted was made by the
gate's own identity, the gate SHALL refuse the version naming that
artifact and the required provenance, as offered by the gate (27, 96),
unless the policy names that identity as a signer that may claim both
provenance and acceptance (108), or the policy names no gate (118). A
policy names a gate where some signer it requires a claim of, for
acceptance or for any channel, may claim acceptance. Where it names
none, the gate's own identity, as the policy names it for provenance,
SHALL be the one host 108 admits; an identity the policy does not name
SHALL be admitted by nothing. No verdict but the gate's SHALL read the
policy so (core.7).

#### Scenario: A build host running the gate

- **WHEN** the policy names `host` for provenance alone and names `gate` as a signer that may claim acceptance, and the gate is run signing as `host` over a version whose provenance `host` made
- **THEN** the version is refused naming the required provenance and the first artifact, as offered by the gate, and nothing is attached

#### Scenario: A policy that names no gate

- **WHEN** the policy names `laptop` for provenance alone, no signer it requires a claim of may claim acceptance, and the gate signing as `laptop` judges a version whose provenance `laptop` made
- **THEN** the version is accepted, and the acceptance is attached to its index

#### Scenario: A host the policy does not name

- **WHEN** the policy names `laptop` for provenance alone and no gate, and the gate signing as `desk` judges a version whose provenance `desk` made
- **THEN** the version is refused naming the required provenance found wanting, and nothing is attached

### Requirement: A one-host acceptance says so

Where the policy names one signer that may claim both provenance and
acceptance and that signer, as the gate, accepts a version whose
provenance it made, or where the policy names no gate and the gate's
own identity accepts a version whose provenance it made (118), its
acceptance claim SHALL say the host accepted what it offered, naming
the signer; every other acceptance claim SHALL say it was separated,
naming the signers that made the provenance (108). A second judgment of
a version the gate's own identity accepted so SHALL answer it accepted
already, attaching nothing more (94).

#### Scenario: The solo laptop

- **WHEN** the policy lets `laptop` claim provenance and acceptance, and the gate signing as `laptop` accepts a version whose provenance `laptop` made
- **THEN** the acceptance claim says one host offered and accepted, naming `laptop`

#### Scenario: The solo laptop with no gate named

- **WHEN** the policy names `laptop` for provenance alone and no gate, and the gate signing as `laptop` accepts a version whose provenance `laptop` made
- **THEN** the acceptance claim says one host offered and accepted, naming `laptop`

#### Scenario: Told twice with no gate named

- **WHEN** the gate signing as `laptop` under a policy that names no gate is told again of a version it accepted
- **THEN** the answer is accepted already, naming the acceptance it attached before, and nothing more is attached

#### Scenario: A separated gate

- **WHEN** the gate signing as `gate` accepts a version whose provenance `host` made
- **THEN** the acceptance claim says it was separated, naming `host` as the offerer
