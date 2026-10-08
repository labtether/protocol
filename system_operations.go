package protocol

import (
	"fmt"
	"strings"
	"unicode/utf8"
)

// --- Platform Enrichment Data Structs ---
// ProcessListData is sent from hub to agent to request a process list.
type ProcessListData struct {
	RequestID string `json:"request_id"`
	SortBy    string `json:"sort_by,omitempty"` // "cpu" or "memory", default "cpu"
	Limit     int    `json:"limit,omitempty"`   // default 25
}

// ProcessListedData is sent from agent to hub with the process list result.
type ProcessListedData struct {
	RequestID string        `json:"request_id"`
	Processes []ProcessInfo `json:"processes"`
	Error     string        `json:"error,omitempty"`
}

// ProcessInfo describes a single running process.
type ProcessInfo struct {
	PID     int     `json:"pid"`
	Name    string  `json:"name"`
	User    string  `json:"user"`
	CPUPct  float64 `json:"cpu_pct"`
	MemPct  float64 `json:"mem_pct"`
	MemRSS  int64   `json:"mem_rss"`
	Command string  `json:"command"`
}

// ProcessKillData is sent from hub to agent to signal a process.
type ProcessKillData struct {
	PID    int    `json:"pid"`
	Signal string `json:"signal"` // "SIGTERM" | "SIGKILL" | "SIGINT" | "SIGHUP"; empty defaults to SIGTERM
}

// ProcessKillResultData is sent from agent to hub with the signal delivery result.
type ProcessKillResultData struct {
	PID     int    `json:"pid"`
	Success bool   `json:"success"`
	Error   string `json:"error,omitempty"`
}

// ServiceListData is sent from hub to agent to request a service list.
type ServiceListData struct {
	RequestID string `json:"request_id"`
}

// ServiceListedData is sent from agent to hub with the service list result.
type ServiceListedData struct {
	RequestID string        `json:"request_id"`
	Services  []ServiceInfo `json:"services"`
	Error     string        `json:"error,omitempty"`
}

// ServiceInfo describes a single systemd service unit.
type ServiceInfo struct {
	Name        string `json:"name"`
	Description string `json:"description"`
	ActiveState string `json:"active_state"` // active, inactive, failed, activating, deactivating
	SubState    string `json:"sub_state"`    // running, dead, exited, etc.
	Enabled     string `json:"enabled"`      // enabled, disabled, static, masked
	LoadState   string `json:"load_state"`   // loaded, not-found, masked
}

// ServiceActionData is sent from hub to agent to perform a service action.
type ServiceActionData struct {
	RequestID string `json:"request_id"`
	Service   string `json:"service"`
	Action    string `json:"action"` // start, stop, restart, enable, disable
}

// ServiceResultData is sent from agent to hub with the service action result.
type ServiceResultData struct {
	RequestID string `json:"request_id"`
	OK        bool   `json:"ok"`
	Output    string `json:"output,omitempty"`
	Error     string `json:"error,omitempty"`
}

// DiskListData is sent from hub to agent to request mount/disk info.
type DiskListData struct {
	RequestID string `json:"request_id"`
}

// DiskListedData is sent from agent to hub with mount/disk info results.
type DiskListedData struct {
	RequestID string      `json:"request_id"`
	Mounts    []MountInfo `json:"mounts"`
	Error     string      `json:"error,omitempty"`
}

// MountInfo describes a single mounted filesystem.
type MountInfo struct {
	Device     string  `json:"device"`
	MountPoint string  `json:"mount_point"`
	FSType     string  `json:"fs_type"`
	Total      uint64  `json:"total"`
	Used       uint64  `json:"used"`
	Available  uint64  `json:"available"`
	UsePct     float64 `json:"use_pct"`
}

// NetworkListData is sent from hub to agent to request network interface info.
type NetworkListData struct {
	RequestID string `json:"request_id"`
}

// NetworkListedData is sent from agent to hub with network interface results.
type NetworkListedData struct {
	RequestID  string         `json:"request_id"`
	Interfaces []NetInterface `json:"interfaces"`
	Error      string         `json:"error,omitempty"`
}

