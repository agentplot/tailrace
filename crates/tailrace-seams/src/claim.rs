//! A claim about one digest by one signer, and the envelope that carries
//! it signed (7, 8, 13): an in-toto statement in DSSE's shape, the form
//! section 9 fixes for every attestation (design D5).

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{canonical, BuildId, Digest, Revision};

/// The payload type of an envelope holding an in-toto statement.
pub const IN_TOTO: &str = "application/vnd.in-toto+json";

/// The type every in-toto statement names itself by.
pub const STATEMENT: &str = "https://in-toto.io/Statement/v1";

/// What the predicate type of every claim tailrace's form writes begins
/// with; the kind and `/v1` follow.
pub const PREDICATES: &str = "https://agentplot.github.io/tailrace/claim/";

text! {
    /// A signer, by the identity it signs as (79), and by which a policy
    /// names it (82, 104).
    SignerId
}

text! {
    /// A check, by the name the landing gives it (78, 116).
    Check
}

/// What a claim says of its digest (section 3, claim).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimKind {
    /// A build produced it from a revision.
    Provenance,
    /// Checks passed on a build (78).
    Verdict,
    /// A test ran.
    Test,
    /// A reviewer approved.
    Review,
    /// A verifier rebuilt it and the digests matched or did not (30).
    Rebuild,
    /// The gate accepted it (26).
    Acceptance,
    /// A channel moved to it (39, 105).
    Promotion,
    /// Another party's claim, carried as made, and where it was found
    /// (12).
    Carried,
    /// The publisher's policy for layouts, about a version's index: the
    /// claims a layout must require, as the publishing repository
    /// declared them at the revision the claim names (113).
    LayoutPolicy,
}

impl ClaimKind {
    const ALL: [ClaimKind; 9] = [
        ClaimKind::Provenance,
        ClaimKind::Verdict,
        ClaimKind::Test,
        ClaimKind::Review,
        ClaimKind::Rebuild,
        ClaimKind::Acceptance,
        ClaimKind::Promotion,
        ClaimKind::Carried,
        ClaimKind::LayoutPolicy,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ClaimKind::Provenance => "provenance",
            ClaimKind::Verdict => "verdict",
            ClaimKind::Test => "test",
            ClaimKind::Review => "review",
            ClaimKind::Rebuild => "rebuild",
            ClaimKind::Acceptance => "acceptance",
            ClaimKind::Promotion => "promotion",
            ClaimKind::Carried => "carried",
            ClaimKind::LayoutPolicy => "layout_policy",
        }
    }

    /// The in-toto predicate type a claim of this kind is written under.
    pub fn predicate_type(self) -> String {
        format!("{PREDICATES}{}/v1", self.name())
    }

    fn of_predicate_type(text: &str) -> Option<ClaimKind> {
        let name = text.strip_prefix(PREDICATES)?.strip_suffix("/v1")?;
        ClaimKind::ALL.into_iter().find(|k| k.name() == name)
    }
}

impl fmt::Display for ClaimKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// 7, 13: a statement about exactly one digest, of one kind, made for one
/// revision and for the build identity where it has one. `body` is what
/// the claim's kind says, fixed by the units that make each kind.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    pub subject: Digest,
    pub kind: ClaimKind,
    pub revision: Revision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildId>,
    pub body: Value,
}

#[derive(Serialize, Deserialize)]
struct Statement {
    #[serde(rename = "_type")]
    statement: String,
    subject: Vec<Subject>,
    #[serde(rename = "predicateType")]
    predicate_type: String,
    predicate: Predicate,
}

#[derive(Serialize, Deserialize)]
struct Subject {
    digest: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct Predicate {
    revision: Revision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    build: Option<BuildId>,
    body: Value,
}

impl Claim {
    /// The bytes a signer signs: the claim as an in-toto statement whose
    /// subject is the digest, whose predicate type is the kind, and whose
    /// predicate is the revision, the build and the body, so every signer
    /// signs the same bytes for the same claim.
    pub fn payload(&self) -> Vec<u8> {
        canonical(&Statement {
            statement: STATEMENT.into(),
            subject: vec![Subject {
                digest: BTreeMap::from([(
                    self.subject.algorithm().to_string(),
                    self.subject.hex().to_string(),
                )]),
            }],
            predicate_type: self.kind.predicate_type(),
            predicate: Predicate {
                revision: self.revision.clone(),
                build: self.build.clone(),
                body: self.body.clone(),
            },
        })
    }

