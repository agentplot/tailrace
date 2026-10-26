## ADDED Requirements

### Requirement: A file naming no gate names its one provenance signer as the gate

Where `[signers]` at the revision asked names no signer that may claim
`acceptance` and exactly one that may claim `provenance`, the profile
SHALL answer that signer as one that may claim `acceptance` too, in
every audience's claims: the repository's gate, and the promoter of its
given and made channels, as the one host 108 admits (118, C.10). It
SHALL answer every other file as written, and write nothing into it.

#### Scenario: A laptop's policy

- **WHEN** the file at the revision lists `[signers.laptop]` with `kinds = ["provenance"]`, `[acceptance]` requires provenance by `laptop`, and no channel is written
- **THEN** the policy for acceptance names `laptop` with `provenance` and `acceptance`, and the policies for `beta` and `stable` each require acceptance by `laptop` and are promoted by `laptop`

#### Scenario: The gate accepts as one host

- **WHEN** the gate signing as `laptop` judges, under that file, a version whose provenance `laptop` made
- **THEN** the version is accepted, its acceptance claim saying one host offered and accepted, naming `laptop`, and `beta` moves; the gate read `laptop`'s reach from the profile's answer alone (core.7)

#### Scenario: A file that names a gate

- **WHEN** the file lists `[signers.laptop]` for provenance and `[signers.gate]` with `kinds = ["acceptance"]`
- **THEN** the policy for acceptance names `laptop` with `provenance` alone, and a gate signing as `laptop` over a version `laptop` offered refuses it as offered by the gate

#### Scenario: Two publishers and no gate

- **WHEN** the file lists `[signers.laptop]` and `[signers.desk]`, each with `kinds = ["provenance"]`, and no signer that may claim acceptance
- **THEN** both are answered with `provenance` alone, and the policy for `beta` names no signer that may promote it
