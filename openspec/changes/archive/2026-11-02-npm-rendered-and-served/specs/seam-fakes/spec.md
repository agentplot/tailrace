## ADDED Requirements

### Requirement: The fake projection serves its layouts
The fake projection SHALL serve a layout it rendered: a request whose
path names an ecosystem name and version the layout holds is answered
`200` with that artifact's metadata as its body; one naming a version the
layout lacks asks the core once and answers `200` where the core's answer
is held, and otherwise a status of `404`, `403` or `502` for absent,
refused or unreadable, with the answer's words as its body. It SHALL
record every request and every version it asked of the core, and a test
SHALL be able to script it to serve none, after which every request is
absent naming serving (86, 115).

#### Scenario: A version held and a version lacked
- **WHEN** the fake projection renders one version, then serves a request for it and a request for a second version the core answers refused
- **THEN** the first answer is `200` and asked the core nothing, the second is `403` carrying the refusal's words, and the test reads back both requests and the one version asked

#### Scenario: The fake scripted to serve none
- **WHEN** a test scripts the fake projection to serve none and asks it to serve a request
- **THEN** the answer is absent naming serving, and the core is asked nothing
