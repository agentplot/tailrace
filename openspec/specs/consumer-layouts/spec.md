# consumer-layouts Specification

## Purpose
Fills a checkout's consumer layouts from the store with the versions its
lockfiles pin, each judged under the consuming repository's own layout
policy, else, through the signers the host trusts, the publisher's, else
the given, so a client reads only what the policy admits, and keeps on
the host what each layout holds (47 to 51, 61 to 63, 99, 113, C.10).

## Requirements

### Requirement: A layout holds the lockfile's versions and no other
For each bound projection whose pinned answers pins for the checkout,
sync SHALL render that ecosystem's layout from the pinned versions the
policy admits and no other, and SHALL ask the store only for what the
pins name (50, S4). A projection whose pinned answers absent SHALL be
passed over, and no layout of it is rendered or recorded.

#### Scenario: S4, a lockfile's versions and no other
- **WHEN** a checkout whose lockfile pins two family versions is synced, the store holds a third version of the same name, and the policy admits all three
- **THEN** the layout holds exactly the two pinned versions, the store was asked nothing about the third, and the layout's place names no host

#### Scenario: No lockfile, no layout
- **WHEN** a checkout is synced and a bound projection answers pinned absent
- **THEN** no layout of that ecosystem is rendered, recorded or named in the answer as refused or unreadable

### Requirement: A pin is found by ecosystem, name and version alone
Each pin SHALL be resolved through the store name
`artifact/<ecosystem>/<name>@<version>` to the version's index, and the
artifact of that ecosystem and name found in the index by its metadata
(138, 4). The digests resolved SHALL be the ones the layout records
(core.4). A name that resolves to anything but an index naming that
artifact SHALL be unreadable, naming the name and the digest.

#### Scenario: A pin resolved through its index
- **WHEN** a pin's name resolves to an index naming an artifact of that ecosystem, name and version
- **THEN** the layout holds that artifact, and the record names the index's and the artifact's digests

#### Scenario: A name at something that is no such index
- **WHEN** a pin's name resolves to a digest holding no index, or an index naming no artifact of that ecosystem and name
- **THEN** that pin is answered unreadable naming the name and the digest, and nothing is rendered from it

### Requirement: A pin the store does not name is absent
A pin whose name the store does not hold SHALL be answered absent,
naming the name, never refused; every other pin SHALL be judged and
rendered all the same, and an absence SHALL not count against the sync
(51, 58).

#### Scenario: A third-party pin
- **WHEN** a lockfile pins a family version the store names and a package tailrace never published
- **THEN** the family version is held, the other pin is answered absent naming its name, and the sync answers no refusal

### Requirement: The consumer's own layout line at its landed revision
Where the consuming repository's policy file at the landed revision sync
runs for writes a policy for layouts, every version of every layout of
the checkout SHALL be judged under it, read once per sync and never from
the checkout (113, 65, core.22).

#### Scenario: The policy read at the landed revision
- **WHEN** a checkout whose working tree changes the policy file is synced for a landed revision whose layout line requires a claim the working tree's does not
- **THEN** the policy was asked at the landed revision, and a version lacking that claim is refused

#### Scenario: The consumer's line over the publisher's
- **WHEN** a consumer whose file writes a layout line requiring provenance syncs a version whose index carries a trusted publisher's layout-policy claim requiring acceptance
- **THEN** the version is judged under the consumer's own line, and the answer and the record name it at the consumer's revision

### Requirement: The publisher's policy where the consumer writes none
Where the consuming repository writes no layout policy, with no policy
file or a file with no layout line alike, each version SHALL be judged
under the publisher's policy for layouts, read from the layout-policy
claim on its index (113). Only such a claim by a signer the host's
bindings trust SHALL count; any other is kept, never counted (C.10, 118,
9). Two counted claims of different content SHALL be unreadable, naming
both, neither chosen.

#### Scenario: Judged under the publisher's policy
- **WHEN** a consumer with no policy file, on a host that trusts the signer `gate`, syncs a version whose index carries a layout-policy claim by `gate` requiring acceptance by `gate`, and an acceptance claim by `gate`
- **THEN** the version is held, and the answer and the record say it was judged under the publisher's policy at the revision the claim names

#### Scenario: A consumer file with no layout line
- **WHEN** a consumer whose file names a gate and no layout line syncs a version whose index carries a trusted publisher's layout-policy claim
- **THEN** the version is judged under the publisher's policy, not the consumer's given

#### Scenario: A publisher's policy the host does not trust
- **WHEN** a consumer with no policy file syncs a version whose only layout-policy claim was made by a signer the host's bindings do not trust
- **THEN** the claim is not counted and stays attached, and the version is judged under the given

#### Scenario: Two publisher's policies on one index
- **WHEN** a version's index carries two layout-policy claims of different content, each by a trusted signer
- **THEN** that version is answered unreadable naming both claims, and it is not held

### Requirement: A publisher's required claim counts through a trusted signer
Each claim the publisher's policy requires SHALL count only through a
signer that policy names and the host trusts too, verified by the key
the host trusts it by (C.10, core.7).

#### Scenario: A required claim by a signer the host does not trust
- **WHEN** the trusted publisher's policy requires acceptance by `gate` or `box`, the host trusts `gate` alone, and the only acceptance claim is by `box`
- **THEN** the version is refused naming acceptance as the claim found wanting, checked against `gate` alone

