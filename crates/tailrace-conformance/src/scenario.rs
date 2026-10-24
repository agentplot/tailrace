//! A scenario and its citations, a group run over a rig made new for each
//! scenario, and the one report every profile gets, a fake included (93,
//! rule core.33; design D1, D3).

use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// One scenario: what it is called, the statement scenario it plays where
/// it plays one, the clauses of Parts A and B it exercises, and what it
/// runs over its rig, panicking on an assertion that fails.
pub struct Scenario<R: ?Sized> {
    pub name: &'static str,
    pub plays: Option<&'static str>,
    pub cites: &'static [u16],
    pub run: fn(&R),
}

/// What every rig names: the profile it binds, which a report names.
pub trait Rig {
    fn profile(&self) -> String;
}

/// One scenario that failed: the scenario, what it plays, the clauses it
/// cites, and why, in the words of the assertion that failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub scenario: &'static str,
    pub plays: Option<&'static str>,
    pub cites: &'static [u16],
    pub why: String,
}

/// What a group's run came to: the profile, the group, how many scenarios
/// ran, and every one that failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub profile: String,
    pub group: String,
    pub ran: usize,
    pub failures: Vec<Failure>,
}

/// Runs every scenario of `group` on a rig of its own, made by `make`,
/// and keeps every outcome: a failure, or a rig that cannot be made, does
/// not stop the scenarios after it.
pub fn run<R: Rig + ?Sized>(
    group: &str,
    scenarios: &[Scenario<R>],
    make: &dyn Fn() -> Box<R>,
) -> Report {
    let mut profile = None;
    let mut failures = Vec::new();
    for scenario in scenarios {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let rig = make();
            let name = rig.profile();
            let ran = catch_unwind(AssertUnwindSafe(|| (scenario.run)(&*rig)));
            (name, ran)
        }));
        let why = match outcome {
            Ok((name, ran)) => {
                profile.get_or_insert(name);
                ran.err().map(|panic| said(panic.as_ref()))
            }
            Err(panic) => Some(format!(
                "the rig could not be made: {}",
                said(panic.as_ref())
            )),
        };
        if let Some(why) = why {
            failures.push(Failure {
                scenario: scenario.name,
                plays: scenario.plays,
                cites: scenario.cites,
                why,
            });
        }
    }
    Report {
        profile: profile.unwrap_or_else(|| make().profile()),
        group: group.to_string(),
        ran: scenarios.len(),
        failures,
    }
}

/// What a panic said, as the assertion wrote it.
fn said(panic: &(dyn std::any::Any + Send)) -> String {
    if let Some(words) = panic.downcast_ref::<&str>() {
        words.to_string()
    } else if let Some(words) = panic.downcast_ref::<String>() {
        words.clone()
    } else {
        "it panicked with no words".into()
    }
}

impl Report {
    /// Whether every scenario passed.
    pub fn passed(&self) -> bool {
        self.failures.is_empty()
    }

    /// Returns when every scenario passed, and panics with the whole
    /// report when any failed, so a profile that fails the suite fails
    /// its test (93).
    pub fn assert(&self) {
        if !self.passed() {
            panic!("{self}");
        }
    }
}

fn scenarios(n: usize) -> String {
    match n {
        1 => "1 scenario".into(),
        n => format!("{n} scenarios"),
    }
}

