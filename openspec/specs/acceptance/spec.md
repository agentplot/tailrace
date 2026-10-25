# acceptance Specification

## Purpose
The gate: the one party that judges an offered version under the policy
for acceptance at the governed branch's landed head, accepts it whole
with a claim of its own or refuses it whole naming the claim found
wanting, and moves each channel that follows each publish (25 to 30, 94
to 96, 107, 108).

## Requirements

### Requirement: The gate judges a version by its index's digest

The gate SHALL be asked with a branch, a component where the repository
declares several, a version, what told it of the version (a publish's
answer, or an asker) and the identity it signs as. It SHALL resolve the
version's name in the store to its index's digest before reading
anything of the version, and SHALL name that digest in its answer
(core.4). A name that names nothing SHALL be answered absent, naming it.

#### Scenario: The gate resolves the name first

- **WHEN** the gate is asked to judge `0.2.0` of a repository whose store names `v0.2.0` at an index
- **THEN** its answer names that index's digest, and nothing of the version is read before the name is resolved

#### Scenario: A version never offered

- **WHEN** the gate is asked to judge a version whose name the store does not hold
- **THEN** the answer is absent from the store, naming the version's tag, and nothing is written

### Requirement: The policy is read at the governed branch's landed head

The gate SHALL read the policy for acceptance, and for each channel it
judges, at the head of the governed branch as the landing seam lands it
at the moment it judges, never at the version's own revision and never
from a checkout (107, 65, core.22). It SHALL name that revision in its
answer.

#### Scenario: A policy landed after the publish binds the judgment

- **WHEN** a version is published at one revision, and the branch then lands a revision whose acceptance policy requires a verdict the version does not carry
- **THEN** the gate refuses the version under the later policy, naming the later revision as the one it read

#### Scenario: A branch head the landing has not landed

- **WHEN** the branch's head is a revision the landing seam says the branch has not landed
- **THEN** the gate is refused naming the branch and the revision, and nothing is read of the policy or written

### Requirement: The gate judges only a landed revision's version

The gate SHALL check that the landing landed the revision the version's
tag names on the governed branch, and SHALL judge by the claims
attached, the landing and the policy alone: it never rebuilds, never
reads an artifact's bytes, and asks nothing else whether to accept (29).

#### Scenario: A version whose revision did not land

- **WHEN** the version's tag names a revision the branch has not landed
- **THEN** the gate is refused naming the branch and that revision, and nothing is attached

#### Scenario: No bytes are read

- **WHEN** the gate judges a version
- **THEN** the store is asked to read the index and the claims attached to each digest, and never the bytes of an artifact

### Requirement: Each artifact is judged against each required claim in order

For each artifact the index names, in order, the gate SHALL take the
acceptance policy's required claims in order, each satisfied only by a
claim on the artifact or the index naming the version's revision, whose
maker the signer seam tells is a signer the required claim names, within
that signer's reach (9, 13, 80, 116, core.7). A verdict SHALL count only
carried in the artifact's provenance, for the check and build required.

#### Scenario: Every required claim found

- **WHEN** acceptance requires provenance from `host` and a verdict of `test` from `landing`, and each artifact carries provenance from `host` holding a passing `test` verdict from `landing` for its build
- **THEN** the version is accepted

#### Scenario: A claim from a signer the policy does not name

- **WHEN** acceptance requires a verdict of `test` from `landing`, and the verdict each artifact carries was signed by a signer the policy does not name
- **THEN** the version is refused naming the required verdict and the first artifact, and the unnamed signer's claim stays attached

#### Scenario: A claim beyond its signer's reach

- **WHEN** a signer the policy lets claim only provenance made a verdict of `test`
- **THEN** that verdict does not count, and the version is refused naming the required verdict

#### Scenario: A claim about another revision

- **WHEN** an artifact carries a provenance from the named signer that names a revision other than the version's
- **THEN** it does not count, and the version is refused naming the required provenance

### Requirement: An expired signer's claim does not count at acceptance

A claim whose maker's identity the signer seam reports expired SHALL be
reported as such and SHALL NOT count at acceptance (81); the refusal
names the claim and that its signer expired. A claim that counted when
the gate accepted keeps counting after (117), which the gate never
revisits.

#### Scenario: Provenance from an expired signer

