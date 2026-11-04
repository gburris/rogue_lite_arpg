//! Starting, waiting on and cleaning up the external programs the commands drive.

use std::path::Path;
use std::process::{Child, Command};
use std::time::{Duration, Instant};
use std::{env, thread};

use anyhow::{Context, Result, bail, ensure};

const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// A running program that is killed if it is still running when dropped, so an error part way
/// through a sweep never leaves a capture or a game window behind.
pub(super) struct ChildProcess {
    name: String,
    child: Child,
}

impl ChildProcess {
    /// Starts `command`, naming it `name` in warnings and errors.
    pub(super) fn spawn(name: &str, command: &mut Command) -> Result<Self> {
        let child = command
            .spawn()
            .with_context(|| format!("could not start {name}"))?;

        Ok(Self {
            name: name.to_owned(),
            child,
        })
    }

    /// Waits for the program to exit, killing it with a warning once `timeout` passes.
    pub(super) fn wait_or_kill(mut self, timeout: Duration) -> Result<()> {
        let deadline = Instant::now() + timeout;

        while Instant::now() < deadline {
            if let Some(status) = self.child.try_wait()? {
                if !status.success() {
                    eprintln!("warning: {} exited with {status}", self.name);
                }
                return Ok(());
            }
            thread::sleep(POLL_INTERVAL);
        }

        eprintln!("warning: {} did not exit in time, stopping it", self.name);
        self.kill();
        Ok(())
    }

    fn kill(&mut self) {
        // Either call fails only when the program already exited, which is the goal anyway.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for ChildProcess {
    fn drop(&mut self) {
        let still_running = matches!(self.child.try_wait(), Ok(None));
        if still_running {
            self.kill();
        }
    }
}

/// Runs `command` to completion, failing when it can't start or exits unsuccessfully.
pub(super) fn run_to_completion(name: &str, command: &mut Command) -> Result<()> {
    let status = command
        .status()
        .with_context(|| format!("could not start {name}"))?;

    ensure!(status.success(), "{name} failed with {status}");
    Ok(())
}

/// Fails unless `program` is found on `PATH`.
pub(super) fn require_on_path(program: &str) -> Result<()> {
    if is_on_path(program) {
        Ok(())
    } else {
        bail!("{program} is not on PATH")
    }
}

fn is_on_path(program: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };
    let file_name = format!("{program}{}", env::consts::EXE_SUFFIX);

    env::split_paths(&path).any(|directory| is_file(&directory.join(&file_name)))
}

fn is_file(path: &Path) -> bool {
    path.metadata().is_ok_and(|metadata| metadata.is_file())
}
