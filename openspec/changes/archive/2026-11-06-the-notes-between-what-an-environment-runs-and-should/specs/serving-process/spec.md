## MODIFIED Requirements

### Requirement: A repository's page and its JSON are its status answer

The process SHALL answer `GET /<owner>/<name>/status.json` with the
status answer the core gives in the named checkout, as `timber
status --json` prints it there, and `GET /<owner>/<name>/status` with a page
rendering that answer whole: each channel with its component, how it
moves, its version or that it is empty, its chain with each promotion's
asker and the notes of the version it moved to beneath it; the
environments' four counts and every environment with its state, what it
should run, what it runs and when it converged, what it waits on, and
beside one that is behind the notes of every version between beneath
it; and the record of what this host published with each version's
notes beneath it, every refusal and unreadable answer shown with the
source's own words (57, 58, 59, 98, 132, 134). Each request SHALL
compute the answer anew, and SHALL write nothing but the host's copy of
a chain a store ledger holds and of each environment's reports, reach
the store only to fetch them, and ask no signer or target (60). A path
the process does not answer SHALL be 404, and a method other than `GET`
405.

#### Scenario: The page and the JSON agree

- **WHEN** the page and the JSON of one named repository are asked
- **THEN** the JSON is the status answer in that checkout, and the page names every channel, version and promotion the JSON names, and every environment with its state

#### Scenario: A promotion after the page was read

- **WHEN** a channel is promoted after the page was asked, and the page is asked again
- **THEN** the second page shows the new version, with no restart and no timer between

#### Scenario: The changelog beside the channel

- **WHEN** a channel was promoted to two versions and the repository's page is asked
- **THEN** beneath each promotion the page shows who asked and the notes of the version it moved to, escaped, the same text the JSON carries; and a version whose notes are unreadable shows the unreadable answer with the source's words in their place

#### Scenario: The notes of what this host published

- **WHEN** this host published a version and the repository's page is asked
- **THEN** the page shows that version's notes beneath its row in the record, the same text the JSON carries

#### Scenario: The notes between beside an environment that is behind

- **WHEN** an environment runs 0.2.0 and its channel is at 0.4.0, and the repository's page is asked
- **THEN** beneath the environment the page shows 0.3.0 and 0.4.0 with their notes, escaped, the same text the JSON carries, and the page holds no script

### Requirement: The root answers every repository it knows together

The process SHALL answer `GET /status.json` with every repository the
host's file names, ordered by owner and name, each as the core's row of
its status answer in its checkout, computed anew on the request, and
`GET /` with a page rendering those rows: each repository linked to its
own page, every channel's head, and the environments' four counts or
the answer given in their place (132). An entry whose checkout is gone
or whose status cannot be asked SHALL be its own row's unreadable
answer, naming the path, and every other row SHALL be shown (58). A
host whose file names no repository SHALL answer an empty list, and a
file that cannot be read SHALL be the root's unreadable answer naming
the file with the parser's words. The root SHALL write nothing but the
host's copy each status writes, and ask no signer or target (60).

#### Scenario: Two repositories together

- **WHEN** two checkouts of two repositories have each named theirs, and `/status.json` and `/` are asked
- **THEN** the JSON holds two rows, each equal to the core's row of that repository's status but for the moment asked, and the page names both repositories, every channel's head and each row's four counts

#### Scenario: One checkout gone

- **WHEN** one of two named repositories' checkouts has been removed
- **THEN** its row is unreadable naming the path, and the other row is shown

#### Scenario: Nothing named yet

- **WHEN** the host's file is absent and `/status.json` is asked
- **THEN** the answer is an empty list of repositories, status 200
