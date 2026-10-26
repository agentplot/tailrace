# Design

## Context

The statement this rests on, at agentplot/blueprints b720017: 15, 16,
52 to 54, 58, 74 to 76, 96, 118, 130, 135, 139, C.1, C.3, C.5, C.6,
C.16, and the madswan-design rulings given with the unit (8e649e5): `left` is the
build's (139), not a projection's; the pool job's push is a step of the
repository's own workflow after the queue's run step, with a credential
that writes cache entries and nothing else (135, core.14); and there is
no setup command, the guide shows the workflow.

What the tree holds now (the bolt `tailrace-sync-and-serve`, a41dbfd):

- `tailrace push [revision]` (`crates/tailrace-core/src/push.rs`) asks the
  landing, reads its verdicts, and writes each passing verdict's build
  identity with `cache::write(seams, &Output { name: <check>, files: []
  }, &build)`; the moon cache projection packs
  `<root>/.moon/cache/outputs/<hash>.tar.gz`, `<root>` being the checkout
  the bindings were made in.
- `cache::sync` resolves `cache/<build>` and renders each entry through
  every projection that renders cache entries, with `fill` (private) and
  `packs_none`; `tailrace sync` renders into `history.workdir()`.
- `publish` (`publish.rs`, `Run::component`) asks landed, declared, the
  projections, the verdicts, then `build.ensure` and `build.report` per
  output, `judge`, `claim`, the packs, the offer and the index. It never
  reads the store's cache.
- The moon build (`profiles/build/moon/`) runs `moon exec` in a detached
  worktree `<common>/tailrace/builds/<commit>`, whose
  `.moon/cache/outputs` is its own directory unless the repository's own
  shell links it, as this repository's devenv shell links every
  checkout's to `<common>/moon-outputs`.
- `bindings::bind` requires `[landing]` and `[store]` for every command
  and `[signing]` for publish.

Vendor facts read for this change:

- moon 2.6.1 (`moon exec --help` in the devenv shell): no mode that
  replays from the cache without running a task on a miss; `--cache`
  takes read-write, read, write and off. The run report is
  `.moon/cache/runReport.json` under the workspace root, each `RunTask`
  action with a status (`passed`, `failed`, `cached`,
  `cached-from-remote`) and a `hash-generation` operation
  (`report.rs`). moon archives every task that passes, with declared
  outputs or none, as `<hash>.tar.gz` (the push unit's design, read in a
  scratch workspace).
- GitHub Actions: a push made with `GITHUB_TOKEN` starts no `push`
  workflow; a push made with a GitHub App installation token does
  (docs.github.com, "Triggering a workflow from a workflow"). sluice
  lands on GitHub as a leased push by the host's or the pool's app, so a
  workflow `on: push` to the landed branch runs once per landing.
- Amazon ECR: pushing to one repository needs
  `ecr:BatchCheckLayerAvailability`, `ecr:InitiateLayerUpload`,
  `ecr:UploadLayerPart`, `ecr:CompleteLayerUpload`, `ecr:PutImage` and
  `ecr:BatchGetImage` on that repository's ARN, and
  `ecr:GetAuthorizationToken` on `*` (AWS, "IAM permissions for pushing
  an image to an Amazon ECR private repository").
- sluice's Actions pool (agentplot/sluice `openspec/changes/actions-pool`,
  in flight): the run job has `permissions: contents: read`, no
  environment and no identity token; `sluice job run --claim <c>
  --branch <b>` makes the run's checkout from the combination's ref and
  runs its checks there. It does not yet say where that checkout is.

## Goals / Non-Goals

**Goals:**

- A check job on a runner that keeps nothing pushes what its run made,
  before the landing, with nothing of the landing read.
- A publish on any host offers the bytes the landing's checks built,
  read from the store, and never what its own host built in their place.
- A reader of the self-hosting guide can tell which of the two ways is
  theirs and copy what it needs.

**Non-Goals:**

