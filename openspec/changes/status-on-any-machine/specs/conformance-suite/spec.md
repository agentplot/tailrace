## ADDED Requirements

### Requirement: Channels in status over the fakes

The status group SHALL hold status's channels to 31, 36, 58 to 61, 63,
101 and 131 over the fakes, below its marker, each scenario asking the
landing, the policy and the keeper alone and writing nothing:

- every channel the policy declares, at its version or empty, with its
  chain (31, 101, 125, 131);
- a channel read as last read, with the store unreachable, nothing
  shown refused, playing S10 as far as status goes (36, 60, 61);
- one chain unreadable and one forked beside a channel shown, each
  carrying the keeper's words (58, 59, 105);
- everything kept deleted, the channels answered the same and the
  record the same once rebuilt, playing S12 for the channels (63).

#### Scenario: Every channel at its version or empty

- **WHEN** the fake policy declares `beta` and `stable` of the world's component and the fake keeper holds two promotions of `stable`
- **THEN** status names `beta` empty and `stable` at its head's version with both promotions newest first, and the store, signer and targets record no call

#### Scenario: S10 for status

- **WHEN** the fake store faults every operation, the fake keeper is made unreachable after one answer, and status is asked
- **THEN** each channel is shown as last read with its time and the keeper's words, no part is refused, and the store records no call

#### Scenario: One channel's answer beside another's

- **WHEN** the fake keeper's chain is faulted unreadable once with the words `connection reset` and another channel is forked
- **THEN** the first channel asked is unreadable carrying `connection reset`, the forked one refused naming both claims, and every other channel shown

#### Scenario: S12 for the channels

- **WHEN** status is asked, the host's record is deleted, and status is asked again
- **THEN** the channels' part of both answers is equal, and after a rebuild so is the record's

### Requirement: The page answers from the one core beside the command line

The surfaces group SHALL hold every surface to answer status's channels
as the core does, below its marker, for 36, 56, 57 and 132, and SHALL
run over the command line and over the served page, the page asked
for its JSON through the process's own handling of a request. A
surface's answer and the core's SHALL be equal value for value but for
the moment each asked.

#### Scenario: The command line and the page answer the channels

- **WHEN** status is asked through each surface with JSON output over the fakes, with a channel promoted
- **THEN** each answer equals the core's status answer over the same binding, value for value but for the moment asked, which lies between the moments before and after the asking
