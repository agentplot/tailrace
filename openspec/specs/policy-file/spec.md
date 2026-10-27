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

`[acceptance]` and `[layout]` SHALL each hold `require`, a list of
claims in the order the verdict reads them, each with its `kind`, its
`check` where it is a verdict, and the `signers` by their names in
`[signers]` (82). A channel's table SHALL hold such a list in `require`
or leave it out, requiring nothing of its own, and MAY name `bound_to`,
`on_each_publish`, `prereleases` or `approvers` (83, 37, 95, C.10). The
acceptance and layout tables SHALL name no channel.

#### Scenario: Acceptance requires two claims

- **WHEN** `[acceptance]` requires provenance from `laptop`, then a verdict of the check `cli:test` from `box`
- **THEN** the acceptance policy answers those two claims in that order, each with its signers as `[signers]` names them

#### Scenario: A bound channel

- **WHEN** `[channels.next]` names `bound_to = "edge"` and `[channels.edge]` is declared
- **THEN** the policy for the channel `next` names `edge` as the channel it is bound to

#### Scenario: Acceptance naming a channel

- **WHEN** `[acceptance]` names `bound_to`
- **THEN** the answer is unreadable, naming the file and the table

#### Scenario: A signer no table names

- **WHEN** a required claim names a signer `[signers]` does not hold
- **THEN** the answer is unreadable, naming the file, the revision and the signer

### Requirement: An audience with no table is absent

Where the revision holds no policy file, or its file holds no table for
acceptance or for a channel asked, the answer SHALL be absent naming
the audience (82, 58), except for a channel the givens declare, a held
one the repository has switched on, or one an environment's words make,
whose policy SHALL be answered as given (C.10); where the revision holds
no policy file, the answer for a layout SHALL be absent naming it, and
where its file holds no `[layout]`, the answer for a layout SHALL be the
given policy for a layout (C.10). A file that cannot be read or parsed
SHALL be unreadable naming the file and the revision, with git's or the
parser's own words (59).

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

#### Scenario: A given channel with no line

- **WHEN** the file holds no `[channels]` and the policy for `stable` is asked
- **THEN** it answers the gate's acceptance as the one claim required, `beta` as the channel it is bound to, the gate's signers as its promoters, and no approvers

#### Scenario: A channel neither given nor declared

- **WHEN** the policy for `edge` is asked and neither the file nor the fragment declares it and no environment makes it
- **THEN** the answer is absent naming the channel

### Requirement: The profile passes the policy group

The profile SHALL pass the conformance suite's policy group, played
against a scratch repository whose commits hold the policy files the
group asks for (93, core.33).

#### Scenario: The group over the file

- **WHEN** the policy group runs against the profile over a scratch repository
- **THEN** every scenario of the group passes

### Requirement: The given policy for a layout admits what the gate accepted

A file with no `[layout]` SHALL answer for a layout, marked as given,
one required claim: acceptance, by every signer its `[signers]` lets
claim acceptance, in the file's order, and by no signer where it names
none that may (C.10). The one host 108 admits as the gate SHALL be the
gate for publishing, not reading: the given SHALL never name a signer
the file lets claim provenance alone, and its reader reads a claim it
names no signer for through its host's trusted signers (118).

#### Scenario: A gate named

- **WHEN** the file names `[signers.gate]` with `kinds = ["acceptance"]` and `[signers.laptop]` with `kinds = ["provenance"]`, and no `[layout]`
- **THEN** the policy for a layout requires acceptance by `gate` alone, verified by the key `[signers.gate]` names

#### Scenario: One publisher and no gate

- **WHEN** the file names one signer, `laptop`, with `kinds = ["provenance"]`, and no `[layout]`
- **THEN** the policy for a layout requires acceptance naming no signer, marked as given, and never names `laptop` for it

#### Scenario: Several publishers and no gate

