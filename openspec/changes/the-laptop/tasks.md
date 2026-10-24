# Tasks

Every cargo and moon command runs as `devenv shell -- <command>`. Read
`crates/tailrace/src/lib.rs`, `main.rs` and `commands/`, the profiles
under `crates/tailrace/src/profiles/` and their constructors,
`tailrace-seams/src/bind.rs`, `policy.rs`, `projection.rs` and
`claim.rs`, `tailrace-core/src/declared.rs` and `verify.rs`, and the
conformance crate's `seams/projection.rs`, `seams/policy.rs`,
`seams/binding.rs`, `register.rs` and `lib.rs` before starting. No trait,
shared type, fake or core module changes; a scenario that cannot pass
without one is reported with `crew needs`. The change adds no crate,
language or file type, so no moon project or formatter entry: its tests
run in the `cli` project's `test` task (`check-fast`) and, for the ones
needing moon, sluice and ssh-keygen, its `tools` task (`check-tools`). No
file under `crates/tailrace/src` outside `profiles/` writes a word core.1
rules out: `bindings/` hands each table's key to the `bind` function of
its seam's profile family under `profiles/`, which alone matches the
words (design D2).

## 1. The policy file profile

- [x] 1.1 Write `crates/tailrace/src/profiles/policy/mod.rs` and `policy/file/mod.rs`, add `pub mod policy;` to `profiles/mod.rs`, and implement `Policy` over `.config/tailrace.toml` read with git from the revision's tree: `[signers.<name>]` with one verifier key, `kinds` and `checks`; `[acceptance]`, `[layout]` and `[channels.<name>]` each with an ordered `require` and, for a channel, `bound_to` (design D3). Verify with unit tests over a scratch repository: a host key listed answers a `NamedSigner` with that `verified_by`, kinds and no check; two verifier keys unreadable naming the table; acceptance's two claims in order; a bound channel; acceptance with `bound_to` unreadable; a signer `[signers]` lacks unreadable naming it; no file, and no `[layout]`, absent naming the audience; a malformed file unreadable with the parser's words; a working tree's edit not read.
- [x] 1.2 Run the conformance suite's policy group against the profile in `crates/tailrace/tests/policy_file.rs`, its rig writing each policy the group asks for as a commit of a scratch repository and making the repository unreachable for the unreachable scenario. Verify `devenv shell -- moon run cli:test` passes with every scenario of the group.

## 2. The file and image projections

- [x] 2.1 Write `crates/tailrace/src/profiles/projection/mod.rs` and `projection/file/mod.rs`, add `pub mod projection;` to `profiles/mod.rs`: pack answers the output's one file's bytes unchanged, metadata `file`, the output's name and the version, `same_again: true`; no file or several refused naming the output and the files; a read error unreadable with the operating system's words; render and serve absent naming the operation (design D4). Verify with unit tests: the bytes `abc` packed at `1.4.0` answer `abc` and that metadata; packed at `1.5.0-pre.1` the same bytes; three files refused naming all three; a removed file unreadable carrying the operating system's words; render absent and nothing written under the path given.
- [x] 2.2 Write `projection/image/mod.rs`: pack checks the one file is a tar holding `oci-layout` and `index.json` at its root by reading ustar headers, answers its bytes unchanged, metadata `image`, the output's name and the version, `same_again: true`; render and serve absent (design D5). Add a small OCI image archive as a test fixture under `crates/tailrace/tests/fixtures/` (made once with the tar format's fixed fields, its making written in a comment of the test that uses it), or build it in the test from fixed bytes. Verify with unit tests: the fixture packs to its own bytes at `0.3.0` and at `0.4.0`; a gzip of it refused saying it is not a tar holding an OCI image layout; a tar of `manifest.json` alone refused naming `oci-layout`; another file than a tar refused; a removed file unreadable.
- [x] 2.3 Run the projection group's two pack scenarios, packed the same again and a projection that cannot be reached, against each projection in `crates/tailrace/tests/projections.rs`, the rig's `output` a file in a temporary directory and `unreachable` removing it; the render and serve scenarios are not run, with a comment citing design D7's departure from core.33. Verify `devenv shell -- moon run cli:test` passes.

