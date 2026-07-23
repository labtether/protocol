package protocol

import (
	"encoding/json"
	"strings"
	"testing"
)

func TestIsKnownMessageType(t *testing.T) {
	known := []string{
		MsgHeartbeat, MsgTelemetry, MsgCommandRequest, MsgCommandResult,
		MsgPowerAction, MsgPowerResult,
		MsgPing, MsgPong, MsgLogStream, MsgLogBatch,
		MsgJournalQuery, MsgJournalEntries,
		MsgConfigUpdate, MsgConfigApplied,
		MsgAgentSettingsApply, MsgAgentSettingsApplied, MsgAgentSettingsState,
		MsgDockerEndpointTest, MsgDockerEndpointTestResult,
		MsgTerminalProbe, MsgTerminalProbed,
		MsgTerminalStart, MsgTerminalStarted, MsgTerminalData,
		MsgTerminalResize, MsgTerminalTmuxKill, MsgTerminalClose, MsgTerminalClosed,
		MsgDesktopStart, MsgDesktopStarted, MsgDesktopData,
		MsgDesktopClose, MsgDesktopClosed,
		MsgDesktopDiagnose, MsgDesktopDiagnosed,
		MsgDesktopListDisplays, MsgDesktopDisplays,
		MsgClipboardGet, MsgClipboardData, MsgClipboardSet, MsgClipboardSetAck,
		MsgDesktopAudioStart, MsgDesktopAudioStop, MsgDesktopAudioData, MsgDesktopAudioState,
		MsgWebRTCCapabilities, MsgWebRTCOffer, MsgWebRTCAnswer, MsgWebRTCICE,
		MsgWebRTCStart, MsgWebRTCStarted, MsgWebRTCStop, MsgWebRTCStopped, MsgWebRTCInput,
		MsgWoLSend, MsgWoLResult,
		MsgFileList, MsgFileListed, MsgFileRead, MsgFileData,
		MsgFileWrite, MsgFileWritten, MsgFileMkdir, MsgFileDelete,
		MsgFileRename, MsgFileCopy, MsgFileResult,
		MsgFileSearch, MsgFileSearchResult,
		MsgProcessList, MsgProcessListed,
		MsgProcessKill, MsgProcessKillResult,
		MsgServiceList, MsgServiceListed, MsgServiceAction, MsgServiceResult,
		MsgDiskList, MsgDiskListed,
		MsgNetworkList, MsgNetworkListed, MsgNetworkAction, MsgNetworkResult,
		MsgPackageList, MsgPackageListed, MsgPackageAction, MsgPackageResult,
		MsgCronList, MsgCronListed,
		MsgUsersList, MsgUsersListed,
		MsgUpdateRequest, MsgUpdateProgress, MsgUpdateResult,
		MsgSSHKeyInstall, MsgSSHKeyInstalled, MsgSSHKeyRemove, MsgSSHKeyRemoved,
		MsgAlertNotify,
		MsgEnrollmentChallenge, MsgEnrollmentProof,
		MsgEnrollmentApproved, MsgEnrollmentRejected,
		MsgWebServiceReport, MsgWebServiceSync,
	}

	for _, msgType := range known {
		if !IsKnownMessageType(msgType) {
			t.Errorf("IsKnownMessageType(%q) = false, want true", msgType)
		}
	}
}

func TestIsKnownMessageType_RejectsUnknown(t *testing.T) {
	unknown := []string{"", "foo", "heartbeat.v2", "HEARTBEAT", "terminal"}
	for _, msgType := range unknown {
		if IsKnownMessageType(msgType) {
			t.Errorf("IsKnownMessageType(%q) = true, want false", msgType)
		}
	}
}

func TestKnownMessageTypesCountMatchesConstants(t *testing.T) {
	// Keep this in sync with message.go as new protocol capabilities are added.
	// If a new constant is added but not to KnownMessageTypes, this test fails.
	const expectedCount = 123
	if len(KnownMessageTypes) != expectedCount {
		t.Errorf("KnownMessageTypes has %d entries, want %d (did you add a new message type?)",
			len(KnownMessageTypes), expectedCount)
	}
}

