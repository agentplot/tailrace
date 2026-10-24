# Tasks

Every cargo and moon command runs as `devenv shell -- <command>`. This
change runs the conformance suite's signer group as
`the-conformance-suite-over-the-fakes` lands it: read
`crates/tailrace-conformance/src/seams/signer.rs`, `src/seams/mod.rs`,
`src/scenario.rs`, `src/register.rs` and `src/lib.rs`'s section on
running a seam's group over a profile before group 4, and take the names
that landed where this design's differ. If the place holds no
`crates/tailrace-conformance/src/seams/signer.rs`, stop and say so with
`crew needs`: this change comes after that unit. Read
`crates/tailrace-seams/src/signer.rs`, `claim.rs` and `answer.rs`, the
fake signer, and `tailrace-core/src/verify.rs`'s `tell` before starting.
If `crates/tailrace/src/profiles/mod.rs` already exists, from the layout
store's unit, add to it rather than writing it. No trait, shared type,
fake or core module changes; a scenario of the group that cannot pass
without one is reported with `crew needs`. The change adds no crate,
language or file type, so no moon project or formatter entry; its tests
run in the `cli` project's existing `test` task (`crates/tailrace`), and no file under
`crates/tailrace/src` outside `profiles/`, nor the tests file, writes any
word core.1 rules out.

## 1. The module and the dependency

- [x] 1.1 Add `ssh-key = { version = "0.6", features = ["ed25519", "getrandom"] }` to `[workspace.dependencies]` in `Cargo.toml` and `ssh-key.workspace = true` to `[dependencies]` in `crates/tailrace/Cargo.toml`, changing no other line (design D8). Add `pub mod profiles;` to `crates/tailrace/src/lib.rs` if it is not there, `pub mod signer;` to `crates/tailrace/src/profiles/mod.rs` (writing the file with the layout store's doc comment if it does not exist), and `crates/tailrace/src/profiles/signer/mod.rs` with its doc comment and `pub mod host_key;`. Verify `devenv shell -- cargo build -p tailrace` passes and `Cargo.lock` gains `ssh-key` and only what it brings.

## 2. sign

- [x] 2.1 Write `crates/tailrace/src/profiles/signer/host_key/mod.rs`: `HostKey::new(signer, key)`, its source, the namespace `tailrace-claim`, and `pae` (design D1, D3, D4). Verify with unit tests in the module: `pae("http://example.com/HelloWorld", b"hello world")` is the DSSE specification's `DSSEv1 29 http://example.com/HelloWorld 11 hello world`, and `pae("", b"")` is `DSSEv1 0  0 `.
- [x] 2.2 Implement `Signer::sign` in the order of design D2. Verify with unit tests over keys made with `PrivateKey::random` and written with `write_openssh_file` and mode 0600 in a `tempfile` directory: signed as the host, the envelope holds the claim, names `host`, and its `sig` parses as an `SshSig` that `PublicKey::verify` accepts in `tailrace-claim` over the envelope's `pae` and refuses in `git`; asked to sign as `gate`, absent naming `gate`, with the key file at mode 0000 so a read would fail; a missing key file absent naming the identity and the path; mode 0644 refused with `which lets others read it` and `chmod 600`; a key encrypted with `PrivateKey::encrypt` refused saying there is none to type (enable ssh-key's `encryption` feature in `[dev-dependencies]` only if that test needs it, or write the encrypted key from a fixed OpenSSH text in the test); an RSA key, from a fixed OpenSSH text in the test, refused naming its algorithm; the path a directory unreadable carrying the operating system's words; a file of other text unreadable carrying ssh-key's words.
- [x] 2.3 Verify the key is read at the moment of use and kept nowhere (spec `host-key-signer`, the requirement on the key read at the moment of use), with unit tests: the key file replaced between two signs, the second envelope verifies with the new public key and not the old; the key file removed after a sign, the next sign absent; the serialized envelope holds neither the key file's text nor the key's 32 secret bytes; the directory after signing lists only the files the test wrote; and `HostKey`'s fields are its signer and path alone.

