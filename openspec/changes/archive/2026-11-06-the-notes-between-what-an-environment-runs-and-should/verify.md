# Verify: the-notes-between-what-an-environment-runs-and-should

This report checks the work on
`change/the-notes-between-what-an-environment-runs-and-should` at
`11840fe` against this change's proposal, design, nine delta specs and
tasks. The branch was cut from `bolt/tailrace-the-nix-fleet-pushed`.
It took that bolt in at `7241623` (design D1), and `7241623` is still
the bolt's head.

**The work holds, and the change is archived.** Every task is ticked,
every check the tasks name passes, and every requirement of the nine
delta specs is met in the code and tested. The statement the change
rests on, agentplot/blueprints `90cf023`, is on `origin/main`.

## The checks

| check | answer |
|---|---|
| `openspec validate the-notes-between-what-an-environment-runs-and-should --strict` | valid |
| `moon run ':#check-fast' --force` | exit 0. All 21 tasks ran uncached: format, lint, toolchain pins, the records' numbers, the hooks, every crate's tests, the conformance suite over the fakes, the coverage check, the herdr plugin under sh and dash, and the site's build. 1584 tests passed and 0 failed. |
| `moon run ':#check-tools' --force` | exit 0. `cli:tools` ran uncached: 101 tests passed and 0 failed. |
| the coverage check, `conformance:coverage` | 143 clauses: 130 cited and 13 pending, read at agentplot/blueprints `90cf023`. 98, 132 and 134 are cited. No register entry names 98, `PARTS` is empty, and no entry names environments in status. 141 is pending with `every-claim-takes-the-catalogued-kind-that-carries-it`, and 142 and 143 with `phase 2 · keyless signing`, each with its reason. 44 and 126 stay with `phase 2`. |
| `git merge-base --is-ancestor 90cf023 origin/main` in agentplot/blueprints, after a fetch | holds (task 5.4's condition) |

## The tasks

Tasks 1.1 to 7.1 are all ticked, 18 of them, and each one's Verify
holds:

- 1.1: the bolt was merged at `236d6c4`. `docs/adr/` holds 0026 (the
  shell carries bun), 0027 (a Nix closure is one artifact) and 0028 (a
  pushed machine's report). The details of rules core.70 to core.73
  cite 0027 and 0028.
- 2.1: `after::reports`, `newest_of` and `reporters` are in
  `crates/timber-core/src/after.rs`. The gate's `newest` reads the
  chain through `ledger::held`, which answers a chain kept as last read
  as unreadable, and it takes no copy. `crates/timber-core/tests/after.rs`
  covers a rollback, two reports at one moment, an older report that
  cannot be told, and a newer one that cannot be told.
- 2.2: `crates/timber-core/src/reports.rs` keeps the copy at
  `<common>/timber/reports/[<component>/]<environment>.json`, written
  beside the file and renamed into place. `tests/status.rs` covers
  every answer the task lists, and one index asked once for two
  environments.
- 2.3: `status::environments` and `environment` are built from one
  `heads` read. The four counts sit beside the list, and `row` carries
  them. Nothing named `no_target` is left in the tree. `tests/status.rs`
  holds each of the ten cases the task names.
- 2.4: `status::between` is covered by `tests/status.rs`: two ahead,
  back, a version skipped, a prerelease, notes that cannot be rendered,
  a commit the history lacks, no semantic version, the same version at
  another index, and the notes byte for byte the changelog's.
- 3.1: `Reach::None` is in `kms/reach.rs`. `kms/tests.rs` covers a
  P-256 envelope against a listed ARN, an Ed25519 envelope refused
  unreached, and every sign absent. `tests/kms_signer.rs` shows that no
  request reaches the stand-in.
- 3.2: `For::Status` in `bindings/mod.rs` binds the store, opened and
  never made, with `Unopened::answering` where it cannot be opened. Its
  signers are `profiles::signer::unreaching()`. `bindings/tests.rs`
  covers each of the four cases the task names.
- 4.1: `commands/status.rs` prints the counts and each environment's
  line. `cli.rs` has the environment behind, as text and as JSON, and
  the store unreachable after a fetch. `status_across_machines.rs` has
  a second machine that never published.
- 4.2: `commands/serve/page.rs` renders the environments and the
  root's counts. `serve.rs` has notes holding `<script>`, escaped, the
  page's JSON equal to status, and the root's counts.
- 4.3: `plugins/herdr/lib/words.jq` renders the counts and each
  environment. The `environments` documents are produced by
  `tests/herdr_documents.rs`, and `run.sh` frames and greps them.
- 5.1 to 5.3: the status group holds the thirteen environment
  scenarios, each citing its clauses. The register test
  `environments_in_status_leave_the_register_with_98_cited_and_no_part_of_132_or_134_waiting`
  passes.
- 5.4: `statement.rs` reads `agentplot/blueprints 90cf023` and holds 1
  to 143 in 20 sections, with 141, 142 and 143 after 13 in A.2. The
  suite's clauses have left `docs/architecture/_open.md`.
- 6.1: record 0029 is accepted, decided by madswan-design on
  2026-11-06. It holds the three options of the ruling and D3, D4, D5,
  D6, D8 and D11, each with the paths not taken, plus seven further
  choices of the build's own. Record 0028 carries the amendment line.
- 6.2: the text of core.32 is as D13 words it. The details of core.32,
  core.27 and core.71 name the copy, and `_open.md`'s C.7 item says
  status reads a key-service signer's report as unread.
- 6.3: `surfaces.mdx` describes the environments, the notes between,
  the copy, the store unreachable and key-service reports unread.
  `herdr.mdx` names what the pane shows.
- 7.1: this report's checks.

## The requirements

Each scenario is listed with what holds it. Unless named otherwise,
core tests are in `crates/timber-core/tests/status.rs`, and suite
scenarios are in `crates/timber-conformance/src/core/status.rs`.

### environment-status

- Two environments of one component:
  `two_environments_of_one_component_are_named_with_their_channels_and_whom_they_wait_on`.
- One component asked: `one_component_asked_shows_its_environments_alone_and_counts_them`.
- No environment declared:
  `a_policy_declaring_no_environment_answers_an_empty_list_with_four_counts_of_none`.
- The head the channels part shows:
  `an_environment_running_its_channels_head_is_converged_and_no_target_is_asked`.
  It compares `should` with the channels part's head.
- A chain that cannot be read:
  `a_chain_that_cannot_be_read_is_what_the_environment_should_run_and_it_is_unread`.
- Two reports, the later counted: the same converged test, in which the
  fake target records no call. Also the suite's "an environment running
  what it should".
- A report by a signer that may not report: `after.rs`
  `a_report_by_a_signer_that_may_not_report_counts_for_nothing_and_stays_attached`
  uses a signer listed with `review` alone. `reporters` keeps only
  signers whose kinds hold `report`. The suite's "a report by a signer
  the policy lets claim none" shows the environment unread and counted
  so.
- A policy that lets no signer report:
  `a_policy_letting_no_signer_report_leaves_every_environment_unread_saying_so`.
- A report only a key service could tell: `after.rs`
  `a_newer_report_whose_signer_cannot_be_told_is_unreadable_naming_it_and_the_key`.
  `kms/tests.rs` and `tests/kms_signer.rs` show that no request reaches
  KMS.
- An older report that cannot be told: `after.rs`
  `an_older_report_whose_signer_cannot_be_told_is_never_told`.
- One index asked once: `one_index_is_asked_its_claims_once_for_two_environments`.
- The copy read with a file reader:
  `the_reports_are_kept_with_their_indexes_and_the_moment_and_nothing_else`.
- The store unreachable after a fetch: the suite's "S10 the
  environments as last fetched", which faults every store operation.
  Also `cli.rs`
  `the_store_unreachable_after_a_fetch_prints_what_each_environment_ran_as_last_fetched`.
- Never fetched, the store unreachable:
  `with_no_copy_and_the_store_unreadable_the_reports_are_unreadable_naming_them`.
  `commands/status.rs` exits 0 whenever an overview is made.
- An index the store does not hold:
  `an_index_the_store_holds_none_of_is_unreadable_naming_it_and_the_copy_is_left`.
- Behind, and Waiting on a person:
  `an_environment_whose_newest_counted_report_names_another_index_is_behind`
  and
  `a_waiting_environment_whose_own_channel_is_behind_the_one_it_is_bound_to_is_waiting`.
- An empty channel: `an_environment_whose_channel_is_empty_is_unread_saying_so`.
- After another, and An environment after another:
  `an_environment_after_another_runs_what_it_should_and_is_never_waiting`.
  Also the suite's "an environment after another".
- Two versions ahead, Back, A version the channel skipped, and
  Prereleases between: the four `between` tests named in task 2.4.
- The notes of the versions between:
  `two_versions_ahead_are_named_with_their_tags_revisions_and_notes_as_the_changelogs`.
- Not behind: the converged test asserts `between` is `None`. `status`
  sets `between` only where the state is behind.
- One version's notes cannot be rendered:
  `a_version_whose_notes_cannot_be_rendered_is_its_own_answer_and_the_rest_shown`.
- A commit the history no longer holds:
  `a_commit_the_history_lacks_makes_the_versions_between_unreadable_naming_it`.
- One of each: `four_environments_one_of_each_are_counted_one_one_one_and_one`.
  Also the suite's "one environment of each state".
- An environment behind, as text and JSON: `cli.rs`
  `an_environment_behind_prints_its_line_and_the_notes_between_and_its_json_is_the_cores`.

### environment-reports

- srv rolled back: `after.rs`
  `a_rolled_back_environments_newest_report_names_the_version_it_went_back_to`.
- The gate on a chain as last read: `after.rs`
  `a_chain_the_ledger_cannot_read_is_unreadable_naming_it`, and
  `apply.rs`
  `a_chain_the_ledger_answers_as_last_read_is_unreadable_and_the_target_is_asked_nothing`.
  `newest` takes no `ReportsCopy`, so no copy is read or written.
- Status on a chain as last read:
  `a_chain_answered_as_last_read_has_its_environments_reports_read_across_it_from_the_store`.
  Also the suite's "an environment's reports read across a chain as
  last read".

### channel-status

- The store unreachable: the suite's S10 scenarios for the channels and
  for the environments.
- Nothing written but the copy:
  `status_with_environments_asks_the_store_only_their_claims_and_writes_only_the_copy`.
  Also
  `status_declaring_no_environment_asks_no_store_signer_target_or_projection_and_writes_nothing`.
- The record deleted: the suite's S12 for the channels.
- The reports' copy deleted:
  `the_reports_copy_deleted_answers_the_same_environments_and_is_written_again`.
  Also the suite's S12 for the environments.
- A row of heads: `a_row_names_every_channels_head_and_the_environments_counts`.
- A row whose channels cannot be read:
  `a_row_whose_channels_cannot_be_read_carries_that_answer`.

### host-bindings

- A key service host asking status:
  `a_key_service_host_asking_status_binds_both_verify_only_reaching_no_service`.
- A clone with no bindings file, and A host that never published:
  `status_with_no_file_binds_the_given_landing_and_the_git_ref_on_its_remote_and_makes_nothing`.
  With no given layout, every store operation answers the opening's
  words, and no file is made.
- A store that cannot be opened:
  `a_git_ref_ledger_binds_status_with_a_store_that_cannot_be_opened_answering_its_words`.
  Also
  `a_file_naming_a_store_ledger_binds_status_opening_the_layout_and_never_making_it`.
- The remaining scenarios (a host key host, a key service host
  publishing, a host naming no signer, a host that signs nothing, a
  repository set up for sluice, and a file naming a landing and no
  ledger) are unchanged, and their tests pass.

### key-service-signer

- Both scenarios: `kms/tests.rs`
  `bound_to_reach_no_service_a_p256_signature_against_a_listed_key_is_unreadable_naming_it`
  and `bound_to_reach_no_service_a_host_keys_envelope_is_refused_unreached`.

### serving-process

- The notes between beside an environment that is behind, and the
  root's four counts: `serve.rs`
  `an_environment_behind_shows_the_notes_between_escaped_and_its_json_is_status`.
  It checks the page's JSON against `timber status --json`, `<script>`
  escaped, no script on the page, and the root's row and page counts.
- Two repositories together: `two_repositories_named_are_answered_together_at_the_root`.
  It checks each row's four counts.
- The scenarios carried over unchanged pass.

### herdr-plugin

- Rows: `run.sh` frames the binary's `environments` documents, a row
  counted 0, 1, 1 and 1. "every: a repository that declares no
  environment shows four counts of none" covers four zeros, and "every:
  environments not read are said, and never counted" covers an
  unreadable part.
- An environment behind, and one that cannot be read: "one: each
  environment with its state, what it runs and should, what it waits on
  and the notes between". It also asserts that `db` and `desk` are
  never shown as running or behind.

### conformance-suite and clause-coverage

- The status group holds the scenarios the delta lists, S11 among them,
  each citing its clauses. `surfaces.rs` and `tests/conformance.rs` hold
  the command line and the page's JSON to the core's answer.
- The register and the statement: see the coverage check above.

## Observations

These are not failures:

- The build amended this change's own `environment-status` delta in
  `194393c`. "A commit the history no longer holds" now makes the whole
  between unreadable, since the history's walk of the tags fails before
  any version's notes are rendered. A new scenario, "One version's
  notes cannot be rendered", holds the per-version answer. Record 0029
  names the choice and the path not taken.
- An environment no counted report names renders as "the store (bound)
  holds no a counted report of environment db of repository". The
  `what` is the bolt's wording, joined to the absent answer's "holds
  no". It reads awkwardly but carries the reason 98 asks for.
- Some herdr fixtures (`kinds`) are hand-written documents whose
  environments' part is unreadable for a target. The pane renders
  whatever the answer says, so they still exercise the "cannot be read"
  path.

## Decisions

This stage made no decision. The ruling the change carries is
madswan-design's, recorded in the statement at agentplot/blueprints
`90cf023` (on `origin/main`), in record 0029 and in rule core.32, all
committed on this branch.
