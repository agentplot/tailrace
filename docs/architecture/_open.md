# Open

Decisions waiting, and code that stands against a clause or a rule.
Deleted when empty.

## Code that stands against a clause

- **C.7, the public half in the policy.** The statement has the policy carry a key-service signer's public half, read there at the revision that counted the claim, with the key service asked only where the policy carries none. Built: verify reads the public half from KMS at each call, and a key pending deletion makes every claim it signed unreadable (ADR 0017). Unit `a-policy-carries-a-key-service-signers-public-half`.
- **114, the process stays while a layout is served.** Built: the serving process ends after ten minutes idle whatever it serves (ADR 0018). Comes with the served layouts, unit `the-specs-agree-with-the-accepted-records` and bolt 5.
- **129, the record says it names a host.** Built: the promotion claim names the host's signer as asker and says nothing of its kind (ADR 0016). Unit `a-promotion-says-its-asker-names-a-host`.
- **C.13, "no device that asks for a promotion commits anything".** Holds because the asking host is the gate (108); an asker that is not the gate is not built.

## Decisions waiting

- **Clause 54 and a registry's lifecycle rule.** A rule expiring untagged manifests may take an accepted version's attestations, which are untagged referrers; ADR 0015 left it unread.
