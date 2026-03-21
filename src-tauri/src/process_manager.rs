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

/// Stop an instance by killing the process and its direct children.
pub fn stop_instance(instance_id: &str) -> Result<(), String> {
    let pid = {
        let map = get_map().lock().unwrap();
        map.get(instance_id).copied()
    };

    match pid {
        Some(pid) => {
            let sysinfo_pid = sysinfo::Pid::from_u32(pid);

            // Refresh all processes so we can find children.
            // We use `everything()` here because we need parent info + exe info
            // to properly walk the tree — but this only happens on explicit stop,
            // not in hot-path polling.
            let mut sys = get_system().lock().unwrap();
            sys.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                ProcessRefreshKind::nothing(),
            );

            // Kill child processes first (content processes, GPU process, etc.)
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

            // Kill the main browser process
            if let Some(process) = sys.process(sysinfo_pid) {
                process.kill();
                drop(sys);
                // The background watcher task will detect the exit and unregister
                Ok(())
            } else {
                drop(sys);
                // Process already dead, clean up
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
        true, // remove from cache if dead so subsequent checks are accurate
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

    for (_pid, process) in sys.processes() {
        // Only consider camoufox/firefox processes.
        let exe_name = process
            .exe()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !exe_name.contains("camoufox") && !exe_name.contains("firefox") {
            continue;
        }

        let cmd = process.cmd();
        for (i, arg) in cmd.iter().enumerate() {
            let arg_lower = arg.to_string_lossy().to_lowercase().replace('/', "\\");

            // Match `--profile <dir>` as two separate arguments.
            if arg_lower == "--profile" {
                if let Some(next) = cmd.get(i + 1) {
                    let next_lower = next.to_string_lossy().to_lowercase().replace('/', "\\");
                    if next_lower == profile_str {
                        return Some(process.pid().as_u32());
                    }
                }
            }

            // Also catch cases where the profile path appears directly in an arg.
            if arg_lower.contains(&profile_str) {
                return Some(process.pid().as_u32());
            }
        }
    }

    None
}
