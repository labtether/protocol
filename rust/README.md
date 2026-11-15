# LabTether protocol for Rust

The same messages as the Go package at the root of this repository, for Rust
hubs and agents. Both read and write the same JSON.

## Use

```toml
[dependencies]
labtether-protocol = { git = "https://github.com/labtether/protocol" }
```

```rust
use labtether_protocol::{HeartbeatData, MSG_HEARTBEAT, Message};

let heartbeat = HeartbeatData { asset_id: "nas01".into(), ..Default::default() };
let message = Message::with_data(MSG_HEARTBEAT, "abc-123", &heartbeat)?;
let text = serde_json::to_string(&message)?;

let received: Message = serde_json::from_str(&text)?;
let payload: HeartbeatData = received.decode()?;
```

Type and field names follow the Go package: `HeartbeatData.AssetID` is
`HeartbeatData.asset_id`, and `MsgHeartbeat` is `MSG_HEARTBEAT`.

## Differences from Go

- A list or map is written as `[]` or `{}` where Go writes `null` for a nil
  one. Both sides read either form.
- A `null` in place of text, a number or a boolean is an error. Go ignores it.
  Neither side writes one.

## Checks

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

`tests/go_parity.rs` reads the Go source and fails when a message type,
payload, field or closed value exists on one side only. `tests/conformance.rs`
runs the fixtures in `../testdata/conformance/` that the Go tests also run.
