//! The one conformance suite that holds every profile to Parts A and B of
//! the statement (93): a profile that fails it is not one. Every scenario
//! cites the clauses of Parts A and B it exercises, as data the coverage
//! check reads, and asserts what each clause it cites requires of the
//! case it sets up (rule core.33; design D1).
//!
//! # The groups
//!
//! Scenarios sit in groups (design D2), each a module whose `scenarios()`
//! lists them in order and whose `GROUP` names it:
//!
//! | group | module | runs over |
//! |---|---|---|
//! | the store | [`seams::store`] | a [`StoreRig`](seams::store::StoreRig) |
//! | the build | [`seams::build`] | a [`BuildRig`](seams::build::BuildRig) |
//! | the landing | [`seams::landing`] | a [`LandingRig`](seams::landing::LandingRig) |
//! | the signer | [`seams::signer`] | a [`SignerRig`](seams::signer::SignerRig) |
//! | the policy | [`seams::policy`] | a [`PolicyRig`](seams::policy::PolicyRig) |
//! | a projection | [`seams::projection`] | a [`ProjectionRig`](seams::projection::ProjectionRig) |
//! | a target | [`seams::target`] | a [`TargetRig`](seams::target::TargetRig) |
//! | the keeper | [`seams::keeper`] | a [`KeeperRig`](seams::keeper::KeeperRig) |
//! | the binding | [`seams::binding`] | a [`BindingRig`](seams::binding::BindingRig) |
//! | version | [`core::version`] | the [`FakeWorld`](world::FakeWorld) |
//! | publish | [`core::publish`] | the `FakeWorld` |
//! | verify | [`core::verify`] | the `FakeWorld` |
//! | status | [`core::status`] | the `FakeWorld` |
//! | the surfaces | [`core::surfaces`] | a [`Surface`](core::surfaces::Surface) |
//! | the gate | [`core::gate`] | nothing yet: bolt 4's gate unit |
//! | the ledger | [`core::ledger`] | nothing yet: bolt 4's keeper and promotion |
//! | the cache | [`core::cache`] | nothing yet: bolt 3's cache |
//! | the record | [`core::record`] | nothing yet: bolt 3's push after a landing |
//! | the layouts | [`core::layouts`] | nothing yet: bolt 5's render under a policy |
//!
//! A seam's group runs over a rig: the seam's profile, what a scenario
//! sets up through it, and how it makes the source unreachable. The
//! core's groups run over one world of fakes and a real git history
//! (design D4), since many of them script what no real profile can be
//! told to do.
//!
//! # Adding to a group
//!
//! The last line of every group's list is its marker. A later unit adds
//! its scenarios below the marker of the group named for what it builds,
//! and the functions they name below the module's last function, and
//! changes nothing above the marker, so units of one bolt add side by
//! side without touching each other's lines. A unit that builds a new
//! part of the core adds its module and one line at the end of
//! [`groups`]. A clause a scenario cites for the first time leaves the
//! register in the same commit (`register`, rule core.33).
//!
//! # Running a seam's group over a profile
//!
//! A profile's own tests write the rig for its seam and run the seam's
//! group with [`run`], a fresh rig for each scenario:
//!
//! ```ignore
//! use tailrace_conformance::{run, seams::store};
//!
//! #[test]
//! fn the_profile_keeps_the_stores_contract() {
//!     run(store::GROUP, &store::scenarios(), &|| Box::new(TheProfilesRig::new())).assert();
//! }
//! ```
//!
//! [`Report::assert`] fails the test with one report naming the profile,
//! the group, and per failed scenario what it plays, its clauses and why,
//! the same for a fake as for any profile (93). The fakes run every group
//! here, in `tests/fakes.rs`; the command line runs the surfaces' group in
//! its own crate's tests.

pub mod core;
pub mod fakes;
pub mod fixture;
pub mod scenario;
pub mod seams;
pub mod world;

pub use scenario::*;

