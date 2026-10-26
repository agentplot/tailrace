# Spec Delta

## MODIFIED Requirements

### Requirement: Each audience lists the claims it requires, in order

`[acceptance]` and `[layout]` SHALL each hold `require`, a list of
claims in the order the verdict reads them, each with its `kind`, its
`check` where it is a verdict, and the `signers` by their names in
`[signers]` (82). A channel's table SHALL hold such a list in `require`
or leave it out, requiring nothing of its own, and MAY name `bound_to`
or `on_each_publish` (83, 37, 95). The acceptance and layout tables
SHALL name no channel.

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
58), except for a channel the file gives or an environment's words make,
whose policy SHALL be answered as given (C.10). A file that cannot be
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
- **THEN** it answers the gate's acceptance as the one claim required, `beta` as the channel it is bound to, and the gate's signers as its promoters

#### Scenario: A channel neither given nor declared

- **WHEN** the policy for `edge` is asked and the file neither declares nor makes it
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

A `bound_to` SHALL name a channel of the same component that the file
declares, gives or makes; one that names a channel of no such kind
SHALL be unreadable naming the table and the channel it names (37,
125). A channel's component and how it moves SHALL be read the same way
whether the channels are listed or one channel's policy is asked, so
the two never disagree (83).

#### Scenario: A binding across components

- **WHEN** `[channels.lib.next]` names `bound_to = "edge"`, and only `[channels.web.edge]` declares an `edge`
- **THEN** the channels are unreadable, naming `[channels.lib.next]` and `edge`, and so is the policy for `next` of `lib`

#### Scenario: A binding to a channel not declared

- **WHEN** `[channels.next]` names `bound_to = "edge"` and the file neither declares, gives nor makes an `edge`
- **THEN** the channels are unreadable, naming `[channels.next]` and `edge`

#### Scenario: A binding to a given channel

- **WHEN** `[channels.next]` names `bound_to = "beta"` and the file writes no line under `beta`
- **THEN** `next` is bound to the given `beta`

### Requirement: The profile lists the channels the file declares

The profile SHALL answer channels with `beta` and `stable` of every
component, each channel an environment's words make, and each channel
table the file holds, once each, ordered by name and then component,
each with its component and how it moves (131, C.10). Where the
revision holds no policy file the answer SHALL be `beta` and `stable`
of the one component. A revision the repository does not hold, or a
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

- **WHEN** `[channels.lts]` names only `require`
- **THEN** `lts` is listed moving by a promotion alone

### Requirement: The given channels' policy names the gate

The policy for `beta`, `stable` or a channel an environment's words
make SHALL require, first, an acceptance by the signers `[signers]`
lets claim acceptance, then any `require` a line under it names, in
order; and SHALL name those same signers, the gate, as its promoters
whatever the line names (32, C.10). A hand-declared channel's promoters
SHALL be the signers its `require` names that may claim promotion.

#### Scenario: beta with no line

- **WHEN** `[signers.gate]` names kinds `acceptance` and `promotion`, `[signers.laptop]` kinds `provenance`, and no `[channels]`
- **THEN** the policy for `beta` requires an acceptance by `gate` and names `gate` as its one promoter

#### Scenario: A hand-declared channel's promoters

- **WHEN** `[channels.lts]` requires an acceptance from `gate`, and `[signers.release]` with kinds `promotion` is named in its `require`
- **THEN** the policy for `lts` names `release` as its promoter

### Requirement: The environments' words are read

An environment SHALL be declared as `[environments.<name>]` where no
component is declared and `[environments.<component>.<name>]` where
components are, naming its `channel`, `policy` (`auto` or `manual`,
`auto` where unwritten) and `after`, a list of environment names of the
same component (C.10). Any other `policy`, or a `channel` the file does
not declare, give or make, SHALL be unreadable naming the table. Its
other keys SHALL be left to their readers.

#### Scenario: An environment with a target's own key

- **WHEN** `[environments.site]` names `channel = "stable"` and `bucket = "example-site"`
- **THEN** the channels answer is read, `site` makes no channel, and `bucket` is not read

#### Scenario: An unknown policy word

- **WHEN** `[environments.prod]` names `policy = "sometimes"`
- **THEN** the channels are unreadable naming `[environments.prod]` and `sometimes`

### Requirement: An environment's channel is named for it

The channel a manual or after environment is given SHALL take the
environment's name, in its component, and a line under it SHALL be read
as a line under a given channel (C.10). A channel so made that shares a
name with a given channel, or a line under it naming another binding or
`on_each_publish`, SHALL be unreadable naming the table.

#### Scenario: A manual environment named beta

- **WHEN** `[environments.beta]` names `channel = "stable"` and `policy = "manual"`
- **THEN** the channels are unreadable naming `[environments.beta]`

#### Scenario: A line that restates prod's binding

- **WHEN** the manual environment `prod` names `stable`, and `[channels.prod]` names `bound_to = "stable"` and a `require`
- **THEN** `prod` is listed bound to `stable`, and its policy requires the gate's acceptance and then the line's claims
