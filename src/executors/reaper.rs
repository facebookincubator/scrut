/// Find all child processes (and their children) below a given process ID
use anyhow::Context;
use anyhow::Result;
use sysinfo::AsU32;
use sysinfo::Pid;
use sysinfo::ProcessExt;
use sysinfo::RefreshKind;
use sysinfo::Signal;
use sysinfo::System;
use sysinfo::SystemExt;
use tracing::warn;

/// Reads all *currently* running processes and kills all whose' parent PID
/// is the provided one (and all their children etc).
///
/// NOTE: there are race conditions, this is not fully safe on systems with
/// high frequency of changing processes. Read on: https://github.com/oconnor663/duct.py/blob/master/gotchas.md#killing-grandchild-processes
///
/// NOTE: Depending on the amount of running processes (and the OS), refreshing
/// all running processes can take multiple seconds!
pub(super) fn reap(parent_pid: u32) -> Result<()> {
    let mut sys = System::new_with_specifics(RefreshKind::new());
    sys.refresh_processes();
    reap_child_processes(&sys, parent_pid)
}

/// Kill all child processes (and their children) of a given parental process ID
fn reap_child_processes(sys: &System, parent_pid: u32) -> Result<()> {
    get_child_processes(sys, parent_pid)
        .context("get child processes for reaping")?
        .iter()
        .for_each(|pid| {
            if let Some(child) = sys.process(*pid) {
                if !child.kill(Signal::Kill) {
                    warn!(
                        "failed to kill child process {} ({}) of {}",
                        *pid,
                        child.name(),
                        parent_pid
                    );
                }
            }
        });
    Ok(())
}

fn get_child_processes(sys: &System, parent_pid: u32) -> Result<Vec<Pid>> {
    let mut processes: Vec<Pid> = vec![];
    for (pid, process) in sys.processes() {
        if let Some(parent) = process.parent() {
            if parent.as_u32() != parent_pid {
                continue;
            }
            processes.push(*pid);
            let children = get_child_processes(sys, pid.as_u32())
                .with_context(|| format!("get sub-child process for {} -> {}", parent_pid, *pid))?;
            processes.extend(children);
        }
    }
    Ok(processes)
}
