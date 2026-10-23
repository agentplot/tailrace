---
number: 12
title: A clause a later unit builds is named pending, not failed
status: proposed
date: 2026-10-23
decision-makers:
- Chuck Swanberg
---

# A clause a later unit builds is named pending, not failed

Status: proposed to the user on 2026-10-23, as `briefs.15` has it; accepted only on their yes.

Deciders: madswan-design, 2026-10-23, on the question of trc-1's construct of the-conformance-suite-over-the-fakes (bolt tailrace-the-core); the user on their card.
Sources: statement clause 93 and section 12 (`agentplot/blueprints:design/tailrace/requirements.md`); rule core.33; `design/tailrace/models/phase-1-bolts.md` at 251de29; flywheel-next's `machines/pending.yaml` and `check.py` (fcaac6be), the same rule in the sister kit.

## Context and problem statement

Section 12 says the conformance suite fails when a clause of Parts A and B is cited by nothing, and core.33 says a scenario cites only the clauses it exercises. Phase 1's bolt 1 is done when the suite passes over the fakes, and bolt 1 builds no gate, channel, ledger, environment, consumer layout, cache or bindings file, so about forty clauses have nothing to exercise until bolts 2 to 5. Read literally, the coverage check fails bolt 1's gate and its unit cannot land; citing those clauses from scenarios that do not exercise them breaks core.33. How does the suite treat a clause whose behavior a later unit builds?

## Considered options

1. A register in the conformance crate naming each such clause with the unit of the plan that will cite it and why; the check reports a registered clause as pending and fails a clause neither cited nor registered; an entry leaves the register with the commit that cites its clause, so a clause both cited and registered fails too.
2. The same register, appended to and never edited, removed whole by phase 1's last unit.
3. Coverage checked only over the clauses bolt 1 builds, the full check turned on by a later unit.
4. The statement or core.33 amended first to say how a clause waits, the unit blocked until then.

## Decision outcome

Option 1. The check's purpose is that no clause is forgotten and none is cited falsely; a register that names who will cite each waiting clause keeps both, and the debt is read in the open on every run. An entry that must leave with the commit citing its clause keeps the register true to the suite at every revision, where an append-only one (2) goes stale and hides a clause cited early or never. Checking only what bolt 1 builds (3) puts the list of waiting clauses in the check's code, where nobody reads it, and turns the full check on by a hand later. Amending the rule first (4) is what this record does, in the same commit as the rule, and need not block the unit: the rule as amended is the one the unit lands under.

The same rule holds in flywheel-next, whose model checker reads `machines/pending.yaml` the same way (fcaac6be), so the family's two statements are held to one rule.

### Consequences

- Good: bolt 1 lands with a suite that passes and names the forty clauses bolts 2 to 5 will cite, each with its unit; a later unit that cites a clause must also strike its entry, or the check fails and says so.
- Good: no scenario cites a clause it does not exercise (core.33 holds), and no clause is silently uncovered.
- Bad: a unit's plan may change, leaving an entry naming a unit that no longer exists; the check reads the unit's name as a note, not a reference, so such an entry is corrected by hand when the plan moves.
- Neutral: S5 and S13 are played in bolt 1 as far as verify's answer, the claim by a signer the policy does not name listed and not counted, nothing moved, the version readable by digest; the gate's refusal and acceptance are appended under the gate's marker by bolt 4's gate unit, which strikes 25 to 30 from the register.

## More information

The construct's question, in its words, named the clauses: 25 to 30, 94 to 96, 107, 108, 112 (acceptance); 32, 33, 36 to 39, 101, 103, 104, 106, 109, 125 (channels); 41, 44, 45, 97, 98, 119 (environments); 46, 47, 49, 51, 99, 113 to 115 (consumers); 52, 54, 55 (cache); 63 (rebuild); 118 (bindings file). The register's form in the crate is the construct's.
