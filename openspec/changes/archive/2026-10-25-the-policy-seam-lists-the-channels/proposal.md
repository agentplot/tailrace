# Proposal

## Why

Nothing can yet ask which channels a repository has. The policy seam
answers what one audience requires (82), so a reader that already knows
a channel's name can read its policy, but `tailrace status` cannot list
the channels it should show, and nothing says which component a channel
belongs to (125) except the gate's own reading of the file (the gate's
design D6). The statement now gives the policy seam the question (131,
agentplot/blueprints 673d334). It is a change to a seam trait, so it is
a unit of its own, landed before the units that read it
(phase-1-bolts.md, How the units stay apart): `status-on-any-machine`
and `the-ledger-and-promotion`.

## What Changes

- **The policy seam lists the channels.** Given a repository at a landed
  revision, the policy answers the channels its policy declares, each
  with the component it belongs to (125) and the channel it is bound to
  where it is one (37, 83), ordered by name. A repository that declares
  one component has every channel in it, the one it declares or, where
  it declares none, the component named `repository` (123, 131). A
  policy that declares no channel answers an empty list, never absent.
- **The policy file answers it.** `[channels.<name>]` names its
  component with `component = "<name>"`, the key the gate already reads,
  and may leave it out where the repository declares one component. A
  channel that names no component where several are declared, names one
  the file does not declare, or is bound to a channel the file does not
  declare or that belongs to another component, is unreadable naming
  its table (125). The same reading answers the policy for that channel,
  so the listing and policy-for never disagree on a binding (83).
- **The fake policy answers it** from what a test scripts: every channel
  it holds a policy for, in the component the test puts it in, else
  `repository`, bound as its policy says.
- **The suite holds every policy profile to it.** Four scenarios below
  the policy group's marker: the channels of two components with a
  binding, one component holding every channel, a policy declaring no
  channel, and a policy that cannot be reached. The rig gains a way to
  declare components and channels.
- **The suite adopts the statement at 673d334.** Its list of clauses
  stops at 125 (5b00e15), so 131 cannot be cited until the list holds
  it: the list gains 126 to 131 where the statement places them, and B.8
  is named the ledger. 125 leaves the register, cited here; 126 to 130
  join it, each with the unit or phase that will cite it.
- The core's `landed` module gains the one call that asks the channels
  at a landed revision, as `policy_at` asks a policy, so a reader never
  asks at a revision that is not landed (65, core.22).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `seam-contracts`: the policy's interface holds `channels` beside
  `policy-for`, and the channel a policy declares is a type of the
  seams.
- `policy-file`: a channel's table names its component and is bound
  within it; the profile answers the channels the file declares.
- `seam-fakes`: the fake policy answers the channels as scripted.
- `conformance-suite`: the policy's contract holds 131 and 125.
- `clause-coverage`: the suite's clauses run from 1 to 131.

## Out of scope

- Reading the listing: `status-on-any-machine` shows each channel and
  its chain, and `the-ledger-and-promotion` names a promotion's
  component from it. Both come after this unit.
- The gate's own reading of `component` and `on_each_publish` stays as
  the gate unit built it; moving the gate onto the seam's listing is a
  later change the planner places.
- Channels declared by a pattern over branch names (127): phase 2. The
  listing holds the channels the file declares by name.
- Anything a channel requires (`require`), which policy-for answers as
  it does today.

## Impact

- `crates/tailrace-seams`: one operation on `Policy`, one type
  (`DeclaredChannel`), and the one component's name moved here from the
  core so the fake and the profile name it alike. Every `Policy`
  implementor gains the operation: the file profile, the fake, the
  seams' own `Nowhere` and the two `Watched` test policies.
- `crates/tailrace-core`: `declared::REPOSITORY` re-exports the seams'
  name, and `landed::channels_at` asks at a landed revision.
- `crates/tailrace-fakes`, `crates/tailrace-conformance`,
  `crates/tailrace/src/profiles/policy/file/` and its conformance test.
- The self-hosting guide says what the policy file's channel keys mean
  to the policy seam.

## Touches

```
crates/tailrace-seams/src/policy.rs
crates/tailrace-seams/src/artifact.rs
crates/tailrace-seams/src/lib.rs
crates/tailrace-core/src/declared.rs
crates/tailrace-core/src/landed.rs
crates/tailrace-core/tests/verify.rs
crates/tailrace-fakes/src/policy.rs
crates/tailrace-conformance/src/seams/policy.rs
crates/tailrace-conformance/src/fakes.rs
crates/tailrace-conformance/src/core/verify.rs
crates/tailrace-conformance/src/statement.rs
crates/tailrace-conformance/src/register.rs
crates/tailrace-conformance/src/lib.rs
crates/tailrace/src/profiles/policy/file/mod.rs
crates/tailrace/src/profiles/policy/file/tests.rs
crates/tailrace/tests/policy_file.rs
site/src/content/docs/guides/self-hosting.mdx
```
