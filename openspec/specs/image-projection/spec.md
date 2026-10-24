# image-projection Specification

## Purpose
The projection for an image: an output the build left as an OCI image
archive, packed as the archive's own bytes, the image as it would be
pushed (C.11).

## Requirements

### Requirement: An image archive packs as its own bytes

pack SHALL answer the bytes of the output's one file unchanged when it
is a tar archive holding an OCI image layout at its root, an
`oci-layout` file and an `index.json`, with metadata naming the
ecosystem `image`, the output's name and the version given (84, 4). It
runs no packager and opens no archive to change it (C.11).

#### Scenario: An archive as docker save writes one

- **WHEN** an output named `api` whose one file is a tar holding `oci-layout`, `index.json` and `blobs/sha256/…` is packed at `0.3.0`
- **THEN** pack answers that file's bytes unchanged and metadata naming `image`, `api` and `0.3.0`

### Requirement: The version is never in the bytes

The same output SHALL pack to the same bytes under any version, and
pack SHALL promise the same output and version pack to the same bytes
again (87, 24, 18).

#### Scenario: Two versions, one set of bytes

- **WHEN** one image output is packed at `0.3.0` and again at `0.4.0`
- **THEN** both answers hold the same bytes and each says it packs the same again

### Requirement: An output that is not an image archive is refused

An output whose build left other than one file, or whose file is not a
tar holding `oci-layout` and `index.json` at its root, SHALL be refused
naming the output and what it holds instead. A file that cannot be read
SHALL be unreadable naming it, with the operating system's words (58,
59).

#### Scenario: A compressed archive

- **WHEN** the output's file is a gzip-compressed tar
- **THEN** pack is refused, naming the output and saying the file is not a tar holding an OCI image layout

#### Scenario: A legacy docker archive

- **WHEN** the output's file is a tar holding `manifest.json` and no `oci-layout`
- **THEN** pack is refused, naming the output and the missing `oci-layout`

### Requirement: Render and serve are not built

render and serve SHALL answer absent, naming the operation and the
projection, until a later unit builds them.

#### Scenario: serve asked

- **WHEN** serve is asked of a layout
- **THEN** it answers absent naming serve of the image projection, and starts nothing

### Requirement: The projection passes the projection group's pack scenarios

The profile SHALL pass the projection group's scenarios of pack: packed
the same again, and a projection that cannot be reached (84, 87, 58,
59).

#### Scenario: The pack scenarios over the image projection

- **WHEN** those scenarios run against the profile over an image archive in a temporary directory
- **THEN** each passes
