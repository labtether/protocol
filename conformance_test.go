package protocol

import (
	"bytes"
	"encoding/json"
	"os"
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
