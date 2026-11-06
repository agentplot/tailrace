//! A fleet that answers as a test sets, in clan's, Nix's and the
//! machine's place, so the profile's refusals and the target group run in
//! `check-fast` (design D5). Each machine runs what its last update
//! switched it to, and its markers are held in memory, written as the
//! machine's shell writes them, its time the script's clock. Every
//! command asked is recorded in the order asked.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use timber_seams::{Answer, Not, Source, Why};

use super::fleet::Fleet;
use super::marker::{MarkerPath, Read};
use crate::profiles::target::nix_closure::StorePath;

/// What a machine runs before any update, as a test's machine does.
pub(super) const BEFORE: &str = "/nix/store/2f3c8vq4h0lp1d5m7r9s6w8x0y2z4b6n-nixos-system-before";

/// The time each mark writes.
pub(super) const CLOCK: &str = "2026-11-06T10:00:00Z";

/// A command the fleet was asked to run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Asked {
    Eval { flake: PathBuf, machine: String },
    Realise(String),
    Update { flake: PathBuf, machine: String },
    Read { flake: PathBuf, machine: String },
    Mark { flake: PathBuf, machine: String },
}

impl Asked {
    /// The command's name, as a test lists what ran.
    pub fn name(&self) -> &'static str {
        match self {
            Asked::Eval { .. } => "eval",
            Asked::Realise(_) => "realise",
            Asked::Update { .. } => "update",
            Asked::Read { .. } => "read",
            Asked::Mark { .. } => "mark",
        }
    }
}

#[derive(Default)]
struct State {
    toplevels: BTreeMap<String, String>,
    unsubstitutable: BTreeMap<String, String>,
    update_fails: Option<String>,
    switches_to: BTreeMap<String, String>,
    runs: BTreeMap<String, String>,
    unreachable: BTreeMap<String, String>,
    mark_fails: Option<String>,
    markers: BTreeMap<(String, String), Vec<u8>>,
    asked: Vec<Asked>,
}

/// The scripted fleet; a clone shares its state.
#[derive(Clone)]
pub(super) struct Scripted {
    state: Rc<RefCell<State>>,
    source: Source,
}

impl Scripted {
    pub fn new(source: &Source) -> Scripted {
        Scripted {
            state: Rc::default(),
            source: source.clone(),
        }
    }

    /// `machine`'s toplevel evaluates to `path` in every flake.
    pub fn evaluates(&self, machine: &str, path: &str) {
        let mut state = self.state.borrow_mut();
        state.toplevels.insert(machine.into(), path.into());
    }

    /// No substituter holds `path`, and Nix says so in `words`.
    pub fn lacks(&self, path: &str, words: &str) {
        let mut state = self.state.borrow_mut();
        state.unsubstitutable.insert(path.into(), words.into());
    }

    /// Every update fails, clan saying `words`.
    pub fn update_fails(&self, words: &str) {
        self.state.borrow_mut().update_fails = Some(words.into());
    }

    /// An update of `machine` leaves it running `path`, whatever it was
    /// asked to switch to.
    pub fn switches_to(&self, machine: &str, path: &str) {
        let mut state = self.state.borrow_mut();
        state.switches_to.insert(machine.into(), path.into());
    }

    /// `machine` runs `path` now, switched by hand.
    pub fn runs(&self, machine: &str, path: &str) {
        let mut state = self.state.borrow_mut();
        state.runs.insert(machine.into(), path.into());
    }

    /// `machine` cannot be reached, ssh saying `words`.
    pub fn unreachable(&self, machine: &str, words: &str) {
        let mut state = self.state.borrow_mut();
        state.unreachable.insert(machine.into(), words.into());
    }

    /// Every mark fails, the machine saying `words`.
    pub fn mark_fails(&self, words: &str) {
        self.state.borrow_mut().mark_fails = Some(words.into());
    }

    /// The marker at `marker` on `machine`, as its bytes, where one is
    /// held.
    pub fn marker(&self, machine: &str, marker: &MarkerPath) -> Option<Vec<u8>> {
        let state = self.state.borrow();
        state
            .markers
            .get(&(machine.to_string(), marker.to_string()))
            .cloned()
    }

