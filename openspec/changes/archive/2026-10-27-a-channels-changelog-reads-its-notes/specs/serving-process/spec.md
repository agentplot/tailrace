## MODIFIED Requirements

### Requirement: A repository's page and its JSON are its status answer

The process SHALL answer `GET /<owner>/<name>/status.json` with the
status answer the core gives in the named checkout, as `tailrace
status --json` prints it there, and `GET /<owner>/<name>/status` with a page
rendering that answer whole: each channel with its component, how it
moves, its version or that it is empty, its chain with each promotion's
asker and the notes of the version it moved to beneath it, the
environments' part, and the record of what this host published with each
version's notes beneath it, every refusal and unreadable answer shown
with the source's own words (57, 58, 59, 132, 134). Each request SHALL
compute the answer anew, and SHALL write nothing and ask no store,
signer or target (60). A path the process does not answer SHALL be 404,
and a method other than `GET` 405.

#### Scenario: The page and the JSON agree

- **WHEN** the page and the JSON of one named repository are asked
- **THEN** the JSON is the status answer in that checkout, and the page names every channel, version and promotion the JSON names, and the environments' answer

#### Scenario: A promotion after the page was read

- **WHEN** a channel is promoted after the page was asked, and the page is asked again
- **THEN** the second page shows the new version, with no restart and no timer between

#### Scenario: The changelog beside the channel

- **WHEN** a channel was promoted to two versions and the repository's page is asked
- **THEN** beneath each promotion the page shows who asked and the notes of the version it moved to, escaped, the same text the JSON carries; and a version whose notes are unreadable shows the unreadable answer with the source's words in their place

#### Scenario: The notes of what this host published

- **WHEN** this host published a version and the repository's page is asked
- **THEN** the page shows that version's notes beneath its row in the record, the same text the JSON carries
