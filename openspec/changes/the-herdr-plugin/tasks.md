# Tasks

Every cargo and moon command runs as `devenv shell -- <command>`. Code
starts once bolt 4 (`bolt/tailrace-the-gate-and-the-channels`) has
landed on `main` and this bolt has taken it: the pane reads its
`tailrace serve`, `/serving.json`, `/status.json` and
`/<owner>/<name>/status.json`. Read `crates/tailrace/src/commands/serve/`,
`commands/status.rs`, `crates/tailrace-core/src/status.rs` and
`served.rs`, `crates/tailrace/tests/serve.rs`, and agentplot/sluice
`plugins/herdr/` before starting.

## 1. The plugin's project and gate

- [x] 1.1 Add `plugins/herdr/` with `herdr-plugin.toml`, `bin/open.sh`, an empty-frame `bin/status.sh`, `lib/common.sh`, `lib/words.jq`, `README.md` and `moon.yml` as design D1 and D7 have them; add `plugin-herdr` to `.moon/workspace.yml`; add `pkgs.dash` and `pkgs.curl` to `devenv.nix`; add the jq and fixture excludes, each with its reason, and the stand-ins' shellcheck and shfmt includes to `treefmt.nix`; add `tests/manifest.sh` (spec `herdr-plugin`, the plugin is kept outside the core). Verify `devenv shell -- moon run plugin-herdr:manifest` passes and `devenv shell -- treefmt --ci` passes.

## 2. The documents are the binary's

- [x] 2.1 Add `crates/tailrace/tests/herdr_documents.rs` as design D6 part 2 has it, writing `plugins/herdr/tests/docs/never-published/` with `TAILRACE_HERDR_DOCS=write` and comparing otherwise; add `/plugins/herdr/tests/docs/**/*` to `cli:test`'s inputs in `crates/tailrace/moon.yml` (spec `herdr-plugin`, the pane shows what status prints on a machine that never published). Verify `devenv shell -- moon run cli:test` passes, and fails naming the field when one committed document is edited.

## 3. Finding, starting and asking the process

- [x] 3.1 In `lib/common.sh`, the address once and the ask of D2, and finding or starting the process as D3 has it; add `tests/run.sh`, `tests/dash.sh` and the stand-ins `tests/fake/curl`, `tailrace` and `herdr` of D6 part 1, with tests for: a process answering starts nothing; none answering starts `tailrace serve` once with a working directory that is no checkout; something else answering starts nothing and is shown with its words; a start that never answers is said and not repeated; `tailrace` missing from `PATH` is said; two `--once` runs started together leave one marker; only the core's address is asked and no command but `tailrace serve` runs (spec `herdr-plugin`, the pane starts the process and reads the host's one process). Verify `devenv shell -- moon run plugin-herdr:test-sh plugin-herdr:test-dash` passes.

## 4. The two views

- [x] 4.1 In `lib/words.jq` and `bin/status.sh`, the every-repository and one-repository frames, the answer rendering and the keys of D4 and D5, redrawing every five seconds and on `r`; add committed `.frame` files for the never-published documents and for hand-made documents holding each answer kind at each place it can stand, a row with counts, an empty list and a repository answered absent; test the cursor, opening a row, going back and `--repo` (spec `herdr-plugin`, every requirement on what the pane shows and that it computes nothing). Verify `devenv shell -- moon run plugin-herdr:test-sh plugin-herdr:test-dash` passes.
- [x] 4.2 Fill in `README.md` with what the pane shows, installing it, the key snippet and what it needs; mark the herdr row of `site/src/content/docs/guides/integrations/surfaces.mdx` built and say there that the pane shows every repository and one whole, read from the host's one process, which it starts where none answers. Verify `devenv shell -- moon run ':#check-fast'` passes.

## 5. The real process, by hand

- [ ] 5.1 Add `tests/host.sh` and the `host` task tagged `check-host` as design D6 part 3 has it, refusing to run where anything answers at the core's address (spec `herdr-plugin`, two panes on one host). This task cannot run in the gate, because it binds the core's port. Verify it by hand: `devenv shell -- moon run plugin-herdr:host` passes on a host where nothing answers at `127.0.0.1:7420`, and refuses, naming the address, while a `tailrace serve` runs.
