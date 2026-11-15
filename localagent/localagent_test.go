package localagent

import (
	"encoding/json"
	"slices"
	"strings"
	"testing"
)

// The ten agent states the desktop apps must decode. A fixture that is
// removed or renamed would silently drop a state from every app's tests.
var wantFixtures = []string{
	"alerts-with-temperature",
	"auth-failed-enrollment-token-rejected",
	"auth-failed-token-rejected",
	"connected",
	"connected-token-persistence-failed",
	"connecting-transient-host-unreachable",
	"disconnected-no-transport",
	"enrollment-pending",
	"enrollment-refused-hub-words",
	"minimal-optional-fields-absent",
}

// Every fixture must meet the schema and the code rules; the apps test their
// decoders against these files.
func TestStatusFixturesMeetContract(t *testing.T) {
	names, err := StatusFixtureNames()
	if err != nil {
		t.Fatal(err)
	}
	if !slices.Equal(names, wantFixtures) {
		t.Fatalf("fixtures %v, want %v", names, wantFixtures)
	}
	for _, name := range names {
		t.Run(name, func(t *testing.T) {
			body, err := StatusFixture(name)
			if err != nil {
				t.Fatal(err)
			}
			if err := ValidateStatus(body); err != nil {
				t.Fatalf("fixture breaks the contract:\n%v", err)
			}
		})
	}
}

// The closed sets are written twice, in the schema and in codes.v1.json.
// An app that reads one must not see a different set from one that reads the
// other.
func TestCodesMatchSchema(t *testing.T) {
	codes, err := LoadCodes()
	if err != nil {
		t.Fatal(err)
	}
	checker, err := loadStatusSchema()
	if err != nil {
		t.Fatal(err)
	}
	properties := checker.root["properties"].(map[string]any)
	enum := func(field string) []string {
		var values []string
		for _, value := range properties[field].(map[string]any)["enum"].([]any) {
			values = append(values, value.(string))
		}
		slices.Sort(values)
		return values
	}
	sorted := func(values []string) []string {
		out := slices.Clone(values)
		slices.Sort(out)
		return out
	}
	if got, want := enum("connection_state"), sorted(codes.ConnectionStates); !slices.Equal(got, want) {
		t.Errorf("connection_state: schema %v, codes %v", got, want)
	}
	if got, want := enum("last_error"), sorted(codes.LastErrors()); !slices.Equal(got, want) {
		t.Errorf("last_error: schema %v, codes %v", got, want)
	}
	if got, want := enum("last_error_reason"), sorted(codes.LastErrorReasons); !slices.Equal(got, want) {
		t.Errorf("last_error_reason: schema %v, codes %v", got, want)
	}
	if len(codes.LastErrorByState) != len(codes.ConnectionStates) {
		t.Errorf("last_error_by_state has %d states, want %d", len(codes.LastErrorByState), len(codes.ConnectionStates))
	}
	for code, reasons := range codes.LastErrorReasonByLastError {
		if code != "" && !slices.Contains(codes.LastErrors(), code) {
			t.Errorf("last_error_reason_by_last_error has unknown last_error %q", code)
		}
		for _, reason := range reasons {
			if reason != "" && !slices.Contains(codes.LastErrorReasons, reason) {
				t.Errorf("last_error %q allows unknown reason %q", code, reason)
			}
		}
	}
	for _, code := range append(codes.LastErrors(), "") {
		if _, ok := codes.LastErrorReasonByLastError[code]; !ok {
			t.Errorf("last_error %q has no reason list", code)
		}
	}
}

// ValidateStatus must reject bodies that break the contract; a check that
// passes everything would let a wrong fixture or agent through.
func TestValidateStatusRejectsBrokenBodies(t *testing.T) {
	alerts := make([]any, 21)
	for i := range alerts {
		alerts[i] = map[string]any{"id": "a", "severity": "high", "title": "t", "summary": "s", "state": "firing", "timestamp": "2026-11-16T09:12:40Z"}
	}
	cases := []struct {
		name   string
		change func(body map[string]any)
		want   string
	}{
		{"unknown state", func(b map[string]any) { b["connection_state"] = "online" }, `"online" is not one of`},
		{"missing required field", func(b map[string]any) { delete(b, "agent_version") }, `missing required field "agent_version"`},
		{"wrong type", func(b map[string]any) { b["connected"] = "true" }, "$.connected: want boolean, got string"},
		{"unknown field", func(b map[string]any) { b["hub_connection_state"] = "connected" }, `unknown field "hub_connection_state"`},
		{"timestamp without zone", func(b map[string]any) { b["started_at"] = "2026-11-16T07:26:34" }, "$.started_at"},
		{"metric out of range", func(b map[string]any) { b["metrics"].(map[string]any)["cpu_percent"] = json.Number("140") }, "is above 100"},
		{"too many alerts", func(b map[string]any) { b["alerts"] = alerts }, "more than 20 items"},
		{"last error from another state", func(b map[string]any) { b["last_error"] = "transient" }, `last_error "transient" does not go with connection_state "connected"`},
		{"reason without failure", func(b map[string]any) { b["last_error_reason"] = "timed_out" }, `last_error_reason "timed_out" does not go with last_error ""`},
		{"connected while connecting", func(b map[string]any) {
			b["connection_state"] = "connecting"
			b["last_error"] = "enrollment_pending"
		}, `connected is true with connection_state "connecting"`},
		{"enrollment error while connected", func(b map[string]any) { b["enrollment_error"] = "words" }, "enrollment_error is sent while connected"},
	}
	base, err := StatusFixture("connected")
	if err != nil {
		t.Fatal(err)
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			value, err := decodeJSON(base)
			if err != nil {
				t.Fatal(err)
			}
			tc.change(value.(map[string]any))
			body, err := json.Marshal(value)
			if err != nil {
				t.Fatal(err)
			}
			err = ValidateStatus(body)
			if err == nil || !strings.Contains(err.Error(), tc.want) {
				t.Fatalf("got %v, want an error containing %q", err, tc.want)
			}
		})
	}
}

// A schema keyword this checker does not apply must fail loudly instead of
// being skipped.
func TestSchemaCheckerRefusesUnknownKeyword(t *testing.T) {
	_, err := newSchemaChecker(map[string]any{"type": "object", "oneOf": []any{}})
	if err == nil || !strings.Contains(err.Error(), `unsupported keyword "oneOf"`) {
		t.Fatalf("got %v", err)
	}
}