- **WHEN** the file names two signers that may claim provenance, none that may claim acceptance, and no `[layout]`
- **THEN** the policy for a layout requires acceptance naming no signer, marked as given, never absent and never unreadable

### Requirement: The given names no signer, and a written line stands alone

The given SHALL name no signer of its own: its gate is read from the
file alone (116, C.10). A file's own `[layout]` SHALL be answered as
written, not marked as given, with nothing of the given laid beneath it.

#### Scenario: A layout line written

- **WHEN** the file's `[layout]` requires provenance from `laptop`
- **THEN** the policy for a layout requires that provenance alone, and no acceptance, and is not marked as given

### Requirement: The policy for a layout carries the file's signers

The policy for a layout, given or written, SHALL carry every signer
`[signers]` names, in the file's order, each with what it may claim, so
a consumer reads each claim through its file's signers alone where the
file names one for it, and through its host's trusted signers only
where the file names none (C.10, 118).

#### Scenario: The signers carried

- **WHEN** the file names `[signers.gate]` with `kinds = ["acceptance"]` and `[signers.curator]` with `kinds = ["layout_policy"]`, with or without a `[layout]`
- **THEN** the policy for a layout carries `gate` then `curator`, each with its verifier and the kinds the file lets it claim

### Requirement: A channel's table names its component

Where the repository declares no component, a channel SHALL be declared
as `[channels.<name>]`, in the one component (123). Where it declares
components, a channel SHALL be declared as
`[channels.<component>.<name>]`, the component one a `[[component]]`
declares (125, C.10). A table of another form, or naming a component
not declared, SHALL be unreadable naming the file, the revision, the
table and the form to use. The profile reads only each component's
`name`.

#### Scenario: One component has every channel

- **WHEN** the file declares no `[[component]]`, and holds `[channels.edge]` and `[channels.next]`
- **THEN** both channels are of no declared component, spelled `edge` and `next`

#### Scenario: The one declared component

- **WHEN** the file declares `[[component]]` `lib` and holds `[channels.lib.edge]`
- **THEN** `edge` belongs to `lib`, spelled `lib/edge`

#### Scenario: A channel naming no component among several

- **WHEN** the file declares `lib` and `web` and holds `[channels.edge]` with `require`
- **THEN** the channels are unreadable, naming the file, the revision, `[channels.edge]`, and `[channels.<component>.<name>]` as the form, and so is the policy for any channel

#### Scenario: A channel naming a component not declared

- **WHEN** the file declares `lib` and holds `[channels.app.edge]`
- **THEN** the channels are unreadable, naming `[channels.app.edge]` and `app`

#### Scenario: The component key of before

- **WHEN** the file declares `lib` and holds `[channels.edge]` naming `component = "lib"`
- **THEN** the channels are unreadable naming `[channels.edge]` and `[channels.lib.edge]` as the form to use

### Requirement: A channel is bound within its component

A `bound_to` SHALL name a channel of the same component that the table
in force lists, given, switched on, declared by hand or made by an
environment; one that names a channel of no
such kind SHALL be unreadable naming the table and the channel it names
(37, 125). A channel's component and how it moves SHALL be read the same way
whether the channels are listed or one channel's policy is asked, so
the two never disagree (83).

#### Scenario: A binding across components

- **WHEN** `[channels.lib.next]` names `bound_to = "edge"`, and only `[channels.web.edge]` declares an `edge`
- **THEN** the channels are unreadable, naming `[channels.lib.next]` and `edge`, and so is the policy for `next` of `lib`

#### Scenario: A binding to a channel not declared

- **WHEN** `[channels.next]` names `bound_to = "edge"` and neither the file nor the fragment declares an `edge` and no environment makes one
- **THEN** the channels are unreadable, naming `[channels.next]` and `edge`

#### Scenario: A binding to a given channel

- **WHEN** `[channels.next]` names `bound_to = "beta"` and the file writes no line under `beta`
- **THEN** `next` is bound to the given `beta`

### Requirement: The profile lists the channels the file declares