### Requirement: The given where neither writes a layout policy
Where the consumer writes no layout policy and no layout-policy claim on
the version's index counts, the version SHALL be judged under the given
policy for a layout: acceptance by the gate (C.10). Where the consumer
has a policy file, the gate SHALL be the one its file's given names,
verified by the keys the file names (116); where it has none, the gate
SHALL be the signers the host trusts (C.10, 118). Every version SHALL
thus be judged under a policy, and none held with none (core.17).

#### Scenario: A consumer with no layout line, and no publisher's
- **WHEN** a consumer whose policy file holds no layout line syncs, among versions whose indexes carry no layout-policy claim, one its file's gate accepted and one it did not
- **THEN** the accepted version is held under the given at the consumer's revision, and the other is refused naming acceptance as the claim found wanting

#### Scenario: No policy file and no publisher's
- **WHEN** a consumer with no policy file, on a host that trusts `gate`, syncs a version whose index carries no layout-policy claim and an acceptance by `gate`
- **THEN** the version is held under the given, and the answer and the record name the given with no revision

#### Scenario: A host that trusts no signer
- **WHEN** a consumer with no policy file syncs on a host whose bindings file names no `[trust]`
- **THEN** every version the store names is refused naming acceptance with no signer checked, and the layout holds none

### Requirement: Only versions whose claims satisfy the policy are held
A version SHALL be held only when, for each claim the policy requires,
in the policy's order, a claim attached to the version's index or to the
artifact counts: made by a signer the policy names for that claim and
allowed its kind and check (47, 9, 116, core.7). A claim by an expired
signer SHALL count and be reported expired (81, 117).

#### Scenario: Every required claim counts
- **WHEN** the policy requires acceptance by the gate's signer and provenance by the build host's, and the index carries the one and the artifact the other, each by its named signer
- **THEN** the version is held

#### Scenario: A claim by a signer the policy does not name
- **WHEN** the only acceptance claim on a version's index is by a signer the policy does not name
- **THEN** the version is not held, and the claim stays attached as it was

### Requirement: A refused version is answered naming the claim found wanting
A version the policy refuses SHALL be answered to whoever ran sync,
naming the version, its index's digest, the first required claim in
the policy's order that no attached claim satisfies, and the signers
checked (99, 58). A version held by an earlier sync and refused now
SHALL leave the layout (47).

#### Scenario: A version that fails the policy
- **WHEN** a pinned version lacks the acceptance claim the policy requires first and carries the provenance it requires second
- **THEN** the answer names the version, the index, acceptance as the claim found wanting and the signers it was checked against, and the layout does not hold the version

#### Scenario: A version refused after it was held
- **WHEN** a layout holds a version and the consumer's policy, landed since, requires a claim the version lacks, and the checkout is synced again
- **THEN** the version is answered refused and the layout no longer holds it

### Requirement: A layout that cannot be refreshed answers what it holds
When the policy, or the store for any pin of a layout, cannot be read,
or the render fails, that layout SHALL be left as it was, and sync SHALL
answer what it holds, when it was last rendered and under which policy,
from the host's record, beside the unreadable answer that stopped it;
it SHALL never answer an absent version as refused or a refused one as
absent (51, 58, 59, S10).

#### Scenario: The store unreachable
- **WHEN** a checkout's layout was rendered holding one version and the store cannot be reached on the next sync
- **THEN** the layout's files are unchanged, and the answer names that version as held, the time of the last render, the policy it was rendered under, and the store's own words, and names nothing refused

#### Scenario: Never rendered and unreachable
- **WHEN** a checkout that has no layout yet is synced while the store cannot be reached
- **THEN** the answer says the layout holds nothing and was never rendered, beside the store's own words

### Requirement: The host keeps what each layout holds
After each render, the host SHALL keep one file per layout under the
checkout's own git directory naming the ecosystem, the place, when it
was rendered, the policy it was rendered under, each version held with
its index's and artifact's digests, and each refused with its claim
found wanting; the file SHALL be readable with a file reader and no
tailrace process, and hold no credential (61, 62, 64, core.20).

#### Scenario: The record read with a file reader
- **WHEN** a layout has been rendered
- **THEN** the checkout's git directory holds one file for it, in a form a program reads, naming every version held and refused and the policy and time of the render

#### Scenario: Two checkouts keep their own
- **WHEN** two worktrees of one repository pin different versions and each is synced
- **THEN** each worktree's record and layout hold its own pins alone

### Requirement: The record is rebuilt by the next sync
The record of rendered layouts SHALL be deletable, and the next sync
SHALL write it again to the same state from the store, git and the
lockfiles, but for the time of the render (63, S12).

#### Scenario: The record deleted
- **WHEN** a layout's record is deleted and the checkout is synced again with nothing else changed
- **THEN** the record holds the same versions, refusals and policy as before, and the layout the same files

### Requirement: A layout's place is in its checkout and names no host
Each ecosystem's layout SHALL be rendered at `.tailrace/layouts/<ecosystem>/`
under the checkout's working tree, a place the same on every host, and
`.tailrace/` SHALL ignore itself to git, so the consumer's configuration
names the layout once and its lockfile names no host (49, S4, core.18).

#### Scenario: The place of a layout
- **WHEN** a checkout is synced on two hosts at different paths
- **THEN** each layout is at `.tailrace/layouts/<ecosystem>/` relative to the working tree on both, and `git status` shows nothing under `.tailrace/`
