# Design

## Context

`status::channels` (crates/tailrace-core/src/status.rs) answers each
declared channel as `ChannelStatus { declared, at: Answer<Channel> }`,
`Channel::At { head, chain: Vec<(Digest, Promotion)>, read }` from the
ledger; every `Promotion` already holds `to: Moved { version, index }`
and `asked_by`. The command line (`crates/tailrace/src/commands/status.rs`,
`promoted`) and the page (`commands/serve/page.rs`, `promoted`) print
each promotion with its asker. So the changelog lacks only the notes.

A version's notes are rendered by `notes::render(history, component,
version, tag, revision, noted)`, where `Noted { last, commits }` names the
last release tag and the commits the version was derived from. Status
gets a `Noted` today from the host's record alone (the archived
a-version-carries-its-notes, its D7), so only versions this host
published show notes. A rebuild recovers a `Noted` without the record
(its D8): `record::noted` checks the tag's revision landed, reads the
policy file there with `Declared::read`, and runs `version::derive`,
whose `last` and `since` are the commits the publish rendered from.

That D7 rejected deriving the commits again in status because status
had no binding to ask the landing. `status::channels` now runs over
`Seams` and already asks the landing of the branch's head
(`Landed::check`), so the rebuild's way is open to it.

Status must reach no store, signer or target and write nothing (60,
core.32); reading the landing, the policy, the ledger and git is what it
already does.

## Goals / Non-Goals

**Goals:** each promotion of each chain shown with its asker and the
notes of the version it moved to, on the command line, in its JSON and
on the page, the same on every machine; the page also showing the
notes of the versions this host published, which the answer carries
already.

**Non-Goals:** the notes between what an environment runs and what it
should; the herdr pane; an asker named by an identity provider (129);
reading whether an index names notes from the store.

## Decisions

### D1. The notes are rendered again from git, by the rebuild's derivation

For each distinct version a chain moved its channel to, status finds
the tag `<prefix><version>` among `history.tags(&landed_head, prefix)`,
checks its revision landed on the asked branch (`Landed::check`), and
derives its `Noted` there exactly as a rebuild does, then renders with
`notes::render` under the template that revision holds. The derivation
moves out of `record.rs` into `notes.rs` as one function,
`pub(crate) fn noted(history, at: &Landed, component: &ComponentName,
tag: &Tag) -> Answer<Noted>`, which `record::rebuild` and status both
call; its behaviour is unchanged. A private function in `status.rs`,
`tagged(landing: &dyn Landing, history, head: &Landed, component:
&Component, version: &Version) -> Answer<String>`, does the whole of one
version's notes for status: tag, landed, noted, render.

The component's prefix comes from `Declared::read(history,
&landed_head)`, read once per status ask; a component the head's policy
file no longer declares, or a file that cannot be read, answers every
one of its versions' notes with that answer.

Alternative: read the `notes` artifact from the store, the bytes
themselves. Rejected: 60 and core.32, status asks no store. Alternative:
take the `Noted` from the host's record where it holds the version, and
derive only otherwise. Rejected: two hosts would read one chain through
two paths, and 36 asks every reader one answer; the derivation is the
one a rebuild already trusts to give the record's bytes back (63).

### D2. The notes ride beside the chain, keyed by version

`ChannelStatus` gains `notes: Vec<Logged>`, `Logged { version: Version,
index: Digest, notes: Answer<String> }`, the answer serialized with
`answer_json` (57). One entry per distinct `(version, index)` the chain
moved to, in the order the chain first reaches it, oldest first as
`chain` is held. Empty where the channel is empty or `at` is not `Ok`.
`Channel` and the ledger's answer are unchanged: `promote` and `repair`
answer them too, and they hold no history.

Within one status ask, a version's notes are rendered once and shared by
every channel of its component that moved to it (`beta` and `stable`
usually share most), through a map keyed by component and version that
lives for the call.

Alternative: put the notes into each chain element. Rejected: it changes
the ledger's `Channel`, which other commands answer, and renders a
version moved to twice twice over.

### D3. Each version's notes are their own answer

The notes' failures stay in that version's `Logged.notes`, never in the
channel's `at` (58, core.19):

- the tag not among those the landed head reaches: unreadable, source
  the local history, naming `the notes of <component> <version>` and
  saying `the history holds no tag <tag> reached from <head>`;
- the tag's revision not landed, or the landing unreadable: the
  landing's answer as `Landed::check` gives it;
- `derive` answering `Unchanged`, a commit the history no longer holds,
  or git-cliff's words on the template: as `noted` and `render` answer
  them today.

Never absent: the version is in the chain, so its notes exist in the
store; only this host's history cannot render them.

### D4. The surfaces render the notes beneath each promotion

On the command line, `promoted` prints the promotion's line as now, then
the `Logged` whose version and index the promotion moved to: its text
indented two spaces past the chain's indent, trailing spaces trimmed as
the record's notes are printed, or `render::not` at that indent. The
page's `promoted` gains a `<pre>` holding the escaped text after the
promotion's words inside its `<li>`, or the `not` HTML in its place; the
record's table gains a row beneath each version whose `Shown.notes` is
`Some`, one cell spanning the table holding the notes the same way.
Neither computes anything: they look up the version in the answer (56,
core.31).

### D5. The register's part for 132's notes leaves with the scenario

The conformance surfaces group gains, below its marker, "the command
line and the page answer each promotion's notes", citing 56, 57 and 132:
two versions published and promoted, the channels asked through each
surface, the answer equal to the core's, and each version's notes equal
to the bytes its index's `notes` artifact holds in the fake store. The
`Part { clause: 132, part: "each version's notes", .. }` entry and the
`NOTES` unit constant leave `register.rs` in the same commit (core.33).
No scenario cites 134: `statement.rs` lists clauses to 132.

## Risks / Trade-offs

- [A version published before notes were carried shows notes its index
  does not name] → status cannot tell without reading the store (60);
  every version the bolt's publishes make carries notes, so this touches
  only versions published before a-version-carries-its-notes. Named for
  the design agent: whether the statement wants such a version's notes
  answered absent, and how a status that reads no store would know.
- [A landing read per distinct version] → bounded by the versions a
  channel ever held, once per ask; the same reads a rebuild makes. A
  landing answered as last fetched still gives the notes, and the
  landed head check already says when.
- [Long chains print long text] → status already prints every promotion;
  the notes are the changelog the statement asks for. `--component`
  narrows it as today.
- [The prefix read from the head's policy file, the derivation from the
  tag's] → a component renamed since a version was published finds no
  tag under the new prefix, and answers that version's notes unreadable
  naming the tag; no notes are guessed.