func TestDockerMessageTypesAreKnown(t *testing.T) {
	dockerTypes := []string{
		MsgDockerDiscovery, MsgDockerDiscoveryDelta, MsgDockerStats, MsgDockerEvents,
		MsgDockerAction, MsgDockerActionResult,
		MsgDockerEndpointTest, MsgDockerEndpointTestResult,
		MsgDockerLogsStart, MsgDockerLogsStop, MsgDockerLogsStream,
		MsgDockerExecStart, MsgDockerExecStarted, MsgDockerExecData,
		MsgDockerExecInput, MsgDockerExecResize, MsgDockerExecClose, MsgDockerExecClosed,
		MsgDockerComposeAction, MsgDockerComposeResult,
	}
	for _, msgType := range dockerTypes {
		if !IsKnownMessageType(msgType) {
			t.Errorf("message type %q not registered in KnownMessageTypes", msgType)
		}
	}
}

func TestDockerEndpointTestContractRoundTrip(t *testing.T) {
	request := DockerEndpointTestData{
		RequestID: "docker-test-1",
		AssetID:   "asset-1",
		Endpoint:  "unix:///run/docker.sock",
	}
	encoded, err := json.Marshal(request)
	if err != nil {
		t.Fatalf("Marshal request: %v", err)
	}
	var decoded DockerEndpointTestData
	if err := json.Unmarshal(encoded, &decoded); err != nil {
		t.Fatalf("Unmarshal request: %v", err)
	}
	if decoded != request {
		t.Fatalf("request round trip = %+v, want %+v", decoded, request)
	}

	result := DockerEndpointTestResultData{
		RequestID: request.RequestID,
		AssetID:   request.AssetID,
		Endpoint:  request.Endpoint,
		Status:    DockerEndpointTestStatusFailed,
		Code:      DockerEndpointTestCodeTimeout,
		Message:   "Docker endpoint test timed out",
	}
	encoded, err = json.Marshal(result)
	if err != nil {
		t.Fatalf("Marshal result: %v", err)
	}
	var decodedResult DockerEndpointTestResultData
	if err := json.Unmarshal(encoded, &decodedResult); err != nil {
		t.Fatalf("Unmarshal result: %v", err)
	}
	if decodedResult != result {
		t.Fatalf("result round trip = %+v, want %+v", decodedResult, result)
	}
	if err := decoded.Validate(); err != nil {
		t.Fatalf("valid request rejected: %v", err)
	}
	if err := decodedResult.Validate(); err != nil {
		t.Fatalf("valid result rejected: %v", err)
	}
}

func TestDockerEndpointTestClosedEnums(t *testing.T) {
	for _, status := range []DockerEndpointTestStatus{
		DockerEndpointTestStatusReachable,
		DockerEndpointTestStatusRejected,
		DockerEndpointTestStatusFailed,
	} {
		if !status.Valid() {
			t.Fatalf("status %q should be valid", status)
		}
	}
	if DockerEndpointTestStatus("succeeded").Valid() {
		t.Fatal("unexpected open-ended Docker endpoint status")
	}

	for _, code := range []DockerEndpointTestCode{
		DockerEndpointTestCodeInvalidRequest,
		DockerEndpointTestCodeAssetMismatch,
		DockerEndpointTestCodeBusy,
		DockerEndpointTestCodeUnreachable,
		DockerEndpointTestCodeTimeout,
	} {
		if !code.Valid() {
			t.Fatalf("code %q should be valid", code)
		}
	}
	if DockerEndpointTestCode("arbitrary").Valid() {
		t.Fatal("unexpected open-ended Docker endpoint code")
	}
}

