# Claude Code Rust Port

Rust-first harness runtime research and implementation workspace.

## Purpose

This repository studies the exposed Claude Code architecture at the systems level and rebuilds the useful harness ideas as an original Rust-native CLI/runtime. The target is not path parity or source transliteration. The primary target is a usable, inspectable Claude Code-style CLI Hamza can run directly; clean reusable runtime components are a secondary outcome that can later donate into Horizon/Rune.

## Current Status

The repository already contains:

- a Cargo workspace
- Rust crates for core/session/tools/commands/runtime/cli boundaries
- architecture and implementation planning docs
- retained JSON snapshot/reference material used to understand architectural surfaces

The repository is still early, but the Rust MVP lane is already partially implemented and tracked incrementally through the active GitHub issue queue.

## Workspace Layout

```text
.
├── Cargo.toml
├── Cargo.lock
├── ARCHITECTURE.md
├── PORTING_PLAN.md
├── archive/
│   └── reference_data/
└── crates/
    ├── harness-cli/
    ├── harness-commands/
    ├── harness-core/
    ├── harness-runtime/
    ├── harness-session/
    └── harness-tools/
```

## Reference Data Note

`archive/reference_data/` is the canonical home for retained JSON snapshot material from the architecture study. It is kept in the repo for design context and documentation, but it is not part of the active Rust runtime path. Moving it out of `src/` keeps the primary Claude Code CLI/runtime surface visually clean while preserving the research artifacts that informed the port.

## Rust MVP Target

The first meaningful milestone is:

- typed core models
- session + transcript persistence
- tool and command registries
- deterministic routing
- runtime turn processor with structured events
- CLI commands for summary, route, bootstrap, resume, tools, commands, session listing, and session inspection

See:

- `ARCHITECTURE.md`
- `PORTING_PLAN.md`
- the active GitHub issue queue for the next atomic slice

## Quickstart

Build the workspace:

```bash
cargo check
```

Run tests:

```bash
cargo test -p harness-core
cargo test -p harness-tools
cargo test -p harness-commands
cargo test -p harness-session
cargo test -p harness-runtime
cargo test -p harness-cli
cargo test
```

Run clippy strictly:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Run the CLI:

```bash
cargo run -p harness-cli -- --help
```

## CLI Usage Examples

The examples below reflect the current seeded runtime surface and are protected by `cargo test -p harness-cli`. `bootstrap` creates a session file under `.sessions/`, which is gitignored, so the README uses stable placeholders for generated values that vary per run: `<session-id>` for ids, `<created-at-ms>` and `<updated-at-ms>` for persisted recency/activity metadata, and matching `.sessions/<session-id>.json` session paths plus `.sessions/<session-id>.transcript.json` transcript paths.

Each persisted session now ships a sibling transcript file at `.sessions/<session-id>.transcript.json` in a deterministic format: `{ session_id, created_at_ms, updated_at_ms, entries: [{ turn_index, prompt }] }`. Entries are appended in `turn_index` order, rewritten on every bootstrap and resume, and inspectable through `transcript-show <id>` and `transcript-show latest`.

### `summary`

```bash
cargo run -q -p harness-cli -- summary
```

```text
commands=3 tools=3 denied_prefixes=bash
```

### `route "review bash"`

```bash
cargo run -q -p harness-cli -- route "review bash"
```

```json
[
  {
    "kind": "command",
    "name": "review",
    "score": 1
  },
  {
    "kind": "tool",
    "name": "Bash",
    "score": 1
  }
]
```

### `tools`

```bash
cargo run -q -p harness-cli -- tools
```

```json
[
  {
    "name": "ReadFile",
    "description": "Read a file from disk"
  },
  {
    "name": "EditFile",
    "description": "Edit a file on disk"
  },
  {
    "name": "Bash",
    "description": "Execute shell commands"
  }
]
```

### `commands`

```bash
cargo run -q -p harness-cli -- commands
```

```json
[
  {
    "name": "review",
    "description": "Review code or diffs"
  },
  {
    "name": "agents",
    "description": "Inspect agent state"
  },
  {
    "name": "setup",
    "description": "Show runtime setup state"
  }
]
```

### `bootstrap "review bash"`

```bash
cargo run -q -p harness-cli -- bootstrap "review bash"
```

