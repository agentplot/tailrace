## ADDED Requirements

### Requirement: The fake keeper answers as last read

The fake keeper SHALL keep, per channel, the chain as it last answered
it, and once made unreachable SHALL answer a channel it has answered
with that chain, the time it answered and the words it was made
unreachable with, and a channel it never answered unreadable with those
words; append SHALL then be unreadable (C.13, C.14). Every chain it
answers while reachable SHALL say it was read from the keeper itself.

#### Scenario: Unreachable after a read

- **WHEN** the fake keeper answers `stable`'s chain, is made unreachable with the words `ref unreadable`, and is asked again
- **THEN** it answers the same claims, the time of the first answer and `ref unreadable`, and an append is unreadable with those words
