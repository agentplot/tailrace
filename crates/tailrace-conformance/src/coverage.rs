//! The coverage check: every clause of Parts A and B cited by a scenario
//! or named in the register, and nothing else, and every part the
//! register names of a clause a scenario cites (section 12, rule core.33,
//! ADR 0012; design D8). It finds what is missing or malformed; whether a
//! citation is honest is the reviewer's to judge.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::register::{Part, Pending};
use crate::Cited;

/// What the check finds wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Finding {
    /// No scenario cites the clause and the register does not name it.
    Uncovered { clause: u16 },
    /// A scenario cites the clause and the register still names it.
    CitedAndRegistered {
        clause: u16,
        scenarios: Vec<(&'static str, &'static str)>,
    },
    /// A scenario cites a number that is no clause of Parts A and B.
    CitesNoClause {
        group: &'static str,
        scenario: &'static str,
        number: u16,
    },
    /// A scenario cites nothing.
    CitesNothing {
        group: &'static str,
        scenario: &'static str,
    },
    /// The register names a number that is no clause of Parts A and B.
    RegistersNoClause { number: u16 },
    /// A register entry names no unit, or no reason.
    Unexplained { clause: u16, unit: bool, why: bool },
    /// The register names the clause more than once.
    RegisteredTwice { clause: u16 },
    /// The register names a part of the clause, which no scenario cites,
    /// so it waits whole.
    PartOfUncited { clause: u16 },
}

/// What the check came to: the findings, the clauses that wait, the parts
/// of cited clauses that wait, the clauses cited, and how many clauses
/// there are, how many are cited and how many registered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Coverage {
    pub findings: Vec<Finding>,
    pub pending: Vec<Pending>,
    pub parts: Vec<Part>,
    pub cited_clauses: Vec<u16>,
    pub clauses: usize,
    pub cited: usize,
    pub registered: usize,
}

impl Coverage {
    pub fn passed(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Reads `citations`, `pending` and `waiting`, the parts of clauses that
/// wait, against the clauses `parts` holds.
pub fn check(
    parts: &[(&str, &[u16])],
    citations: &[Cited],
    pending: &[Pending],
    waiting: &[Part],
) -> Coverage {
    let clauses: BTreeSet<u16> = parts.iter().flat_map(|(_, c)| c.iter().copied()).collect();
    let mut findings = Vec::new();

    let mut citing: BTreeMap<u16, Vec<(&'static str, &'static str)>> = BTreeMap::new();
    for cited in citations {
        if cited.cites.is_empty() {
            findings.push(Finding::CitesNothing {
                group: cited.group,
                scenario: cited.name,
            });
        }
        for &number in cited.cites {
            if clauses.contains(&number) {
                citing
                    .entry(number)
                    .or_default()
                    .push((cited.group, cited.name));
            } else {
                findings.push(Finding::CitesNoClause {
                    group: cited.group,
                    scenario: cited.name,
                    number,
                });
            }
        }
    }

    let mut registered: BTreeMap<u16, usize> = BTreeMap::new();
    for entry in pending {
        *registered.entry(entry.clause).or_default() += 1;
        if !clauses.contains(&entry.clause) {
            findings.push(Finding::RegistersNoClause {
                number: entry.clause,
            });
        }
        let (unit, why) = (entry.unit.trim().is_empty(), entry.why.trim().is_empty());
        if unit || why {
            findings.push(Finding::Unexplained {
                clause: entry.clause,
                unit,
                why,
            });
        }
    }
    for (&clause, &times) in &registered {
        if times > 1 {
            findings.push(Finding::RegisteredTwice { clause });
        }
    }

    for part in waiting {
        if !citing.contains_key(&part.clause) {
            findings.push(Finding::PartOfUncited {
                clause: part.clause,
            });
        }
        let (unit, why) = (part.unit.trim().is_empty(), part.why.trim().is_empty());
        if unit || why {
            findings.push(Finding::Unexplained {
                clause: part.clause,
                unit,
                why,
            });
        }
    }

    for &clause in &clauses {
        match (citing.get(&clause), registered.contains_key(&clause)) {
            (None, false) => findings.push(Finding::Uncovered { clause }),
            (Some(scenarios), true) => findings.push(Finding::CitedAndRegistered {
                clause,
                scenarios: scenarios.clone(),
            }),
            _ => {}
        }
    }

    Coverage {
        findings,
        pending: pending.to_vec(),
        parts: waiting.to_vec(),
        cited_clauses: citing.keys().copied().collect(),
        clauses: clauses.len(),
        cited: citing.len(),
        registered: registered.keys().filter(|c| clauses.contains(c)).count(),
    }
}

fn scenarios(scenarios: &[(&str, &str)]) -> String {
    scenarios
        .iter()
        .map(|(group, name)| format!("`{name}` of {group}"))
        .collect::<Vec<_>>()
        .join(", ")
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::Uncovered { clause } => write!(
                f,
                "uncovered {clause}: no scenario cites it and the register does not name it"
            ),
            Finding::CitedAndRegistered {
                clause,
                scenarios: by,
            } => write!(
                f,
                "cited and registered {clause}: {} cites it, so it leaves the register",
                scenarios(by)
            ),
            Finding::CitesNoClause {
                group,
                scenario,
                number,
            } => write!(
                f,
                "no such clause {number}: `{scenario}` of {group} cites it"
            ),
            Finding::CitesNothing { group, scenario } => {
                write!(f, "cites nothing: `{scenario}` of {group}")
            }
            Finding::RegistersNoClause { number } => {
                write!(f, "no such clause {number}: the register names it")
            }
            Finding::Unexplained { clause, unit, why } => {
                let missing = match (unit, why) {
                    (true, true) => "no unit and no reason",
                    (true, false) => "no unit",
                    _ => "no reason",
                };
                write!(
                    f,
                    "unexplained {clause}: the register names it with {missing}"
                )
            }
            Finding::RegisteredTwice { clause } => {
                write!(
                    f,
                    "registered twice {clause}: the register names it more than once"
                )
            }
            Finding::PartOfUncited { clause } => write!(
                f,
                "part of uncited {clause}: the register names a part of it, and no scenario cites it, so it waits whole"
            ),
        }
    }
}

