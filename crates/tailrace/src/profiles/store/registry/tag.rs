//! A name as a tag (design D8): every byte of the name's UTF-8 in
//! `[A-Za-z0-9.-]` kept, every other byte, `_` among them, written `_`
//! and two lower-case hex digits, and a leading `.` or `-` written the
//! same way, so one name is one tag and a version's name reads as itself.

use tailrace_seams::{Name, ALGORITHM};

/// The longest tag the distribution specification allows.
const LONGEST: usize = 128;

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
