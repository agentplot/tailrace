# Proposal

## Why

A machine cut off from its landing's remote cannot say whether a
revision landed: today both landing profiles answer unreadable the
moment the remote does not answer, so status on such a machine shows no
channel even where the ledger kept its chain, and every command that
asks the landing stops. 061bfa8 rules that the landing then answers
from the host's copy of the branch as last fetched, saying when and
carrying the remote's words, never a refusal and never a guess, and
that a publish reading not landed from such a copy refuses naming the
copy's time and asks again later
(`agentplot/blueprints:design/tailrace/requirements.md` 77, C.5, C.6 as
amended at 061bfa8; 59, 61, 78). The status unit ahead of this one in
the bolt left this reading here.

## What Changes

- **The landing says how it read its answer.** Landed and verdicts-of
  answer with how they were read beside what they answer: from the
  remote itself, or from the host's copy as last fetched, with when it
  was fetched and the remote's own words of why it did not answer, the
  same two the ledger's chain already says (60, 61, 77, 78). A landing
  with no remote bound reads the repository itself, which is its
  source. **BREAKING** for every caller of the landing seam, in the
  core and the tests.
- **Every read of the remote keeps a copy.** Each ref the landing reads
  from the remote, the branch for a plain branch, sluice's log and run
  records for sluice, is fetched into the host's copy, tailrace's own
  refs beside the repository's, remade at each fetch so the copy says
  when it was fetched (C.5, C.6). The copy is the repository's git
  refs, readable with git and rebuilt by the next fetch (62, 63).
- **Where the remote does not answer, the copy answers.** Landed and
  verdicts-of read the copy in place of the remote, answering as they
  would from the remote, with the oldest fetch time of the refs the
  answer rests on and the remote's words. A question the copy cannot
  answer, because nothing of it was ever fetched, is unreadable with
  the remote's words, as today, never not landed and never an empty
  set (58, 77).
- **Not landed from a copy is a refusal of its own.** Every command
  that asks whether a revision landed, publish first, refuses a no read
  from a copy naming the branch, the revision, when the copy was
  fetched and the remote's words, and saying to ask again once the
  remote answers, so a program tells it from a revision the remote
  itself has not landed (14, 59, 77). A yes from a copy is a yes: a
  branch only moves forward, so a revision it held when fetched it
  holds now. Nothing in tailrace asks again by itself (130).
- **Status says when the landing was fetched.** The channels' part of
  status names how the landing of the branch's head was read: from its
  remote, or from the copy as last fetched, with the time in UTC and
  the remote's words, on the command line, in its JSON and on the
  host's page (36, 56, 57, 60, 77). A machine cut off from both its
  landing's remote and its ledger's now shows every channel from the
  ledger's kept chain, each saying when.
- **The suite holds every landing profile to it.** The landing's group
  gains a scenario of a landing read as last fetched: read once, cut
  off, and read again, the same answers saying when and with the
  remote's words, and a revision landed after the fetch answered not
  landed from the copy, never landed by guess. The plain branch's
  group runs a second time with a remote bound, since a plain branch
  with none reads only itself; the fake landing answers as last fetched
  as the fake keeper answers as last read (93).

Out of scope, and what each waits on:

- Who asks a refused publish again, and when (77's "asks again later"):
  whatever runs the publish, which the host's bindings name (118, 130,
  C.16); no unit of the plan carves one yet. This unit makes the
  refusal one a program can tell apart and leaves the asking to it.
- The ledger's own copy as last read, and status's showing of it (60,
  C.13, C.14): built by the ledger and status units, and unchanged here.
- An empty channel read from the ledger's copy carrying the copy's time
  (60 as amended at b64d367), and a bare host's git-ref ledger on the
  landing's remote (118 as amended there): the units that bolt's review
  gave them to, not this one.
- Counting the copy among what status counts as read by the person's
  own `git fetch`: tailrace's copy is its own refs, moved only by its
  own reads (design D2).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `seam-contracts`: the landing's contract says how each answer was
  read, from the remote or from the copy as last fetched, and the
  refusal of a revision not landed as last fetched.
- `branch-landing`: the branch is read on the remote where it answers
  and from the copy where it does not, and unreadable only where no
  copy answers.
- `sluice-landing`: sluice's log and run records are read on the
  remote where it answers and from the copy where it does not, and
  unreadable only where no copy answers.
- `publishing`: a publish refuses a revision read not landed from a
  copy naming the copy's time and the remote's words.
- `seam-fakes`: the fake landing answers as last fetched once made
  unreachable.
- `conformance-suite`: the landing's group holds every landing to an
  answer read as last fetched; the publish group holds the refusal
  from a copy, and the status group the landing read from one.
- `channel-status` (made by `status-on-any-machine`, ahead of this
  unit in the bolt): status says how the landing of the branch's head
  was read.

## Impact

- `crates/tailrace-seams`: `landing.rs` (the answers' types and the
  contract), `answer.rs` (how an answer was read, moved from
  `keeper.rs`, and the new refusal), `keeper.rs`, `lib.rs`.
- `crates/tailrace/src/profiles/landing`: both profiles' reads, the
  copy and its time.
- `crates/tailrace-core`: `landed.rs` (the landed revision carries how
  it was read, and the refusal from a copy), `publish.rs` and
  `status.rs`; every other caller of the landing reads through
  `Landed::check` and is unchanged.
- `crates/tailrace`: status's text and the host's page render the
  landing's read; the UTC rendering moves to the seams crate.
- `crates/tailrace-fakes`, `crates/tailrace-conformance`: the fake
  landing, the landing group, the fake rig and publish's group.
- Tests across the crates that call the landing or match its answers.
- The landings page of the site.
- No crate, dependency, language or file type is added.

## Touches

```
crates/tailrace-seams/src/landing.rs
crates/tailrace-seams/src/answer.rs
crates/tailrace-seams/src/keeper.rs
crates/tailrace-seams/src/lib.rs
crates/tailrace-seams/src/bind.rs
crates/tailrace/src/profiles/landing/**
crates/tailrace/src/commands/status.rs
crates/tailrace/src/commands/serve/page.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/src/render.rs
crates/tailrace/tests/**
crates/tailrace-core/src/landed.rs
crates/tailrace-core/src/publish.rs
crates/tailrace-core/src/status.rs
crates/tailrace-core/src/cache.rs
crates/tailrace-core/src/push.rs
crates/tailrace-core/src/record.rs
crates/tailrace-core/tests/**
crates/tailrace-fakes/src/landing.rs
crates/tailrace-fakes/src/lib.rs
crates/tailrace-conformance/src/seams/landing.rs
crates/tailrace-conformance/src/seams/host.rs
crates/tailrace-conformance/src/fakes.rs
crates/tailrace-conformance/src/core/publish.rs
crates/tailrace-conformance/src/core/status.rs
crates/tailrace-conformance/src/core/cache.rs
crates/tailrace-conformance/src/core/gate.rs
crates/tailrace-conformance/src/core/verify.rs
crates/tailrace-conformance/tests/**
site/src/content/docs/guides/integrations/landings.mdx
```
