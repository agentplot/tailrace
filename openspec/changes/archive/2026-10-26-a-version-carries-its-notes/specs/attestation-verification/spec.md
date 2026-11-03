## ADDED Requirements

### Requirement: Verify shows the notes the index names

Where the version's index names an artifact of ecosystem `notes`, verify
SHALL read its bytes from the store by the digest the index names and
show them beside the version, with the notes' digest; the notes' claims
are told with every other artifact's. An index that names no notes SHALL
show none, and notes whose bytes are not text SHALL be answered
unreadable naming the digest (3, 58, 134, core.4).

#### Scenario: A published version's notes

- **WHEN** a published version is verified, as text and with `--json`
- **THEN** the answer names the notes' digest and their text, equal to the bytes the store holds, the text answer prints them after the claims, and the notes' provenance claim is told by the host's signer

#### Scenario: A version with no notes

- **WHEN** a version whose index names no `notes` artifact is verified
- **THEN** the answer shows no notes, and every claim is told as before
