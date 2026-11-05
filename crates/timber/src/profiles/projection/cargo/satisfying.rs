//! satisfying (140, 23; design D8): which versions satisfy a range, and
//! which of them is newest, by Cargo's own rules: the range read by the
//! `semver` crate's `VersionReq`, which Cargo parses requirements with,
//! so a bare `1` reads as `^1` and a prerelease is left out unless a
//! comparator names one of the same version, and versions ordered by
//! `semver`'s order, Cargo's own.

use timber_seams::{Answer, Not, Range, Version, Why};

use super::CargoProjection;

/// Those of `versions` that satisfy `range`, newest first; a version
/// that is no version Cargo reads satisfying none. Refused naming the
/// range with the parser's words where Cargo reads no requirement in it.
pub(super) fn satisfying(range: &Range, versions: &[Version]) -> Answer<Vec<Version>> {
    let req = semver::VersionReq::parse(range.as_str()).map_err(|e| {
        Not::refused(
            &CargoProjection::source(),
            Why::Said,
            Some(format!(
                "the range {range} is no requirement Cargo reads: {e}"
            )),
        )
    })?;
    let mut kept: Vec<(semver::Version, &Version)> = versions
        .iter()
        .filter_map(|v| semver::Version::parse(v.as_str()).ok().map(|p| (p, v)))
        .filter(|(parsed, _)| req.matches(parsed))
        .collect();
    kept.sort_by(|(a, _), (b, _)| b.cmp(a));
    Ok(kept.into_iter().map(|(_, v)| v.clone()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn versions(texts: &[&str]) -> Vec<Version> {
        texts.iter().map(|v| Version::new(*v)).collect()
    }

    // Requirement: Whether a version satisfies a range, by Cargo's rules

    #[test]
    fn the_versions_a_range_admits_are_cargos_newest_first() {
        assert_eq!(
            satisfying(
                &Range::new("1"),
                &versions(&["1.2.0", "2.0.0", "1.5.0-rc.1", "1.4.0", "latest"])
            ),
            Ok(versions(&["1.4.0", "1.2.0"]))
        );
    }

    #[test]
    fn the_newer_of_two_versions_is_cargos() {
        assert_eq!(
            satisfying(&Range::new("1"), &versions(&["1.9.0", "1.10.0"])),
            Ok(versions(&["1.10.0", "1.9.0"]))
        );
        assert_eq!(
            satisfying(
                &Range::new(">=1.0.0-rc.1"),
                &versions(&["1.0.0-rc.1", "1.0.0"])
            ),
            Ok(versions(&["1.0.0", "1.0.0-rc.1"]))
        );
    }

    #[test]
    fn a_range_cargo_cannot_read_is_refused_naming_it_with_the_parsers_words() {
        let words = semver::VersionReq::parse("not-a-range")
            .unwrap_err()
            .to_string();
        match satisfying(&Range::new("not-a-range"), &versions(&["1.0.0"])) {
            Err(Not::Refused(r)) => {
                assert_eq!(r.source, CargoProjection::source());
                assert_eq!(
                    r.said,
                    Some(format!(
                        "the range not-a-range is no requirement Cargo reads: {words}"
                    ))
                );
            }
            other => panic!("answered {other:?}"),
        }
    }
}
