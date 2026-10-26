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
//!
//! Beside them, the parts: each part of a clause a scenario cites whose
//! behaviour a later unit builds, with the part in a few words, the unit
//! that will build it and why it waits (section 12). The check prints
//! each as pending on every run, and fails a part of a clause no scenario
//! cites, which waits whole and belongs among the clauses.

/// A clause that waits: the unit that will cite it, and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pending {
    pub clause: u16,
    pub unit: &'static str,
    pub why: &'static str,
}

const RENDER: &str = "bolt 5 · render under a policy";
const SERVED: &str = "bolt 5 · the first served projection";
const TARGET: &str = "phase 2 · the first target";
const REBUILD: &str = "phase 2 · the verifier's rebuild";
const PHASE_2: &str = "phase 2";
const NOTES: &str = "the notes unit, which the user has not said yes to";

// One entry per line, as the module says, wider than the formatter's
// width.
#[rustfmt::skip]
pub const PENDING: &[Pending] = &[
    Pending { clause: 47, unit: RENDER, why: "A consumer layout holds only versions its policy satisfies, and bolt 1 renders under no policy." },
    Pending { clause: 51, unit: RENDER, why: "A layout that cannot be refreshed answers what it holds, and bolt 1 keeps no layout." },
    Pending { clause: 99, unit: RENDER, why: "A version a layout's policy refuses is answered naming the claim, and bolt 1 renders under no policy." },
    Pending { clause: 113, unit: RENDER, why: "A consuming repository's layout is rendered under its own policy, and bolt 1 renders under no policy." },
    Pending { clause: 46, unit: SERVED, why: "A consumer reads in its ecosystem's own form, which the first served projection serves." },
    Pending { clause: 49, unit: SERVED, why: "A client's recorded address names no host, which a served layout keeps." },
    Pending { clause: 114, unit: SERVED, why: "A served layout is reached at the core's address, which bolt 5's first served projection serves and cites; the process is status-on-any-machine's, proven in the command line's tests." },
    Pending { clause: 115, unit: SERVED, why: "The host runs a served layout for as long as a client needs it, and bolt 1 serves none." },
    Pending { clause: 41, unit: TARGET, why: "An environment that waits converges on an approver's promotion, S6's environment half, and phase 1 leaves every target to phase 2." },
    Pending { clause: 97, unit: TARGET, why: "An approval is a promotion of the environment's own channel, S6's environment half, and phase 1 leaves every target to phase 2." },
    Pending { clause: 42, unit: TARGET, why: "A reconciler verifies before it applies, and phase 1 leaves every target to phase 2." },
    Pending { clause: 44, unit: TARGET, why: "Pushed to or pulling is the environment's own, and phase 1 leaves every target to phase 2." },
    Pending { clause: 45, unit: TARGET, why: "A reconciler acts with the environment's own credentials, and phase 1 leaves every target to phase 2." },
    Pending { clause: 119, unit: TARGET, why: "A reconciler leaves a marker of what it applied, and phase 1 leaves every target to phase 2." },
    Pending { clause: 126, unit: TARGET, why: "The roster is a target's, and phase 1 leaves every target to phase 2." },
    Pending { clause: 98, unit: TARGET, why: "What each environment should run and runs is reported by a target, and phase 1 leaves every target to phase 2." },
    Pending { clause: 55, unit: REBUILD, why: "A verifier's rebuild reads no cache entry it did not write, and phase 1 leaves the rebuild to phase 2." },
    Pending { clause: 112, unit: REBUILD, why: "A verifier compares the digests of outputs, and phase 1 leaves the rebuild to phase 2." },
    Pending { clause: 127, unit: PHASE_2, why: "A channel declared by a pattern over branch names, which no unit of phase 1 carves." },
    Pending { clause: 128, unit: PHASE_2, why: "Several promotions in one act, which no unit of phase 1 carves." },
    Pending { clause: 129, unit: PHASE_2, why: "The asker named by an identity the owner's identity provider issued, and phase 1 binds none." },
];

/// A part of a clause a scenario cites that waits: the part, the unit
/// that will build it, and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Part {
    pub clause: u16,
    pub part: &'static str,
    pub unit: &'static str,
    pub why: &'static str,
}

// One part per line, as the entries are.
#[rustfmt::skip]
pub const PARTS: &[Part] = &[
    Part { clause: 132, part: "what every environment should run, runs and waits on, and the four counts of each repository's row", unit: TARGET, why: "The pages show environments once a target reports them, and phase 1 leaves every target to phase 2." },
    Part { clause: 132, part: "each version's notes", unit: NOTES, why: "No version carries notes yet, and the notes unit waits on the user." },
];
