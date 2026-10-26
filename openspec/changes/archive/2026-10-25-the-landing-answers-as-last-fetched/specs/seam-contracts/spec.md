## MODIFIED Requirements

### Requirement: The landing's contract

The landing SHALL answer landed with whether a branch has landed a
revision (77), and verdicts-of with every verdict recorded for a
revision, each naming its check, its build identity, its outcome and
its signer, with its signed envelope as the landing recorded it, and
where the landing found it, in the landing's own words (12, 78). Each
answer SHALL say how it was read: from the landing's remote, or the
repository itself where no remote is bound, or from the host's copy as
last fetched, because the remote did not answer, with when that copy
was fetched and the remote's own words of why it did not answer, the
two ways a chain says it was read (59, 60, 61, 77, C.5, C.6).

#### Scenario: A revision with no verdicts

- **WHEN** verdicts-of is asked for a revision the landing recorded no verdict for
- **THEN** it answers an empty set of verdicts, which is not absent and not unreadable

#### Scenario: Where a verdict was found

- **WHEN** verdicts-of answers a verdict
- **THEN** the verdict says where the landing found it

#### Scenario: An answer read from the remote

- **WHEN** landed or verdicts-of is answered while the landing's remote answers
- **THEN** the answer says it was read from the landing itself

#### Scenario: An answer read as last fetched

- **WHEN** landed or verdicts-of is answered while the landing's remote does not answer, from a copy the host fetched before
- **THEN** the answer says it was read from the copy, with the time the copy was fetched and the remote's words verbatim, and it is neither refused nor unreadable

## ADDED Requirements

### Requirement: Not landed as last fetched is a refusal of its own

A revision the landing answers not landed from the host's copy SHALL
be refused naming the branch, the revision and the time the copy was
fetched, carrying the remote's words verbatim beside tailrace's and
saying to ask again once the remote answers; it SHALL be told apart
from a revision the landing answers not landed from its remote, in the
JSON form as in the text (14, 57, 58, 59, 77).

#### Scenario: The two refusals told apart

- **WHEN** one revision is refused as not landed by a landing that answered from its remote, and another by a landing that answered from its copy as last fetched
- **THEN** the second's refusal names the copy's time and the remote's words and says to ask again, the first's names neither, and both read back from JSON as they were made

#### Scenario: A yes from the copy

- **WHEN** the landing answers landed from its copy as last fetched
- **THEN** the revision is landed, and the landed revision says it was read from the copy and when