    /// The marker at `marker` on `machine` set to `bytes`, as a hand or
    /// an earlier timber left it.
    pub fn holds(&self, machine: &str, marker: &MarkerPath, bytes: &[u8]) {
        let mut state = self.state.borrow_mut();
        state
            .markers
            .insert((machine.into(), marker.to_string()), bytes.to_vec());
    }

    /// Every command asked, in order.
    pub fn asked(&self) -> Vec<Asked> {
        self.state.borrow().asked.clone()
    }

    /// The names of every command asked, in order.
    pub fn ran(&self) -> Vec<&'static str> {
        self.state.borrow().asked.iter().map(Asked::name).collect()
    }

    fn ask(&self, asked: Asked) {
        self.state.borrow_mut().asked.push(asked);
    }

    /// ssh's words where `machine` cannot be reached.
    fn reach(&self, machine: &str, what: String) -> Answer<()> {
        match self.state.borrow().unreachable.get(machine) {
            Some(words) => Err(Not::unreadable(&self.source, what, Some(words.clone()))),
            None => Ok(()),
        }
    }
}

impl Fleet for Scripted {
    fn eval_toplevel(&self, flake: &Path, machine: &str) -> Answer<String> {
        self.ask(Asked::Eval {
            flake: flake.into(),
            machine: machine.into(),
        });
        let toplevel = self.state.borrow().toplevels.get(machine).cloned();
        toplevel.ok_or_else(|| {
            Not::unreadable(
                &self.source,
                format!(
                    "the toplevel of {machine} in the flake at {}",
                    flake.display()
                ),
                Some(format!(
                    "nixosConfigurations names no {machine}; darwinConfigurations names no \
                     {machine}"
                )),
            )
        })
    }

    fn realise(&self, root: &StorePath) -> Answer<()> {
        self.ask(Asked::Realise(root.to_string()));
        match self.state.borrow().unsubstitutable.get(root.as_str()) {
            Some(words) => Err(Not::refused(&self.source, Why::Said, Some(words.clone()))),
            None => Ok(()),
        }
    }

    fn update(&self, flake: &Path, machine: &str) -> Answer<()> {
        self.ask(Asked::Update {
            flake: flake.into(),
            machine: machine.into(),
        });
        let what = format!("clan machines update {machine}");
        self.reach(machine, what.clone())?;
        let mut state = self.state.borrow_mut();
        if let Some(words) = &state.update_fails {
            return Err(Not::unreadable(&self.source, what, Some(words.clone())));
        }
        let to = state
            .switches_to
            .get(machine)
            .or_else(|| state.toplevels.get(machine))
            .cloned()
            .unwrap_or_else(|| panic!("the test scripted no toplevel of {machine}"));
        state.runs.insert(machine.into(), to);
        Ok(())
    }

    fn read(&self, flake: &Path, machine: &str, marker: &MarkerPath) -> Answer<Read> {
        self.ask(Asked::Read {
            flake: flake.into(),
            machine: machine.into(),
        });
        self.reach(
            machine,
            format!("clan ssh {machine}: what it runs and the marker {marker}"),
        )?;
        let runs = self.state.borrow().runs.get(machine).cloned();
        Ok(Read {
            runs: runs.unwrap_or_else(|| BEFORE.into()),
            marker: self.marker(machine, marker),
        })
    }

    fn mark(&self, flake: &Path, machine: &str, marker: &MarkerPath, head: &str) -> Answer<()> {
        self.ask(Asked::Mark {
            flake: flake.into(),
            machine: machine.into(),
        });
        let what = format!("clan ssh {machine}: the write of the marker {marker}");
        self.reach(machine, what.clone())?;
        if let Some(words) = self.state.borrow().mark_fails.clone() {
            return Err(Not::unreadable(&self.source, what, Some(words)));
        }
        self.holds(
            machine,
            marker,
            format!("{head}\"at\":\"{CLOCK}\"}}\n").as_bytes(),
        );
        Ok(())
    }
}
