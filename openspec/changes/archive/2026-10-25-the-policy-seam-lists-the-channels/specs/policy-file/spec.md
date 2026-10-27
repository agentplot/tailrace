## ADDED Requirements

### Requirement: A channel's table names its component

A `[channels.<name>]` table SHALL name the component the channel belongs
to with `component`, by a name a `[[component]]` table declares, and
MAY leave it out where the repository declares one component, or none:
the channel then belongs to the one declared, or to the one named
`repository` (123, 125, 131). A table that names no component where
several are declared, or names one the file does not declare, SHALL be
unreadable naming the file, the revision, the table and the components
the file declares. The profile reads only the `name` of each
`[[component]]` table.

#### Scenario: One component has every channel

- **WHEN** the file declares no `[[component]]`, and `[channels.beta]` and `[channels.stable]` name no component
- **THEN** both channels belong to the component `repository`

#### Scenario: The one declared component

- **WHEN** the file declares one `[[component]]` named `lib`, and `[channels.beta]` names no component
- **THEN** `beta` belongs to `lib`

#### Scenario: A channel naming no component among several

- **WHEN** the file declares `lib` and `web`, and `[channels.beta]` names no component
- **THEN** the channels are unreadable, naming the file, the revision, `[channels.beta]`, `lib` and `web`, and so is the policy for `beta`

#### Scenario: A channel naming a component not declared

- **WHEN** `[channels.beta]` names `component = "app"` and the file declares no component `app`
- **THEN** the channels are unreadable, naming `[channels.beta]` and `app`

### Requirement: A channel is bound within its component

A `bound_to` SHALL name a channel the file declares, of the same
component; one that names another component's channel, or a channel
the file does not declare, SHALL be unreadable naming the table and the
channel it names (37, 125). A channel's component and binding SHALL be
read the same way whether the channels are listed or one channel's
policy is asked, so the two never disagree (83).

#### Scenario: A binding across components

- **WHEN** `[channels.stable]` of `lib` names `bound_to = "edge"`, and `[channels.edge]` belongs to `web`
- **THEN** the channels are unreadable, naming `[channels.stable]` and `edge`, and so is the policy for `stable`

#### Scenario: A binding to a channel not declared

- **WHEN** `[channels.stable]` names `bound_to = "beta"` and the file holds no `[channels.beta]`
- **THEN** the channels are unreadable, naming `[channels.stable]` and `beta`

### Requirement: The profile lists the channels the file declares

The profile SHALL answer channels with one entry for each
`[channels.<name>]` table of the file at the revision asked, ordered by
name, each with its component and its `bound_to` where it names one
(131). Where the revision holds no policy file, or a file with no
`[channels]`, the answer SHALL be an empty list. A revision the
repository does not hold, or a file that cannot be read, SHALL answer
as policy-for answers it (58, 59).

#### Scenario: Two components' channels

- **WHEN** the file declares `lib` and `web`, `[channels.beta]` and `[channels.stable]` of `lib` with `stable` bound to `beta`, and `[channels.edge]` of `web`
- **THEN** the channels answer `beta` of `lib`, `edge` of `web` and `stable` of `lib` bound to `beta`, in that order

#### Scenario: No channel declared

- **WHEN** the file at the revision holds `[acceptance]` and no `[channels]`, or the revision holds no policy file
- **THEN** the channels answer an empty list

#### Scenario: A checkout's channel is not read

- **WHEN** the working tree's policy file declares a channel the file at the revision does not
- **THEN** the channels at that revision do not list it (65, core.22)
