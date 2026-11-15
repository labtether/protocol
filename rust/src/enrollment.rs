//! Alert pushes, enrollment approval and the shared asset ID rule.

use serde::{Deserialize, Serialize};

use crate::wire::is_default;

/// AlertNotifyData is sent from the hub to the agent when an alert fires or resolves.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AlertNotifyData {
    pub id: String,
    /// critical, high, medium, low
    pub severity: String,
    pub title: String,
    pub summary: String,
    /// firing, resolved
    pub state: String,
    pub timestamp: String,
}

/// EnrollmentApprovedData is sent from hub to agent when enrollment is approved.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EnrollmentApprovedData {
    pub token: String,
    pub asset_id: String,
}

/// EnrollmentChallengeData is sent from hub to agent while pending enrollment.
/// The agent signs the challenge and returns EnrollmentProofData.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EnrollmentChallengeData {
    pub connection_id: String,
    pub nonce: String,
    #[serde(skip_serializing_if = "is_default")]
    pub expires_at: String,
}

/// EnrollmentProofData is sent from agent to hub to prove possession of the
/// device private key for pending enrollment verification.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EnrollmentProofData {
    pub connection_id: String,
    pub nonce: String,
    pub key_algorithm: String,
    /// base64 raw public key bytes
    pub public_key: String,
    /// human-friendly fingerprint
    pub fingerprint: String,
    /// base64 signature over canonical challenge payload
    pub signature: String,
}

/// EnrollmentRejectedData is sent from hub to agent when enrollment is rejected.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EnrollmentRejectedData {
    #[serde(skip_serializing_if = "is_default")]
    pub reason: String,
}

/// The asset ID form the hub and every agent agree on: lower case, trimmed,
/// at most 64 bytes, and `-` in place of anything outside `a-z 0-9 - _ .`.
///
/// The 64-byte cut happens before characters are read, as in the shipping Go
/// code, so a character split by the cut becomes one `-` per leftover byte.
pub fn canonical_enrollment_asset_id(asset_id: &str) -> String {
    const MAX_ASSET_ID_BYTES: usize = 64;

    // Go lowers each character to one character; Rust may give several.
    let lowered: String = asset_id
        .trim()
        .chars()
        .map(|c| c.to_lowercase().next().unwrap_or(c))
        .collect();
    let bytes = &lowered.as_bytes()[..lowered.len().min(MAX_ASSET_ID_BYTES)];

    let mut normalized = String::with_capacity(bytes.len());
    for chunk in bytes.utf8_chunks() {
        for c in chunk.valid().chars() {
            let keep = c.is_ascii_lowercase() || c.is_ascii_digit() || "-_.".contains(c);
            normalized.push(if keep { c } else { '-' });
        }
        normalized.extend(chunk.invalid().iter().map(|_| '-'));
    }
    normalized.trim_matches('-').to_owned()
}
