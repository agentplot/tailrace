//! The signer seam (B.4) over a key in AWS KMS that the host uses
//! through its own role and can never export (C.7): an asymmetric
//! `ECC_NIST_P256` signing key named by its key ARN, which is the
//! signer's name. A claim is signed by KMS over the digest of the
//! envelope's DSSE pre-authentication encoding, and told apart by the
//! key's public half, read from KMS at each verify; a disabled key's
//! signatures are told as expired (79 to 81).
//!
//! The profile is the key's ARN and how it reaches KMS, and nothing
//! else: the host's credentials are found as the AWS tools find them at
//! each call and dropped before it answers, so no credential is kept
//! (64, rule core.21).
