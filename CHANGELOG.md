# Changelog

All notable changes to `github.com/labtether/protocol`. The Hub and the Go
agent must pin the same protocol version.

## v1.7.1 - unreleased

Changes since v1.7.0.

### Command exit codes

- `CommandResultData` (`command.result`) gains optional `exit_code`. Agents
  set it when they know the command's exit status, including `0`. It is
  absent when unknown: older agents, a command that never started, or one
  stopped by a signal or timeout. `status` still decides success, so Hubs
  that ignore the field keep working.
- Conformance fixtures cover a result without `exit_code`, with `0` and with a
  failing code.

### SSH host keys

- `SSHKeyInstalledData` (`ssh_key.installed`) gains optional `host_keys`: the
  device's SSH host public keys in authorized_keys format. The hub pins them
  so its SSH fallback can check the device's host key. Older agents omit it.

## v1.7.0

Changes since v1.6.5. All wire changes are additive: every message type and
field that existed in v1.6.5 keeps its name, JSON shape and meaning, so older
messages are unchanged.

### Compose v2 wire messages

- New message type `docker.compose.action.v2` (`MsgDockerComposeActionV2`,
  Hub to agent). It uses the existing `DockerComposeActionData` payload and
  must carry the exact, ordered `config_files` list to run. Agents report the
  result with the existing `docker.compose.result` message.
- `DockerComposeActionData` gains optional `endpoint_id` and `config_files`.
  Legacy `docker.compose.action` messages omit both.
- `DockerDiscoveryData` (`docker.discovery`) and `DockerDiscoveryDeltaData`
  (`docker.discovery.delta`) gain optional `compose_endpoint_id` and
  `compose_action_version`. An agent sends version `2` when it accepts
  `docker.compose.action.v2`; a Hub must not send the v2 message to an agent
  that has not advertised it.
- `DockerComposeStack` gains optional `config_dir`, `config_files` (the
  ordered Compose files) and `selection_error` (why no file set could be
  chosen).
- Endpoint identities must be JSON strings. A number or other type is rejected
  when decoding instead of turning into an empty selector.
- Conformance fixtures cover both the legacy and the v2 Compose messages.

### Project

- The module's `go` directive is now 1.27.2 (was 1.26.1). Consumers need Go
  1.27.2 or newer.
- Wire types are grouped into files by domain; no type or field changed in the
  move.
- Licensed under the Apache License 2.0. Added a security policy.
