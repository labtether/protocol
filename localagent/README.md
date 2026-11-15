# Local agent contract

The macOS and Windows apps start the agent as a child process and read its
state from the agent's local API. This folder defines what that API sends, so
that the Go agent, the Rust agent and both apps test against one copy.

Version 1 covers `GET /agent/status` only.

## Files

| File | What it pins |
|---|---|
| `status.v1.schema.json` | JSON Schema (2020-12) for the `/agent/status` body: required and optional fields, types, the closed sets, the timestamp and uptime formats, the `metrics` object and the `alerts` array (at most 20). No other field is allowed. |
| `codes.v1.json` | The closed sets as data: connection states, the `last_error` codes allowed with each state, the `last_error_reason` codes allowed with each `last_error`, and the rules between fields that a schema cannot say. |
| `fixtures/status/*.json` | One real body per agent state, listed below. |
| `localagent.go`, `schema.go`, `codes.go`, `shape.go` | Go package `github.com/labtether/protocol/localagent`. It embeds the files and offers `ValidateStatus` (schema plus code rules) and `CompareStatusShape` (same field names and JSON types). Standard library only. |
| `localagent_test.go` | Checks every fixture against the schema and codes, checks that the schema and codes list the same sets, and checks that broken bodies are refused. |

The schema check in `schema.go` applies only the keywords this schema uses
and refuses a schema with any other keyword.

## Rules from the Go agent

- `connected` is true only when `connection_state` is `connected`. A socket
  that waits for enrollment approval reports `connected: false`,
  `connection_state: connecting`, `last_error: enrollment_pending`.
- `last_error` and `last_error_reason` are never sent empty; they are omitted.
  `last_error_reason` comes only with `transient` (optional) and `auth_failed`
  (always `token_rejected`).
- `last_error` can be the two persistence codes joined by a comma:
  `agent_token_persistence_failed,consumed_enrollment_token_cleanup_failed`.
- `enrollment_error` is the Hub's own text, one line, at most 300
  characters. It is never sent while `connected` is true. It can come with
  `disconnected`, as in `enrollment-refused-hub-words.json`.
- `disconnected_at` is omitted while connected, while enrollment waits for
  approval, and before the first failure.
- Times are Go `time.Time` JSON: RFC 3339 with up to nine fraction digits,
  trailing zeros dropped, no fraction when it is zero. `started_at` and
  `metrics.collected_at` are UTC (`Z`). `disconnected_at` after a failed
  connect uses the host's local zone, so it can carry an offset such as
  `+10:00`. Alert times keep the zone the Hub sent.
- `uptime` is Go duration text in whole seconds (`45s`, `3m2s`, `2h14m33s`).
  The Go type omits it only when `started_at` is the zero time; a running
  agent always sends it.
- `update_available` is always `false` and `latest_version` is never sent by
  the Go agent.
- Alert `severity` and `state` are passed through from the Hub unchecked.

## Fixtures

Each body was produced by the Go agent's own `/agent/status` handler
(`labtether-agent` `internal/agentcore`, commit `d0b6a7c`), driven into the
state by a test, and copied unchanged except for these values:

- `uptime` is set to `2h14m33s`, which matches `started_at` and
  `metrics.collected_at`.
- `disconnected_at` is set to `2026-11-14T09:38:51.260417093Z`.

`started_at`, `metrics.collected_at` and alert times were fixed inside the
test, so the handler wrote them as they appear.

| Fixture | State |
|---|---|
| `connected` | Authenticated Hub connection |
| `connecting-transient-host-unreachable` | Connect failed, no route to the Hub, retrying |
| `enrollment-pending` | Socket open, enrollment waits for approval |
| `auth-failed-token-rejected` | The Hub refused the agent token (401) |
| `auth-failed-enrollment-token-rejected` | The Hub refused the enrollment token, with its words |
| `connected-token-persistence-failed` | Connected, but the agent token could not be saved |
| `enrollment-refused-hub-words` | An admin rejected the device; the Hub closed the socket |
| `disconnected-no-transport` | No Hub address configured |
| `alerts-with-temperature` | Connected, with two alerts and a temperature reading |
| `minimal-optional-fields-absent` | Every optional field omitted, zero `started_at` |

## Who runs what

- This repo: `go test ./localagent/` checks the files against each other.
- The agent: a test drives `/agent/status` into each state and calls
  `ValidateStatus` and `CompareStatusShape` against the fixture of the same
  name. A Rust agent must pass the same check.
- The macOS and Windows apps: copy these files at a pinned protocol commit
  and decode every fixture with the production decoder setup.

A change to the status body changes the agent, this folder and both apps
together. A field that is added or removed needs a new schema version.
