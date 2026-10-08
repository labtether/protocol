package protocol

// HeartbeatData mirrors assets.HeartbeatRequest fields for WebSocket transport.
type HeartbeatData struct {
	AssetID      string            `json:"asset_id"`
	Type         string            `json:"type"`
	Name         string            `json:"name"`
	Source       string            `json:"source"`
	GroupID      string            `json:"group_id,omitempty"`
	Status       string            `json:"status,omitempty"`
	Platform     string            `json:"platform,omitempty"`
	Metadata     map[string]string `json:"metadata,omitempty"`
	Capabilities []string          `json:"capabilities,omitempty"` // e.g. ["webrtc", "desktop", "terminal", "files"]
	Connectors   []ConnectorInfo   `json:"connectors,omitempty"`
}

// ConnectorInfo describes a locally discovered connector endpoint.
type ConnectorInfo struct {
	Type      string `json:"type"`
	Endpoint  string `json:"endpoint"`
	Reachable bool   `json:"reachable"`
}

// TelemetryData carries a telemetry sample over WebSocket.
type TelemetryData struct {
	AssetID          string   `json:"asset_id"`
	CPUPercent       float64  `json:"cpu_percent"`
	MemoryPercent    float64  `json:"memory_percent"`
	DiskPercent      float64  `json:"disk_percent"`
	NetRXBytesPerSec float64  `json:"net_rx_bytes_per_sec"`
	NetTXBytesPerSec float64  `json:"net_tx_bytes_per_sec"`
	TempCelsius      *float64 `json:"temp_celsius,omitempty"`
}

// CommandRequestData is sent from hub to agent to execute a command.
type CommandRequestData struct {
	JobID     string `json:"job_id"`
	SessionID string `json:"session_id"`
	CommandID string `json:"command_id"`
	Command   string `json:"command"`
	Timeout   int    `json:"timeout"`
}

// CommandResultData is sent from agent to hub with the command result.
type CommandResultData struct {
	JobID     string `json:"job_id"`
	SessionID string `json:"session_id"`
	CommandID string `json:"command_id"`
	Status    string `json:"status"`
	Output    string `json:"output"`
}

// LogStreamData carries a single log entry from agent to hub.
type LogStreamData struct {
	AssetID   string `json:"asset_id"`
	Timestamp string `json:"timestamp"`
	Level     string `json:"level"`
	Message   string `json:"message"`
	Source    string `json:"source"`
}

// LogBatchData carries multiple buffered log entries from agent to hub.
type LogBatchData struct {
	AssetID string          `json:"asset_id"`
	Entries []LogStreamData `json:"entries"`
}

// JournalQueryData is sent from hub to agent to query historical journal entries.
type JournalQueryData struct {
	RequestID string `json:"request_id"`
	Since     string `json:"since,omitempty"`    // e.g. "1h ago", "2026-02-24 08:00:00"
	Until     string `json:"until,omitempty"`    // e.g. "now", "2026-02-24 09:00:00"
	Unit      string `json:"unit,omitempty"`     // systemd unit name (e.g. "ssh.service")
	Priority  string `json:"priority,omitempty"` // debug|info|notice|warning|err|crit|alert|emerg
	Search    string `json:"search,omitempty"`   // free-text grep filter
	Limit     int    `json:"limit"`              // max entries
}

// JournalEntriesData is sent from agent to hub with historical journal entries.
type JournalEntriesData struct {
	RequestID string          `json:"request_id"`
	Entries   []LogStreamData `json:"entries"`
	Error     string          `json:"error,omitempty"`
}

// ConfigUpdateData is sent from hub to agent to update runtime configuration.
type ConfigUpdateData struct {
	CollectIntervalSec   *int    `json:"collect_interval_sec,omitempty"`
	HeartbeatIntervalSec *int    `json:"heartbeat_interval_sec,omitempty"`
	LogLevel             *string `json:"log_level,omitempty"`
}

// ConfigAppliedData is sent from agent to hub confirming config application.
type ConfigAppliedData struct {
	CollectIntervalSec   int `json:"collect_interval_sec"`
	HeartbeatIntervalSec int `json:"heartbeat_interval_sec"`
}

// AgentSettingsApplyData is sent from hub to agent to apply settings.
type AgentSettingsApplyData struct {
	RequestID           string            `json:"request_id,omitempty"`
	Revision            string            `json:"revision,omitempty"`
	Values              map[string]string `json:"values"`
	ExpectedFingerprint string            `json:"expected_fingerprint,omitempty"`
}

// AgentSettingsAppliedData is sent from agent to hub after settings apply attempt.
type AgentSettingsAppliedData struct {
	RequestID       string            `json:"request_id,omitempty"`
	Revision        string            `json:"revision,omitempty"`
	Applied         bool              `json:"applied"`
	RestartRequired bool              `json:"restart_required,omitempty"`
	Error           string            `json:"error,omitempty"`
	AppliedValues   map[string]string `json:"applied_values,omitempty"`
	Fingerprint     string            `json:"fingerprint,omitempty"`
	AppliedAt       string            `json:"applied_at,omitempty"`
}

// AgentSettingsStateData is sent from agent to hub to report current effective settings.
type AgentSettingsStateData struct {
	Revision             string            `json:"revision,omitempty"`
	Values               map[string]string `json:"values,omitempty"`
	Fingerprint          string            `json:"fingerprint,omitempty"`
	AllowRemoteOverrides bool              `json:"allow_remote_overrides"`
	ReportedAt           string            `json:"reported_at,omitempty"`
}

// UpdateRequestData is sent from hub to agent to request a system update.
type UpdateRequestData struct {
	JobID    string   `json:"job_id"`
	Mode     string   `json:"mode"`
	Force    bool     `json:"force,omitempty"`
	Packages []string `json:"packages,omitempty"`
}

// UpdateProgressData is sent from agent to hub with update progress.
type UpdateProgressData struct {
	JobID   string `json:"job_id"`
	Stage   string `json:"stage"`
	Message string `json:"message"`
}

// UpdateResultData is sent from agent to hub with the final update result.
type UpdateResultData struct {
	JobID  string `json:"job_id"`
	Status string `json:"status"`
	Output string `json:"output"`
	Error  string `json:"error,omitempty"`
}