- Running anything after a landing: the core starts nothing (130). The
  driver, the forge's job or a person runs the commands.
- A record of pushes, as for the push after a landing (61).
- Writing a workflow into a repository.

## Decisions

### D1. `left` on the build seam

`crates/tailrace-seams/src/build.rs`:

```rust
/// 139: one build identity a checkout's run left.
pub struct Left {
    pub build: BuildId,
    pub tasks: Vec<String>,
    pub outcome: Outcome,
    pub files: Vec<String>,
}

pub trait Build {
    fn ensure(..);
    fn report(..);
    /// 139: the build identities whose outputs the checkout at `at`
    /// holds, read from the build's own records and nothing of
    /// tailrace's; an empty list where it ran nothing.
    fn left(&self, at: &str) -> Answer<Vec<Left>>;
}
```

`at` is a path, as `SyncAsk::at` is. `Left` round-trips through JSON
like `Report`.

Through moon (`profiles/build/moon/`): `left` opens as every operation
does (the common git directory, then `moon --version`, so a moon that
cannot run is unreadable first), then reads `<at>/.moon/cache/runReport.json`;
none there answers `Ok(vec![])`. `RunReport` gains an iterator over its
tasks. Each task with a hash and an outcome whose archive
`<at>/.moon/cache/outputs/<hash>.tar.gz` exists (the directory followed
where it is a link) gives one `Left`, grouped by hash in the report's
order, its `files` the declared outputs present under `at` from `moon
task <target> --json`, as report reads them. A failed task leaves no
archive, so it is never named; `outcome` is carried for a build profile
that names failures. No task is run.

The fake build (`tailrace-fakes/src/build.rs`) gains `BuildOp::Left` and
`left_in(at, Vec<Left>)` to script a checkout; an unscripted checkout
answers an empty list. The build's conformance group gains two
scenarios below its marker, citing 74 and 75 (139 is not yet in the
statement list): what an ensure left, and a checkout that ran nothing.
`BuildRig` gains `ran_in(&Revision) -> String`, the checkout an ensure
of that revision ran in, and `fresh_checkout() -> String`; the moon rig
answers `<common>/tailrace/builds/<commit>` and a new empty directory.

Alternative: `left` on the projection seam, beside pack. Rejected by the
ruling: which entries a run left is the build's knowledge, and the
projection seam keeps pack, render, serve and pinned.

### D2. `tailrace push --left`

`crates/tailrace-core/src/push.rs` gains:

```rust
pub struct LeftAsk { pub at: String }
pub struct PushedLeft { pub at: String, pub entries: Vec<(BuildId, Answer<cache::Written>)> }
pub fn push_left(seams: &Seams, ask: &LeftAsk) -> Answer<PushedLeft>;
```

`seams.build.left(&ask.at)?`, then for each `Left` whose outcome is a
pass, once per build identity in order, `cache::write(seams, &Output {
name: OutputName::new(&left.tasks[0]), files: left.files }, &left.build)`.
Its answers are the push's, as D1 of the push after a landing says. No
landing, verdict, policy, signer or record is asked. JSON: `{ "at",
"entries": [ { "build", "answer", .. } ] }`, by the same serializer shape
as `Push`.

`commands/push.rs`: `--left`, conflicting with `revision` and
`--branch` through clap (`conflicts_with_all`), so either beside it exits
2 before the core is asked. `at` is the working tree of the repository
the command runs in, `History::workdir()`, absent naming it where there
is none; it is the moon cache projection's root, so pack reads the
archives `left` named. Text and exit as `tailrace push`.

### D3. Bindings for a push before a landing

`For::Left` beside `For::Push`. `bind` reads `[landing]` as optional for
it: where the file names none, the landing is `NoLanding`, a stub like
`NoLedger` answering absent naming `binding of the landing` for every
operation, which `push_left` never asks. `[signing]` is not needed, as
for push, and the layout is made as for push. Every other command keeps
its absent answer naming `[landing]`.

