//! Heartbeats, telemetry, commands, logs, settings and updates.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::wire::{is_default, null_default};

/// HeartbeatData mirrors assets.HeartbeatRequest fields for WebSocket transport.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HeartbeatData {
    pub asset_id: String,
    pub r#type: String,
    pub name: String,
    pub source: String,
    #[serde(skip_serializing_if = "is_default")]
    pub group_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub status: String,
    #[serde(skip_serializing_if = "is_default")]
    pub platform: String,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub metadata: BTreeMap<String, String>,
    /// e.g. ["webrtc", "desktop", "terminal", "files"]
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub capabilities: Vec<String>,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub connectors: Vec<ConnectorInfo>,
}

/// ConnectorInfo describes a locally discovered connector endpoint.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConnectorInfo {
    pub r#type: String,
    pub endpoint: String,
    pub reachable: bool,
}

/// TelemetryData carries a telemetry sample over WebSocket.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TelemetryData {
    pub asset_id: String,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub disk_percent: f64,
    pub net_rx_bytes_per_sec: f64,
    pub net_tx_bytes_per_sec: f64,
    #[serde(skip_serializing_if = "is_default")]
    pub temp_celsius: Option<f64>,
}

/// CommandRequestData is sent from hub to agent to execute a command.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CommandRequestData {
    pub job_id: String,
    pub session_id: String,
    pub command_id: String,
    pub command: String,
    pub timeout: i64,
}

/// CommandResultData is sent from agent to hub with the command result.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CommandResultData {
    pub job_id: String,
    pub session_id: String,
    pub command_id: String,
    pub status: String,
    pub output: String,
    /// ExitCode is the command's exit status when the agent knows it. It is
    /// nil when unknown: an older agent, a command that never started, or one
    /// stopped by a signal or timeout. Status still decides success.
    #[serde(skip_serializing_if = "is_default")]
    pub exit_code: Option<i64>,
}

/// LogStreamData carries a single log entry from agent to hub.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LogStreamData {
    pub asset_id: String,
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub source: String,
}

/// LogBatchData carries multiple buffered log entries from agent to hub.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LogBatchData {
    pub asset_id: String,
    #[serde(deserialize_with = "null_default")]
    pub entries: Vec<LogStreamData>,
}

/// JournalQueryData is sent from hub to agent to query historical journal entries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct JournalQueryData {
    pub request_id: String,
    /// e.g. "1h ago", "2026-02-24 08:00:00"
    #[serde(skip_serializing_if = "is_default")]
    pub since: String,
    /// e.g. "now", "2026-02-24 09:00:00"
    #[serde(skip_serializing_if = "is_default")]
    pub until: String,
    /// systemd unit name (e.g. "ssh.service")
    #[serde(skip_serializing_if = "is_default")]
    pub unit: String,
    /// debug|info|notice|warning|err|crit|alert|emerg
    #[serde(skip_serializing_if = "is_default")]
    pub priority: String,
    /// free-text grep filter
    #[serde(skip_serializing_if = "is_default")]
    pub search: String,
    /// max entries
    pub limit: i64,
}

/// JournalEntriesData is sent from agent to hub with historical journal entries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct JournalEntriesData {
    pub request_id: String,
    #[serde(deserialize_with = "null_default")]
    pub entries: Vec<LogStreamData>,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// ConfigUpdateData is sent from hub to agent to update runtime configuration.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfigUpdateData {
    #[serde(skip_serializing_if = "is_default")]
    pub collect_interval_sec: Option<i64>,
    #[serde(skip_serializing_if = "is_default")]
    pub heartbeat_interval_sec: Option<i64>,
    #[serde(skip_serializing_if = "is_default")]
    pub log_level: Option<String>,
}

/// ConfigAppliedData is sent from agent to hub confirming config application.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfigAppliedData {
    pub collect_interval_sec: i64,
    pub heartbeat_interval_sec: i64,
}

/// AgentSettingsApplyData is sent from hub to agent to apply settings.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentSettingsApplyData {
    #[serde(skip_serializing_if = "is_default")]
    pub request_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub revision: String,
    #[serde(deserialize_with = "null_default")]
    pub values: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "is_default")]
    pub expected_fingerprint: String,
}

/// AgentSettingsAppliedData is sent from agent to hub after settings apply attempt.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentSettingsAppliedData {
    #[serde(skip_serializing_if = "is_default")]
    pub request_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub revision: String,
    pub applied: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub restart_required: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub applied_values: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "is_default")]
    pub fingerprint: String,
    #[serde(skip_serializing_if = "is_default")]
    pub applied_at: String,
}

/// AgentSettingsStateData is sent from agent to hub to report current effective settings.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentSettingsStateData {
    #[serde(skip_serializing_if = "is_default")]
    pub revision: String,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub values: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "is_default")]
    pub fingerprint: String,
    pub allow_remote_overrides: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub reported_at: String,
}

/// UpdateRequestData is sent from hub to agent to request a system update.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateRequestData {
    pub job_id: String,
    pub mode: String,
    #[serde(skip_serializing_if = "is_default")]
    pub force: bool,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub packages: Vec<String>,
}

/// UpdateProgressData is sent from agent to hub with update progress.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateProgressData {
    pub job_id: String,
    pub stage: String,
    pub message: String,
}

/// UpdateResultData is sent from agent to hub with the final update result.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateResultData {
    pub job_id: String,
    pub status: String,
    pub output: String,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}
