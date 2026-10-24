//! The register: each clause of Parts A and B that no scenario cites yet,
//! with the unit of the plan that will cite it and why it waits (rule
//! core.33, ADR 0012; design D7). Every run of the coverage check prints
//! each as pending. An entry leaves the register in the commit whose
//! scenario first cites its clause, and the check fails a clause both
//! cited and registered.
//!
//! One entry per line, grouped by unit in the plan's order, so the units
//! of one bolt each remove a block of their own. A unit is a note for a
//! reader, named by its bolt and what it builds; the check reads it as
//! text, never as a reference.

/// A clause that waits: the unit that will cite it, and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pending {
    pub clause: u16,
    pub unit: &'static str,
    pub why: &'static str,
}

const REGISTRY: &str = "bolt 3 · the registry store";
const CACHE: &str = "bolt 3 · the cache";
const PUSH: &str = "bolt 3 · the push after a landing";
const GATE: &str = "bolt 4 · the gate";
const PROMOTION: &str = "bolt 4 · the keeper and promotion";
const ACROSS: &str = "bolt 4 · status across machines";
const RENDER: &str = "bolt 5 · render under a policy";
const SERVED: &str = "bolt 5 · the first served projection";
const TARGET: &str = "phase 2 · the first target";
const REBUILD: &str = "phase 2 · the verifier's rebuild";

// One entry per line, as the module says, wider than the formatter's
// width.
#[rustfmt::skip]
pub const PENDING: &[Pending] = &[
    Pending { clause: 109, unit: REGISTRY, why: "A name moved where the store cannot move it by expected digest needs a store that cannot." },
    Pending { clause: 52, unit: CACHE, why: "Cache entries are written and read by the cache, which bolt 1 does not build." },
    Pending { clause: 53, unit: CACHE, why: "A cache entry is never an artifact, and bolt 1 has no cache entry." },
    Pending { clause: 54, unit: CACHE, why: "A cache entry may be removed at any time, and bolt 1 has no cache entry." },
    Pending { clause: 100, unit: CACHE, why: "The same bytes are one store object whether a cache entry or an artifact names them, and bolt 1 has no cache entry." },
    Pending { clause: 63, unit: PUSH, why: "The record is rebuilt from the store and git by the push after a landing." },
    Pending { clause: 25, unit: GATE, why: "Acceptance is the gate's; bolt 4's gate unit adds the gate's half of S5 and S13 and takes 25 to 30 out." },
    Pending { clause: 26, unit: GATE, why: "Acceptance changes no bytes and is a claim, and bolt 1 builds no gate." },
    Pending { clause: 27, unit: GATE, why: "The offering and accepting credentials are kept apart by the gate, which bolt 1 does not build." },
    Pending { clause: 28, unit: GATE, why: "Acceptance of a version is acceptance of every digest it names, and bolt 1 accepts nothing." },
    Pending { clause: 29, unit: GATE, why: "The gate judges by the claims, the landing and the policy alone, and bolt 1 builds no gate." },
    Pending { clause: 30, unit: GATE, why: "The gate counts an independent rebuild's claim, and bolt 1 builds no gate." },
    Pending { clause: 94, unit: GATE, why: "Nothing accepted is un-accepted, and bolt 1 accepts nothing." },
    Pending { clause: 95, unit: GATE, why: "The gate judges an offered version on being told of it, and bolt 1 builds no gate." },
    Pending { clause: 96, unit: GATE, why: "No host that builds holds the accepting credential, and bolt 1 has none." },
    Pending { clause: 107, unit: GATE, why: "The gate reads the policy at the governed branch's landed head, and bolt 1 builds no gate." },
    Pending { clause: 108, unit: GATE, why: "One host as both offerer and gate is the gate's to say, and bolt 1 builds no gate." },
    Pending { clause: 32, unit: PROMOTION, why: "A channel moves only by a promotion, and bolt 1 promotes nothing." },
    Pending { clause: 33, unit: PROMOTION, why: "A promotion is recorded in the ledger before the channel moves, and bolt 1 keeps no ledger." },
    Pending { clause: 37, unit: PROMOTION, why: "A bound channel takes only versions its bound channel held, and bolt 1 binds no channel." },
    Pending { clause: 38, unit: PROMOTION, why: "Moving a channel away withdraws nothing, and bolt 1 moves no channel." },
    Pending { clause: 39, unit: PROMOTION, why: "Every promotion is a claim attached to the index it moves to, and bolt 1 promotes nothing." },
    Pending { clause: 41, unit: PROMOTION, why: "An environment that waits converges on an approver's promotion, and bolt 1 promotes nothing." },
    Pending { clause: 97, unit: PROMOTION, why: "An approval is a promotion, and bolt 1 promotes nothing." },
    Pending { clause: 101, unit: PROMOTION, why: "One record is the source of a channel's every state, and bolt 1 keeps no channel." },
    Pending { clause: 106, unit: PROMOTION, why: "The keeper's chain is that record and the store is rewritten from it, and bolt 1 keeps no ledger." },
    Pending { clause: 125, unit: PROMOTION, why: "A channel belongs to one component, and bolt 1 declares no channel." },
    Pending { clause: 36, unit: ACROSS, why: "A channel resolves to one version for every reader, read across machines." },
    Pending { clause: 98, unit: ACROSS, why: "What each environment should run and runs is told to every asker across machines." },
    Pending { clause: 47, unit: RENDER, why: "A consumer layout holds only versions its policy satisfies, and bolt 1 renders under no policy." },
    Pending { clause: 51, unit: RENDER, why: "A layout that cannot be refreshed answers what it holds, and bolt 1 keeps no layout." },
    Pending { clause: 99, unit: RENDER, why: "A version a layout's policy refuses is answered naming the claim, and bolt 1 renders under no policy." },
    Pending { clause: 113, unit: RENDER, why: "A consuming repository's layout is rendered under its own policy, and bolt 1 renders under no policy." },
    Pending { clause: 46, unit: SERVED, why: "A consumer reads in its ecosystem's own form, which the first served projection serves." },
    Pending { clause: 49, unit: SERVED, why: "A client's recorded address names no host, which a served layout keeps." },
    Pending { clause: 114, unit: SERVED, why: "A served layout is reached by one fixed name and port, and bolt 1 serves none." },
    Pending { clause: 115, unit: SERVED, why: "The host runs a served layout for as long as a client needs it, and bolt 1 serves none." },
    Pending { clause: 42, unit: TARGET, why: "A reconciler verifies before it applies, and phase 1 leaves every target to phase 2." },
    Pending { clause: 44, unit: TARGET, why: "Pushed to or pulling is the environment's own, and phase 1 leaves every target to phase 2." },
    Pending { clause: 45, unit: TARGET, why: "A reconciler acts with the environment's own credentials, and phase 1 leaves every target to phase 2." },
    Pending { clause: 119, unit: TARGET, why: "A reconciler leaves a marker of what it applied, and phase 1 leaves every target to phase 2." },
    Pending { clause: 55, unit: REBUILD, why: "A verifier's rebuild reads no cache entry it did not write, and phase 1 leaves the rebuild to phase 2." },
    Pending { clause: 112, unit: REBUILD, why: "A verifier compares the digests of outputs, and phase 1 leaves the rebuild to phase 2." },
];
