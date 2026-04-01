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
- CLI commands for summary, route, bootstrap, tools, commands, session listing, and session inspection

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

The examples below reflect the current seeded runtime surface and are protected by `cargo test -p harness-cli`. `bootstrap` creates a session file under `.sessions/`, which is gitignored, so the README uses stable placeholders for generated values that vary per run: `<session-id>` for ids, `<created-at-ms>` for persisted recency metadata, and matching `.sessions/<session-id>.json` paths.

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
    }
  ],
  "persisted_path": ".sessions/<session-id>.json"
}
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
  "messages": [
    "review bash"
  ],
  "usage": {
    "input_tokens": 2,
    "output_tokens": 2
  }
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
- `harness-session` recency metadata, newest-first listing, and latest-session lookup
- README-backed CLI output regression coverage for `summary`, `route <prompt>`, `tools`, `commands`, and `sessions`
- README-backed persisted-session example coverage for `bootstrap <prompt>`, `session-show <id>`, and `session-show latest`, with generated session identifiers normalized to `<session-id>` and generated recency metadata normalized to `<created-at-ms>` in test assertions

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
