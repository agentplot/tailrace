## ADDED Requirements

### Requirement: The profile can be bound to reach no key service

The profile SHALL be bindable verify-only and reaching no service, for
status (60). Its verify SHALL refuse unreached what it refuses before
any request to KMS, and where it would ask KMS SHALL answer unreadable
naming the key the policy lists, saying status reaches no key service
and a key-service signer's public half is read from the service until
the policy carries it (C.7, rule core.42). Sign SHALL answer absent for
every identity.

#### Scenario: A P-256 signature against a listed key

- **WHEN** the profile bound to reach no service verifies an envelope carrying a P-256 signature against signers naming a key ARN
- **THEN** it answers unreadable naming that key and saying status reaches no key service, and no request reaches KMS

#### Scenario: A host key's envelope

- **WHEN** the profile bound to reach no service verifies an envelope carrying only an Ed25519 signature
- **THEN** it answers refused unreached, as the verify-only profile does