The profile SHALL answer channels with every channel the table in force
lists, the givens of every component among them, and each channel an
environment's words make, once each, ordered by name and then
component, each with its component and how it moves (131, C.10). Where
the revision holds no policy file the answer SHALL be the givens the
fragment declares, of the one component. A revision the repository does not hold, or a
file that cannot be read, SHALL answer as policy-for answers it (58,
59).

#### Scenario: Two components' channels

- **WHEN** the file declares `lib` and `web`, `[channels.lib.next]` bound to `beta`, and `[channels.web.edge]` with `on_each_publish = true`
- **THEN** the channels answer `beta` of `lib` and of `web` on each publish, `edge` of `web` on each publish, `next` of `lib` bound to `beta`, and `stable` of each bound to its `beta`, in that order

#### Scenario: No channel declared

- **WHEN** the file at the revision holds `[acceptance]` and no `[channels]` or `[environments]`, or the revision holds no policy file
- **THEN** the channels answer `beta` moving on each publish and `stable` bound to `beta`

#### Scenario: A checkout's channel is not read

- **WHEN** the working tree's policy file declares a channel the file at the revision does not
- **THEN** the channels at that revision do not list it (65, core.22)

### Requirement: How a declared channel moves is read from its table

A channel declared by hand SHALL move on each accepted publish where
its table names `on_each_publish = true`, be bound where it names
`bound_to`, and move by a promotion alone where it names neither (95,
37, 131). A table naming both SHALL be unreadable naming it, since a
bound channel takes no version from a publish (37).

#### Scenario: A hand-declared following channel

- **WHEN** `[channels.edge]` names `on_each_publish = true`
- **THEN** `edge` is listed moving on each publish

#### Scenario: A channel of neither key

- **WHEN** `[channels.edge]` names only `require`
- **THEN** `edge` is listed moving by a promotion alone

### Requirement: The given channels' policy names the gate

The policy for a channel the givens hold that the table in force lists,
or a channel an environment's words make, SHALL require, first, an acceptance by the signers `[signers]`
lets claim acceptance, then any `require` a line under it names, in
order; and SHALL name those same signers, the gate, as its promoters
whatever the line names (32, C.10). A hand-declared channel's promoters
SHALL be the signers its `require` names that may claim promotion.

#### Scenario: beta with no line

- **WHEN** `[signers.gate]` names kinds `acceptance` and `promotion`, `[signers.laptop]` kinds `provenance`, and no `[channels]`
- **THEN** the policy for `beta` requires an acceptance by `gate` and names `gate` as its one promoter

#### Scenario: A hand-declared channel's promoters

- **WHEN** `[channels.edge]` requires an acceptance from `gate`, and `[signers.release]` with kinds `promotion` is named in its `require`
- **THEN** the policy for `edge` names `release` as its promoter

#### Scenario: A given's line naming another promoter

- **WHEN** `[channels.alpha]` requires a promotion from `release`, a signer of kinds `promotion` alone
- **THEN** the policy for `alpha` requires the gate's acceptance and then that claim, and names only the gate as its promoter

### Requirement: A following channel's prereleases are read from its table

A channel whose table in force moves on each publish SHALL take
releases alone unless the table names `prereleases = true`, and then
prereleases too (C.10). `prereleases` in a table in force that does not
move on each publish SHALL be unreadable naming the table and the key,
since such a channel takes no version from a publish (37).

#### Scenario: A hand-declared channel taking prereleases

- **WHEN** `[channels.edge]` names `on_each_publish = true` and `prereleases = true`
- **THEN** `edge` is listed moving on each publish with prereleases

#### Scenario: prereleases on a bound channel

- **WHEN** `[channels.stable]` names `prereleases = true`
- **THEN** the channels are unreadable naming `[channels.stable]` and `prereleases`

### Requirement: approvers names who may ask

