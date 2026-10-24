# Spec Delta

## ADDED Requirements

### Requirement: The signer keeps no credential

The signer's group SHALL hold a signer to 64: a credential is reached at
the moment of use and kept nowhere, so once the identity's credential is
taken away after a signature, the next sign answers no identity at hand.
The scenario SHALL sit below the group's marker and cite 64, and 64
SHALL leave the register in the same commit (core.33).

#### Scenario: The credential taken away after a signature

- **WHEN** a claim is signed in the identity at hand, the rig then takes that identity away, and a claim is signed again
- **THEN** the first sign answers an envelope and the second answers absent naming the identity

#### Scenario: 64 leaves the register

- **WHEN** the coverage check runs over the tree with the scenario added
- **THEN** 64 is cited by the signer group, is not in the register, and the check passes
