# host-key-signer Specification

## ADDED Requirements

### Requirement: The profile can be bound to verify only

The host-key profile SHALL be bindable with no signer and no key file,
for a host that verifies and signs nothing: verify then tells a claim by
the keys the policy lists exactly as a profile with a key does, and sign
answers absent for every identity, saying no identity is at hand, with
no file read (79, 80, 64).

#### Scenario: Verify on a host with no key

- **WHEN** the verify-only profile is asked who made an envelope the host `laptop` signed, given a policy listing `laptop`'s public key
- **THEN** it answers `laptop`, having read no key file

#### Scenario: Sign on a host with no key

- **WHEN** the verify-only profile is asked to sign a claim as any identity
- **THEN** it answers absent, saying no identity is at hand, and reads no file
