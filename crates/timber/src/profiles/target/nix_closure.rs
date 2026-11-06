//! A Nix closure's form, read here for every Nix target profile (C.12;
//! design D6). A machine's build task leaves one file naming its
//! closure's roots, one absolute store path per line,
//! `<store dir>/<32 characters of Nix's base32>-<name>`, as `nix build
//! --print-out-paths` prints them; the component declares it as an
//! output, and the file projection packs it as the output's one file.
//! The store holds that artifact and no store path of the closure: the
//! paths are the Nix binary cache's (section 10).
//!
//! A profile finds the artifact of an output in a version's index with
//! [`artifact_of`], reads its bytes through its own store binding, and
//! reads them into the closure's roots with [`roots`]; each answers
//! refused naming what it found wanting, in the asking profile's name.
//! The profiles share this module so that none calls another (rule
//! core.2). It is the profile's own code and links no crate of Nix's.

use std::collections::HashMap;
use std::fmt;

use timber_seams::{Answer, Artifact, IndexMeta, Not, OutputName, Source, Why};

/// Where a Nix store holds its paths unless `NIX_STORE_DIR` names
/// another directory.
pub const STORE_DIR: &str = "/nix/store";

/// The variable naming another store directory than [`STORE_DIR`], as
/// Nix itself reads it.
const STORE_DIR_ENV: &str = "NIX_STORE_DIR";

/// The 32 characters of Nix's base32: the digits and the lowercase
/// letters but `e`, `o`, `u` and `t`.
const BASE32: &str = "0123456789abcdfghijklmnpqrsvwxyz";

/// How many characters of Nix's base32 a store path's hash is.
const HASH_LEN: usize = 32;

/// The most characters a store path's name may hold, as Nix bounds it.
const NAME_MAX: usize = 211;

/// The ecosystem the file projection packs in, as an index names a
/// closure file's artifact; read from the index, so no profile asks that
/// projection anything (rule core.2).
const FILE: &str = "file";

/// One root of a closure: an absolute store path, read checked.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StorePath(String);

impl StorePath {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StorePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The directory this host's Nix store holds its paths in:
/// `NIX_STORE_DIR` where it names one, else [`STORE_DIR`].
pub fn store_dir() -> String {
    store_dir_of(std::env::var(STORE_DIR_ENV).ok())
}

/// The store directory `given` names, its trailing `/` dropped, or
/// [`STORE_DIR`] where it names none.
fn store_dir_of(given: Option<String>) -> String {
    match given.as_deref().map(|dir| dir.trim_end_matches('/')) {
        Some(dir) if !dir.is_empty() => dir.to_string(),
        _ => STORE_DIR.to_string(),
    }
}

/// The file projection's artifact of `output` among those `index`
/// names, refused naming the output and the version where it names none
/// (design D8).
pub fn artifact_of<'a>(
    index: &'a IndexMeta,
    output: &OutputName,
    source: &Source,
) -> Answer<&'a Artifact> {
    index
        .artifacts()
        .iter()
        .find(|a| a.meta.ecosystem().as_str() == FILE && a.meta.name().as_str() == output.as_str())
        .ok_or_else(|| {
            refused(
                source,
                format!(
                    "version {} of {} names no {FILE} artifact of the output {output}, \
                     so it carries no closure of it",
                    index.version(),
                    index.component()
                ),
            )
        })
}

