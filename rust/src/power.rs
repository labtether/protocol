//! Host power actions and Wake-on-LAN.

use serde::{Deserialize, Serialize};

use crate::wire::{ValidationError, is_default, wire_string};

wire_string! {
    /// PowerAction is the closed set of host power operations that the hub may
    /// request from an agent. These operations intentionally use a dedicated typed
    /// message instead of the general command channel so they cannot be confused
    /// with or weakened by raw-shell execution policy.
    PowerAction {
        REBOOT = "reboot",
        SHUTDOWN = "shutdown",
    }
}

wire_string! {
    /// PowerResultStatus is the closed set of outcomes an agent may report. An
    /// accepted result means the operating system accepted the request; it does not
    /// claim that the machine has already completed the power transition.
    PowerResultStatus {
        ACCEPTED = "accepted",
        UNSUPPORTED = "unsupported",
        REJECTED = "rejected",
        FAILED = "failed",
    }
}

wire_string! {
    /// PowerResultCode is a machine-readable, closed reason for a non-accepted
    /// result. It is omitted when the status is accepted.
    PowerResultCode {
        INVALID_REQUEST = "invalid_request",
        ASSET_MISMATCH = "asset_mismatch",
        CAPABILITY_DENIED = "capability_denied",
        BUSY = "busy",
        UNSUPPORTED_PLATFORM = "unsupported_platform",
        EXECUTION_FAILED = "execution_failed",
        EXECUTION_TIMEOUT = "execution_timeout",
    }
}

/// PowerActionData is sent from the hub to the agent. RequestID is duplicated
/// in the outer Message.ID so both the envelope and payload can be correlated.
/// AssetID lets the agent reject a message routed to the wrong connection.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PowerActionData {
    pub request_id: String,
    pub asset_id: String,
    pub action: PowerAction,
}

/// PowerResultData is sent from the agent to the hub after the operating system
/// accepts or rejects a power request.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PowerResultData {
    pub request_id: String,
    pub asset_id: String,
    pub action: PowerAction,
    pub status: PowerResultStatus,
    #[serde(skip_serializing_if = "is_default")]
    pub code: PowerResultCode,
    #[serde(skip_serializing_if = "is_default")]
    pub message: String,
}

/// WoLSendData requests a Wake-on-LAN packet send operation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WoLSendData {
    #[serde(skip_serializing_if = "is_default")]
    pub request_id: String,
    pub mac: String,
    #[serde(skip_serializing_if = "is_default")]
    pub broadcast: String,
}

/// WoLResultData reports result of a WoL send operation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WoLResultData {
    #[serde(skip_serializing_if = "is_default")]
    pub request_id: String,
    pub mac: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

impl PowerActionData {
    /// Enforces the bounded, closed power action wire contract.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_action(&self.request_id, &self.asset_id, &self.action)
    }
}

impl PowerResultData {
    /// Enforces status/code compatibility in addition to the action bounds. An
    /// accepted result cannot carry an error code, and every non-success status
    /// has a closed set of valid reasons.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_action(&self.request_id, &self.asset_id, &self.action)?;
        if !self.status.is_valid() {
            return Err(ValidationError::new("invalid status"));
        }
        if self.message.len() > 256 {
            return Err(ValidationError::new("message too long"));
        }

        let code = &self.code;
        let (allowed, problem): (&[PowerResultCode], &str) =
            if self.status == PowerResultStatus::ACCEPTED {
                (&[], "accepted result must not include a code")
            } else if self.status == PowerResultStatus::UNSUPPORTED {
                (
                    &[PowerResultCode::UNSUPPORTED_PLATFORM],
                    "invalid unsupported result code",
                )
            } else if self.status == PowerResultStatus::REJECTED {
                (
                    &[
                        PowerResultCode::INVALID_REQUEST,
                        PowerResultCode::ASSET_MISMATCH,
                        PowerResultCode::CAPABILITY_DENIED,
                        PowerResultCode::BUSY,
                    ],
                    "invalid rejected result code",
                )
            } else {
                (
                    &[
                        PowerResultCode::EXECUTION_FAILED,
                        PowerResultCode::EXECUTION_TIMEOUT,
                    ],
                    "invalid failed result code",
                )
            };
        let accepted = self.status == PowerResultStatus::ACCEPTED;
        if (accepted && !code.as_str().is_empty()) || (!accepted && !allowed.contains(code)) {
            return Err(ValidationError::new(problem));
        }
        Ok(())
    }
}

fn validate_action(
    request_id: &str,
    asset_id: &str,
    action: &PowerAction,
) -> Result<(), ValidationError> {
    if request_id.trim().is_empty() || request_id.len() > 128 {
        return Err(ValidationError::new("invalid request_id"));
    }
    if asset_id.trim().is_empty() || asset_id.len() > 256 {
        return Err(ValidationError::new("invalid asset_id"));
    }
    if !action.is_valid() {
        return Err(ValidationError::new("invalid action"));
    }
    Ok(())
}