func TestDockerEndpointTestRequestValidationFailsClosed(t *testing.T) {
	valid := DockerEndpointTestData{
		RequestID: "docker-test-1",
		AssetID:   "asset-1",
		Endpoint:  "unix:///run/docker.sock",
	}
	if err := valid.Validate(); err != nil {
		t.Fatalf("valid request rejected: %v", err)
	}

	tests := []struct {
		name string
		data DockerEndpointTestData
	}{
		{name: "empty request id", data: DockerEndpointTestData{AssetID: valid.AssetID, Endpoint: valid.Endpoint}},
		{name: "overlong request id", data: DockerEndpointTestData{RequestID: strings.Repeat("r", DockerEndpointTestRequestIDMaxBytes+1), AssetID: valid.AssetID, Endpoint: valid.Endpoint}},
		{name: "request id control", data: DockerEndpointTestData{RequestID: "request\n1", AssetID: valid.AssetID, Endpoint: valid.Endpoint}},
		{name: "asset invalid utf8", data: DockerEndpointTestData{RequestID: valid.RequestID, AssetID: string([]byte{0xff}), Endpoint: valid.Endpoint}},
		{name: "overlong asset", data: DockerEndpointTestData{RequestID: valid.RequestID, AssetID: strings.Repeat("a", DockerEndpointTestAssetIDMaxBytes+1), Endpoint: valid.Endpoint}},
		{name: "empty endpoint", data: DockerEndpointTestData{RequestID: valid.RequestID, AssetID: valid.AssetID}},
		{name: "endpoint control", data: DockerEndpointTestData{RequestID: valid.RequestID, AssetID: valid.AssetID, Endpoint: "unix:///run/docker.sock\x00"}},
		{name: "endpoint invalid utf8", data: DockerEndpointTestData{RequestID: valid.RequestID, AssetID: valid.AssetID, Endpoint: string([]byte{0xff})}},
		{name: "overlong endpoint", data: DockerEndpointTestData{RequestID: valid.RequestID, AssetID: valid.AssetID, Endpoint: strings.Repeat("e", DockerEndpointTestEndpointMaxBytes+1)}},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if err := tt.data.Validate(); err == nil {
				t.Fatal("expected validation failure")
			}
		})
	}
}