/// The roots `bytes`, the bytes of `artifact`, name, in the order its
/// lines give them, each a store path under `store_dir`. The last line
/// may end with a newline. Refused naming the artifact where it holds no
/// line, and naming the artifact and the line where a line is empty,
/// repeats one before it, or is no store path (design D6).
pub fn roots(
    artifact: &Artifact,
    bytes: &[u8],
    store_dir: &str,
    source: &Source,
) -> Answer<Vec<StorePath>> {
    let file = ClosureFile(artifact);
    let text = bytes.strip_suffix(b"\n").unwrap_or(bytes);
    if bytes.is_empty() {
        return Err(refused(source, format!("{file} holds no line")));
    }
    let mut seen: HashMap<&[u8], usize> = HashMap::new();
    let mut roots = Vec::new();
    for (number, line) in (1..).zip(text.split(|&b| b == b'\n')) {
        let at = |problem: String| {
            refused(
                source,
                format!(
                    "line {number} of {file}, `{}`, {problem}",
                    String::from_utf8_lossy(line)
                ),
            )
        };
        if line.is_empty() {
            return Err(refused(source, format!("line {number} of {file} is empty")));
        }
        if let Some(first) = seen.insert(line, number) {
            return Err(at(format!("repeats line {first}")));
        }
        let path = std::str::from_utf8(line)
            .map_err(|_| at("is no store path: it is not text".into()))
            .and_then(|path| {
                store_path(path, store_dir).map_err(|problem| {
                    at(format!(
                        "is no store path {store_dir}/<hash>-<name>: {problem}"
                    ))
                })
            })?;
        roots.push(path);
    }
    Ok(roots)
}

/// `text` read as a store path under `store_dir`, or what keeps it from
/// being one.
fn store_path(text: &str, store_dir: &str) -> Result<StorePath, String> {
    let base = text
        .strip_prefix(store_dir)
        .and_then(|rest| rest.strip_prefix('/'))
        .ok_or_else(|| format!("it is not in the store directory {store_dir}"))?;
    let (hash, name) = base
        .split_once('-')
        .ok_or_else(|| "no `-` follows its hash".to_string())?;
    let length = hash.chars().count();
    if length != HASH_LEN {
        return Err(format!("its hash is {length} characters, not {HASH_LEN}"));
    }
    if let Some(c) = hash.chars().find(|&c| !BASE32.contains(c)) {
        return Err(format!("its hash holds `{c}`, outside Nix's base32"));
    }
    name_problem(name).map_or_else(|| Ok(StorePath(text.to_string())), Err)
}

/// What keeps `name` from being a store path's name, as Nix reads one:
/// none, more than [`NAME_MAX`] characters, a character but a letter, a
/// digit or one of `+-._?=`, or `.` or `..` alone or before a `-`.
fn name_problem(name: &str) -> Option<String> {
    if name.is_empty() {
        return Some("it names nothing after its hash".into());
    }
    let length = name.chars().count();
    if length > NAME_MAX {
        return Some(format!(
            "its name is {length} characters, more than {NAME_MAX}"
        ));
    }
    if let Some(c) = name
        .chars()
        .find(|&c| !(c.is_ascii_alphanumeric() || "+-._?=".contains(c)))
    {
        return Some(format!("its name holds `{c}`"));
    }
    let dots = name.split('-').next().unwrap_or(name);
    if dots == "." || dots == ".." {
        return Some(format!("its name begins `{dots}`"));
    }
    None
}

/// A closure file's artifact, as a refusal names it.
pub struct ClosureFile<'a>(pub &'a Artifact);

impl fmt::Display for ClosureFile<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let meta = &self.0.meta;
        write!(
            f,
            "the closure file {}@{} ({})",
            meta.name(),
            meta.version(),
            self.0.digest
        )
    }
}

fn refused(source: &Source, said: String) -> Not {
    Not::refused(source, Why::Said, Some(said))
}

#[cfg(test)]
mod tests {
    use timber_seams::{
        ArtifactMeta, ComponentName, Digest, Ecosystem, EcosystemName, Seam, Version,
    };

    use super::*;

    const ROOT: &str = "/nix/store/0c7yzjm6jy3dxfyb8zd6x0fv1r6l1sbm-nixos-system-srv-25.05";
    const LEAF: &str = "/nix/store/9kn2wd8q1ycx0l6hzm4fjrb3gv5sap7i-hello-2.12.1";

    fn source() -> Source {
        Source::new(Seam::Target, "nix_cache")
    }

    fn artifact(ecosystem: &str, name: &str, bytes: &[u8]) -> Artifact {
        Artifact {
            digest: Digest::of(bytes),
            meta: ArtifactMeta::new(
                Ecosystem::new(ecosystem),
                EcosystemName::new(name),
                Version::new("0.4.0"),
            )
            .unwrap(),
        }
    }

