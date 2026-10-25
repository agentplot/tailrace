//! The store's group over a store whose reads lag its moves, as a
//! registry's may: within the bound its rig declares it keeps the group,
//! with no bound it fails, and a name answered at a digest it was never
//! at fails without a wait (C.1, 36).

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use tailrace_conformance::fakes::WORDS;
use tailrace_conformance::seams::store::{self, StoreRig};
use tailrace_conformance::{run, Rig};
use tailrace_fakes::{FakeStore, Fault, StoreOp, Times};
use tailrace_seams::{
    Answer, Digest, Envelope, Held, Manifest, Name, Not, Put, Seam, Source, Store,
};

const LAGGING: &str = "a store whose reads lag its moves";

/// How many resolves after a move answer the name's former place.
const LAG: usize = 2;

/// A store over the fake whose next `LAG` resolves of a name after each
/// move answer the place it was at, nothing where it was made from
/// nothing, or `elsewhere` where it is set, and every one after where it
/// is.
struct Lagging {
    inner: FakeStore,
    elsewhere: Option<Digest>,
    lags: RefCell<HashMap<Name, (Option<Digest>, usize)>>,
}

impl Store for Lagging {
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        self.inner.put_bytes(bytes)
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
        let was = self.inner.resolve(name).ok();
        self.inner.name(name, to, expected)?;
        let was = self.elsewhere.clone().or(was);
        self.lags.borrow_mut().insert(name.clone(), (was, LAG));
        Ok(())
    }
    fn resolve(&self, name: &Name) -> Answer<Digest> {
        if let Some((was, left)) = self.lags.borrow_mut().get_mut(name) {
            if *left > 0 {
                *left -= 1;
                return was.clone().ok_or_else(|| {
                    Not::absent(&Source::new(Seam::Store, LAGGING), format!("name {name}"))
                });
            }
        }
        self.inner.resolve(name)
    }
    fn read(&self, digest: &Digest) -> Answer<Held> {
        self.inner.read(digest)
    }
}

struct LaggingRig {
    store: Lagging,
    within: Duration,
}

impl LaggingRig {
    fn new(within: Duration, elsewhere: Option<Digest>) -> Self {
        LaggingRig {
            store: Lagging {
                inner: FakeStore::new(),
                elsewhere,
                lags: RefCell::default(),
            },
            within,
        }
    }
}

impl Rig for LaggingRig {
    fn profile(&self) -> String {
        LAGGING.into()
    }
}

impl StoreRig for LaggingRig {
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

    fn settles_within(&self) -> Duration {
        self.within
    }
}

/// The report's words over a lagging store whose rig declares `within`,
/// which fails the one scenario that reads a moved name.
fn failed(within: Duration, elsewhere: Option<Digest>) -> String {
    let report = run(store::GROUP, &store::scenarios(), &|| {
        Box::new(LaggingRig::new(within, elsewhere.clone()))
    });
    let failed: Vec<&str> = report.failures.iter().map(|f| f.scenario).collect();
    assert_eq!(failed, ["a name moved from where it is not"], "{report}");
    report.to_string()
}

#[test]
fn a_store_whose_reads_lag_within_its_bound_keeps_the_group() {
    run(store::GROUP, &store::scenarios(), &|| {
        Box::new(LaggingRig::new(Duration::from_secs(5), None))
    })
    .assert();
}

#[test]
fn a_store_whose_reads_lag_with_no_bound_fails_naming_the_former_place() {
    let text = failed(Duration::ZERO, None);
    for words in [
        "- `a name moved from where it is not` (clauses 70, 71): ",
        "resolve answered Absent(",
    ] {
        assert!(text.contains(words), "{text}\ndoes not say: {words}");
    }
}

#[test]
fn a_name_at_a_digest_it_was_never_at_fails_without_a_wait() {
    let started = Instant::now();
    let text = failed(
        Duration::from_secs(30),
        Some(Digest::of(b"a digest no name was at")),
    );
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the group waited {:?} on a digest the name was never at",
        started.elapsed()
    );
    assert!(text.contains("a refused move moved the name"), "{text}");
}