## 3. A host key that only verifies

- [x] 3.1 Add `HostKey::verifying()` to `crates/tailrace/src/profiles/signer/host_key/mod.rs`, with no signer and no key path: verify unchanged, sign absent saying no identity is at hand, reading no file (design D6). Verify with unit tests in `host_key/tests.rs`: an envelope a keyed `HostKey` signed as `laptop` is told `laptop` by the verify-only profile given a policy listing `laptop`'s key; sign asked as `laptop` answers absent, with `HOME` pointed at an empty directory so a read would fail.

## 4. The bindings file

- [x] 4.1 Write `crates/tailrace/src/bindings/mod.rs` and add `pub mod bindings;` to `lib.rs`: the file's path from `XDG_CONFIG_HOME` or `HOME` (design D1); the reader of `[store] layout`, `[landing] sluice | branch` with an optional `remote`, and `[signing] signer, key`; `~/` expanded against `HOME`; absent naming the path when there is no file; unreadable, with the parser's words where it has them, for a malformed file, a table with no key or two, a relative path, `[build]`, `[projections]`, `[ledger]` or an unknown table. Verify with unit tests over files in a temporary directory, the path and the home passed in rather than read from the process's environment: each table read; each refusal above unreadable naming the file and the table or key; no file absent naming the path.
- [x] 4.2 Add the `bind` functions of design D2 to `profiles/store/mod.rs`, `profiles/landing/mod.rs`, `profiles/signer/mod.rs`, `profiles/build/mod.rs`, `profiles/policy/mod.rs` and `profiles/projection/mod.rs`, each matching its keys to its profiles and answering unreadable naming a key it does not know. Make the `Binding` from what the file names through them, for publish or for verify (design D2), `bindings/` naming no tool: the layout made by `Layout::create` when its path is missing or an empty directory and the command is publish, else `Layout::open`; `Sluice` or `PlainBranch` at the repository with the remote; `HostKey::new` or `HostKey::verifying`; `Moon::new`; the policy file profile; `file` and `image` projections; a keeper answering absent naming the ledger's binding; no target. `Binding.signer` becomes `Option<SignerId>`. Verify with unit tests: a file naming a layout path that does not exist makes the layout for publish and, for verify, answers unreadable and leaves no directory; with no `[signing]`, the binding's signer is `None` and its signers hold the verify-only host key; the keeper's append and chain answer absent naming the ledger's binding.

## 5. The commands bound to the file

- [x] 5.1 Change `run` to take `&Answer<Binding>` and `main` to load the binding for publish and verify only (design D2); remove `unbound()`. publish with no signer answers absent naming the `[signing]` binding before anything is read. Update `crates/tailrace/tests/conformance.rs` to pass `Ok(binding)`. Verify `devenv shell -- moon run cli:test` passes, the surfaces group among it.
- [x] 5.2 Update `crates/tailrace/tests/cli.rs`, which runs the built binary: with `XDG_CONFIG_HOME` at an empty directory, `tailrace publish HEAD` exits 4 naming the bindings file's path and writes no tag, file or record; with a file that is not TOML, `tailrace verify 0.1.0 --json` prints one JSON document saying unreadable with the parser's words and exits 3; with a file naming both sluice and a branch, publish exits 3 naming `[landing]`; `tailrace status --json` with no file prints the record's entry and exits 0. Verify `devenv shell -- moon run cli:test` passes.

## 6. The host group and the register

