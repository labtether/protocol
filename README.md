<div align="center">

<img src=".github/logo.svg" alt="LabTether" width="120" />

</div>

# github.com/labtether/protocol

Shared wire protocol types for LabTether agent-hub WebSocket communication.

This module contains the `Message` envelope, all message type constants, and typed data structs for every protocol message payload. It is imported by both the hub and the agent, ensuring they share a single source of truth for the protocol schema.

## Install

```bash
go get github.com/labtether/protocol@latest
```

## Usage

```go
import "github.com/labtether/protocol"

msg := protocol.Message{
    Type: protocol.MsgHeartbeat,
    ID:   "abc-123",
}
```

## Design constraints

- Zero dependencies on hub-internal packages.
- Only standard library imports (`encoding/json`).
- Extracted from `hub/internal/agentmgr/message.go`.
