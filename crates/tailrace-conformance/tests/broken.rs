//! The store's group over stores that break its contract: each fails the
//! group, reported as any profile's failure is (93; design D9).

use tailrace_conformance::fakes::{OtherBytesRig, WORDS};
use tailrace_conformance::seams::store::{self, StoreRig};
use tailrace_conformance::{run, Report, Rig};
use tailrace_fakes::{FakeStore, Fault, StoreOp, Times};
use tailrace_seams::{Answer, Digest, Envelope, Held, Manifest, Name, Put, Store};

/// A store over the fake that answers every put as not held already.
struct NeverHeldAlready {
    inner: FakeStore,
}

impl Store for NeverHeldAlready {
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        let put = self.inner.put_bytes(bytes)?;
        Ok(Put {
            held_already: false,
            ..put
        })
    }
    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        self.inner.put_artifact(manifest)
    }
    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
        self.inner.attach(subject, attestation)
    }
    fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
        self.inner.claims_of(digest)
    }
    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        self.inner.name(name, to, expected)
    }
    fn resolve(&self, name: &Name) -> Answer<Digest> {
        self.inner.resolve(name)
    }
    fn read(&self, digest: &Digest) -> Answer<Held> {
        self.inner.read(digest)
    }
}

const NEVER_HELD: &str = "a store that never holds bytes already";

struct NeverHeldAlreadyRig {
    store: NeverHeldAlready,
}

impl Rig for NeverHeldAlreadyRig {
    fn profile(&self) -> String {
        NEVER_HELD.into()
    }
}

impl StoreRig for NeverHeldAlreadyRig {
    fn store(&self) -> &dyn Store {
        &self.store
    }

    fn unreachable(&self) -> String {
        use StoreOp::*;
        for op in [PutBytes, PutArtifact, Attach, ClaimsOf, Name, Resolve, Read] {
            self.store
                .inner
                .fault(op, Fault::Unreadable(WORDS.into()), Times::UntilHealed);
        }
        WORDS.into()
    }
}

fn failed(report: &Report, profile: &str, scenarios: &[&str]) -> String {
    assert!(!report.passed(), "{report}");
    let failed: Vec<&str> = report.failures.iter().map(|f| f.scenario).collect();
    assert_eq!(failed, scenarios, "{report}");
    let text = report.to_string();
    assert!(
        text.starts_with(&format!(
            "the store group over the profile `{profile}`: {} of {} scenarios failed",
            scenarios.len(),
            store::scenarios().len()
        )),
        "{text}"
    );
    text
}

#[test]
fn a_store_that_never_holds_bytes_already_fails_naming_the_bytes_put_twice() {
    let report = run(store::GROUP, &store::scenarios(), &|| {
        Box::new(NeverHeldAlreadyRig {
            store: NeverHeldAlready {
                inner: FakeStore::new(),
            },
        })
    });
    let text = failed(&report, NEVER_HELD, &["the same bytes put twice"]);
    assert_eq!(report.failures[0].cites, [1, 5, 66]);
    for words in [
        "- `the same bytes put twice` (clauses 1, 5, 66): ",
        "the second put answered Put { digest: ",
        "held_already: false }",
    ] {
        assert!(text.contains(words), "{text}\ndoes not say: {words}");
    }
    let caught = std::panic::catch_unwind(|| report.assert()).expect_err("it passed");
    let said = caught.downcast_ref::<String>().expect("the report's words");
    assert_eq!(said, &text);
}

#[test]
fn a_store_holding_other_bytes_under_a_digest_fails_naming_what_it_read() {
    const OTHER: &str = "a store holding other bytes under a digest";
    let report = run(store::GROUP, &store::scenarios(), &|| {
        Box::new(OtherBytesRig::named(OTHER))
    });
    let text = failed(
        &report,
        OTHER,
        &["the same bytes put twice", "bytes read back"],
    );
    for words in [
        "- `the same bytes put twice` (clauses 1, 5, 66): ",
        "- `bytes read back` (clauses 1, 72, 73): ",
        "the first read answered bytes whose digest is",
        &format!(
            "read of the bytes put answered {:?}",
            String::from_utf8_lossy(OtherBytesRig::OTHER)
        ),
    ] {
        assert!(text.contains(words), "{text}\ndoes not say: {words}");
    }
}
