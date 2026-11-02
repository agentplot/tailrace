//! A name as a tag (design D8): every byte of the name's UTF-8 in
//! `[A-Za-z0-9.-]` kept, every other byte, `_` among them, written `_`
//! and two lower-case hex digits, and a leading `.` or `-` written the
//! same way, so one name is one tag and a version's name reads as itself;
//! and a name as one path component of a repository (design D12), as the
//! distribution protocol allows, lowercased.

use tailrace_seams::{Name, ALGORITHM};

/// The longest tag the distribution specification allows.
const LONGEST: usize = 128;

/// The longest repository path, host included, the distribution
/// specification lets a client rely on (design D12).
const LONGEST_PATH: usize = 255;

/// The tag `name` is held under, or the words naming why no tag can
/// hold it: an empty name, one escaping past 128 characters, and one in
/// the shape of the referrers tag, which the store keeps for a subject's
/// referrers (design D6).
pub(super) fn of(name: &Name) -> Result<String, String> {
    let mut tag = String::new();
    for (i, byte) in name.as_str().bytes().enumerate() {
        let kept = byte.is_ascii_alphanumeric() || (i > 0 && matches!(byte, b'.' | b'-'));
        if kept {
            tag.push(char::from(byte));
        } else {
            tag.push_str(&format!("_{byte:02x}"));
        }
    }
    if tag.is_empty() {
        return Err("the empty name, which no tag can hold".into());
    }
    if tag.len() > LONGEST {
        return Err(format!(
            "the name {name}, whose tag {tag} is {} characters, past the {LONGEST} a tag holds",
            tag.len()
        ));
    }
    if is_referrers(&tag) {
        return Err(format!(
            "the name {name}, whose tag {tag} is a subject's referrers tag"
        ));
    }
    Ok(tag)
}

/// C.1: `text` lowercased and written as one path component the
/// distribution protocol allows, `[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*`:
/// each byte of `[a-z0-9]` kept; a `.` or `-` kept where the byte before
/// it was kept as a letter or digit and the byte after it is one; every
/// other byte, `_` among them, written `_` and two lower-case hex digits;
/// and a first byte not in `[a-z1-9]` written so and the component begun
/// with `0`, so two texts that differ once lowercased never share one
/// (design D12).
pub(super) fn component(text: &str) -> String {
    let lower = text.to_lowercase();
    let bytes = lower.as_bytes();
    let alnum = |b: Option<&u8>| b.is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit());
    let mut out = String::new();
    let mut kept_alnum = false;
    for (i, &byte) in bytes.iter().enumerate() {
        let keep = if i == 0 {
            byte.is_ascii_lowercase() || matches!(byte, b'1'..=b'9')
        } else if matches!(byte, b'.' | b'-') {
            kept_alnum && alnum(bytes.get(i + 1))
        } else {
            alnum(Some(&byte))
        };
        if keep {
            out.push(char::from(byte));
        } else {
            if i == 0 {
                out.push('0');
            }
            out.push_str(&format!("_{byte:02x}"));
        }
        kept_alnum = keep && alnum(Some(&byte));
    }
    out
}

/// The repository path `parts` make under `declared`, each written as one
/// path component, for what `held` names as written; refused naming it,
/// the path and its length where the path runs, with `host` before it,
/// past 255 characters (design D12).
pub(super) fn repository(
    host: &str,
    declared: &str,
    parts: &[&str],
    held: &str,
) -> Result<String, String> {
    let mut path = declared.to_string();
    for part in parts {
        path.push('/');
        path.push_str(&component(part));
    }
    let length = host.len() + 1 + path.len();
    if length > LONGEST_PATH {
        return Err(format!(
            "{held} is held in the repository {host}/{path}, which is {length} characters, past \
             the {LONGEST_PATH} a repository's path holds"
        ));
    }
    Ok(path)
}

/// [`of`] read back: the text of the name `tag` was written for, each
/// `_` and two hex digits its byte; none for a tag `of` could not have
/// written (design D8).
pub(super) fn read(tag: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let mut rest = tag.as_bytes();
    while let Some((&first, after)) = rest.split_first() {
        if first == b'_' {
            let hex = std::str::from_utf8(after.get(..2)?).ok()?;
            if hex.bytes().any(|b| b.is_ascii_uppercase()) {
                return None;
            }
            bytes.push(u8::from_str_radix(hex, 16).ok()?);
            rest = &after[2..];
        } else {
            bytes.push(first);
            rest = after;
        }
    }
    let text = String::from_utf8(bytes).ok()?;
    (of(&Name::new(&text)).ok()? == tag).then_some(text)
}

/// The referrers tag of the digest whose hex is `hex` (design D6).
pub(super) fn referrers(hex: &str) -> String {
    format!("{ALGORITHM}-{hex}")
}

fn is_referrers(tag: &str) -> bool {
    tag.strip_prefix(ALGORITHM)
        .and_then(|rest| rest.strip_prefix('-'))
        .is_some_and(|hex| {
            hex.len() == 64 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        })
}

#[cfg(test)]
mod tests {
    use tailrace_seams::Digest;

    use super::*;

    fn tag(name: &str) -> String {
        of(&Name::new(name)).unwrap()
    }

    /// What the distribution specification lets a tag be.
    fn valid(tag: &str) -> bool {
        let mut bytes = tag.bytes();
        let first = bytes
            .next()
            .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_');
        first
            && tag.len() <= LONGEST
            && bytes.all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    }

    #[test]
    fn a_version_name_is_its_own_tag() {
        for name in ["lib-v1.3.0", "v0.1.0", "web-v1.0.0-rc.1"] {
            assert_eq!(tag(name), name);
        }
    }

