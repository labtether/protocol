//! Terminal, SSH key, desktop, WebRTC, clipboard and audio sessions.

use serde::{Deserialize, Serialize};

use crate::wire::{is_default, null_default};

/// TerminalProbeResponse is sent from agent to hub with tmux availability info.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalProbeResponse {
    pub has_tmux: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub tmux_path: String,
}

/// TerminalStartData is sent from hub to agent to start a PTY shell session.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalStartData {
    pub session_id: String,
    pub cols: i64,
    pub rows: i64,
    /// optional: override default shell
    #[serde(skip_serializing_if = "is_default")]
    pub shell: String,
    /// if true, start inside tmux
    #[serde(skip_serializing_if = "is_default")]
    pub use_tmux: bool,
    /// tmux session name to create/attach
    #[serde(skip_serializing_if = "is_default")]
    pub tmux_session: String,
}

/// TerminalStartedData is sent from agent to hub when the PTY shell is ready.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalStartedData {
    pub session_id: String,
    /// true if running inside tmux
    #[serde(skip_serializing_if = "is_default")]
    pub tmux_attached: bool,
}

/// TerminalDataPayload carries terminal I/O data (base64-encoded bytes).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalDataPayload {
    pub session_id: String,
    /// base64-encoded terminal bytes
    pub data: String,
}

/// TerminalResizeData is sent from hub to agent to resize the PTY window.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalResizeData {
    pub session_id: String,
    pub cols: i64,
    pub rows: i64,
}

/// TerminalTmuxKillData is sent from hub to agent to end a saved tmux session.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalTmuxKillData {
    pub job_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub session_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub command_id: String,
    pub tmux_session: String,
    #[serde(skip_serializing_if = "is_default")]
    pub timeout: i64,
}

/// TerminalCloseData is sent to close a terminal session.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalCloseData {
    pub session_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub reason: String,
}

/// SSHKeyInstallData is sent from hub to agent to install the hub's public key.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SSHKeyInstallData {
    pub public_key: String,
    #[serde(skip_serializing_if = "is_default")]
    pub target_user: String,
}

/// SSHKeyRemoveData is sent from hub to agent to remove the hub's public key.
/// Same shape as SSHKeyInstallData: it holds the public key to match and remove.
pub type SSHKeyRemoveData = SSHKeyInstallData;

/// SSHKeyInstalledData is sent from agent to hub confirming key installation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SSHKeyInstalledData {
    pub username: String,
    pub hostname: String,
    pub home_dir: String,
    /// HostKeys lists the device's SSH host public keys in authorized_keys
    /// format, read by the agent from the SSH server's host key files. The hub
    /// pins them for its SSH fallback. Older agents omit the field.
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub host_keys: Vec<String>,
}

/// DesktopStartData is sent from hub to agent to start a VNC session.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopStartData {
    pub session_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub width: i64,
    #[serde(skip_serializing_if = "is_default")]
    pub height: i64,
    /// low, medium, high
    #[serde(skip_serializing_if = "is_default")]
    pub quality: String,
    /// e.g. ":0"
    #[serde(skip_serializing_if = "is_default")]
    pub display: String,
    #[serde(skip_serializing_if = "is_default")]
    pub vnc_password: String,
}

/// DesktopStartedData is sent from agent to hub when VNC is ready.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopStartedData {
    pub session_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub width: i64,
    #[serde(skip_serializing_if = "is_default")]
    pub height: i64,
}

/// DesktopDataPayload carries VNC protocol bytes (base64-encoded).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopDataPayload {
    pub session_id: String,
    /// base64-encoded VNC bytes
    pub data: String,
}

/// DesktopCloseData is sent to close a desktop session.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopCloseData {
    pub session_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub reason: String,
}

/// WebRTCCapabilitiesData describes the agent's WebRTC streaming capabilities.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebRTCCapabilitiesData {
    pub available: bool,
    /// e.g. "unsupported_platform:darwin", "gst_launch_not_found"
    #[serde(skip_serializing_if = "is_default")]
    pub unavailable_reason: String,
    /// e.g. ["nvenc_h264", "vaapi_h264", "vp8", "x264"]
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub video_encoders: Vec<String>,
    /// e.g. ["pulseaudio", "pipewire"]
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub audio_sources: Vec<String>,
    /// available X11 displays
    #[serde(skip_serializing_if = "is_default", deserialize_with = "null_default")]
    pub displays: Vec<String>,
    /// x11, wayland, headless
    #[serde(skip_serializing_if = "is_default")]
    pub desktop_session_type: String,
    /// x11, wayland_pipewire, headless
    #[serde(skip_serializing_if = "is_default")]
    pub desktop_backend: String,
    /// ximagesrc, pipewiresrc
    #[serde(skip_serializing_if = "is_default")]
    pub capture_backend: String,
    pub vnc_real_desktop_supported: bool,
    pub webrtc_real_desktop_supported: bool,
}

/// WebRTCSessionData is sent with webrtc.start.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebRTCSessionData {
    pub session_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub display: String,
    /// low, medium, high
    #[serde(skip_serializing_if = "is_default")]
    pub quality: String,
    #[serde(skip_serializing_if = "is_default")]
    pub width: i64,
    #[serde(skip_serializing_if = "is_default")]
    pub height: i64,
    #[serde(skip_serializing_if = "is_default")]
    pub fps: i64,
    pub audio_enabled: bool,
}

/// WebRTCSDPData carries SDP offer/answer payloads.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebRTCSDPData {
    pub session_id: String,
    /// "offer" or "answer"
    pub r#type: String,
    pub sdp: String,
}

