//! The provenance claim a publish attaches to each artifact: what this
//! host observed of the build an output came from, and the landing's
//! verdicts naming that build, carried as the landing made them and where
//! they were found (11 to 13, rules core.8, core.9; design D7).

use serde::{Deserialize, Serialize};
use tailrace_seams::{
    BuildId, Check, Claim, ClaimKind, Digest, Ecosystem, EcosystemName, Envelope, Outcome,
    OutputName, Revision, Seam, SignerId, Verdict, Version,
};

/// 11, 13: the body of a provenance claim: the output, what its
/// projection made of it, the build's outcome, and every verdict of the
/// landing naming the output's build identity (12).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub output: OutputName,
    pub ecosystem: Ecosystem,
    pub name: EcosystemName,
    pub version: Version,
    pub outcome: Outcome,
    pub carried: Vec<Carried>,
}

/// 12: another party's verdict, carried inside this host's claim: where
/// it was found, the landing's reading of it, and its envelope byte for
/// byte, never decoded and never signed again (rule core.8).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Carried {
    pub found: Found,
    pub check: Check,
    pub build: BuildId,
    pub outcome: Outcome,
    pub signer: SignerId,
    pub envelope: Envelope,
}

/// Where a carried verdict was found: the seam that answered it and the
/// revision it was asked for.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Found {
    pub seam: Seam,
    pub revision: Revision,
}

impl Carried {
    /// `verdict` as the landing answered it for `revision`.
    pub fn from_landing(verdict: &Verdict, revision: &Revision) -> Self {
        Carried {
            found: Found {
                seam: Seam::Landing,
                revision: revision.clone(),
            },
            check: verdict.check.clone(),
            build: verdict.build.clone(),
            outcome: verdict.outcome,
            signer: verdict.signer.clone(),
            envelope: verdict.envelope.clone(),
        }
    }
}

impl Provenance {
    /// 7, 13: the claim about `subject`, the artifact's digest, made for
    /// `revision` and for `build`, the build identity the output came
    /// from (rule core.9).
    pub fn claim(&self, subject: Digest, revision: Revision, build: BuildId) -> Claim {
        Claim {
            subject,
            kind: ClaimKind::Provenance,
            revision,
            build: Some(build),
            body: serde_json::to_value(self).expect("a provenance is plain data"),
        }
    }

    /// The provenance `claim` holds, or `None` for a claim of another
    /// kind or one whose body is not a provenance's.
    pub fn of(claim: &Claim) -> Option<Provenance> {
        if claim.kind != ClaimKind::Provenance {
            return None;
        }
        serde_json::from_value(claim.body.clone()).ok()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tailrace_seams::{Signature, IN_TOTO};

    use super::*;

    fn provenance(carried: Vec<Carried>) -> Provenance {
        Provenance {
            output: OutputName::new("lib-pack"),
            ecosystem: Ecosystem::new("eco"),
            name: EcosystemName::new("lib"),
            version: Version::new("1.3.0"),
            outcome: Outcome::Pass,
            carried,
        }
    }

    fn sealed(claim: &Claim) -> Envelope {
        Envelope::signed(
            IN_TOTO,
            claim.payload(),
            Signature {
                signer: SignerId::new("host"),
                keyid: String::new(),
                sig: vec![1, 2, 3],
            },
        )
    }

    fn theirs() -> Verdict {
        Verdict {
            check: Check::new("test"),
            build: BuildId::new("b7e1"),
            outcome: Outcome::Pass,
            signer: SignerId::new("landing"),
            envelope: Envelope::signed(
                "application/vnd.their-own+json",
                vec![0xff, 0x00, b'{', 0x80, b'"', b'\n'],
                Signature {
                    signer: SignerId::new("landing"),
                    keyid: String::new(),
                    sig: vec![9, 8, 7, 0],
                },
            ),
        }
    }

    #[test]
    fn a_claim_names_the_subject_revision_and_build_identity() {
        let subject = Digest::of(b"artifact");
        let claim = provenance(vec![]).claim(
            subject.clone(),
            Revision::new("4f2a9c1"),
            BuildId::new("b7e1"),
        );
        assert_eq!(claim.subject, subject);
        assert_eq!(claim.kind, ClaimKind::Provenance);
        assert_eq!(claim.revision, Revision::new("4f2a9c1"));
        assert_eq!(claim.build, Some(BuildId::new("b7e1")));
        let back = Claim::from_envelope(&sealed(&claim)).unwrap();
        assert_eq!(back, claim);
        assert_eq!(Provenance::of(&back), Some(provenance(vec![])));
        assert_eq!(claim.body["output"], "lib-pack");
        assert_eq!(claim.body["outcome"], "pass");
    }

    #[test]
    fn a_carried_envelope_reads_back_byte_for_byte() {
        let revision = Revision::new("4f2a9c1");
        let verdict = theirs();
        let carried = Carried::from_landing(&verdict, &revision);
        assert_eq!(
            carried.found,
            Found {
                seam: Seam::Landing,
                revision: revision.clone()
            }
        );
        let claim = provenance(vec![carried]).claim(
            Digest::of(b"artifact"),
            revision,
            BuildId::new("b7e1"),
        );
        let back = Provenance::of(&Claim::from_envelope(&sealed(&claim)).unwrap()).unwrap();
        let [found] = <[Carried; 1]>::try_from(back.carried).unwrap();
        assert_eq!(found.envelope, verdict.envelope);
        assert_eq!(found.envelope.payload, verdict.envelope.payload);
        assert_eq!(found.envelope.signatures, verdict.envelope.signatures);
        assert_eq!(found.envelope.digest(), verdict.envelope.digest());
        assert_eq!(
            (&found.check, &found.build, found.outcome, &found.signer),
            (
                &verdict.check,
                &verdict.build,
                verdict.outcome,
                &verdict.signer
            )
        );
    }

    #[test]
    fn a_claim_with_no_verdicts_carries_an_empty_list() {
        let claim = provenance(vec![]).claim(
            Digest::of(b"artifact"),
            Revision::new("4f2a9c1"),
            BuildId::new("b7e1"),
        );
        assert_eq!(claim.body["carried"], json!([]));
        let back = Provenance::of(&claim).unwrap();
        assert!(back.carried.is_empty());
    }

    #[test]
    fn a_claim_of_another_kind_holds_no_provenance() {
        let claim = Claim {
            kind: ClaimKind::Acceptance,
            ..provenance(vec![]).claim(
                Digest::of(b"artifact"),
                Revision::new("4f2a9c1"),
                BuildId::new("b7e1"),
            )
        };
        assert_eq!(Provenance::of(&claim), None);
    }
}