    /// The claim an envelope holds, or `None` for an envelope in any other
    /// form, such as another party's, which is carried as made (12) and
    /// never read as a claim of tailrace's form, and for one holding any
    /// number of signatures but one (7).
    pub fn from_envelope(envelope: &Envelope) -> Option<Claim> {
        if envelope.payload_type != IN_TOTO || envelope.sole().is_none() {
            return None;
        }
        let statement: Statement = serde_json::from_slice(&envelope.payload).ok()?;
        if statement.statement != STATEMENT {
            return None;
        }
        let [subject] = <[Subject; 1]>::try_from(statement.subject).ok()?;
        let [(algorithm, hex)] =
            <[(String, String); 1]>::try_from(subject.digest.into_iter().collect::<Vec<_>>())
                .ok()?;
        Some(Claim {
            subject: Digest::parse(&format!("{algorithm}:{hex}")).ok()?,
            kind: ClaimKind::of_predicate_type(&statement.predicate_type)?,
            revision: statement.predicate.revision,
            build: statement.predicate.build,
            body: statement.predicate.body,
        })
    }
}

/// 7, 8, 12: a payload signed, in DSSE's shape: the payload's type, the
/// payload, and the signatures over them. A claim of tailrace's form holds
/// exactly one signature, its one signer's (7); an envelope carried from
/// another party holds every signature it was made with, in the order
/// made, each with its key id as written (12, C.5).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    pub payload_type: String,
    pub payload: Vec<u8>,
    pub signatures: Vec<Signature>,
}

/// One signer's signature, as its signer profile makes and reads it: the
/// identity it was found under, the key id as its maker wrote it, empty
/// when the maker named none, and the signature's bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    pub signer: SignerId,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub keyid: String,
    pub sig: Vec<u8>,
}

impl Envelope {
    /// `payload` of `payload_type` under `signature` alone: the form of
    /// every claim tailrace signs (7).
    pub fn signed(
        payload_type: impl Into<String>,
        payload: Vec<u8>,
        signature: Signature,
    ) -> Envelope {
        Envelope {
            payload_type: payload_type.into(),
            payload,
            signatures: vec![signature],
        }
    }

    /// The one signature of an envelope that holds exactly one.
    pub fn sole(&self) -> Option<&Signature> {
        match self.signatures.as_slice() {
            [one] => Some(one),
            _ => None,
        }
    }

    /// The envelope's own digest, by which a promotion names the one it
    /// follows (105, 120): the digest of its canonical JSON.
    pub fn digest(&self) -> Digest {
        Digest::of(&canonical(self))
    }
}

