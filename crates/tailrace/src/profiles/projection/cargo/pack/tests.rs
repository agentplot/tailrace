//! pack's own steps over checkouts written as text and crates made in
//! memory, with no cargo run (design D11).

use tailrace_fakes::{FakeStore, Fault, StoreOp, Times};
use tailrace_seams::{artifact_name, Component, ComponentName, Digest, OutputName, PackAt, Store};
use tempfile::TempDir;

use super::super::fixture::{crate_of, packed_crate};
use super::*;

/// A checkout holding `files`, and the projection of it reading crates
/// from `store`.
fn checkout_with(files: &[(&str, &str)], store: FakeStore) -> (TempDir, CargoProjection) {
    let dir = TempDir::new().unwrap();
    for (path, text) in files {
        let at = dir.path().join(path);
        fs::create_dir_all(at.parent().unwrap()).unwrap();
        fs::write(at, text).unwrap();
    }
    let projection = CargoProjection::new(dir.path(), Box::new(store));
    (dir, projection)
}

fn checkout(files: &[(&str, &str)]) -> (TempDir, CargoProjection) {
    checkout_with(files, FakeStore::new())
}

fn output(name: &str, files: &[&str]) -> Output {
    Output {
        name: OutputName::new(name),
        files: files.iter().map(|f| f.to_string()).collect(),
    }
}

/// The crate file a packaging task leaves for `name`, packed from
/// `path_in_vcs`, written under the checkout's target directory, and its
/// path relative to the checkout.
fn left(dir: &TempDir, name: &str, path_in_vcs: Option<&str>) -> String {
    let file = format!("target/package/{name}-0.0.0.crate");
    let manifest = format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\n");
    let at = dir.path().join(&file);
    fs::create_dir_all(at.parent().unwrap()).unwrap();
    fs::write(at, packed_crate(name, "0.0.0", &manifest, path_in_vcs)).unwrap();
    file
}

fn refused<T: std::fmt::Debug>(answer: Answer<T>) -> String {
    match answer {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source, CargoProjection::source());
            assert_eq!(r.why, Why::Said);
            r.said.unwrap()
        }
        other => panic!("answered {other:?}"),
    }
}

// Requirement: A crate packed by Cargo's own packager, design D1

#[test]
fn the_one_crate_file_names_the_package_and_the_manifest_it_was_packed_from() {
    let (dir, projection) = checkout(&[]);
    let file = left(&dir, "fam-a", Some("crates/fam-a"));
    let found = projection
        .crate_of(&output("fam-a", &[&file, "target/package/fam-a.log"]))
        .unwrap();
    assert_eq!(
        found,
        Found {
            name: "fam-a".into(),
            manifest: dir.path().join("crates/fam-a/Cargo.toml"),
        }
    );
    let absolute = dir.path().join(&file).display().to_string();
    assert_eq!(
        projection.crate_of(&output("fam-a", &[&absolute])),
        Ok(found)
    );
    let root = left(&dir, "fam-root", Some(""));
    assert_eq!(
        projection
            .crate_of(&output("fam", &[&root]))
            .unwrap()
            .manifest,
        dir.path().join("Cargo.toml")
    );
}

#[test]
fn an_output_with_no_crate_file_is_refused_naming_it_and_its_files() {
    let (_dir, projection) = checkout(&[("crates/fam-a/Cargo.toml", "[package]\n")]);
    assert_eq!(
        refused(projection.crate_of(&output("fam-a", &["crates/fam-a/Cargo.toml"]))),
        "the output fam-a left no crate file among its files, crates/fam-a/Cargo.toml, and \
         the cargo projection packs one crate"
    );
    assert!(refused(projection.crate_of(&output("fam-a", &[]))).contains("its files, no file"));
}

#[test]
fn an_output_with_two_crate_files_is_refused_naming_both() {
    let (dir, projection) = checkout(&[]);
    let a = left(&dir, "fam-a", Some("crates/fam-a"));
    let b = left(&dir, "fam-b", Some("crates/fam-b"));
    assert_eq!(
        refused(projection.crate_of(&output("fam", &[&a, &b]))),
        format!(
            "the output fam left 2 crate files, {}, {}, among its files, {a}, {b}, and the \
             cargo projection packs one crate",
            dir.path().join(&a).display(),
            dir.path().join(&b).display()
        )
    );
}

