package protocol

import (
	"fmt"
	"strings"
	"unicode/utf8"
)

const (
	DockerEndpointTestRequestIDMaxBytes = 128
	DockerEndpointTestAssetIDMaxBytes   = 256
	DockerEndpointTestEndpointMaxBytes  = 2048
	DockerEndpointTestMessageMaxBytes   = 256
)

// DockerEndpointTestStatus is the closed set of native endpoint probe outcomes.
type DockerEndpointTestStatus string

const (
	DockerEndpointTestStatusReachable DockerEndpointTestStatus = "reachable"
	DockerEndpointTestStatusRejected  DockerEndpointTestStatus = "rejected"
	DockerEndpointTestStatusFailed    DockerEndpointTestStatus = "failed"
)

// Valid reports whether a Docker endpoint test status is part of the wire contract.
func (s DockerEndpointTestStatus) Valid() bool {
	switch s {
	case DockerEndpointTestStatusReachable, DockerEndpointTestStatusRejected, DockerEndpointTestStatusFailed:
		return true
	default:
		return false
	}
}

// DockerEndpointTestCode is the closed machine-readable reason for a
// rejected or failed endpoint probe. Reachable results do not carry a code.
type DockerEndpointTestCode string

const (
	DockerEndpointTestCodeInvalidRequest DockerEndpointTestCode = "invalid_request"
	DockerEndpointTestCodeAssetMismatch  DockerEndpointTestCode = "asset_mismatch"
	DockerEndpointTestCodeBusy           DockerEndpointTestCode = "busy"
	DockerEndpointTestCodeUnreachable    DockerEndpointTestCode = "unreachable"
	DockerEndpointTestCodeTimeout        DockerEndpointTestCode = "timeout"
)

// Valid reports whether a Docker endpoint test code is part of the wire contract.
func (c DockerEndpointTestCode) Valid() bool {
	switch c {
	case DockerEndpointTestCodeInvalidRequest,
		DockerEndpointTestCodeAssetMismatch,
		DockerEndpointTestCodeBusy,
		DockerEndpointTestCodeUnreachable,
		DockerEndpointTestCodeTimeout:
		return true
	default:
		return false
	}
}

// DockerEndpointTestData asks an agent to validate and probe a Docker endpoint
// with its native Docker client. It is deliberately separate from the generic
// command channel so endpoint probing cannot widen remote shell policy.
type DockerEndpointTestData struct {
	RequestID string `json:"request_id"`
	AssetID   string `json:"asset_id"`
	Endpoint  string `json:"endpoint"`
}

// Validate enforces bounded identifiers and endpoint text before a probe is run.
func (d DockerEndpointTestData) Validate() error {
	if err := validateDockerEndpointTestWireString(d.RequestID, DockerEndpointTestRequestIDMaxBytes, true); err != nil {
		return fmt.Errorf("invalid request_id: %w", err)
	}
	if err := validateDockerEndpointTestWireString(d.AssetID, DockerEndpointTestAssetIDMaxBytes, true); err != nil {
		return fmt.Errorf("invalid asset_id: %w", err)
	}
	if err := validateDockerEndpointTestWireString(d.Endpoint, DockerEndpointTestEndpointMaxBytes, true); err != nil {
		return fmt.Errorf("invalid endpoint: %w", err)
	}
	return nil
}

// DockerEndpointTestResultData reports the correlated native endpoint probe.
type DockerEndpointTestResultData struct {
	RequestID string                   `json:"request_id"`
	AssetID   string                   `json:"asset_id"`
	Endpoint  string                   `json:"endpoint,omitempty"`
	Status    DockerEndpointTestStatus `json:"status"`
	Code      DockerEndpointTestCode   `json:"code,omitempty"`
	Message   string                   `json:"message,omitempty"`
}

// Validate enforces bounds and the exact status/code compatibility contract.
// Endpoint may be omitted only when invalid input cannot be echoed safely.
func (d DockerEndpointTestResultData) Validate() error {
	if err := validateDockerEndpointTestWireString(d.RequestID, DockerEndpointTestRequestIDMaxBytes, true); err != nil {
		return fmt.Errorf("invalid request_id: %w", err)
	}
	if err := validateDockerEndpointTestWireString(d.AssetID, DockerEndpointTestAssetIDMaxBytes, true); err != nil {
		return fmt.Errorf("invalid asset_id: %w", err)
	}
	if d.Endpoint == "" {
		if d.Status != DockerEndpointTestStatusRejected || d.Code != DockerEndpointTestCodeInvalidRequest {
			return fmt.Errorf("endpoint is required")
		}
	} else if err := validateDockerEndpointTestWireString(d.Endpoint, DockerEndpointTestEndpointMaxBytes, true); err != nil {
		return fmt.Errorf("invalid endpoint: %w", err)
	}
	if err := validateDockerEndpointTestWireString(d.Message, DockerEndpointTestMessageMaxBytes, false); err != nil {
		return fmt.Errorf("invalid message: %w", err)
	}
	if !d.Status.Valid() {
		return fmt.Errorf("invalid status")
	}

	switch d.Status {
	case DockerEndpointTestStatusReachable:
		if d.Code != "" {
			return fmt.Errorf("reachable result must not include a code")
		}
	case DockerEndpointTestStatusRejected:
		switch d.Code {
		case DockerEndpointTestCodeInvalidRequest,
			DockerEndpointTestCodeAssetMismatch,
			DockerEndpointTestCodeBusy:
		default:
			return fmt.Errorf("invalid rejected result code")
		}
	case DockerEndpointTestStatusFailed:
		if d.Code != DockerEndpointTestCodeUnreachable && d.Code != DockerEndpointTestCodeTimeout {
			return fmt.Errorf("invalid failed result code")
		}
	}
	return nil
}

func validateDockerEndpointTestWireString(value string, maxBytes int, required bool) error {
	if !utf8.ValidString(value) {
		return fmt.Errorf("must be valid UTF-8")
	}
	if required && strings.TrimSpace(value) == "" {
		return fmt.Errorf("is required")
	}
	if len(value) > maxBytes {
		return fmt.Errorf("exceeds %d bytes", maxBytes)
	}
	if strings.IndexFunc(value, func(r rune) bool { return r < 0x20 || r == 0x7f }) >= 0 {
		return fmt.Errorf("contains a control character")
	}
	return nil
}
