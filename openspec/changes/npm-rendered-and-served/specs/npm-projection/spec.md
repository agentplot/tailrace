## Purpose

The npm profile of the projection seam (B.6, C.11): an npm package packed
with npm's own packager, a lockfile's pins read as npm spells them, a
registry read layout of tarballs and packuments, and npm's registry
protocol served from that layout, so a consumer's `npm install` reads the
family's packages from what tailrace rendered under the policy (46, 49,
84 to 87, 99, 114, 115, 137).

## ADDED Requirements

### Requirement: An npm output packs with npm's own packager
Given an output, a build identity and a version, the npm projection SHALL
pack the package whose `package.json` is the shallowest the output's
files hold, from a copy of the files the build left under that
directory, with the version set by `npm pkg set version=<version>` and
the tarball made by `npm pack` with no script of the package run (C.11,
15). The artifact's bytes SHALL be that tarball as `npm pack` wrote it,
and its metadata SHALL name the ecosystem `npm`, the package's `name` as
its `package.json` gives it, and the version (4, 84). The build's own
files SHALL be left as they were, and the projection SHALL answer that it
cannot promise the same bytes again (87).

#### Scenario: A package packed at a version
- **WHEN** the npm projection packs an output whose files are a package `@family/widget` at the version `1.4.0`
- **THEN** the bytes are the tarball npm made, whose `package/package.json` names `@family/widget` at `1.4.0`, the metadata names `npm`, `@family/widget` and `1.4.0`, and the output's own `package.json` is unchanged

#### Scenario: No script of the package runs
- **WHEN** the package's `package.json` declares `prepack` and `prepare` scripts and is packed
- **THEN** neither script ran, and the tarball holds only files the build left

#### Scenario: An output with no package
- **WHEN** the npm projection packs an output none of whose files is a `package.json`
- **THEN** the pack is refused naming the output and saying it holds no `package.json`, and nothing runs

#### Scenario: npm refuses the version
- **WHEN** npm refuses to set or pack the version given
- **THEN** the pack is refused carrying npm's own words verbatim (59)

#### Scenario: npm not on the host
- **WHEN** the npm projection packs where no `npm` can be run
- **THEN** the pack is unreadable naming `npm` with the system's own words

#### Scenario: No cache entries
- **WHEN** the npm projection is asked to pack an output with no version, or to render cache entries
- **THEN** each answer is absent naming cache entries, and nothing runs

### Requirement: npm answers what a lockfile pins
pinned SHALL read the checkout's `npm-shrinkwrap.json` where it holds
one, else its `package-lock.json`, and SHALL answer every package the
lockfile's `packages` pins, by the package's own name and its version as
the lockfile spells them, each once; a linked entry and the root entry
SHALL be passed over. A checkout holding neither file SHALL be absent
naming the lockfile; a lockfile of version 1, or one that does not
parse, SHALL be unreadable naming the file and what could not be read
(137, 58).

#### Scenario: A lockfile's pins
- **WHEN** a checkout's `package-lock.json` pins `@family/widget` at `1.4.0` twice under two paths, `left-pad` at `1.3.0`, and an alias `w` of `@family/widget` at `1.2.0`
- **THEN** pinned answers `@family/widget` at `1.4.0`, `left-pad` at `1.3.0` and `@family/widget` at `1.2.0`, each once

#### Scenario: No lockfile
- **WHEN** a checkout holds no `npm-shrinkwrap.json` and no `package-lock.json`
- **THEN** pinned is absent naming the lockfile, never an empty list

#### Scenario: A lockfile npm 6 wrote
- **WHEN** a checkout's `package-lock.json` is of `lockfileVersion` 1
- **THEN** pinned is unreadable naming the file and its lockfile version

### Requirement: npm renders a registry read layout
Given artifacts, the npm projection SHALL render at the layout's place
each artifact's tarball, read from the store by its digest, and one
packument per package name listing exactly the versions given, each
with the `package.json` its tarball holds, its tarball's path in the
layout, its `integrity` and its `shasum`, and a `latest` tag naming the
highest version given by npm's own version order (85, 23). A version not
given SHALL not be in the layout after the render (47). Rendering the
same artifacts again SHALL give the same layout. The layout SHALL name no
host, no store and no place on disk (49).

