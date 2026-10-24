//! A digest: the name bytes have by their content alone (1), and the
//! canonical JSON a value is digested in.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

/// The one algorithm a digest is computed with (design D3). A second is
/// an addition to [`Digest::parse`], never a change to [`Digest::of`].
pub const ALGORITHM: &str = "sha256";

const HEX_LEN: usize = 64;

/// The name of bytes by their content alone, the same on every host and
/// in every store (1): `sha256:` and 64 lowercase hex digits.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Digest(String);

impl Digest {
    /// 1: the digest of `bytes`, computed from them alone.
    pub fn of(bytes: &[u8]) -> Self {
        let hash = Sha256::digest(bytes);
        let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
        Digest(format!("{ALGORITHM}:{hex}"))
    }

    /// A digest read from its text, which is only ever `sha256:` and 64
    /// lowercase hex digits; any other text is rejected, naming it.
    pub fn parse(text: &str) -> Result<Self, BadDigest> {
        let bad = |problem| BadDigest {
            text: text.to_string(),
            problem,
        };
        let hex = match text.split_once(':') {
            Some((ALGORITHM, hex)) => hex,
            _ => return Err(bad(Problem::Algorithm)),
        };
        if hex.len() != HEX_LEN {
            return Err(bad(Problem::Length(hex.len())));
        }
        if let Some(c) = hex.chars().find(|c| !matches!(c, '0'..='9' | 'a'..='f')) {
            return Err(bad(Problem::Character(c)));
        }
        Ok(Digest(text.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The algorithm the digest names.
    pub fn algorithm(&self) -> &str {
        ALGORITHM
    }

    /// The hex digits after the algorithm.
    pub fn hex(&self) -> &str {
        &self.0[ALGORITHM.len() + 1..]
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for Digest {
    type Error = BadDigest;

    fn try_from(text: String) -> Result<Self, BadDigest> {
        Digest::parse(&text)
    }
}

impl From<Digest> for String {
    fn from(digest: Digest) -> String {
        digest.0
    }
}

/// A text read as a digest that is not one, named as it was given.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BadDigest {
    pub text: String,
    pub problem: Problem,
}

/// What is wrong with a text read as a digest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Problem {
    /// It names no algorithm, or one other than `sha256`.
    Algorithm,
    /// Its hex is this many digits, not 64.
    Length(usize),
    /// Its hex holds this character, outside `0-9a-f`.
    Character(char),
}

impl fmt::Display for BadDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}` is not a digest: ", self.text)?;
        match self.problem {
            Problem::Algorithm => write!(f, "it names no `{ALGORITHM}:` algorithm"),
            Problem::Length(n) => write!(f, "its hex is {n} digits, not {HEX_LEN}"),
            Problem::Character(c) => write!(f, "its hex holds `{c}`, outside 0-9a-f"),
        }
    }
}

impl std::error::Error for BadDigest {}

/// The canonical JSON of a value: its fields in the order its type
/// declares them, and every map's keys sorted, so the same value gives
/// the same bytes, and so the same digest, wherever it is written.
pub fn canonical(value: &impl Serialize) -> Vec<u8> {
    serde_json::to_vec(value).expect("a type that crosses a seam serializes to JSON")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_bytes_give_equal_digests() {
        assert_eq!(Digest::of(b"tailrace"), Digest::of(b"tailrace"));
        assert_ne!(Digest::of(b"tailrace"), Digest::of(b"tailrace "));
        assert_ne!(Digest::of(b""), Digest::of(b"\0"));
    }

    #[test]
    fn the_empty_input_has_its_known_digest() {
        assert_eq!(
            Digest::of(b"").as_str(),
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn a_digest_reads_back_from_its_text() {
        let d = Digest::of(b"bytes");
        assert_eq!(Digest::parse(d.as_str()), Ok(d.clone()));
        assert_eq!(d.algorithm(), "sha256");
        assert_eq!(d.hex().len(), 64);
        let json = serde_json::to_string(&d).unwrap();
        assert_eq!(json, format!("\"{d}\""));
        assert_eq!(serde_json::from_str::<Digest>(&json).unwrap(), d);
    }

    #[test]
    fn a_text_with_no_algorithm_is_rejected_naming_it() {
        let hex = Digest::of(b"x").hex().to_string();
        for text in [hex.clone(), format!("sha512:{hex}"), format!(":{hex}")] {
            let err = Digest::parse(&text).unwrap_err();
            assert_eq!(err.text, text);
            assert_eq!(err.problem, Problem::Algorithm);
            assert!(err.to_string().contains(&text));
        }
    }

    #[test]
    fn a_text_of_the_wrong_length_is_rejected_naming_it() {
        let text = "sha256:abc123";
        let err = Digest::parse(text).unwrap_err();
        assert_eq!(err.text, text);
        assert_eq!(err.problem, Problem::Length(6));
        assert!(err.to_string().contains(text));
    }

    #[test]
    fn a_text_with_a_character_outside_hex_is_rejected_naming_it() {
        let mut text = Digest::of(b"x").as_str().to_string();
        text.replace_range(text.len() - 1.., "g");
        let err = Digest::parse(&text).unwrap_err();
        assert_eq!(err.text, text);
        assert_eq!(err.problem, Problem::Character('g'));

        let upper = Digest::of(b"x")
            .as_str()
            .to_uppercase()
            .replace("SHA256", "sha256");
        assert!(matches!(
            Digest::parse(&upper).unwrap_err().problem,
            Problem::Character(c) if c.is_ascii_uppercase()
        ));
    }

    #[test]
    fn a_text_that_is_not_a_digest_is_refused_as_json_naming_it() {
        let err = serde_json::from_str::<Digest>("\"sha256:nope\"").unwrap_err();
        assert!(err.to_string().contains("sha256:nope"), "{err}");
    }
}