#[test]
fn a_crate_packed_outside_git_is_refused_naming_it() {
    let (dir, projection) = checkout(&[]);
    let file = left(&dir, "fam-a", None);
    assert_eq!(
        refused(projection.crate_of(&output("fam-a", &[&file]))),
        format!(
            "the crate {} names no place in the repository it was packed from, which cargo \
             package writes as path_in_vcs in a git checkout",
            dir.path().join(&file).display()
        )
    );
}

#[test]
fn a_crate_file_that_is_no_crate_is_unreadable_in_the_readers_words() {
    let (dir, projection) = checkout(&[("fam-a.crate", "not gzip")]);
    match projection.crate_of(&output("fam-a", &["fam-a.crate"])) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.what, dir.path().join("fam-a.crate").display().to_string());
            assert!(u.said.is_some_and(|s| !s.is_empty()));
        }
        other => panic!("answered {other:?}"),
    }
}

// Requirement: A crate packed by Cargo's own packager, and a crate of
// another component depended on as the manifest writes, design D3

/// The workspace's manifest: its members, the version they inherit and a
/// dependency they may inherit.
const ROOT: &str = "[workspace]\n\
members = [\"crates/*\", \"base/*\"]\n\
\n\
[workspace.package]\n\
version = \"0.0.0\" # the tag's\n\
\n\
[workspace.dependencies]\n\
fam-d = { path = \"crates/fam-d\", version = \"0.0.0\" }\n\
serde = \"1\"\n";

const FAM_A: &str = "[package]\n\
name = \"fam-a\"\n\
version.workspace = true\n\
\n\
[dependencies]\n\
fam-b = { path = \"../fam-b\", version = \"0.0.0\", registry = \"tailrace\" }\n\
serde = \"1\"\n\
\n\
[dev-dependencies]\n\
fam-test = { path = \"../fam-test\" }\n";

const FAM_B: &str = "[package]\n\
name = \"fam-b\"\n\
version = \"0.0.0\"\n\
\n\
[target.'cfg(unix)'.dependencies]\n\
fam-c = { path = \"../fam-c\" }\n";

const FAM_C: &str = "[package]\nname = \"fam-c\"\nversion = \"0.0.0\"\n";

const FAM_D: &str = "[package]\nname = \"fam-d\"\nversion.workspace = true\n";

const BASE_C: &str = "[package]\nname = \"base-c\"\nversion = \"0.3.1\"\n";

/// A workspace of the component `fam`, whose paths hold `crates/fam-*`,
/// and of `base`, beside `extra` for `fam-a`'s manifest.
fn workspace(fam_a: &str, store: FakeStore) -> (TempDir, CargoProjection) {
    checkout_with(
        &[
            ("Cargo.toml", ROOT),
            ("crates/fam-a/Cargo.toml", fam_a),
            ("crates/fam-b/Cargo.toml", FAM_B),
            ("crates/fam-c/Cargo.toml", FAM_C),
            ("crates/fam-d/Cargo.toml", FAM_D),
            (
                "crates/fam-test/Cargo.toml",
                "[package]\nname = \"fam-test\"\n",
            ),
            ("base/base-c/Cargo.toml", BASE_C),
        ],
        store,
    )
}

/// The component `fam` and an output for each of `crates` with the crate
/// file its packaging task left.
fn fam(dir: &TempDir, crates: &[&str]) -> (Component, Vec<Output>) {
    let outputs: Vec<Output> = crates
        .iter()
        .map(|name| {
            let path = format!("crates/{name}");
            let file = left(dir, name, Some(path.as_str()));
            output(name, &[&file])
        })
        .collect();
    let component = Component {
        name: ComponentName::new("fam"),
        outputs: outputs.iter().map(|o| o.name.clone()).collect(),
        paths: vec!["crates/fam-*/**".into()],
        prefix: "fam-v".into(),
    };
    (component, outputs)
}

