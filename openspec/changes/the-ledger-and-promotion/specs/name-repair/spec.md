## ADDED Requirements

### Requirement: A chain read as last read moves no name

Moving a channel's name SHALL refuse to act on a chain the ledger
answered as last read: the answer SHALL be unreadable naming the
channel's chain and carrying the ledger's words, and the store SHALL not
be written (101, 106, core.4).

#### Scenario: The ledger unreachable after a read

- **WHEN** the channel's chain is answered as last read because the ledger cannot be reached
- **THEN** the answer is unreadable with the ledger's words, and the store is asked nothing
