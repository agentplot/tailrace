//! What a policy lists to verify a host key's signer (design D5, D6):
//! ssh-keygen's allowed-signers form with the principal left out, since
//! the signer's id is the principal: `[options] keytype base64 [comment]`,
//! where the one option read is `valid-before="<time>Z"`.

use ssh_key::PublicKey;
use tailrace_seams::{Answer, NamedSigner, Not};

use super::HostKey;

/// A host key as a policy lists it: its public half, and where the
/// listing carries one, the second since the epoch from which a signature
/// by it is told as expired (81).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Listed {
    pub key: PublicKey,
    pub valid_before: Option<i64>,
}

/// `named`'s listing: `None` when no word of its verifier names an SSH
/// key type, as another profile's signer, and unreadable naming the
/// signer when one does and the listing cannot be read, so a broken
/// listing is never passed over (58).
pub(super) fn read(named: &NamedSigner) -> Answer<Option<Listed>> {
    let words: Vec<&str> = named.verified_by.split_whitespace().collect();
    let Some(at) = words.iter().position(|w| is_key_type(w)) else {
        return Ok(None);
    };
    let broken = |said: String| {
        Not::unreadable(
            &HostKey::source(),
            format!("the key the policy lists for the signer {}", named.id),
            Some(said),
        )
    };
    let valid_before = match words[..at] {
        [] => None,
        [given] => Some(options(given).map_err(broken)?),
        _ => {
            let said =
                format!("{at} words come before the key type, and one, its options, is read");
            return Err(broken(said));
        }
    };
    let Some(body) = words.get(at + 1) else {
        return Err(broken(format!("no key follows the key type {}", words[at])));
    };
    let key = PublicKey::from_openssh(&format!("{} {body}", words[at]))
        .map_err(|e| broken(e.to_string()))?;
    Ok(Some(Listed { key, valid_before }))
}

/// A word naming an SSH key type, as an allowed-signers line has one.
fn is_key_type(word: &str) -> bool {
    word.starts_with("ssh-") || word.starts_with("ecdsa-") || word.starts_with("sk-")
}

/// The listing's options, `valid-before` the one read, its time in double
/// quotes as ssh-keygen writes it.
fn options(given: &str) -> Result<i64, String> {
    let mut found = None;
    for option in given.split(',') {
        let value = option
            .split_once('=')
            .filter(|(name, _)| name.eq_ignore_ascii_case("valid-before"))
            .map(|(_, value)| value)
            .ok_or_else(|| {
                format!("{option} is not valid-before, the one option a host key's listing reads")
            })?;
        let time = value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .ok_or_else(|| format!("the valid-before time {value} is not in double quotes"))?;
        if found.replace(seconds(time)?).is_some() {
            return Err(format!("valid-before is given twice in {given}"));
        }
    }
    found.ok_or_else(|| format!("{given} names no valid-before"))
}

/// A `valid-before` time in ssh-keygen's forms, `YYYYMMDD` or
/// `YYYYMMDDHHMM[SS]`, ending in `Z`, as seconds since the epoch: a time
/// without `Z` is read by ssh-keygen in the host's own zone, and a policy
/// answers the same on every host (design D6).
fn seconds(time: &str) -> Result<i64, String> {
    let digits = time.strip_suffix('Z').ok_or_else(|| {
        format!("the valid-before time {time} must end in Z, so every host reads it in UTC")
    })?;
    let unformed =
        || format!("the valid-before time {time} is not YYYYMMDD or YYYYMMDDHHMM[SS] and Z");
    if ![8, 12, 14].contains(&digits.len()) || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(unformed());
    }
    let field = |from: usize| {
        digits
            .get(from..from + 2)
            .map_or(0, |two| two.parse::<i64>().unwrap_or(0))
    };
    let year = digits[..4].parse::<i64>().map_err(|_| unformed())?;
    let (month, day) = (field(4), field(6));
    let (hour, minute, second) = (field(8), field(10), field(12));
    if !(1..=12).contains(&month)
        || !(1..=days_in(year, month)).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return Err(format!("the valid-before time {time} names no such time"));
    }
    Ok(days(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second)
}

