## MODIFIED Requirements

### Requirement: Ensure is one run of the declared tasks at the revision

Ensure SHALL run moon once over the tasks of every output asked, in a
checkout holding exactly the revision asked and nothing of any other
checkout but moon's outputs directory, which it shares with the
checkout the build is bound over, as every checkout on a host shares it,
continuing past a task that fails, and replaying what moon has cached
(74, C.3). It SHALL answer the outputs present, or refused naming every
output whose task declared files the run left absent or whose task did
not run. tailrace SHALL never run an output's command itself (15,
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

## ADDED Requirements

### Requirement: Left reads moon's run report and outputs directory

Left SHALL answer, for a checkout, each task moon's run report in that
checkout records with a hash whose archive moon's outputs directory
holds, grouped by hash: the hash as the build identity, the tasks with
that hash, a pass, and the files the tasks declare that are there; it
SHALL read nothing of tailrace's and run no task (139, C.3). A checkout
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
