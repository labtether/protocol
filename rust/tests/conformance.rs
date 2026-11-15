//! Runs the language-neutral fixtures in `testdata/conformance/` through the
//! Rust types. The Go package runs the same files, so a fixture both pass is
//! JSON that both sides read and write the same way.

mod common;

use labtether_protocol::{
    DockerComposeActionData, DockerDiscoveryData, DockerDiscoveryDeltaData, HeartbeatData,
    MSG_PING, Message, canonical_enrollment_asset_id, is_known_message_type,
};
use serde::Deserialize;
use serde_json::Value;
use serde_json::value::RawValue;

fn fixture(name: &str) -> String {
    let path = format!(
        "{}/../testdata/conformance/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

#[derive(Deserialize)]
struct WireFixture {
    schema: String,
    cases: Vec<WireCase>,
}

#[derive(Deserialize)]
struct WireCase {
    name: String,
    data_contract: String,
    message: Box<RawValue>,
}

/// Risk: the Rust types read or write a message differently from Go, so a
/// Rust agent and a Go hub disagree about a field.
#[test]
fn agent_wire_fixtures_round_trip() {
    let fixture: WireFixture = serde_json::from_str(&fixture("agent-wire-v1.json")).unwrap();
    assert_eq!(fixture.schema, "labtether-agent-wire-conformance/v1");
    assert!(
        !fixture.cases.is_empty(),
        "wire conformance fixture has no cases"
    );
    let payloads = common::payloads();

    for case in &fixture.cases {
        let name = &case.name;
        let message: Message = serde_json::from_str(case.message.get()).unwrap();
        assert!(
            is_known_message_type(&message.r#type),
            "{name}: unknown message type"
        );
        assert!(!message.id.is_empty(), "{name}: message ID is empty");

        let original: Value = serde_json::from_str(case.message.get()).unwrap();
        assert_eq!(
            serde_json::to_value(&message).unwrap(),
            original,
            "{name}: envelope changed"
        );

        let payload = payloads
            .get(case.data_contract.as_str())
            .unwrap_or_else(|| panic!("{name}: unsupported data contract {}", case.data_contract));
        let round_trip = (payload.round_trip)(original["data"].clone())
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(
            round_trip, original["data"],
            "{name}: payload changed wire shape"
        );
    }
}

#[derive(Deserialize)]
struct EnrollmentFixture {
    identity: Identity,
}

#[derive(Deserialize)]
struct Identity {
    canonical_asset_id_cases: Vec<AssetIdCase>,
}

#[derive(Deserialize)]
struct AssetIdCase {
    input: String,
    expected: String,
}

/// Risk: a Rust agent derives a different asset ID from the hub for the same
/// device name, and the hub treats a known device as a new one.
#[test]
fn canonical_asset_ids_match_the_shared_cases() {
    let fixture: EnrollmentFixture =
        serde_json::from_str(&fixture("connection-enrollment-v1.json")).unwrap();
    let cases = fixture.identity.canonical_asset_id_cases;
    assert!(
        cases.len() >= 4,
        "fixture must cover case, punctuation and invalid input"
    );
    for case in cases {
        assert_eq!(
            canonical_enrollment_asset_id(&case.input),
            case.expected,
            "input {:?}",
            case.input
        );
    }
}

/// Risk: a non-string Compose endpoint identity decodes to an empty selector
/// and an action runs against the wrong endpoint.
#[test]
fn compose_endpoint_identity_must_be_text() {
    assert!(serde_json::from_str::<DockerDiscoveryData>(r#"{"compose_endpoint_id":42}"#).is_err());
    assert!(
        serde_json::from_str::<DockerDiscoveryDeltaData>(r#"{"compose_endpoint_id":42}"#).is_err()
    );
    assert!(serde_json::from_str::<DockerComposeActionData>(r#"{"endpoint_id":42}"#).is_err());
}

/// Risk: a message without a payload gains empty `id` or `data` fields, or
/// decodes to an all-default payload that a handler then acts on.
#[test]
fn message_without_payload_stays_bare() {
    let ping = Message::bare(MSG_PING, "");
    assert_eq!(serde_json::to_string(&ping).unwrap(), r#"{"type":"ping"}"#);
    assert!(ping.decode::<HeartbeatData>().is_err());

    let heartbeat = Message::with_data("heartbeat", "h-1", &HeartbeatData::default()).unwrap();
    let decoded: Message =
        serde_json::from_str(&serde_json::to_string(&heartbeat).unwrap()).unwrap();
    assert_eq!(
        decoded.decode::<HeartbeatData>().unwrap(),
        HeartbeatData::default()
    );
}
