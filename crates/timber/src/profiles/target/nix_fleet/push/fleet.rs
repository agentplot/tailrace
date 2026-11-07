//! The push profile's commands against the fleet, Nix and the machine,
//! behind a trait of the profile's own, which is no seam: the target seam
//! stays the one interface (rule core.2; design D5). [`Clan`] runs
//! `clan`, `nix` and `nix-store` from the host's `PATH`, which on a
//! deployer is the fleet's own shell, so each is the version the fleet
//! pins; what each prints on a failure is the answer, verbatim (59). A
//! test's scripted fleet answers in their place.

use std::path::Path;

use timber_seams::{Answer, Not, Source, Why};

use super::marker::{marked, MarkerPath, Read};
use crate::profiles::run::{Ran, Tool, GIT_REPOSITORY_ENV};
use crate::profiles::target::nix_closure::StorePath;

/// The configurations a fleet's flake names its machines' systems
/// under, in the order the toplevel is looked for in them (design D3).
pub(super) const CONFIGURATIONS: [&str; 2] = ["nixosConfigurations", "darwinConfigurations"];

/// What the push profile asks of the fleet.
pub(super) trait Fleet {
    /// The store path `machine`'s toplevel evaluates to in the flake at
    /// `flake`, under the first of [`CONFIGURATIONS`] naming it.
    fn eval_toplevel(&self, flake: &Path, machine: &str) -> Answer<String>;

    /// `root` realised in this host's Nix store from its substituters
    /// alone, building nothing; refused in Nix's words where it cannot be
    /// (42, I3).
    fn realise(&self, root: &StorePath) -> Answer<()>;

    /// One `clan machines update` of `machine` from the flake at `flake`,
    /// with this host as the build host and host keys checked strictly
    /// (90, C.12).
    fn update(&self, flake: &Path, machine: &str) -> Answer<()>;

    /// What `machine` runs and its marker at `marker`, read in one run on
    /// it as the fleet at `flake` reaches it (89, 119).
    fn read(&self, flake: &Path, machine: &str, marker: &MarkerPath) -> Answer<Read>;

    /// The marker at `marker` on `machine` written whole: `head`, the
    /// marker's JSON up to its time, and the machine's time (119).
    fn mark(&self, flake: &Path, machine: &str, marker: &MarkerPath, head: &str) -> Answer<()>;
}

/// The fleet's own tools, run from `PATH`.
pub(super) struct Clan {
    clan: Tool,
    nix: Tool,
    nix_store: Tool,
    source: Source,
}

impl Clan {
    /// `clan`, `nix` and `nix-store` as found on `PATH`, answering as
    /// `source`.
    pub fn new(source: &Source) -> Clan {
        Clan {
            clan: Tool::new("clan", source.clone()),
            nix: Tool::new("nix", source.clone()),
            nix_store: Tool::new("nix-store", source.clone()),
            source: source.clone(),
        }
    }

    /// `nix eval` of `attribute` in the flake at `flake`, with `rest`.
    fn eval(&self, flake: &Path, attribute: &str, rest: &[&str]) -> Answer<Ran> {
        let installable = format!("{}#{attribute}", flake.display());
        let mut args = vec![
            "--extra-experimental-features",
            "nix-command flakes",
            "eval",
            installable.as_str(),
        ];
        args.extend(rest);
        self.nix.ran(None, &args, GIT_REPOSITORY_ENV)
    }

    /// `clan ssh <machine>` running `script` with `sh -c` on the machine,
    /// with host keys checked strictly. clan's ssh answers no exit of the
    /// command it ran, so the run answers what it printed whatever it
    /// exits with, and the script's own end line says it finished.
    fn ssh(&self, flake: &Path, machine: &str, script: &str) -> Answer<Ran> {
        let args = [
            "ssh",
            machine,
            "--flake",
            &flake.display().to_string(),
            "--host-key-check",
            "strict",
            "-c",
            "sh",
            "-c",
            script,
        ];
        self.clan.ran(None, &args, GIT_REPOSITORY_ENV)
    }

