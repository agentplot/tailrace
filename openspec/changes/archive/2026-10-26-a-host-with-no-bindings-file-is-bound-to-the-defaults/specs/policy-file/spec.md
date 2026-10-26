## ADDED Requirements

### Requirement: A file naming no gate names its one provenance signer as the gate

Where `[signers]` at the revision asked names no signer that may claim
`acceptance` and exactly one that may claim `provenance`, the profile
SHALL answer that signer as one that may claim `acceptance` too, in
every audience's claims: the repository's gate, and the promoter of its
given and made channels, as the one host 108 admits (118, C.10). It
SHALL answer a file that names a signer that may claim `acceptance`, or
none that may claim `provenance`, as written, and write nothing into any
file.

#### Scenario: A laptop's policy

- **WHEN** the file at the revision lists `[signers.laptop]` with `kinds = ["provenance"]`, `[acceptance]` requires provenance by `laptop`, and no channel is written
- **THEN** the policy for acceptance names `laptop` with `provenance` and `acceptance`, and the policies for `beta` and `stable` each require acceptance by `laptop` and are promoted by `laptop`

#### Scenario: The gate accepts as one host

- **WHEN** the gate signing as `laptop` judges, under that file, a version whose provenance `laptop` made
- **THEN** the version is accepted, its acceptance claim saying one host offered and accepted, naming `laptop`, and `beta` moves; the gate read `laptop`'s reach from the profile's answer alone (core.7)

#### Scenario: A file that names a gate

- **WHEN** the file lists `[signers.laptop]` for provenance and `[signers.gate]` with `kinds = ["acceptance"]`
- **THEN** the policy for acceptance names `laptop` with `provenance` alone, and a gate signing as `laptop` over a version `laptop` offered refuses it as offered by the gate

### Requirement: Several publishers and no gate is refused in words

Where `[signers]` at the revision asked names no signer that may claim
`acceptance` and two or more that may claim `provenance`, the profile
SHALL answer the policy for acceptance, and the policy for each given or
made channel, refused, wanting acceptance by no signer, with its own
words `several publishers and no gate, name an accepting signer`, since
108 admits no host (58, C.10). The policy for `[layout]` and for a
channel declared by hand SHALL be answered as written, and every channel
SHALL still be listed.

#### Scenario: Two publishers and no gate

- **WHEN** the file lists `[signers.laptop]` and `[signers.desk]`, each with `kinds = ["provenance"]`, and no signer that may claim acceptance
- **THEN** the policy for acceptance, and for `beta` and `stable` with no line written, is refused wanting acceptance by no signer, carrying `several publishers and no gate, name an accepting signer` verbatim

#### Scenario: The gate accepts nothing

- **WHEN** a gate signing as `laptop` judges, under that file, a version whose provenance `laptop` made
- **THEN** nothing is accepted, the gate's answer carries the policy's refusal with those words, no channel moves, and the ledger is not written

#### Scenario: What else the file answers

- **WHEN** that file also has a `[layout]` table and `[channels.edge]` declared by hand
- **THEN** the policy for `[layout]` and for `edge` are answered as written, and the channels listed are `beta`, `edge` and `stable`
