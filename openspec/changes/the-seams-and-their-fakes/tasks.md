# Tasks

Every cargo and moon command runs as `devenv shell -- <command>`. The
crates' moon projects and their `test` tasks exist (ADR 0011); this
change adds no crate, language or file type, so no project or formatter
entry is added.

## 1. Dependencies

- [x] 1.1 Add `sha2` to `[workspace.dependencies]` in `Cargo.toml`; in `crates/tailrace-seams/Cargo.toml` take `sha2` and `serde_json` from the workspace and drop `anyhow`; in `crates/tailrace-fakes/Cargo.toml` drop `tailrace-core` and `anyhow`, and in its `moon.yml` drop `core` from `dependsOn` and `/crates/tailrace-core/**/*` from the test task's inputs (design D3, D9). Verify `cargo check --workspace` succeeds and `Cargo.lock` gains `sha2`.

## 2. The answers and the shared types

- [x] 2.1 Write `answer.rs`: `Answer<T>`, `Not`, `Absent`, `Refusal`, `Unreadable`, `Source`, `Seam` and `Why` as design D1 says, `Not` not implementing `std::error::Error`. Verify with unit tests that each of the three keeps its source and `said` byte for byte through a JSON round trip, and that a refusal's words are carried beside tailrace's reason (spec seam-contracts: Four answers, and no other; A refusal carries the source's own words).
- [x] 2.2 Write `digest.rs`: `Digest::of`, `Digest::parse`, serialization as its text (design D3). Verify with unit tests: equal bytes give equal digests and the known SHA-256 of the empty input; a text with no algorithm, the wrong length or a non-hex character is rejected naming it (spec: A digest is a function of bytes alone).
- [x] 2.3 Write `artifact.rs`: revision, branch, component (its outputs, paths and tag prefix, 123), output name and output, build identity and outcome, version, ecosystem and ecosystem name, `ArtifactMeta`, `IndexMeta`, `Manifest`, `Held`, `Name` (design D4). Verify with unit tests: metadata with an empty part is rejected naming it; an index rejects an artifact of another version; every type round-trips through JSON (spec: An artifact says what its bytes are; The types that cross a seam).
- [x] 2.4 Write `claim.rs`: `Claim`, `ClaimKind`, `Envelope`, `Signature`, `SignerId`, `NamedSigner`, `Claim::payload` as an in-toto statement, `Claim::from_envelope`, `Envelope::digest` (design D5). Verify with unit tests: a claim with no build identity reads back with none; the same envelope gives the same digest; a claim holding another party's envelope reads it back byte for byte; an envelope in another form answers no claim (spec: A claim is about one digest by one signer).

## 3. The eight traits and the binding

- [x] 3.1 Write `store.rs`, `build.rs`, `landing.rs`, `signer.rs`, `policy.rs`, `projection.rs`, `target.rs` and `keeper.rs`, each with its trait and the types only it passes, with the signatures of design D6 and each method's doc comment citing the clause it is, and `Required` as the enum by audience (83). Verify `cargo clippy -p tailrace-seams --all-targets -- -D warnings` passes and a test that boxes each trait as `dyn` compiles (spec: One interface per seam).
- [x] 3.2 Write `bind.rs`: `Seams` holding one store, build, landing, policy and keeper, the signers, projections by ecosystem and targets by environment, with lookups by ecosystem and environment (design D8). Its test lands with the fakes, in 4.3.
- [x] 3.3 Rewrite `lib.rs`'s doc comment as the table of seams, traits and clauses, as `sluice-seams` keeps one, and re-export every module. Verify `grep -rniwE 'oci|ecr|ghcr|oras|cosign|moon|sluice|kms|npm|maven|cargo|flux|gh' crates/tailrace-seams/src` finds nothing (spec: The seams name no tool, core.1).

## 4. The fakes

- [ ] 4.1 Write `script.rs` in `tailrace-fakes`: `Fault`, `Times`, `fault`, `heal`, and the `Call<Op>` record (design D7). Verify with unit tests that a fault scripted once answers once and then heals, and one scripted until healed answers every time.
- [ ] 4.2 Write one module per fake, `store.rs`, `build.rs`, `landing.rs`, `signer.rs`, `policy.rs`, `projection.rs`, `target.rs`, `keeper.rs`, each `Clone` over shared state, with the scripting and the behavior of design D7, and every operation faultable and recorded. Verify with each module's tests the scenarios of spec seam-fakes and the seam-contracts scenarios for that seam: the same bytes put twice held once; a name moved from where it is not refused naming where it is, and made from nothing; an artifact read back as its manifest; attestations held beside their digest and absent for a digest not held; unreadable carrying `connection refused` verbatim and never empty; outputs unmade named; no verdicts answered empty; an expired signer answered expired; a changed envelope refused; acceptance naming no channel; a pack that is not the same again saying so; an unreadable environment; a stale predecessor refused naming the head; a fork refused naming both; the order of puts recorded, and nothing written the second time.
- [ ] 4.3 Write `Fakes::new()` and `Fakes::seams()` in `lib.rs`, and rewrite its doc comment (design D8). Verify with a test that binds every fake, with two projections and one target, asks each fake every operation once through the binding, finds each projection by ecosystem and the target by environment, and sees every call on the handles the test kept (spec: One fake per seam; Scriptable per test; The core is handed one binding).

## 5. The gate

- [ ] 5.1 Verify `devenv shell -- moon run ':#check-fast'` passes: format, lint, toolchain, ADR numbers, every crate's tests and the site's build.
