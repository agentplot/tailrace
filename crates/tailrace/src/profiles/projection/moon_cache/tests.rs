//! Spec moon-cache-projection, over a temporary checkout and an OCI
//! layout store (design D5).

use std::fs;
use std::path::{Path, PathBuf};

use tailrace_fakes::{FakeStore, StoreOp};
use tailrace_seams::{
    Artifact, ArtifactMeta, BuildId, CacheEntry, Contents, Digest, Ecosystem, EcosystemName, Holds,
    Manifest, Not, Output, OutputName, Seam, Store, Version, Why,
};
use tempfile::TempDir;

use super::*;
use crate::profiles::store::layout::Layout as OciLayout;

const B7E1: &str = "b7e1";
const C3D4: &str = "c3d4";

/// A checkout and a layout store, each in its own temporary directory.
struct Rig {
    _dir: TempDir,
    checkout: PathBuf,
    layout: OciLayout,
}

impl Rig {
    fn new() -> Rig {
        let dir = TempDir::new().unwrap();
        let checkout = dir.path().join("checkout");
        fs::create_dir_all(&checkout).unwrap();
        let layout = OciLayout::create(dir.path().join("layout")).unwrap();
        Rig {
            _dir: dir,
            checkout,
            layout,
        }
    }

    fn projection(&self) -> MoonCache {
        MoonCache::new(&self.checkout, Box::new(self.layout.clone()))
    }

    fn outputs(&self) -> PathBuf {
        self.checkout.join(".moon/cache/outputs")
    }

    /// The archive moon left for `hash`, holding `bytes`.
    fn left(&self, hash: &str, bytes: &[u8]) {
        fs::create_dir_all(self.outputs()).unwrap();
        fs::write(self.outputs().join(format!("{hash}.tar.gz")), bytes).unwrap();
    }

    /// The entry of `hash` whose bytes the layout holds.
    fn held(&self, hash: &str, bytes: &[u8]) -> CacheEntry {
        let digest = self.layout.put_bytes(bytes).unwrap().digest;
        CacheEntry {
            build: BuildId::new(hash),
            digest,
        }
    }

    fn render(&self, entries: &[CacheEntry]) -> Answer<Rendered> {
        self.projection().render(
            Contents::CacheEntries(entries),
            self.checkout.to_str().unwrap(),
        )
    }
}

fn site() -> Output {
    Output {
        name: OutputName::new("site"),
        files: vec!["dist/site".into()],
    }
}

/// Every file under `dir`, by path, with its bytes.
fn files(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(dir: &Path, found: &mut Vec<(PathBuf, Vec<u8>)>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, found);
            } else {
                found.push((path.clone(), fs::read(&path).unwrap()));
            }
        }
    }
    let mut found = Vec::new();
    walk(dir, &mut found);
    found.sort();
    found
}

fn absent_naming_artifacts<T: std::fmt::Debug>(answer: Answer<T>) {
    assert_eq!(
        answer.map(|_| ()),
        Err(Not::absent(&MoonCache::source(), "artifacts"))
    );
}

// Requirement: pack answers the archive moon left for a task hash

#[test]
fn an_archive_packs_as_its_bytes_unchanged_with_no_metadata() {
    let rig = Rig::new();
    rig.left(B7E1, b"moon's archive \x00\xff");
    let packed = rig
        .projection()
        .pack(&site(), &BuildId::new(B7E1), None)
        .unwrap();
    assert_eq!(
        packed,
        Packed {
            bytes: b"moon's archive \x00\xff".to_vec(),
            meta: None,
            same_again: false,
        }
    );
}

#[test]
fn a_hash_with_no_archive_is_absent_naming_its_path() {
    let rig = Rig::new();
    let path = rig.outputs().join("b7e1.tar.gz");
    assert_eq!(
        rig.projection().pack(&site(), &BuildId::new(B7E1), None),
        Err(Not::absent(
            &MoonCache::source(),
            path.display().to_string()
        ))
    );
    assert_eq!(files(&rig.checkout), [], "the pack wrote");
}

#[test]
fn an_outputs_directory_that_is_a_file_is_unreadable_in_the_operating_systems_words() {
    let rig = Rig::new();
    fs::create_dir_all(rig.checkout.join(".moon/cache")).unwrap();
    fs::write(rig.outputs(), b"not a directory").unwrap();
    let path = rig.outputs().join("b7e1.tar.gz");
    let words = fs::read(&path).unwrap_err().to_string();
    match rig.projection().pack(&site(), &BuildId::new(B7E1), None) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source.seam, Seam::Projection);
            assert_eq!(u.source.profile, ECOSYSTEM);
            assert_eq!(u.what, path.display().to_string());
            assert_eq!(u.said, Some(words));
        }
        other => panic!("answered {other:?}"),
    }
}

