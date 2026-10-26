//! How every command renders the core's answer: the exit code of each of
//! the four answers, the answer as one JSON document (57), a reason with
//! the source's own words on a line of their own (58, 59, rule core.19),
//! and a moment as a person reads a time in UTC.

use std::io::{self, Write};

use serde::Serialize;
use tailrace_core::answer_json;
use tailrace_seams::{Answer, Not};

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

/// `not` as a person reads it: tailrace's reason, then the source's own
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

/// `seconds` since the Unix epoch as an RFC 3339 time in UTC, the civil
/// date from the count of days by the proleptic Gregorian calendar.
pub fn utc(seconds: u64) -> String {
    let (days, of_day) = (seconds / 86_400, seconds % 86_400);
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let of_era = z.rem_euclid(146_097);
    let year_of_era = (of_era - of_era / 1_460 + of_era / 36_524 - of_era / 146_096) / 365;
    let of_year = of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted = (5 * of_year + 2) / 153;
    let day = of_year - (153 * shifted + 2) / 5 + 1;
    let month = if shifted < 10 {
        shifted + 3
    } else {
        shifted - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        of_day / 3_600,
        of_day % 3_600 / 60,
        of_day % 60
    )
}
