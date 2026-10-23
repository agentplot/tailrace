//! What the binary says of itself: the revision it was built from and the
//! build that made it, never a version (statement 24, C.11). The flake
//! gives the revision as TAILRACE_REVISION, since a nix build has no
//! `.git`; a cargo build in a checkout reads git's HEAD.

use std::process::Command;

fn main() {
    let revision = std::env::var("TAILRACE_REVISION")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(git_head)
        .unwrap_or_else(|| "unknown".to_string());
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let compiler = Command::new(&rustc)
        .arg("-V")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "rustc".to_string());
    let target = std::env::var("TARGET").unwrap_or_default();
    println!("cargo:rustc-env=TAILRACE_REVISION={revision}");
    println!("cargo:rustc-env=TAILRACE_BUILD={compiler} for {target}");
    println!("cargo:rerun-if-env-changed=TAILRACE_REVISION");
    if let Some(head) = git_path("HEAD") {
        println!("cargo:rerun-if-changed={head}");
    }
}

fn git_head() -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn git_path(name: &str) -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-path", name])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}
