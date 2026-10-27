//! Spec key-service-signer: the profile holds its key and its reach and
//! nothing else; its answers are `tests/kms_signer.rs`'s, over a
//! stand-in for KMS.

use super::*;

const ARN: &str = "arn:aws:kms:us-west-2:111122223333:key/1234abcd-12ab-34cd-56ef-1234567890ab";

#[test]
fn the_profile_holds_its_key_and_its_reach_and_nothing_else() {
    let key = KeyArn::parse(ARN).unwrap();
    let Kms { signing, reach } = Kms::new(key.clone());
    assert_eq!(signing, Some(key.clone()));
    assert!(matches!(reach, Reach::Host));
    let Kms { signing, reach } = Kms::verifying();
    assert_eq!(signing, None);
    assert!(matches!(reach, Reach::Host));
    assert_eq!(Kms::new(key.clone()).key(), Some(&key));
}
