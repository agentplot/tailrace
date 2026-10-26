## Context

`version::derive` (`crates/tailrace-core/src/version.rs`) finds a
component's last release tag and, for a revision no tag names yet, the
commits since it that touch the component's paths, through
`History::commits`, which is git-cliff-core's own commit walk with its
path filter; it then bumps with git-cliff's `Changelog::bump_version`.
It keeps neither the commits nor anything rendered from them, and for a
revision a tag already names it reads no commits at all.
`publish::Run::component` claims the name, packs each output, and on a
first publish `offer`s each artifact (bytes, manifest, provenance
claim), then puts the index, attaches the layout-policy claim and moves
the name; on a repeat, `again` compares the packs with what the index
names. `published::Entry` is the host's record of a version; `status`
reads it and nothing else; `record::rebuild` rebuilds it from the tags
and the store; `verify` resolves a version's name and tells every claim
on the index and its artifacts.

The statement is `agentplot/blueprints:design/tailrace/requirements.md`
at 80ab275: 134 (a version carries its notes), with 6, 17, 18, 21, 22,
58, 60, 61, 63, 132 and section 9 (the version rule is git-cliff). The
conformance suite's statement list is read at 5b00e15, whose clauses end
at 125, so no scenario may cite 134 yet.

Bolt tailrace-the-gate-and-the-channels is not on this bolt. Its gate
(`gate.rs` there) judges every artifact an index names against every
required claim, and its status nests this bolt's record part, `Status`,
inside a wider `Overview` that the host's web page renders.

## Goals / Non-Goals

**Goals:** notes rendered in the version run from the version's own
commits; carried as an artifact the index names, put and claimed before
the index; compared, never rewritten, on a repeat; shown by publish,
status and verify; kept in the record only as the commits they came
from, and rebuilt.

**Non-Goals:** the notes between two versions and a channel's changelog;
the web page and the pane; the gate's judgment of the notes; a template
of the authors' choosing; a notes artifact for a version published
before this change.

## Decisions

### D1. The run that derives the version keeps its commits

`Versioned` gains `since: Vec<Commit>`, the commits the version is
derived from: those `History::commits` answers from the last release tag
to the landed revision under the component's paths. `derive` reads them
on both of its paths, so a revision a tag already names (a repeat, or a
rebuild) has the same commits its first publish had; the held path
reads them after it has found `last`, which it already computes the same
way. `touched` and `claim` are unchanged.

Alternative: render the notes in `publish` from a second commit walk.
Rejected: 134 says the run that derives the version renders them, and a
second walk could disagree with the one that chose the bump.

### D2. One fixed form, rendered by git-cliff

A new module, `crates/tailrace-core/src/notes.rs`, holds
`render(version, last, commits) -> Answer<Rendered>`, where `Rendered`
holds the text and its bytes. It builds one git-cliff `Release` whose
version is the bare version, whose previous release is `last`'s version
and whose commits are `since`, and renders it with git-cliff-core's
`Changelog::generate` under a `Config` the module fixes:

- `conventional_commits = true`, `filter_unconventional = false`,
  `filter_commits = false`, so every commit is kept;
- `commit_parsers` grouping `feat` as Features, `fix` as Fixes, and
  every other commit, conventional or not, as Other, with a breaking
  change marked `(breaking)` on its line;
- a body template written as a constant beside it: `## <version>`, a
  line `Since <last tag>.` where there is one, then each group that has
  commits as `### <group>` and one line per commit, `- <subject>
  (<seven-character revision>)`, in the order the history gives them;
- no header, no footer, `trim = true`, `format = false`, offline, and no
  timestamp or other release field in the template.

Nothing in it reads the clock, the host or the checkout's path, so the
bytes depend on the version, `last` and the commits alone (18, 21).
A template error is unreadable naming the component, with git-cliff's
words (58, 59).

Alternative: git-cliff's own default template. Rejected: it prints the
release's date, which would make two publishes of one revision differ.
Alternative: a template the policy file names. Not taken: the design
names none (named in the final message for the design agent).