fn holds(path: &str) -> bool {
    path.starts_with("crates/fam-")
}

/// The plan of packing `fam-a` at 1.4.0 as a component of `crates`.
fn plan_of(dir: &TempDir, projection: &CargoProjection, crates: &[&str]) -> Answer<Plan> {
    let (component, outputs) = fam(dir, crates);
    let version = Version::new("1.4.0");
    let at = PackAt {
        version: &version,
        component: &component,
        outputs: &outputs,
        holds: &holds,
    };
    let root = canonical(dir.path());
    projection.plan(&root.join("crates/fam-a/Cargo.toml"), &root, &at)
}

/// Each file a plan edits, relative to the checkout, and its text.
fn edited(dir: &TempDir, plan: &Plan) -> Vec<(String, String)> {
    let root = canonical(dir.path());
    plan.texts()
        .into_iter()
        .map(|(path, text)| {
            let path = path.strip_prefix(&root).unwrap().display().to_string();
            (path, text)
        })
        .collect()
}

#[test]
fn a_sibling_and_a_sibling_of_a_sibling_take_the_version_in_the_registry() {
    let (dir, projection) = workspace(FAM_A, FakeStore::new());
    let plan = plan_of(&dir, &projection, &["fam-a", "fam-b", "fam-c"]).unwrap();
    assert_eq!(
        edited(&dir, &plan),
        [
            (
                "Cargo.toml".to_string(),
                ROOT.replace("\"0.0.0\" # the", "\"1.4.0\" # the")
            ),
            (
                "crates/fam-a/Cargo.toml".to_string(),
                FAM_A.replace(
                    "version = \"0.0.0\", registry",
                    "version = \"1.4.0\", registry"
                )
            ),
            (
                "crates/fam-b/Cargo.toml".to_string(),
                FAM_B
                    .replace("version = \"0.0.0\"", "version = \"1.4.0\"")
                    .replace(
                        "fam-c = { path = \"../fam-c\" }",
                        "fam-c = { path = \"../fam-c\", version = \"1.4.0\", registry = \
                         \"tailrace\" }"
                    )
            ),
            (
                "crates/fam-c/Cargo.toml".to_string(),
                FAM_C.replace("\"0.0.0\"", "\"1.4.0\"")
            ),
        ]
    );
    assert_eq!(plan.others, []);
}

#[test]
fn a_sibling_none_of_the_components_outputs_packs_is_refused_naming_it_and_the_component() {
    let (dir, projection) = workspace(FAM_A, FakeStore::new());
    let said = refused(plan_of(&dir, &projection, &["fam-a", "fam-c"]));
    assert_eq!(
        said,
        format!(
            "the crate fam-b, at {}, is of the component fam by its paths, and none of the \
             component's outputs packs it, so no consumer could resolve it",
            canonical(dir.path())
                .join("crates/fam-b/Cargo.toml")
                .display()
        )
    );
}

#[test]
fn an_inherited_dependency_on_a_sibling_is_given_the_version_where_the_workspace_writes_it() {
    let fam_a = "[package]\nname = \"fam-a\"\nversion.workspace = true\n\n\
                 [build-dependencies]\nfam-d = { workspace = true }\n";
    let (dir, projection) = workspace(fam_a, FakeStore::new());
    let plan = plan_of(&dir, &projection, &["fam-a", "fam-d"]).unwrap();
    assert_eq!(
        edited(&dir, &plan),
        [(
            "Cargo.toml".to_string(),
            ROOT.replace("\"0.0.0\" # the", "\"1.4.0\" # the").replace(
                "fam-d = { path = \"crates/fam-d\", version = \"0.0.0\" }",
                "fam-d = { path = \"crates/fam-d\", version = \"1.4.0\", registry = \
                     \"tailrace\" }"
            )
        ),],
        "fam-a's own manifest, which inherits its version, was edited"
    );
}

