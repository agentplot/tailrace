# Open

Decisions waiting, and code that stands against a clause or a rule.
Deleted when empty.

## Code that stands against a clause

- **C.7, the public half in the policy.** The statement has the policy carry a key-service signer's public half, read there at the revision that counted the claim, with the key service asked only where the policy carries none (ADR 0017 as amended). Built: verify reads the public half from KMS at each call, and a key pending deletion makes every claim it signed unreadable; status, which reaches no key service (60), reads a report only a key service could tell as unread naming the key, so the environment it names is unread until the policy carries the public half (ADR 0029). Unit `a-policy-carries-a-key-service-signers-public-half`.
- **114, the process stays while a layout is served** (ADR 0018 as amended). Built: the serving process ends after ten minutes idle whatever it serves. Comes with the served layouts, unit `the-specs-agree-with-the-accepted-records` and bolt 5.
- **129, the record says it names a host, and a publish's promotion names its cause** (ADR 0016 as amended). Built: the promotion claim names the host's signer as asker and says nothing of its kind, and the gate's promotion on a publish names no cause. Unit `a-promotion-says-its-asker-names-a-host`.
- **C.13, "no device that asks for a promotion commits anything".** Holds because the asking host is the gate (108); an asker that is not the gate is not built.
- **141 and section 9, the catalogued kinds.** Built: every kind of claim takes a predicate type of timber's own under `https://agentplot.github.io/timber/claim/` (`crates/timber-seams/src/claim.rs`); a provenance is to take SLSA's Provenance, an acceptance and a rebuild SLSA's Verification Summary Attestation, and a test in-toto's Test Result (ADR 0022, core.57). Unit `every-claim-takes-the-catalogued-kind-that-carries-it`.
- **135, a publish never builds to offer where the landing has verdicts.** Built: publish asks the build seam to ensure every output whatever the landing carries (`crates/timber-core/src/publish.rs`), so where the landing has verdicts an output whose cache entry is gone is built before the publish refuses it naming the output, where 135 builds nothing (core.10, core.30). Unit `a-publish-with-verdicts-builds-nothing`.

## Decisions waiting

- **Clause 54 and a registry's lifecycle rule.** A rule expiring untagged manifests may take an accepted version's attestations, which are untagged referrers; ADR 0015 left it unread.
- **A claim already signed under timber's own type for a catalogued kind.** What verify answers for a provenance, an acceptance or a rebuild signed before ADR 0022's change, under `https://agentplot.github.io/timber/claim/<kind>/v1`, waits on the unit that moves the types (10, 141).