### D3. The notes are an artifact of the reserved ecosystem `notes`

The core names the ecosystem `notes` in one constant, `NOTES`. The notes'
artifact metadata is `ArtifactMeta::new(NOTES, <component name>,
<version>)`, its one layer the rendered bytes; the core packs it itself,
since the notes are no output and have no projection (84 packs
outputs). The index names it after the outputs' artifacts. Because it
is an ordinary artifact of the index, `copy` carries it and its claim to
another store unchanged (103), and `render`'s `locate`, which finds an
artifact by ecosystem and name, never meets it, since no projection may
be bound under `notes` (D9).

Alternative: a field of its own in `IndexMeta`. Rejected: 134 makes the
notes an artifact named in the index and 28 judges every digest it
names; a separate field would also change the seam type and the OCI
encoding for no gain. The store's encoding (`profiles/store/oci.rs`)
already writes an artifact of any ecosystem whose name a media type can
carry, `notes` among them, so no profile changes.

### D4. The notes' claim is the publisher's provenance, naming no build

The notes are attached one claim of kind `Provenance`, signed as the
party running the publish, about the notes' digest, naming the revision
and `build: None` (core.9: a build identity where it has one), whose
body is `{"notes": {"component", "version", "last", "commits": [...]}}`,
the commits by their full revisions. `Provenance::of` reads only an
output's provenance body, so it answers `None` for this one, and nothing
that counts a carried verdict reads it.

Alternative: a new claim kind, `notes`. Not taken: the policy vocabulary
(116) names the kinds a signer may claim, and a new kind would leave
every policy that requires a provenance unable to count the notes.
Kind `Provenance` says what the statement's glossary says of it, that
the party produced it from a revision, and the notes count wherever the
publisher's provenance does. What acceptance requires of the notes is
the gate's, on the gate-and-channels bolt (named in the final message).

### D5. Publish: rendered with the version, put before the index

In `Run::component`, the notes are rendered from the `Versioned` right
after the name is claimed and before the outputs are packed, so nothing
of them exists before the claim (22, core.13). On a first publish,
`offer` puts the notes' bytes and their artifact after the outputs',
checks the bytes' digest as it does an output's (2, core.5), then signs
and attaches the notes' claim after the outputs' provenance claims; the
index names the notes last among its artifacts; then the layout-policy
claim and the name, as today (core.12).

On a repeat, `again` finds the artifact of ecosystem `notes` and the
component's name among those the index names. Found, its one layer must
be `Digest::of` the rendered bytes and its metadata the notes' metadata;
otherwise the publish is refused `Why::Differed` naming the notes'
metadata, the layer the index names and the digest rendered now (18),
before anything is put. Not found, the index predates notes: nothing is
added and the answer names no notes (134). The answer's notes claim on a
repeat is read from the store as `again` reads the provenance claims:
the claims of kind provenance this signer made about the notes' digest.

`Published::Version` gains `notes: Option<PublishedNotes>`, where
`PublishedNotes { digest, bytes, text, claims: Vec<Attached> }`, skipped
in JSON when none. `tailrace publish`'s text answer prints the notes'
line, `notes <digest>`, its claim beneath it as an artifact's are, then
the text indented four spaces.

### D6. The record keeps the commits, never the text

`Entry` gains `notes: Option<Noted>`, `Noted { last: Option<String>,
commits: Vec<Revision> }`, with `#[serde(default, skip_serializing_if =
"Option::is_none")]`, so an entry written before this change reads back
as it was and a repeat publish of an index without notes writes the
same entry, answered held. A publish writes `Some` exactly when the
index names notes. The record keeps neither the notes' text nor their
digest: both are the store's (61, and the host-record requirement that
it keeps no artifact digest).

`last` is the last release tag's name as the history spells it, the
same string the notes render after `Since`.

### D7. Status renders the notes again from git