#[test]
fn a_crate_of_another_component_keeps_its_requirement_and_its_manifest() {
    let fam_a = "[package]\nname = \"fam-a\"\nversion.workspace = true\n\n\
                 [dependencies]\nbase-c = { path = \"../../base/base-c\", version = \"0.3\" }\n";
    let (dir, projection) = workspace(fam_a, FakeStore::new());
    let plan = plan_of(&dir, &projection, &["fam-a"]).unwrap();
    assert_eq!(
        edited(&dir, &plan),
        [
            (
                "Cargo.toml".to_string(),
                ROOT.replace("\"0.0.0\" # the", "\"1.4.0\" # the")
            ),
            (
                "crates/fam-a/Cargo.toml".to_string(),
                fam_a.replace(
                    "version = \"0.3\" }",
                    "version = \"0.3\", registry = \"tailrace\" }"
                )
            ),
        ],
        "base-c's manifest was edited"
    );
    assert_eq!(
        plan.others,
        [Other {
            name: "base-c".into(),
            requirement: "0.3".into()
        }]
    );
}

#[test]
fn a_crate_of_another_component_with_no_version_is_refused_naming_it() {
    let fam_a = "[package]\nname = \"fam-a\"\nversion.workspace = true\n\n\
                 [dependencies]\nbase-c = { path = \"../../base/base-c\" }\n";
    let (dir, projection) = workspace(fam_a, FakeStore::new());
    assert_eq!(
        refused(plan_of(&dir, &projection, &["fam-a"])),
        format!(
            "the dependency on base-c, of another component than fam, in {} writes no \
             version, and a crate of another component is packed at the requirement its \
             manifest writes",
            canonical(dir.path())
                .join("crates/fam-a/Cargo.toml")
                .display()
        )
    );
}

// Requirement: A crate of another component is depended on as the
// manifest writes, design D3: the store's listing read before cargo runs

/// A store naming `base-c` at each of `versions`.
fn holding(versions: &[&str]) -> FakeStore {
    let store = FakeStore::new();
    let digest: Digest = store.put_bytes(b"an index").unwrap().digest;
    for version in versions {
        let name = artifact_name(
            &Ecosystem::new(ECOSYSTEM),
            &EcosystemName::new("base-c"),
            &Version::new(*version),
        );
        store.name(&name, &digest, None).unwrap();
    }
    store
}

fn base_c() -> [Other; 1] {
    [Other {
        name: "base-c".into(),
        requirement: "0.3".into(),
    }]
}

#[test]
fn a_crate_the_store_holds_within_the_requirement_is_packed_on() {
    let (_dir, projection) = checkout_with(&[], holding(&["0.4.0", "0.3.1"]));
    assert_eq!(projection.within_store(&base_c()), Ok(()));
}

#[test]
fn a_crate_the_store_holds_only_outside_the_requirement_is_refused_naming_what_it_holds() {
    let (_dir, projection) = checkout_with(&[], holding(&["0.4.0", "0.2.9"]));
    assert_eq!(
        refused(projection.within_store(&base_c())),
        "base-c of another component is required at 0.3, and the store holds it only at \
         0.2.9, 0.4.0"
    );
}

#[test]
fn a_crate_the_store_names_at_no_version_is_refused_saying_so() {
    let (_dir, projection) = checkout_with(&[], holding(&[]));
    assert_eq!(
        refused(projection.within_store(&base_c())),
        "base-c of another component is required at 0.3, and the store holds no version of it"
    );
}

#[test]
fn a_listing_the_store_cannot_read_is_the_answer_as_the_store_gave_it() {
    let store = holding(&["0.3.1"]);
    store.fault(
        StoreOp::Versions,
        Fault::Unreadable("the registry is down".into()),
        Times::Once,
    );
    let expected = store
        .versions(&Ecosystem::new(ECOSYSTEM), &EcosystemName::new("base-c"))
        .unwrap_err();
    store.fault(
        StoreOp::Versions,
        Fault::Unreadable("the registry is down".into()),
        Times::Once,
    );
    let (_dir, projection) = checkout_with(&[], store);
    assert_eq!(projection.within_store(&base_c()), Err(expected));
}

// Requirement: A pack leaves the checkout as it was and refuses what it
// cannot promise

