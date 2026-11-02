//! serve (86, 115, 99; design D9): Cargo's sparse registry protocol
//! answered from a rendered layout, in the one shape of request,
//! response and miss the host's process serves every ecosystem in. The
//! request's path, below `/<owner>/<name>/cargo/`, names
//!
//! - `config.json`: the registry's configuration, `dl` under the base the
//!   request reached and no `api`, since the registry takes no publish;
//! - an index file, `1/<n>`, `2/<n>`, `3/<c>/<n>` or `<ab>/<cd>/<n>`:
//!   the layout's own, never asking the core, since what a layout lists
//!   is what the policy admitted when sync rendered it;
//! - `crates/<name>/<version>/download`: the crate's bytes, one the
//!   layout lacks asked of the core once and then answered from the
//!   layout (50).
//!
//! Every answer that is not the file or bytes asked for is `text/plain`,
//! the answer's own rendering, the source's words verbatim, which Cargo
//! prints under `body:`: `404` absent, `403` refused, `502` unreadable
//! (58, 59, 99). Every other path, the empty one among them, is `404`,
//! and a method other than `GET` or `HEAD` is `405`, so every request is
//! answered with a response; a layout file that cannot be read for
//! another reason than its absence is unreadable naming it.

use std::fs;
use std::io;
use std::path::Path;

use serde_json::json;
use tailrace_seams::{
    Answer, EcosystemName, Layout, Miss, Not, Pinned, Request, Response, Version,
};

use super::render::{crate_file_name, index_path, is_crate_name, unreadable, INDEX};
use super::CargoProjection;

const JSON: &str = "application/json";
const TEXT: &str = "text/plain; charset=utf-8";
const CRATE: &str = "application/octet-stream";

/// A `text/plain` response of `words`, with `status`.
fn words(status: u16, words: impl Into<String>) -> Response {
    let mut words = words.into();
    if !words.ends_with('\n') {
        words.push('\n');
    }
    Response {
        status,
        content_type: TEXT.into(),
        body: words.into_bytes(),
    }
}

/// The response Cargo reads for `not`, an answer that is not the bytes
/// it asked for: `404` absent, `403` refused, `502` unreadable, the words
/// the answer's own.
fn not_answered(not: &Not) -> Response {
    let status = match not {
        Not::Absent(_) => 404,
        Not::Refused(_) => 403,
        Not::Unreadable(_) => 502,
    };
    words(status, not.to_string())
}

/// What a request's path names in Cargo's sparse protocol.
enum Asked {
    Config,
    Index(String),
    Download(Pinned),
    Nothing,
}

/// The path, its slashes at either end left out, read as the
/// configuration's, an index file's or a crate's download.
fn asked(path: &str) -> Asked {
    let path = path.trim_matches('/');
    if path == "config.json" {
        return Asked::Config;
    }
    if let Some(rest) = path.strip_prefix("crates/") {
        let parts: Vec<&str> = rest.split('/').collect();
        return match parts[..] {
            [name, version, "download"]
                if is_crate_name(name) && !version.is_empty() && !version.contains(['\\', '/']) =>
            {
                Asked::Download(Pinned {
                    name: EcosystemName::new(name),
                    version: Version::new(version),
                })
            }
            _ => Asked::Nothing,
        };
    }
    let name = path.rsplit('/').next().unwrap_or_default();
    let indexed = is_crate_name(name)
        && name == name.to_ascii_lowercase()
        && index_path(name) == Path::new(INDEX).join(path);
    if indexed {
        Asked::Index(path.to_string())
    } else {
        Asked::Nothing
    }
}

/// The bytes at `path`, none where there is no such file; unreadable
/// naming it with the system's words where it cannot be read.
fn read(path: &Path) -> Answer<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(unreadable(path, &e)),
    }
}

/// The registry's configuration: downloads under `base`, and no `api`.
fn config(base: &str) -> Response {
    let dl = format!(
        "{}/crates/{{crate}}/{{version}}/download",
        base.trim_end_matches('/')
    );
    Response {
        status: 200,
        content_type: JSON.into(),
        body: serde_json::to_vec(&json!({ "dl": dl })).expect("a configuration is plain data"),
    }
}

/// The index file at `path` under the layout's index at `root`; `404`
/// where the layout holds no version of its name.
fn index(root: &Path, path: &str) -> Answer<Response> {
    let name = path.rsplit('/').next().unwrap_or_default();
    Ok(match read(&root.join(INDEX).join(path))? {
        Some(bytes) => Response {
            status: 200,
            content_type: TEXT.into(),
            body: bytes,
        },
        None => words(
            404,
            format!(
                "the layout holds no version of {name}; tailrace sync holds a crate a \
                 Cargo.toml declares in tailrace, so declare it there and run tailrace sync"
            ),
        ),
    })
}

