//! A DSSE envelope as sluice writes a run record: the payload and each
//! signature in base64, the standard alphabet with padding, read into
//! the seams' envelope whole and written back in DSSE's own form, so
//! every byte sluice wrote is carried (12, C.5; design D7).

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use tailrace_seams::{Envelope, Signature, SignerId};

/// The envelope's JSON, its fields in the order sluice writes them.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Dsse {
    payload: String,
    payload_type: String,
    signatures: Vec<DsseSignature>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DsseSignature {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    keyid: String,
    sig: String,
}

/// The envelope `bytes` hold, each signature found under `signer`; or
/// why they hold none, naming what is not a DSSE envelope or not base64.
pub(super) fn read(bytes: &[u8], signer: &SignerId) -> Result<Envelope, String> {
    let dsse: Dsse = serde_json::from_slice(bytes)
        .map_err(|e| format!("the record is not a DSSE envelope: {e}"))?;
    let payload = STANDARD
        .decode(&dsse.payload)
        .map_err(|e| format!("the envelope's payload is not base64: {e}"))?;
    let signatures = dsse
        .signatures
        .into_iter()
        .enumerate()
        .map(|(n, s)| {
            Ok(Signature {
                signer: signer.clone(),
                keyid: s.keyid,
                sig: STANDARD
                    .decode(&s.sig)
                    .map_err(|e| format!("signature {} is not base64: {e}", n + 1))?,
            })
        })
        .collect::<Result<_, String>>()?;
    Ok(Envelope {
        payload_type: dsse.payload_type,
        payload,
        signatures,
    })
}

/// `envelope` in DSSE's form, as sluice writes it.
pub(super) fn write(envelope: &Envelope) -> Vec<u8> {
    serde_json::to_vec(&Dsse {
        payload: STANDARD.encode(&envelope.payload),
        payload_type: envelope.payload_type.clone(),
        signatures: envelope
            .signatures
            .iter()
            .map(|s| DsseSignature {
                keyid: s.keyid.clone(),
                sig: STANDARD.encode(&s.sig),
            })
            .collect(),
    })
    .expect("an envelope is plain data")
}

#[cfg(test)]
mod tests {
    use tailrace_seams::IN_TOTO;

    use super::*;

    /// A run record in sluice v0.37.0's shape: an in-toto statement of one
    /// result, signed by `ssh-keygen -Y sign -n sluice-run` over DSSE's
    /// pre-authentication encoding, the key named by its fingerprint.
    const RECORD: &str = r#"{"payload":"eyJfdHlwZSI6Imh0dHBzOi8vaW4tdG90by5pby9TdGF0ZW1lbnQvdjEiLCJzdWJqZWN0IjpbeyJuYW1lIjoiYXBwOnRlc3RAbGludXgteDY0IiwiZGlnZXN0Ijp7InNoYTI1NiI6IjQ4NjNhOWFmIn19XSwicHJlZGljYXRlVHlwZSI6Imh0dHBzOi8vYWdlbnRwbG90LmdpdGh1Yi5pby9zbHVpY2UvcnVuL3YxIiwicHJlZGljYXRlIjp7InJlc3VsdHMiOlt7ImNoZWNrIjoiYXBwOnRlc3QiLCJrZXkiOiI0ODYzYTlhZiIsIm91dGNvbWUiOiJwYXNzIn1dLCJjbGFpbXMiOnsiZXhlY3V0b3IiOiJwcm9jZXNzIiwiY29uZmluZWQiOmZhbHNlLCJzeXN0ZW0iOiJsaW51eC14NjQifSwiYnJhbmNoIjoibWFpbiIsImNvbWJpbmF0aW9uIjoiYzBmZmVlIiwiYXQiOiIyMDI2LTEwLTE5VDEyOjAwOjAwWiJ9fQ==","payloadType":"application/vnd.in-toto+json","signatures":[{"keyid":"SHA256:CdVlctE2MfRT0SW66nFxbFKlCYivo5gE7QtIXVFsRxI","sig":"LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWdYaGYrRnZZdFJnbkh0cjFGWTJCa2YrZEhUTQppRlJUeTVIclR1dEI4d2VVd0FBQUFLYzJ4MWFXTmxMWEoxYmdBQUFBQUFBQUFHYzJoaE5URXlBQUFBVXdBQUFBdHpjMmd0ClpXUXlOVFV4T1FBQUFFQnVuZ0ljbm52TVdUbVlnVGszcVN2K2RDYmI5UEdEVHFzZDFLdkIwVStyUGo1WW5vU0tXVHB3ZTgKSURWMTA0MXZyelUzMUtSeng0TkJvS0pMbXhId01DCi0tLS0tRU5EIFNTSCBTSUdOQVRVUkUtLS0tLQo="}]}"#;

    #[test]
    fn a_record_reads_whole_and_writes_back_byte_for_byte() {
        let envelope = read(RECORD.as_bytes(), &SignerId::new("box")).unwrap();
        assert_eq!(envelope.payload_type, IN_TOTO);
        let statement: serde_json::Value = serde_json::from_slice(&envelope.payload).unwrap();
        assert_eq!(
            statement["predicateType"],
            "https://agentplot.github.io/sluice/run/v1"
        );
        let [signature] = <[Signature; 1]>::try_from(envelope.signatures.clone()).unwrap();
        assert_eq!(signature.signer, SignerId::new("box"));
        assert_eq!(
            signature.keyid,
            "SHA256:CdVlctE2MfRT0SW66nFxbFKlCYivo5gE7QtIXVFsRxI"
        );
        assert!(signature
            .sig
            .starts_with(b"-----BEGIN SSH SIGNATURE-----\n"));
        assert_eq!(write(&envelope), RECORD.as_bytes());
    }

    #[test]
    fn two_signatures_and_a_key_id_left_unnamed_write_back_as_read() {
        let two = r#"{"payload":"e30=","payloadType":"application/vnd.in-toto+json","signatures":[{"keyid":"SHA256:a","sig":"AQI="},{"sig":"AwQ="}]}"#;
        let envelope = read(two.as_bytes(), &SignerId::new("forge")).unwrap();
        assert_eq!(envelope.payload, b"{}");
        assert_eq!(
            envelope
                .signatures
                .iter()
                .map(|s| (s.keyid.as_str(), s.sig.as_slice()))
                .collect::<Vec<_>>(),
            [("SHA256:a", [1, 2].as_slice()), ("", [3, 4].as_slice())]
        );
        assert_eq!(write(&envelope), two.as_bytes());
    }

    #[test]
    fn what_is_not_an_envelope_or_not_base64_says_why() {
        let signer = SignerId::new("box");
        let not_json = read(b"a record changed by hand", &signer).unwrap_err();
        assert!(
            not_json.starts_with("the record is not a DSSE envelope"),
            "{not_json}"
        );
        let not_dsse = read(br#"{"payload":"e30="}"#, &signer).unwrap_err();
        assert!(
            not_dsse.starts_with("the record is not a DSSE envelope"),
            "{not_dsse}"
        );
        let payload = read(
            br#"{"payload":"not base64!","payloadType":"t","signatures":[]}"#,
            &signer,
        )
        .unwrap_err();
        assert!(
            payload.starts_with("the envelope's payload is not base64"),
            "{payload}"
        );
        let sig = read(
            br#"{"payload":"e30=","payloadType":"t","signatures":[{"sig":"AQI="},{"sig":"%%"}]}"#,
            &signer,
        )
        .unwrap_err();
        assert!(sig.starts_with("signature 2 is not base64"), "{sig}");
    }
}
