## ADDED Requirements

### Requirement: The file names the signers the host trusts

The file SHALL name the signers the host trusts, one `[trust.<signer>]`
table each, holding exactly one key whose string value is the text the
signer's profile verifies it by, in the policy file's `[signers]` form
and with no `kinds` or `checks` (section 2, 118, C.10), and never a
credential (64, core.21). A file with no `[trust]` SHALL trust no
signer. The binding SHALL carry the trusted signers to sync alone.

#### Scenario: A signer trusted

- **WHEN** the file names `[trust.gate]` with `ssh = "ssh-ed25519 AAAA… gate@box"`
- **THEN** the binding trusts one signer, `gate`, verified by that key

#### Scenario: No trust named

- **WHEN** the file names `[store]`, `[landing]` and `[signing]` and no `[trust]`
- **THEN** the binding trusts no signer, and publish and verify answer as before

### Requirement: A broken trust table is unreadable
A trust table naming no key, two keys, or a value that is not a string
SHALL be unreadable naming the file and the table (58, 59).

#### Scenario: A trust table with two keys

- **WHEN** `[trust.gate]` names both `ssh` and `kms`
- **THEN** sync answers unreadable naming the file and `[trust.gate]`, and nothing is read or written
