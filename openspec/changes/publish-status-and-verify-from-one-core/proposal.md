# Proposal

## Why

The seams and their fakes are in place and a landed revision can take a
version; nothing yet turns that version into artifacts in a store, says
who signed what is attached to them, or answers what this host has
published. This unit is the rest of bolt 1's core: `tailrace publish`,
`tailrace verify` and `tailrace status` as one core over the seams,
reached first from the command line, so every later bolt binds real
profiles to a core that already keeps Part A (14 to 20, 56 to 65;
ADR 0002, 0004, 0005, 0007; phase-1-bolts.md, Bolt 1).

## What Changes

- **Publish refuses what has not landed, first.** A publish asks the
  landing seam whether the branch has landed the revision before it
  reads anything else; a revision not landed is refused naming the
  branch, and nothing is read or offered (14, core.11).
- **Every output is ensured and judged before anything is offered.**
  For each component it publishes, a publish asks the build seam to
  ensure every output the component declares in one asking (74), reads
  each output's build identity and outcome (75), and reads the
  landing's verdicts for the revision (78). An output the build could
  not make, or whose outcome is not a pass, refuses the version whole
  naming it (16, 111). Where the landing carries verdicts, an output
  whose build identity no verdict names is refused naming the output
  and the identities the verdicts carry (15). Only then is the
  version's name claimed, so a refused version claims no name (22,
  core.12, core.13).
- **Each artifact is packed, put and attested; the index comes last.**
  Each output is packed by the projection the policy file at the landed
  revision names for it in its component (C.10),
  with the version, and its bytes and metadata are put in the store; a
  store that answers a digest other than the bytes' own is refused
  naming both (2, 84, core.5). Each artifact gets one provenance claim,
  signed in the host's own identity, naming the revision, the output's
  build identity and the build's outcome, and carrying, as the landing
  made them, the verdicts that name that build identity, with where
  they were found (11, 12, 13, core.8, core.9). The claim is attached
  beside the artifact's digest (8, core.27). The index is put after
  every artifact and its claims, and the version's name is then moved
  in the store from nothing to the index (6, 17, core.12, core.6).
- **A second publish of a revision ends where the first did.** A
  publish of a revision whose version's name already names an index
  compares each packed artifact with the one the index names; all the
  same, nothing is written and the answer names the same index (5, 18,
  S2); one different, the publish is refused naming the artifact and
  both digests, nothing is offered, and the name stays with the first
  index (18).
- **The answer a program reads.** A publish ends with an answer that
  names, per component, the index's digest, the version, each
  artifact's digest, ecosystem and name, and each claim with the digest
  it is attached to and its signer (19, 57). A publish never accepts,
  promotes or moves a channel (20, core.10).
- **This host keeps which versions it published.** After the version's
  name is moved, the publish writes one file per version to the host's
  record: the component, the version, the revision, the branch, the
  index's digest and the store it went to, and nothing the store or git
  holds besides (61, 62). A repeat publish writes nothing.
- **Verify says who made each attestation.** `tailrace verify` resolves
  a version's name to its index's digest (core.4), reads every claim
  attached to the index and each artifact, and asks the signer seam
  which signer the policy names made each one, and each verdict a
  provenance carries (80). A signer whose identity has expired is
  reported as made by that signer and expired, never as valid and never
  as absent (81, 117). A claim no named signer made, or beyond what its
  signer may claim, is listed, kept, and never counted (9, 116,
  core.7). Verify writes nothing.
- **Status answers from what this host keeps.** `tailrace status`
  reads the host's record alone: which versions of which components
  this host published from which revisions, at which index. It reaches
  no store, signer or target and writes nothing (60, 61, core.32).
  Nothing kept is an absence; a record that cannot be parsed is
  unreadable naming the file (58).
- **The command line is the first surface.** `tailrace publish`,
  `tailrace verify` and `tailrace status` parse their arguments, call
  the one core and render its answer, as text or, with `--json`, as the
  answer itself; none computes anything of its own (56, 57, core.31).
  Absent, refused and unreadable are three answers with three exit
  codes, and a refusal prints the source's words beside tailrace's (58,
  59, core.19).
- **The site stops saying publish is the gate's move.** The paragraph
  of the site's how-it-works page that says `tailrace publish` verifies,
  copies and moves `beta` is rewritten to say what publish and verify do
  (20, 25).

Out of scope, and what each waits on:

- The conformance suite's scenarios, S1, S2, S5 and S13 among them,
  over the fakes: `the-conformance-suite-over-the-fakes`.
- Binding the commands to real profiles through the host's bindings
  file (92, 118): bolt 2's laptop unit. Until then the installed binary
  answers `publish` and `verify` with an absence naming the unbound
  seams, and `status` reads the host's record.
- Rebuilding the record from the store and git (63, S12) and the record
  of a push after a landing: bolt 3.
- The gate's verdict, acceptance, channels and the ledger (25 to 39):
  bolt 4. Verify reports signers; it reaches no verdict.
- Status of channels and environments (98, S10, S11): bolt 4.

## Capabilities

### New Capabilities

- `publishing`: one landed revision turned into a version per
  component: the landing first, every output ensured and judged, each
  artifact packed, put and attested, the index last, the repeat ending
  where the first did, and the answer a program reads.
- `attestation-verification`: which signer the policy names made each
  attestation on a version, expired signers reported as such, and
  claims from no named signer kept and never counted.
- `host-record`: what this host keeps of the versions it published, and
  status answered from it alone.
- `command-line`: the command line as the first surface, calling the
  one core for publish, verify and status and rendering its answer.

### Modified Capabilities

None. The projection each output is packed by, which the policy file
names per component (C.10), is a requirement of `publishing`, read
beside the components `a-landed-revision-takes-its-version` reads.

## Impact

- `crates/tailrace-core/`: modules for publish, the provenance claim,
  verify and the host's record; the components read with a projection
  per output; the history's common directory and a branch's head.
- `crates/tailrace-seams/src/answer.rs`: the refusals this unit makes
  and the host as a source; `claim.rs` gains the provenance kind where
  the seams do not hold it.
- `crates/tailrace/`: the command line, its three commands and their
  rendering, with tests over the fakes and of the built binary.
- `site/src/content/docs/how-it-works.mdx`: the publish and verify
  paragraph.
- No new dependency: `clap`, `serde_json` and the fakes are in the
  workspace already.

## Touches

```
Cargo.lock
crates/tailrace-core/**
crates/tailrace-seams/src/answer.rs
crates/tailrace-seams/src/claim.rs
crates/tailrace/**
site/src/content/docs/how-it-works.mdx
```