/// 82: a signer as a policy names it: its identity, what verifies it (a
/// text its signer profile reads, a public key or a key's name), and the
/// kinds of claim, and for a verdict the checks, it may claim (116).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedSigner {
    pub id: SignerId,
    pub verified_by: String,
    pub kinds: Vec<ClaimKind>,
    pub checks: Vec<Check>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::round_trip;

    fn claim(build: Option<&str>) -> Claim {
        Claim {
            subject: Digest::of(b"index"),
            kind: ClaimKind::Provenance,
            revision: Revision::new("4f2a9c1"),
            build: build.map(BuildId::new),
            body: json!({"outputs": ["site"]}),
        }
    }

    fn signature(signer: &str, keyid: &str, sig: &[u8]) -> Signature {
        Signature {
            signer: SignerId::new(signer),
            keyid: keyid.into(),
            sig: sig.to_vec(),
        }
    }

    fn sealed(claim: &Claim, signer: &str) -> Envelope {
        Envelope::signed(IN_TOTO, claim.payload(), signature(signer, "", &[1, 2, 3]))
    }

    #[test]
    fn a_claim_reads_back_from_its_envelope() {
        let c = claim(Some("b7e1"));
        assert_eq!(Claim::from_envelope(&sealed(&c, "host")), Some(c));
    }

    #[test]
    fn a_claim_with_no_build_identity_reads_back_with_none() {
        let c = claim(None);
        let back = Claim::from_envelope(&sealed(&c, "host")).unwrap();
        assert_eq!(back.build, None);
        assert_eq!(back, c);
        let statement: Value = serde_json::from_slice(&c.payload()).unwrap();
        assert!(statement["predicate"].get("build").is_none(), "{statement}");
        assert_eq!(
            serde_json::from_str::<Claim>(&serde_json::to_string(&c).unwrap())
                .unwrap()
                .build,
            None
        );
    }

    #[test]
    fn the_payload_is_an_in_toto_statement_about_the_digest() {
        let c = claim(Some("b7e1"));
        let statement: Value = serde_json::from_slice(&c.payload()).unwrap();
        assert_eq!(statement["_type"], STATEMENT);
        assert_eq!(
            statement["subject"],
            json!([{"digest": {"sha256": c.subject.hex()}}])
        );
        assert_eq!(
            statement["predicateType"],
            "https://agentplot.github.io/tailrace/claim/provenance/v1"
        );
        assert_eq!(statement["predicate"]["revision"], "4f2a9c1");
        assert_eq!(statement["predicate"]["build"], "b7e1");
        assert_eq!(c.payload(), claim(Some("b7e1")).payload());
    }

    #[test]
    fn every_kind_reads_back_from_its_predicate_type() {
        for kind in ClaimKind::ALL {
            let c = Claim {
                kind,
                ..claim(None)
            };
            assert_eq!(
                Claim::from_envelope(&sealed(&c, "host")).unwrap().kind,
                kind
            );
        }
    }

    #[test]
    fn a_layout_policy_about_an_index_reads_back_with_its_revision_and_body() {
        let required = json!({"claims": [{
            "kind": "acceptance",
            "check": null,
            "signers": [{
                "id": "gate",
                "verified_by": "the gate's key",
                "kinds": ["acceptance"],
                "checks": [],
            }],
        }]});
        let c = Claim {
            subject: Digest::of(b"index"),
            kind: ClaimKind::LayoutPolicy,
            revision: Revision::new("4f2a9c1"),
            build: None,
            body: required.clone(),
        };
        let statement: Value = serde_json::from_slice(&c.payload()).unwrap();
        assert_eq!(
            statement["predicateType"],
            "https://agentplot.github.io/tailrace/claim/layout_policy/v1"
        );
        let back = Claim::from_envelope(&sealed(&c, "host")).unwrap();
        assert_eq!(back.kind, ClaimKind::LayoutPolicy);
        assert_eq!(back.subject, Digest::of(b"index"));
        assert_eq!(back.revision, Revision::new("4f2a9c1"));
        assert_eq!(back.build, None);
        assert_eq!(back.body, required);
        assert_eq!(
            serde_json::to_value(ClaimKind::LayoutPolicy).unwrap(),
            "layout_policy"
        );
    }

    #[test]
    fn the_same_envelope_gives_the_same_digest() {
        let e = sealed(&claim(None), "gate");
        let held_elsewhere: Envelope = serde_json::from_slice(&canonical(&e)).unwrap();
        assert_eq!(e.digest(), held_elsewhere.digest());
        assert_eq!(e.digest(), e.clone().digest());
        assert_ne!(e.digest(), sealed(&claim(Some("b7e1")), "gate").digest());
        assert_ne!(e.digest(), sealed(&claim(None), "other").digest());
    }

    #[test]
    fn a_claim_holding_another_partys_envelope_reads_it_back_byte_for_byte() {
        let theirs = Envelope::signed(
            "application/vnd.their-own+json",
            vec![0xff, 0x00, b'{', 0x80, b'"'],
            signature("landing", "", &[9, 8, 7, 0]),
        );
        let carrier = Claim {
            kind: ClaimKind::Carried,
            body: json!({"found": "the landing's record", "envelope": theirs}),
            ..claim(Some("b7e1"))
        };
        let back = Claim::from_envelope(&sealed(&carrier, "host")).unwrap();
        let found: Envelope = serde_json::from_value(back.body["envelope"].clone()).unwrap();
        assert_eq!(found, theirs);
        assert_eq!(found.payload, theirs.payload);
        assert_eq!(found.signatures, theirs.signatures);
        assert_eq!(found.digest(), theirs.digest());
    }

    #[test]
    fn an_envelope_in_another_form_answers_no_claim() {
        let c = claim(None);
        let other_type = Envelope {
            payload_type: "application/vnd.their-own+json".into(),
            ..sealed(&c, "landing")
        };
        assert_eq!(Claim::from_envelope(&other_type), None);

        let mut statement: Value = serde_json::from_slice(&c.payload()).unwrap();
        statement["predicateType"] = json!("https://example.org/their-run/v1");
        let other_predicate = Envelope {
            payload: serde_json::to_vec(&statement).unwrap(),
            ..sealed(&c, "landing")
        };
        assert_eq!(Claim::from_envelope(&other_predicate), None);

        statement["predicateType"] = json!(ClaimKind::Verdict.predicate_type());
        statement["subject"] = json!([{"digest": {"sha256": c.subject.hex()}}, {"digest": {"sha256": c.subject.hex()}}]);
        let two_subjects = Envelope {
            payload: serde_json::to_vec(&statement).unwrap(),
            ..sealed(&c, "landing")
        };
        assert_eq!(Claim::from_envelope(&two_subjects), None);

        let not_json = Envelope {
            payload: b"not a statement".to_vec(),
            ..sealed(&c, "landing")
        };
        assert_eq!(Claim::from_envelope(&not_json), None);
    }

    #[test]
    fn every_type_round_trips_through_json() {
        round_trip(SignerId::new("gate"));
        round_trip(Check::new("test"));
        for kind in ClaimKind::ALL {
            round_trip(kind);
        }
        round_trip(claim(None));
        round_trip(claim(Some("b7e1")));
        round_trip(sealed(&claim(None), "host"));
        round_trip(signature("host", "", &[0, 255]));
        round_trip(signature("host", "SHA256:4kq0", &[0, 255]));
        round_trip(NamedSigner {
            id: SignerId::new("landing"),
            verified_by: "a key's name".into(),
            kinds: vec![ClaimKind::Verdict],
            checks: vec![Check::new("test"), Check::new("lint")],
        });
    }

    #[test]
    fn an_envelope_of_two_signatures_round_trips_in_order_with_its_key_ids() {
        let theirs = Envelope {
            payload_type: IN_TOTO.into(),
            payload: b"{\"_type\":\"theirs\"}".to_vec(),
            signatures: vec![
                signature("forge", "SHA256:f0rge", &[9, 8]),
                signature("box", "SHA256:b0x", &[7, 6, 0]),
            ],
        };
        let json = serde_json::to_value(&theirs).unwrap();
        assert_eq!(json["signatures"][0]["keyid"], "SHA256:f0rge", "{json}");
        assert_eq!(json["signatures"][1]["keyid"], "SHA256:b0x", "{json}");
        let back: Envelope = serde_json::from_value(json).unwrap();
        assert_eq!(back, theirs);
        assert_eq!(
            back.signatures
                .iter()
                .map(|s| (s.signer.as_str(), s.keyid.as_str()))
                .collect::<Vec<_>>(),
            [("forge", "SHA256:f0rge"), ("box", "SHA256:b0x")]
        );
        assert_eq!(back.sole(), None);
        round_trip(theirs);
    }

    #[test]
    fn a_key_id_left_empty_is_not_written() {
        let json = serde_json::to_value(sealed(&claim(None), "host")).unwrap();
        let [written] =
            <[Value; 1]>::try_from(json["signatures"].as_array().unwrap().clone()).unwrap();
        assert!(written.get("keyid").is_none(), "{written}");
        let back: Signature = serde_json::from_value(written).unwrap();
        assert_eq!(back.keyid, "");
    }

    #[test]
    fn tailraces_form_with_two_signatures_or_none_is_no_claim() {
        let c = claim(Some("b7e1"));
        let one = sealed(&c, "host");
        assert_eq!(one.sole(), Some(&signature("host", "", &[1, 2, 3])));
        assert_eq!(Claim::from_envelope(&one), Some(c.clone()));

        let two = Envelope {
            signatures: vec![
                signature("host", "", &[1, 2, 3]),
                signature("gate", "", &[4]),
            ],
            ..one.clone()
        };
        assert_eq!(two.sole(), None);
        assert_eq!(Claim::from_envelope(&two), None);

        let none = Envelope {
            signatures: vec![],
            ..one
        };
        assert_eq!(none.sole(), None);
        assert_eq!(Claim::from_envelope(&none), None);
    }
}
