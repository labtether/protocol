//! Web service discovery.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::wire::{is_default, null_default};

/// WebServiceReportData is sent by the agent with discovered web services.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebServiceReportData {
    pub host_asset_id: String,
    #[serde(deserialize_with = "null_default")]
    pub services: Vec<DiscoveredWebService>,
    #[serde(skip_serializing_if = "is_default")]
    pub discovery: Option<WebServiceDiscoveryStats>,
}

/// WebServiceDiscoveryStats captures one discovery cycle summary from an agent.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebServiceDiscoveryStats {
    pub collected_at: String,
    pub cycle_duration_ms: i64,
    pub total_services: i64,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub sources: BTreeMap<String, WebServiceDiscoverySourceStat>,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub final_source_count: BTreeMap<String, i64>,
}

/// WebServiceDiscoverySourceStat captures per-source cycle behavior and yield.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebServiceDiscoverySourceStat {
    pub enabled: bool,
    pub duration_ms: i64,
    pub services_found: i64,
}

/// WebServiceHealthPoint captures one status check sample for a web service.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebServiceHealthPoint {
    pub at: String,
    pub status: String,
    #[serde(skip_serializing_if = "is_default")]
    pub response_ms: i64,
}

/// WebServiceHealthSummary captures rolling uptime and recent status history.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebServiceHealthSummary {
    pub window: String,
    pub checks: i64,
    pub up_checks: i64,
    pub uptime_percent: f64,
    #[serde(skip_serializing_if = "is_default")]
    pub last_checked_at: String,
    #[serde(skip_serializing_if = "is_default")]
    pub last_change_at: String,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub recent: Vec<WebServiceHealthPoint>,
}

/// DiscoveredWebService represents a single discovered web service.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DiscoveredWebService {
    pub id: String,
    pub service_key: String,
    pub name: String,
    pub category: String,
    pub url: String,
    pub source: String,
    pub status: String,
    pub response_ms: i64,
    #[serde(skip_serializing_if = "is_default")]
    pub container_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub service_unit: String,
    pub host_asset_id: String,
    pub icon_key: String,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub metadata: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "is_default")]
    pub health: Option<WebServiceHealthSummary>,
}
