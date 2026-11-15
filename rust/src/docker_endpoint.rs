//! The Docker endpoint probe.

use serde::{Deserialize, Serialize};

use crate::wire::{ValidationError, is_default, validate_wire_string, wire_string};

pub const DOCKER_ENDPOINT_TEST_REQUEST_ID_MAX_BYTES: usize = 128;
pub const DOCKER_ENDPOINT_TEST_ASSET_ID_MAX_BYTES: usize = 256;
pub const DOCKER_ENDPOINT_TEST_ENDPOINT_MAX_BYTES: usize = 2048;
pub const DOCKER_ENDPOINT_TEST_MESSAGE_MAX_BYTES: usize = 256;

wire_string! {
    /// DockerEndpointTestStatus is the closed set of native endpoint probe outcomes.
    DockerEndpointTestStatus {
        REACHABLE = "reachable",
        REJECTED = "rejected",
        FAILED = "failed",
    }
}

wire_string! {
    /// DockerEndpointTestCode is the closed machine-readable reason for a
    /// rejected or failed endpoint probe. Reachable results do not carry a code.
    DockerEndpointTestCode {
        INVALID_REQUEST = "invalid_request",
        ASSET_MISMATCH = "asset_mismatch",
        BUSY = "busy",
        UNREACHABLE = "unreachable",
        TIMEOUT = "timeout",
    }
}

/// DockerEndpointTestData asks an agent to validate and probe a Docker endpoint
/// with its native Docker client. It is deliberately separate from the generic
/// command channel so endpoint probing cannot widen remote shell policy.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerEndpointTestData {
    pub request_id: String,
    pub asset_id: String,
    pub endpoint: String,
}

/// DockerEndpointTestResultData reports the correlated native endpoint probe.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerEndpointTestResultData {
    pub request_id: String,
    pub asset_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub endpoint: String,
    pub status: DockerEndpointTestStatus,
    #[serde(skip_serializing_if = "is_default")]
    pub code: DockerEndpointTestCode,
    #[serde(skip_serializing_if = "is_default")]
    pub message: String,
}

impl DockerEndpointTestData {
    /// Enforces bounded identifiers and endpoint text before a probe is run.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_ids(&self.request_id, &self.asset_id)?;
        validate_wire_string(
            &self.endpoint,
            DOCKER_ENDPOINT_TEST_ENDPOINT_MAX_BYTES,
            true,
        )
        .map_err(|error| error.invalid("endpoint"))
    }
}

impl DockerEndpointTestResultData {
    /// Enforces bounds and the exact status/code compatibility contract.
    /// Endpoint may be omitted only when invalid input cannot be echoed safely.
    pub fn validate(&self) -> Result<(), ValidationError> {
        type Status = DockerEndpointTestStatus;
        type Code = DockerEndpointTestCode;

        validate_ids(&self.request_id, &self.asset_id)?;
        if self.endpoint.is_empty() {
            if self.status != Status::REJECTED || self.code != Code::INVALID_REQUEST {
                return Err(ValidationError::new("endpoint is required"));
            }
        } else {
            validate_wire_string(
                &self.endpoint,
                DOCKER_ENDPOINT_TEST_ENDPOINT_MAX_BYTES,
                true,
            )
            .map_err(|error| error.invalid("endpoint"))?;
        }
        validate_wire_string(&self.message, DOCKER_ENDPOINT_TEST_MESSAGE_MAX_BYTES, false)
            .map_err(|error| error.invalid("message"))?;
        if !self.status.is_valid() {
            return Err(ValidationError::new("invalid status"));
        }

        let code = &self.code;
        if self.status == Status::REACHABLE {
            if !code.as_str().is_empty() {
                return Err(ValidationError::new(
                    "reachable result must not include a code",
                ));
            }
        } else if self.status == Status::REJECTED {
            if ![Code::INVALID_REQUEST, Code::ASSET_MISMATCH, Code::BUSY].contains(code) {
                return Err(ValidationError::new("invalid rejected result code"));
            }
        } else if ![Code::UNREACHABLE, Code::TIMEOUT].contains(code) {
            return Err(ValidationError::new("invalid failed result code"));
        }
        Ok(())
    }
}

fn validate_ids(request_id: &str, asset_id: &str) -> Result<(), ValidationError> {
    validate_wire_string(request_id, DOCKER_ENDPOINT_TEST_REQUEST_ID_MAX_BYTES, true)
        .map_err(|error| error.invalid("request_id"))?;
    validate_wire_string(asset_id, DOCKER_ENDPOINT_TEST_ASSET_ID_MAX_BYTES, true)
        .map_err(|error| error.invalid("asset_id"))
}
