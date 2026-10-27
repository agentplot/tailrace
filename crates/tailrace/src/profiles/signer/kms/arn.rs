//! A KMS key named by its key ARN, the one name a key service signer is
//! bound and listed by (C.7; design D1):
//! `arn:<partition>:kms:<region>:<account>:key/<key id>`. An alias names
//! no key, since it can be moved to another one, and so does a bare key
//! id, which names no region or account.

use std::fmt;

/// The form a key ARN takes, as every refusal of a name says it.
const FORM: &str =
    "a key is named by its key ARN, arn:<partition>:kms:<region>:<account>:key/<key id>";

/// A KMS key's ARN: the signer's identity, and the region KMS is reached
/// in for it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyArn {
    text: String,
    region: String,
}

/// Why a text is no key ARN, in words naming the text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotKeyArn {
    /// It begins `arn:` and names `:kms:`, so it means a KMS key, and it
    /// is no key ARN: an alias's ARN, or a broken one.
    Broken(String),
    /// It names no KMS key at all, as another profile's verifier.
    Other(String),
}

impl NotKeyArn {
    /// The words saying why.
    pub fn words(&self) -> &str {
        match self {
            NotKeyArn::Broken(words) | NotKeyArn::Other(words) => words,
        }
    }
}

impl KeyArn {
    /// `text` as a key ARN, or why it is none.
    pub fn parse(text: &str) -> Result<KeyArn, NotKeyArn> {
        if !(text.starts_with("arn:") && text.contains(":kms:")) {
            return Err(NotKeyArn::Other(format!(
                "{text} is no KMS key ARN; {FORM}"
            )));
        }
        let broken = |why: String| NotKeyArn::Broken(format!("{text} {why}; {FORM}"));
        let parts: Vec<&str> = text.splitn(6, ':').collect();
        let [_, partition, service, region, account, resource] = parts[..] else {
            return Err(broken("has fewer than the six parts of an ARN".into()));
        };
        if service != "kms" {
            return Err(broken(format!("names the service {service}, not kms")));
        }
        if !is_name(partition) {
            return Err(broken(format!("names no partition in {partition:?}")));
        }
        if !is_name(region) {
            return Err(broken(format!("names no region in {region:?}")));
        }
        if account.len() != 12 || !account.bytes().all(|b| b.is_ascii_digit()) {
            return Err(broken(format!(
                "names the account {account:?}, which is not twelve digits"
            )));
        }
        match resource.split_once('/') {
            Some(("key", id)) if !id.is_empty() && id.bytes().all(is_id) => Ok(KeyArn {
                text: text.to_string(),
                region: region.to_string(),
            }),
            Some(("alias", alias)) => Err(broken(format!(
                "names the alias {alias}, and an alias can be moved to another key"
            ))),
            _ => Err(broken(format!("names {resource}, which is no key"))),
        }
    }

    /// The region the key is held in, where KMS is reached for it.
    pub fn region(&self) -> &str {
        &self.region
    }

    /// The ARN as written.
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

impl fmt::Display for KeyArn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// A partition's or a region's name: lower-case letters, digits and
/// hyphens.
fn is_name(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// A byte of a key id: a UUID's, or a multi-Region key's `mrk-` and hex.
fn is_id(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'-'
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARN: &str = "arn:aws:kms:us-west-2:111122223333:key/1234abcd-12ab-34cd-56ef-1234567890ab";

    fn broken(text: &str) -> String {
        match KeyArn::parse(text) {
            Err(NotKeyArn::Broken(words)) => words,
            other => panic!("{text} parsed as {other:?}, not a broken KMS name"),
        }
    }

    fn other(text: &str) -> String {
        match KeyArn::parse(text) {
            Err(NotKeyArn::Other(words)) => words,
            other => panic!("{text} parsed as {other:?}, not a name of no KMS key"),
        }
    }

    #[test]
    fn a_key_arn_parses_with_its_region() {
        let arn = KeyArn::parse(ARN).unwrap();
        assert_eq!(arn.region(), "us-west-2");
        assert_eq!(arn.as_str(), ARN);
        assert_eq!(arn.to_string(), ARN);
        let mrk = "arn:aws-us-gov:kms:us-gov-west-1:111122223333:key/mrk-1234abcd12ab34cd56ef1234567890ab";
        assert_eq!(KeyArn::parse(mrk).unwrap().region(), "us-gov-west-1");
    }

    #[test]
    fn an_alias_arn_is_a_broken_kms_name() {
        let words = broken("arn:aws:kms:us-west-2:111122223333:alias/fleet");
        assert_eq!(
            words,
            "arn:aws:kms:us-west-2:111122223333:alias/fleet names the alias fleet, and an \
             alias can be moved to another key; a key is named by its key ARN, \
             arn:<partition>:kms:<region>:<account>:key/<key id>"
        );
    }

    #[test]
    fn a_broken_kms_arn_says_what_is_wrong() {
        for (text, says) in [
            (
                "arn:aws:kms:us-west-2:111122223333",
                "fewer than the six parts",
            ),
            ("arn:aws:kms::111122223333:key/1234abcd", "names no region"),
            (
                "arn::kms:us-west-2:111122223333:key/1234abcd",
                "names no partition",
            ),
            (
                "arn:aws:kms:us-west-2:1111:key/1234abcd",
                "not twelve digits",
            ),
            ("arn:aws:kms:us-west-2:111122223333:key/", "which is no key"),
            (
                "arn:aws:kms:us-west-2:111122223333:key/12 34",
                "which is no key",
            ),
            (
                "arn:aws:iam::111122223333:role/x:kms:y",
                "names the service iam",
            ),
        ] {
            assert!(broken(text).contains(says), "{text}: {}", broken(text));
        }
    }

    #[test]
    fn a_bare_id_an_alias_name_or_another_key_names_no_kms_key() {
        for text in [
            "1234abcd-12ab-34cd-56ef-1234567890ab",
            "alias/fleet",
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl",
            "",
        ] {
            assert_eq!(other(text), format!("{text} is no KMS key ARN; {FORM}"));
        }
    }
}
