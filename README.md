<div align="center">

<img src=".github/logo.svg" alt="LabTether" width="120" />

</div>

# LabTether protocol

The messages the [LabTether](https://github.com/labtether/labtether) hub and
its [agents](https://github.com/labtether/labtether-agent) send each other over
their WebSocket connection.

It holds the `Message` envelope, every message type and a typed struct for each
payload. The hub and the agent both import it, so they always agree on the
format.

## Install

```bash
go get github.com/labtether/protocol@latest
```

## Use

```go
import "github.com/labtether/protocol"

msg := protocol.Message{
    Type: protocol.MsgHeartbeat,
    ID:   "abc-123",
}
```

## Rules

- Standard library only (`encoding/json`). No imports from the hub.
- Wire changes are additive. [CHANGELOG.md](CHANGELOG.md) lists what each
  release adds.
- The hub and the Go agent must pin the same protocol version. Release the
  protocol first, then the agent, then the hub.

If you work with an AI coding agent, point it at [AGENTS.md](AGENTS.md) first.

## Security

Report vulnerabilities privately as described in [SECURITY.md](SECURITY.md).

## License

[Apache 2.0](LICENSE). Copyright 2026 LabTether.