### D4. Publish fills from the store, then refuses an output it lacks

`Ask` gains `tree: String`, the working tree the cache is rendered into;
`commands/publish.rs` sets it from `History::workdir()`, absent naming
the working tree where there is none, before the core is asked.

In `Run`, a `filled: Option<Answer<BTreeSet<BuildId>>>` computed once per
publish, after the verdicts and before the first ensure, only where the
verdicts are not empty: for each passing verdict's build identity, once,
`store.resolve(&cache_name(build))`; an absence leaves it out; a digest
is rendered at `ask.tree` with `cache::fill` (made `pub(crate)`), and a
build identity rendered by at least one projection is in the set, while
an absence from `fill` (no projection renders cache entries) leaves it
out. Any refusal or unreadable answer is the publish's whole answer. The
store is asked only to resolve and read.

Then, in `Run::component`, after `judge` and before `claim`: where the
verdicts are not empty, the first output whose `report.build` is not in
the set refuses the version with `Why::NotInStore { output, build }`,
sourced to the bound store. Nothing has been claimed, packed or put. A
landing with no verdicts skips both steps and builds as before (C.6).

`Why::NotInStore` is added to `tailrace-seams/src/answer.rs` with its
JSON form and a round-trip test, and rendered by the command line's
refusal lines as `<output> from <build>: no cache entry in the store`.

Why after judge: the core knows an output's build identity only from
report, which follows ensure; the verdicts name checks, and which check
makes which output is the build profile's. So on a miss moon may run the
task, and what it made is refused, never offered (135: it never builds
to offer). An output no verdict names is refused by judge first, naming
the identities the verdicts carry, as before (15).

Alternative: refuse before ensure when any passing verdict's entry is
missing. Rejected: a landing's checks are mostly tasks with no output
the authors publish, so a missing test log would refuse a version whose
outputs are all held, and the refusal could name no output.

Alternative: on a host holding the archive locally, offer it without
the store. Rejected: the intent and 135 have the store answer every
publish of a landing with verdicts, and the driver pushes first, so the
landing host's publish reads what it pushed.

### D5. The revision's checkout shares the outputs directory