/// A scenario's citations, as the coverage check reads them: its group,
/// its name, the statement scenario it plays, and its clauses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cited {
    pub group: &'static str,
    pub name: &'static str,
    pub plays: Option<&'static str>,
    pub cites: &'static [u16],
}

/// A group, by name, and its scenarios' citations in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub name: &'static str,
    pub scenarios: Vec<Cited>,
}

fn group<R: ?Sized>(name: &'static str, scenarios: Vec<Scenario<R>>) -> Group {
    Group {
        name,
        scenarios: scenarios
            .into_iter()
            .map(|s| Cited {
                group: name,
                name: s.name,
                plays: s.plays,
                cites: s.cites,
            })
            .collect(),
    }
}

/// Every group of the suite, in order (design D2).
pub fn groups() -> Vec<Group> {
    use crate::core::{
        cache, gate, layouts, ledger, publish, record, status, surfaces, verify, version,
    };
    use crate::seams::{
        binding, build, keeper, landing, policy, projection, signer, store, target,
    };
    vec![
        group(store::GROUP, store::scenarios()),
        group(build::GROUP, build::scenarios()),
        group(landing::GROUP, landing::scenarios()),
        group(signer::GROUP, signer::scenarios()),
        group(policy::GROUP, policy::scenarios()),
        group(projection::GROUP, projection::scenarios()),
        group(target::GROUP, target::scenarios()),
        group(keeper::GROUP, keeper::scenarios()),
        group(binding::GROUP, binding::scenarios()),
        group(version::GROUP, version::scenarios()),
        group(publish::GROUP, publish::scenarios()),
        group(verify::GROUP, verify::scenarios()),
        group(status::GROUP, status::scenarios()),
        group(surfaces::GROUP, surfaces::scenarios()),
        group(gate::GROUP, gate::scenarios()),
        group(ledger::GROUP, ledger::scenarios()),
        group(cache::GROUP, cache::scenarios()),
        group(record::GROUP, record::scenarios()),
        group(layouts::GROUP, layouts::scenarios()),
    ]
}

/// Every scenario's citations, group by group.
pub fn citations() -> Vec<Cited> {
    groups().into_iter().flat_map(|g| g.scenarios).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The groups of design D2: one per seam, the binding, and one per
    /// part of the core.
    const DESIGNED: [&str; 19] = [
        "store",
        "build",
        "landing",
        "signer",
        "policy",
        "projection",
        "target",
        "keeper",
        "binding",
        "version",
        "publish",
        "verify",
        "status",
        "surfaces",
        "gate",
        "ledger",
        "cache",
        "record",
        "layouts",
    ];

    #[test]
    fn every_group_of_the_design_is_listed_once() {
        let names: Vec<&str> = groups().iter().map(|g| g.name).collect();
        assert_eq!(names, DESIGNED);
        let cited: Vec<&str> = citations().iter().map(|c| c.group).collect();
        for group in cited {
            assert!(DESIGNED.contains(&group), "{group} is no group");
        }
    }

    #[test]
    fn s1_says_it_plays_s1_and_cites_the_order_the_answer_and_no_move() {
        let s1: Vec<Cited> = citations()
            .into_iter()
            .filter(|c| c.plays == Some("S1"))
            .collect();
        assert_eq!(s1.len(), 1, "{s1:?}");
        assert_eq!(s1[0].group, "publish");
        for clause in [17, 19, 20] {
            assert!(s1[0].cites.contains(&clause), "S1 does not cite {clause}");
        }
    }

    #[test]
    fn every_scenario_has_a_name_and_a_clause() {
        for cited in citations() {
            assert!(
                !cited.name.is_empty(),
                "a scenario of {} has no name",
                cited.group
            );
            assert!(
                !cited.cites.is_empty(),
                "`{}` of {} cites nothing",
                cited.name,
                cited.group
            );
        }
        for group in groups() {
            let mut names: Vec<&str> = group.scenarios.iter().map(|c| c.name).collect();
            names.sort_unstable();
            let all = names.len();
            names.dedup();
            assert_eq!(
                names.len(),
                all,
                "two scenarios of {} share a name",
                group.name
            );
        }
    }
}
