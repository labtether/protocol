package protocol

import (
	"fmt"
	"strings"
)

// PowerAction is the closed set of host power operations that the hub may
// request from an agent. These operations intentionally use a dedicated typed
// message instead of the general command channel so they cannot be confused
// with or weakened by raw-shell execution policy.
type PowerAction string

const (
	PowerActionReboot   PowerAction = "reboot"
	PowerActionShutdown PowerAction = "shutdown"
)

// Valid reports whether a power action is part of the wire contract.
func (a PowerAction) Valid() bool {
	switch a {
	case PowerActionReboot, PowerActionShutdown:
		return true
	default:
		return false
	}
}

// PowerResultStatus is the closed set of outcomes an agent may report. An
// accepted result means the operating system accepted the request; it does not
// claim that the machine has already completed the power transition.
type PowerResultStatus string

const (
	PowerResultAccepted    PowerResultStatus = "accepted"
	PowerResultUnsupported PowerResultStatus = "unsupported"
	PowerResultRejected    PowerResultStatus = "rejected"
	PowerResultFailed      PowerResultStatus = "failed"
)

// Valid reports whether a result status is part of the wire contract.
func (s PowerResultStatus) Valid() bool {
	switch s {
	case PowerResultAccepted, PowerResultUnsupported, PowerResultRejected, PowerResultFailed:
		return true
	default:
		return false
	}
}

// PowerResultCode is a machine-readable, closed reason for a non-accepted
// result. It is omitted when Status is PowerResultAccepted.
type PowerResultCode string

const (
	PowerResultCodeInvalidRequest      PowerResultCode = "invalid_request"
	PowerResultCodeAssetMismatch       PowerResultCode = "asset_mismatch"
	PowerResultCodeCapabilityDenied    PowerResultCode = "capability_denied"
	PowerResultCodeBusy                PowerResultCode = "busy"
	PowerResultCodeUnsupportedPlatform PowerResultCode = "unsupported_platform"
	PowerResultCodeExecutionFailed     PowerResultCode = "execution_failed"
	PowerResultCodeExecutionTimeout    PowerResultCode = "execution_timeout"
)

// Valid reports whether a result code is part of the wire contract.
func (c PowerResultCode) Valid() bool {
	switch c {
	case PowerResultCodeInvalidRequest,
		PowerResultCodeAssetMismatch,
		PowerResultCodeCapabilityDenied,
		PowerResultCodeBusy,
		PowerResultCodeUnsupportedPlatform,
		PowerResultCodeExecutionFailed,
		PowerResultCodeExecutionTimeout:
		return true
	default:
		return false
	}
}

// PowerActionData is sent from the hub to the agent. RequestID is duplicated
// in the outer Message.ID so both the envelope and payload can be correlated.
// AssetID lets the agent reject a message routed to the wrong connection.
type PowerActionData struct {
	RequestID string      `json:"request_id"`
	AssetID   string      `json:"asset_id"`
	Action    PowerAction `json:"action"`
}

// Validate enforces the bounded, closed power action wire contract.
func (d PowerActionData) Validate() error {
	if strings.TrimSpace(d.RequestID) == "" || len(d.RequestID) > 128 {
		return fmt.Errorf("invalid request_id")
	}
	if strings.TrimSpace(d.AssetID) == "" || len(d.AssetID) > 256 {
		return fmt.Errorf("invalid asset_id")
	}
	if !d.Action.Valid() {
		return fmt.Errorf("invalid action")
	}
	return nil
}

// PowerResultData is sent from the agent to the hub after the operating system
// accepts or rejects a power request.
type PowerResultData struct {
	RequestID string            `json:"request_id"`
	AssetID   string            `json:"asset_id"`
	Action    PowerAction       `json:"action"`
	Status    PowerResultStatus `json:"status"`
	Code      PowerResultCode   `json:"code,omitempty"`
	Message   string            `json:"message,omitempty"`
}

// Validate enforces status/code compatibility in addition to the action
// bounds. An accepted result cannot carry an error code, and every non-success
// status has a closed set of valid reasons.
func (d PowerResultData) Validate() error {
	if err := (PowerActionData{RequestID: d.RequestID, AssetID: d.AssetID, Action: d.Action}).Validate(); err != nil {
		return err
	}
	if !d.Status.Valid() {
		return fmt.Errorf("invalid status")
	}
	if len(d.Message) > 256 {
		return fmt.Errorf("message too long")
	}

	switch d.Status {
	case PowerResultAccepted:
		if d.Code != "" {
			return fmt.Errorf("accepted result must not include a code")
		}
	case PowerResultUnsupported:
		if d.Code != PowerResultCodeUnsupportedPlatform {
			return fmt.Errorf("invalid unsupported result code")
		}
	case PowerResultRejected:
		switch d.Code {
		case PowerResultCodeInvalidRequest,
			PowerResultCodeAssetMismatch,
			PowerResultCodeCapabilityDenied,
			PowerResultCodeBusy:
		default:
			return fmt.Errorf("invalid rejected result code")
		}
	case PowerResultFailed:
		if d.Code != PowerResultCodeExecutionFailed && d.Code != PowerResultCodeExecutionTimeout {
			return fmt.Errorf("invalid failed result code")
		}
	}
	return nil
}

// WoLSendData requests a Wake-on-LAN packet send operation.
type WoLSendData struct {
	RequestID string `json:"request_id,omitempty"`
	MAC       string `json:"mac"`
	Broadcast string `json:"broadcast,omitempty"`
}

// WoLResultData reports result of a WoL send operation.
type WoLResultData struct {
	RequestID string `json:"request_id,omitempty"`
	MAC       string `json:"mac"`
	OK        bool   `json:"ok"`
	Error     string `json:"error,omitempty"`
}
