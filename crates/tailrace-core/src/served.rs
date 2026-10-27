//! The one address a host's serving process answers at, and the name a
//! repository takes in its path (49, 114, rule core.36; design D4): the
//! core's own name and port, which no setting, flag or variable moves,
//! and the owner and name the last two parts of the path of the
//! repository's landing's remote, whatever host holds it, so the address
//! a client records names no host and no one owner's store.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The name the host's serving process answers at.
pub const SERVING_HOST: &str = "127.0.0.1";

/// The port the host's serving process answers at.
pub const SERVING_PORT: u16 = 7420;

/// 114: a repository as the serving process's path names it, its owner
/// and its name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RepositoryName {
    pub owner: String,
    pub name: String,
}

impl RepositoryName {
    /// The owner and name of the remote at `url`, the last two parts of
    /// its path without a trailing `/` or `.git`: of
    /// `scheme://[user@]host[:port]/<path>`, or of `[user@]host:<path>`;
    /// `None` for a local path, a path of fewer than two parts, or one
    /// whose last two parts are not both named.
    pub fn from_remote(url: &str) -> Option<RepositoryName> {
        let path = match url.split_once("://") {
            Some((_, rest)) => {
                let (host, path) = rest.split_once('/')?;
                if host.is_empty() {
                    return None;
                }
                path
            }
            None => {
                let (host, path) = url.split_once(':')?;
                if host.is_empty() || host.contains('/') {
                    return None;
                }
                path
            }
        };
        let path = path.strip_suffix('/').unwrap_or(path);
        let path = path.strip_suffix(".git").unwrap_or(path);
        let mut parts = path.rsplit('/');
        let name = parts.next()?;
        let owner = parts.next()?;
        if owner.is_empty() || name.is_empty() {
            return None;
        }
        Some(RepositoryName {
            owner: owner.to_string(),
            name: name.to_string(),
        })
    }

    /// The repository's path on the serving process, `/owner/name`.
    pub fn path(&self) -> String {
        format!("/{}/{}", self.owner, self.name)
    }
}

impl fmt::Display for RepositoryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(url: &str) -> Option<String> {
        RepositoryName::from_remote(url).map(|r| r.to_string())
    }

    #[test]
    fn a_remote_of_every_form_names_its_last_two_parts() {
        for url in [
            "https://example.com/agentplot/tailrace.git",
            "ssh://git@example.com:22/agentplot/tailrace",
            "git@example.com:agentplot/tailrace",
            "https://example.com/group/agentplot/tailrace/",
        ] {
            assert_eq!(named(url).as_deref(), Some("agentplot/tailrace"), "{url}");
        }
    }

    #[test]
    fn a_local_path_or_a_path_of_one_part_names_none() {
        for url in [
            "/srv/tailrace.git",
            "file:///srv/tailrace.git",
            "https://example.com/tailrace",
            "https://example.com//tailrace",
        ] {
            assert_eq!(named(url), None, "{url}");
        }
    }

    #[test]
    fn the_path_is_the_owner_then_the_name() {
        let name = RepositoryName::from_remote("https://example.com/o/r.git").unwrap();
        assert_eq!(name.path(), "/o/r");
        assert_eq!((SERVING_HOST, SERVING_PORT), ("127.0.0.1", 7420));
    }
}