```json
{
  "session": {
    "session_id": "<session-id>",
    "created_at_ms": <created-at-ms>,
    "updated_at_ms": <updated-at-ms>,
    "messages": [
      "review bash"
    ],
    "usage": {
      "input_tokens": 2,
      "output_tokens": 2
    }
  },
  "transcript": {
    "entries": [
      {
        "turn_index": 0,
        "prompt": "review bash"
      }
    ],
    "flushed": true
  },
  "matches": [
    {
      "kind": "command",
      "name": "review",
      "score": 1
    },
    {
      "kind": "tool",
      "name": "Bash",
      "score": 1
    }
  ],
  "denials": [
    {
      "subject": "Bash",
      "reason": "tool blocked by permission policy"
    }
  ],
  "command_results": [
    {
      "name": "review",
      "handled": true,
      "message": "command 'review' would handle prompt \"review bash\""
    }
  ],
  "tool_results": [],
  "events": [
    {
      "SessionStarted": {
        "session_id": "<session-id>"
      }
    },
    {
      "PromptReceived": {
        "prompt": "review bash"
      }
    },
    {
      "RouteComputed": {
        "match_count": 2
      }
    },
    {
      "CommandMatched": {
        "name": "review",
        "score": 1
      }
    },
    {
      "ToolMatched": {
        "name": "Bash",
        "score": 1
      }
    },
    {
      "PermissionDenied": {
        "subject": "Bash",
        "reason": "tool blocked by permission policy"
      }
    },
    {
      "CommandInvoked": {
        "name": "review"
      }
    },
    {
      "CommandCompleted": {
        "name": "review",
        "handled": true
      }
    },
    {
      "TurnCompleted": {
        "stop_reason": "completed"
      }
    },
    {
      "SessionPersisted": {
        "path": ".sessions/<session-id>.json"
      }
    },
    {
      "TranscriptPersisted": {
        "path": ".sessions/<session-id>.transcript.json"
      }
    }
  ],
  "persisted_path": ".sessions/<session-id>.json",
  "persisted_transcript_path": ".sessions/<session-id>.transcript.json"
}
```

### `resume <id> "review summary"`

Append a new turn to an existing persisted session. Pass either an explicit session id or the literal `latest` target. The resumed turn is appended to the same session file rather than starting a new session, and `updated_at_ms` is refreshed so subsequent `latest` lookups point at the most recently active session.

```bash
cargo run -q -p harness-cli -- resume <session-id> "review summary"
```

```json
{
  "resumed_session_id": "<session-id>",
  "appended_turn_index": 1,
  "session": {
    "session_id": "<session-id>",
    "created_at_ms": <created-at-ms>,
    "updated_at_ms": <updated-at-ms>,
    "messages": [
      "review bash",
      "review summary"
    ],
    "usage": {
      "input_tokens": 4,
      "output_tokens": 4
    }
  },
  "transcript": {
    "entries": [
      {
        "turn_index": 0,
        "prompt": "review bash"
      },
      {
        "turn_index": 1,
        "prompt": "review summary"
      }
    ],
    "flushed": true
  },
  "matches": [
    {
      "kind": "command",
      "name": "review",
      "score": 1
    }
  ],
  "denials": [],
  "command_results": [
    {
      "name": "review",
      "handled": true,
      "message": "command 'review' would handle prompt \"review summary\""
    }
  ],
  "tool_results": [],
  "events": [
    {
      "SessionResumed": {
        "session_id": "<session-id>",
        "turn_index": 1
      }
    },
    {
      "PromptReceived": {
        "prompt": "review summary"
      }
    },
    {
      "RouteComputed": {
        "match_count": 1
      }
    },
    {
      "CommandMatched": {
        "name": "review",
        "score": 1
      }
    },
    {
      "CommandInvoked": {
        "name": "review"
      }
    },
    {
      "CommandCompleted": {
        "name": "review",
        "handled": true
      }
    },
    {
      "TurnCompleted": {
        "stop_reason": "completed"
      }
    },
    {
      "SessionPersisted": {
        "path": ".sessions/<session-id>.json"
      }
    },
    {
      "TranscriptPersisted": {
        "path": ".sessions/<session-id>.transcript.json"
      }
    }
  ],
  "persisted_path": ".sessions/<session-id>.json",
  "persisted_transcript_path": ".sessions/<session-id>.transcript.json"
}
```

`latest` is supported as the resume target too:

```bash
cargo run -q -p harness-cli -- resume latest "review summary"
```

### `sessions`

```bash
cargo run -q -p harness-cli -- sessions
```

```json
[
  {
    "session_id": "<session-id>",
    "created_at_ms": <created-at-ms>,
    "updated_at_ms": <updated-at-ms>,
    "message_count": 1,
    "persisted_path": ".sessions/<session-id>.json"
  }
]
```

### `session-show <id>`

```bash
cargo run -q -p harness-cli -- session-show <session-id>
```

```json
{
  "session_id": "<session-id>",
  "created_at_ms": <created-at-ms>,
  "updated_at_ms": <updated-at-ms>,
  "messages": [
    "review bash"
  ],
  "usage": {
    "input_tokens": 2,
    "output_tokens": 2
  }
}
```

### `session-show latest`

```bash
cargo run -q -p harness-cli -- session-show latest
```

```json
{
  "session_id": "<session-id>",
  "created_at_ms": <created-at-ms>,
  "updated_at_ms": <updated-at-ms>,
  "messages": [
    "review bash"
  ],
  "usage": {
    "input_tokens": 2,
    "output_tokens": 2
  }
}
```

