## ADDED Requirements

### Requirement: The projection's group holds serve
The projection's group SHALL hold a projection to serve, below its
marker, over the rig's own request for a version, which the rig names
since a request's form is the ecosystem's: a version the layout holds is
answered and asks the core nothing (86, 46); a version it lacks is asked
of the core once, and answered once the core renders it (115, 50); a
refusal the core answers is carried in the response in its own words
(99, 59); and one layout served under one base from two places answers
the same and names neither place (49). Each scenario SHALL be not
applicable to a projection whose clients read files, which SHALL answer
absent naming serving. The group's scenario "served where the clients
read no files" SHALL ask serve for the rig's request of a version the
layout holds, in place of an address, citing 86 as before.

#### Scenario: A version held, served
- **WHEN** a projection that serves renders two of the rig's artifacts and is asked to serve the rig's request for one
- **THEN** the response answers it, and the core was asked nothing

#### Scenario: A version lacked, asked of the core
- **WHEN** a projection that serves renders one of two artifacts and is asked for the other, the core rendering it on the ask
- **THEN** the core was asked once for that version's ecosystem name and version, and the response answers it

#### Scenario: A refusal in the response
- **WHEN** a projection that serves is asked for a version it lacks and the core answers it refused with the words `acceptance wanting`
- **THEN** the response is no answer of the version and carries `acceptance wanting` verbatim

#### Scenario: One base, two places
- **WHEN** one set of artifacts is rendered at two places and the same request is served from each under one base
- **THEN** the two responses are the same, and neither names either place

#### Scenario: Every projection runs it
- **WHEN** the projection group runs over the fake projection, the npm projection, and the file, image and moon cache profiles
- **THEN** the serve scenarios pass over the fake and npm, and are not applicable to the three profiles, each of which answers absent naming serving

### Requirement: The npm projection runs the projection group
The npm projection SHALL run the projection group whole: its render,
serve, pinned and unreachable scenarios over tarballs a test makes, in
`check-fast`, with no npm on the host, and its pack scenarios with the
npm the tree pins, in `check-tools` (93, core.33).

#### Scenario: The group over npm
- **WHEN** the conformance tests of the npm projection run in both layers
- **THEN** every scenario of the projection group passes or is not applicable with its reason, and none is left out

### Requirement: The layouts' group holds a render on a miss
The layouts' group SHALL hold the core to a render on a miss, below its
marker, over the fakes: a version the policy admits is added to the
layout and the record and only it is read; a refused one is answered
naming the claim and changes nothing; and the served layout's address is
the core's name and port with the repository in the path (115, 50, 47,
99, 114, 49). 46, 49, 114 and 115 SHALL leave the register in the commit
whose scenario first cites them.

#### Scenario: A miss over the fakes
- **WHEN** a layout rendered holding one version is asked for a second the policy admits
- **THEN** the projection is asked to render both versions' artifacts, the record names both, and the fake store was asked about the second alone

#### Scenario: A refused miss over the fakes
- **WHEN** a layout is asked for a version whose index lacks the claim the policy requires
- **THEN** the answer names that claim as wanting, and the projection was asked to render nothing

#### Scenario: The served address over the fakes
- **WHEN** the served address is asked for two remotes of one repository
- **THEN** both answer one address at port 7420 with the owner and name in the path

#### Scenario: The register after serving
- **WHEN** the coverage check runs after these scenarios are added
- **THEN** 46, 49, 114 and 115 are cited and none of them is in the register