// NetInterface describes a single network interface and its counters.
type NetInterface struct {
	Name      string   `json:"name"`
	State     string   `json:"state"`
	MAC       string   `json:"mac"`
	MTU       int      `json:"mtu"`
	IPs       []string `json:"ips"`
	RXBytes   uint64   `json:"rx_bytes"`
	TXBytes   uint64   `json:"tx_bytes"`
	RXPackets uint64   `json:"rx_packets"`
	TXPackets uint64   `json:"tx_packets"`
}

// NetworkActionData is sent from hub to agent to perform a network action.
type NetworkActionData struct {
	RequestID    string `json:"request_id"`
	Action       string `json:"action"` // apply|rollback
	Method       string `json:"method,omitempty"`
	Connection   string `json:"connection,omitempty"`    // nmcli connection name
	VerifyTarget string `json:"verify_target,omitempty"` // optional host/ip to ping
}

// NetworkResultData is sent from agent to hub with network action results.
type NetworkResultData struct {
	RequestID         string `json:"request_id"`
	OK                bool   `json:"ok"`
	Output            string `json:"output,omitempty"`
	Error             string `json:"error,omitempty"`
	RollbackAttempted bool   `json:"rollback_attempted,omitempty"`
	RollbackSucceeded bool   `json:"rollback_succeeded,omitempty"`
	RollbackOutput    string `json:"rollback_output,omitempty"`
	RollbackReference string `json:"rollback_reference,omitempty"`
}

// PackageInventory identifies the package inventory requested from an agent.
// The empty value is retained as a compatibility alias for installed packages.
type PackageInventory string

const (
	PackageInventoryInstalled  PackageInventory = "installed"
	PackageInventoryUpgradable PackageInventory = "upgradable"

	// PackageInventoryMaxItems bounds package arrays carried in one control
	// message. Implementations should reject oversized results rather than
	// silently presenting a partial update inventory as complete.
	PackageInventoryMaxItems = 10_000
	PackageRequestIDMaxBytes = 512
	PackageNameMaxBytes      = 1_024
	PackageVersionMaxBytes   = 1_024
	PackageStatusMaxBytes    = 256
	PackageErrorMaxBytes     = 4_096
)

// Valid reports whether the inventory is a supported wire value. Empty is
// valid for compatibility with agents and hubs predating the discriminator.
func (i PackageInventory) Valid() bool {
	return i == "" || i == PackageInventoryInstalled || i == PackageInventoryUpgradable
}

// PackageListData is sent from hub to agent to request installed or upgradable
// packages. Inventory is omitted for the legacy installed-package request.
type PackageListData struct {
	RequestID string           `json:"request_id"`
	Inventory PackageInventory `json:"inventory,omitempty"`
}

// Validate checks the bounded package inventory request contract.
func (d PackageListData) Validate() error {
	if err := validatePackageWireString(d.RequestID, PackageRequestIDMaxBytes, true); err != nil {
		return fmt.Errorf("request_id: %w", err)
	}
	if !d.Inventory.Valid() {
		return fmt.Errorf("invalid package inventory %q", d.Inventory)
	}
	return nil
}

// PackageListedData is sent from agent to hub with a package inventory result.
// Upgradable responses must echo PackageInventoryUpgradable so a new hub can
// fail closed when connected to an older agent that ignored the request field.
type PackageListedData struct {
	RequestID string           `json:"request_id"`
	Inventory PackageInventory `json:"inventory,omitempty"`
	Packages  []PackageInfo    `json:"packages"`
	Error     string           `json:"error,omitempty"`
}

