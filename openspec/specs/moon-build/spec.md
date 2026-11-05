# moon-build Specification

## Purpose
The build seam bound to moon (C.3): which moon task makes each declared
output, how one asking runs them, and how the task hash becomes the
build identity a landing's verdicts and timber's provenance share.

## Requirements

### Requirement: The repository names the task that makes each output

The moon build SHALL take the moon task that makes each output from
the repository's policy file at the revision asked about, never from a
checkout, where each output is named with the task's target (65, 92,
C.3). An output the file names no task for SHALL be refused by ensure
as one the build could not make, and answered absent by report.

#### Scenario: An output and its task

- **WHEN** the policy file at a revision names the task `web:build` for the output `site`, and ensure is asked for `site` at that revision
- **THEN** moon runs `web:build`, and report of `site` answers that task's hash

#### Scenario: An output with no task

- **WHEN** ensure is asked for an output the policy file at the revision names no task for
- **THEN** ensure is refused naming that output among those the build could not make

#### Scenario: A checkout's edit is not read

- **WHEN** the publisher's checkout changes the task named for an output, uncommitted, and ensure is asked of the landed revision
- **THEN** moon runs the task the policy file at the revision names

### Requirement: Ensure is one run of the declared tasks at the revision

Ensure SHALL run moon once over the tasks of every output asked, in a
checkout holding exactly the revision asked and nothing of any other
checkout but moon's outputs directory, which it shares with the
checkout the build is bound over, as every checkout on a host shares it,
continuing past a task that fails, and replaying what moon has cached
(74, C.3). It SHALL answer the outputs present, or refused naming every
output whose task declared files the run left absent or whose task did
not run. timber SHALL never run an output's command itself (15,
core.10).

#### Scenario: Three outputs in one asking

- **WHEN** ensure is asked for three outputs whose tasks pass, pass and fail, each leaving its declared files
- **THEN** moon is run once over the three tasks, and ensure answers the outputs present

#### Scenario: Outputs the build could not make

- **WHEN** ensure is asked for three outputs of which two tasks leave none of their declared files
- **THEN** ensure is refused naming exactly those two outputs, after the third task has run

#### Scenario: The same revision ensured again

- **WHEN** ensure is asked again for outputs already made at the revision
- **THEN** moon answers each task from its cache, and report answers the same build identity as before

#### Scenario: An archive placed in the bound checkout is replayed

- **WHEN** the archive of an output's task hash, made on another host, is placed in the outputs directory of the checkout the build is bound over, and ensure is asked for that output at the revision in a repository where the task has never run
- **THEN** moon answers the task from its cache without running its command, and the output's declared files hold the bytes the other host's build wrote

### Requirement: Report reads moon's own record

Report SHALL answer, for an output at a revision, the hash moon gave
its task as the build identity, a pass when moon ran the task to
success or answered it from its cache and a fail when the task failed,
and the paths of the files the task declares as its outputs, read from
what moon recorded of the task in the revision's checkout (75, C.3).
An output whose task moon has no record of at the revision SHALL be
answered absent naming the output.

#### Scenario: A failed task's report

- **WHEN** a task fails after writing its declared files, and its output is reported
- **THEN** report answers the task's hash, a fail, and where the files are

#### Scenario: Nothing ensured

- **WHEN** report is asked of an output at a revision no ensure has run moon for
- **THEN** the answer is absent naming the output

### Requirement: One build identity on every host

The same revision SHALL give the same build identity for an output on
any host and in any checkout, and the build identity SHALL be the key
sluice's verdicts carry for the same task, so a verdict names the build
whose outputs are offered (13, 76, C.3).

#### Scenario: Two checkouts

- **WHEN** the same revision is ensured in two clones of one repository in two directories
- **THEN** both report the same build identity for each output

#### Scenario: The landing's key is the build's identity

- **WHEN** sluice lands a revision whose check is the task that makes an output, and the moon build ensures and reports that output at the landed revision
- **THEN** the build identity report answers is the key sluice's verdict for that check carries

### Requirement: A build that cannot be run is unreadable

Where moon or the repository cannot be reached, every operation SHALL
answer unreadable naming the build, carrying the words the tool or the
operating system gave, verbatim, and never refused or absent (58, 59,
core.19).

#### Scenario: moon is not there

- **WHEN** the program the build runs as moon does not exist, and ensure and report are asked
- **THEN** both answer unreadable carrying the operating system's words unchanged

### Requirement: moon's build group passes

The conformance suite's build group SHALL pass over the moon build in
a scratch repository, every scenario on a repository of its own (93,
core.33).

#### Scenario: The build group over moon

- **WHEN** the suite's build group runs over a rig binding the moon build to a scratch repository
- **THEN** the report names the profile `moon` and every scenario passed

### Requirement: Left reads moon's run report and outputs directory

Left SHALL answer, for a checkout, each task moon's run report in that
checkout records with a hash whose archive moon's outputs directory
holds, grouped by hash: the hash as the build identity, the tasks with
that hash, a pass, and the files the tasks declare that are there; it
SHALL read nothing of timber's and run no task (139, C.3). A checkout
with no run report SHALL answer an empty list.

#### Scenario: A run that passed one task and failed another

- **WHEN** moon runs two tasks in a checkout, one passing with declared outputs and one failing, and left is asked of that checkout
- **THEN** left names the passing task's hash with that task, a pass and its declared files, and does not name the failing task, whose archive moon did not write

#### Scenario: A task with no declared outputs

- **WHEN** moon runs a task that passes and declares no outputs, and left is asked
- **THEN** left names its hash with the task, a pass and no files, since moon archives its logs

#### Scenario: A checkout that ran nothing

- **WHEN** left is asked of a checkout with no run report
- **THEN** it answers an empty list and runs no moon task

#### Scenario: An archive removed after the run

- **WHEN** a task passes and its archive is removed from the outputs directory before left is asked
- **THEN** left does not name its hash
