# Design

## Context

See proposal.md, Why. The design rests on
`agentplot/blueprints:design/tailrace/requirements.md` as it stands at
b64d367: 77 (as amended at 061bfa8), 78, C.5 and C.6 (as amended
there), 59, 60, 61, 62, 63, 130, and C.13 for the ledger's own copy,
whose form this unit follows.

Today:

- `Landing::landed` answers `Answer<bool>` and `verdicts_of`
  `Answer<Vec<Verdict>>` (`crates/tailrace-seams/src/landing.rs`).
- `PlainBranch` and `Sluice` each open every operation with
  `git ls-remote` of the bound remote and fetch with `--refmap=` and
  `--no-write-fetch-head`, so no ref is written (the sluice landing's
  D2); a remote that does not answer makes every operation unreadable
  with git's words.
- The ledger already answers from a copy: `Keeper::chain` answers a
  `Chain { claims, read: Read }`, where `Read` is `Source` or
  `Kept { at, said }` (`keeper.rs`), and the git ref ledger keeps
  `refs/tailrace/fetched/<channel>`, deleted and fetched anew at every
  read so its reflog's one entry says when the remote was last read.
- Every command reaches the landing through `Landed::check`
  (`crates/tailrace-core/src/landed.rs`), which refuses `NotLanded`;
  only publish (verdicts) and the profiles' own tests call the seam
  directly.
