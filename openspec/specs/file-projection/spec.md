# file-projection Specification

## Purpose
The projection for a file: an output the build left as one file, packed
as that file's own bytes, so the bytes the checks passed are the bytes
offered (C.11).

## Requirements

### Requirement: A file output packs as its own bytes

pack SHALL answer the bytes of the output's one file unchanged, with
metadata naming the ecosystem `file`, the output's name and the version
given (84, 4). It runs no packager and opens no archive (C.11).

#### Scenario: One file packed

- **WHEN** an output named `site` whose one file holds the bytes `abc` is packed at version `1.4.0`
- **THEN** pack answers the bytes `abc` and metadata naming `file`, `site` and `1.4.0`

### Requirement: The version is never in the bytes

The same output SHALL pack to the same bytes under any version, and
pack SHALL promise the same output and version pack to the same bytes
again (87, 24, 18).

#### Scenario: Two versions, one set of bytes

- **WHEN** one output is packed at `1.4.0` and again at `1.5.0-pre.1`
- **THEN** both answers hold the same bytes, differ only in the version their metadata names, and each says it packs the same again

### Requirement: An output that is not one file is refused

An output whose build left no file or more than one SHALL be refused,
naming the output and the files the build reported. A file that cannot
be read SHALL be unreadable naming it, with the operating system's own
words (58, 59).

#### Scenario: A directory of files

- **WHEN** an output's report names three files
- **THEN** pack is refused, naming the output and the three files

#### Scenario: A file gone

- **WHEN** the output's one file is removed before pack
- **THEN** pack is unreadable, naming the file, with the operating system's words verbatim

### Requirement: Render and serve are not built

render and serve SHALL answer absent, naming the operation and the
projection, until a later unit builds them.

#### Scenario: render asked

- **WHEN** render is asked of two file artifacts
- **THEN** it answers absent naming render of the file projection, and writes nothing

### Requirement: The projection passes the projection group's pack scenarios

The profile SHALL pass the projection group's scenarios of pack: packed
the same again, and a projection that cannot be reached (84, 87, 58,
59).

#### Scenario: The pack scenarios over the file projection

- **WHEN** those scenarios run against the profile over a file in a temporary directory
- **THEN** each passes