#[test]
fn a_pack_at_a_build_identity_that_is_no_task_hash_is_refused_naming_it() {
    let rig = Rig::new();
    for bad in ["../escape", "", "B7E1", "b7e1/x"] {
        match rig.projection().pack(&site(), &BuildId::new(bad), None) {
            Err(Not::Refused(r)) => {
                assert_eq!(r.why, Why::Said);
                let said = r.said.unwrap();
                assert!(
                    said.contains(&format!("{bad} is not a task hash")),
                    "{said}"
                );
            }
            other => panic!("{bad} answered {other:?}"),
        }
    }
}

// Requirement: The moon cache packs no artifact

#[test]
fn a_pack_at_a_version_and_a_render_of_artifacts_are_absent_and_read_nothing() {
    let rig = Rig::new();
    rig.left(B7E1, b"archive");
    let store = FakeStore::new();
    let projection = MoonCache::new(&rig.checkout, Box::new(store.clone()));
    absent_naming_artifacts(projection.pack(
        &site(),
        &BuildId::new(B7E1),
        Some(&Version::new("1.0.0")),
    ));
    let meta = ArtifactMeta::new(
        Ecosystem::new("file"),
        EcosystemName::new("site"),
        Version::new("1.0.0"),
    )
    .unwrap();
    let artifacts = [Artifact {
        digest: Digest::of(b"archive"),
        meta,
    }];
    let before = files(&rig.checkout);
    absent_naming_artifacts(projection.render(
        Contents::Artifacts(&artifacts),
        rig.checkout.to_str().unwrap(),
    ));
    assert_eq!(store.calls(), [], "the store was read");
    assert_eq!(files(&rig.checkout), before, "the render wrote");
}

// Requirement: render writes each entry where moon reads it

#[test]
fn two_entries_render_as_archives_holding_their_bytes_in_the_order_given() {
    let rig = Rig::new();
    let entries = [rig.held(C3D4, b"the second"), rig.held(B7E1, b"the first")];
    let rendered = rig.render(&entries).unwrap();
    assert_eq!(rendered.layout.at, rig.outputs().display().to_string());
    assert_eq!(
        rendered.holds,
        Holds::BuildIds(vec![BuildId::new(C3D4), BuildId::new(B7E1)])
    );
    assert_eq!(
        files(&rig.checkout),
        [
            (rig.outputs().join("b7e1.tar.gz"), b"the first".to_vec()),
            (rig.outputs().join("c3d4.tar.gz"), b"the second".to_vec()),
        ]
    );
}

#[test]
fn a_render_at_a_relative_path_is_under_the_checkout() {
    let rig = Rig::new();
    let entries = [rig.held(B7E1, b"archive")];
    let rendered = rig
        .projection()
        .render(Contents::CacheEntries(&entries), "nested")
        .unwrap();
    let dir = rig.checkout.join("nested/.moon/cache/outputs");
    assert_eq!(rendered.layout.at, dir.display().to_string());
    assert_eq!(fs::read(dir.join("b7e1.tar.gz")).unwrap(), b"archive");
}

#[test]
fn an_outputs_directory_that_is_a_link_is_written_through_and_left_a_link() {
    let rig = Rig::new();
    let shared = rig._dir.path().join("shared-outputs");
    fs::create_dir_all(&shared).unwrap();
    fs::create_dir_all(rig.checkout.join(".moon/cache")).unwrap();
    std::os::unix::fs::symlink(&shared, rig.outputs()).unwrap();

    let entries = [rig.held(B7E1, b"archive")];
    rig.render(&entries).unwrap();
    assert_eq!(fs::read(shared.join("b7e1.tar.gz")).unwrap(), b"archive");
    let link = fs::symlink_metadata(rig.outputs()).unwrap();
    assert!(link.file_type().is_symlink(), "the link was replaced");
    assert_eq!(fs::read_link(rig.outputs()).unwrap(), shared);
}

#[test]
fn an_archive_already_there_is_not_written_over() {
    let rig = Rig::new();
    rig.left(B7E1, b"moon's own run");
    let entries = [rig.held(B7E1, b"another host's run")];
    let rendered = rig.render(&entries).unwrap();
    assert_eq!(rendered.holds, Holds::BuildIds(vec![BuildId::new(B7E1)]));
    assert_eq!(
        fs::read(rig.outputs().join("b7e1.tar.gz")).unwrap(),
        b"moon's own run"
    );
}

