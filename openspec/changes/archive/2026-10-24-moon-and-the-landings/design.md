# Design

## Context

Bolt 1 left the build and landing seams as traits with fakes
(`crates/tailrace-seams/src/build.rs`, `landing.rs`): `ensure(revision,
outputs) -> Answer<()>`, `report(revision, output) -> Answer<Report {
build, outcome, files }>`, `landed(branch, revision) -> Answer<bool>`,
`verdicts_of(revision) -> Answer<Vec<Verdict>>`. The core's publish
already judges outputs against verdicts and refuses one no verdict
names, naming both (15, 111; `tailrace-core/src/publish.rs`, `judge`),
and carries each verdict in its provenance as `Carried` (12). The
layout store unit, beside this one, makes `crates/tailrace/src/profiles/`
and its `mod.rs` (its design D7).

The conformance suite unit (`the-conformance-suite-over-the-fakes`,
committed at 2c3cfcf, in code) gives the build group over a `BuildRig`
(`build()`, `second_host()`, `revision(&[(OutputName, Made)])`,
`unreachable()`) and the landing group over a `LandingRig` whose
`landed(&[Recorded { check, build, outcome, signer }])` stages what a
landing records and lets the landing make the envelope, with
`records_verdicts()` and `not_applicable` for a landing that records
none (its design D3, D3a). The register names 104 for this unit.

What sluice v0.37.0 writes, read from its source
(`crates/sluice/src/profiles/remote_refs.rs`, `local_files.rs`,
`signing/`, `sluice-core/src/records.rs`, `text.rs`) and its statement
(C.6, 23, 108, 109):

- A branch's records are JSON documents and a log, `log.jsonl`, one JSON
  object per line with `event`. A landing writes `{"event":"landing",
  "ids":[…],"from":R0,"to":R1}` before it moves the branch and
  `{"event":"landed","ids":[…],"to":R1}` after. On one host they are
  under `<git common dir>/sluice/<branch>/`; on a remote, in the tree of
  the one commit `refs/sluice/state/<branch>` names, at the same paths.
- A landing writes one commit per entry, first parent the revision
  before it, each ending in trailers, among them `Sluice-Tested: <the
  candidate's revision>`, the combination whose verdict it landed on.
- A signed run record is a file in the tree of
  `refs/sluice/runs/<signer>` on the remote, at
  `<combination>/<system or any>/<16 hex>.json`: a DSSE envelope
  `{"payload": base64, "payloadType": "application/vnd.in-toto+json",
  "signatures": [{"keyid": "SHA256:…", "sig": base64}]}` whose payload
  is an in-toto statement, predicate type
  `https://agentplot.github.io/sluice/run/v1`, one subject
  `<check>@<system>` per result with the key as its sha256 digest, and a
  predicate `{results: [{check, key, outcome, replayed?}], claims:
  {executor, confined, system}, branch, combination, at}`. The
  signature is an SSH signature (PEM) in the namespace `sluice-run`
  over DSSE's pre-authentication encoding. A replay is recorded inside
  the run it came in, `replayed: true`. A branch kept on one host signs
  nothing (sluice ADR 0032).