- **WHEN** the only provenance an artifact carries was made by a signer the signer seam reports expired
- **THEN** the version is refused naming the required provenance, the artifact and that its signer expired

### Requirement: A version is accepted or refused whole

The gate SHALL answer one verdict for the whole version: accepted, when
every artifact satisfies every required claim; or refused, naming the
first artifact found wanting in the index's order, the first required
claim found wanting for it in the policy's order, and why (28, 58). A
seam that cannot be read SHALL be answered unreadable naming what, with
its own words, never as a refusal (58, 59).

#### Scenario: One artifact wanting refuses the version

- **WHEN** a version names two artifacts and the second carries no provenance
- **THEN** the version is refused naming the second artifact and the required provenance, and nothing is attached to either artifact or to the index

#### Scenario: A store that cannot be read

- **WHEN** the store cannot read the claims of an artifact
- **THEN** the answer is unreadable from the store naming that digest, with the store's own words, and is no refusal

### Requirement: Acceptance attaches one claim and changes no bytes

On acceptance the gate SHALL sign one acceptance claim about the index,
in its own identity, naming the version's revision, the revision whose
policy it read, the component, the version and the digests of the
claims that satisfied each required claim, and attach it to the index
(7, 26, 28). It SHALL put no bytes and move no name for the acceptance,
and the index and every artifact keep the digests they were offered
with (26).

#### Scenario: S3, acceptance attached and the bytes the ones offered

- **WHEN** the gate accepts an offered version
- **THEN** the index holds one more claim, an acceptance signed in the gate's identity, every artifact's claims are as they were, the index and every artifact resolve to the digests the publish answered, and the store was asked for no put

#### Scenario: S5, a refused version stays offered

- **WHEN** the gate refuses a version
- **THEN** the store was asked for no write, and the version still reads by its digest with exactly the claims it was offered with

### Requirement: Nothing accepted is un-accepted

Where the index already holds an acceptance claim the gate's identity
made, the gate SHALL answer the version accepted with that claim and
attach nothing, whatever the policy at the branch's head now requires
(94).

#### Scenario: Judging an accepted version again

- **WHEN** the gate is asked to judge a version it accepted, after a later policy that the version does not satisfy has landed
- **THEN** the answer is accepted, naming the acceptance claim already attached, and nothing is written

### Requirement: The gate signs only under the accepting credential

The gate SHALL sign through a bound signer that holds its identity at
hand, and where none does SHALL answer absent naming the identity,
having written nothing (79, 95). The gate's path SHALL take no offering
ask and call no part of a publish (core.14).

#### Scenario: A host with no accepting credential

- **WHEN** the gate is run on a host whose bound signers hold no key for the gate's identity, and the version satisfies the policy
- **THEN** the answer is absent from the signer naming the gate's identity, nothing is attached, and no channel moves

### Requirement: The gate never accepts what it offered, but on the one host

Where the provenance an artifact's acceptance counted was made by the
gate's own identity, the gate SHALL refuse the version naming that
artifact and the required provenance, as offered by the gate (27, 96),
unless the policy names that identity as a signer that may claim both
provenance and acceptance (108).

#### Scenario: A build host running the gate

- **WHEN** the policy names `host` for provenance alone, and the gate is run signing as `host` over a version whose provenance `host` made
- **THEN** the version is refused naming the required provenance and the first artifact, as offered by the gate, and nothing is attached

### Requirement: A one-host acceptance says so

Where the policy names one signer that may claim both provenance and
acceptance and that signer, as the gate, accepts a version whose
provenance it made, its acceptance claim SHALL say the host accepted what
it offered, naming the signer; every other acceptance claim SHALL say
it was separated, naming the signers that made the provenance (108).

#### Scenario: The solo laptop

- **WHEN** the policy lets `laptop` claim provenance and acceptance, and the gate signing as `laptop` accepts a version whose provenance `laptop` made
- **THEN** the acceptance claim says one host offered and accepted, naming `laptop`

#### Scenario: A separated gate

- **WHEN** the gate signing as `gate` accepts a version whose provenance `host` made
- **THEN** the acceptance claim says it was separated, naming `host` as the offerer

### Requirement: An independent rebuild is counted, never made

A required rebuild claim SHALL count only when its maker is a signer
other than the one whose provenance the artifact carries and the claim
says the digests matched; the gate SHALL make no rebuild (30).