// Validate checks the bounded package inventory result contract. Upgradable
// results must contain both current and available versions for every entry.
func (d PackageListedData) Validate() error {
	if err := validatePackageWireString(d.RequestID, PackageRequestIDMaxBytes, true); err != nil {
		return fmt.Errorf("request_id: %w", err)
	}
	if !d.Inventory.Valid() {
		return fmt.Errorf("invalid package inventory %q", d.Inventory)
	}
	if err := validatePackageWireString(d.Error, PackageErrorMaxBytes, false); err != nil {
		return fmt.Errorf("error: %w", err)
	}
	if len(d.Packages) > PackageInventoryMaxItems {
		return fmt.Errorf("package inventory exceeds %d entries", PackageInventoryMaxItems)
	}
	for index, item := range d.Packages {
		if err := validatePackageWireString(item.Name, PackageNameMaxBytes, true); err != nil {
			return fmt.Errorf("package %d name: %w", index+1, err)
		}
		requireUpdateFields := d.Inventory == PackageInventoryUpgradable
		if err := validatePackageWireString(item.Version, PackageVersionMaxBytes, requireUpdateFields); err != nil {
			return fmt.Errorf("package %d version: %w", index+1, err)
		}
		if err := validatePackageWireString(item.Status, PackageStatusMaxBytes, requireUpdateFields); err != nil {
			return fmt.Errorf("package %d status: %w", index+1, err)
		}
		if err := validatePackageWireString(item.AvailableVersion, PackageVersionMaxBytes, false); err != nil {
			return fmt.Errorf("package %d available_version: %w", index+1, err)
		}
		if d.Inventory == PackageInventoryUpgradable && (strings.TrimSpace(item.AvailableVersion) == "" || item.Status != string(PackageInventoryUpgradable)) {
			return fmt.Errorf("upgradable package %d is incomplete", index+1)
		}
	}
	return nil
}

// PackageInfo describes a single installed or upgradable package. Version is
// the installed/current version; AvailableVersion is required for an
// upgradable inventory entry and omitted from legacy installed inventories.
type PackageInfo struct {
	Name             string `json:"name"`
	Version          string `json:"version"`
	AvailableVersion string `json:"available_version,omitempty"`
	Status           string `json:"status"` // "installed", "upgradable", "config-files", etc.
}

func validatePackageWireString(value string, maxBytes int, required bool) error {
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

// PackageActionData is sent from hub to agent to run package-manager actions.
const (
	PackageActionInstall = "install"
	PackageActionRemove  = "remove"
	PackageActionUpgrade = "upgrade"
	// PackageActionUpdate is a compatibility alias accepted at API/agent
	// boundaries. Senders should emit PackageActionUpgrade on the agent wire.
	PackageActionUpdate = "update"
)

type PackageActionData struct {
	RequestID string   `json:"request_id"`
	Action    string   `json:"action"` // canonical: install|remove|upgrade; update accepted as an alias
	Packages  []string `json:"packages,omitempty"`
}

// PackageResultData is sent from agent to hub with package action results.
type PackageResultData struct {
	RequestID      string `json:"request_id"`
	OK             bool   `json:"ok"`
	Output         string `json:"output"`
	Error          string `json:"error,omitempty"`
	RebootRequired bool   `json:"reboot_required,omitempty"`
}

// CronListData is sent from hub to agent to request cron/timer entries.
type CronListData struct {
	RequestID string `json:"request_id"`
}

// CronListedData is sent from agent to hub with cron/timer entries.
type CronListedData struct {
	RequestID string      `json:"request_id"`
	Entries   []CronEntry `json:"entries"`
	Error     string      `json:"error,omitempty"`
}

// CronEntry describes a single cron job or systemd timer.
type CronEntry struct {
	Source   string `json:"source"`   // "systemd-timer" or "crontab"
	Schedule string `json:"schedule"` // cron expression or systemd calendar spec
	Command  string `json:"command"`
	User     string `json:"user"`
	NextRun  string `json:"next_run,omitempty"` // RFC3339 or empty
	LastRun  string `json:"last_run,omitempty"` // RFC3339 or empty
}

// UsersListData is sent from hub to agent to request active user sessions.
type UsersListData struct {
	RequestID string `json:"request_id"`
}

// UsersListedData is sent from agent to hub with active user sessions.
type UsersListedData struct {
	RequestID string        `json:"request_id"`
	Sessions  []UserSession `json:"sessions"`
	Error     string        `json:"error,omitempty"`
}

// UserSession describes a single logged-in user session.
type UserSession struct {
	Username    string `json:"username"`
	Terminal    string `json:"terminal"`
	RemoteHost  string `json:"remote_host,omitempty"`
	LoginTime   string `json:"login_time"` // RFC3339 or human-readable
	SessionType string `json:"session_type,omitempty"`
	Display     string `json:"display,omitempty"`
}
