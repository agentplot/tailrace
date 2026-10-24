//! One group per seam of Part B, each over a rig a caller supplies, and
//! the binding's (design D2, D3).

pub mod binding;
pub mod build;
pub mod keeper;
pub mod landing;
pub mod policy;
pub mod projection;
pub mod signer;
pub mod store;
pub mod target;

use std::fmt::Debug;

use tailrace_seams::{Answer, Not, Seam};

/// That `asked` answered unreadable, from `seam`, carrying `words`
/// unchanged (58, 59).
pub(crate) fn unreadable<T: Debug>(answer: Answer<T>, seam: Seam, words: &str, asked: &str) {
    match answer {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source.seam, seam, "{asked} answered {u:?}");
            assert_eq!(
                u.said.as_deref(),
                Some(words),
                "{asked} did not carry the {seam}'s words unchanged: {u:?}"
            );
        }
        other => panic!("{asked} of a {seam} that cannot be reached answered {other:?}"),
    }
}

/// That `asked` answered absent, from `seam`, and named `what`.
pub(crate) fn absent<T: Debug>(answer: Answer<T>, seam: Seam, what: &str, asked: &str) {
    match answer {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source.seam, seam, "{asked} answered {a:?}");
            assert!(
                a.what.contains(what),
                "{asked} answered {a:?}, naming no {what}"
            );
        }
        other => panic!("{asked} answered {other:?}, not absent"),
    }
}