#### Scenario: A rebuild by the offering host

- **WHEN** acceptance requires a rebuild, and the only rebuild claim on an artifact was made by the signer that made its provenance
- **THEN** it does not count, and the version is refused naming the required rebuild

#### Scenario: A rebuild that did not match

- **WHEN** a second signer's rebuild claim on an artifact says the digests did not match
- **THEN** it does not count, and the version is refused naming the required rebuild

### Requirement: A channel that follows each publish is declared in the policy file

The core SHALL read, at the governed branch's landed head, which
channels follow each publish: each `[channels.<name>]` table of
`.config/tailrace.toml` with `on_each_publish = true`, belonging to the
component its `component` key names, or to the one component where the
repository declares one (32, 125). A following channel that names no
component in a repository of several, or that names `bound_to`, SHALL
be unreadable naming its table (37).

#### Scenario: beta follows each publish

- **WHEN** the policy file at the branch's head holds `[channels.beta]` with `on_each_publish = true` in a repository of one component
- **THEN** `beta` follows each publish of that component

#### Scenario: A following channel bound to another

- **WHEN** `[channels.stable]` holds both `on_each_publish = true` and `bound_to = "beta"`
- **THEN** the gate's answer for `stable` is unreadable naming the table, and `stable` does not move

### Requirement: The gate moves each following channel after accepting what a publish told it of

When the gate accepts a version it was told of by a publish's answer, it
SHALL judge the version under each following channel's own policy, and
for each satisfied SHALL move that channel to the version as its own
act: never a publisher's, and never for a version it was asked to judge
by anyone else (95, 20, core.10). Each channel SHALL be answered on its
own.

#### Scenario: S3, beta moves on acceptance

- **WHEN** the gate, told by a publish's answer, accepts a version, and `beta` follows each publish and requires acceptance from the gate
- **THEN** `beta` names the version's index, and the answer for `beta` names the promotion

#### Scenario: Asked rather than told

- **WHEN** the gate accepts a version it was asked to judge by an asker, not told of by a publish
- **THEN** no channel moves

#### Scenario: A channel's own policy refuses

- **WHEN** `beta` follows each publish and requires a review the version does not carry
- **THEN** the version is accepted, `beta` does not move, and the answer for `beta` is refused naming the required review

### Requirement: A following channel moves through the ledger first

The gate SHALL sign a promotion claim about the index naming the
channel, the versions it moves from and to, the rule that asked, the
policy and claims that satisfied it, when, and its predecessor; append
it after the chain's head through the ledger seam; attach it to the
index; and only then move the channel's name in the store (33, 39, 105,
core.6, core.15). A refused append or a fork SHALL be reported, nothing
retried (35, core.34).

#### Scenario: The ledger holds the move before the name

- **WHEN** the gate moves `beta`
- **THEN** the promotion is held at the head of `beta`'s chain, naming the previous head as its predecessor, before the store is asked to move the name `beta`, and the promotion is attached to the index

#### Scenario: A predecessor that is no longer the head

- **WHEN** another promotion of `beta` is appended between the gate's read of the chain and its append
- **THEN** the answer for `beta` is refused naming the claim at the head, the store's name `beta` is not moved, and nothing is retried

#### Scenario: A channel already at the version

- **WHEN** the head of `beta`'s chain is already a promotion to the version's index
- **THEN** nothing is appended or moved, and the answer for `beta` says it was there already

### Requirement: S9, a stolen build credential moves no channel

A claim of acceptance or promotion signed by a key the policy does not
let claim it SHALL never count, so a host holding only an offering
credential can offer and attach claims and can neither accept nor move
a channel; what it offered stays unaccepted and attributable to the
signer that made its provenance (9, 27, 96, S9).

#### Scenario: A thief's acceptance on the index

- **WHEN** a build host's key signs an acceptance claim and attaches it to an offered index, and `beta` requires acceptance from `gate`
- **THEN** a judgment of `beta`'s policy counts no acceptance, `beta` does not move, and the index's claims name the build host as the maker of its provenance and of the forged claim

#### Scenario: A publish moves no channel

- **WHEN** a publish offers a version in a repository where `beta` follows each publish, and no gate runs
- **THEN** `beta` names nothing new, the chain of `beta` holds no new promotion, and the version is offered and not accepted
