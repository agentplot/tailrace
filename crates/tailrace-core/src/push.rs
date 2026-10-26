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
//! `tailrace push --left` is the push before a landing, run in the
//! checkout a check job's run left, as a pool's run job does on a runner
//! that keeps nothing: it asks the build what that checkout's run left
//! and writes the cache entry of each build identity that passed, asking
//! nothing of the landing, which has decided nothing yet (52, 135, 139).
//! Each build identity is answered on its own, as after a landing.
//!
//! Neither push offers anything: it puts no manifest, attaches no claim,
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

/// Each build identity's answer as a program reads it, the build
/// identity beside it.
fn entries_json(entries: &[(BuildId, Answer<cache::Written>)]) -> Vec<Value> {
    entries
        .iter()
        .map(|(build, answer)| {
            let mut value = answer_json(answer);
            value["build"] = json!(build);
            value
        })
        .collect()
}

impl Serialize for Push {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        json!({
            "branch": self.branch,
            "revision": self.revision,
            "entries": entries_json(&self.entries),
        })
        .serialize(serializer)
    }
}

/// What a push before a landing is asked: the checkout whose run it
/// writes, as the build is given it (139).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LeftAsk {
    pub at: String,
}

/// 57: what a push before a landing answers, per build identity that
/// passed of those the checkout's run left.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PushedLeft {
    pub at: String,
    /// Each build identity that passed, once, in the order the build
    /// names them, with what its write answered.
    pub entries: Vec<(BuildId, Answer<cache::Written>)>,
}

impl Serialize for PushedLeft {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        json!({
            "at": self.at,
            "entries": entries_json(&self.entries),
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
    for verdict in seams.landing.verdicts_of(landed.revision())?.verdicts {
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

/// 52, 135, 139: the cache entry of each build identity the checkout at
/// `ask.at` left with a pass, as the build answers what its run left,
/// written by [`cache::write`] once each in the order the build names
/// them, packed as an output named for its first task with the files the
/// build says are there. A build identity that did not pass is not
/// written. Nothing is asked of the landing, so it runs before the
/// landing is decided; one build identity's answer never stops the next.
pub fn push_left(seams: &Seams, ask: &LeftAsk) -> Answer<PushedLeft> {
    let mut entries: Vec<(BuildId, Answer<cache::Written>)> = Vec::new();
    for left in seams.build.left(&ask.at)? {
        if left.outcome != Outcome::Pass || entries.iter().any(|(b, _)| *b == left.build) {
            continue;
        }
        let task = left
            .tasks
            .first()
            .map_or(left.build.as_str(), String::as_str);
        let output = Output {
            name: OutputName::new(task),
            files: left.files,
        };
        let answer = cache::write(seams, &output, &left.build);
        entries.push((left.build, answer));
    }
    Ok(PushedLeft {
        at: ask.at.clone(),
        entries,
    })
}