- git writes no reflog entry for an update that leaves a ref at the
  object it names (checked with git 2.54, the tree's pin): a copy's
  time can only be kept by remaking the ref.

## Goals / Non-Goals

**Goals:**

- One answer type per landing operation that says how it was read,
  the same `Read` the ledger's chain carries.
- A copy of what each landing profile reads from the remote that is
  honest: an answer from it is what the remote said at the copy's
  time, and where the copy cannot say, the answer is unreadable.
- A no from a copy told apart from a no from the remote, everywhere a
  command asks whether a revision landed.

**Non-Goals:**

- Retrying anything. Nothing in tailrace asks again by itself (130);
  the refusal says to ask again, and whatever runs the publish does.
- Reading the person's own remote-tracking refs (D2).
- Any change to how the ledger reads or keeps its chain.

## Decisions

### D1. `Read` crosses every seam that may answer from a copy

`Read` moves from `keeper.rs` to `answer.rs`, its form and JSON
unchanged (`"source"`, or `{"kept": {"at", "said"}}`), its doc saying
how any answer was read: from the source, or as the host last read or
fetched it, at `at` seconds since the epoch, for `said`, the source's
own words. `keeper.rs` uses it as before, and `lib.rs`'s `pub use`
keeps every path.

The landing's operations answer:

```rust
pub struct IsLanded { pub landed: bool, pub read: Read }
pub struct Verdicts { pub verdicts: Vec<Verdict>, pub read: Read }

fn landed(&self, branch: &Branch, revision: &Revision) -> Answer<IsLanded>;
fn verdicts_of(&self, revision: &Revision) -> Answer<Verdicts>;
```

A landing with no remote bound reads the repository, or the host's own
sluice files, which are the landing itself: `Read::Source`.

Alternatives: a generic `AsRead<T>` (one more pattern beside `Chain`,
which the keeper already set); a second operation, `landed_as_fetched`
(a caller could skip it, and a seam is one interface, core.2);
`Read` left in `keeper.rs` and imported by `landing.rs` (it is no
longer the keeper's alone).

### D2. The copy is tailrace's own refs, remade at each fetch

The copy lives under `refs/tailrace/landing/`, the remote's ref name
with its `refs/` prefix dropped: `refs/tailrace/landing/heads/<branch>`,
`refs/tailrace/landing/sluice/state/<branch>`,
`refs/tailrace/landing/sluice/runs/<signer>`. The fetch itself keeps
`--refmap=` and `--no-write-fetch-head`, so no remote-tracking ref or
`FETCH_HEAD` moves; after it answers, each copy ref is deleted and made
again at the object the remote's listing named, with
`core.logAllRefUpdates=always` and `--create-reflog`, so its reflog's
one entry is the fetch's time, read back with
`git reflog show --date=unix --format=%gd -n 1` as the git ref
ledger's `when` reads its own. The copy is the repository's git refs:
readable with git and no tailrace process (62), and rebuilt by the next
read that reaches the remote (63).

Alternatives: git's own remote-tracking refs, which a person's
`git fetch` also moves (a bare host logs no reflog by default, sluice's
refs have none, a configured refspec may map elsewhere, and a reflog
there says when the ref changed, not when it was fetched); a file of
fetch times beside the bindings (a second record to keep in step with
the refs, and a record no git reader finds); the copy named under the
remote's name (a bound remote may be a URL, which no ref name holds).

### D3. What each profile copies, so the copy never guesses

- **The plain branch** copies each branch an operation fetches, as it
  fetches it: a copy of a branch is that branch as the remote held it.
  A branch with no copy, or one the remote did not list when last read,
  cannot be told apart from one never fetched, so from the copy it is
  unreadable, never not landed.
- **sluice** copies, at every opening read the remote answers, every
  ref its listing names under `refs/sluice/state/` and
  `refs/sluice/runs/`, fetched in one `git fetch`, the old copies
  deleted in one `update-ref --stdin` transaction and the new made in a
  second, so the copy is the listing whole as of one fetch: a signer's
  runs ref missing from it was missing from the remote then, and an
  empty set of verdicts read from it is the remote's answer, not a
  guess. A crash between the two transactions leaves no copy, and the
  next read with the remote down is unreadable. This replaces the
  sluice landing's selective fetch (its D2), at the cost of fetching
  every signer's record at each open; a fetch of refs already held is
  a negotiation and no transfer.

### D4. Where the remote does not answer

"Does not answer" is the remote's listing, or a fetch from it,
answering unreadable; any other failure, the repository itself
unreadable or a record malformed, answers as today. The profile then
reads its listing from the copy (`git for-each-ref` under
`refs/tailrace/landing/`, each name mapped back to the remote's), makes
no fetch, and runs the same reads over it, every object already held
from the fetch that made the copy. The answer carries
`Read::Kept { at, said }`: `said` the failure's own words, `at` the
oldest fetch time among the copy refs the answer read (for sluice, all
one fetch), and for the plain branch's verdicts-of, which reads no
branch, the newest copy's, the last time the remote answered. Where
the copy holds nothing the operation needs, the failure itself is the
answer, as `GitRef::kept` returns it.

For sluice, a branch whose log the remote's listing does not hold is
read from the host's own files today (sluice C.6); from the copy, a
missing log copy is unreadable, since the copy cannot say whether the
remote held one.

### D5. The core: a landed revision says how it was read

`Landed` gains `read: Read` and `Landed::read()`. `Landed::check`:

- `IsLanded { landed: true, read }` → `Landed { branch, revision, read }`.
  A branch only moves forward, so a revision it held when fetched it
  holds now; every caller goes on as for a yes from the remote.
- `IsLanded { landed: false, read: Read::Source }` → refused
  `Why::NotLanded`, as today.
- `IsLanded { landed: false, read: Read::Kept { at, said } }` → refused
  `Why::NotLandedAsFetched { branch, revision, fetched: at }`, with
  `said: Some(said)`, the remote's words beside tailrace's (59). Its
  text: `<branch> had not landed <revision> as last fetched <UTC time>;
  ask again once the landing's remote answers`.

Every command that asks the landing does so through `Landed::check`,
so publish, verify, the gate, promote, push, the cache, the record and
status all refuse the same way; only publish's `Run` reads
`verdicts_of`, and takes `.verdicts` from its answer, so verdicts read
from a copy count as those read from the remote, being the landing's
signed records either way (core.7). A refusal from a copy exits as any
refusal does; a program reads `refused.why.not_landed_as_fetched` in
the JSON to know it may ask again.

The UTC rendering `render::utc` holds moves to `tailrace-seams` beside
`Read`, so `Why`'s text names the time as a person reads it;
`render::utc` calls it.

### D6. Status: the channels' part says how the landing was read

`Channels` gains `landing: Read`, the read of the landed head the
channels were read at. The command line's text prints it under the
branch and revision, `landing: as last fetched <UTC>` and the words
indented beneath, as the ledger's kept chain is printed, and nothing
where it was read from the landing itself; `--json` carries it in the
one form; the repository's page renders the same. A head refused as
not landed from a copy is the channels' part's refusal, the record's
part answering, as for any refusal there today. A repository's row is
unchanged: it shows the heads, and the page beneath it the landing.

### D7. The fake landing and the suite

`FakeLanding` gains `unreachable(words)`, as `FakeKeeper` has. Each
answer it gives while reachable copies its whole state with the time
of the answer; once unreachable, every question is answered from that
copy with `Read::Kept`, and before any answer, unreadable with the
words. Scripted faults keep working as now.

`LandingRig` gains `reads_a_remote(&self) -> bool`, and `unreachable`'s
doc says a landing read before answers from its copy. The landing
group gains `a landing read as last fetched` (cites 59, 61, 77, 78),
not applicable where the rig reads no remote: it lands a revision,
reads both operations, lands a second, makes the landing unreachable,
and asks again, the first answered as before from the copy at a time no
earlier than the first read with the rig's words, the second not landed
from the copy. `a landing that cannot be reached` is unchanged: a
landing never read answers unreadable. `landing_branch.rs` runs the
group a second time over a branch bound to a scratch remote, the rig
pushing what it lands and removing the remote to cut it off; the
sluice rig reads a remote already. The fake rig in
`tailrace-conformance/src/fakes.rs` reads a remote and uses
`FakeLanding::unreachable`.

The publish group gains the refusal from a copy and the publish from a
copy (cites 14, 59, 77); the status group gains the landing read as
last fetched (cites 36, 60, 77). Every clause cited is cited already,
so the register and the statement's list are unchanged.

### D8. The site

The landings page says that with a remote bound the landing is read on
the remote and, where it does not answer, from the host's copy as last
fetched, saying when; that a no from the copy is refused saying to ask
again; and where the copy lives.

## Risks / Trade-offs

- [Rule core.32 says a read writes nothing, and 60 names only the
  ledger's fetch as the write a status makes] → the landing's copy is
  the same kind of write 77 needs, the remote's refs fetched into the
  host's copy and nothing else, and today's fetch already writes
  objects. The gap goes to the design agent; nothing here writes a
  store's name or bytes.
- [A copy fetched from one remote answers after the binding names
  another] → the copy is named without the remote (D2); an answer from
  it says when it was fetched and the new remote's words, never that
  the new remote said it. Rare, since a host's landing remote is fixed
  by its bindings or the checkout's `origin`.
- [sluice's open fetches every signer's records] → each fetch after the
  first moves only what changed; status already fetched the branch.
- [A crash between deleting and remaking the copy] → the copy is gone,
  not wrong; the next read that reaches the remote makes it again.
- [Changing the seam's answer types touches every test that calls the
  landing] → the change is mechanical, `.landed` and `.verdicts` taken
  from the answer; the Touches block names every file.

## Migration Plan

None: the copy is made by the first read after this lands, and a host
with no copy answers with the remote down as it does today.