A channel's table MAY name `approvers`, a list of signers by their
names in `[signers]`, and the policy for the channel SHALL answer them
as the signers that may ask for it; for a channel a manual environment
makes, the policy SHALL answer the `approvers` the environment's table
names; where neither names `approvers`, the policy SHALL answer none,
any asker the gate verifies (129, C.10). A name `[signers]` does not hold SHALL be unreadable naming
the table and the name, and an empty list SHALL be unreadable naming
the table, since it would read as naming no one.

#### Scenario: Approvers under a given

- **WHEN** `[signers.ci]` and `[signers.desk]` are declared and `[channels.stable]` names `approvers = ["ci"]`
- **THEN** the policy for `stable` names `ci` as the one signer that may ask, and its promoters are still the gate's signers

#### Scenario: An approver no table names

- **WHEN** `[channels.stable]` names `approvers = ["nobody"]` and `[signers]` holds no `nobody`
- **THEN** the channels are unreadable naming `[channels.stable]` and `nobody`

### Requirement: The environments' words are read

An environment SHALL be declared as `[environments.<name>]` where no
component is declared and `[environments.<component>.<name>]` where
components are. Its keys SHALL be C.10's words and no others: its
`channel`, `policy` (`auto` or `manual`, `auto` where unwritten),
`after`, a list of environment names of the same component, `target`,
which the profile SHALL leave to the target's reader, reading nothing
of its value, and, where the environment is manual, `approvers`, the
signers that may say yes, read as a channel's `approvers` is (C.10, as
ruled at agentplot/blueprints 088430c). Any other `policy`, or a
`channel` the file does not declare, give or make, SHALL be unreadable
naming the table. `approvers` on an environment that is not manual
SHALL be unreadable naming the table and the key, since no one says yes
to it. Any other key SHALL be unreadable naming the table and the key,
since a target's own settings sit in the target's table and nowhere
else (C.10).

#### Scenario: An environment naming its target

- **WHEN** `[environments.site]` names `channel = "stable"` and a `target`
- **THEN** the channels answer is read, and `site` makes no channel

#### Scenario: An environment with a target's own key

- **WHEN** `[environments.site]` names `channel = "stable"` and `bucket = "example-site"`
- **THEN** the channels are unreadable naming `[environments.site]` and `bucket`, and saying a target's settings sit in the target's table

#### Scenario: Approvers in a manual environment's table

- **WHEN** `[signers.ci]` is declared and `[environments.prod]` names `channel = "stable"`, `policy = "manual"` and `approvers = ["ci"]`
- **THEN** the channels answer lists `prod` bound to `stable`, and the policy for `prod` names `ci` as the one signer that may ask

#### Scenario: Approvers on an environment that moves on its own

- **WHEN** `[environments.site]` names `channel = "stable"` and `approvers = ["ci"]`, and no `policy`
- **THEN** the channels are unreadable naming `[environments.site]` and `approvers`

#### Scenario: An unknown policy word

- **WHEN** `[environments.prod]` names `policy = "sometimes"`
- **THEN** the channels are unreadable naming `[environments.prod]` and `sometimes`

### Requirement: An environment's channel is named for it

The channel a manual or after environment is given SHALL take the
environment's name, in its component, and a line under it SHALL add
claims after the gate's acceptance (C.10). A channel so made that
shares a name with a channel the fragment holds, held or not, or with
another made channel, or a line under it naming another binding or
`on_each_publish`, SHALL be unreadable naming the table; so SHALL a
line under it naming `approvers`, saying that who may ask for it is
written on the environment, since the made channel takes its
environment's (C.10, as ruled at agentplot/blueprints 088430c).

#### Scenario: A manual environment named beta

- **WHEN** `[environments.beta]` names `channel = "stable"` and `policy = "manual"`
- **THEN** the channels are unreadable naming `[environments.beta]`

#### Scenario: A line that restates prod's binding

- **WHEN** the manual environment `prod` names `stable`, and `[channels.prod]` names `bound_to = "stable"` and a `require`
- **THEN** `prod` is listed bound to `stable`, and its policy requires the gate's acceptance and then the line's claims

