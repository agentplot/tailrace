# Tasks

Every cargo and moon command runs as `devenv shell -- <command>`. Read
`crates/tailrace-seams/src/signer.rs`, `claim.rs` and `answer.rs`, the
host-key profile (`crates/tailrace/src/profiles/signer/host_key/`), its
test `crates/tailrace/tests/host_key_signer.rs`, the suite's signer group
(`crates/tailrace-conformance/src/seams/signer.rs`), `tailrace-core/src/verify.rs`'s
`tell`, `bindings/mod.rs`, and `tests/registry_ecr.rs` with the `tools-ecr`
task in `crates/tailrace/moon.yml` before starting. No trait, shared type,
fake, conformance scenario or core module changes; a need for one is
reported with `crew needs`. The change adds no crate, language, file
type or plugin directory: its tests run in the `cli` project's existing
`test` task, and the account run in a new task of the same project.
Outside `crates/tailrace/src/profiles/` and the profile's own test
files, no file this change writes holds a word core.1 rules out.

## 1. The dependencies and the module

- [x] 1.1 Add to `[workspace.dependencies]` in `Cargo.toml`, changing no other line: `aws-config` and `aws-sdk-kms` with default features off and the features of design D6, `aws-smithy-http-client` (rustls on ring), `tokio` with `rt`, `net` and `time`, and `p256` with `ecdsa` and `pkcs8`; and the matching `.workspace = true` lines in `crates/tailrace/Cargo.toml`, with `aws-smithy-http-client`'s `test-util` in `[dev-dependencies]`. Add `pub mod kms;` to `crates/tailrace/src/profiles/signer/mod.rs` and `crates/tailrace/src/profiles/signer/kms/mod.rs` with its doc comment citing B.4, C.7, 64 and core.21. Verify `devenv shell -- cargo build -p tailrace` passes, `cargo tree -p tailrace -i aws-lc-sys` finds nothing, and `Cargo.lock` gains only what these bring. If the SDK cannot build without its default aws-lc client, stop and say so with `crew needs` (design, Risks).

## 2. The key and the reach

- [x] 2.1 Write `KeyArn` in `profiles/signer/kms/` (design D1): parse `arn:<partition>:kms:<region>:<account>:key/<id>`, answering its region and its text; an alias ARN, a bare id, an alias name or any other text is not a key ARN, and a text that begins `arn:` and names `:kms:` but fails to parse is told apart from one that names no KMS key at all. Verify with unit tests: the ARN in the spec parses with region `us-west-2`; `arn:aws:kms:us-west-2:111122223333:alias/fleet`, `1234abcd-…`, `alias/fleet` and `ssh-ed25519 AAAA…` do not, the first being a broken KMS name and the last no KMS name.
- [x] 2.2 Write `Reach` (design D5, D6): `Host`, which at each call loads the default AWS config in the ARN's region with the rustls-on-ring HTTPS client and a 3 s connect timeout, resolves its credentials once and builds a KMS client with them; and `Given { credentials, http, endpoint }`, which builds the client from those directly and reads no AWS file. Each call runs on a current-thread Tokio runtime made and dropped inside it. `Kms` holds the optional `KeyArn` and the `Reach` and nothing else. Verify with a unit test that `Kms`'s fields are those two, and with 4.3's tests that no credential outlives a call.

## 3. sign and verify

- [x] 3.1 Implement `Signer::sign` in the order of design D3, reading answers as design D4's table has them, the envelope holding the claim's payload, `IN_TOTO`, and one `Signature` whose `signer` and `keyid` are the ARN and whose `sig` is KMS's DER signature over the SHA-256 of `host_key::pae(IN_TOTO, payload)`, sent as `DIGEST` with `ECDSA_SHA_256`. `Kms::verifying()` answers every sign absent, saying no identity is at hand. Verify with 4.3's tests.
- [x] 3.2 Implement `Signer::verify` in the order of design D7, normalizing S before `p256` verifies, and reading answers as design D4's table has them. Verify with 4.3's tests.
- [x] 3.3 Verify `grep -rn 'tailrace_core' crates/tailrace/src/profiles` finds nothing (core.2).

## 4. The stand-in and the profile's tests

- [x] 4.1 Write the stand-in for KMS's JSON API in `crates/tailrace/tests/kms/mod.rs` (design D9): keys by ARN with a `p256` signing key, state, spec, usage and origin; `Sign`, `GetPublicKey`, `DescribeKey`, `EnableKey` and `DisableKey` answered as KMS's API reference has them, including `DisabledException` for Sign on a disabled key, `KMSInvalidStateException` for Sign and GetPublicKey on a key pending deletion, success for GetPublicKey on a disabled key, and `NotFoundException` for an unknown ARN; an injected error per operation; a request count; and the switchable credentials provider. Verify with a test in `kms_signer.rs` that a key made in the stand-in signs through the SDK and its signature verifies with `p256` against the SPKI GetPublicKey answers.
- [x] 4.2 In `crates/tailrace/tests/kms_signer.rs`, run the suite's signer group with `run(signer::GROUP, &signer::scenarios(), …)` over a rig of the stand-in, a fresh stand-in per scenario, as design D9 has it. Verify `devenv shell -- cargo test -p tailrace --test kms_signer` passes, the report naming the key service profile, every scenario run and none not applicable (assert on `report.ran` and an empty `report.not_applicable`).
- [x] 4.3 In the same file, test each scenario of spec `key-service-signer` the group does not: bound by an alias, unreadable; asked to sign as `gate`, absent with a request count of 0; nothing of a known access key or session token in the envelope's bytes; `AccessDeniedException` on Sign, absent carrying KMS's message; a disabled key at sign, absent; an `RSA_2048` key and an `EXTERNAL` origin, refused naming them; verified by a verify-only profile, named not expired; the envelope renamed to `gate` in signer and keyid, named `fleet`; the payload changed, refused; a host key's envelope against a KMS signer and a host key signer, refused naming both with a request count of 0; S5's second key, refused with words carrying its ARN; disabled then enabled, expired then not; pending deletion, unreadable carrying KMS's words; `AccessDeniedException` on GetPublicKey, unreadable; a listed RSA signing key, unreadable naming its spec; a loopback port nothing listens on, sign and verify each unreadable within ten seconds; the verify-only profile's sign, absent with a request count of 0. Verify `devenv shell -- cargo test -p tailrace --test kms_signer` passes.

