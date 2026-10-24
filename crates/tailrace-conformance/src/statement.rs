//! The clauses of Parts A and B of the statement, by section, in the
//! order the statement gives them (section 12; design D7). This list
//! changes only in the commit that adopts a change of the statement.

/// The statement's revision the clauses were read at.
pub const READ_AT: &str = "agentplot/blueprints 5b00e15";

/// Each section of Parts A and B, and its clauses.
pub const PARTS: &[(&str, &[u16])] = &[
    ("A.1 Artifacts and digests", &[1, 2, 3, 4, 5, 6]),
    (
        "A.2 Claims and attestations",
        &[7, 8, 9, 116, 10, 11, 12, 13],
    ),
    (
        "A.3 Publishing a landing",
        &[14, 15, 16, 111, 17, 18, 19, 20],
    ),
    ("A.4 Versions", &[21, 22, 23, 24, 110, 123, 124]),
    (
        "A.5 Acceptance",
        &[25, 94, 117, 95, 26, 27, 96, 108, 28, 29, 107, 30, 112],
    ),
    (
        "A.6 Channels and promotion",
        &[
            31, 125, 32, 33, 34, 35, 105, 109, 36, 37, 38, 39, 101, 106, 102, 103, 104,
        ],
    ),
    ("A.7 Environments", &[40, 41, 97, 42, 43, 119, 98, 44, 45]),
    (
        "A.8 Consumers",
        &[46, 47, 113, 99, 48, 49, 114, 50, 115, 51],
    ),
    ("A.9 The cache", &[52, 100, 53, 54, 55]),
    ("A.10 Askers and surfaces", &[56, 57, 58, 59, 60]),
    ("A.11 What tailrace keeps", &[61, 62, 63, 64, 65]),
    ("B.1 The store", &[66, 67, 68, 69, 70, 71, 72, 73]),
    ("B.2 The build", &[74, 75, 76]),
    ("B.3 The landing", &[77, 78]),
    ("B.4 The signer", &[79, 80, 81]),
    ("B.5 The policy", &[82, 83]),
    ("B.6 A projection", &[84, 85, 86, 87]),
    ("B.7 A target", &[88, 89, 90, 91]),
    ("B.8 The keeper", &[120, 121, 122]),
    ("B.9 Binding the contracts", &[92, 118, 93]),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_parts_hold_every_clause_from_1_to_125_once_in_20_sections() {
        assert_eq!(PARTS.len(), 20);
        let mut clauses: Vec<u16> = PARTS.iter().flat_map(|(_, c)| c.iter().copied()).collect();
        assert_eq!(clauses.len(), 125);
        clauses.sort_unstable();
        assert_eq!(clauses, (1..=125).collect::<Vec<u16>>());
        let sections: Vec<&str> = PARTS
            .iter()
            .map(|(s, _)| s.split(' ').next().unwrap())
            .collect();
        let named: Vec<String> = (1..=11)
            .map(|n| format!("A.{n}"))
            .chain((1..=9).map(|n| format!("B.{n}")))
            .collect();
        assert_eq!(sections, named);
        assert!(READ_AT.starts_with("agentplot/blueprints "));
    }
}
