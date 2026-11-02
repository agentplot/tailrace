//! The one address a host's serving process answers at, and the name a
//! repository takes in its path (49, 114, rule core.36; design D4): the
//! core's own name and port, which no setting, flag or variable moves,
//! and the owner and name the last two parts of the path of the
//! repository's landing's remote, whatever host holds it, so the address
//! a client records names no host and no one owner's store.

/// The name the host's serving process answers at.
pub const SERVING_HOST: &str = "127.0.0.1";

/// The port the host's serving process answers at.
pub const SERVING_PORT: u16 = 7420;

/// 114: a repository as the serving process's path names it, defined in
/// the seams since an index names its publishing repository too (138;
/// design D4).
pub use tailrace_seams::RepositoryName;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_address_is_the_cores_own() {
        assert_eq!((SERVING_HOST, SERVING_PORT), ("127.0.0.1", 7420));
    }
}