## 5. The bindings

- [x] 5.1 Move the reading of `[signing]` into `profiles::signer::read` and add `plan` and `Signing` as design D8 has them; `bindings/mod.rs` hands the `[signing]` table to `read`, makes `path_of` `pub(crate)`, takes the binding's signer from the family's `Signing`, and names no key service. Keep `bindings/tests.rs`'s host key tests passing, changed only where they build the old `Signing`. Verify `devenv shell -- cargo test -p tailrace bindings` passes, and `grep -niw kms crates/tailrace/src/bindings` finds nothing.
- [x] 5.2 In `crates/tailrace/src/profiles/signer/tests.rs`, test spec `host-bindings`' new scenarios: `[signing] kms` alone reads as the key service signer with the ARN as signer; `kms` with `signer` or with `key` unreadable naming `[signing]` and saying the table names `kms` alone; `kms` naming an alias unreadable naming the value; and `plan` for a host key, a KMS key and no `[signing]` in design D8's order, binding nothing that reads a file or reaches a service. Verify `devenv shell -- cargo test -p tailrace profiles::signer` passes.

## 6. The run against a test account

- [x] 6.1 Write `crates/tailrace/tests/kms_account.rs` behind the `tools` feature, as design D9 has it: the signer group over `Reach::Host` against `TAILRACE_TOOLS_KMS_KEY`, with `TAILRACE_TOOLS_KMS_STRANGER` as the stranger, the key enabled and waited for before each scenario, expire disabling and waiting, the key enabled again when the group ends whether it passed or not, and a panic naming both settings and what the session needs (`kms:Sign`, `kms:GetPublicKey`, `kms:DescribeKey`, `kms:EnableKey`, `kms:DisableKey` on both keys, both `ECC_NIST_P256` signing keys) on a host that sets neither. Add the `kms` task to `crates/tailrace/moon.yml`, `cargo test -p tailrace --features tools --test kms_account`, tagged `check-kms`, its inputs those of `tools-ecr` with `$TAILRACE_TOOLS_KMS_KEY` and `$TAILRACE_TOOLS_KMS_STRANGER` in place of `$TAILRACE_TOOLS_ECR`, with a comment as `tools-ecr` has saying it runs outside the gate (core.38). Verify `devenv shell -- moon run cli:kms` on a host with neither setting fails naming both, and that `moon query tasks` lists `cli:kms` with the one tag `check-kms`.
- [ ] 6.2 Run `devenv shell -- moon run ':#check-kms'` with a live session on a host whose `devenv.local.nix` names two P-256 signing keys in a test account. This needs a live AWS session and two keys only ops can make: if none is at hand, leave the box open and say so in the final report, naming what ops must provide. Verify every scenario of the group passes against the account and the key is enabled afterwards (`aws kms describe-key` shows `Enabled`).

## 7. The record and the site

- [x] 7.1 Write `docs/adr/0016-the-key-service-signer-takes-a-p-256-key-by-its-arn-and-reads-its-standing-from-its-state.md` in the form of ADR 0015, status proposed, holding design D10's decisions with their considered options and sources (79 to 81, 117, C.7; rules core.1, core.7, core.19, core.21, core.38; KMS's API reference for Sign, GetPublicKey and key states). Verify `devenv shell -- moon run root:adrs` passes.
- [x] 7.2 In `site/src/content/docs/guides/integrations/signers.mdx`, mark the key in AWS KMS `built` and add a section after the host key's: make an asymmetric `ECC_NIST_P256` `SIGN_VERIFY` key; grant the signing host's role `kms:Sign` and `kms:DescribeKey`, and every verifying account `kms:GetPublicKey` and `kms:DescribeKey`; bind it with `[signing] kms = "<key ARN>"`; list it in the policy as `[signers.<name>] kms = "<key ARN>"`; disable a key to retire it, after which its signatures are reported expired; never schedule a key's deletion while anything it signed is in use, since its claims become unreadable. In `site/src/content/docs/guides/self-hosting.mdx`, show `[signing] kms` beside the host key's table and say the credential is the host's AWS chain, kept nowhere. Verify the site's moon build task passes in 8.1.

## 8. The gate

- [x] 8.1 Verify `devenv shell -- moon run ':#check-fast'` passes (format, lint with every feature, the toolchain, the records' numbers, the conformance coverage, every crate's tests and the site's build) and `devenv shell -- moon run ':#check-tools'` passes with nothing reaching AWS.
