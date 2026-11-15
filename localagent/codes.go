package localagent

import (
	"bytes"
	"encoding/json"
	"fmt"
	"slices"
)

// Codes is codes.v1.json: the closed sets of /agent/status and the rules
// between fields. An empty string in a list means the field is omitted.
type Codes struct {
	Schema                         string              `json:"schema"`
	Notes                          []string            `json:"notes"`
	ConnectionStates               []string            `json:"connection_states"`
	LastErrorReasons               []string            `json:"last_error_reasons"`
	LastErrorByState               map[string][]string `json:"last_error_by_state"`
	LastErrorReasonByLastError     map[string][]string `json:"last_error_reason_by_last_error"`
	ConnectedTrueOnlyInState       string              `json:"connected_true_only_in_state"`
	AbsentWhenConnected            []string            `json:"absent_when_connected"`
	AbsentDisconnectedAtStates     []string            `json:"absent_disconnected_at_states"`
	AbsentDisconnectedAtLastErrors []string            `json:"absent_disconnected_at_last_errors"`
}

// CodesSchema is the value of the schema field in codes.v1.json.
const CodesSchema = "labtether-local-agent-codes/v1"

// LoadCodes reads codes.v1.json strictly.
func LoadCodes() (*Codes, error) {
	raw, err := Files.ReadFile(CodesPath)
	if err != nil {
		return nil, err
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.DisallowUnknownFields()
	var codes Codes
	if err := decoder.Decode(&codes); err != nil {
		return nil, fmt.Errorf("%s: %w", CodesPath, err)
	}
	if codes.Schema != CodesSchema {
		return nil, fmt.Errorf("%s: schema %q, want %q", CodesPath, codes.Schema, CodesSchema)
	}
	return &codes, nil
}

// LastErrors returns every last_error code, without the empty entry.
func (c *Codes) LastErrors() []string {
	var all []string
	for _, state := range c.ConnectionStates {
		for _, code := range c.LastErrorByState[state] {
			if code != "" && !slices.Contains(all, code) {
				all = append(all, code)
			}
		}
	}
	return all
}

// check applies the rules between fields to a decoded status body. Type
// problems are left to the schema check.
func (c *Codes) check(value any) []error {
	body, ok := value.(map[string]any)
	if !ok {
		return nil
	}
	state, _ := body["connection_state"].(string)
	lastError, _ := body["last_error"].(string)
	reason, _ := body["last_error_reason"].(string)
	connected, _ := body["connected"].(bool)

	var problems []error
	allowed, known := c.LastErrorByState[state]
	if !known {
		problems = append(problems, fmt.Errorf("connection_state %q is not in %s", state, CodesPath))
	} else if !slices.Contains(allowed, lastError) {
		problems = append(problems, fmt.Errorf("last_error %q does not go with connection_state %q", lastError, state))
	}
	if reasons, ok := c.LastErrorReasonByLastError[lastError]; !ok {
		problems = append(problems, fmt.Errorf("last_error %q is not in %s", lastError, CodesPath))
	} else if !slices.Contains(reasons, reason) {
		problems = append(problems, fmt.Errorf("last_error_reason %q does not go with last_error %q", reason, lastError))
	}
	if connected != (state == c.ConnectedTrueOnlyInState) {
		problems = append(problems, fmt.Errorf("connected is %v with connection_state %q", connected, state))
	}
	if connected {
		for _, field := range c.AbsentWhenConnected {
			if _, present := body[field]; present {
				problems = append(problems, fmt.Errorf("%s is sent while connected", field))
			}
		}
	}
	if _, present := body["disconnected_at"]; present {
		if slices.Contains(c.AbsentDisconnectedAtStates, state) {
			problems = append(problems, fmt.Errorf("disconnected_at is sent with connection_state %q", state))
		}
		if slices.Contains(c.AbsentDisconnectedAtLastErrors, lastError) {
			problems = append(problems, fmt.Errorf("disconnected_at is sent with last_error %q", lastError))
		}
	}
	return problems
}
