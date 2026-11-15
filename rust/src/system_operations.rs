//! Processes, services, disks, network interfaces, cron and user sessions.

use serde::{Deserialize, Serialize};

use crate::wire::{is_default, null_default};

/// --- Platform Enrichment Data Structs ---
/// ProcessListData is sent from hub to agent to request a process list.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProcessListData {
    pub request_id: String,
    /// "cpu" or "memory", default "cpu"
    #[serde(skip_serializing_if = "is_default")]
    pub sort_by: String,
    /// default 25
    #[serde(skip_serializing_if = "is_default")]
    pub limit: i64,
}

/// ProcessListedData is sent from agent to hub with the process list result.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProcessListedData {
    pub request_id: String,
    #[serde(deserialize_with = "null_default")]
    pub processes: Vec<ProcessInfo>,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// ProcessInfo describes a single running process.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProcessInfo {
    pub pid: i64,
    pub name: String,
    pub user: String,
    pub cpu_pct: f64,
    pub mem_pct: f64,
    pub mem_rss: i64,
    pub command: String,
}

/// ProcessKillData is sent from hub to agent to signal a process.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProcessKillData {
    pub pid: i64,
    /// "SIGTERM" | "SIGKILL" | "SIGINT" | "SIGHUP"; empty defaults to SIGTERM
    pub signal: String,
}

/// ProcessKillResultData is sent from agent to hub with the signal delivery result.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProcessKillResultData {
    pub pid: i64,
    pub success: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// ServiceListData is sent from hub to agent to request a service list.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ServiceListData {
    pub request_id: String,
}

/// ServiceListedData is sent from agent to hub with the service list result.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ServiceListedData {
    pub request_id: String,
    #[serde(deserialize_with = "null_default")]
    pub services: Vec<ServiceInfo>,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// ServiceInfo describes a single systemd service unit.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ServiceInfo {
    pub name: String,
    pub description: String,
    /// active, inactive, failed, activating, deactivating
    pub active_state: String,
    /// running, dead, exited, etc.
    pub sub_state: String,
    /// enabled, disabled, static, masked
    pub enabled: String,
    /// loaded, not-found, masked
    pub load_state: String,
}

/// ServiceActionData is sent from hub to agent to perform a service action.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ServiceActionData {
    pub request_id: String,
    pub service: String,
    /// start, stop, restart, enable, disable
    pub action: String,
}

/// ServiceResultData is sent from agent to hub with the service action result.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ServiceResultData {
    pub request_id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub output: String,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// DiskListData is sent from hub to agent to request mount/disk info.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DiskListData {
    pub request_id: String,
}

/// DiskListedData is sent from agent to hub with mount/disk info results.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DiskListedData {
    pub request_id: String,
    #[serde(deserialize_with = "null_default")]
    pub mounts: Vec<MountInfo>,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// MountInfo describes a single mounted filesystem.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MountInfo {
    pub device: String,
    pub mount_point: String,
    pub fs_type: String,
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub use_pct: f64,
}

/// NetworkListData is sent from hub to agent to request network interface info.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkListData {
    pub request_id: String,
}

/// NetworkListedData is sent from agent to hub with network interface results.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkListedData {
    pub request_id: String,
    #[serde(deserialize_with = "null_default")]
    pub interfaces: Vec<NetInterface>,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// NetInterface describes a single network interface and its counters.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetInterface {
    pub name: String,
    pub state: String,
    pub mac: String,
    pub mtu: i64,
    #[serde(deserialize_with = "null_default")]
    pub ips: Vec<String>,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
}

/// NetworkActionData is sent from hub to agent to perform a network action.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkActionData {
    pub request_id: String,
    /// apply|rollback
    pub action: String,
    #[serde(skip_serializing_if = "is_default")]
    pub method: String,
    /// nmcli connection name
    #[serde(skip_serializing_if = "is_default")]
    pub connection: String,
    /// optional host/ip to ping
    #[serde(skip_serializing_if = "is_default")]
    pub verify_target: String,
}

/// NetworkResultData is sent from agent to hub with network action results.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkResultData {
    pub request_id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub output: String,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
    #[serde(skip_serializing_if = "is_default")]
    pub rollback_attempted: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub rollback_succeeded: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub rollback_output: String,
    #[serde(skip_serializing_if = "is_default")]
    pub rollback_reference: String,
}

/// CronListData is sent from hub to agent to request cron/timer entries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CronListData {
    pub request_id: String,
}

/// CronListedData is sent from agent to hub with cron/timer entries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CronListedData {
    pub request_id: String,
    #[serde(deserialize_with = "null_default")]
    pub entries: Vec<CronEntry>,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// CronEntry describes a single cron job or systemd timer.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CronEntry {
    /// "systemd-timer" or "crontab"
    pub source: String,
    /// cron expression or systemd calendar spec
    pub schedule: String,
    pub command: String,
    pub user: String,
    /// RFC3339 or empty
    #[serde(skip_serializing_if = "is_default")]
    pub next_run: String,
    /// RFC3339 or empty
    #[serde(skip_serializing_if = "is_default")]
    pub last_run: String,
}

/// UsersListData is sent from hub to agent to request active user sessions.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UsersListData {
    pub request_id: String,
}

/// UsersListedData is sent from agent to hub with active user sessions.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UsersListedData {
    pub request_id: String,
    #[serde(deserialize_with = "null_default")]
    pub sessions: Vec<UserSession>,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// UserSession describes a single logged-in user session.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UserSession {
    pub username: String,
    pub terminal: String,
    #[serde(skip_serializing_if = "is_default")]
    pub remote_host: String,
    /// RFC3339 or human-readable
    pub login_time: String,
    #[serde(skip_serializing_if = "is_default")]
    pub session_type: String,
    #[serde(skip_serializing_if = "is_default")]
    pub display: String,
}
