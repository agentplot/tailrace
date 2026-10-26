## Purpose

Fills a checkout's consumer layouts from the store with the versions its
lockfiles pin, each judged under the consuming repository's policy for
layouts or the publisher's, so a client reads only what the policy
admits, and keeps on the host what each layout holds (47 to 51, 61 to
63, 99, 113).

## ADDED Requirements

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

### Requirement: The consumer's own policy at its landed revision
A layout SHALL be rendered under the consuming repository's policy for
layouts, read at the landed revision sync runs for, never from the
checkout (113, 65, core.22). It SHALL be read once per sync and the
same policy SHALL judge every layout of the checkout.

#### Scenario: The policy read at the landed revision
- **WHEN** a checkout whose working tree changes the policy file is synced for a landed revision whose policy file requires a claim the working tree's does not
- **THEN** the policy was asked at the landed revision, and a version lacking that claim is refused

### Requirement: The publisher's policy where the consumer declares none
Where the consuming repository declares no policy for layouts at that
revision, each version SHALL be judged under the publisher's policy for
layouts, read from the one layout-policy claim attached to the
version's index, reaching no repository but the consumer's own (113).
Two layout-policy claims of different content on one index SHALL be
unreadable, naming both, and neither chosen.

#### Scenario: Judged under the publisher's policy
- **WHEN** a consumer with no policy for layouts syncs a version whose index carries a layout-policy claim requiring the gate's acceptance, and the index carries an acceptance claim by the signer it names
- **THEN** the version is held, and the answer and the record say it was judged under the publisher's policy at the revision the claim names

#### Scenario: Two publisher's policies on one index
- **WHEN** a version's index carries two layout-policy claims of different content
- **THEN** that version is answered unreadable naming both claims, and it is not held

### Requirement: No layout is rendered with no policy
A version for which neither the consumer's policy nor a layout-policy
claim on its index gives a policy for layouts SHALL be refused, naming
the version and the publisher's layout policy as the claim found
wanting, and SHALL NOT be held (core.17).

#### Scenario: No policy anywhere
- **WHEN** a consumer with no policy for layouts syncs a version whose index carries no layout-policy claim
- **THEN** the version is refused naming the publisher's layout policy as wanting, and the layout does not hold it

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
