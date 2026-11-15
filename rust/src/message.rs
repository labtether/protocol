//! The message envelope and every message type.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::wire::is_default;

macro_rules! message_types {
    ($($(#[$meta:meta])* $name:ident = $value:literal),+ $(,)?) => {
        $(
            $(#[$meta])*
            pub const $name: &str = $value;
        )+

        /// Every valid agent protocol message type.
        pub const KNOWN_MESSAGE_TYPES: &[&str] = &[$($value),+];
    };
}

message_types! {
    MSG_HEARTBEAT = "heartbeat",
    MSG_TELEMETRY = "telemetry",
    MSG_COMMAND_REQUEST = "command.request",
    MSG_COMMAND_RESULT = "command.result",
    MSG_POWER_ACTION = "power.action",
    MSG_POWER_RESULT = "power.result",
    MSG_PING = "ping",
    MSG_PONG = "pong",
    MSG_LOG_STREAM = "log.stream",
    MSG_LOG_BATCH = "log.batch",
    /// Hub → Agent: historical journalctl query
    MSG_JOURNAL_QUERY = "journal.query",
    /// Agent → Hub: historical journalctl query response
    MSG_JOURNAL_ENTRIES = "journal.entries",
    MSG_CONFIG_UPDATE = "config.update",
    MSG_CONFIG_APPLIED = "config.applied",
    MSG_AGENT_SETTINGS_APPLY = "agent.settings.apply",
    MSG_AGENT_SETTINGS_APPLIED = "agent.settings.applied",
    MSG_AGENT_SETTINGS_STATE = "agent.settings.state",
    MSG_UPDATE_REQUEST = "update.request",
    MSG_UPDATE_PROGRESS = "update.progress",
    MSG_UPDATE_RESULT = "update.result",

    // Terminal session messages (interactive PTY over agent WebSocket).
    /// Hub → Agent: probe tmux availability
    MSG_TERMINAL_PROBE = "terminal.probe",
    /// Agent → Hub: tmux probe result
    MSG_TERMINAL_PROBED = "terminal.probed",
    /// Hub → Agent: start a PTY shell
    MSG_TERMINAL_START = "terminal.start",
    /// Agent → Hub: shell ready
    MSG_TERMINAL_STARTED = "terminal.started",
    /// Bidirectional: stdin/stdout chunks (base64)
    MSG_TERMINAL_DATA = "terminal.data",
    /// Hub → Agent: change PTY window size
    MSG_TERMINAL_RESIZE = "terminal.resize",
    /// Hub → Agent: end a saved tmux session by stable name
    MSG_TERMINAL_TMUX_KILL = "terminal.tmux.kill",
    /// Hub → Agent: end terminal session
    MSG_TERMINAL_CLOSE = "terminal.close",
    /// Agent → Hub: session ended
    MSG_TERMINAL_CLOSED = "terminal.closed",

    // SSH key auto-provisioning messages.
    /// Hub → Agent: install public key
    MSG_SSH_KEY_INSTALL = "ssh_key.install",
    /// Agent → Hub: key installed confirmation
    MSG_SSH_KEY_INSTALLED = "ssh_key.installed",
    /// Hub → Agent: remove public key
    MSG_SSH_KEY_REMOVE = "ssh_key.remove",
    /// Agent → Hub: key removed confirmation
    MSG_SSH_KEY_REMOVED = "ssh_key.removed",

    // Desktop (VNC) session messages.
    /// Hub → Agent: start VNC + bridge
    MSG_DESKTOP_START = "desktop.start",
    /// Agent → Hub: VNC ready
    MSG_DESKTOP_STARTED = "desktop.started",
    /// Bidirectional: base64 VNC bytes
    MSG_DESKTOP_DATA = "desktop.data",
    /// Hub → Agent: end desktop session
    MSG_DESKTOP_CLOSE = "desktop.close",
    /// Agent → Hub: session ended
    MSG_DESKTOP_CLOSED = "desktop.closed",
    /// Hub → Agent: enumerate displays
    MSG_DESKTOP_LIST_DISPLAYS = "desktop.list-displays",
    /// Agent → Hub: display enumeration response
    MSG_DESKTOP_DISPLAYS = "desktop.displays",
    /// Hub → Agent: request desktop stack diagnostic
    MSG_DESKTOP_DIAGNOSE = "desktop.diagnose",
    /// Agent → Hub: desktop stack diagnostic response
    MSG_DESKTOP_DIAGNOSED = "desktop.diagnosed",

    // WebRTC streaming messages.
    /// Agent → Hub: available encoders, audio sources
    MSG_WEBRTC_CAPABILITIES = "webrtc.capabilities",
    /// Hub → Agent: SDP offer from browser
    MSG_WEBRTC_OFFER = "webrtc.offer",
    /// Agent → Hub: SDP answer
    MSG_WEBRTC_ANSWER = "webrtc.answer",
    /// Bidirectional: ICE candidate
    MSG_WEBRTC_ICE = "webrtc.ice",
    /// Hub → Agent: start WebRTC session
    MSG_WEBRTC_START = "webrtc.start",
    /// Agent → Hub: WebRTC session ready
    MSG_WEBRTC_STARTED = "webrtc.started",
    /// Hub → Agent: stop WebRTC session
    MSG_WEBRTC_STOP = "webrtc.stop",
    /// Agent → Hub: WebRTC session ended
    MSG_WEBRTC_STOPPED = "webrtc.stopped",
    /// Hub → Agent: keyboard/mouse input (fallback)
    MSG_WEBRTC_INPUT = "webrtc.input",

    // Clipboard sync messages.
    /// Hub → Agent: read remote clipboard
    MSG_CLIPBOARD_GET = "clipboard.get",
    /// Agent → Hub: clipboard contents
    MSG_CLIPBOARD_DATA = "clipboard.data",
    /// Hub → Agent: write to remote clipboard
    MSG_CLIPBOARD_SET = "clipboard.set",
    /// Agent → Hub: write confirmation
    MSG_CLIPBOARD_SET_ACK = "clipboard.set_ack",

    // Desktop audio sideband messages (VNC sessions).
    /// Hub → Agent: start audio capture
    MSG_DESKTOP_AUDIO_START = "desktop.audio.start",
    /// Hub → Agent: stop audio capture
    MSG_DESKTOP_AUDIO_STOP = "desktop.audio.stop",
    /// Agent → Hub: Opus audio frame
    MSG_DESKTOP_AUDIO_DATA = "desktop.audio.data",
    /// Agent → Hub: state change
    MSG_DESKTOP_AUDIO_STATE = "desktop.audio.state",

    // Wake-on-LAN messages.
    /// Hub → Agent: send a WoL packet on local network
    MSG_WOL_SEND = "wol.send",
    /// Agent → Hub: WoL send result
    MSG_WOL_RESULT = "wol.result",

    // File transfer messages.
    /// Hub → Agent: list directory
    MSG_FILE_LIST = "file.list",
    /// Agent → Hub: directory listing result
    MSG_FILE_LISTED = "file.listed",
    /// Hub → Agent: read file
    MSG_FILE_READ = "file.read",
    /// Agent → Hub: file data chunk (base64)
    MSG_FILE_DATA = "file.data",
    /// Hub → Agent: write file chunk (base64)
    MSG_FILE_WRITE = "file.write",
    /// Agent → Hub: write confirmation
    MSG_FILE_WRITTEN = "file.written",
    /// Hub → Agent: create directory
    MSG_FILE_MKDIR = "file.mkdir",
    /// Hub → Agent: delete file/directory
    MSG_FILE_DELETE = "file.delete",
    /// Hub → Agent: rename/move file or directory
    MSG_FILE_RENAME = "file.rename",
    /// Hub → Agent: copy file/directory
    MSG_FILE_COPY = "file.copy",
    /// Agent → Hub: operation result (mkdir, delete, rename, copy)
    MSG_FILE_RESULT = "file.result",
    /// Hub → Agent: recursive filename search
    MSG_FILE_SEARCH = "file.search",
    /// Agent → Hub: filename search results
    MSG_FILE_SEARCH_RESULT = "file.search_result",

    // Process list messages.
    /// Hub → Agent: request process list
    MSG_PROCESS_LIST = "process.list",
    /// Agent → Hub: process list response
    MSG_PROCESS_LISTED = "process.listed",
    /// Hub → Agent: send signal to a process
    MSG_PROCESS_KILL = "process.kill",
    /// Agent → Hub: signal delivery result
    MSG_PROCESS_KILL_RESULT = "process.kill_result",

    // Service management messages.
    /// Hub → Agent: request service list
    MSG_SERVICE_LIST = "service.list",
    /// Agent → Hub: service list response
    MSG_SERVICE_LISTED = "service.listed",
    /// Hub → Agent: perform service action
    MSG_SERVICE_ACTION = "service.action",
    /// Agent → Hub: service action result
    MSG_SERVICE_RESULT = "service.result",

    // Disk/mount info messages.
    /// Hub → Agent: request mount/disk info
    MSG_DISK_LIST = "disk.list",
    /// Agent → Hub: mount/disk info response
    MSG_DISK_LISTED = "disk.listed",

    // Network interface messages.
    /// Hub → Agent: request network interface info
    MSG_NETWORK_LIST = "network.list",
    /// Agent → Hub: network interface response
    MSG_NETWORK_LISTED = "network.listed",
    /// Hub → Agent: perform network action (apply/rollback)
    MSG_NETWORK_ACTION = "network.action",
    /// Agent → Hub: network action result
    MSG_NETWORK_RESULT = "network.result",

    // Package inventory messages.
    /// Hub → Agent: request installed packages
    MSG_PACKAGE_LIST = "package.list",
    /// Agent → Hub: installed packages response
    MSG_PACKAGE_LISTED = "package.listed",
    /// Hub → Agent: run package action (install/remove/upgrade)
    MSG_PACKAGE_ACTION = "package.action",
    /// Agent → Hub: package action result
    MSG_PACKAGE_RESULT = "package.result",

    // Cron/timer visibility messages.
    /// Hub → Agent: request cron/timer entries
    MSG_CRON_LIST = "cron.list",
    /// Agent → Hub: cron/timer entries response
    MSG_CRON_LISTED = "cron.listed",

    // User session messages.
    /// Hub → Agent: request active user sessions
    MSG_USERS_LIST = "users.list",
    /// Agent → Hub: active user sessions response
    MSG_USERS_LISTED = "users.listed",

    // Alert notification messages.
    /// Hub → Agent: push alert for local display
    MSG_ALERT_NOTIFY = "alert.notify",

    // Enrollment approval messages.
    /// Hub → Agent: prove device-key possession
    MSG_ENROLLMENT_CHALLENGE = "enrollment.challenge",
    /// Agent → Hub: signed challenge proof
    MSG_ENROLLMENT_PROOF = "enrollment.proof",
    /// Hub → Agent: enrollment approved, token attached
    MSG_ENROLLMENT_APPROVED = "enrollment.approved",
    /// Hub → Agent: enrollment rejected
    MSG_ENROLLMENT_REJECTED = "enrollment.rejected",

    // Docker container management messages.
    /// Agent → Hub: full inventory
    MSG_DOCKER_DISCOVERY = "docker.discovery",
    /// Agent → Hub: incremental inventory update
    MSG_DOCKER_DISCOVERY_DELTA = "docker.discovery.delta",
    /// Agent → Hub: container stats
    MSG_DOCKER_STATS = "docker.stats",
    /// Agent → Hub: daemon events
    MSG_DOCKER_EVENTS = "docker.events",
    /// Hub → Agent: lifecycle command
    MSG_DOCKER_ACTION = "docker.action",
    /// Agent → Hub: action result
    MSG_DOCKER_ACTION_RESULT = "docker.action.result",
    /// Hub → Agent: test a validated Docker endpoint
    MSG_DOCKER_ENDPOINT_TEST = "docker.endpoint.test",
    /// Agent → Hub: Docker endpoint test result
    MSG_DOCKER_ENDPOINT_TEST_RESULT = "docker.endpoint.test.result",
    /// Hub → Agent: start log stream
    MSG_DOCKER_LOGS_START = "docker.logs.start",
    /// Hub → Agent: stop log stream
    MSG_DOCKER_LOGS_STOP = "docker.logs.stop",
    /// Agent → Hub: log lines
    MSG_DOCKER_LOGS_STREAM = "docker.logs.stream",
    /// Hub → Agent: start exec session
    MSG_DOCKER_EXEC_START = "docker.exec.start",
    /// Agent → Hub: exec ready
    MSG_DOCKER_EXEC_STARTED = "docker.exec.started",
    /// Agent → Hub: exec output
    MSG_DOCKER_EXEC_DATA = "docker.exec.data",
    /// Hub → Agent: exec stdin
    MSG_DOCKER_EXEC_INPUT = "docker.exec.input",
    /// Hub → Agent: resize exec PTY
    MSG_DOCKER_EXEC_RESIZE = "docker.exec.resize",
    /// Hub → Agent: close exec
    MSG_DOCKER_EXEC_CLOSE = "docker.exec.close",
    /// Agent → Hub: exec ended
    MSG_DOCKER_EXEC_CLOSED = "docker.exec.closed",
    /// Hub → Agent: stack operation
    MSG_DOCKER_COMPOSE_ACTION = "docker.compose.action",
    /// Hub → Agent: exact ordered config selection
    MSG_DOCKER_COMPOSE_ACTION_V2 = "docker.compose.action.v2",
    /// Agent → Hub: stack result
    MSG_DOCKER_COMPOSE_RESULT = "docker.compose.result",

    // Web service discovery messages.
    /// Agent → Hub: discovered services
    MSG_WEBSERVICE_REPORT = "webservice.report",
    /// Hub → Agent: request immediate rediscovery
    MSG_WEBSERVICE_SYNC = "webservice.sync",
}

/// Reports whether the message type is a valid protocol message.
pub fn is_known_message_type(message_type: &str) -> bool {
    KNOWN_MESSAGE_TYPES.contains(&message_type)
}

/// Message is the envelope for all agent ↔ hub WebSocket communication.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Message {
    pub r#type: String,
    #[serde(skip_serializing_if = "is_default")]
    pub id: String,
    /// The payload, kept as the JSON text it arrived in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Box<RawValue>>,
}

impl Message {
    /// Builds a message with no payload, such as `ping`.
    pub fn bare(message_type: &str, id: &str) -> Self {
        Self {
            r#type: message_type.to_owned(),
            id: id.to_owned(),
            data: None,
        }
    }

    /// Builds a message carrying `payload`.
    pub fn with_data<T: Serialize>(
        message_type: &str,
        id: &str,
        payload: &T,
    ) -> serde_json::Result<Self> {
        let mut message = Self::bare(message_type, id);
        message.data = Some(serde_json::value::to_raw_value(payload)?);
        Ok(message)
    }

    /// Decodes the payload. A message without one is an error, as in Go.
    pub fn decode<T: DeserializeOwned>(&self) -> serde_json::Result<T> {
        serde_json::from_str(self.data.as_deref().map_or("", RawValue::get))
    }
}
