## MODIFIED Requirements

### Requirement: A publish refuses a revision not landed, before anything else

A publish SHALL ask the landing seam whether the branch has landed the
revision before it reads the policy file, asks the build, packs, puts or
signs anything. A revision the branch has not landed SHALL be refused
naming the branch and the revision, and nothing SHALL be read or offered
(14, core.11). Where the landing answered not landed from the host's
copy as last fetched, the refusal SHALL also name when the copy was
fetched, carry the remote's words and say to ask again once the remote
answers (77). A revision the landing answered landed from such a copy
SHALL be published as one it answered landed from its remote.

#### Scenario: A revision the branch has not landed

- **WHEN** a revision the fake landing has not landed on the branch is published
- **THEN** the answer is refused naming the branch and the revision, and the fake build, projections, signer and store record no call

#### Scenario: The landing cannot be read

- **WHEN** the fake landing answers unreadable for the branch and revision
- **THEN** the publish answers that unreadable answer with the landing's words, and nothing is offered

#### Scenario: Not landed as last fetched

- **WHEN** the fake landing answers not landed from its copy as last fetched, with a time and the remote's words
- **THEN** the publish is refused naming the branch, the revision, that time and those words verbatim, saying to ask again, the command line exits as for any refusal, and the fake build, projections, signer and store record no call

#### Scenario: Landed as last fetched

- **WHEN** the fake landing answers landed from its copy as last fetched
- **THEN** the publish goes on as for a revision landed on the remote, and its answer equals that one's
