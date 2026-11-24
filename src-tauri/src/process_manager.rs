use std::collections::HashMap;
use std::sync::Mutex;

use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

// ── Global state ────────────────────────────────────────────────────────────
// We keep two separate mutexes so that the (cheap) map lookup never has to
// contend with the (expensive) sysinfo refresh, and vice-versa.

/// Process registry: instance ID → PID.
static PROCESS_MAP: std::sync::OnceLock<Mutex<HashMap<String, u32>>> = std::sync::OnceLock::new();

/// Cached `System` instance – reused across calls to avoid the cost of
/// re-enumerating every process on the machine each time.
static SYSTEM: std::sync::OnceLock<Mutex<System>> = std::sync::OnceLock::new();

fn get_map() -> &'static Mutex<HashMap<String, u32>> {
    PROCESS_MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_system() -> &'static Mutex<System> {
    SYSTEM.get_or_init(|| Mutex::new(System::new()))
}

// ── Minimal ProcessRefreshKind ──────────────────────────────────────────────
// We only need to know whether a PID exists – no CPU, memory, cmd, etc.
fn minimal_refresh_kind() -> ProcessRefreshKind {
    ProcessRefreshKind::nothing()
}

// ── Registry operations ─────────────────────────────────────────────────────

/// Register a running instance with its PID.
pub fn register(instance_id: &str, pid: u32) {
    let mut map = get_map().lock().unwrap();
    map.insert(instance_id.to_string(), pid);
}

/// Remove an instance from the registry.
pub fn unregister(instance_id: &str) {
    let mut map = get_map().lock().unwrap();
    map.remove(instance_id);
}

/// Update the stored PID for an instance (e.g. after discovering the real
/// browser PID once the Windows launcher stub has exited).
pub fn update_pid(instance_id: &str, new_pid: u32) {
    let mut map = get_map().lock().unwrap();
    if map.contains_key(instance_id) {
        map.insert(instance_id.to_string(), new_pid);
    }
}

// ── Queries ─────────────────────────────────────────────────────────────────

/// Check whether a specific instance is registered **and** its PID is alive.
///
/// The map lock is released *before* the sysinfo check so we never hold the
/// cheap lock while doing an expensive OS call.
pub fn is_running(instance_id: &str) -> bool {
    let pid = {
        let map = get_map().lock().unwrap();
        map.get(instance_id).copied()
    };
    match pid {
        Some(pid) => is_pid_alive(pid),
        None => false,
    }
}

/// Return all currently-running instance IDs (validated against sysinfo).
///
/// Takes a snapshot of the map, drops the lock, bulk-refreshes only the PIDs
/// we care about, then prunes any stale entries.
pub fn get_running_ids() -> Vec<String> {
    // 1. Snapshot the map (cheap – drop lock immediately).
    let snapshot: Vec<(String, u32)> = {
        let map = get_map().lock().unwrap();
        map.iter().map(|(id, &pid)| (id.clone(), pid)).collect()
    };

    if snapshot.is_empty() {
        return Vec::new();
    }

    // 2. Bulk-refresh only the PIDs we care about.
    let pids: Vec<sysinfo::Pid> = snapshot
        .iter()
        .map(|(_, pid)| sysinfo::Pid::from_u32(*pid))
        .collect();

    let mut sys = get_system().lock().unwrap();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&pids),
        false,
        minimal_refresh_kind(),
    );

    // 3. Classify alive / dead.
    let mut alive = Vec::new();
    let mut dead = Vec::new();
    for (id, pid) in &snapshot {
        if sys.process(sysinfo::Pid::from_u32(*pid)).is_some() {
            alive.push(id.clone());
        } else {
            dead.push(id.clone());
        }
    }
    drop(sys); // release System lock before touching the map lock

    // 4. Prune stale entries.
    if !dead.is_empty() {
        let mut map = get_map().lock().unwrap();
        for id in &dead {
            map.remove(id);
        }
    }

    alive
}

/// Return a snapshot of running instances as (id, pid) pairs.
#[derive(Serialize, Clone, Debug)]
pub struct RunningInstance {
    pub id: String,
    pub pid: u32,
}

pub fn get_running_instances() -> Vec<RunningInstance> {
    // 1. Snapshot the map.
    let snapshot: Vec<(String, u32)> = {
        let map = get_map().lock().unwrap();
        map.iter().map(|(id, &pid)| (id.clone(), pid)).collect()
    };

    if snapshot.is_empty() {
        return Vec::new();
    }

    // 2. Bulk-refresh only the PIDs we care about.
    let pids: Vec<sysinfo::Pid> = snapshot
        .iter()
        .map(|(_, pid)| sysinfo::Pid::from_u32(*pid))
        .collect();

    let mut sys = get_system().lock().unwrap();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&pids),
        false,
        minimal_refresh_kind(),
    );

    // 3. Classify alive / dead.
    let mut alive = Vec::new();
    let mut dead = Vec::new();
    for (id, pid) in &snapshot {
        if sys.process(sysinfo::Pid::from_u32(*pid)).is_some() {
            alive.push(RunningInstance {
                id: id.clone(),
                pid: *pid,
            });
        } else {
            dead.push(id.clone());
        }
    }
    drop(sys);

    // 4. Prune stale entries.
    if !dead.is_empty() {
        let mut map = get_map().lock().unwrap();
        for id in &dead {
            map.remove(id);
        }
    }

    alive
}