    /// Unreadable naming `what` on `machine`, carrying what `ran` printed
    /// on its stderr, trimmed, or its exit where it printed nothing.
    fn unfinished(&self, what: String, ran: &Ran) -> Not {
        let words = String::from_utf8_lossy(&ran.stderr).trim().to_string();
        Not::unreadable(
            &self.source,
            what,
            Some(if words.is_empty() {
                format!("clan ssh printed no answer, and {}", ran.status)
            } else {
                words
            }),
        )
    }
}

impl Fleet for Clan {
    fn eval_toplevel(&self, flake: &Path, machine: &str) -> Answer<String> {
        let mut said = Vec::new();
        for configurations in CONFIGURATIONS {
            let ran = self.eval(
                flake,
                configurations,
                &["--json", "--apply", "builtins.attrNames"],
            )?;
            if ran.code != Some(0) {
                let words = String::from_utf8_lossy(&ran.stderr).trim().to_string();
                said.push(format!("{configurations}: {words}"));
                continue;
            }
            let names: Vec<String> = serde_json::from_slice(&ran.stdout).map_err(|e| {
                Not::unreadable(
                    &self.source,
                    format!(
                        "the names of {configurations} in the flake at {}",
                        flake.display()
                    ),
                    Some(format!("nix eval printed no list of names: {e}")),
                )
            })?;
            if !names.iter().any(|name| name == machine) {
                said.push(format!("{configurations} names no {machine}"));
                continue;
            }
            let attribute =
                format!("{configurations}.{machine}.config.system.build.toplevel.outPath");
            let ran = self.eval(flake, &attribute, &["--raw"])?;
            if ran.code != Some(0) {
                return Err(self.nix.failed(&["eval", attribute.as_str()], &ran));
            }
            return Ok(String::from_utf8_lossy(&ran.stdout).trim().to_string());
        }
        Err(Not::unreadable(
            &self.source,
            format!(
                "the toplevel of {machine} in the flake at {}",
                flake.display()
            ),
            Some(said.join("; ")),
        ))
    }

    fn realise(&self, root: &StorePath) -> Answer<()> {
        let args = ["--realise", root.as_str(), "--option", "max-jobs", "0"];
        let ran = self.nix_store.ran(None, &args, GIT_REPOSITORY_ENV)?;
        if ran.code == Some(0) {
            return Ok(());
        }
        let words = String::from_utf8_lossy(&ran.stderr).trim().to_string();
        Err(Not::refused(
            &self.source,
            Why::Said,
            Some(if words.is_empty() {
                format!("nix-store --realise {root} with no build, {}", ran.status)
            } else {
                words
            }),
        ))
    }

    fn update(&self, flake: &Path, machine: &str) -> Answer<()> {
        let flake = flake.display().to_string();
        let args = [
            "machines",
            "update",
            machine,
            "--flake",
            &flake,
            "--build-host",
            "localhost",
            "--host-key-check",
            "strict",
        ];
        self.clan
            .output(None, &args, GIT_REPOSITORY_ENV, &[0])
            .map(drop)
    }

    fn read(&self, flake: &Path, machine: &str, marker: &MarkerPath) -> Answer<Read> {
        let ran = self.ssh(flake, machine, &marker.read_script())?;
        Read::parse(&ran.stdout).ok_or_else(|| {
            self.unfinished(
                format!("clan ssh {machine}: what it runs and the marker {marker}"),
                &ran,
            )
        })
    }

    fn mark(&self, flake: &Path, machine: &str, marker: &MarkerPath, head: &str) -> Answer<()> {
        let ran = self.ssh(flake, machine, &marker.mark_script(head))?;
        if marked(&ran.stdout) {
            return Ok(());
        }
        Err(self.unfinished(
            format!("clan ssh {machine}: the write of the marker {marker}"),
            &ran,
        ))
    }
}
