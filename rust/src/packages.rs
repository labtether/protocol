//! Package inventories and package actions.

use serde::{Deserialize, Serialize};

use crate::wire::{ValidationError, is_default, null_default, validate_wire_string, wire_string};

/// PACKAGE_INVENTORY_MAX_ITEMS bounds package arrays carried in one control
/// message. Implementations should reject oversized results rather than
/// silently presenting a partial update inventory as complete.
pub const PACKAGE_INVENTORY_MAX_ITEMS: usize = 10_000;
pub const PACKAGE_REQUEST_ID_MAX_BYTES: usize = 512;
pub const PACKAGE_NAME_MAX_BYTES: usize = 1_024;
pub const PACKAGE_VERSION_MAX_BYTES: usize = 1_024;
pub const PACKAGE_STATUS_MAX_BYTES: usize = 256;
pub const PACKAGE_ERROR_MAX_BYTES: usize = 4_096;

pub const PACKAGE_ACTION_INSTALL: &str = "install";
pub const PACKAGE_ACTION_REMOVE: &str = "remove";
pub const PACKAGE_ACTION_UPGRADE: &str = "upgrade";
/// A compatibility alias accepted at API/agent boundaries. Senders should emit
/// PACKAGE_ACTION_UPGRADE on the agent wire.
pub const PACKAGE_ACTION_UPDATE: &str = "update";

wire_string! {
    /// PackageInventory identifies the package inventory requested from an agent.
    /// The empty value is retained as a compatibility alias for installed
    /// packages, so `is_valid` accepts it.
    PackageInventory or_empty {
        INSTALLED = "installed",
        UPGRADABLE = "upgradable",
    }
}

/// PackageListData is sent from hub to agent to request installed or upgradable
/// packages. Inventory is omitted for the legacy installed-package request.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PackageListData {
    pub request_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub inventory: PackageInventory,
}

/// PackageListedData is sent from agent to hub with a package inventory result.
/// Upgradable responses must echo PackageInventoryUpgradable so a new hub can
/// fail closed when connected to an older agent that ignored the request field.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PackageListedData {
    pub request_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub inventory: PackageInventory,
    #[serde(deserialize_with = "null_default")]
    pub packages: Vec<PackageInfo>,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// PackageInfo describes a single installed or upgradable package. Version is
/// the installed/current version; AvailableVersion is required for an
/// upgradable inventory entry and omitted from legacy installed inventories.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "is_default")]
    pub available_version: String,
    /// "installed", "upgradable", "config-files", etc.
    pub status: String,
}

/// PackageActionData is sent from hub to agent to run package-manager actions.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PackageActionData {
    pub request_id: String,
    /// canonical: install|remove|upgrade; update accepted as an alias
    pub action: String,
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub packages: Vec<String>,
}

/// PackageResultData is sent from agent to hub with package action results.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PackageResultData {
    pub request_id: String,
    pub ok: bool,
    pub output: String,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
    #[serde(skip_serializing_if = "is_default")]
    pub reboot_required: bool,
}

impl PackageListData {
    /// Checks the bounded package inventory request contract.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_request(&self.request_id, &self.inventory)
    }
}

impl PackageListedData {
    /// Checks the bounded package inventory result contract. Upgradable
    /// results must contain both current and available versions for every entry.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_request(&self.request_id, &self.inventory)?;
        validate_wire_string(&self.error, PACKAGE_ERROR_MAX_BYTES, false)
            .map_err(|error| error.field("error"))?;
        if self.packages.len() > PACKAGE_INVENTORY_MAX_ITEMS {
            return Err(ValidationError::new(format!(
                "package inventory exceeds {PACKAGE_INVENTORY_MAX_ITEMS} entries"
            )));
        }
        let upgradable = self.inventory == PackageInventory::UPGRADABLE;
        for (index, item) in self.packages.iter().enumerate() {
            let number = index + 1;
            let check = |value: &str, max_bytes, required, field: &str| {
                validate_wire_string(value, max_bytes, required)
                    .map_err(|error| error.field(&format!("package {number} {field}")))
            };
            check(&item.name, PACKAGE_NAME_MAX_BYTES, true, "name")?;
            check(
                &item.version,
                PACKAGE_VERSION_MAX_BYTES,
                upgradable,
                "version",
            )?;
            check(&item.status, PACKAGE_STATUS_MAX_BYTES, upgradable, "status")?;
            check(
                &item.available_version,
                PACKAGE_VERSION_MAX_BYTES,
                false,
                "available_version",
            )?;
            let complete = !item.available_version.trim().is_empty()
                && item.status == PackageInventory::UPGRADABLE.as_str();
            if upgradable && !complete {
                return Err(ValidationError::new(format!(
                    "upgradable package {number} is incomplete"
                )));
            }
        }
        Ok(())
    }
}

fn validate_request(request_id: &str, inventory: &PackageInventory) -> Result<(), ValidationError> {
    validate_wire_string(request_id, PACKAGE_REQUEST_ID_MAX_BYTES, true)
        .map_err(|error| error.field("request_id"))?;
    if !inventory.is_valid() {
        return Err(ValidationError::new(format!(
            "invalid package inventory {:?}",
            inventory.as_str()
        )));
    }
    Ok(())
}