#[test]
fn bytes_the_store_answers_under_another_digest_are_refused_and_nothing_is_written() {
    let rig = Rig::new();
    let store = FakeStore::new();
    let digest = store.put_bytes(b"archive").unwrap().digest;
    store.corrupt(&digest, b"other bytes");
    let projection = MoonCache::new(&rig.checkout, Box::new(store));
    let entries = [CacheEntry {
        build: BuildId::new(B7E1),
        digest: digest.clone(),
    }];
    match projection.render(
        Contents::CacheEntries(&entries),
        rig.checkout.to_str().unwrap(),
    ) {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source, MoonCache::source());
            assert_eq!(
                r.why,
                Why::OtherBytes {
                    expected: digest,
                    answered: Digest::of(b"other bytes"),
                }
            );
        }
        other => panic!("answered {other:?}"),
    }
    assert_eq!(files(&rig.checkout), [], "an archive was written");
}

#[test]
fn an_entry_naming_a_manifest_is_refused() {
    let rig = Rig::new();
    let meta = ArtifactMeta::new(
        Ecosystem::new("file"),
        EcosystemName::new("site"),
        Version::new("1.0.0"),
    )
    .unwrap();
    let layer = rig.layout.put_bytes(b"site").unwrap().digest;
    let manifest = rig
        .layout
        .put_artifact(&Manifest::Artifact {
            meta,
            layers: vec![layer],
        })
        .unwrap()
        .digest;
    let entries = [CacheEntry {
        build: BuildId::new(B7E1),
        digest: manifest,
    }];
    match rig.render(&entries) {
        Err(Not::Refused(r)) => {
            assert_eq!(r.why, Why::Said);
            assert!(r.said.unwrap().contains("holds a manifest"));
        }
        other => panic!("answered {other:?}"),
    }
    assert_eq!(files(&rig.checkout), []);
}

#[test]
fn an_entry_the_store_does_not_hold_is_the_stores_answer() {
    let rig = Rig::new();
    let entries = [CacheEntry {
        build: BuildId::new(B7E1),
        digest: Digest::of(b"never put"),
    }];
    match rig.render(&entries) {
        Err(Not::Absent(a)) => assert_eq!(&a.source, rig.layout.source()),
        other => panic!("answered {other:?}"),
    }
    assert!(!rig.outputs().join("b7e1.tar.gz").exists());
}

// Requirement: A build identity that is no task hash is refused

#[test]
fn a_build_identity_with_a_path_in_it_refuses_the_render_and_nothing_is_written() {
    let rig = Rig::new();
    let store = FakeStore::new();
    let digest = store.put_bytes(b"archive").unwrap().digest;
    let projection = MoonCache::new(&rig.checkout, Box::new(store.clone()));
    let entries = [
        CacheEntry {
            build: BuildId::new(B7E1),
            digest: digest.clone(),
        },
        CacheEntry {
            build: BuildId::new("../escape"),
            digest,
        },
    ];
    match projection.render(
        Contents::CacheEntries(&entries),
        rig.checkout.to_str().unwrap(),
    ) {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source, MoonCache::source());
            assert!(r.said.unwrap().contains("../escape"));
        }
        other => panic!("answered {other:?}"),
    }
    let ops: Vec<StoreOp> = store.calls().into_iter().map(|c| c.op).collect();
    assert_eq!(ops, [StoreOp::PutBytes], "the store was read");
    assert_eq!(files(&rig.checkout), [], "a file was written");
    assert!(
        !rig.checkout.join(".moon").exists(),
        "the render made a directory"
    );
}

#[test]
fn serve_is_absent() {
    let rig = Rig::new();
    assert!(matches!(
        rig.projection().serve(&Layout { at: "at".into() }),
        Err(Not::Absent(_))
    ));
}

#[test]
fn the_pins_of_a_checkout_are_absent_naming_the_lockfile() {
    let rig = Rig::new();
    let absent = Not::absent(&MoonCache::source(), "lockfile");
    for checkout in [rig.checkout.clone(), rig.checkout.join("no checkout")] {
        assert_eq!(
            rig.projection().pinned(checkout.to_str().unwrap()),
            Err(absent.clone()),
            "the pins of {}",
            checkout.display()
        );
    }
    assert_eq!(files(&rig.checkout), [], "a file was written");
}
