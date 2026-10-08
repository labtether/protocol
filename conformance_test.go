package protocol

import (
	"bytes"
	"encoding/json"
	"os"
	"strings"
	"testing"
)

type conformanceFixture struct {
	Schema string                   `json:"schema"`
	Cases  []conformanceFixtureCase `json:"cases"`
}

type conformanceFixtureCase struct {
	Name         string          `json:"name"`
	DataContract string          `json:"data_contract"`
	Message      json.RawMessage `json:"message"`
}

func TestAgentWireConformanceFixtures(t *testing.T) {
	t.Run("agent wire messages", testAgentWireMessages)
	t.Run("connection and enrollment", testConnectionEnrollmentContract)
}

func testAgentWireMessages(t *testing.T) {
	raw, err := os.ReadFile("testdata/conformance/agent-wire-v1.json")
	if err != nil {
		t.Fatal(err)
	}
	var fixture conformanceFixture
	decodeStrictJSON(t, raw, &fixture)
	if fixture.Schema != "labtether-agent-wire-conformance/v1" {
		t.Fatalf("unexpected fixture schema %q", fixture.Schema)
	}
	if len(fixture.Cases) == 0 {
		t.Fatal("wire conformance fixture has no cases")
	}

	for _, fixtureCase := range fixture.Cases {
		t.Run(fixtureCase.Name, func(t *testing.T) {
			var message Message
			decodeStrictJSON(t, fixtureCase.Message, &message)
			if !IsKnownMessageType(message.Type) {
				t.Fatalf("fixture uses unknown message type %q", message.Type)
			}
			if message.ID == "" {
				t.Fatal("fixture message ID is empty")
			}

			var contract any
			switch fixtureCase.DataContract {
			case "HeartbeatData":
				contract = &HeartbeatData{}
			case "EnrollmentChallengeData":
				contract = &EnrollmentChallengeData{}
			case "EnrollmentProofData":
				contract = &EnrollmentProofData{}
			case "PowerActionData":
				contract = &PowerActionData{}
			default:
				t.Fatalf("unsupported data contract %q", fixtureCase.DataContract)
			}
			decodeStrictJSON(t, message.Data, contract)

			roundTripData, err := json.Marshal(contract)
			if err != nil {
				t.Fatal(err)
			}
			assertJSONEqual(t, message.Data, roundTripData)
		})
	}
}

type connectionEnrollmentContract struct {
	Schema    string `json:"schema"`
	Discovery struct {
		Path           string   `json:"path"`
		RequiredFields []string `json:"required_fields"`
		TransportCases []struct {
			Name                   string `json:"name"`
			OriginScheme           string `json:"origin_scheme"`
			OriginHostClass        string `json:"origin_host_class"`
			WebSocketScheme        string `json:"websocket_scheme"`
			AllowInsecureTransport bool   `json:"allow_insecure_transport"`
		} `json:"transport_cases"`
		TrustCases []struct {
			Name             string `json:"name"`
			CustomCA         bool   `json:"custom_ca"`
			SkipVerification bool   `json:"skip_verification"`
			Accepted         bool   `json:"accepted"`
		} `json:"trust_cases"`
	} `json:"discovery"`
	Identity struct {
		ProofVersion          string `json:"proof_version"`
		CanonicalAssetIDCases []struct {
			Input    string `json:"input"`
			Expected string `json:"expected"`
		} `json:"canonical_asset_id_cases"`
	} `json:"identity"`
	StateMachine struct {
		States      []string `json:"states"`
		Transitions []struct {
			From  string `json:"from"`
			Event string `json:"event"`
			To    string `json:"to"`
		} `json:"transitions"`
		CommitInvariants []string `json:"commit_invariants"`
	} `json:"state_machine"`
}

