# landed-policy Specification

## Purpose
What tailrace reads of a repository at a revision its branch has landed:
the authors' components and version rule from the policy file at that
revision, and the policy for an audience, never from a checkout.

## Requirements

### Requirement: Nothing is read at a revision the branch has not landed

The core SHALL ask the landing seam whether the branch has landed the
revision before it reads anything of the repository at that revision.
When the branch has not landed it, the answer SHALL be a refusal naming
the branch and the revision, and nothing else SHALL be read (14, 65, 77).

#### Scenario: A revision the branch has not landed

- **WHEN** the core is asked to read the policy at a revision the landing seam says the branch has not landed
- **THEN** the answer is refused, naming the branch and the revision, and no file, commit, tag or policy is read

#### Scenario: The landing seam cannot be read

- **WHEN** the landing seam answers unreadable for a branch and revision
- **THEN** the answer is that same unreadable answer, carrying the landing's own words, and never a refusal or an absence

### Requirement: The policy file is read at the landed revision, never from a checkout

The policy file SHALL be read from the repository's history at the
landed revision and never from a working tree, so the file a checkout
holds, edited or not, changes nothing (65, C.10, core.22).

#### Scenario: A checkout's edit is not read

- **WHEN** a checkout's working tree holds a policy file that differs from the one at the landed revision
- **THEN** the components and rule read are those of the file at the landed revision

#### Scenario: A file committed after the revision is not read

- **WHEN** the branch holds a later commit that changes the policy file
- **THEN** the components and rule read at the earlier landed revision are those of the file at that revision

### Requirement: The authors declare components

Each component the policy file declares SHALL carry its name, its
outputs, the paths whose changes version it and the prefix its tags
carry (123). A file that gives two components one name or one prefix
SHALL be refused naming both components.

#### Scenario: Two components declared

- **WHEN** the policy file at the landed revision declares two components with their outputs, paths and prefixes
- **THEN** both are read, each with its own outputs, paths and prefix

#### Scenario: Two components with one prefix

- **WHEN** the policy file declares two components that carry the same tag prefix
- **THEN** the answer is refused, naming both components and the prefix

### Requirement: A repository that declares no component is one component

When the repository holds no policy file at the landed revision, or its
file declares no component, the repository SHALL be one component whose
paths are the whole repository and whose prefix is the rule's default
(123).

#### Scenario: No policy file

- **WHEN** the landed revision holds no policy file
- **THEN** the repository is one component over every path, with the default prefix and the default rule

#### Scenario: A file with no component

- **WHEN** the policy file at the landed revision sets the version rule and declares no component
- **THEN** the repository is one component over every path, under that rule

### Requirement: A policy file that cannot be read is unreadable

A policy file at the landed revision that cannot be parsed SHALL be
answered unreadable, naming the file and the revision and carrying the
parser's own words, and never treated as absent or as one component
(58, 59, core.19).

#### Scenario: A malformed file

- **WHEN** the policy file at the landed revision is not well formed
- **THEN** the answer is unreadable, naming the file and the revision, with the parser's words verbatim

### Requirement: The policy for an audience is asked only at a landed revision

The core SHALL ask the policy seam for an audience's requirements only
for a revision the landing seam has said is landed, and SHALL answer
what the policy seam answers (65, 82, 83).

#### Scenario: The policy asked after the landing

- **WHEN** the core is asked the acceptance policy at a revision the branch has landed
- **THEN** the landing seam is asked first, the policy seam is asked for acceptance at that revision, and its answer is returned as given

#### Scenario: The policy not asked before the landing

- **WHEN** the core is asked a policy at a revision the branch has not landed
- **THEN** the policy seam is never asked