func TestDockerEndpointTestResultValidationFailsClosed(t *testing.T) {
	valid := DockerEndpointTestResultData{
		RequestID: "docker-test-1",
		AssetID:   "asset-1",
		Endpoint:  "unix:///run/docker.sock",
		Status:    DockerEndpointTestStatusFailed,
		Code:      DockerEndpointTestCodeTimeout,
		Message:   "Docker endpoint test timed out",
	}
	if err := valid.Validate(); err != nil {
		t.Fatalf("valid result rejected: %v", err)
	}

	tests := []struct {
		name   string
		result DockerEndpointTestResultData
	}{
		{name: "invalid request id", result: DockerEndpointTestResultData{AssetID: valid.AssetID, Endpoint: valid.Endpoint, Status: DockerEndpointTestStatusFailed, Code: DockerEndpointTestCodeTimeout}},
		{name: "invalid asset id", result: DockerEndpointTestResultData{RequestID: valid.RequestID, AssetID: "asset\n1", Endpoint: valid.Endpoint, Status: DockerEndpointTestStatusFailed, Code: DockerEndpointTestCodeTimeout}},
		{name: "invalid endpoint utf8", result: DockerEndpointTestResultData{RequestID: valid.RequestID, AssetID: valid.AssetID, Endpoint: string([]byte{0xff}), Status: DockerEndpointTestStatusFailed, Code: DockerEndpointTestCodeTimeout}},
		{name: "invalid status", result: DockerEndpointTestResultData{RequestID: valid.RequestID, AssetID: valid.AssetID, Endpoint: valid.Endpoint, Status: "succeeded"}},
		{name: "reachable with code", result: DockerEndpointTestResultData{RequestID: valid.RequestID, AssetID: valid.AssetID, Endpoint: valid.Endpoint, Status: DockerEndpointTestStatusReachable, Code: DockerEndpointTestCodeUnreachable}},
		{name: "rejected with failed code", result: DockerEndpointTestResultData{RequestID: valid.RequestID, AssetID: valid.AssetID, Endpoint: valid.Endpoint, Status: DockerEndpointTestStatusRejected, Code: DockerEndpointTestCodeTimeout}},
		{name: "failed with rejected code", result: DockerEndpointTestResultData{RequestID: valid.RequestID, AssetID: valid.AssetID, Endpoint: valid.Endpoint, Status: DockerEndpointTestStatusFailed, Code: DockerEndpointTestCodeBusy}},
		{name: "failed without endpoint", result: DockerEndpointTestResultData{RequestID: valid.RequestID, AssetID: valid.AssetID, Status: DockerEndpointTestStatusFailed, Code: DockerEndpointTestCodeTimeout}},
		{name: "asset mismatch without endpoint", result: DockerEndpointTestResultData{RequestID: valid.RequestID, AssetID: valid.AssetID, Status: DockerEndpointTestStatusRejected, Code: DockerEndpointTestCodeAssetMismatch}},
		{name: "message control", result: DockerEndpointTestResultData{RequestID: valid.RequestID, AssetID: valid.AssetID, Endpoint: valid.Endpoint, Status: DockerEndpointTestStatusFailed, Code: DockerEndpointTestCodeTimeout, Message: "bad\nmessage"}},
		{name: "message invalid utf8", result: DockerEndpointTestResultData{RequestID: valid.RequestID, AssetID: valid.AssetID, Endpoint: valid.Endpoint, Status: DockerEndpointTestStatusFailed, Code: DockerEndpointTestCodeTimeout, Message: string([]byte{0xff})}},
		{name: "overlong message", result: DockerEndpointTestResultData{RequestID: valid.RequestID, AssetID: valid.AssetID, Endpoint: valid.Endpoint, Status: DockerEndpointTestStatusFailed, Code: DockerEndpointTestCodeTimeout, Message: strings.Repeat("m", DockerEndpointTestMessageMaxBytes+1)}},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if err := tt.result.Validate(); err == nil {
				t.Fatal("expected validation failure")
			}
		})
	}

	invalidWithoutEndpoint := DockerEndpointTestResultData{
		RequestID: valid.RequestID,
		AssetID:   valid.AssetID,
		Status:    DockerEndpointTestStatusRejected,
		Code:      DockerEndpointTestCodeInvalidRequest,
		Message:   "invalid Docker endpoint test request",
	}
	if err := invalidWithoutEndpoint.Validate(); err != nil {
		t.Fatalf("invalid-request result may safely omit endpoint: %v", err)
	}
}

func TestMessageEnvelopeRoundTrip(t *testing.T) {
	msg := Message{
		Type: MsgHeartbeat,
		ID:   "test-123",
		Data: json.RawMessage(`{"asset_id":"node-1"}`),
	}

	encoded, err := json.Marshal(msg)
	if err != nil {
		t.Fatalf("Marshal: %v", err)
	}

	var decoded Message
	if err := json.Unmarshal(encoded, &decoded); err != nil {
		t.Fatalf("Unmarshal: %v", err)
	}

	if decoded.Type != msg.Type {
		t.Errorf("Type = %q, want %q", decoded.Type, msg.Type)
	}
	if decoded.ID != msg.ID {
		t.Errorf("ID = %q, want %q", decoded.ID, msg.ID)
	}
}

func TestMessageEnvelopeRejectsEmptyType(t *testing.T) {
	raw := `{"type":"","id":"x"}`
	var msg Message
	if err := json.Unmarshal([]byte(raw), &msg); err != nil {
		t.Fatalf("Unmarshal: %v", err)
	}
	if IsKnownMessageType(msg.Type) {
		t.Error("empty type should not be known")
	}
}
