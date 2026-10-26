## Why

Every version now carries its notes, an artifact the version rule renders
with no build behind it, and the gate asks every artifact of a version for
every claim the acceptance policy lists. The notes can never carry a build
verdict, so once the gate and the notes meet on one tree every version a
policy asks a verdict of is refused, naming its notes. The statement says
the policy names what a built artifact requires and what a rendered one
requires, and the gate judges each artifact against the claims required of
artifacts made its way (134, 25, 28; agentplot/blueprints 2b8fc53).

## What Changes

- The policy seam's answer says, for each claim it requires, which
  artifacts it is asked of: every artifact, or a built artifact alone
  (82, 134).
- The policy file asks a build's claims, a verdict, a test, a review and a
  rebuild, of a built artifact alone, and the publisher's provenance and
  the claims about the version as a whole, the gate's acceptance and a
  promotion, of every artifact. The file gains no key: it reads what it
  holds today and answers each claim with the artifacts it is asked of
  (C.10).
- The gate tells how each artifact of a version was made from the claims
  attached to it: an artifact whose publisher's provenance, counted under
  the acceptance policy, names no build identity is a rendered artifact;
  every other artifact is judged as built (13, 29, 134).
- The gate judges each artifact against the claims asked of artifacts made
  its way, for acceptance and for each channel that follows each publish,
  so a version whose notes carry no build verdict is accepted when its
  outputs' claims count, and refused naming the artifact and the claim
  found wanting where they do not (25, 28, 95).
- `tailrace policy` shows a claim asked of built artifacts alone as such,
  so a reader sees why a version's notes are asked no verdict (56, 60).
- The gate's conformance group gains the scenarios that prove it, and the
  self-hosting guide and an ADR say how the gate tells a rendered artifact.

Out of scope: a key in the policy file with which the authors would choose
for themselves which artifacts a claim is asked of (the file's form for it
is not written in C.10; it waits on the design); judging a consumer layout
or `verify` by an artifact's making, since a layout holds one ecosystem's
artifact and never the notes; and adopting a later revision of the
statement into the conformance suite's clause list. This change is
written against the gate on `bolt/tailrace-the-gate-and-the-channels`
(the-gate-accepts-under-the-policy) and the notes of
a-version-carries-its-notes on this bolt; its code waits until that bolt
lands on main and main is taken into this bolt.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `acceptance`: each artifact is judged against the claims the policy asks
  of artifacts made its way, at acceptance and for each following channel.
- `policy-file`: each required claim is answered with the artifacts it is
  asked of, read from its kind.
- `seam-contracts`: the policy's answer names, for each required claim,
  the artifacts it is asked of.
- `command-line`: `tailrace policy` shows a claim asked of built artifacts
  alone.

## Impact

- `tailrace-seams`: the required claim gains the artifacts it is asked of;
  every place that makes one names it.
- `tailrace-core`: the gate tells each artifact's making once per
  judgment and judges acceptance and each following channel by it.
- `tailrace`: the policy file profile and the policy command; the CLI's
  end-to-end tests of a publish that tells the gate.
- `tailrace-fakes`, `tailrace-conformance`: the fakes and scenarios that
  make required claims, and the gate group's new scenarios.
- Rules kept: core.7 (a claim counts only through a signer the policy
  names; the making is read only from a provenance that counts), core.28
  (a version with one refused artifact is refused whole), core.31 (the
  policy command renders the seam's answer and computes nothing).

## Touches

```
crates/tailrace-seams/src/**
crates/tailrace-core/src/**
crates/tailrace-core/tests/**
crates/tailrace-fakes/src/**
crates/tailrace-conformance/src/**
crates/tailrace/src/profiles/policy/file/**
crates/tailrace/src/commands/policy.rs
crates/tailrace/tests/**
docs/adr/**
site/src/content/docs/guides/self-hosting.mdx
```
