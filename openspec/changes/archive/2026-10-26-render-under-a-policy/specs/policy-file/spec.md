## MODIFIED Requirements

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

## ADDED Requirements

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