- [x] 6.1 Add `crates/tailrace-conformance/src/seams/host.rs` with the `host` group and its rig (design D7), registered in `seams/mod.rs` and `lib.rs` beside the other groups; its one scenario, "the host's bindings beside the landing's", cites 118. Remove 118's line from `register.rs` and, if no other entry names it, the `LAPTOP` constant. Run the group over `bindings/` in `crates/tailrace/tests/host.rs`, the rig's config directory a temporary one holding `sluice/config.toml`. Verify `devenv shell -- moon run conformance:test cli:test` passes and the coverage check (`moon run ':#check-fast'`) reports 118 cited and no longer pending.

## 7. The laptop end to end

- [ ] 7.1 Write `crates/tailrace/tests/laptop.rs` behind `#![cfg(feature = "tools")]`, over the scratch harness and its sluice gate, running `env!("CARGO_BIN_EXE_tailrace")` with `HOME` and `XDG_CONFIG_HOME` in the scratch directory and none of the `SLUICE_*` or `MOON_*` variables (design D8): the repository's moon tasks leave `dist/notes.txt` and `dist/image.tar` (the image fixture copied by its task), its policy file declares both outputs, `[projections]` naming `file` and `image`, `[signers.laptop]` with an `ssh-keygen` key's public half, `[signers.box]` with the gate's, and `[acceptance]` requiring provenance from `laptop`; the bindings file names a fresh layout path, `[landing] sluice` with the remote, and `[signing]` with that key at mode 0600. Add `--test laptop` to the `tools` task's command in `crates/tailrace/moon.yml`.
- [ ] 7.2 In that test, S1: after sluice lands a revision, `tailrace publish <revision> --json` exits 0 naming the version the history derives, the index, two artifacts with their digests and ecosystem names `file` and `image`, a provenance on each signed by `laptop`, and sluice's verdicts carried in each; the layout's `index.json` names no ref but the version's. S2: the same command again exits 0 naming the same index, and a listing of every file under the layout and the host's record, with each file's size and digest, is unchanged. Verify `devenv shell -- moon run cli:tools` passes.
- [ ] 7.3 In that test, the second machine: a second clone of the remote, the layout copied recursively to another directory, and a second config directory whose file names the copy and `[landing] sluice` with the clone's remote and no `[signing]`; `tailrace verify <version> --json` there exits 0 and tells every provenance on the index's artifacts as made by `laptop` and counting, and each of sluice's verdicts with the provenance it was carried in. Verify `devenv shell -- moon run cli:tools` passes.
- [ ] 7.4 By hand, because the gate reaches neither a family repository's remote nor a second machine: build the binary with `nix build` from the flake at this branch; on the laptop, with a bindings file naming a layout, sluice and the host's key, publish a landed revision of a family repository that lands through sluice and whose policy file declares a file output and lists the laptop's key; publish it again and see the same index; copy the layout and clone the repository on a second machine with a bindings file naming the copy and no key, and run `tailrace verify <version>`. Record the two answers and the machines in the change's verify notes.

## 8. The record and the site

- [ ] 8.1 Write `docs/adr/0013-the-host-binds-its-seams-in-one-file-beside-sluice-s.md` (MADR, as 0011): the bindings file's place and form (design D1), what the host binds and what the repository binds (D2), and the policy file profile as the policy seam's one profile and why one is enough (D3, core.3). Verify `devenv shell -- moon run root:adrs` passes.
- [ ] 8.2 Update the site: `guides/self-hosting.mdx` gains the bindings file, with D1's example, and the policy file's signers and audiences, with D3's; `guides/integrations/stores.mdx` says the layout is bound by the host's file; `landings.mdx` says sluice and a branch are bound by it; `signers.mdx` says the host key is bound by `[signing]` and verify-only where none is named; `projections.mdx` marks the file and image projections' pack built and their render and serve not; the roadmap marks the laptop unit landed. Verify `devenv shell -- moon run site:build` passes with every internal link checked.

## 9. The gate

- [ ] 9.1 Run `devenv shell -- moon run ':#check-fast' ':#check-tools'` and fix what fails. Verify both layers pass, and `grep -rn` over `crates/tailrace/src` outside `profiles/` finds none of the words core.1 rules out.
