//! The build and machine a stress run used, so summary lines from runs weeks apart can be told
//! apart.

use std::fmt::Write as _;
use std::process::Command;

use sysinfo::{CpuRefreshKind, RefreshKind, System};

/// The commit, uncommitted changes, Cargo profile and CPU of the running game. Git fields describe
/// the working directory's checkout when the run ends, and a field is `None` when it can't be
/// read, such as when `git` isn't on `PATH`.
pub(super) struct BuildInfo {
    commit: Option<String>,
    dirty: Option<bool>,
    profile: &'static str,
    cpu: Option<String>,
}

impl BuildInfo {
    pub(super) fn read() -> Self {
        Self {
            commit: git_output(&["rev-parse", "--short", "HEAD"]),
            dirty: git_output(&["status", "--porcelain"]).map(|changes| !changes.is_empty()),
            profile: profile_name(),
            cpu: cpu_name(),
        }
    }

    /// Appends ` git=… dirty=… profile=… cpu=…`, with `-` for a value that couldn't be read.
    pub(super) fn write_fields(&self, summary: &mut String) {
        write!(
            summary,
            " git={} dirty={} profile={} cpu={}",
            self.commit.as_deref().unwrap_or("-"),
            self.dirty
                .map_or("-", |dirty| if dirty { "true" } else { "false" }),
            self.profile,
            self.cpu.as_deref().unwrap_or("-"),
        )
        .expect("writing to a String cannot fail");
    }
}

/// Trimmed standard output of a successful `git` command in the working directory.
fn git_output(arguments: &[&str]) -> Option<String> {
    let output = Command::new("git").args(arguments).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// The Cargo profile the game was built with, as far as features and assertions tell.
fn profile_name() -> &'static str {
    if cfg!(feature = "profile") && !cfg!(debug_assertions) {
        "profiling"
    } else if cfg!(debug_assertions) {
        "dev"
    } else {
        "release"
    }
}

/// The CPU's brand name, with spaces as `_` so the summary stays one `key=value` per word.
fn cpu_name() -> Option<String> {
    let system =
        System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing()));
    let brand = system.cpus().first()?.brand().trim();
    if brand.is_empty() {
        return None;
    }
    Some(brand.split_whitespace().collect::<Vec<_>>().join("_"))
}