fn clauses(cites: &[u16]) -> String {
    let numbers: Vec<String> = cites.iter().map(u16::to_string).collect();
    match numbers.as_slice() {
        [one] => format!("clause {one}"),
        _ => format!("clauses {}", numbers.join(", ")),
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the {} group over the profile `{}`: ",
            self.group, self.profile
        )?;
        if self.passed() {
            return write!(f, "{} passed", scenarios(self.ran));
        }
        write!(
            f,
            "{} of {} failed",
            self.failures.len(),
            scenarios(self.ran)
        )?;
        for failure in &self.failures {
            write!(f, "\n- `{}`", failure.scenario)?;
            if let Some(plays) = failure.plays {
                write!(f, ", playing {plays}")?;
            }
            write!(f, " ({}): {}", clauses(failure.cites), failure.why)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    /// A rig that records which scenarios ran on it.
    struct Probe {
        ran: Rc<RefCell<Vec<&'static str>>>,
    }

    impl Rig for Probe {
        fn profile(&self) -> String {
            "a probe".into()
        }
    }

    fn first(rig: &Probe) {
        rig.ran.borrow_mut().push("first");
        panic!("the first answered absent");
    }

    fn second(rig: &Probe) {
        rig.ran.borrow_mut().push("second");
    }

    fn third(rig: &Probe) {
        rig.ran.borrow_mut().push("third");
        assert_eq!(1, 2, "the third answered {}", "refused");
    }

    fn group() -> Vec<Scenario<Probe>> {
        vec![
            Scenario {
                name: "the first",
                plays: Some("S1"),
                cites: &[14, 17],
                run: first,
            },
            Scenario {
                name: "the second",
                plays: None,
                cites: &[66],
                run: second,
            },
            Scenario {
                name: "the third",
                plays: None,
                cites: &[1],
                run: third,
            },
        ]
    }

    fn probe(ran: &Rc<RefCell<Vec<&'static str>>>) -> impl Fn() -> Box<Probe> {
        let ran = ran.clone();
        move || Box::new(Probe { ran: ran.clone() })
    }

    #[test]
    fn two_failures_are_both_reported_and_every_scenario_runs() {
        let ran = Rc::new(RefCell::new(Vec::new()));
        let report = run("probe", &group(), &probe(&ran));
        assert_eq!(*ran.borrow(), ["first", "second", "third"]);
        assert_eq!(report.profile, "a probe");
        assert_eq!(report.group, "probe");
        assert_eq!(report.ran, 3);
        assert_eq!(
            report.failures,
            [
                Failure {
                    scenario: "the first",
                    plays: Some("S1"),
                    cites: &[14, 17],
                    why: "the first answered absent".into(),
                },
                Failure {
                    scenario: "the third",
                    plays: None,
                    cites: &[1],
                    why: "assertion `left == right` failed: the third answered refused\n  \
                          left: 1\n right: 2"
                        .into(),
                },
            ]
        );
        let text = report.to_string();
        for words in [
            "the probe group over the profile `a probe`: 2 of 3 scenarios failed",
            "- `the first`, playing S1 (clauses 14, 17): the first answered absent",
            "- `the third` (clause 1): assertion",
            "the third answered refused",
        ] {
            assert!(text.contains(words), "{text}\ndoes not say: {words}");
        }
        assert!(!text.contains("the second"), "{text}");
        let asserted = catch_unwind(AssertUnwindSafe(|| report.assert()))
            .expect_err("a report with failures asserted");
        assert_eq!(said(asserted.as_ref()), text);
    }

    #[test]
    fn a_group_that_passes_reports_nothing_and_asserts() {
        let ran = Rc::new(RefCell::new(Vec::new()));
        let passing: Vec<Scenario<Probe>> = group()
            .into_iter()
            .filter(|s| s.name == "the second")
            .collect();
        let report = run("probe", &passing, &probe(&ran));
        assert!(report.passed());
        assert_eq!(report.failures, []);
        assert_eq!(report.ran, 1);
        report.assert();
        assert_eq!(
            report.to_string(),
            "the probe group over the profile `a probe`: 1 scenario passed"
        );
    }

    #[test]
    fn each_scenario_runs_on_a_rig_of_its_own() {
        let made = Rc::new(RefCell::new(0));
        let ran = Rc::new(RefCell::new(Vec::new()));
        let make = || {
            *made.borrow_mut() += 1;
            Box::new(Probe { ran: ran.clone() })
        };
        run("probe", &group(), &make);
        assert_eq!(*made.borrow(), 3);
    }

    #[test]
    fn a_rig_that_cannot_be_made_fails_its_scenario_and_not_the_next() {
        let made = Rc::new(RefCell::new(0));
        let ran = Rc::new(RefCell::new(Vec::new()));
        let make = || {
            *made.borrow_mut() += 1;
            if *made.borrow() == 2 {
                panic!("no store answers at that address");
            }
            Box::new(Probe { ran: ran.clone() })
        };
        let report = run("probe", &group(), &make);
        assert_eq!(*ran.borrow(), ["first", "third"]);
        assert_eq!(report.failures[1].scenario, "the second");
        assert_eq!(
            report.failures[1].why,
            "the rig could not be made: no store answers at that address"
        );
    }
}
