//! The messages the LabTether hub and its agents send each other over their
//! WebSocket connection.
//!
//! This crate mirrors the Go package at the root of this repository. Both
//! read and write the same JSON, and `tests/go_parity.rs` fails when one side
//! gains a message type, payload or field the other lacks.
//!
//! Differences from Go that a caller can see:
//!
//! - A list or map is written as `[]` or `{}` where Go writes `null` for a
//!   nil one. Both sides read either form.
//! - A `null` in place of text, a number or a boolean is an error here. Go
//!   ignores it. Neither side writes one.

mod agent_runtime;
mod docker;
mod docker_endpoint;
mod enrollment;
mod files;
mod message;
mod packages;
mod power;
mod remote_access;
mod system_operations;
mod webservices;
mod wire;

pub use agent_runtime::*;
pub use docker::*;
pub use docker_endpoint::*;
pub use enrollment::*;
pub use files::*;
pub use message::*;
pub use packages::*;
pub use power::*;
pub use remote_access::*;
pub use system_operations::*;
pub use webservices::*;
pub use wire::ValidationError;