func testConnectionEnrollmentContract(t *testing.T) {
	raw, err := os.ReadFile("testdata/conformance/connection-enrollment-v1.json")
	if err != nil {
		t.Fatal(err)
	}
	var fixture connectionEnrollmentContract
	decodeStrictJSON(t, raw, &fixture)
	if fixture.Schema != "labtether-connection-enrollment/v1" {
		t.Fatalf("unexpected fixture schema %q", fixture.Schema)
	}
	if fixture.Discovery.Path != "/api/v1/discover" {
		t.Fatalf("unexpected discovery path %q", fixture.Discovery.Path)
	}

	requiredDiscoveryFields := map[string]bool{
		"api_base_url": false,
		"hub_ws_url":   false,
		"enroll_url":   false,
	}
	for _, field := range fixture.Discovery.RequiredFields {
		if _, known := requiredDiscoveryFields[field]; !known {
			t.Fatalf("unknown required discovery field %q", field)
		}
		requiredDiscoveryFields[field] = true
	}
	for field, present := range requiredDiscoveryFields {
		if !present {
			t.Fatalf("missing required discovery field %q", field)
		}
	}

	transportCases := make(map[string]bool)
	for _, fixtureCase := range fixture.Discovery.TransportCases {
		if fixtureCase.Name == "" {
			t.Fatal("transport case has no name")
		}
		if fixtureCase.OriginScheme != "http" && fixtureCase.OriginScheme != "https" {
			t.Fatalf("transport case %q has invalid origin scheme %q", fixtureCase.Name, fixtureCase.OriginScheme)
		}
		expectedWebSocketScheme := "wss"
		if fixtureCase.OriginScheme == "http" {
			expectedWebSocketScheme = "ws"
			if !fixtureCase.AllowInsecureTransport || fixtureCase.OriginHostClass != "loopback" {
				t.Fatalf("transport case %q permits HTTP without explicit loopback-only insecure mode", fixtureCase.Name)
			}
		} else if fixtureCase.AllowInsecureTransport || fixtureCase.OriginHostClass != "any" {
			t.Fatalf("transport case %q has invalid secure-origin policy", fixtureCase.Name)
		}
		if fixtureCase.WebSocketScheme != expectedWebSocketScheme {
			t.Fatalf("transport case %q maps %s to %s, want %s", fixtureCase.Name, fixtureCase.OriginScheme, fixtureCase.WebSocketScheme, expectedWebSocketScheme)
		}
		transportCases[fixtureCase.Name] = true
	}
	for _, requiredCase := range []string{"secure-origin", "explicit-insecure-loopback"} {
		if !transportCases[requiredCase] {
			t.Fatalf("missing transport case %q", requiredCase)
		}
	}

	trustCases := make(map[string]bool)
	for _, fixtureCase := range fixture.Discovery.TrustCases {
		if fixtureCase.Name == "" {
			t.Fatal("trust case has no name")
		}
		if fixtureCase.CustomCA && fixtureCase.SkipVerification && fixtureCase.Accepted {
			t.Fatalf("trust case %q accepts contradictory CA and skip-verification settings", fixtureCase.Name)
		}
		trustCases[fixtureCase.Name] = true
	}
	for _, requiredCase := range []string{
		"system-trust",
		"custom-ca",
		"reject-custom-ca-and-skip-conflict",
	} {
		if !trustCases[requiredCase] {
			t.Fatalf("missing trust case %q", requiredCase)
		}
	}

	if fixture.Identity.ProofVersion != "v2" {
		t.Fatalf("connection continuity must use canonical identity proof v2, got %q", fixture.Identity.ProofVersion)
	}
	if len(fixture.Identity.CanonicalAssetIDCases) < 4 {
		t.Fatal("canonical identity fixture must cover case, punctuation, and invalid input")
	}
	seenCanonicalInputs := make(map[string]bool)
	for _, fixtureCase := range fixture.Identity.CanonicalAssetIDCases {
		if seenCanonicalInputs[fixtureCase.Input] {
			t.Fatalf("duplicate canonical identity input %q", fixtureCase.Input)
		}
		seenCanonicalInputs[fixtureCase.Input] = true
		if got := canonicalEnrollmentAssetID(fixtureCase.Input); got != fixtureCase.Expected {
			t.Fatalf(
				"canonical asset ID mapping for %q = %q, want %q",
				fixtureCase.Input,
				got,
				fixtureCase.Expected,
			)
		}
	}

	states := make(map[string]bool)
	for _, state := range fixture.StateMachine.States {
		if state == "" || states[state] {
			t.Fatalf("invalid or duplicate enrollment state %q", state)
		}
		states[state] = true
	}
	for _, transition := range fixture.StateMachine.Transitions {
		if !states[transition.From] || !states[transition.To] || transition.Event == "" {
			t.Fatalf("invalid enrollment transition %q --%q--> %q", transition.From, transition.Event, transition.To)
		}
	}

	invariants := make(map[string]bool)
	for _, invariant := range fixture.StateMachine.CommitInvariants {
		if invariant == "" || invariants[invariant] {
			t.Fatalf("invalid or duplicate commit invariant %q", invariant)
		}
		invariants[invariant] = true
	}
	for _, requiredInvariant := range []string{
		"canonical_asset_id_matches_hub",
		"device_key_is_preserved",
		"durable_credential_is_private",
		"one_use_token_is_removed_after_commit",
		"revoked_durable_credential_is_denied",
		"native_wrapper_state_is_isolated_from_service_state",
		"failed_commit_rolls_back_without_starting_child",
	} {
		if !invariants[requiredInvariant] {
			t.Fatalf("missing enrollment commit invariant %q", requiredInvariant)
		}
	}
}

// canonicalEnrollmentAssetID is the versioned reference behavior shared by the
// Hub and agent. It intentionally truncates bytes before ranging over runes so
// a split UTF-8 sequence has the same behavior as the shipping implementations.
func canonicalEnrollmentAssetID(assetID string) string {
	const maxAssetIDBytes = 64

	canonical := strings.ToLower(strings.TrimSpace(assetID))
	if len(canonical) > maxAssetIDBytes {
		canonical = canonical[:maxAssetIDBytes]
	}
	var normalized strings.Builder
	normalized.Grow(len(canonical))
	for _, ch := range canonical {
		switch {
		case ch >= 'a' && ch <= 'z':
			normalized.WriteRune(ch)
		case ch >= '0' && ch <= '9':
			normalized.WriteRune(ch)
		case ch == '-' || ch == '_' || ch == '.':
			normalized.WriteRune(ch)
		default:
			normalized.WriteByte('-')
		}
	}
	return strings.Trim(normalized.String(), "-")
}

func decodeStrictJSON(t *testing.T, raw []byte, destination any) {
	t.Helper()
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(destination); err != nil {
		t.Fatal(err)
	}
}

func assertJSONEqual(t *testing.T, want, got []byte) {
	t.Helper()
	var wantValue any
	var gotValue any
	if err := json.Unmarshal(want, &wantValue); err != nil {
		t.Fatal(err)
	}
	if err := json.Unmarshal(got, &gotValue); err != nil {
		t.Fatal(err)
	}
	wantCanonical, _ := json.Marshal(wantValue)
	gotCanonical, _ := json.Marshal(gotValue)
	if !bytes.Equal(wantCanonical, gotCanonical) {
		t.Fatalf("JSON round-trip changed wire shape:\nwant %s\ngot  %s", wantCanonical, gotCanonical)
	}
}