What moon 2.6 does, read from sluice's moon runner
(`crates/sluice/src/profiles/moon.rs`), which drives it the same way:
`moon exec <targets…> --on-failure continue` runs exactly those
targets; `.moon/cache/runReport.json` holds one action per task,
`RunTask(<target>)`, with `status` `passed`, `failed`, `cached` or
`cached-from-remote` and a `hash-generation` operation whose
`meta.hash` is the task's hash, the key sluice signs; `moon query
tasks` and `moon task <target> --json` describe tasks.

## Goals / Non-Goals

**Goals:**

- Three profiles that implement their seam and nothing of the core, and
  name no other profile (core.1, core.2).
- The join of 13 proven against real tools: the hash moon reports for
  an output is the key sluice's verdict carries.
- Every envelope sluice wrote carried whole, and nothing verified or
  signed by a landing (core.7, core.8).
- The suite's build and landing groups passing over the real profiles
  with no change to any scenario above a marker.

**Non-Goals:**

- Verifying an SSH signature: the host-key signer unit.
- Choosing a profile for a host or reading the host's bindings file:
  the laptop unit. The profiles here are constructed by a caller.
- moon's outputs shared across checkouts, or filled from a store: bolt
  3's moon's cache unit.
- Pruning the build checkouts this profile makes.

## Decisions

### D1. The envelope holds every signature, and a verdict says where it was found

As the design agent ruled for this unit (12, C.5), in
`crates/tailrace-seams/src/claim.rs`:

```rust
pub struct Envelope { pub payload_type: String, pub payload: Vec<u8>, pub signatures: Vec<Signature> }
pub struct Signature {
    pub signer: SignerId,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub keyid: String,                 // as the maker wrote it; empty when it named none
    pub sig: Vec<u8>,
}
impl Envelope {
    pub fn signed(payload_type: impl Into<String>, payload: Vec<u8>, signature: Signature) -> Envelope;
    /// The one signature of an envelope that holds exactly one.
    pub fn sole(&self) -> Option<&Signature>;
}
```

`Claim::from_envelope` answers `None` unless `sole()` is `Some`, so an
envelope of tailrace's form holds one signature (7) and one with any
other number is no claim of tailrace's form. A carried envelope keeps
every signature in order, each `signer` the identity the landing found
it under and `keyid` as written. `Envelope::digest()` is unchanged in
form (the canonical JSON), and no envelope is held anywhere yet, so the
field's rename moves no stored digest.

In `landing.rs`, `Verdict` gains `pub found: String`: where the landing
found it, in its own words. In the core's `provenance.rs`, `Found`
gains `pub at: String`, and `Carried::from_landing` copies the
verdict's `found` into it, so the carrier's claim says where it was
found (12).

The fake signer's `verify` names a named signer one of whose signatures
the envelope holds and verifies; its `sign` makes `Envelope::signed`.
The signer trait's doc says the same. Every literal that builds an
`Envelope`, a `Signature` or a `Verdict` (the fakes, the core's
provenance and its tests, the suite's rigs and world) follows; no
behaviour of theirs changes.

Alternatives: a second type for a carried envelope beside `Envelope`,
which the signer seam's `verify(&Envelope, …)` could not be asked of
without a trait change; dropping the key id, which the design agent
ruled out (C.5: "carried whole").

### D2. Where the profiles live, and how they run git and moon

`crates/tailrace/src/profiles/mod.rs` gains `pub mod build;`,
`pub mod landing;` and `mod run;` beside the layout store's
`pub mod store;` (whichever unit lands second keeps the other's lines).
`profiles/build/mod.rs` holds `pub mod moon;`; `profiles/landing/mod.rs`
holds `pub mod branch;` and `pub mod sluice;`.

`profiles/run.rs` is the one way the profiles run a tool: a `Tool {
program: PathBuf, source: Source }` with `output(dir, args, env_drop)`
that answers the command's output, or `Not::Unreadable` naming the
source with the tool's words verbatim: the spawn's `io::Error`
`to_string()` when it cannot start, and its stderr trimmed when it
exits other than the caller allows. Each profile takes the programs it
runs at construction (`git` and `moon` by default, found on `PATH`), so
a rig can point one at a path that is gone. Nothing under `profiles/`
imports `tailrace_core` (core.2), and the words moon and sluice appear
only under `profiles/` and the tests (core.1).

Every operation of every profile begins with one read of its source,
the same command for every operation of that profile, whose failure is
the operation's unreadable answer: `git -C <repository> rev-parse
--git-common-dir` for the branch landing and the moon build, and `git
-C <repository> ls-remote <remote> refs/sluice/*` for the sluice landing
bound to a remote. So one unreachable source gives the same words to
every operation, as the suite's `unreadable` asserts (58, 59).

Objects are fetched with `git fetch --no-tags --no-write-fetch-head
<remote> <ref>…` and read by the object id `ls-remote` answered, so the
profiles write no ref into the repository.

### D3. moon: which task makes an output

The repository names it in its policy file at the revision, in a table
the core's reader leaves to its owner (`declared.rs`):

```toml
[build.outputs.site]
task = "web:build"
```

`Moon` reads `.config/tailrace.toml` with `git show
<revision>:.config/tailrace.toml`, never from a checkout (65, core.22),
and takes `task` of each output's table. A file or a table that is
missing names no task for the output. The table is per output so the
declared-command build (C.4) adds its `command`, `inputs` and `shell`
keys beside `task` later. The design agent agreed to a map from output
to task (construct's question 4); where it lives is this design's.

Alternatives: the output's name as the moon target, which the suite's
outputs (`site`, `server`) and the projections' keys cannot be; a tag
on the moon task, which moon's config would carry for tailrace.

### D4. moon: ensure and report

`Moon::new(repository: PathBuf)` with `.moon_program(path)` and
`.git_program(path)`; its source is `Source::new(Seam::Build, "moon")`.

A revision's checkout is a detached worktree at
`<git common dir>/tailrace/builds/<revision>`, made by `git worktree add
--detach` the first time and reused after, holding exactly the
revision's tree and nothing of the publisher's checkout (core.11).

**ensure**: the outputs whose task the policy file names are run in one
`moon exec <targets…> --on-failure continue` in the checkout, stdin from
`/dev/null`, every `MOON_*` variable of tailrace's own environment
removed, after removing a run report left there. moon's exit status is
not the answer: a failed task is an outcome (75). A run that leaves no
run report is unreadable with moon's words. Then each output is unmade
when its task has no `RunTask` action in the report, or when its task's
declared output files and globs (`moon task <target> --json`,
`outputFiles` and `outputGlobs`, relative to the workspace) match no
file in the checkout. Unmade outputs, with those that name no task,
refuse `Why::Unmade { outputs }`; otherwise `Ok(())`.

**report**: `moon task <target> --json` first, which makes report
unreadable whenever moon cannot run. Then the run report's action for
the target: `hash-generation`'s `meta.hash` is the `BuildId`; `passed`,
`cached` and `cached-from-remote` are `Outcome::Pass`, `failed`
`Outcome::Fail`; `files` are the absolute paths in the checkout that
the task's declared outputs match. No checkout, no report or no action
for the target answers absent naming the output. moon caches only a
task that passed, so a cached answer is a pass.

The hash is the key sluice signs, since sluice reads the same field of
the same report (C.3, C.5). The end-to-end test proves it (D9).

Alternatives: `lastRun.json` under `.moon/cache/states/`, which sluice
reads only for an exit code and a time; running moon in the publisher's
checkout, which holds what the revision does not (core.11).

### D5. sluice: landed

`Sluice::new(repository: PathBuf, remote: Option<String>)` with
`.git_program(path)`; its source is `Source::new(Seam::Landing,
"sluice")`.

The branch's log is the remote's when a remote is bound and the
remote's listing holds `refs/sluice/state/<branch>`: `log.jsonl` of
that commit's tree, fetched. Otherwise it is
`<git common dir>/sluice/<branch>/log.jsonl`, and a missing file is a
branch sluice has landed nothing on. Only `landing` and `landed` lines
are read; any other event, and any field the lines carry beyond `ids`,
`from` and `to`, is ignored.

A landing is done when a `landed` line follows its `landing` line with
the same `ids` and `to`. Its commits are `git rev-list --first-parent
<from>..<to>`, the commits fetched from the remote first when it is
bound. The revision is landed when it is one of them. A landing begun
and not done lands nothing.

### D6. sluice: verdicts-of

With no remote bound, sluice signed nothing: after the opening read,
the answer is an empty set (sluice ADR 0032). With a remote:

1. The revision's `Sluice-Tested` trailer (`git log -1
   --format='%(trailers:key=Sluice-Tested,valueonly)' <revision>`) is
   the combination. A revision with none was landed by no verdict:
   an empty set.
2. For each `refs/sluice/runs/<signer>` the listing holds, the tip is
   fetched and its files under `<combination>/` read (`git ls-tree -r`,
   `git cat-file blob`).
3. Each file is parsed as the DSSE envelope and the run statement
   above. A file that is not one, or whose statement is not a run, is
   unreadable naming the signer's ref and the path, with the parser's
   words; sluice writes none, so one is a record changed by hand. A
   statement whose `combination` is not the combination is left out:
   the path is not signed, the statement is.
4. Each result that is not `replayed` is one `Verdict`: `check` the
   result's `check`, `build` its `key`, `outcome` its `outcome`,
   `signer` the ref's `<signer>`, `envelope` the whole file (D7),
   `found` `<remote> refs/sluice/runs/<signer>:<path>`.
5. Each result that is `replayed` is carried as every result recorded
   with the same `key` and the same `claims.system`, not replayed, with
   outcome `pass`, in any file under any signer's ref, each a `Verdict`
   from its own file as in 4. Which of them counts is verify's, through
   the signers the policy names (9, 104, core.7). A replay with none
   gives no verdict; the replay's own envelope is never carried as a
   pass, since sluice signs only what ran (C.5; sluice 108).

The parsed files are kept by blob id for the life of the handle, so a
publish reads each once.

Alternatives: reading only the signers a policy names, which the
landing seam is not given and which would make the landing judge
(core.7); leaving a malformed record out as sluice's own reader does,
which would answer an unreadable record as no verdict (58).

### D7. A DSSE envelope as sluice wrote it

`profiles/landing/sluice/dsse.rs` reads the file into `Envelope {
payload_type: payloadType, payload: base64-decoded payload, signatures:
[Signature { signer: <signer>, keyid, sig: base64-decoded sig }] }`,
and writes one back in DSSE's JSON. The base64 is the standard alphabet
with padding, sluice's encoding, so decoding and encoding again gives
the bytes sluice wrote. `base64` becomes a workspace dependency for it.

### D8. A plain branch

`PlainBranch::new(repository: PathBuf, remote: Option<String>)` with
`.git_program(path)`; its source is `Source::new(Seam::Landing,
"branch")`. **landed**: the branch's head is `refs/heads/<branch>` on
the remote's listing when a remote is bound, fetched, and the
repository's own otherwise; a branch that does not exist has landed
nothing; `git merge-base --is-ancestor <revision> <head>` exiting 0 is
landed, 1 not landed, anything else unreadable with git's words.
**verdicts-of**: `git rev-parse --verify --quiet <revision>^{commit}`
succeeding answers an empty set; failing answers absent naming the
revision.

### D9. The tests, by layer

`crates/tailrace/Cargo.toml` gains `[features] tools = []`. Files that
need moon, sluice or `ssh-keygen` begin `#![cfg(feature = "tools")]`,
so `cargo test -p tailrace`, the `check-fast` task, compiles them to
nothing. `crates/tailrace/moon.yml` gains

```yaml
  tools:
    command: 'cargo test -p tailrace --features tools --test build_moon --test landing_sluice --test landings_end_to_end'
    inputs: [the test task's inputs]
    tags: [check-tools]
    options: {runFromWorkspaceRoot: true}
```

and the root `lint` task, with the commit hook's clippy in
`devenv.nix`, gains `--all-features`, so the tools tests are linted.

`crates/tailrace/tests/scratch/mod.rs`, shared by the files below:

- a scratch repository in a temporary directory, made with the git
  command under a fixed author, committer and dates and no global or
  system configuration, as the suite's fixture is made; a bare remote
  and a second clone;
- a moon workspace in it: `.moon/workspace.yml` with its projects and
  `vcs`, each project's `moon.yml` with shell tasks that write their
  declared outputs and exit as told, and the policy file's
  `[build.outputs]`;
- a recorder of sluice's records as sluice v0.37.0 writes them (Context):
  landing commits carrying `Sluice-Tested`, the state commit's
  `log.jsonl` and `version` pushed to `refs/sluice/state/<branch>`, and
  run records signed by `ssh-keygen -Y sign -n sluice-run` over DSSE's
  pre-authentication encoding with a key per signer, `keyid` the key's
  `SHA256:` fingerprint, added to `refs/sluice/runs/<signer>`.

The recorder stands in for sluice only where the suite asks a landing
to record a given build identity, which no real sluice landing can be
told; a real sluice landing in the same layer proves the reader against
what sluice itself writes.

| file | layer | what |
|---|---|---|
| `tests/build_moon.rs` | tools | `MoonRig`: a scratch repository per scenario; `revision` commits the tasks and the policy file and ensures them in this checkout and in a second clone's, so the build has left them; `second_host` the second clone's `Moon`; `unreachable` removes the link in the rig's directory that `moon_program` names and answers the operating system's words. The build group, and the moon-build spec's scenarios the group does not play |
| `tests/landing_sluice.rs` | tools | `SluiceRig` over the recorder with a bare remote; `unreachable` removes the remote and answers the words `git ls-remote` gives for it. The landing group, and the sluice-landing spec's scenarios: a landing of two entries, a landing begun and not done, a commit before sluice, a branch on one host, two checks, two signers, a replay and a pass on another system, a malformed record, the envelope written back. Then a real landing: `sluice submit main` in a scratch repository whose remote branch's policy names the signer `box`, with `HOME` and every `XDG_*` directory in the test's own temporary directory and no `SLUICE_*` variable of the gate's run, the host's `[signing]` naming `box` and its key; landed and verdicts-of over it, the envelope byte for byte as the file under `refs/sluice/runs/box` |
| `tests/landing_branch.rs` | fast | `BranchRig`, `records_verdicts()` false; the landing group, its recorded-verdicts scenario reported not applicable; the branch-landing spec's scenarios |
| `tests/landings_end_to_end.rs` | tools | the core's `publish` and `verify` over moon and a landing, the store, signer, policy and projections the fakes: the join (the hash report answers is the key of sluice's verdict for the same task); an output whose task the gate did not require refused naming the output and the verdicts' identities, with no put (15); S13 over the plain branch |

The sluice policy keys for signers and trust, and the host's `[signing]`
table, are taken from sluice v0.37.0's own documentation
(`docs/examples`, the executors-and-systems guide) and its peers tests.

### D10. The tools layer and its pins

`.config/sluice.toml` requires `["#check-fast", "#check-tools"]` on
`bolt/*` and `main`. `devenv.nix` fetches sluice's release binary as it
fetches proto's, `v0.37.0`, one sha256 per asset sluice releases
(`aarch64-darwin`, `aarch64-linux`, `x86_64-linux`), and adds
`pkgs.openssh`; on a system sluice releases nothing for, sluice is not
in the shell and the tools tests fail at their first step naming it.
moon is already `.prototools`' (core.26). `.moon/workspace.yml`'s
comment and `CLAUDE.md`'s layers table name `check-tools`.

### D11. 104 leaves the register

Below the landing group's marker, *two signers, each by its own name*,
citing 104: `not_applicable` where `records_verdicts()` is false;
otherwise a revision landed with a `test` pass by `box` and one by
`forge`, both on one build, read back as two verdicts, each naming its
recorder and every signature of its envelope that signer. Its entry,
and the unit's name constant if nothing else uses it, leave
`register.rs` in the same commit (core.33). The fake rig passes it, as
every rig that records verdicts must.

## Risks / Trade-offs

- [The recorder writes sluice's format from a reading of sluice's
  source, and sluice may change it] → the real sluice landing in the
  same layer fails when the reader and sluice disagree; sluice's version
  is pinned and moves only in a commit that runs that test.
- [A sluice landing with its trailer edited by hand names another
  combination] → the trailer is unsigned, but every verdict names a key,
  and an output is offered only when a verdict names its build's own
  hash (15), so a forged trailer cannot get other bytes offered.
- [Running sluice inside the gate's own sluice run] → the nested sluice
  gets a home, config, state and cache of its own and none of the
  gate's `SLUICE_*` variables; sluice's own tools layer does the same.
- [The replay search reads every record of every signer] → once per
  handle, cached by blob; a family repository's runs refs hold
  thousands of records at most.
- [A landed revision whose output task sluice's gate did not run, being
  unaffected, has no verdict naming its hash, and a publish of it is
  refused (15)] → that is 15 as written; whether an earlier landing's
  verdict for the same hash should stand for it is the statement's to
  say, and is named for the design agent.
- [Build checkouts accumulate under the common directory, one per
  revision published] → each is a worktree git can prune; pruning is
  left to the cache unit of bolt 3.
- [`moon task --json` field names are read from moon 2.6's output, not
  from a schema] → the build group's tests run real moon at the pinned
  version, so a renamed field fails them.
- [The envelope's change touches every crate that builds one, beside
  the host-key signer and the laptop units of this bolt] → the change
  is mechanical and lands with this unit; crew names the overlap.
