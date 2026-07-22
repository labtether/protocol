package protocol

import (
	"encoding/json"
	"testing"
)

func TestPowerActionClosedEnum(t *testing.T) {
	for _, action := range []PowerAction{PowerActionReboot, PowerActionShutdown} {
		if !action.Valid() {
			t.Fatalf("expected action %q to be valid", action)
		}
	}
	for _, action := range []PowerAction{"", "restart", "Reboot", "shell"} {
		if action.Valid() {
			t.Fatalf("expected action %q to be rejected", action)
		}
	}
}

func TestPowerResultClosedEnums(t *testing.T) {
	for _, status := range []PowerResultStatus{
		PowerResultAccepted,
		PowerResultUnsupported,
		PowerResultRejected,
		PowerResultFailed,
	} {
		if !status.Valid() {
			t.Fatalf("expected status %q to be valid", status)
		}
	}
	if PowerResultStatus("succeeded").Valid() {
		t.Fatal("unexpected open-ended power result status")
	}

	for _, code := range []PowerResultCode{
		PowerResultCodeInvalidRequest,
		PowerResultCodeAssetMismatch,
		PowerResultCodeCapabilityDenied,
		PowerResultCodeBusy,
		PowerResultCodeUnsupportedPlatform,
		PowerResultCodeExecutionFailed,
		PowerResultCodeExecutionTimeout,
	} {
		if !code.Valid() {
			t.Fatalf("expected code %q to be valid", code)
		}
	}
	if PowerResultCode("arbitrary").Valid() {
		t.Fatal("unexpected open-ended power result code")
	}
}

func TestPowerWireRoundTrip(t *testing.T) {
	want := PowerActionData{
		RequestID: "power-123",
		AssetID:   "node-1",
		Action:    PowerActionReboot,
	}
	raw, err := json.Marshal(want)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}

	var got PowerActionData
	if err := json.Unmarshal(raw, &got); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	if got != want {
		t.Fatalf("round-trip mismatch: got %+v want %+v", got, want)
	}
	if err := got.Validate(); err != nil {
		t.Fatalf("valid action rejected: %v", err)
	}
}

func TestPowerResultValidationFailsClosed(t *testing.T) {
	valid := PowerResultData{
		RequestID: "power-123",
		AssetID:   "node-1",
		Action:    PowerActionReboot,
		Status:    PowerResultRejected,
		Code:      PowerResultCodeCapabilityDenied,
		Message:   "power capability denied",
	}
	if err := valid.Validate(); err != nil {
		t.Fatalf("valid result rejected: %v", err)
	}

	invalid := []PowerResultData{
		{RequestID: "power-123", AssetID: "node-1", Action: PowerActionReboot, Status: PowerResultAccepted, Code: PowerResultCodeExecutionFailed},
		{RequestID: "power-123", AssetID: "node-1", Action: PowerActionReboot, Status: PowerResultRejected, Code: PowerResultCodeExecutionFailed},
		{RequestID: "power-123", AssetID: "node-1", Action: PowerActionReboot, Status: PowerResultUnsupported},
	}
	for i, result := range invalid {
		if err := result.Validate(); err == nil {
			t.Fatalf("case %d: expected validation failure", i)
		}
	}
}