#### Scenario: Two versions of one package
- **WHEN** the npm projection renders `@family/widget` at `1.2.0` and `1.4.0`
- **THEN** the layout holds both tarballs with the bytes the store holds, and one packument naming both versions, each with its tarball's integrity, and `latest` naming `1.4.0`

#### Scenario: A version that left
- **WHEN** a layout holding `1.2.0` and `1.4.0` is rendered again given only `1.4.0`
- **THEN** the packument names only `1.4.0`, and the tarball of `1.2.0` is not in the layout

#### Scenario: A tarball the store does not hold
- **WHEN** the store answers absent or unreadable for an artifact's bytes
- **THEN** the render answers that, naming the artifact, and the layout is as it was

### Requirement: npm's registry protocol served from the layout
The npm projection SHALL answer npm's own requests of a served layout: a
package's packument, asked by its name, scoped names escaped as npm
escapes them, with every tarball's address made absolute under the base
it is served at; and a tarball, by the path its packument names, with
its bytes (86, 46). A packument SHALL be answered from the layout alone.
A tarball of a version the layout lacks SHALL be asked of the core once,
and answered from the layout after (115, 50). Every answer that is not
the bytes asked for SHALL be a JSON object whose `error` names what was
not answered, so npm prints it: `404` for an absent package or version,
`403` for a refused version, naming the version and the claim found
wanting, and `502` for an unreadable one, carrying the source's own
words verbatim (99, 58, 59).

#### Scenario: A scoped packument
- **WHEN** npm asks a served layout holding `@family/widget` at `1.4.0` for `@family%2fwidget`, served at `http://127.0.0.1:7420/agentplot/consumer/npm`
- **THEN** the answer is `200` with the packument, whose `1.4.0` tarball is `http://127.0.0.1:7420/agentplot/consumer/npm/@family/widget/-/widget-1.4.0.tgz`

#### Scenario: A tarball held
- **WHEN** npm asks for the tarball of a version the layout holds
- **THEN** the answer is `200` with the tarball's bytes, and the core was asked for nothing

#### Scenario: A tarball the layout lacks and the policy admits
- **WHEN** npm asks for the tarball of `1.3.0`, which the layout lacks, and the core renders it into the layout
- **THEN** the core was asked once for `@family/widget` at `1.3.0`, and the answer is `200` with its bytes

#### Scenario: A tarball the policy refuses
- **WHEN** npm asks for the tarball of a version the layout lacks and the core answers it refused, acceptance wanting
- **THEN** the answer is `403` with an `error` naming the version and acceptance as the claim found wanting, and npm's install fails printing those words

#### Scenario: A package no layout holds
- **WHEN** npm asks for the packument of a name the layout holds no version of
- **THEN** the answer is `404` with an `error` naming the package, and the core was asked for nothing

### Requirement: A lockfile npm writes names no host
A family package installed from a served layout SHALL be recorded in the
consumer's `package-lock.json` at the core's one name and port with the
consuming repository's owner and name in the path, the same on every
host and in every checkout of the repository (49, 114, core.18).

#### Scenario: npm install on two hosts
- **WHEN** one consuming repository's checkouts on two hosts, at different paths under different homes, each sync and run `npm install` for `@family/widget` at `1.4.0`
- **THEN** both lockfiles record `resolved` as `http://127.0.0.1:7420/<owner>/<name>/npm/@family/widget/-/widget-1.4.0.tgz` with the same integrity, and neither names a home, a checkout or a store

#### Scenario: Installed only from the layout
- **WHEN** a consumer whose `.npmrc` names the served address for the `@family` scope runs `npm install` with a version the store does not name in its lockfile
- **THEN** the install fails naming that version as absent, and no other registry was asked for any `@family` package
