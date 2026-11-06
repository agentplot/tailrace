## ADDED Requirements

### Requirement: A bound channel names the environments it comes after

The channels answer SHALL carry, for a bound channel an environment's
`after` makes, the environments it comes after, each by its name within
the channel's component (125, 131, C.10). A bound channel that no
`after` makes SHALL carry none. Its machine-readable form SHALL read a
bound channel written with no environments as coming after none, and
SHALL write such a channel as it wrote it before (57).

#### Scenario: second after first

- **WHEN** the channels answer lists `second`, bound to `first`, made by `second`'s `after = ["first"]`
- **THEN** `second` names `first` as the one environment it comes after

#### Scenario: A bound channel written before

- **WHEN** a channels answer whose bound channel names no environments is read
- **THEN** the channel reads as bound and coming after none, and writes back the same

### Requirement: A report is a kind of claim

A report SHALL be a claim kind a signer may be listed for, beside
provenance, verdict, test, review, rebuild, acceptance and promotion
(116, C.12). A claim the policy requires of kind report SHALL name the
environment it is a report of, as a verdict names its check. It SHALL
be asked of every artifact, and a report on the version's index SHALL
satisfy it (7, 134).

#### Scenario: A report requirement written and read back

- **WHEN** a requirement of a report of `first` by the signers `deployer` is written in its machine-readable form and read back
- **THEN** it is the same value, of kind report, naming `first`, asked of every artifact

#### Scenario: A report on the index

- **WHEN** a version's index carries a report of `first` by `deployer`, and a channel's policy requires one
- **THEN** the requirement is satisfied for every artifact the index names