`status` becomes `status(history, kept, ask)`. Each version it answers
is `Shown { #[serde(flatten)] entry: Entry, notes: Option<Answer<String>> }`,
the notes serialized in the one answer form (`answer_json`) where
`Some`. For an entry with `notes: Some(noted)`, status reads each
commit's message from the local history by its revision, a new
`History::message(&Revision) -> Answer<String>` that opens nothing
remote, and renders them with `notes::render(entry.version, noted.last,
commits)`, the same function and form publish used (D2). A revision the
history does not hold answers that version's notes unreadable naming it;
the version itself and every other version still answer (58, core.19).
Status reaches no store, signer or target, and writes nothing (60,
core.32); it reads only what the record and git hold, as 132 asks of
the page.

`tailrace status` already opens the history; it now passes it in. Its
text answer prints the notes indented four spaces beneath the version's
line, or the unreadable answer's words in their place.

Alternative: keep the notes' text in the record. Rejected: 61 keeps
nothing the store holds already, and the text is the store's; the
statement's one exception, the chain, is named there and the notes are
not. Alternative: read the notes from the store in status. Rejected:
60. Alternative: derive the commits again in status. Rejected: that
needs the component's paths from the policy at a landed revision, which
status, handed no binding, cannot ask the landing for.

### D8. A rebuild derives the commits again

In `record::rebuild`, for a tag whose index names an artifact of
ecosystem `notes`, the commits come from `derive` at the tag's revision:
`Landed::check` for that revision (the rebuild already asks the landing
of each tag), `Declared::read` at it, the component by name, then
`derive`, whose held path now answers `since` (D1). Its `last` and
`since` become the entry's `Noted`, so a deleted record is rebuilt byte
for byte (63, S12). A tag whose index names no notes gets `notes: None`.
The store is asked only to resolve and read, as today.

### D9. No output is packed as `notes`

`Declared::read` refuses a file in which any component's projection
table names the ecosystem `notes`, with `Not::refused(.., Why::Said,
Some(words))`, the words naming the component, the output and that
`notes` is the ecosystem of a version's notes. It is read before any
build is asked, as every policy-file refusal is.

### D10. Verify shows the notes from the store

`Verified` gains `notes: Option<Answer<VerifiedNotes>>`,
`VerifiedNotes { digest, text }`: where the index names an artifact of
ecosystem `notes`, verify reads its manifest and its one layer through
`store.read`, and answers the text, or unreadable naming the digest when
the bytes are not UTF-8. Its claims are already told by the loop over
the index's artifacts. `tailrace verify`'s text answer prints the notes
after the claims, indented two spaces under a `notes <digest>` line.

### D11. The conformance suite holds the notes without citing 134

The publish group's S1 check reads the index's artifact of ecosystem
`notes`, its bytes, and its one provenance claim with no build; S2
asserts the notes unchanged; the status group asserts status's notes
equal the stored bytes and the entry names the commits. They cite the
clauses they exercise that the list holds: 6, 17 and 21 in S1, 18 in
S2, 60 and 61 in status. The commit adopting the statement past 5b00e15
adds 134.

## Risks / Trade-offs

- [Status renders what the publish rendered only while the form and
  git-cliff stay the same] → One function and one constant template serve
  both; a core test and the status group assert status's notes equal the
  stored bytes; a change to the template or to git-cliff that alters the
  bytes shows in those tests, and is a change to make with the design
  agent, since an upgraded host would show other notes than the version
  carries.
- [A repeat publish on a host whose tailrace renders differently is
  refused] → Intended (18, 134): the notes are never rewritten; the
  refusal names both digests.
- [The gate on the gate-and-channels bolt judges the notes against every
  required claim, and finds a required verdict wanting] → Named as a
  design question; the notes carry the publisher's provenance so a policy
  requiring only a provenance counts them.
- [The unit naming every artifact `artifact/<ecosystem>/<name>@<version>`
  will name the notes too] → Harmless under 138; named for the planner.
- [A first release's notes hold the component's whole history] →
  Accepted: they are the commits the version is derived from.

## Migration Plan

Nothing to migrate. Entries and indexes written before this change read
as they were and carry no notes; a repeat publish of such a version
writes nothing. The next new version of each component carries notes.