`Moon::checkout` makes `<checkout>/.moon/cache/outputs` a symbolic link
to the bound repository's `.moon/cache/outputs`, resolved through any
link it is (so this repository's `<common>/moon-outputs`), making the
target directory where it is missing. A real directory already there,
left by an earlier version, is removed and linked: its archives are
cache, which may go at any time (54). It is done at every ensure, so a
checkout made before this change is linked on its next use. This is
C.3's outputs directory shared by every checkout on a host; nothing
else of the bound checkout is shared.

### D6. Tests that publish a landing with verdicts push first

Every test that publishes a landing carrying verdicts now has the
store hold the entries: the core's publish tests and the conformance
world seed the fake store and fake projection with an entry per passing
build identity (`World::pushed`), and the tools-layer tests that publish
a sluice landing (`laptop.rs`, `landings_end_to_end.rs`, `cli.rs` where
it does) run `tailrace push` first, as the guide tells a landing host to.

The tools layer gains `crates/tailrace/tests/pool_run_publish.rs`,
listed in `crates/tailrace/moon.yml`'s `tools` task as its siblings are:
in a scratch repository with moon and sluice, a clone runs `moon exec`
over the gate's check tasks at the candidate's revision, as a pool's
run job does, then `tailrace push --left` with a bindings file naming
only a layout store; the clone is deleted; sluice lands the candidate in
the host repository; a fresh clone with the same layout, sluice and a
host key runs `tailrace publish`. The publish offers the version, each
artifact's bytes are those the first clone wrote, and the build
checkout's run report records each output's task `cached`. A second test
deletes one archive before the push and the publish is refused naming
that output.

### D7. The guide: which of two ways is yours

`site/src/content/docs/guides/self-hosting.mdx` gains a section, "After
a landing", in the reader's terms:

- **You land from your own machines.** sluice runs the checks on hosts
  you keep, and whatever submitted the work waits for it: crew, or `wt
  submit` waiting. Once it learns the work landed, it runs, on the host
  that ran the checks, `tailrace push <revision>` and then `tailrace
  publish <revision>`. The repository carries nothing for it; sluice
  runs nothing after a landing (118, C.16).
- **Your checks run on sluice's Actions pool.** No host keeps what the
  checks built, so each run job pushes before the landing is decided,
  and a workflow of the repository's publishes after it.

For the second, the page shows:

1. The step to add to the run job of the pool's workflow, after the
   step that runs `sluice job run`: install `tailrace` from its GitHub
   release, write a bindings file naming only `[store] registry`, reach
   the registry through Docker's credential helper for ECR with the
   cache key's AWS variables from repository secrets, and run `tailrace
   push --left` in the checkout sluice's step ran the checks in, with
   `if: ${{ !cancelled() }}`. The run job holds no identity token
   (sluice's pool keeps it out of the job that runs checks), so the
   credential is an access key, not a role assumed by OIDC.
2. The policy that key's IAM user holds: the push actions above on the
   cache entries' repositories alone, `repository/cache/*`, and
   `ecr:GetAuthorizationToken` on `*`; so a check that reads the key can
   write cache entries and offer, accept or move nothing (96, S9,
   core.14). The operator makes the repository `cache/moon` once before
   the first push, as C.1 says of a registry that makes repositories on
   push only by a template, so the key needs no `ecr:CreateRepository`.
   The page says the scope holds once the store names cache entries
   under `cache/<build>` apart from every component (C.1); until then
   the registry profile keeps every name in its one bound repository.
3. `.github/workflows/tailrace.yml`, the repository's own: `on: push` to
   the landed branch; `concurrency` one at a time, never cancelling;
   `permissions: contents: write` for the version's tag (22) and
   `id-token: write` for the store role, assumed by OIDC here since this
   job runs no check; `environment: tailrace`, holding the publisher's
   host key as a secret; steps: checkout with the whole history and
   sluice's refs (`refs/sluice/*`) fetched, the repository's own
   toolchain so moon runs, `tailrace` installed, the bindings file
   naming the registry, sluice on `origin` and the key, then `tailrace
   publish "$GITHUB_SHA" --json`.
4. That the policy file names the job's signer by the key's public half
   with `kinds = ["provenance"]`, and that a missing entry refuses the
   publish naming the output, which the pool's next run of that check
   pushes.

The builds page's moon row says `left` and the shared outputs directory;
the surfaces page's command line row names `push --left`; how it works
says publish reads the landing's outputs from the store.

## Risks / Trade-offs

- [A sluice landing published without a push first now refuses] → The
  refusal names each output and its build identity, the guide's first
  way says push comes first, and the existing tests show it.
- [On a miss moon runs the task in the publish's checkout before the
  refusal] → Nothing it made is offered; the cost is the one task. A
  replay-only mode would avoid it and moon 2.6.1 has none.
- [A check run on a pull request's head, which runs with no secrets,
  pushes nothing] → A landing that counts that run (sluice 113) finds
  the entry missing and the publish refuses naming the output; the guide
  says the pool's next run of the check pushes it. Named for the design
  agent.
- [The pool's step depends on sluice's template keeping room for it and
  saying where its checkout is] → Named out of scope; the guide's step
  names the checkout sluice's run step made.
- [The cache-only scope needs cache entries in repositories of their
  own, `cache/<build>` (C.1)] → The store-naming unit; until then the
  guide's scope is the statement's and says so, and no key narrower than
  the whole bound repository exists.

## Migration Plan

A host that publishes sluice landings adds `tailrace push` before
`tailrace publish`. Build checkouts made earlier are linked on their
next ensure (D5). Nothing kept changes form.
