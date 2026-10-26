# Spec Delta

## MODIFIED Requirements

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
the audience asked, the answer SHALL be absent naming the audience (82,
58), except for a channel the givens declare, a held one the
repository has switched on, or one an environment's words make, whose
policy SHALL be answered as given (C.10). A file that cannot be
read or parsed SHALL be unreadable naming the file and the revision,
with git's or the parser's own words (59).

#### Scenario: No layout policy

- **WHEN** the file holds `[acceptance]` and no `[layout]`
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

## ADDED Requirements

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