    /// The roots `text` names, read as the closure file of
    /// `swancloud-srv` at 0.4.0 under the default store directory.
    fn read(text: &str) -> Answer<Vec<StorePath>> {
        let bytes = text.as_bytes();
        roots(
            &artifact(FILE, "swancloud-srv", bytes),
            bytes,
            STORE_DIR,
            &source(),
        )
    }

    /// The words `text` is refused in, naming the artifact.
    fn refused_with(text: &str) -> String {
        match read(text) {
            Err(Not::Refused(r)) => {
                assert_eq!(r.source, source());
                assert_eq!(r.why, Why::Said);
                let said = r.said.unwrap();
                let named = format!(
                    "the closure file swancloud-srv@0.4.0 ({})",
                    Digest::of(text.as_bytes())
                );
                assert!(said.contains(&named), "{said} names not {named}");
                said
            }
            other => panic!("{text:?} answered {other:?}"),
        }
    }

    fn paths(roots: &[StorePath]) -> Vec<&str> {
        roots.iter().map(StorePath::as_str).collect()
    }

    #[test]
    fn one_line_is_one_root() {
        assert_eq!(paths(&read(&format!("{ROOT}\n")).unwrap()), [ROOT]);
        assert_eq!(paths(&read(ROOT).unwrap()), [ROOT]);
    }

    #[test]
    fn two_lines_are_two_roots_in_their_order() {
        let roots = read(&format!("{LEAF}\n{ROOT}\n")).unwrap();
        assert_eq!(paths(&roots), [LEAF, ROOT]);
    }

    #[test]
    fn an_empty_file_is_refused_as_holding_no_line() {
        let said = refused_with("");
        assert!(said.ends_with("holds no line"), "{said}");
    }

    #[test]
    fn an_empty_line_is_refused_naming_it() {
        let said = refused_with(&format!("{ROOT}\n\n{LEAF}\n"));
        assert!(said.starts_with("line 2 of the closure file"), "{said}");
        assert!(said.ends_with("is empty"), "{said}");
        let said = refused_with("\n");
        assert!(said.starts_with("line 1 of"), "{said}");
    }

    #[test]
    fn a_repeated_line_is_refused_naming_it_and_the_first() {
        let said = refused_with(&format!("{ROOT}\n{LEAF}\n{ROOT}\n"));
        assert!(said.starts_with("line 3 of the closure file"), "{said}");
        assert!(said.contains(&format!("`{ROOT}`")), "{said}");
        assert!(said.ends_with("repeats line 1"), "{said}");
    }

    #[test]
    fn result_is_no_store_path() {
        let said = refused_with("result\n");
        assert!(said.contains("line 1 of"), "{said}");
        assert!(said.contains("`result`"), "{said}");
        assert!(
            said.ends_with("is no store path /nix/store/<hash>-<name>: it is not in the store directory /nix/store"),
            "{said}"
        );
    }

    #[test]
    fn a_path_outside_the_store_directory_is_no_store_path() {
        for path in [
            "/tmp/0c7yzjm6jy3dxfyb8zd6x0fv1r6l1sbm-hello",
            "/nix/store-other/0c7yzjm6jy3dxfyb8zd6x0fv1r6l1sbm-hello",
            "nix/store/0c7yzjm6jy3dxfyb8zd6x0fv1r6l1sbm-hello",
        ] {
            let said = refused_with(path);
            assert!(said.contains(&format!("`{path}`")), "{said}");
            assert!(
                said.ends_with("it is not in the store directory /nix/store"),
                "{said}"
            );
        }
    }

    #[test]
    fn a_hash_of_31_characters_is_no_store_path() {
        let said = refused_with("/nix/store/0c7yzjm6jy3dxfyb8zd6x0fv1r6l1sb-hello\n");
        assert!(
            said.ends_with("its hash is 31 characters, not 32"),
            "{said}"
        );
    }

    #[test]
    fn a_hash_with_a_character_outside_nixs_base32_is_no_store_path() {
        for (c, path) in [
            ('e', "/nix/store/0c7yzjm6jy3dxfyb8zd6x0fv1r6l1sbe-hello"),
            ('u', "/nix/store/uc7yzjm6jy3dxfyb8zd6x0fv1r6l1sbm-hello"),
            ('A', "/nix/store/0c7yzjm6jy3dxfyb8zd6x0fv1r6l1sbA-hello"),
        ] {
            let said = refused_with(path);
            assert!(
                said.ends_with(&format!("its hash holds `{c}`, outside Nix's base32")),
                "{said}"
            );
        }
    }