/// WebRTCICEData carries an ICE candidate.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebRTCICEData {
    pub session_id: String,
    pub candidate: String,
    #[serde(skip_serializing_if = "is_default")]
    pub sdp_mid: String,
    #[serde(skip_serializing_if = "is_default")]
    pub sdp_mline_index: Option<i64>,
}

/// WebRTCStartedData confirms WebRTC session is ready for signaling.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebRTCStartedData {
    pub session_id: String,
    pub video_encoder: String,
    #[serde(skip_serializing_if = "is_default")]
    pub audio_source: String,
}

/// WebRTCStoppedData confirms WebRTC session teardown.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebRTCStoppedData {
    pub session_id: String,
    #[serde(skip_serializing_if = "is_default")]
    pub reason: String,
}

/// WebRTCInputData carries keyboard/mouse events over signaling (fallback for DataChannel).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebRTCInputData {
    pub session_id: String,
    /// keydown, keyup, mousemove, mousedown, mouseup
    pub r#type: String,
    #[serde(skip_serializing_if = "is_default")]
    pub key_code: i64,
    #[serde(skip_serializing_if = "is_default")]
    pub code: String,
    #[serde(skip_serializing_if = "is_default")]
    pub key: String,
    #[serde(skip_serializing_if = "is_default")]
    pub x: i64,
    #[serde(skip_serializing_if = "is_default")]
    pub y: i64,
    #[serde(skip_serializing_if = "is_default")]
    pub button: i64,
    #[serde(skip_serializing_if = "is_default")]
    pub delta_y: i64,
}

/// ClipboardGetData requests the agent's clipboard contents.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClipboardGetData {
    pub request_id: String,
    /// "text" or "image"
    pub format: String,
}

/// ClipboardDataPayload carries clipboard contents from agent to hub.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClipboardDataPayload {
    pub request_id: String,
    /// "text" or "image/png"
    pub format: String,
    /// plaintext content
    #[serde(skip_serializing_if = "is_default")]
    pub text: String,
    /// base64-encoded binary (images)
    #[serde(skip_serializing_if = "is_default")]
    pub data: String,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// ClipboardSetData writes content to the agent's clipboard.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClipboardSetData {
    pub request_id: String,
    /// "text" or "image/png"
    pub format: String,
    #[serde(skip_serializing_if = "is_default")]
    pub text: String,
    /// base64-encoded binary
    #[serde(skip_serializing_if = "is_default")]
    pub data: String,
}

/// ClipboardSetAckData confirms clipboard write.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClipboardSetAckData {
    pub request_id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// DesktopAudioStartData requests audio capture start for a VNC session.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopAudioStartData {
    pub session_id: String,
    /// bps, default 128000
    #[serde(skip_serializing_if = "is_default")]
    pub bitrate: i64,
}

/// DesktopAudioStopData requests audio capture stop for a VNC session.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopAudioStopData {
    pub session_id: String,
}

/// DesktopAudioDataPayload carries an Opus audio frame from agent to hub.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopAudioDataPayload {
    pub session_id: String,
    /// base64-encoded Opus frame
    pub data: String,
    /// unix milliseconds
    pub timestamp: i64,
}

/// DesktopAudioStateData reports audio capture state changes.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopAudioStateData {
    pub session_id: String,
    /// "started", "stopped", "unavailable"
    pub state: String,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// DisplayInfo describes one monitor/display geometry on a node.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplayInfo {
    pub name: String,
    pub width: i64,
    pub height: i64,
    pub primary: bool,
    pub offset_x: i64,
    pub offset_y: i64,
}

/// DisplayListData carries the display enumeration response.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplayListData {
    pub request_id: String,
    #[serde(deserialize_with = "null_default")]
    pub displays: Vec<DisplayInfo>,
    #[serde(skip_serializing_if = "is_default")]
    pub error: String,
}

/// DesktopDiagnosticRequest is sent Hub → Agent to request a desktop stack diagnostic.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopDiagnosticRequest {
    pub request_id: String,
}

/// DesktopDiagnosticData carries the full desktop stack diagnostic response from the agent.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopDiagnosticData {
    pub request_id: String,

    #[serde(skip_serializing_if = "is_default")]
    pub desktop_session_type: String,
    #[serde(skip_serializing_if = "is_default")]
    pub desktop_backend: String,
    #[serde(skip_serializing_if = "is_default")]
    pub desktop_user: String,
    #[serde(skip_serializing_if = "is_default")]
    pub real_display: String,
    pub vnc_real_desktop_supported: bool,
    pub webrtc_real_desktop_supported: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub capture_backend: String,

    /// Xvfb state
    pub xvfb_running: bool,
    #[serde(deserialize_with = "null_default")]
    pub xvfb_displays: Vec<String>,
    #[serde(deserialize_with = "null_default")]
    pub xvfb_pids: Vec<i64>,

    /// X11 display state
    #[serde(deserialize_with = "null_default")]
    pub active_displays: Vec<String>,
    pub env_display: String,

    /// VNC state
    pub x11vnc_running: bool,
    pub x11vnc_display: String,
    pub x11vnc_port: i64,

    /// Bootstrap shell state
    pub bootstrap_running: bool,
    pub xterm_available: bool,

    /// WebRTC state
    pub gst_launch_available: bool,
    pub gst_inspect_available: bool,
    #[serde(deserialize_with = "null_default")]
    pub video_encoders: Vec<String>,
    #[serde(deserialize_with = "null_default")]
    pub audio_sources: Vec<String>,
    pub webrtc_available: bool,
    pub webrtc_reason: String,

    /// Active sessions
    pub active_vnc_sessions: i64,
    pub active_webrtc_sessions: i64,

    /// Framebuffer check
    pub framebuffer_has_content: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub framebuffer_error: String,
}
