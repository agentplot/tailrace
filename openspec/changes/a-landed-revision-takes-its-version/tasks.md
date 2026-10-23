# Tasks

Every cargo and moon command runs as `devenv shell -- <command>`. This
change builds on `the-seams-and-their-fakes` as it lands: read its
`answer.rs`, `artifact.rs` and the fakes before starting. It adds no
crate, language or file type, so no project or formatter entry is
added. Nothing under `crates/tailrace-core/src/` names `cargo`, `moon`,
`sluice` or any other word core.1 lists, in code or in comments.

## 1. Dependencies

- [ ] 1.1 Add to `[workspace.dependencies]` in `Cargo.toml`: `git-cliff-core = { version = "2.14", default-features = false, features = ["repo"] }`, `git2 = { version = "0.21", default-features = false }`, `glob = "0.3"`, `regex = "1"`, `semver = "1"`, `tempfile = "3"`; take the first five and `toml` in `crates/tailrace-core/Cargo.toml`, and `tempfile` and `tailrace-fakes` as dev-dependencies; in `crates/tailrace-core/moon.yml` add `fakes` to `dependsOn` and `/crates/tailrace-fakes/**/*` to the test task's inputs (design D3, D8). Verify `cargo check --workspace --all-targets` succeeds and `Cargo.lock` holds one `git2` (`cargo tree -d -p tailrace-core` shows no second git2).

## 2. The answers this change gives

- [ ] 2.1 In `crates/tailrace-seams/src/answer.rs` add the history as a source (`Source::history`, naming git and the remote or `local`) and the refusals `Why::NotLanded { branch, revision }`, `Why::DeclaredTwice { first, second, what }` and `Why::NameHeld { name, revision, other }` (design D7). Verify with unit tests in `answer.rs` that each round-trips through JSON with its source and git's words byte for byte.

## 3. The landed revision and the history

- [ ] 3.1 Write `crates/tailrace-core/src/landed.rs`: `Landed` with private fields, `Landed::check` and `policy_at` (design D2). Verify with tests over `FakeLanding` and `FakePolicy`: a revision not landed is refused naming the branch and revision and the fake policy records no call; the landing unreadable passes through with its words; a landed revision asks the landing first and returns the policy's answer as given (spec landed-policy: Nothing is read at a revision the branch has not landed; The policy for an audience is asked only at a landed revision).
- [ ] 3.2 Write `crates/tailrace-core/src/history.rs`: `History::open`, the blob at a path in a landed revision's tree, the commits touching paths through git-cliff-core's `commits`, the tags carrying a prefix from the local and the shared history with their peeled revisions and reachability, and the branch's remote (design D3, D6). Write the fixture helper `crates/tailrace-core/tests/common/mod.rs` that makes a repository, commits with given messages under given paths, tags, a bare shared repository and clones, with the git command in a `tempfile` directory and a fixed author. Verify with tests in `crates/tailrace-core/tests/history.rs`: a path that is not a repository is unreadable with git's words; a blob is read from the revision while the working tree holds another; a commit outside the paths is not listed; an annotated and a lightweight tag both peel to their commit; a tag on a revision the landed one does not reach is not kept; a tag only the shared repository holds is read.

## 4. The components and the rule

- [ ] 4.1 Write `crates/tailrace-core/src/declared.rs`: `Declared { rule, components }` and `Declared::read(&History, &Landed)` over the shape of design D4, with the rustdoc listing every key and its default. Verify with tests in `crates/tailrace-core/tests/declared.rs`, one per scenario of spec landed-policy: a checkout's edit and a later commit are not read; two components are read with their own outputs, paths and prefixes; two with one prefix or one name are refused naming both; no file and a file with no component give one component named `repository` over `**` with the default or declared rule; a malformed file is unreadable naming the file and revision with the parser's words.

## 5. The version and its claim

- [ ] 5.1 Write `crates/tailrace-core/src/version.rs`: `Derived`, `derive` and `touched` as design D5 orders them, the bump through git-cliff-core's `Changelog::new` and `bump_version()` with the rule's settings. Verify with tests in `crates/tailrace-core/tests/version.rs`, one per scenario of spec component-versions under: A version is derived from history alone; The rule runs over the component's own commits and tags; A feature or a fix makes a release, anything else a prerelease (1.3.0, 1.2.4, `1.2.4-pre.3` ordered between 1.2.3 and 1.2.4, the initial version); Tags are ordered by the version rules (1.10.0 over a later 1.9.0); A revision already claimed keeps its version; A publish versions the components a landing touched (S15: one component touched, the other unanswered; a shared path versions both); and with `FakeBuild` bound beside, the fake build records no call (The build never learns the version).
- [ ] 5.2 Add `claim` to `version.rs` as design D6 says: the local lightweight tag without force, the branch's remote, `git push --porcelain` and `git ls-remote` as git commands, the local tag removed on a refusal. Verify with tests in `crates/tailrace-core/tests/claim.rs` over a bare shared repository and two clones: a new name lands in both histories; the same revision claimed again changes nothing and is not an error; a branch with no remote claims locally alone; a name the other clone pushed at another revision is refused naming both revisions with git's words verbatim, and neither history's tag moved; a shared repository that is gone answers unreadable with git's words, never refused; a derivation after a claim gives the claimed version.
- [ ] 5.3 Rewrite `crates/tailrace-core/src/lib.rs`'s doc comment to list the modules and the clauses each keeps, and re-export them. Verify `grep -rniwE 'oci|ecr|ghcr|oras|cosign|moon|sluice|kms|npm|maven|cargo|flux|gh' crates/tailrace-core/src` finds nothing (core.1).

## 6. The gate

- [ ] 6.1 Verify `devenv shell -- moon run ':#check-fast'` passes: format, lint, toolchain, ADR numbers, every crate's tests and the site's build.
- [ ] 6.2 Verify by hand that `nix build` of the flake's default package still builds with libgit2 compiled from source, since no check-fast task builds the flake (design, Risks).
