# policy-file Specification

## Purpose
The policy seam over `.config/tailrace.toml` at a landed revision: the
signers the repository's authors name, what each may claim, and the
claims acceptance, a layout or a channel requires (C.10).

## Requirements

### Requirement: The policy is read from the file at the revision asked

The profile SHALL read `.config/tailrace.toml` from the tree of the
revision it is asked about, in the repository's history, and never from
a working tree (65, core.22).

#### Scenario: A checkout's edit is not read

- **WHEN** the working tree's policy file names a signer the file at the revision does not
- **THEN** the acceptance policy at that revision names only the signers of the committed file

### Requirement: Signers are named with their verifier and their reach

Each `[signers.<name>]` table SHALL name one signer: exactly one key
other than `kinds` and `checks` whose string value is the text the
signer's profile verifies by, `kinds` the claim kinds it may make, and
`checks` the checks it may claim a verdict of, by the landing's names
(82, 116). A signer is bound by no output and no ecosystem.

#### Scenario: A host key listed

- **WHEN** the file holds `[signers.laptop]` with `ssh = "ssh-ed25519 AAAA… host@laptop"` and `kinds = ["provenance"]`
- **THEN** a policy requiring provenance from `laptop` names a signer whose id is `laptop`, verified by `ssh-ed25519 AAAA… host@laptop`, who may claim provenance and no check

#### Scenario: A signer with two verifiers

- **WHEN** a `[signers.<name>]` table holds two keys besides `kinds` and `checks`
- **THEN** the answer is unreadable, naming the file, the revision and the table

### Requirement: Each audience lists the claims it requires, in order

`[acceptance]`, `[layout]` and `[channels.<name>]` SHALL each hold
`require`, a list of claims in the order the verdict reads them, each
with its `kind`, its `check` where it is a verdict, and the `signers` by
their names in `[signers]`; a channel's table MAY name `bound_to` (82,
83, 37). The acceptance and layout tables SHALL name no channel.

#### Scenario: Acceptance requires two claims

- **WHEN** `[acceptance]` requires provenance from `laptop`, then a verdict of the check `cli:test` from `box`
- **THEN** the acceptance policy answers those two claims in that order, each with its signers as `[signers]` names them

#### Scenario: A bound channel

- **WHEN** `[channels.stable]` names `bound_to = "beta"`
- **THEN** the policy for the channel `stable` names `beta` as the channel it is bound to

#### Scenario: Acceptance naming a channel

- **WHEN** `[acceptance]` names `bound_to`
- **THEN** the answer is unreadable, naming the file and the table

#### Scenario: A signer no table names

- **WHEN** a required claim names a signer `[signers]` does not hold
- **THEN** the answer is unreadable, naming the file, the revision and the signer

### Requirement: An audience with no table is absent

Where the revision holds no policy file, the answer for every audience
SHALL be absent naming the audience; where its file holds no table for
acceptance or for a channel asked, the answer SHALL be absent naming
the audience (82, 58); and where its file holds no `[layout]`, the
answer for a layout SHALL be the given policy for a layout (C.10). A
file that cannot be read or parsed SHALL be unreadable naming the file
and the revision, with git's or the parser's own words (59).

#### Scenario: No layout policy

- **WHEN** the file holds `[acceptance]` and no `[layout]`
- **THEN** the policy for a layout is the given one, marked as given, never absent

#### Scenario: No policy for a channel

- **WHEN** the file holds `[acceptance]` and no `[channels.nightly]`
- **THEN** the policy for the channel `nightly` is absent, naming the audience

#### Scenario: No policy file

- **WHEN** the revision's tree holds no `.config/tailrace.toml`
- **THEN** the policy for a layout is absent, naming the audience

#### Scenario: A malformed file

- **WHEN** the file at the revision is not well formed
- **THEN** the answer is unreadable, naming the file and the revision, with the parser's words verbatim

### Requirement: The profile passes the policy group

The profile SHALL pass the conformance suite's policy group, played
against a scratch repository whose commits hold the policy files the
group asks for (93, core.33).

#### Scenario: The group over the file

- **WHEN** the policy group runs against the profile over a scratch repository
- **THEN** every scenario of the group passes

### Requirement: The given policy for a layout admits what the gate accepted

A file with no `[layout]` SHALL answer for a layout, marked as given,
one required claim:
acceptance, by every signer its `[signers]` lets claim acceptance, in
the file's order; where it names none that may and exactly one that may
claim provenance, by that one, as the one host 108 admits; and
otherwise by no signer, so the given admits no version (C.10, 108).

#### Scenario: A gate named

- **WHEN** the file names `[signers.gate]` with `kinds = ["acceptance"]` and `[signers.laptop]` with `kinds = ["provenance"]`, and no `[layout]`
- **THEN** the policy for a layout requires acceptance by `gate` alone, verified by the key `[signers.gate]` names

#### Scenario: One publisher and no gate

- **WHEN** the file names one signer, `laptop`, with `kinds = ["provenance"]`, and no `[layout]`
- **THEN** the policy for a layout requires acceptance by `laptop`

#### Scenario: Several publishers and no gate

- **WHEN** the file names two signers that may claim provenance, none that may claim acceptance, and no `[layout]`
- **THEN** the policy for a layout requires acceptance by no signer, never absent and never unreadable

### Requirement: The given names no signer, and a written line stands alone

The given SHALL name no signer of its own: its gate is read from the
file alone (116, C.10). A file's own `[layout]` SHALL be answered as
written, not marked as given, with nothing of the given laid beneath it.

#### Scenario: A layout line written

- **WHEN** the file's `[layout]` requires provenance from `laptop`
- **THEN** the policy for a layout requires that provenance alone, and no acceptance, and is not marked as given