fn days_in(year: i64, month: i64) -> i64 {
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// The days from 1970-01-01 to a date of the proleptic Gregorian calendar.
fn days(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let of_era = year - era * 400;
    let of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let of_cycle = of_era * 365 + of_era / 4 - of_era / 100 + of_year;
    era * 146_097 + of_cycle - 719_468
}

#[cfg(test)]
mod tests {
    use ssh_key::rand_core::OsRng;
    use ssh_key::{Algorithm, PrivateKey};
    use tailrace_seams::{Seam, SignerId};

    use super::*;

    fn named(verified_by: &str) -> NamedSigner {
        NamedSigner {
            id: SignerId::new("host"),
            verified_by: verified_by.into(),
            kinds: vec![],
            checks: vec![],
        }
    }

    fn public() -> String {
        PrivateKey::random(&mut OsRng, Algorithm::Ed25519)
            .unwrap()
            .public_key()
            .to_openssh()
            .unwrap()
    }

    /// The words an unreadable listing carries, once it names the signer.
    fn broken(verified_by: &str) -> String {
        match read(&named(verified_by)) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source.seam, Seam::Signer);
                assert_eq!(u.what, "the key the policy lists for the signer host");
                u.said.expect("the words it was read with")
            }
            other => panic!("{verified_by} read as {other:?}, not unreadable"),
        }
    }

    #[test]
    fn the_times_are_seconds_since_the_epoch_in_utc() {
        assert_eq!(seconds("20000101Z"), Ok(946_684_800));
        assert_eq!(seconds("19700101Z"), Ok(0));
        assert_eq!(
            seconds("202701011230Z"),
            Ok(1_798_761_600 + 12 * 3_600 + 30 * 60)
        );
        assert_eq!(
            seconds("20270101123045Z"),
            Ok(1_798_761_600 + 12 * 3_600 + 30 * 60 + 45)
        );
        assert_eq!(seconds("20240229Z"), Ok(1_709_164_800));
        assert_eq!(seconds("20991231235959Z"), Ok(4_102_444_799));
    }

    #[test]
    fn a_time_that_is_none_of_ssh_keygens_forms_is_refused_in_words() {
        for time in ["2027010Z", "202701011Z", "2027-01-01Z", "Z"] {
            assert!(
                seconds(time).unwrap_err().contains("is not YYYYMMDD"),
                "{time}"
            );
        }
        for time in [
            "20230229Z",
            "20271301Z",
            "20270132Z",
            "202701012400Z",
            "20270101235960Z",
        ] {
            assert!(
                seconds(time).unwrap_err().contains("names no such time"),
                "{time}"
            );
        }
    }

    #[test]
    fn a_listed_key_reads_with_its_comment_and_without_a_time() {
        let key = public();
        let listed = read(&named(&format!("{key} host@laptop")))
            .unwrap()
            .unwrap();
        assert_eq!(listed.key, PublicKey::from_openssh(&key).unwrap());
        assert_eq!(listed.valid_before, None);
    }

    #[test]
    fn a_listed_key_reads_with_its_valid_before() {
        let key = public();
        let listed = read(&named(&format!("valid-before=\"20270101Z\" {key}")))
            .unwrap()
            .unwrap();
        assert_eq!(listed.key, PublicKey::from_openssh(&key).unwrap());
        assert_eq!(listed.valid_before, Some(1_798_761_600));
    }

    #[test]
    fn a_time_without_its_zone_is_unreadable_naming_the_signer() {
        let said = broken(&format!("valid-before=\"20991231\" {}", public()));
        assert_eq!(
            said,
            "the valid-before time 20991231 must end in Z, so every host reads it in UTC"
        );
    }

    #[test]
    fn another_option_is_unreadable_naming_the_signer() {
        let said = broken(&format!("valid-after=\"20000101Z\" {}", public()));
        assert!(
            said.starts_with("valid-after=\"20000101Z\" is not valid-before"),
            "{said}"
        );
        let said = broken(&format!(
            "valid-before=\"20991231Z\",cert-authority {}",
            public()
        ));
        assert!(
            said.starts_with("cert-authority is not valid-before"),
            "{said}"
        );
        let said = broken(&format!("valid-before=20991231Z {}", public()));
        assert!(said.contains("is not in double quotes"), "{said}");
    }

    #[test]
    fn a_broken_key_is_unreadable_in_the_parsers_words() {
        let words = PublicKey::from_openssh("ssh-ed25519 notakey")
            .unwrap_err()
            .to_string();
        assert_eq!(broken("ssh-ed25519 notakey"), words);
        assert!(broken("ssh-ed25519").contains("no key follows the key type ssh-ed25519"));
        assert!(broken(&format!("host@laptop extra {}", public())).contains("2 words"));
    }

    #[test]
    fn a_verifier_naming_no_ssh_key_type_is_passed_over() {
        for verified_by in [
            "arn:aws:kms:us-east-1:111122223333:key/1234abcd",
            "a key's name",
            "",
        ] {
            assert_eq!(read(&named(verified_by)), Ok(None), "{verified_by}");
        }
    }
}
