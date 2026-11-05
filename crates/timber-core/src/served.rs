//! The one address a host's serving process answers at, and the name a
//! repository takes in its path (49, 114, rule core.36; design D4): the
//! core's own name and port, which no setting, flag or variable moves,
//! and the owner and name the last two parts of the path of the
//! repository's landing's remote, whatever host holds it, so the address
//! a client records names no host and no one owner's store. A served
//! layout is reached at that address with the repository and the
//! ecosystem in the path, the same on every host and in every checkout
//! (rule core.18).

use timber_seams::Ecosystem;

/// The name the host's serving process answers at.
pub const SERVING_HOST: &str = "127.0.0.1";

/// The port the host's serving process answers at.
pub const SERVING_PORT: u16 = 7420;

/// 114: the host's serving process's address, `http://<name>:<port>/`.
pub fn origin() -> String {
    format!("http://{SERVING_HOST}:{SERVING_PORT}/")
}

/// 114, 49: where the layout of `ecosystem` of `repository` is served,
/// `http://<name>:<port>/<owner>/<name>/<ecosystem>/`, naming no
/// directory and no store.
pub fn layout_address(repository: &RepositoryName, ecosystem: &Ecosystem) -> String {
    format!(
        "{}{}/{}/{ecosystem}/",
        origin(),
        repository.owner,
        repository.name
    )
}

/// 114: a repository as the serving process's path names it, defined in
/// the seams since an index names its publishing repository too (138;
/// design D4).
pub use timber_seams::RepositoryName;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_address_is_the_cores_own() {
        assert_eq!((SERVING_HOST, SERVING_PORT), ("127.0.0.1", 7420));
    }

    #[test]
    fn the_origin_is_the_cores_name_and_port() {
        assert_eq!(origin(), "http://127.0.0.1:7420/");
    }

    #[test]
    fn two_remotes_of_one_repository_serve_its_layout_at_one_address() {
        let npm = Ecosystem::new("npm");
        for url in [
            "https://example.com/agentplot/consumer.git",
            "git@example.com:agentplot/consumer",
        ] {
            let repository = RepositoryName::from_remote(url).unwrap();
            assert_eq!(
                layout_address(&repository, &npm),
                "http://127.0.0.1:7420/agentplot/consumer/npm/",
                "{url}"
            );
        }
    }
}