#### Scenario: Approvers under a made channel

- **WHEN** the manual environment `prod` names `stable`, and `[channels.prod]` names `approvers = ["ci"]`
- **THEN** the channels are unreadable naming `[channels.prod]` and `approvers`, and saying who may ask for `prod` is written on `[environments.prod]`

#### Scenario: A manual environment named for a held given

- **WHEN** `[environments.lts]` names `channel = "stable"` and `policy = "manual"`, and the file writes no `[channels.lts]`
- **THEN** the channels are unreadable naming `[environments.lts]`

### Requirement: The table in force lays the fragment beneath the file

The profile SHALL read every answer from one table in force: the
repository's file as written, with the fragment's table of each channel
it holds laid beneath the repository's table of that channel in every
component, as `[channels.<name>]` where no component is declared and
`[channels.<component>.<name>]` where components are (C.10, 125). A key
the repository writes SHALL replace the fragment's; `on_each_publish`
and `bound_to` SHALL be replaced together where either is written, and
with them the fragment's `prereleases` unless the repository writes its
own (C.10). A fragment that does not parse, or holds a table other than
a channel's, SHALL make every answer unreadable naming the givens; the
shipped fragment SHALL parse.

#### Scenario: A line under one component's given

- **WHEN** the file declares `lib` and `web` and `[channels.lib.stable]` requires a rebuild from `verifier`
- **THEN** the policy for `stable` of `lib` requires the gate's acceptance and then the rebuild, and the policy for `stable` of `web` the gate's acceptance alone, both bound to their component's `beta`

#### Scenario: A line that restates how a given moves

- **WHEN** `[channels.stable]` names `bound_to = "beta"` and a `require`
- **THEN** `stable` is listed bound to `beta`, and its policy requires the gate's acceptance and then the line's claims

#### Scenario: A fragment that does not parse

- **WHEN** the profile is given a fragment that is not well formed, and the channels or any policy are asked
- **THEN** the answer is unreadable naming the givens, with the parser's words verbatim

#### Scenario: The shipped fragment

- **WHEN** the shipped fragment is read
- **THEN** it parses, holds only channels' tables, declares `beta` moving on each publish with releases alone and `stable` bound to `beta`, and holds `alpha` moving on each publish with prereleases and `lts` bound to `stable`, both marked held

### Requirement: The fragment marks what it holds back

The fragment SHALL mark a channel it holds until a repository switches
it on with `held = true`, a key of the fragment alone: such a channel
SHALL be in force wherever the repository writes its table, and nowhere
else (C.10). `held`, or `on`, written in the repository's file SHALL be
unreadable naming the table and the key, saying that writing a
channel's table is the switch and that nothing turns `beta` or `stable`
off.

#### Scenario: A held given switched on by an empty table

- **WHEN** the file declares no component and holds `[channels.lts]` with nothing under it
- **THEN** `lts` is listed bound to `stable`, its keys the fragment's

#### Scenario: held written by the repository

- **WHEN** `[channels.stable]` names `held = true`
- **THEN** the channels are unreadable naming `[channels.stable]` and `held`, and saying that nothing turns `stable` off

### Requirement: A file naming no gate names its one provenance signer as the gate

Where `[signers]` at the revision asked names no signer that may claim
`acceptance` and exactly one that may claim `provenance`, the profile
SHALL answer that signer as one that may claim `acceptance` too, in
every audience's claims but a layout's, whose given gate is for
publishing, not reading: the repository's gate, and the promoter of its
given and made channels, as the one host 108 admits (118, C.10). It
SHALL answer a file that names a signer that may claim `acceptance`, or
none that may claim `provenance`, as written, and write nothing into any
file.

#### Scenario: A laptop's policy

