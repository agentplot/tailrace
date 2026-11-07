---
number: 30
title: The shell declares and approves the MCP servers, and the tree tracks OpenSpec's Claude Code files
status: accepted
date: 2026-11-07
---

# The shell declares and approves the MCP servers, and the tree tracks OpenSpec's Claude Code files

Status: accepted on the brief of the chore timber-devenv-declares-its-mcp-servers, 2026-11-07, which decides it in its words: "timber's devenv declares its MCP servers in claude.code.mcpServers, devenv's hosted mcp.devenv.sh named among them, and approves each by name in the project settings devenv writes (enabledMcpjsonServers), so agent sessions start them unasked; the generated .mcp.json and .claude/settings.json are ignored in git, what its Claude Code settings hold today moves into devenv's options, and it tracks OpenSpec's Claude Code skills and commands as its pinned OpenSpec writes them (agentplot/boomdrive docs/adr/0022, accepted)". The family's decision is boomdrive's record 0022, accepted by the user on 2026-11-07; how timber carries it out below is the chore's own, cheap to change.
Sources: agentplot/boomdrive record 0022 and its amendment of 2026-11-07; ADR 0008; rules core.25, core.26; devenv's `src/modules/integrations/claude.nix` and `src/modules/files.nix` at devenv.lock's pin; `.config/sluice.toml`; flume's, boomdrive's, boomdrive-cloud's and boomdrive-web's chores of the same brief, 2026-11-07.

## Context and problem statement

timber's devenv declared no MCP server, the tree held no Claude Code settings, and no OpenSpec was pinned: its changes were written with whatever `openspec` the host had. Record 0022 has each repository of the family declare its servers in its devenv, approve them for its trusted sessions, ignore the two files devenv writes and track OpenSpec's Claude Code files as its pinned OpenSpec writes them, and leaves to each repository's change that its places hold the servers before their agents start.

A timber place's readiness is sluice's equip of it, which sources the script `devenv print-dev-env` prints (`.config/sluice.toml`'s toolchain): that script holds the shell's `enterShell` and none of devenv's tasks, so devenv's own task that writes `.mcp.json` and `.claude/settings.json` never runs there.

## Considered options

1. As boomdrive's amendment does it: `enterShell` runs devenv's files script itself; the settings approve every server `claude.code.mcpServers` names; devenv's `git-hooks-run` hook is off; OpenSpec 1.14.3 is pinned in the shell and `root:toolchain` holds the skills to it.
2. The files left to devenv's task alone, so a place equipped by sluice has neither until someone enters the shell there.
3. The skills and commands copied from a sibling repository rather than written by a pinned OpenSpec of timber's own.

## Decision outcome

Option 1, so timber's change matches the family's and a place's agent starts the servers.

- **The servers.** devenv's hosted `mcp.devenv.sh` alone, named because a list of the repository's own replaces devenv's default. No other server is asked of timber.
- **The approval.** devenv has no option for `enabledMcpjsonServers`; it is added to the settings file devenv writes, naming every server `claude.code.mcpServers` names, so a server added there is approved with it.
- **The place's files.** `enterShell` runs devenv's files script from the repository's root, writing nothing to standard output, and says on standard error where it could not write them; the shell is entered either way. Running a task from equip was the other path, which would change the environment sluice's gate runs every check under.
- **devenv's edit hook.** `claude.code.hooks.git-hooks-run` is off. It runs prek after every edit, and an agent here runs each tool through `devenv shell --`, so prek is not on its session's PATH and the hook would fail at each edit; the pre-commit hook runs treefmt and clippy on what is staged, as flume and boomdrive-cloud decide.
- **Settings moved.** The tree held no Claude Code settings, so nothing moves into devenv's options; `treefmt.nix`'s exclude of `.claude/settings.json`, a file git now ignores, is dropped.
- **OpenSpec.** 1.14.3, the release flume, sluice and boomdrive pin, built as they build it. The skills and commands are those `openspec init --tools claude --profile custom` writes with the ten workflows the family tracks, propose, explore, new, continue, apply, ff, sync, archive, bulk-archive and verify, read from the user's OpenSpec configuration. `root:toolchain` fails while `openspec --version` is not the pin or a skill names another release (core.26).

### Consequences

- A bump of OpenSpec changes the version and its two hashes in `devenv.nix` and writes the skills and commands again, in one commit.
- A session in a clone nobody has trusted still asks to approve the servers (record 0022).
- A Claude Code setting of the repository's goes into devenv's options in `devenv.nix`, never into the ignored files.