/// The crate `pin` names from the layout at `root`; one the layout lacks
/// asked of the core once through `miss`, and answered from the layout
/// once the core holds it (115, 50).
fn download(root: &Path, pin: &Pinned, miss: Miss<'_>) -> Answer<Response> {
    let path = root.join(crate_file_name(pin.name.as_str(), pin.version.as_str()));
    let bytes = match read(&path)? {
        Some(bytes) => bytes,
        None => {
            if let Err(not) = miss(pin) {
                return Ok(not_answered(&not));
            }
            match read(&path)? {
                Some(bytes) => bytes,
                None => {
                    return Ok(words(
                        502,
                        format!(
                            "the layout holds no crate of {} {} after the core held it",
                            pin.name, pin.version
                        ),
                    ))
                }
            }
        }
    };
    Ok(Response {
        status: 200,
        content_type: CRATE.into(),
        body: bytes,
    })
}

impl CargoProjection {
    /// 86, 115, 99: `request` of the layout at `layout`, answered in
    /// Cargo's sparse registry protocol.
    pub(super) fn serve_request(
        &self,
        layout: &Layout,
        request: &Request,
        miss: Miss<'_>,
    ) -> Answer<Response> {
        if request.method != "GET" && request.method != "HEAD" {
            return Ok(words(
                405,
                format!(
                    "{} is not a method Cargo's sparse registry answers",
                    request.method
                ),
            ));
        }
        let root = self.under_root(&layout.at);
        match asked(&request.path) {
            Asked::Config => Ok(config(&request.base)),
            Asked::Index(path) => index(&root, &path),
            Asked::Download(pin) => download(&root, &pin, miss),
            Asked::Nothing => Ok(words(
                404,
                format!(
                    "{:?} names no configuration, index file or crate of Cargo's sparse registry",
                    request.path
                ),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::path::PathBuf;

    use tailrace_seams::{
        Artifact, ArtifactMeta, Contents, Ecosystem, Manifest, Projection, Seam, Source, Store, Why,
    };
    use tempfile::TempDir;

    use super::super::fixture::crate_of;
    use super::super::ECOSYSTEM;
    use super::*;
    use crate::profiles::store::layout::Layout as OciLayout;

    /// Where the tests render, under the checkout.
    const AT: &str = "layouts/cargo";

    /// The base a client reached the layout at.
    const BASE: &str = "http://127.0.0.1:7420/o/r/cargo";

    /// A checkout and a layout store, each in its own temporary directory.
    struct Rig {
        _dir: TempDir,
        checkout: PathBuf,
        store: OciLayout,
    }

    impl Rig {
        fn new() -> Rig {
            let dir = TempDir::new().unwrap();
            let checkout = dir.path().join("checkout");
            fs::create_dir_all(&checkout).unwrap();
            let store = OciLayout::create(dir.path().join("store")).unwrap();
            Rig {
                _dir: dir,
                checkout,
                store,
            }
        }

        fn projection(&self) -> CargoProjection {
            CargoProjection::new(&self.checkout, Box::new(self.store.clone()))
        }

        /// The crate of `name` at `version` held in the store as a
        /// publish holds it, and its bytes.
        fn held(&self, name: &str, version: &str) -> (Artifact, Vec<u8>) {
            let manifest = format!("[package]\nname = \"{name}\"\nversion = \"{version}\"\n");
            let bytes = crate_of(name, version, &manifest);
            let layer = self.store.put_bytes(&bytes).unwrap().digest;
            let meta = ArtifactMeta::new(
                Ecosystem::new(ECOSYSTEM),
                EcosystemName::new(name),
                Version::new(version),
            )
            .unwrap();
            let digest = self
                .store
                .put_artifact(&Manifest::Artifact {
                    meta: meta.clone(),
                    layers: vec![layer],
                })
                .unwrap()
                .digest;
            (Artifact { digest, meta }, bytes)
        }

        /// The layout of `artifacts` rendered at [`AT`].
        fn render(&self, artifacts: &[Artifact]) -> Layout {
            self.projection()
                .render(Contents::Artifacts(artifacts), AT)
                .unwrap()
                .layout
        }
    }

    /// The `GET` of `path` under [`BASE`].
    fn get(path: &str) -> Request {
        Request {
            method: "GET".into(),
            path: path.into(),
            base: BASE.into(),
        }
    }

    /// The versions each miss asked the core for, the core answering each
    /// with `answer`.
    struct Asked<F: Fn(&Pinned) -> Answer<()>> {
        pins: RefCell<Vec<Pinned>>,
        answer: F,
    }

    impl<F: Fn(&Pinned) -> Answer<()>> Asked<F> {
        fn new(answer: F) -> Self {
            Asked {
                pins: RefCell::new(Vec::new()),
                answer,
            }
        }

        fn miss(&self, pin: &Pinned) -> Answer<()> {
            self.pins.borrow_mut().push(pin.clone());
            (self.answer)(pin)
        }

        fn pins(&self) -> Vec<String> {
            self.pins
                .borrow()
                .iter()
                .map(|p| format!("{} {}", p.name, p.version))
                .collect()
        }
    }

    fn serve(
        rig: &Rig,
        layout: &Layout,
        request: &Request,
        asked: &Asked<impl Fn(&Pinned) -> Answer<()>>,
    ) -> Response {
        rig.projection()
            .serve(layout, request, &|pin| asked.miss(pin))
            .unwrap()
    }

    fn text(response: &Response) -> String {
        String::from_utf8(response.body.clone()).unwrap()
    }

    #[test]
    fn a_layout_holding_two_crates_serves_each_path_asking_the_core_nothing() {
        let rig = Rig::new();
        let (one, one_bytes) = rig.held("fam-a", "1.0.0");
        let (two, _) = rig.held("fam-a", "1.1.0");
        let layout = rig.render(&[one, two]);
        let asked = Asked::new(|_| Ok(()));

        let config = serve(&rig, &layout, &get("config.json"), &asked);
        assert_eq!((config.status, config.content_type.as_str()), (200, JSON));
        let config: serde_json::Value = serde_json::from_slice(&config.body).unwrap();
        assert_eq!(
            config,
            json!({ "dl": format!("{BASE}/crates/{{crate}}/{{version}}/download") }),
            "the configuration names other than downloads under the base, or an api"
        );

        let index = serve(&rig, &layout, &get("fa/m-/fam-a"), &asked);
        assert_eq!((index.status, index.content_type.as_str()), (200, TEXT));
        assert_eq!(
            index.body,
            fs::read(rig.checkout.join(AT).join("index/fa/m-/fam-a")).unwrap(),
            "the index file is other than the layout's"
        );

        let crate_ = serve(&rig, &layout, &get("crates/fam-a/1.0.0/download"), &asked);
        assert_eq!(
            (crate_.status, crate_.content_type.as_str(), &crate_.body),
            (200, CRATE, &one_bytes)
        );
        assert_eq!(asked.pins(), Vec::<String>::new(), "the core was asked");
    }

    #[test]
    fn the_configuration_names_downloads_under_the_base_the_request_reached() {
        let rig = Rig::new();
        let layout = rig.render(&[]);
        let asked = Asked::new(|_| Ok(()));
        let request = Request {
            base: "http://127.0.0.1:59999/o/r/cargo/".into(),
            ..get("config.json")
        };
        let config: serde_json::Value =
            serde_json::from_slice(&serve(&rig, &layout, &request, &asked).body).unwrap();
        assert_eq!(
            config["dl"],
            "http://127.0.0.1:59999/o/r/cargo/crates/{crate}/{version}/download"
        );
    }

    #[test]
    fn each_index_path_cargo_names_is_answered_from_the_layout() {
        let rig = Rig::new();
        let names = ["a", "ab", "abc", "fam-a"];
        let artifacts: Vec<Artifact> = names.iter().map(|n| rig.held(n, "1.0.0").0).collect();
        let layout = rig.render(&artifacts);
        let asked = Asked::new(|_| Ok(()));
        for path in ["1/a", "2/ab", "3/a/abc", "fa/m-/fam-a"] {
            let served = serve(&rig, &layout, &get(path), &asked);
            assert_eq!(served.status, 200, "{path}: {}", text(&served));
            assert!(text(&served).contains("\"vers\":\"1.0.0\""), "{path}");
        }
        for path in ["fa/m-/FAM-A", "fa/xx/fam-a", "2/a", "index/fa/m-/fam-a"] {
            let served = serve(&rig, &layout, &get(path), &asked);
            assert_eq!(served.status, 404, "{path}");
        }
        assert_eq!(asked.pins(), Vec::<String>::new(), "the core was asked");
    }

    #[test]
    fn an_index_file_of_a_name_the_layout_lacks_is_not_found_and_never_asks_the_core() {
        let rig = Rig::new();
        let (one, _) = rig.held("fam-a", "1.0.0");
        let layout = rig.render(&[one]);
        let asked = Asked::new(|_| Ok(()));
        let served = serve(&rig, &layout, &get("fa/m-/fam-b"), &asked);
        assert_eq!((served.status, served.content_type.as_str()), (404, TEXT));
        assert!(text(&served).contains("fam-b"), "{}", text(&served));
        assert_eq!(asked.pins(), Vec::<String>::new(), "the core was asked");
    }

    #[test]
    fn an_index_file_lists_only_the_versions_given_never_a_refused_one() {
        let rig = Rig::new();
        let (admitted, _) = rig.held("fam-a", "1.1.0");
        // 1.2.0 is held in the store, and the policy refused it, so the
        // core never gives it to render.
        rig.held("fam-a", "1.2.0");
        let layout = rig.render(&[admitted]);
        let asked = Asked::new(|_| Ok(()));
        let served = serve(&rig, &layout, &get("fa/m-/fam-a"), &asked);
        let versions: Vec<String> = text(&served)
            .lines()
            .map(|line| {
                let line: serde_json::Value = serde_json::from_str(line).unwrap();
                line["vers"].as_str().unwrap().to_string()
            })
            .collect();
        assert_eq!(versions, ["1.1.0"]);
    }

    #[test]
    fn a_crate_the_layout_lacks_is_asked_of_the_core_once_and_answered_once_held() {
        let rig = Rig::new();
        let (one, _) = rig.held("fam-a", "1.0.0");
        let (two, two_bytes) = rig.held("fam-a", "1.1.0");
        let layout = rig.render(std::slice::from_ref(&one));
        // The core holds the version asked by rendering both into the layout.
        let both = [one, two];
        let asked = Asked::new(|_| {
            rig.projection()
                .render(Contents::Artifacts(&both), AT)
                .map(|_| ())
        });
        let served = serve(&rig, &layout, &get("crates/fam-a/1.1.0/download"), &asked);
        assert_eq!(asked.pins(), ["fam-a 1.1.0"]);
        assert_eq!((served.status, &served.body), (200, &two_bytes));
    }

    #[test]
    fn a_crate_the_core_refuses_is_forbidden_carrying_its_words_verbatim() {
        let rig = Rig::new();
        let (one, _) = rig.held("fam-a", "1.0.0");
        let layout = rig.render(&[one]);
        let words = "acceptance wanting";
        let asked = Asked::new(|_| {
            Err(Not::refused(
                &Source::new(Seam::Policy, "consumer"),
                Why::Said,
                Some(words.into()),
            ))
        });
        let served = serve(&rig, &layout, &get("crates/fam-a/1.2.0/download"), &asked);
        assert_eq!(asked.pins(), ["fam-a 1.2.0"]);
        assert_eq!((served.status, served.content_type.as_str()), (403, TEXT));
        assert!(text(&served).contains(words), "{}", text(&served));
    }

    #[test]
    fn a_crate_the_core_holds_none_of_or_cannot_read_is_not_found_or_a_bad_gateway() {
        let rig = Rig::new();
        let layout = rig.render(&[]);
        let source = Source::new(Seam::Store, "registry");
        for (not, status) in [
            (Not::absent(&source, "artifact/cargo/fam-a@9.9.9"), 404),
            (
                Not::unreadable(
                    &source,
                    "artifact/cargo/fam-a@9.9.9",
                    Some("connection refused".into()),
                ),
                502,
            ),
        ] {
            let asked = Asked::new(|_| Err(not.clone()));
            let served = serve(&rig, &layout, &get("crates/fam-a/9.9.9/download"), &asked);
            assert_eq!(served.status, status);
            assert_eq!(text(&served).trim_end(), not.to_string());
        }
    }

    #[test]
    fn a_method_other_than_get_or_head_is_not_allowed() {
        let rig = Rig::new();
        let (one, _) = rig.held("fam-a", "1.0.0");
        let layout = rig.render(&[one]);
        let asked = Asked::new(|_| Ok(()));
        let head = Request {
            method: "HEAD".into(),
            ..get("crates/fam-a/1.0.0/download")
        };
        assert_eq!(serve(&rig, &layout, &head, &asked).status, 200);
        for method in ["PUT", "DELETE", "POST"] {
            let request = Request {
                method: method.into(),
                ..get("crates/fam-a/1.0.0/download")
            };
            let served = serve(&rig, &layout, &request, &asked);
            assert_eq!((served.status, served.content_type.as_str()), (405, TEXT));
        }
        assert_eq!(asked.pins(), Vec::<String>::new(), "the core was asked");
    }

    #[test]
    fn the_empty_path_and_any_other_are_answered_not_found() {
        let rig = Rig::new();
        let layout = rig.render(&[]);
        let asked = Asked::new(|_| Ok(()));
        for path in [
            "",
            "/",
            "api/v1/crates",
            "crates/fam-a/1.0.0",
            "crates/../x/1/download",
        ] {
            let served = serve(&rig, &layout, &get(path), &asked);
            assert_eq!(
                (served.status, served.content_type.as_str()),
                (404, TEXT),
                "{path:?}"
            );
        }
        assert_eq!(asked.pins(), Vec::<String>::new(), "the core was asked");
    }
}
