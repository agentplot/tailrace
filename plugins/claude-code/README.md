# tailrace for Claude Code

Keeps a Claude Code session publishing a tailrace repository only through
`tailrace publish`: an ecosystem's own publish command is refused before
it runs, and the refusal names `tailrace publish` for a landed revision
instead.

```
/plugin marketplace add agentplot/tailrace
/plugin install tailrace@tailrace
```

One hook: before each shell command, a guard (`scripts/guard.sh`, tested
by `tests/hooks.sh`). In a repository whose HEAD holds
`.config/tailrace.toml` it refuses a command that runs any of these, as a
segment of a longer command too:

| ecosystem | command |
|---|---|
| npm | `npm publish`, `pnpm publish`, `yarn publish`, `yarn npm publish`, `bun publish` |
| Cargo | `cargo publish` |
| Maven | `mvn deploy`, `mvn deploy:*`, `mvnw deploy`, `gradle publish`, `gradlew publish` |
| Python | `uv publish`, `poetry publish`, `hatch publish`, `flit publish`, `twine upload` |
| NuGet | `dotnet nuget push`, `nuget push` |
| an image | `docker push`, `podman push` |

A command is read as words, never run: the program's verb anywhere after
it counts (`mvn clean deploy`, `npm --workspace ui publish`, `npm run
publish`), a dry run counts, and a command only quoted (`echo "npm
publish"`) does not. Everything else, and every command in a repository
whose HEAD holds no `.config/tailrace.toml`, goes through with no output.
The guard keeps an agent from the obvious command; a script that
publishes is beyond what it sees, and the store's credentials and the
gate stop the rest.

The plugin answers nothing of the core's (56, core.31). It reads only
whether HEAD holds `.config/tailrace.toml`, never the file's contents, and
never runs `tailrace`, so it refuses the same on a host that has none. Its
refusal names the command to run and no version, component or channel,
each of which is the core's to derive.

Guide: <https://agentplot.github.io/tailrace/guides/claude-code/>.

This directory sits outside tailrace's core: no crate imports it, so it
may name the commands it refuses (section 9, rule core.1).
