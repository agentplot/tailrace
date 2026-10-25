//! The store's group over a store whose names take no condition, as a
//! registry's take none: its check before a move reads a replica one move
//! behind, so a move from the place a name was at before its last move
//! goes through. Declaring its names conditional it fails the group's
//! refusal of a move from where the name is not, as ECR did; declaring
//! them not, the group repairs the name from the ledger in that
//! refusal's stead, and it keeps the group (C.1, 70, 109).

use std::cell::RefCell;
use std::collections::HashMap;

use tailrace_conformance::fakes::WORDS;
use tailrace_conformance::seams::store::{self, StoreRig};
use tailrace_conformance::{run, Report, Rig};
use tailrace_fakes::{FakeStore, Fault, StoreOp, Times};
use tailrace_seams::{
    Answer, Digest, Envelope, Held, Manifest, Name, Not, Put, Seam, Source, Store, Why,
};

const UNCONDITIONAL: &str = "a store whose names take no condition";

/// A store over the fake whose check before a move admits the place the
/// name was at before its last move as well as where it is, and whose
/// move then puts the name wherever it is.
struct Unconditional {
    inner: FakeStore,
    before: RefCell<HashMap<Name, Option<Digest>>>,
}

impl Store for Unconditional {
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
        let at = self.inner.resolve(name).ok();
        let behind = self.before.borrow().get(name).cloned();
        if expected != at.as_ref() && behind.as_ref().map(Option::as_ref) != Some(expected) {
            return Err(Not::refused(
                &Source::new(Seam::Store, UNCONDITIONAL),
                Why::NameAt { at },
                None,
            ));
        }
        self.inner.name(name, to, at.as_ref())?;
        self.before.borrow_mut().insert(name.clone(), at);
        Ok(())
    }
    fn resolve(&self, name: &Name) -> Answer<Digest> {
        self.inner.resolve(name)
    }
    fn read(&self, digest: &Digest) -> Answer<Held> {
        self.inner.read(digest)
    }
}

struct UnconditionalRig {
    store: Unconditional,
    declares: bool,
}

impl Rig for UnconditionalRig {
    fn profile(&self) -> String {
        UNCONDITIONAL.into()
    }
}

impl StoreRig for UnconditionalRig {
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

    fn names_conditional(&self) -> bool {
        self.declares
    }
}

/// The group's report over the store, its names declared conditional or
/// not.
fn report(declares: bool) -> Report {
    run(store::GROUP, &store::scenarios(), &|| {
        Box::new(UnconditionalRig {
            store: Unconditional {
                inner: FakeStore::new(),
                before: RefCell::default(),
            },
            declares,
        })
    })
}

fn not_applicable(report: &Report) -> Vec<&str> {
    report.not_applicable.iter().map(|n| n.scenario).collect()
}

#[test]
fn declaring_its_names_conditional_it_fails_the_refusal_of_a_move_from_a_place_left() {
    let report = report(true);
    let failed: Vec<&str> = report.failures.iter().map(|f| f.scenario).collect();
    assert_eq!(failed, ["a name moved from where it is not"], "{report}");
    assert_eq!(
        not_applicable(&report),
        ["a name moved from a stale place, repaired from the ledger"],
        "{report}"
    );
    let text = report.to_string();
    assert!(
        text.contains("a move from nothing of a name made answered Ok(()), not refused"),
        "{text}"
    );
}

#[test]
fn declaring_them_not_conditional_the_group_repairs_from_the_ledger_in_its_stead() {
    let report = report(false);
    report.assert();
    assert_eq!(
        not_applicable(&report),
        ["a name moved from where it is not"],
        "{report}"
    );
    let text = report.to_string();
    assert!(
        text.contains("not applicable: the profile's names take no condition"),
        "{text}"
    );
}