fn request_graceful_close(pid: u32) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let _ = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string()])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .output();
    }
}

/// Request graceful browser shutdown so SQLite/session files flush cleanly,
/// escalating to force-kill only if the process does not exit in time.
pub fn stop_instance(instance_id: &str) -> Result<(), String> {
    let pid = {
        let map = get_map().lock().unwrap();
        map.get(instance_id).copied()
    };

    match pid {
        Some(pid) => {
            if !is_pid_alive(pid) {
                unregister(instance_id);
                crate::socks5_relay::stop_for_instance(instance_id);
                return Ok(());
            }

            request_graceful_close(pid);
            for _ in 0..25 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if !is_pid_alive(pid) {
                    return Ok(());
                }
            }

            let sysinfo_pid = sysinfo::Pid::from_u32(pid);
            let mut sys = get_system().lock().unwrap();
            sys.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                ProcessRefreshKind::nothing(),
            );

            let children: Vec<sysinfo::Pid> = sys
                .processes()
                .iter()
                .filter_map(|(child_pid, process)| {
                    if process.parent() == Some(sysinfo_pid) {
                        Some(*child_pid)
                    } else {
                        None
                    }
                })
                .collect();

            for child_pid in &children {
                if let Some(process) = sys.process(*child_pid) {
                    process.kill();
                }
            }

            if let Some(process) = sys.process(sysinfo_pid) {
                process.kill();
                drop(sys);
                Ok(())
            } else {
                drop(sys);
                unregister(instance_id);
                Ok(())
            }
        }
        None => Err(format!("Instance '{}' is not running", instance_id)),
    }
}

/// Check if any instances are currently running.
pub fn has_running_instances() -> bool {
    !get_running_ids().is_empty()
}

/// Check if a specific PID is alive.
///
/// Uses the cached System and refreshes only the single PID in question.
pub fn is_pid_alive(pid: u32) -> bool {
    let sysinfo_pid = sysinfo::Pid::from_u32(pid);
    let mut sys = get_system().lock().unwrap();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[sysinfo_pid]),
        // remove from cache if dead so subsequent checks are accurate
        true,
        minimal_refresh_kind(),
    );
    sys.process(sysinfo_pid).is_some()
}

/// Scan all running processes to find the real browser process that was
/// launched with `--profile <instance_dir>`.
///
/// On Windows, camoufox uses Firefox's launcher-process pattern: the initial
/// process is a short-lived stub that spawns the real browser and then exits.
/// After the stub exits we discover the real browser PID by matching its
/// command-line arguments against the profile directory path.
///
/// Returns `Some(pid)` if a matching process is found, `None` otherwise.
pub fn find_browser_pid(profile_dir: &std::path::Path) -> Option<u32> {
    // Normalise for case-insensitive, slash-agnostic comparison on Windows.
    let profile_str = profile_dir
        .to_string_lossy()
        .to_lowercase()
        .replace('/', "\\");

    let mut sys = get_system().lock().unwrap();
    // We need exe path + command-line args.
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_cmd(sysinfo::UpdateKind::Always)
            .with_exe(sysinfo::UpdateKind::Always),
    );

    for process in sys.processes().values() {
        let exe_name = process
            .exe()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        let cmd: Vec<String> = process
            .cmd()
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();

        if is_main_browser_cmdline(&exe_name, &cmd, &profile_str) {
            return Some(process.pid().as_u32());
        }
    }

    None
}

pub(crate) fn is_main_browser_cmdline(exe_name: &str, cmd: &[String], profile_str: &str) -> bool {
    let exe_lower = exe_name.to_lowercase();
    if (!exe_lower.contains("camoufox") && !exe_lower.contains("firefox"))
        || exe_lower.contains("plugin-container")
        || exe_lower.contains("crashhelper")
        || exe_lower.contains("nmhproxy")
    {
        return false;
    }

    let lowered: Vec<String> = cmd
        .iter()
        .map(|a| a.to_lowercase().replace('/', "\\"))
        .collect();

    if lowered.iter().any(|a| a == "-contentproc") {
        return false;
    }

    for (i, arg) in lowered.iter().enumerate() {
        if arg == "--profile" || arg == "-profile" {
            if let Some(next) = lowered.get(i + 1) {
                if next == profile_str {
                    return true;
                }
            }
        }
        if arg.contains(profile_str) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_firefox_content_child_processes_when_matching_profile_pid() {
        let profile = r"c:\users\dev\appdata\roaming\anon\profiles\abc";
        let main_cmd = vec![
            "camoufox.exe".into(),
            "--no-remote".into(),
            "--profile".into(),
            profile.into(),
        ];
        let child_cmd = vec![
            "camoufox.exe".into(),
            "-contentproc".into(),
            "-parentBuildID".into(),
            "20261122".into(),
            "-prefsHandle".into(),
            "123".into(),
            profile.into(),
        ];
        assert!(is_main_browser_cmdline("camoufox.exe", &main_cmd, profile));
        assert!(!is_main_browser_cmdline(
            "camoufox.exe",
            &child_cmd,
            profile
        ));
        assert!(!is_main_browser_cmdline(
            "plugin-container.exe",
            &main_cmd,
            profile
        ));
    }
}
