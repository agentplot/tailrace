//! `tailrace push`: the step after a landing, run on the host that ran
//! its checks, that writes to the store the cache entry of every build
//! identity the landing's passing verdicts name, from what the landing's
//! build left on the host, so any host whose build identity matches
//! replays it (52, 78, C.3; design D1). The landing is asked first, and
//! a revision its branch has not landed is refused before anything else
//! is read (77). Each build identity is answered on its own: an entry
//! whose name the store holds already is answered held at the digest it
//! names, unmoved and with nothing retried (rule core.6), and one whose
//! archive is not on this host is absent and refuses nothing (54).
//!
//! The push offers nothing: it puts no manifest, attaches no claim,
//! signs nothing, asks no policy and moves no name but each entry's own
//! (53, 100). It keeps nothing either: which entries a landing became is
//! what its verdicts and the store answer, so no file of the host's
//! record is written and no other file is kept (61; design D2).

use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use tailrace_seams::{
    Answer, Branch, BuildId, Check, Outcome, Output, OutputName, Revision, Seams,
};

use crate::{answer_json, cache, Landed};

/// What a push is asked: the landed revision whose archives it writes,
/// and the branch that landed it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PushAsk {
    pub branch: Branch,
    pub revision: Revision,
}

/// 57: what a push answers, per build identity a passing verdict names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Push {
    pub branch: Branch,
    pub revision: Revision,
    /// Each build identity a passing verdict names, once, in the order
    /// the verdicts name them, with what its write answered.
    pub entries: Vec<(BuildId, Answer<cache::Written>)>,
}

impl Serialize for Push {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let entries: Vec<Value> = self
            .entries
            .iter()
            .map(|(build, answer)| {
                let mut value = answer_json(answer);
                value["build"] = json!(build);
                value
            })
            .collect();
        json!({
            "branch": self.branch,
            "revision": self.revision,
            "entries": entries,
        })
        .serialize(serializer)
    }
}

/// 52, C.3: the cache entry of each build identity `ask.revision`'s
/// passing verdicts name, written by [`cache::write`]. The landing's
/// verdicts name checks, not the outputs the policy file declares, so
/// each entry is packed as an output named for the first passing
/// verdict's check, carrying no files. A build identity named only by
/// verdicts that did not pass is not written: an entry carries no
/// outcome. One build identity's answer never stops the next.
pub fn push(seams: &Seams, ask: &PushAsk) -> Answer<Push> {
    let landed = Landed::check(seams.landing.as_ref(), &ask.branch, &ask.revision)?;
    let mut builds: Vec<(BuildId, Check)> = Vec::new();
    for verdict in seams.landing.verdicts_of(landed.revision())? {
        if verdict.outcome == Outcome::Pass && !builds.iter().any(|(b, _)| *b == verdict.build) {
            builds.push((verdict.build, verdict.check));
        }
    }
    let entries = builds
        .into_iter()
        .map(|(build, check)| {
            let output = Output {
                name: OutputName::new(check.as_str()),
                files: vec![],
            };
            let answer = cache::write(seams, &output, &build);
            (build, answer)
        })
        .collect();
    Ok(Push {
        branch: ask.branch.clone(),
        revision: ask.revision.clone(),
        entries,
    })
}