- **WHEN** the file at the revision lists `[signers.laptop]` with `kinds = ["provenance"]`, `[acceptance]` requires provenance by `laptop`, and no channel is written
- **THEN** the policy for acceptance names `laptop` with `provenance` and `acceptance`, and the policies for `beta` and `stable` each require acceptance by `laptop` and are promoted by `laptop`

#### Scenario: The gate accepts as one host

- **WHEN** the gate signing as `laptop` judges, under that file, a version whose provenance `laptop` made
- **THEN** the version is accepted, its acceptance claim saying one host offered and accepted, naming `laptop`, and `beta` moves; the gate read `laptop`'s reach from the profile's answer alone (core.7)

#### Scenario: A file that names a gate

- **WHEN** the file lists `[signers.laptop]` for provenance and `[signers.gate]` with `kinds = ["acceptance"]`
- **THEN** the policy for acceptance names `laptop` with `provenance` alone, and a gate signing as `laptop` over a version `laptop` offered refuses it as offered by the gate

### Requirement: Several publishers and no gate is refused in words

Where `[signers]` at the revision asked names no signer that may claim
`acceptance` and two or more that may claim `provenance`, the profile
SHALL answer the policy for acceptance, and the policy for each given or
made channel, refused, wanting acceptance by no signer, with its own
words `several publishers and no gate, name an accepting signer`, since
108 admits no host (58, C.10). The policy for `[layout]` and for a
channel declared by hand SHALL be answered as written, and every channel
SHALL still be listed.

#### Scenario: Two publishers and no gate

- **WHEN** the file lists `[signers.laptop]` and `[signers.desk]`, each with `kinds = ["provenance"]`, and no signer that may claim acceptance
- **THEN** the policy for acceptance, and for `beta` and `stable` with no line written, is refused wanting acceptance by no signer, carrying `several publishers and no gate, name an accepting signer` verbatim

#### Scenario: The gate accepts nothing

- **WHEN** a gate signing as `laptop` judges, under that file, a version whose provenance `laptop` made
- **THEN** nothing is accepted, the gate's answer carries the policy's refusal with those words, no channel moves, and the ledger is not written

#### Scenario: What else the file answers

- **WHEN** that file also has a `[layout]` table and `[channels.edge]` declared by hand
- **THEN** the policy for `[layout]` and for `edge` are answered as written, and the channels listed are `beta`, `edge` and `stable`

### Requirement: Each required claim is answered with the artifacts it is asked of

The profile SHALL answer each claim `[acceptance]` or a channel requires
with the artifacts it is asked of, read from its kind alone: a
provenance, an acceptance and a promotion of every artifact (7, 134); a
verdict, a test, a review, a rebuild and a carried claim of a built
artifact alone, since a rendered artifact has no build (134). The file
SHALL gain no key for it, and `[layout]`'s claims SHALL be asked of
every artifact.

#### Scenario: Acceptance requires two claims of two kinds

- **WHEN** `[acceptance]` requires provenance from `laptop`, then a verdict of the check `cli:test` from `box`
- **THEN** the acceptance policy answers the provenance asked of every artifact and the verdict asked of built artifacts alone, in that order

#### Scenario: A given channel's line

- **WHEN** `[channels.stable]` requires a rebuild from `box`
- **THEN** the policy for `stable` answers the gate's acceptance asked of every artifact, then the rebuild asked of built artifacts alone

#### Scenario: The one host's own given acceptance

- **WHEN** `[signers]` names one signer of kind `provenance` and none of `acceptance`, and the policy for `beta` is asked
- **THEN** it requires that signer's acceptance, asked of every artifact

#### Scenario: A layout's claims

- **WHEN** `[layout]` requires a verdict of `cli:test` from `box`
- **THEN** the layout policy answers that verdict asked of every artifact

#### Scenario: A key naming the artifacts

- **WHEN** an entry of `[acceptance]`'s `require` names a key besides `kind`, `check` and `signers`
- **THEN** the answer is unreadable, naming the file and the table, as for any other unknown key