impl fmt::Display for Coverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for entry in &self.pending {
            writeln!(f, "pending {}  {}  {}", entry.clause, entry.unit, entry.why)?;
        }
        for part in &self.parts {
            writeln!(
                f,
                "pending part of {}, {}  {}  {}",
                part.clause, part.part, part.unit, part.why
            )?;
        }
        let cited: Vec<String> = self.cited_clauses.iter().map(u16::to_string).collect();
        writeln!(f, "cited {}", cited.join(", "))?;
        write!(
            f,
            "{} clauses: {} cited, {} pending",
            self.clauses, self.cited, self.registered
        )?;
        for finding in &self.findings {
            write!(f, "\n{finding}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PARTS: &[(&str, &[u16])] = &[("A.1", &[1, 2, 3]), ("A.2", &[7, 8])];

    fn cited(name: &'static str, cites: &'static [u16]) -> Cited {
        Cited {
            group: "store",
            name,
            plays: None,
            cites,
        }
    }

    fn pending(clause: u16, unit: &'static str, why: &'static str) -> Pending {
        Pending { clause, unit, why }
    }

    const UNIT: &str = "bolt 4 · the gate";
    const WHY: &str = "acceptance is the gate's";

    #[test]
    fn a_clause_neither_cited_nor_registered_is_named() {
        let coverage = check(
            PARTS,
            &[cited("one", &[1, 2]), cited("two", &[3, 8])],
            &[],
            &[],
        );
        assert_eq!(coverage.findings, [Finding::Uncovered { clause: 7 }]);
        assert!(coverage.to_string().contains("uncovered 7"), "{coverage}");
    }

    #[test]
    fn an_entry_without_its_unit_or_reason_is_named() {
        let coverage = check(
            PARTS,
            &[cited("one", &[1, 2, 3])],
            &[pending(7, UNIT, " "), pending(8, "", WHY)],
            &[],
        );
        assert_eq!(
            coverage.findings,
            [
                Finding::Unexplained {
                    clause: 7,
                    unit: false,
                    why: true
                },
                Finding::Unexplained {
                    clause: 8,
                    unit: true,
                    why: false
                },
            ]
        );
        let text = coverage.to_string();
        assert!(
            text.contains("unexplained 7: the register names it with no reason"),
            "{text}"
        );
        assert!(
            text.contains("unexplained 8: the register names it with no unit"),
            "{text}"
        );
    }

    #[test]
    fn a_clause_cited_and_registered_is_named_with_its_scenario() {
        let coverage = check(
            PARTS,
            &[cited("one", &[1, 2, 3, 8]), cited("the gate accepts", &[7])],
            &[pending(7, UNIT, WHY)],
            &[],
        );
        assert_eq!(
            coverage.findings,
            [Finding::CitedAndRegistered {
                clause: 7,
                scenarios: vec![("store", "the gate accepts")]
            }]
        );
        assert!(
            coverage
                .to_string()
                .contains("cited and registered 7: `the gate accepts` of store cites it"),
            "{coverage}"
        );
    }

    #[test]
    fn a_citation_of_no_clause_and_a_scenario_citing_nothing_are_named() {
        let coverage = check(
            PARTS,
            &[cited("one", &[1, 2, 3, 7, 8, 126]), cited("none", &[])],
            &[],
            &[],
        );
        assert_eq!(
            coverage.findings,
            [
                Finding::CitesNoClause {
                    group: "store",
                    scenario: "one",
                    number: 126
                },
                Finding::CitesNothing {
                    group: "store",
                    scenario: "none"
                },
            ]
        );
        let text = coverage.to_string();
        assert!(
            text.contains("no such clause 126: `one` of store cites it"),
            "{text}"
        );
        assert!(text.contains("cites nothing: `none` of store"), "{text}");
    }

    #[test]
    fn a_register_entry_of_no_clause_is_named() {
        let coverage = check(
            PARTS,
            &[cited("one", &[1, 2, 3, 7, 8])],
            &[pending(126, UNIT, WHY)],
            &[],
        );
        assert_eq!(
            coverage.findings,
            [Finding::RegistersNoClause { number: 126 }]
        );
    }

    #[test]
    fn a_clause_registered_twice_is_named() {
        let coverage = check(
            PARTS,
            &[cited("one", &[1, 2, 3, 8])],
            &[pending(7, UNIT, WHY), pending(7, "bolt 5 · render", WHY)],
            &[],
        );
        assert_eq!(coverage.findings, [Finding::RegisteredTwice { clause: 7 }]);
    }

    #[test]
    fn a_pass_returns_the_pending_entries_and_the_counts() {
        let entries = [
            pending(7, UNIT, WHY),
            pending(8, "bolt 5 · render", "layouts"),
        ];
        let coverage = check(
            PARTS,
            &[cited("one", &[1, 2]), cited("two", &[3])],
            &entries,
            &[part(3, "its environments", UNIT, WHY)],
        );
        assert!(coverage.passed(), "{coverage}");
        assert_eq!(coverage.pending, entries);
        assert_eq!(coverage.cited_clauses, [1, 2, 3]);
        assert_eq!(
            (coverage.clauses, coverage.cited, coverage.registered),
            (5, 3, 2)
        );
        assert_eq!(
            coverage.to_string(),
            "pending 7  bolt 4 · the gate  acceptance is the gate's\n\
             pending 8  bolt 5 · render  layouts\n\
             pending part of 3, its environments  bolt 4 · the gate  acceptance is the gate's\n\
             cited 1, 2, 3\n\
             5 clauses: 3 cited, 2 pending"
        );
    }

    fn part(clause: u16, of: &'static str, unit: &'static str, why: &'static str) -> Part {
        Part {
            clause,
            part: of,
            unit,
            why,
        }
    }

    #[test]
    fn a_part_of_a_clause_no_scenario_cites_is_named_waiting_whole() {
        let coverage = check(
            PARTS,
            &[cited("one", &[1, 2, 3, 8])],
            &[pending(7, UNIT, WHY)],
            &[part(7, "its notes", UNIT, WHY)],
        );
        assert_eq!(coverage.findings, [Finding::PartOfUncited { clause: 7 }]);
        assert!(
            coverage
                .to_string()
                .contains("part of uncited 7: the register names a part of it, and no scenario cites it, so it waits whole"),
            "{coverage}"
        );
    }

    #[test]
    fn a_part_without_its_unit_or_reason_is_named() {
        let coverage = check(
            PARTS,
            &[cited("one", &[1, 2, 3, 7, 8])],
            &[],
            &[
                part(7, "its notes", "", WHY),
                part(8, "its counts", UNIT, ""),
            ],
        );
        assert_eq!(
            coverage.findings,
            [
                Finding::Unexplained {
                    clause: 7,
                    unit: true,
                    why: false
                },
                Finding::Unexplained {
                    clause: 8,
                    unit: false,
                    why: true
                },
            ]
        );
    }
}
