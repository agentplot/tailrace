## MODIFIED Requirements

### Requirement: The landing's verdicts are carried as made

A provenance claim SHALL carry every verdict of the landing that names
the artifact's build identity, each envelope byte for byte as the
landing recorded it with every signature and key id it holds, and say
where it was found: the landing, the revision it was asked for, and
where the landing says it found the verdict. tailrace SHALL never
re-sign another party's verdict as its own (12, core.8, C.5).

#### Scenario: A verdict carried byte for byte

- **WHEN** a version is published from a revision the landing recorded two verdicts for, both naming the build identity of one output
- **THEN** that output's provenance claim carries both verdicts' envelopes byte for byte with the landing named as where they were found, and no envelope the store holds is signed by the host over a verdict's payload

#### Scenario: Where the landing found it

- **WHEN** the landing answers a verdict found in an earlier run's record
- **THEN** the provenance claim that carries it names that record as the landing named it