### `transcript-show <id>`

Inspect the persisted transcript for a specific session. The output restates the owning `session_id` and the session's recency metadata so it is self-describing, and lists turns in append order.

```bash
cargo run -q -p harness-cli -- transcript-show <session-id>
```

```json
{
  "session_id": "<session-id>",
  "created_at_ms": <created-at-ms>,
  "updated_at_ms": <updated-at-ms>,
  "entries": [
    {
      "turn_index": 0,
      "prompt": "review bash"
    }
  ]
}
```

### `transcript-show latest`

`latest` resolves to the transcript of the most recently active persisted session, mirroring how `session-show latest` resolves session state.

```bash
cargo run -q -p harness-cli -- transcript-show latest
```

```json
{
  "session_id": "<session-id>",
  "created_at_ms": <created-at-ms>,
  "updated_at_ms": <updated-at-ms>,
  "entries": [
    {
      "turn_index": 0,
      "prompt": "review bash"
    }
  ]
}
```

## Rust Test Coverage Baseline

Current protected Rust surface:

- `harness-core` prompt/name wrappers and token accounting helpers
- seeded tool registry behavior plus permission-policy prefix denial in `harness-tools`
- seeded command registry behavior in `harness-commands`
- `harness-session` save/load round-trip persistence
- transcript compaction behavior in `harness-session`
- deterministic route ordering in `harness-runtime`
- bootstrap permission denial + session persistence behavior in `harness-runtime`
- `harness-session` recency metadata, newest-first listing, latest-session lookup, and activity-ordered `latest` after a persisted session is resumed
- README-backed CLI output regression coverage for `summary`, `route <prompt>`, `tools`, `commands`, and `sessions`
- README-backed persisted-session example coverage for `bootstrap <prompt>`, `session-show <id>`, and `session-show latest`, with generated session identifiers normalized to `<session-id>` and generated recency metadata normalized to `<created-at-ms>` / `<updated-at-ms>` in test assertions
- `harness-runtime` session resume behavior: an appended turn targets the original session id, bumps `updated_at_ms`, and emits a `SessionResumed` event; `resume latest` targets the most recently active session
- README-backed CLI coverage for `resume <id> "review summary"` confirming the resumed turn is appended to the existing persisted session and the output exposes the targeted session id plus the appended turn index
- `harness-session` transcript persistence: save/load round-trip preserves `turn_index` ordering, transcript files are excluded from session listings, and `latest_transcript` follows the most recently updated session
- `harness-runtime` transcript persistence: `bootstrap` writes a transcript file alongside the session, emits a `TranscriptPersisted` event, and `resume` rewrites the transcript so `turn_index` ordering is extended in place
- README-backed CLI coverage for `transcript-show <id>` and `transcript-show latest` confirming the output restates the owning `session_id` and preserves turn ordering

Validation commands:

```bash
cargo test -p harness-core
cargo test -p harness-tools
cargo test -p harness-commands
cargo test -p harness-session
cargo test -p harness-runtime
cargo test -p harness-cli
cargo test
```

More runtime and CLI coverage should continue incrementally through the active issue queue.

The CLI example blocks above are a protected regression surface: if visible seeded output changes, update the README and the `harness-cli` example tests in the same PR.

## Validation Flow

Use the smallest validation command that proves the touched surface, then widen only when the slice needs it:

1. `cargo check` for fast workspace sanity
2. targeted `cargo test -p <crate>` for the crate you changed
3. `cargo run -q -p harness-cli -- <command>` when a CLI-facing slice changes visible behavior or docs
4. `cargo test` before merge when the change crosses crate boundaries or updates shared runtime behavior
5. `cargo clippy --workspace --all-targets -- -D warnings` for code-heavy slices before final merge

For this repository, documentation is part of done. If README examples or command descriptions change, validate them against the actual CLI output before opening the PR.

## Development Workflow

1. create an issue for the slice
2. branch from `main`
3. implement a small atomic unit
4. validate with `cargo check`, `cargo test`, and `cargo clippy --workspace --all-targets -- -D warnings`
5. open a PR
6. merge cleanly

## Public Repo Notes

This repo is a clean-room implementation effort informed by architectural study. It is not an official Anthropic project and is not affiliated with Anthropic.

## Roadmap

- [x] architecture capture
- [x] workspace bootstrap
- [x] core domain types
- [x] session/transcript persistence
- [x] registries
- [x] router/runtime loop
- [x] CLI inspection surface
- [x] CLI usage examples and validation flow
- [x] cleanup of obsolete Python-first scaffolding
- [x] move retained architecture-study snapshots under `archive/reference_data/`
- [x] CLI session resume for persisted sessions (explicit id and `latest`)
- [x] Persisted transcript files per session and CLI transcript inspection (`transcript-show <id>` and `transcript-show latest`)
