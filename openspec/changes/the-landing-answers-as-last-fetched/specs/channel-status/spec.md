## ADDED Requirements

### Requirement: Status says when the landing was fetched

The channels' part of status SHALL say how the landing of the branch's
head was read: from the landing itself, or from the host's copy as last
fetched, with when it was fetched, as seconds since the Unix epoch and
as a person reads a time in UTC, and the remote's words verbatim; the
channels SHALL then be shown as for a landing read from its remote, on
the command line, in its JSON and on the host's page alike (36, 56, 57,
60, 77).

#### Scenario: The landing's remote cannot be reached

- **WHEN** the landing answers the branch's head landed from its copy as last fetched, with a time and the remote's words
- **THEN** status shows every channel the policy declares, says the landing was read as last fetched at that time in UTC with those words verbatim, shows nothing refused, and exits 0

#### Scenario: Cut off from both remotes

- **WHEN** a machine that read the landing and the ledger from one remote is cut off from it, and asks status
- **THEN** every channel is shown at the version of its chain as last read, the landing and each chain saying when they were fetched, and the page and `--json` give the same answer as the command line

#### Scenario: The head not landed as last fetched

- **WHEN** the landing answers the branch's head not landed from its copy
- **THEN** the channels' part is refused naming the copy's time and the remote's words, saying to ask again, and the record's part answers
