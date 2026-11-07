//! How every command renders the core's answer: the exit code of each of
//! the four answers, the answer as one JSON document (57), a reason with
//! the source's own words on a line of their own (58, 59, rule core.19),
//! a moment as a person reads a time in UTC, and one attestation told by
//! the signer that made it (80).

use std::io::{self, Write};

use serde::Serialize;
use timber_core::{answer_json, Told};
use timber_seams::{Answer, Not};

/// An answer.
pub const OK: i32 = 0;
/// A refusal.
pub const REFUSED: i32 = 1;
/// Arguments the command cannot read, as the parser exits.
pub const ARGUMENTS: i32 = 2;
/// An unreadable answer.
pub const UNREADABLE: i32 = 3;
/// An absence.
pub const ABSENT: i32 = 4;

/// The exit code of `not`.
pub fn code(not: &Not) -> i32 {
    match not {
        Not::Absent(_) => ABSENT,
        Not::Refused(_) => REFUSED,
        Not::Unreadable(_) => UNREADABLE,
    }
}

/// The exit code of `answer`.
pub fn exit<T>(answer: &Answer<T>) -> i32 {
    answer.as_ref().err().map_or(OK, code)
}

/// The worst of several exit codes: unreadable over refused over absent
/// over an answer.
pub fn worst(codes: impl IntoIterator<Item = i32>) -> i32 {
    let rank = |c: i32| match c {
        UNREADABLE => 3,
        REFUSED => 2,
        ABSENT => 1,
        _ => 0,
    };
    codes.into_iter().max_by_key(|c| rank(*c)).unwrap_or(OK)
}

/// `answer` as one JSON document, in the core's one form (57).
pub fn json<T: Serialize>(out: &mut dyn Write, answer: &Answer<T>) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *out, &answer_json(answer))?;
    writeln!(out)
}

/// `not` as a person reads it: timber's reason, then the source's own
/// words on a line of their own, unchanged (59).
pub fn not(out: &mut dyn Write, indent: &str, not: &Not) -> io::Result<()> {
    let mut reason = not.clone();
    let said = match &mut reason {
        Not::Absent(_) => None,
        Not::Refused(r) => r.said.take(),
        Not::Unreadable(u) => u.said.take(),
    };
    writeln!(out, "{indent}{reason}")?;
    if let Some(said) = said {
        out.write_all(said.as_bytes())?;
        if !said.ends_with('\n') {
            writeln!(out)?;
        }
    }
    Ok(())
}

/// 80, 116: `told` on one line: its kind, the check of a verdict or the
/// environment of a report (C.12), the digest it is attached to and the
/// provenance it was carried in, the named signer that made it or the
/// signers checked where none did, and whether it counts.
pub fn told(told: &Told) -> String {
    let kind = told
        .kind
        .map_or_else(|| "a claim in another form".to_string(), |k| k.to_string());
    let of = match (&told.check, &told.environment) {
        (Some(check), _) => format!(" of {check}"),
        (None, Some(environment)) => format!(" of {environment}"),
        (None, None) => String::new(),
    };
    let carried = told
        .carried_in
        .as_ref()
        .map_or_else(String::new, |c| format!(", carried in {c}"));
    let made = match &told.made_by {
        Some(signer) if told.expired => format!("made by {signer}, expired"),
        Some(signer) => format!("made by {signer}"),
        None => {
            let checked: Vec<&str> = told.checked.iter().map(|s| s.as_str()).collect();
            format!("made by no named signer (checked {})", checked.join(", "))
        }
    };
    let counts = if told.counts {
        "counts"
    } else {
        "kept, does not count"
    };
    format!("{kind}{of} on {}{carried}: {made}; {counts}", told.on)
}

/// `seconds` since the Unix epoch as an RFC 3339 time in UTC, as every
/// answer names a moment.
pub fn utc(seconds: u64) -> String {
    timber_seams::utc(seconds)
}

#[cfg(test)]
mod tests {
    use timber_seams::{
        BuildId, ClaimKind, Digest, EnvironmentName, OutputName, Seam, SignerId, Source, Why,
    };

    use super::*;

    fn report(made_by: Option<&str>, counts: bool) -> Told {
        Told {
            on: Digest::of(b"index"),
            envelope: Digest::of(b"envelope"),
            kind: Some(ClaimKind::Report),
            check: None,
            environment: Some(EnvironmentName::new("srv")),
            made_by: made_by.map(SignerId::new),
            expired: false,
            counts,
            checked: vec![SignerId::new("deployer"), SignerId::new("gate")],
            carried_in: None,
        }
    }

    #[test]
    fn a_report_is_one_line_naming_its_environment_and_its_signer() {
        let index = Digest::of(b"index");
        assert_eq!(
            told(&report(Some("deployer"), true)),
            format!("report of srv on {index}: made by deployer; counts")
        );
        assert_eq!(
            told(&report(None, false)),
            format!(
                "report of srv on {index}: made by no named signer (checked deployer, gate); \
                 kept, does not count"
            )
        );
    }

    fn printed(n: &Not) -> String {
        let mut out = Vec::new();
        not(&mut out, "  ", n).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn an_output_whose_entry_the_store_lacks_is_one_line_naming_both() {
        let refused = Not::refused(
            &Source::new(Seam::Store, "bound"),
            Why::NotInStore {
                output: OutputName::new("site"),
                build: BuildId::new("b7e1"),
            },
            None,
        );
        assert_eq!(
            printed(&refused),
            "  the store (bound) refused: site from b7e1: no cache entry in the store\n"
        );
        assert_eq!(code(&refused), REFUSED);
    }

    #[test]
    fn the_sources_words_follow_on_lines_of_their_own() {
        let refused = Not::refused(
            &Source::new(Seam::Store, "bound"),
            Why::Said,
            Some("denied".into()),
        );
        assert_eq!(
            printed(&refused),
            "  the store (bound) refused: it refused\ndenied\n"
        );
    }
}
