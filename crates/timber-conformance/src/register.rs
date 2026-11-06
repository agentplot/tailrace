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

const DRIVER: &str = "bolt 5 · the driver and the pool run publish";
const ENVIRONMENTS: &str = "phase 2 · environments in status";
const REBUILD: &str = "phase 2 · the verifier's rebuild";
const PHASE_2: &str = "phase 2";

// One entry per line, as the module says, wider than the formatter's
// width.
#[rustfmt::skip]
pub const PENDING: &[Pending] = &[
    Pending { clause: 135, unit: DRIVER, why: "A check job pushes what its run left before the landing and a publish takes its outputs from the store, and no publish reads the store for them yet." },
    Pending { clause: 139, unit: DRIVER, why: "The build answers what a checkout's run left, and the build seam has no such operation yet." },
    Pending { clause: 98, unit: ENVIRONMENTS, why: "What each environment should run and runs is reported to an asker by status, and environments in status, which no unit of bolt 6 carves, come in phase 2." },
    Pending { clause: 55, unit: REBUILD, why: "A verifier's rebuild reads no cache entry it did not write, and phase 1 leaves the rebuild to phase 2." },
    Pending { clause: 112, unit: REBUILD, why: "A verifier compares the digests of outputs, and phase 1 leaves the rebuild to phase 2." },
    Pending { clause: 133, unit: REBUILD, why: "A verifier's rebuild reads no compilation's cache another party wrote, and phase 1 leaves the rebuild to phase 2." },
    Pending { clause: 44, unit: PHASE_2, why: "Pushed to or pulling is the environment's own, and every target of bolt 6 is pushed to; the Nix fleet's pull profile, the first that pulls, comes in phase 2." },
    Pending { clause: 126, unit: PHASE_2, why: "The roster is a target's, and no target of bolt 6 holds one; the pull profile's roster comes in phase 2." },
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
    Part { clause: 132, part: "what every environment should run, runs and waits on, and the four counts of each repository's row", unit: ENVIRONMENTS, why: "The pages show environments once status reads them of their targets, which no unit of bolt 6 carves." },
    Part { clause: 134, part: "the notes of every version between what an environment runs and what it should", unit: ENVIRONMENTS, why: "The surfaces show environments once status reads them of their targets, which no unit of bolt 6 carves." },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::citations;

    #[test]
    fn store_naming_leaves_the_register() {
        assert!(
            !PENDING.iter().any(|p| p.clause == 138),
            "138 is cited, and still in the register"
        );
        assert!(citations().iter().any(|c| c.cites.contains(&138)));
    }

    #[test]
    fn cargos_registry_takes_137s_range_and_closure_out_of_the_register() {
        assert!(
            !PENDING.iter().any(|p| p.clause == 137),
            "137 is cited, and still in the register"
        );
        let parts: Vec<&Part> = PARTS.iter().filter(|p| p.clause == 137).collect();
        assert_eq!(parts, Vec::<&Part>::new(), "137's range and closure wait");
        let layouts: Vec<&str> = citations()
            .iter()
            .filter(|c| c.group == "layouts" && c.cites.contains(&137))
            .map(|c| c.name)
            .collect();
        for scenario in [
            "the newest admitted within a range",
            "a pinned version's dependencies, two levels down",
        ] {
            assert!(layouts.contains(&scenario), "{scenario} cites no 137");
        }
    }

    #[test]
    fn the_nix_cache_target_leaves_the_register_naming_who_cites_the_rest() {
        let cited: Vec<u16> = citations()
            .iter()
            .flat_map(|c| c.cites.iter().copied())
            .collect();
        for clause in [42, 45, 119, 136] {
            assert!(cited.contains(&clause), "{clause} is cited by no scenario");
            assert!(
                !PENDING.iter().any(|p| p.clause == clause),
                "{clause} is cited, and still in the register"
            );
        }
        let unit = |clause: u16| {
            PENDING
                .iter()
                .find(|p| p.clause == clause)
                .unwrap_or_else(|| panic!("{clause} is not in the register"))
                .unit
        };
        for (clause, waits_on) in [(44, PHASE_2), (126, PHASE_2), (98, ENVIRONMENTS)] {
            assert_eq!(unit(clause), waits_on, "{clause} waits on another unit");
        }
        let parts: Vec<(u16, &str)> = PARTS
            .iter()
            .filter(|p| [132, 134].contains(&p.clause))
            .map(|p| (p.clause, p.unit))
            .collect();
        assert_eq!(parts, [(132, ENVIRONMENTS), (134, ENVIRONMENTS)]);
        for why in PENDING
            .iter()
            .map(|p| p.why)
            .chain(PARTS.iter().map(|p| p.why))
        {
            assert!(
                !why.contains("leaves every target"),
                "an entry still leaves every target to phase 2: {why}"
            );
        }
    }

    #[test]
    fn the_clan_push_profile_leaves_the_register_with_s16_citing_41_and_97() {
        let s16: Vec<&[u16]> = citations()
            .iter()
            .filter(|c| c.group == "after" && c.plays == Some("S16"))
            .map(|c| c.cites)
            .collect();
        assert_eq!(s16.len(), 1, "the after group plays S16 other than once");
        for clause in [41, 97] {
            assert!(s16[0].contains(&clause), "S16 cites no {clause}");
            assert!(
                !PENDING.iter().any(|p| p.clause == clause),
                "{clause} is cited, and still in the register"
            );
        }
        for unit in PENDING
            .iter()
            .map(|p| p.unit)
            .chain(PARTS.iter().map(|p| p.unit))
        {
            assert!(
                !unit.contains("clan push profile"),
                "an entry still waits on the clan push profile: {unit}"
            );
        }
    }
}