    #[test]
    fn a_line_with_no_name_or_a_name_nix_refuses_is_no_store_path() {
        let hash = "/nix/store/0c7yzjm6jy3dxfyb8zd6x0fv1r6l1sbm";
        for (line, problem) in [
            (hash.to_string(), "no `-` follows its hash".to_string()),
            (format!("{hash}-"), "it names nothing after its hash".into()),
            (
                format!("{hash}-hello/bin/hello"),
                "its name holds `/`".into(),
            ),
            (format!("{hash}-hello "), "its name holds ` `".into()),
            (format!("{hash}-.."), "its name begins `..`".into()),
            (format!("{hash}-.-x"), "its name begins `.`".into()),
            (
                format!("{hash}-{}", "a".repeat(212)),
                "its name is 212 characters, more than 211".into(),
            ),
        ] {
            let said = refused_with(&line);
            assert!(said.ends_with(&problem), "{line}: {said}");
        }
        let dotted = format!("{hash}-.hello");
        assert_eq!(paths(&read(&dotted).unwrap()), [dotted.as_str()]);
    }

    #[test]
    fn a_line_that_is_not_text_is_no_store_path() {
        let bytes = b"/nix/store/\xff\n";
        let said = match roots(
            &artifact(FILE, "swancloud-srv", bytes),
            bytes,
            STORE_DIR,
            &source(),
        ) {
            Err(Not::Refused(r)) => r.said.unwrap(),
            other => panic!("{other:?}"),
        };
        assert!(said.ends_with("is no store path: it is not text"), "{said}");
    }

    #[test]
    fn the_store_directory_is_nix_store_unless_nix_store_dir_names_another() {
        assert_eq!(store_dir_of(None), STORE_DIR);
        assert_eq!(store_dir_of(Some(String::new())), STORE_DIR);
        assert_eq!(
            store_dir_of(Some("/srv/nix/store/".into())),
            "/srv/nix/store"
        );
        let path = "/srv/nix/store/0c7yzjm6jy3dxfyb8zd6x0fv1r6l1sbm-hello";
        let bytes = path.as_bytes();
        let file = artifact(FILE, "swancloud-srv", bytes);
        let found = roots(&file, bytes, "/srv/nix/store", &source()).unwrap();
        assert_eq!(paths(&found), [path]);
        assert!(matches!(
            roots(&file, ROOT.as_bytes(), "/srv/nix/store", &source()),
            Err(Not::Refused(_))
        ));
    }

    fn index(artifacts: Vec<Artifact>) -> IndexMeta {
        IndexMeta::new(
            ComponentName::new("swancloud-srv"),
            Version::new("0.4.0"),
            artifacts,
        )
        .unwrap()
    }

    #[test]
    fn the_file_artifact_of_an_output_is_found_in_the_index() {
        let closure = artifact(FILE, "swancloud-srv", ROOT.as_bytes());
        let other = artifact("eco", "swancloud-srv", b"other");
        let index = index(vec![other, closure.clone()]);
        let found = artifact_of(&index, &OutputName::new("swancloud-srv"), &source()).unwrap();
        assert_eq!(found, &closure);
    }

    #[test]
    fn an_output_the_index_lacks_is_refused_naming_it_and_the_version() {
        let index = index(vec![
            artifact("eco", "swancloud-srv", b"other"),
            artifact(FILE, "notes", b"notes"),
        ]);
        match artifact_of(&index, &OutputName::new("swancloud-srv"), &source()) {
            Err(Not::Refused(r)) => {
                assert_eq!(r.source, source());
                assert_eq!(r.why, Why::Said);
                assert_eq!(
                    r.said.as_deref(),
                    Some(
                        "version 0.4.0 of swancloud-srv names no file artifact of the output \
                         swancloud-srv, so it carries no closure of it"
                    )
                );
            }
            other => panic!("an output the index lacks answered {other:?}"),
        }
    }
}