    #[test]
    fn names_holding_what_no_tag_holds_map_to_valid_distinct_tags() {
        let names = [
            "a/b+c",
            "a_2fb+c",
            "_x",
            "a/b",
            "a_2fb",
            "@acme/site",
            "caf\u{e9}",
        ];
        let tags: Vec<String> = names.iter().map(|n| tag(n)).collect();
        assert_eq!(tags[0], "a_2fb_2bc");
        assert_eq!(tags[1], "a_5f2fb_2bc");
        assert_eq!(tags[2], "_5fx");
        assert_eq!(tags[6], "caf_c3_a9");
        for t in &tags {
            assert!(valid(t), "{t} is no valid tag");
        }
        let mut distinct = tags.clone();
        distinct.sort();
        distinct.dedup();
        assert_eq!(distinct.len(), tags.len(), "{tags:?}");
    }

    #[test]
    fn a_leading_dot_or_dash_is_escaped() {
        assert_eq!(tag(".hidden"), "_2ehidden");
        assert_eq!(tag("-flag"), "_2dflag");
        assert_eq!(tag("a.-b"), "a.-b");
        assert!(valid(&tag(".hidden")) && valid(&tag("-flag")));
    }

    /// What the distribution specification lets a path component be,
    /// `[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*`.
    fn a_component(text: &str) -> bool {
        let alnum = |b: &u8| b.is_ascii_lowercase() || b.is_ascii_digit();
        let bytes = text.as_bytes();
        let mut i = 0;
        loop {
            let run = bytes[i..].iter().take_while(|b| alnum(b)).count();
            if run == 0 {
                return false;
            }
            i += run;
            let separator = match bytes.get(i) {
                None => return true,
                Some(b'.') => 1,
                Some(b'_') if bytes.get(i + 1) == Some(&b'_') => 2,
                Some(b'_') => 1,
                Some(b'-') => bytes[i..].iter().take_while(|b| **b == b'-').count(),
                Some(_) => return false,
            };
            i += separator;
        }
    }

    #[test]
    fn a_name_is_one_path_component_lowercased_and_escaped() {
        assert_eq!(component("@acme/site"), "0_40acme_2fsite");
        assert_eq!(component("tailrace-core"), "tailrace-core");
        assert_eq!(component("0x"), "0_30x");
        assert_eq!(component("lib"), "lib");
        assert_eq!(component("Lib"), "lib");
        assert_eq!(component("moon"), "moon");
    }

    #[test]
    fn every_component_is_valid_and_distinct_for_distinct_lowercased_names() {
        let names = [
            "@acme/site",
            "acme-site",
            "acme_site",
            "acme.site",
            "acme--site",
            "acme..site",
            "a-",
            "-a",
            ".a",
            "_a",
            "0",
            "0a",
            "00",
            "1",
            "a",
            "site.",
            "caf\u{e9}",
            "a/b",
            "a_2fb",
            "0_2fb",
            "x__y",
            "UPPER",
            "1.0",
            "a-.b",
            "",
        ];
        let mut seen = std::collections::BTreeMap::new();
        for name in names {
            let written = component(name);
            if !name.is_empty() {
                assert!(a_component(&written), "{name} wrote {written}");
            }
            if let Some(other) = seen.insert(written.clone(), name.to_lowercase()) {
                panic!("{name} and {other} share {written}");
            }
        }
    }

    #[test]
    fn a_repository_path_past_255_characters_is_refused_naming_it() {
        let host = "registry.example";
        assert_eq!(
            repository(
                host,
                "acme/tailrace",
                &["artifact", "npm", "@acme/site"],
                "the name artifact/npm/@acme/site@1.3.0"
            ),
            Ok("acme/tailrace/artifact/npm/0_40acme_2fsite".into())
        );
        let long = "a".repeat(255 - host.len() - 1 - "acme/tailrace/".len());
        let fits = repository(host, "acme/tailrace", &[&long], "the component").unwrap();
        assert_eq!(host.len() + 1 + fits.len(), 255);
        let past = format!("{long}b");
        let words = repository(host, "acme/tailrace", &[&past], "the component").unwrap_err();
        assert!(words.starts_with("the component is held in "), "{words}");
        assert!(
            words.contains(&format!("{host}/acme/tailrace/{past}")),
            "{words}"
        );
        assert!(words.contains("256 characters"), "{words}");
    }

    #[test]
    fn a_tag_reads_back_to_the_name_it_was_written_for() {
        for name in [
            "1.0.0+build.1",
            "1.3.0",
            "v0.1.0",
            "caf\u{e9}",
            ".hidden",
            "a_b",
        ] {
            assert_eq!(read(&tag(name)).as_deref(), Some(name), "{name}");
        }
        assert_eq!(tag("1.0.0+build.1"), "1.0.0_2bbuild.1");
        for not_written in ["_zz", "a_2", "_61", ".x", "-x", "a_2F", "_ff"] {
            assert_eq!(read(not_written), None, "{not_written}");
        }
    }

    #[test]
    fn a_name_no_tag_can_hold_is_refused_naming_it() {
        let long = format!("{}/", "a".repeat(126));
        let words = of(&Name::new(&long)).unwrap_err();
        assert!(words.contains(&long), "{words}");
        assert!(words.contains("129 characters"), "{words}");
        assert_eq!(tag(&"a".repeat(128)).len(), 128);

        let subject = Digest::of(b"site");
        let shaped = referrers(subject.hex());
        assert_eq!(shaped, format!("sha256-{}", subject.hex()));
        let words = of(&Name::new(&shaped)).unwrap_err();
        assert!(words.contains(&shaped), "{words}");
        assert!(words.contains("referrers tag"), "{words}");
        assert_eq!(tag("sha256-abc"), "sha256-abc");

        assert!(of(&Name::new("")).unwrap_err().contains("empty name"));
    }
}
