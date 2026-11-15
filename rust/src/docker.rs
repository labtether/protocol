//! Docker inventory, stats, events, logs, exec and Compose.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::wire::{is_default, null_default};

/// DockerEngineInfo describes the Docker daemon on an agent host.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerEngineInfo {
    pub version: String,
    pub api_version: String,
    pub os: String,
    pub arch: String,
}

/// DockerPortMapping describes a container port mapping.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerPortMapping {
    pub host: i64,
    pub container: i64,
    pub protocol: String,
}

/// DockerMountInfo describes a container mount/bind.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerMountInfo {
    pub r#type: String,
    pub source: String,
    pub destination: String,
}

/// DockerContainerInfo describes a container in the discovery payload.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub created: String,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub ports: Vec<DockerPortMapping>,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub networks: Vec<String>,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub labels: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub mounts: Vec<DockerMountInfo>,
}

/// DockerImageInfo describes a Docker image.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerImageInfo {
    pub id: String,
    #[serde(deserialize_with = "null_default")]
    pub tags: Vec<String>,
    pub size: i64,
    pub created: String,
}

/// DockerNetworkInfo describes a Docker network.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerNetworkInfo {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
}

/// DockerVolumeInfo describes a Docker volume.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerVolumeInfo {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
}

/// DockerComposeStack describes an inferred Compose stack.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerComposeStack {
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "is_default")]
    pub config_file: String,
    #[serde(skip_serializing_if = "is_default")]
    pub config_dir: String,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub config_files: Vec<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub selection_error: String,
    #[serde(deserialize_with = "null_default")]
    pub containers: Vec<String>,
}

/// DockerDiscoveryData is the payload for docker.discovery messages.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerDiscoveryData {
    pub host_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub compose_endpoint_id: String,
    /// 2 supports ordered config-file selection.
    #[serde(skip_serializing_if = "is_default")]
    pub compose_action_version: i64,
    pub engine: DockerEngineInfo,
    #[serde(deserialize_with = "null_default")]
    pub containers: Vec<DockerContainerInfo>,
    #[serde(deserialize_with = "null_default")]
    pub images: Vec<DockerImageInfo>,
    #[serde(deserialize_with = "null_default")]
    pub networks: Vec<DockerNetworkInfo>,
    #[serde(deserialize_with = "null_default")]
    pub volumes: Vec<DockerVolumeInfo>,
    #[serde(deserialize_with = "null_default")]
    pub compose_stacks: Vec<DockerComposeStack>,
}

/// DockerDiscoveryDeltaData is the payload for docker.discovery.delta messages.
/// It allows the agent to upsert/remove subsets of inventory without sending
/// a full docker.discovery snapshot on every change.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerDiscoveryDeltaData {
    pub host_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub compose_endpoint_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub compose_action_version: i64,
    #[serde(skip_serializing_if = "is_default")]
    pub engine: Option<DockerEngineInfo>,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub upsert_containers: Vec<DockerContainerInfo>,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub remove_container_ids: Vec<String>,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub upsert_images: Vec<DockerImageInfo>,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub remove_image_ids: Vec<String>,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub upsert_networks: Vec<DockerNetworkInfo>,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub remove_network_ids: Vec<String>,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub upsert_volumes: Vec<DockerVolumeInfo>,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub remove_volume_names: Vec<String>,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub compose_stacks: Vec<DockerComposeStack>,
    #[serde(skip_serializing_if = "is_default")]
    pub replace_compose_stacks: bool,
}

/// DockerContainerStats holds per-container resource metrics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerContainerStats {
    pub id: String,
    pub cpu_percent: f64,
    pub memory_bytes: i64,
    pub memory_limit: i64,
    pub memory_percent: f64,
    pub net_rx_bytes: i64,
    pub net_tx_bytes: i64,
    pub block_read_bytes: i64,
    pub block_write_bytes: i64,
    pub pids: i64,
}

/// DockerStatsData is the payload for docker.stats messages.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerStatsData {
    pub host_id: String,
    #[serde(deserialize_with = "null_default")]
    pub containers: Vec<DockerContainerStats>,
}

/// DockerEventActor describes the target of a Docker event.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerEventActor {
    pub id: String,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub attributes: BTreeMap<String, String>,
}

/// DockerEventData is the payload for docker.events messages.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerEventData {
    pub host_id: String,
    pub r#type: String,
    pub action: String,
    pub actor: DockerEventActor,
    pub timestamp: i64,
}

/// DockerActionData is the payload for docker.action messages (Hub -> Agent).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerActionData {
    pub request_id: String,
    pub action: String,
    #[serde(skip_serializing_if = "is_default")]
    pub container_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub image_ref: String,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub params: BTreeMap<String, String>,
}

/// DockerActionResultData is the payload for docker.action.result messages.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerActionResultData {
    pub request_id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
    #[serde(skip_serializing_if = "is_default")]
    pub data: String,
}

/// DockerLogsStartData is the payload for docker.logs.start messages.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerLogsStartData {
    pub session_id: String,
    pub container_id: String,
    pub tail: i64,
    pub follow: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub timestamps: bool,
}

/// DockerLogsStopData is the payload for docker.logs.stop messages.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerLogsStopData {
    pub session_id: String,
}

/// DockerLogsStreamData is the payload for docker.logs.stream messages.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerLogsStreamData {
    pub session_id: String,
    /// "stdout" or "stderr"
    pub stream: String,
    /// log line content
    pub data: String,
    #[serde(skip_serializing_if = "is_default")]
    pub timestamp: String,
}

/// DockerExecStartData is the payload for docker.exec.start messages.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerExecStartData {
    pub session_id: String,
    pub container_id: String,
    #[serde(deserialize_with = "null_default")]
    pub command: Vec<String>,
    pub tty: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub cols: i64,
    #[serde(skip_serializing_if = "is_default")]
    pub rows: i64,
}

/// DockerExecStartedData is the payload for docker.exec.started messages.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerExecStartedData {
    pub session_id: String,
}

/// DockerExecDataPayload carries exec session I/O (base64-encoded).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerExecDataPayload {
    pub session_id: String,
    /// base64-encoded bytes
    pub data: String,
}

/// DockerExecInputData carries exec stdin from hub to agent (base64-encoded).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerExecInputData {
    pub session_id: String,
    /// base64-encoded bytes
    pub data: String,
}

/// DockerExecResizeData is the payload for docker.exec.resize messages.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerExecResizeData {
    pub session_id: String,
    pub cols: i64,
    pub rows: i64,
}

/// DockerExecCloseData is the payload for docker.exec.close/closed messages.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerExecCloseData {
    pub session_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub reason: String,
}

/// DockerComposeActionData is the payload for legacy and v2 Compose actions.
/// V2 requires the ordered ConfigFiles selection; legacy messages omit it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerComposeActionData {
    pub request_id: String,
    pub stack_name: String,
    /// up, down, restart, pull, deploy
    pub action: String,
    #[serde(skip_serializing_if = "is_default")]
    pub endpoint_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub config_dir: String,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub config_files: Vec<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub compose_yaml: String,
}

/// DockerComposeResultData is the payload for docker.compose.result messages.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerComposeResultData {
    pub request_id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub output: String,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}