## 3. verify

- [x] 3.1 Write the reading of a `verified_by` as design D5 and D6 have it, in `profiles/signer/host_key/listed.rs`: passed over when no word names an SSH key type, the public key and an optional `valid-before` otherwise, unreadable naming the signer for a broken key, another option, or a time without `Z`; the time in ssh-keygen's `YYYYMMDD` and `YYYYMMDDHHMM[SS]` forms converted to seconds since the epoch in UTC. Verify with unit tests: `20000101Z` is 946684800, `202701011230Z` and `20270101123045Z` the seconds of those UTC times; `20991231` without `Z`, `valid-after=…`, and `ssh-ed25519 notakey` each unreadable naming the signer; `arn:aws:kms:…` passed over.
- [x] 3.2 Implement `Signer::verify` in the order of design D7. Verify with unit tests: an envelope the host signed verified with no key file in the directory names `host`, not expired; renamed to `gate` in its signature and verified against `gate` with another key and `host` with the host's, names `host`; its payload or payload type changed, refused `NoNamedSigner` naming the signers checked; a fake signer's envelope refused, never unreadable; a signature made in `git` refused with words naming `git`; a signer verified by a key's name passed over and named among those checked; listed with `valid-before="20000101Z"`, named and expired; with a `valid-before` a year ahead, not expired.
- [x] 3.3 Play S5's signer half in a unit test (spec `host-key-signer`, the requirement on a signature by no listed key): a verdict claim signed with a second host's key, verified against signers listing only the first host's key, is refused `NoNamedSigner` naming the first host's signer, and its words carry the second key's `SHA256:` fingerprint and the name the envelope gives itself.

## 4. The suite

- [x] 4.1 Add the scenario `the credential taken away after a signature` below the signer group's marker in `crates/tailrace-conformance/src/seams/signer.rs`, `plays: None`, `cites: &[64]`, and its function below the module's last function, changing nothing above the marker; remove 64's line from `crates/tailrace-conformance/src/register.rs` in the same commit (design D10, spec `conformance-suite`). Verify `devenv shell -- moon run conformance:test conformance:coverage` passes, the fakes' run of the signer group passing the new scenario, and the coverage output no longer lists 64 as pending.
- [x] 4.2 Write `crates/tailrace/tests/host_key_signer.rs`: `HostKeyRig` as design D9 has it, and the test that runs the signer group over it with `run(signer::GROUP, &signer::scenarios(), …)`. Verify `devenv shell -- cargo test -p tailrace --test host_key_signer` passes, the report naming the host key's profile, every scenario passed but the unreachable signer's, and that one named not applicable with the rig's reason (assert on the report's `not_applicable` as well as calling `assert`).
- [x] 4.3 Verify `grep -rn 'tailrace_core' crates/tailrace/src/profiles` finds nothing (core.2), and `grep -rniwE 'oci|ecr|ghcr|oras|cosign|moon|sluice|kms|npm|maven|cargo|flux|gh' crates/tailrace/src crates/tailrace/tests/host_key_signer.rs crates/tailrace-conformance/src --exclude-dir=profiles` finds nothing this change added (core.1).

## 5. The site

- [ ] 5.1 In `site/src/content/docs/guides/integrations/signers.mdx`, mark the host key `built` and say in its row, or a short section after the table, what it takes and what the policy lists: an Ed25519 key in OpenSSH's format with no passphrase and mode 0600, made with `ssh-keygen -t ed25519 -N '' -f <path>`; claims signed as SSH signatures in the `tailrace-claim` namespace over the envelope's DSSE encoding; verified by the public key the policy lists for the signer, with an optional `valid-before="YYYYMMDDZ"` after which a signature is reported expired. Verify the site's moon build task passes in the gate (6.1).

## 6. The gate

- [ ] 6.1 Verify `devenv shell -- moon run ':#check-fast'` passes: format, lint, toolchain, ADR numbers, the conformance suite's coverage, every crate's tests and the site's build.