/// `original`, the bytes of the manifest at `path`, with the value
/// `held` names set to `version`.
fn with_version(path: &Path, original: &[u8], held: Held, version: &Version) -> Answer<String> {
    let mut doc: DocumentMut = std::str::from_utf8(original).unwrap().parse().unwrap();
    set_version(doc.as_table_mut(), path, held, version)?;
    Ok(doc.to_string())
}

#[test]
fn a_packages_own_version_is_set_and_the_rest_of_the_file_kept() {
    let original = "# the crate\n[package]\nname = \"fam\"\nversion = \"0.0.0\" # set at publish\n\
                    edition = \"2021\"\n\n[dependencies]\nserde = \"1\"\n";
    let edited = with_version(
        Path::new("Cargo.toml"),
        original.as_bytes(),
        Held::Package,
        &Version::new("1.4.0"),
    )
    .unwrap();
    assert_eq!(
        edited,
        original.replace("\"0.0.0\"", "\"1.4.0\""),
        "more than the version changed"
    );
}

#[test]
fn a_workspaces_version_is_set_where_the_package_inherits_it() {
    let (dir, _projection) = checkout(&[
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"fam\"]\n\n[workspace.package]\nversion = \"0.0.0\"\n",
        ),
        (
            "fam/Cargo.toml",
            "[package]\nname = \"fam\"\nversion.workspace = true\n",
        ),
    ]);
    let manifest = dir.path().join("fam/Cargo.toml");
    let (holder_path, held) = holder(&manifest, dir.path()).unwrap();
    assert_eq!(
        (holder_path.clone(), held),
        (dir.path().join("Cargo.toml"), Held::Workspace)
    );
    let original = fs::read(&holder_path).unwrap();
    let edited = with_version(&holder_path, &original, held, &Version::new("2.0.0-pre.1")).unwrap();
    assert_eq!(
        edited,
        "[workspace]\nmembers = [\"fam\"]\n\n[workspace.package]\nversion = \"2.0.0-pre.1\"\n"
    );
    let inline = dir.path().join("fam/Cargo.toml");
    fs::write(
        &inline,
        "[package]\nname = \"fam\"\nversion = { workspace = true }\n",
    )
    .unwrap();
    assert_eq!(holder(&inline, dir.path()).unwrap().1, Held::Workspace);
}

#[test]
fn a_package_naming_no_version_is_given_one() {
    let original = "[package]\nname = \"fam\"\n";
    let edited = with_version(
        Path::new("Cargo.toml"),
        original.as_bytes(),
        Held::Package,
        &Version::new("1.4.0"),
    )
    .unwrap();
    assert_eq!(edited, "[package]\nname = \"fam\"\nversion = \"1.4.0\"\n");
}

#[test]
fn a_workspace_naming_no_version_is_unreadable_naming_the_value() {
    match with_version(
        Path::new("Cargo.toml"),
        b"[workspace]\nmembers = []\n",
        Held::Workspace,
        &Version::new("1.4.0"),
    ) {
        Err(Not::Unreadable(u)) => assert_eq!(
            u.said.as_deref(),
            Some("the manifest names no workspace.package.version")
        ),
        other => panic!("answered {other:?}"),
    }
}

#[test]
fn a_crate_naming_another_version_is_refused_naming_both() {
    let manifest = Path::new("/checkout/fam/Cargo.toml");
    let bytes = crate_of(
        "fam",
        "1.4.1",
        "[package]\nname = \"fam\"\nversion = \"1.4.1\"\n",
    );
    assert_eq!(
        names_package(&bytes, manifest, "fam", &Version::new("1.4.1")),
        Ok(())
    );
    match names_package(&bytes, manifest, "fam", &Version::new("1.4.0")) {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source, CargoProjection::source());
            assert_eq!(
                r.said.as_deref(),
                Some(
                    "the crate cargo packed of /checkout/fam/Cargo.toml names fam 1.4.1, and \
                     the version given is fam 1.4.0"
                )
            );
        }
        other => panic!("answered {other:?}"),
    }
    match names_package(b"no crate", manifest, "fam", &Version::new("1.4.0")) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.what, "the crate cargo packed of /checkout/fam/Cargo.toml")
        }
        other => panic!("answered {other:?}"),
    }
}
