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
- CLI commands for summary, route, bootstrap, resume, tools, commands, session listing, session inspection, transcript inspection, session export, session comparison, session deletion, session import, session search, and session fork

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

### `session-export <id>`

Export one persisted session as a single machine-readable JSON bundle that packages the session state and its transcript together. The output uses a deterministic shape: `{ exported_session_id, session, transcript }`, where `session` is the same structure printed by `session-show` and `transcript` is the same structure printed by `transcript-show`. The `exported_session_id` confirms which session was exported and always equals the `session_id` inside both nested records. Turn ordering in `transcript.entries` is preserved in `turn_index` order so the bundle is safe to attach to bug reports or archive outside the repo-local `.sessions/` layout.

```bash
cargo run -q -p harness-cli -- session-export <session-id>
```

```json
{
  "exported_session_id": "<session-id>",
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
}
```

### `session-export latest`

`latest` resolves to the most recently active persisted session, mirroring how `session-show latest` and `transcript-show latest` resolve their targets.

```bash
cargo run -q -p harness-cli -- session-export latest
```

```json
{
  "exported_session_id": "<session-id>",
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
}
```

### `session-compare <left-id> <right-id>`

Compare two persisted sessions side-by-side as a single machine-readable JSON bundle. The output uses a deterministic shape: `{ left_session_id, right_session_id, left, right, differences }`. Both `left` and `right` carry the compared session's `session_id`, recency metadata (`created_at_ms`, `updated_at_ms`), and activity metadata (`message_count`, `transcript_entry_count`). `differences` reports signed deltas computed as `right - left` so the comparison direction is preserved, plus a `same_session` flag that is `true` when both sides resolve to the same persisted session. Either side accepts the literal `latest` selector, mirroring how `session-show latest`, `transcript-show latest`, and `session-export latest` resolve the most recently active persisted session.

```bash
cargo run -q -p harness-cli -- session-compare <left-session-id> <right-session-id>
```

```json
{
  "left_session_id": "<left-session-id>",
  "right_session_id": "<right-session-id>",
  "left": {
    "session_id": "<left-session-id>",
    "created_at_ms": <created-at-ms>,
    "updated_at_ms": <updated-at-ms>,
    "message_count": 1,
    "transcript_entry_count": 1
  },
  "right": {
    "session_id": "<right-session-id>",
    "created_at_ms": <created-at-ms>,
    "updated_at_ms": <updated-at-ms>,
    "message_count": 1,
    "transcript_entry_count": 1
  },
  "differences": {
    "same_session": false,
    "created_at_ms_delta": <created-at-ms-delta>,
    "updated_at_ms_delta": <updated-at-ms-delta>,
    "message_count_delta": 0,
    "transcript_entry_count_delta": 0
  }
}
```

### `session-compare latest latest`

Resolving both sides to `latest` yields a deterministic self-comparison where `same_session` is `true` and every delta is `0`. This is the smallest way to confirm the comparison path is healthy without needing two distinct persisted sessions in hand.

```bash
cargo run -q -p harness-cli -- session-compare latest latest
```

```json
{
  "left_session_id": "<session-id>",
  "right_session_id": "<session-id>",
  "left": {
    "session_id": "<session-id>",
    "created_at_ms": <created-at-ms>,
    "updated_at_ms": <updated-at-ms>,
    "message_count": 1,
    "transcript_entry_count": 1
  },
  "right": {
    "session_id": "<session-id>",
    "created_at_ms": <created-at-ms>,
    "updated_at_ms": <updated-at-ms>,
    "message_count": 1,
    "transcript_entry_count": 1
  },
  "differences": {
    "same_session": true,
    "created_at_ms_delta": 0,
    "updated_at_ms_delta": 0,
    "message_count_delta": 0,
    "transcript_entry_count_delta": 0
  }
}
```

### `session-delete <id>`

Remove one persisted session cleanly. Deletion takes both persisted artifacts for the session in one call: the session JSON (`.sessions/<session-id>.json`) and its sibling transcript JSON (`.sessions/<session-id>.transcript.json`). The output uses a deterministic shape: `{ deleted_session_id, removed_paths }`, where `deleted_session_id` confirms which session was targeted and `removed_paths` lists the files that were actually removed in the order the store removed them (session JSON first, then transcript). If the target session does not exist the command fails without deleting anything else.

```bash
cargo run -q -p harness-cli -- session-delete <session-id>
```

```json
{
  "deleted_session_id": "<session-id>",
  "removed_paths": [
    ".sessions/<session-id>.json",
    ".sessions/<session-id>.transcript.json"
  ]
}
```

### `session-delete latest`

`latest` resolves to the most recently active persisted session, mirroring how `session-show latest`, `transcript-show latest`, `session-export latest`, and `session-compare latest latest` resolve their targets. This is the ergonomic way to drop the session you just created without having to copy its id by hand.

```bash
cargo run -q -p harness-cli -- session-delete latest
```

```json
{
  "deleted_session_id": "<session-id>",
  "removed_paths": [
    ".sessions/<session-id>.json",
    ".sessions/<session-id>.transcript.json"
  ]
}
```

### `session-import <bundle-path>`

Restore a persisted session from a bundle file previously emitted by `session-export`. The input must match the exported shape `{ exported_session_id, session, transcript }`: the three ids must agree, and transcript `turn_index` values must be monotonic starting at `0`. On success both persisted artifacts are recreated in the local `.sessions/` directory — the session JSON at `.sessions/<session-id>.json` and the sibling transcript JSON at `.sessions/<session-id>.transcript.json` — preserving the imported session id, recency/activity metadata, and `turn_index` ordering exactly as carried in the bundle. If the bundle is malformed or the target session id already exists locally, the command fails cleanly without overwriting unrelated persisted sessions.

```bash
cargo run -q -p harness-cli -- session-import ./bundle.json
```

```json
{
  "imported_session_id": "<session-id>",
  "session_path": ".sessions/<session-id>.json",
  "transcript_path": ".sessions/<session-id>.transcript.json"
}
```

### `session-find <query>`

Search persisted local sessions by transcript prompt text without mutating any session state. The query is matched case-insensitively as a substring against each persisted transcript entry. The output is a deterministic JSON array of result objects, one per session that contains at least one matching transcript entry, ordered using the same newest-first session ordering as `sessions` (most recently updated session first, then by created-at, then by session id). Each result identifies the matched `session_id` and includes the session's recency metadata (`created_at_ms`, `updated_at_ms`), `message_count`, `persisted_path`, and a `matches` array. Each entry in `matches` records the matched `turn_index` plus the persisted `prompt` text, so the result is useful from the terminal without a follow-up `transcript-show` call. An empty query and a query with no matches both succeed cleanly with an empty array (`[]`) instead of erroring.

```bash
cargo run -q -p harness-cli -- session-find review
```

```json
[
  {
    "session_id": "<session-id>",
    "created_at_ms": <created-at-ms>,
    "updated_at_ms": <updated-at-ms>,
    "message_count": 1,
    "persisted_path": ".sessions/<session-id>.json",
    "matches": [
      {
        "turn_index": 0,
        "prompt": "review bash"
      }
    ]
  }
]
```

### `session-find <unmatched-query>`

An empty query, or a query that matches no persisted transcript entries, returns an empty JSON array instead of erroring. The example below uses a query that no persisted transcript contains, so the output is the deterministic empty result `[]`.

```bash
cargo run -q -p harness-cli -- session-find definitely-not-present
```

```json
[]
```

### `session-fork <source-session-id> "try again"`

Fork a persisted session so a new line of work can diverge from an existing turn without mutating the source. The fork creates a fresh `session_id`, carries forward the source session's messages and transcript in order, and appends the new prompt as the first divergent turn. Both persisted artifacts are written for the forked session — the session JSON at `.sessions/<forked-session-id>.json` and the sibling transcript JSON at `.sessions/<forked-session-id>.transcript.json` — while the source session JSON and transcript are left exactly as they were. The output uses a deterministic shape: `{ source_session_id, forked_session_id, appended_turn_index, session_path, transcript_path }`. `source_session_id` confirms which session the fork diverged from, `forked_session_id` is the new persisted id, and `appended_turn_index` marks where the new prompt landed in the forked transcript (equal to the source's message count).

```bash
cargo run -q -p harness-cli -- session-fork <source-session-id> "try again"
```

```json
{
  "source_session_id": "<source-session-id>",
  "forked_session_id": "<forked-session-id>",
  "appended_turn_index": 1,
  "session_path": ".sessions/<forked-session-id>.json",
  "transcript_path": ".sessions/<forked-session-id>.transcript.json"
}
```

### `session-fork latest "try again"`

`latest` resolves to the most recently active persisted session, mirroring how `session-show latest`, `transcript-show latest`, `session-export latest`, `session-compare latest latest`, and `session-delete latest` resolve their targets. This is the ergonomic way to branch off the session you just worked on without having to copy its id by hand.

```bash
cargo run -q -p harness-cli -- session-fork latest "try again"
```

```json
{
  "source_session_id": "<source-session-id>",
  "forked_session_id": "<forked-session-id>",
  "appended_turn_index": 1,
  "session_path": ".sessions/<forked-session-id>.json",
  "transcript_path": ".sessions/<forked-session-id>.transcript.json"
}
```

### `session-rename <id> <label>`

Attach a deterministic human-readable label to a persisted session so it is easier to recognize in `sessions`, `session-show`, `session-export`, and related output. The rename preserves the existing `session_id`, does not mutate transcript entries or transcript ordering, and does not bump `updated_at_ms` so newest-first ordering stays activity-based. Labels are trimmed of surrounding whitespace, and empty or whitespace-only labels are rejected cleanly. The output uses a deterministic shape: `{ renamed_session_id, applied_label }`, where `renamed_session_id` confirms which session was targeted and `applied_label` is the normalized label that was persisted. Older unlabeled sessions stay readable — the label field only appears in persisted JSON after a session has actually been labeled.

```bash
cargo run -q -p harness-cli -- session-rename <session-id> runtime-review
```

```json
{
  "renamed_session_id": "<session-id>",
  "applied_label": "runtime-review"
}
```

### `session-rename latest <label>`

`latest` resolves to the most recently active persisted session, mirroring how `session-show latest`, `transcript-show latest`, `session-export latest`, `session-compare latest latest`, `session-delete latest`, and `session-fork latest` resolve their targets. This is the ergonomic way to label the session you just worked on without having to copy its id by hand.

```bash
cargo run -q -p harness-cli -- session-rename latest runtime-review
```

```json
{
  "renamed_session_id": "<session-id>",
  "applied_label": "runtime-review"
}
```

### Label selectors (`label:<name>`)

Once a session has been renamed via `session-rename`, the `label:<name>` selector targets that session anywhere a single persisted session id is accepted: `session-show`, `transcript-show`, `resume`, `session-export`, `session-delete`, `session-fork`, `session-rename`, and on either side of `session-compare`. Raw session ids and the `latest` selector continue to behave exactly as before — label support is additive.

```bash
cargo run -q -p harness-cli -- session-show label:runtime-review
cargo run -q -p harness-cli -- session-compare label:runtime-review latest
```

Selector resolution rules:

- `latest` — most recently active persisted session, ordering driven by `updated_at_ms` (unchanged)
- `label:<name>` — the unique persisted session whose normalized label equals `<name>` (whitespace around `<name>` is trimmed)
- anything else — treated as a raw session id and looked up directly

Failure modes are deterministic and distinct so the CLI surfaces the right diagnosis:

- unknown label (no persisted session carries `<name>`) → `session not found: label:<name>`
- ambiguous label (more than one persisted session shares `<name>`) → `ambiguous session label: label "<name>" matches N persisted sessions`
- malformed selector (`label:` with no name) → `malformed session selector: label selector requires a non-empty label after \`label:\``

Machine-readable JSON outputs continue to identify the actual resolved `session_id` values rather than echoing the selector string, so downstream tooling can rely on the resolved id even when the user typed `label:<name>` or `latest`. Older unlabeled sessions and mixed labeled/unlabeled stores keep working — sessions without a label are transparently skipped during label resolution.

### `session-unlabel <id>`

Remove the persisted label from a session without touching its `session_id`, transcript entries, transcript ordering, or `updated_at_ms` — newest-first ordering stays activity-based. The output uses a deterministic shape: `{ unlabeled_session_id, removed_label }`, where `unlabeled_session_id` confirms which session was targeted and `removed_label` is the label that was cleared. Older unlabeled sessions remain backward-compatible: once a label is removed, the session no longer serializes a `label` field at all (no `null`, no empty string). Attempting to unlabel a session that is already unlabeled fails cleanly with `session already unlabeled: <session-id>` so the operation never silently no-ops, and unknown session ids or selectors still surface as `session not found`.

```bash
cargo run -q -p harness-cli -- session-unlabel <session-id>
```

```json
{
  "unlabeled_session_id": "<session-id>",
  "removed_label": "runtime-review"
}
```

### `session-unlabel latest`

`latest` resolves to the most recently active persisted session, and `label:<name>` is accepted here too, mirroring every other single-session command. This closes the label-management loop alongside `session-rename`, `session-labels`, and `label:<name>` selectors: rename a session, discover labels, target by label, and remove a label when it is no longer useful — all without disturbing transcript history.

```bash
cargo run -q -p harness-cli -- session-unlabel latest
cargo run -q -p harness-cli -- session-unlabel label:runtime-review
```

```json
{
  "unlabeled_session_id": "<session-id>",
  "removed_label": "runtime-review"
}
```

### `session-labels`

List every persisted session that currently carries a label, without touching session state or transcripts. Output is a deterministic JSON array ordered using the same newest-first ordering as `sessions` (most recently updated first, then by `created_at_ms`, then by `session_id`, then by `persisted_path`). Each entry exposes `label`, `session_id`, `created_at_ms`, `updated_at_ms`, `message_count`, and `persisted_path` so the listing is useful from the terminal without a follow-up `session-show`. Unlabeled sessions are omitted. Duplicate labels stay visible as separate rows — the listing makes ambiguity discoverable before a `label:<name>` selector would fail with `AmbiguousLabel`.

```bash
cargo run -q -p harness-cli -- session-labels
```

```json
[
  {
    "label": "runtime-review",
    "session_id": "<session-id>",
    "created_at_ms": <created-at-ms>,
    "updated_at_ms": <updated-at-ms>,
    "message_count": 1,
    "persisted_path": ".sessions/<session-id>.json"
  }
]
```

### `session-labels <empty-store>`

If no persisted session carries a label, `session-labels` returns an empty JSON array instead of erroring, so scripts can treat "no labels" and "none yet" identically.

```bash
cargo run -q -p harness-cli -- session-labels
```

```json
[]
```

### `session-retag <id> <label>`

Atomically replace the persisted label on a session that already carries one, in a single step instead of chaining `session-unlabel` with `session-rename`. The retag preserves the existing `session_id`, does not mutate transcript entries or transcript ordering, and does not bump `updated_at_ms` so newest-first ordering stays activity-based. Labels are trimmed of surrounding whitespace, and empty or whitespace-only labels are rejected cleanly. If the requested label normalizes to the same effective value already persisted on the session, the command fails with `session already labeled: ...` rather than silently no-opping. The output uses a deterministic shape: `{ retagged_session_id, previous_label, applied_label }`, where `retagged_session_id` confirms which session was targeted, `previous_label` is the label that was replaced, and `applied_label` is the normalized label that now sits on the session. Older unlabeled sessions remain readable — the label field only appears in persisted JSON after a session has actually been labeled.

```bash
cargo run -q -p harness-cli -- session-retag <session-id> release-candidate
```

```json
{
  "retagged_session_id": "<session-id>",
  "previous_label": "runtime-review",
  "applied_label": "release-candidate"
}
```

### `session-retag latest <label>`

`latest` resolves to the most recently active persisted session, and `label:<old-name>` is accepted here too, mirroring every other single-session command. This makes `session-retag label:<old-name> <new-name>` the ergonomic single-step relabel: find the session by its current label, apply the new one, and keep transcript history untouched.

```bash
cargo run -q -p harness-cli -- session-retag latest release-candidate
cargo run -q -p harness-cli -- session-retag label:runtime-review release-candidate
```

```json
{
  "retagged_session_id": "<session-id>",
  "previous_label": "runtime-review",
  "applied_label": "release-candidate"
}
```

### `session-prune --keep <count>`

Bulk-remove older persisted sessions without touching the newest `<count>` *prune-eligible* (unpinned) sessions. Pinned sessions are always preserved and are reported under `pinned_preserved_count` / `pinned_preserved` regardless of `<count>` — see [`session-pin <id>`](#session-pin-id). Ordering matches `sessions` and `session-labels` (most recently updated first, then `created_at_ms`, then `session_id`, then `persisted_path`), applied only across the unpinned subset, so the "newest N" preserved set is the same one every other command surfaces after excluding pinned sessions. For each pruned session, both persisted artifacts are removed together: the `.sessions/<session-id>.json` file and the sibling `.sessions/<session-id>.transcript.json`. Preserved sessions are never mutated — their label, pinned flag, transcript entries, transcript ordering, and activity metadata stay exactly as they were. The output uses a deterministic shape: `{ kept_count, pruned_count, pinned_preserved_count, removed, pinned_preserved }`, where `removed` is a JSON array — one entry per pruned session — identifying the pruned `session_id` together with the removed `session_path` and `transcript_path`, and `pinned_preserved` is a JSON array of `session_id` values for every pinned session that was held back from pruning. If the store already contains `<count>` or fewer unpinned sessions the call succeeds cleanly with `removed: []`. `--keep 0` is supported and prunes every unpinned persisted session.

```bash
cargo run -q -p harness-cli -- session-prune --keep 1
```

```json
{
  "kept_count": 1,
  "pruned_count": 1,
  "pinned_preserved_count": 0,
  "removed": [
    {
      "session_id": "<pruned-session-id>",
      "session_path": ".sessions/<pruned-session-id>.json",
      "transcript_path": ".sessions/<pruned-session-id>.transcript.json"
    }
  ],
  "pinned_preserved": []
}
```

### `session-prune <no-op>`

When the store already contains `<count>` or fewer unpinned persisted sessions, `session-prune` returns a deterministic empty `removed` array instead of erroring, so scripts can treat "already within the retention budget" and "just ran a prune" identically. Pinned sessions do not count against the retention budget and surface through `pinned_preserved_count` / `pinned_preserved`.

```bash
cargo run -q -p harness-cli -- session-prune --keep 10
```

```json
{
  "kept_count": 1,
  "pruned_count": 0,
  "pinned_preserved_count": 0,
  "removed": [],
  "pinned_preserved": []
}
```

### `session-pin <id>`

Mark a persisted session as pinned so it is permanently excluded from `session-prune`'s retention-based removal regardless of the `--keep` budget. Pin preserves the existing `session_id`, does not mutate transcript entries or transcript ordering, and does not bump `updated_at_ms` so newest-first ordering stays activity-based. Messages, usage, and labels are untouched. The output uses a deterministic shape: `{ pinned_session_id, pinned }`, where `pinned_session_id` confirms which session was targeted and `pinned` is `true` on success. Older unpinned sessions stay byte-compatible: the `pinned` field is only serialized into persisted JSON after a session has actually been pinned. Attempting to pin a session that is already pinned fails cleanly with `session already pinned: <session-id>` so the operation never silently no-ops, and unknown session ids or selectors still surface as `session not found`.

```bash
cargo run -q -p harness-cli -- session-pin <session-id>
```

```json
{
  "pinned_session_id": "<session-id>",
  "pinned": true
}
```

### `session-pin latest` / `session-pin label:<name>`

`latest` resolves to the most recently active persisted session, and `label:<name>` is accepted here too, mirroring every other single-session command via the shared selector path. `session-pin` pairs with [`session-prune`](#session-prune---keep-count) so the sessions you care about can be pinned once and then stay safe from any future prune invocation.

```bash
cargo run -q -p harness-cli -- session-pin latest
cargo run -q -p harness-cli -- session-pin label:runtime-review
```

```json
{
  "pinned_session_id": "<session-id>",
  "pinned": true
}
```

### `session-unpin <id>`

Clear the pinned flag on a persisted session so it becomes eligible for `session-prune` again. Unpin preserves the existing `session_id`, does not mutate transcript entries or transcript ordering, and does not bump `updated_at_ms` so newest-first ordering stays activity-based. Messages, usage, and labels are untouched. The output uses a deterministic shape: `{ unpinned_session_id, pinned }`, where `unpinned_session_id` confirms which session was targeted and `pinned` is `false` on success. Older unpinned sessions stay backward-compatible: once the pin is cleared, the session no longer serializes a `pinned` field at all (no `null`, no `false`). Attempting to unpin a session that is not pinned fails cleanly with `session already unpinned: <session-id>` so the operation never silently no-ops, and unknown session ids or selectors still surface as `session not found`.

```bash
cargo run -q -p harness-cli -- session-unpin <session-id>
```

```json
{
  "unpinned_session_id": "<session-id>",
  "pinned": false
}
```

### `session-unpin latest` / `session-unpin label:<name>`

`latest` resolves to the most recently active persisted session, and `label:<name>` is accepted here too, mirroring every other single-session command. This closes the pin-management loop alongside `session-pin` and `session-prune`: pin the sessions you want to keep, prune the rest on a budget, and unpin anything that no longer needs that protection — all without disturbing transcript history.

```bash
cargo run -q -p harness-cli -- session-unpin latest
cargo run -q -p harness-cli -- session-unpin label:runtime-review
```

```json
{
  "unpinned_session_id": "<session-id>",
  "pinned": false
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
- `harness-session` `SessionExport` bundle round-trip: packages session state plus transcript, confirms the exported session id, and preserves `turn_index` ordering in the exported transcript with deterministic serialization
- `harness-runtime` `export_session` behavior: bundles the persisted session and its transcript for an explicit id, and `latest` resolves to the same bundle
- README-backed CLI coverage for `session-export <id>` and `session-export latest` confirming the output exposes the `exported_session_id` and preserves turn ordering
- `harness-session` `SessionComparison` bundle: pairs two sides with shared recency/activity metadata, reports signed `right - left` deltas (including negative deltas when order is reversed), exposes a `same_session` flag, and serializes deterministically
- `harness-runtime` `compare_sessions` behavior: resolves explicit ids and the `latest` selector on either side, computes deltas against persisted session state plus transcripts, and treats a self-comparison as `same_session: true` with zero deltas
- README-backed CLI coverage for `session-compare <left-id> <right-id>` and `session-compare latest latest` confirming the output identifies both compared session ids and that a `latest latest` self-comparison reports `same_session: true` with every delta equal to `0`
- `harness-session` `SessionStore::delete` behavior: removes both the session JSON and its sibling transcript JSON, reports the removed paths in deterministic order, and fails with `SessionNotFound` without touching sibling sessions when the target does not exist
- `harness-runtime` `delete_session` behavior: resolves the `latest` selector to the most recently active persisted session, removes both persisted artifacts for that session, and leaves untouched sessions intact
- README-backed CLI coverage for `session-delete <id>` and `session-delete latest` confirming the output identifies the deleted session id, lists the removed paths in `session.json` then `transcript.json` order, and that the session disappears from subsequent listings
- `harness-session` `SessionStore::import_bundle` behavior: validates that the bundle's `exported_session_id`, nested `session.session_id`, and nested `transcript.session_id` all agree, rejects bundles whose transcript `turn_index` values are non-monotonic, refuses to overwrite an existing persisted session id, and on success writes both the session JSON and its sibling transcript JSON preserving the imported session id, recency/activity metadata, and turn ordering exactly as carried in the bundle
- `harness-runtime` `import_session` behavior: reads a persisted bundle from a user-supplied path, round-trips a `session-export` bundle into a fresh store, reports the imported session id plus the written session and transcript paths, and fails cleanly when the bundle path is missing or the target session id already exists locally
- README-backed CLI coverage for `session-import <bundle-path>` confirming the output identifies the imported session id and the written session and transcript paths, and that a duplicate import against the same store fails cleanly without touching the already-imported session
- `harness-session` `SessionStore::find` behavior: matches persisted transcript prompt text case-insensitively, orders results using the existing newest-first session ordering, preserves `turn_index` ordering inside each result's `matches` array, and returns an empty result set for both an empty query and a query with no matches without mutating any persisted session state
- `harness-runtime` `find_sessions` behavior: surfaces matches across bootstrap and resume-appended turns for an explicit query, scopes to sessions whose transcripts contain the query, and treats both unmatched queries and the empty query as a clean empty result set
- README-backed CLI coverage for `session-find <query>` confirming a positive search reports the matched session id with `turn_index`-ordered `matches`, and that a query with no matches produces a deterministic empty JSON array
- `harness-session` `SessionStore::fork` behavior: creates a fresh `session_id` rather than mutating the source, copies source messages and transcript entries forward in turn-index order, appends the new prompt as the first divergent turn, persists both the forked session JSON and its sibling transcript JSON, leaves the source session JSON and transcript exactly as they were, and reports `SessionNotFound` cleanly when the source id does not exist
- `harness-runtime` `fork_session` behavior: resolves the `latest` selector to the most recently active persisted session, delegates to the store to write the forked session plus transcript, and fails cleanly for a missing source id without leaving partial persisted artifacts behind
- README-backed CLI coverage for `session-fork <source-session-id> "try again"` and `session-fork latest "try again"` confirming the output identifies both the source and forked session ids, exposes the `appended_turn_index`, and reports the written session and transcript paths while the source session and transcript remain unchanged
- `harness-session` `SessionStore::rename` behavior: trims surrounding whitespace on the label, rejects empty and whitespace-only labels with `InvalidLabel`, reports `SessionNotFound` cleanly when the target session does not exist, preserves the existing `session_id`, does not mutate transcript entries or ordering, and does not bump `updated_at_ms` so newest-first ordering stays activity-based; persisted JSON for unlabeled sessions remains identical in shape (no `label` field is emitted) so older sessions stay readable
- `harness-runtime` `rename_session` behavior: resolves explicit session ids and the `latest` selector to the most recently active persisted session, delegates to the store to persist the normalized label, and fails cleanly with a descriptive error for invalid labels and unknown session ids without mutating any other persisted state
- README-backed CLI coverage for `session-rename <id> <label>` and `session-rename latest <label>` confirming the output identifies the targeted session id and the applied label, that the rename leaves transcript entries and ordering untouched, and that unknown session ids and empty/whitespace-only labels fail cleanly
- `harness-session` `SessionSelector` parsing and `SessionStore::resolve_selector` behavior: dispatches `latest`, `label:<name>`, and raw id forms against persisted state, with unknown labels reported as `SessionNotFound("label:<name>")`, duplicate labels reported as `AmbiguousLabel`, and an empty `label:` reported as `MalformedSelector`; sessions without a label are transparently skipped so mixed labeled/unlabeled stores keep working
- `harness-runtime` label selector behavior: `load_session`, `load_transcript`, `delete_session`, `export_session`, `compare_sessions`, `fork_session`, and `rename_session` all accept `label:<name>` wherever they previously accepted an explicit persisted session id, with raw ids and `latest` unchanged; machine-readable outputs continue to identify the actual resolved `session_id` values rather than the selector string
- README-backed CLI coverage for label-based single-session targeting (`session-show label:<name>`) confirming raw-id targeting still works unchanged after a label is applied, the resolved `session_id` (not the label string) appears in the JSON output, and for `session-compare label:<name> latest` confirming the mixed label-plus-latest path resolves both sides to the correct persisted session ids; unknown, ambiguous, and malformed label selectors fail cleanly with distinct diagnostics
- `harness-session` `SessionStore::list_labels` behavior: emits one entry per labeled persisted session, uses the same newest-first ordering as `list()`, omits unlabeled sessions, keeps duplicate labels visible as separate rows so ambiguity is discoverable, returns a clean empty vector when no persisted session carries a label, and never mutates persisted state
- `harness-runtime` `list_session_labels` behavior: delegates to the store so the CLI surface shares ordering and omission semantics with `list_labels`, and surfaces an empty listing cleanly when no persisted session is labeled
- README-backed CLI coverage for `session-labels` and `session-labels <empty-store>` confirming the listing is newest-first, exposes `label`, `session_id`, recency metadata, `message_count`, and `persisted_path`, omits unlabeled sessions, keeps duplicate labels as separate rows, and returns a deterministic empty JSON array when no persisted session is labeled
- `harness-session` `SessionStore::unlabel` behavior: clears the persisted `label` field while preserving the existing `session_id`, `created_at_ms`, `updated_at_ms`, messages, usage, and transcript entries/ordering, reports `SessionAlreadyUnlabeled` cleanly for a session that carries no label, reports `SessionNotFound` for missing ids, and keeps persisted JSON free of a `null`/empty `label` field so older unlabeled sessions stay backward-compatible
- `harness-runtime` `unlabel_session` behavior: accepts explicit ids, the `latest` selector, and `label:<name>` (via the shared `resolve_selector` path), delegates to the store, and surfaces unknown selectors and already-unlabeled sessions as distinct, descriptive errors without mutating any other persisted state
- README-backed CLI coverage for `session-unlabel <id>`, `session-unlabel latest`, and `session-unlabel label:<name>` confirming the output identifies the resolved `unlabeled_session_id` and the `removed_label`, that the unlabel leaves transcript entries and ordering untouched, that `updated_at_ms` is not bumped, that the unlabeled session disappears from `session-labels` while transcript/session content stays unchanged, and that an already-unlabeled session fails cleanly without touching persisted state
- `harness-session` `SessionStore::retag` behavior: trims surrounding whitespace on the new label, rejects empty and whitespace-only labels with `InvalidLabel`, preserves the existing `session_id` and does not mutate transcript entries, transcript ordering, messages, or `updated_at_ms`, surfaces `SessionAlreadyLabeled` when the requested label normalizes to the same effective value already persisted, surfaces `SessionAlreadyUnlabeled` when the target session has no label to replace, and surfaces `SessionNotFound` cleanly for unknown session ids
- `harness-runtime` `retag_session` behavior: accepts explicit ids, the `latest` selector, and `label:<name>` (via the shared `resolve_selector` path), delegates to the store, and surfaces unknown selectors, already-unlabeled sessions, and same-effective-label attempts as distinct, descriptive errors without mutating any other persisted state
- README-backed CLI coverage for `session-retag <id> <label>`, `session-retag latest <label>`, and `session-retag label:<old-name> <new-name>` confirming the output identifies the resolved `retagged_session_id`, the `previous_label`, and the `applied_label`, that the retag leaves transcript entries and ordering untouched, that `updated_at_ms` is not bumped, that `session-labels` reflects the new label while transcript/session content and ordering stay unchanged, and that a same-effective-label request fails cleanly without touching persisted state
- `harness-session` `SessionStore::prune` behavior: preserves the newest `<keep>` *prune-eligible (unpinned)* persisted sessions using the same newest-first ordering as `list()` (`updated_at_ms` → `created_at_ms` → `session_id` → `persisted_path`) applied only across unpinned sessions, removes both persisted artifacts (`.sessions/<id>.json` and `.sessions/<id>.transcript.json`) together for every older unpinned session, reports `kept_count`, `pruned_count`, `pinned_preserved_count`, a deterministic `removed` array identifying each pruned `session_id` together with the removed session and transcript paths, and a deterministic `pinned_preserved` array listing every pinned session that was held back, leaves preserved sessions' labels, pinned flag, transcript entries, transcript ordering, and activity metadata untouched, supports `--keep 0` to prune every unpinned persisted session, returns a clean empty `removed` listing when the store already contains `<= keep` unpinned sessions, and returns a clean empty listing for a missing root directory
- `harness-runtime` `prune_sessions` behavior: delegates to the store so the CLI surface shares ordering, removal semantics, pinned-preservation, and deterministic output with `SessionStore::prune`, and continues to surface preserved sessions newest-first through `list_sessions` after a prune
- README-backed CLI coverage for `session-prune --keep <count>` and `session-prune <no-op>` confirming the output exposes `kept_count`, `pruned_count`, `pinned_preserved_count`, a `removed` array with `session_id`, `session_path`, and `transcript_path` per pruned entry, and a `pinned_preserved` array of rescued session ids, preserves the newest `<count>` unpinned sessions in the subsequent `sessions` listing, removes both persisted artifacts for every older unpinned session, and returns a deterministic empty `removed` array when the store already contains `<= count` unpinned persisted sessions
- `harness-session` `SessionStore::pin` / `SessionStore::unpin` behavior: sets / clears the persisted `pinned` flag while preserving the existing `session_id`, `created_at_ms`, `updated_at_ms`, messages, usage, label, and transcript entries/ordering; reports `SessionAlreadyPinned` / `SessionAlreadyUnpinned` cleanly when the operation would be a no-op, reports `SessionNotFound` for missing ids, and keeps persisted JSON free of a `pinned: false` field so older unpinned sessions stay byte-compatible
- `harness-runtime` `pin_session` / `unpin_session` behavior: accepts explicit ids, the `latest` selector, and `label:<name>` via the shared `resolve_selector` path, delegates to the store, and surfaces unknown selectors / already-pinned / already-unpinned states as distinct, descriptive errors without mutating any other persisted state; pinned sessions survive `prune_sessions` regardless of `<keep>` and are reported via `pinned_preserved_count` / `pinned_preserved`
- README-backed CLI coverage for `session-pin <id>`, `session-pin latest`, `session-pin label:<name>`, `session-unpin <id>`, `session-unpin latest`, and `session-unpin label:<name>` confirming the output identifies the resolved `pinned_session_id` / `unpinned_session_id` and the resulting pinned state, that pin/unpin leave transcript entries and ordering untouched, that `updated_at_ms` is not bumped (newest-first ordering stays activity-based), that the persisted JSON carries `pinned: true` only while pinned and omits the field entirely after unpin, and CLI coverage for `session-prune --keep <count>` with a pinned session confirming the pinned session is excluded from pruning and surfaces via `pinned_preserved` while other older unpinned sessions are still removed deterministically

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
- [x] CLI session export for persisted session bundles (`session-export <id>` and `session-export latest`) in a deterministic JSON shape packaging session state plus transcript
- [x] CLI session comparison for persisted sessions (`session-compare <left-id> <right-id>` with `latest` accepted on either side) in a deterministic JSON shape that identifies both compared session ids and reports signed deltas for recency metadata and transcript/turn counts
- [x] CLI session deletion for persisted sessions (`session-delete <id>` and `session-delete latest`) that removes both the session JSON and its sibling transcript JSON in one call, with deterministic JSON output identifying the deleted session id plus the removed paths, and a clean failure when the target session does not exist
- [x] CLI session import for persisted session bundles (`session-import <bundle-path>`) that accepts the deterministic `{ exported_session_id, session, transcript }` shape emitted by `session-export`, recreates both persisted artifacts preserving the imported session id, recency/activity metadata, and transcript `turn_index` ordering, and fails cleanly without overwriting unrelated persisted sessions when the bundle is invalid or the target session id already exists locally
- [x] CLI session search for persisted transcripts (`session-find <query>`) that case-insensitively matches transcript prompt text without mutating session state, returns a deterministic JSON array ordered using the existing newest-first session ordering, identifies each matched `session_id` with recency/activity metadata plus a `matches` array of `{ turn_index, prompt }` so results are useful from the terminal, and treats both empty queries and queries with no matches as a clean empty array instead of an error
- [x] CLI session fork for persisted sessions (`session-fork <id> <prompt>` and `session-fork latest <prompt>`) that creates a fresh persisted session id rather than mutating the source, carries the source session messages and transcript forward in turn-index order, appends the new prompt as the first divergent turn, writes both forked persisted artifacts (`.sessions/<forked-session-id>.json` and its sibling transcript JSON), and emits a deterministic `{ source_session_id, forked_session_id, appended_turn_index, session_path, transcript_path }` shape while leaving the source session and transcript unchanged
- [x] CLI session rename for persisted sessions (`session-rename <id> <label>` and `session-rename latest <label>`) that attaches a trimmed, non-empty human-readable label to persisted session metadata while preserving the existing `session_id`, leaving transcript entries and ordering untouched, and not bumping `updated_at_ms` so newest-first ordering stays activity-based; emits a deterministic `{ renamed_session_id, applied_label }` shape, fails cleanly for unknown sessions and empty/whitespace-only labels, and keeps older unlabeled sessions readable by only emitting the label field once a session has actually been labeled
- [x] CLI session-labels for persisted sessions (`session-labels`) that lists every persisted session carrying a label without mutating session state, emits a deterministic JSON array ordered using the existing newest-first persisted-session ordering, exposes `label`, `session_id`, recency metadata (`created_at_ms`, `updated_at_ms`), `message_count`, and `persisted_path` per entry, omits unlabeled sessions, keeps duplicate labels visible as separate rows so ambiguity is discoverable before a `label:<name>` selector would fail, and returns a clean empty JSON array when no persisted session carries a label
- [x] CLI label selectors (`label:<name>`) for persisted sessions accepted anywhere a single persisted session id is accepted (`session-show`, `transcript-show`, `resume`, `session-export`, `session-delete`, `session-fork`, `session-rename`, and either side of `session-compare`); raw session ids and `latest` keep their existing behavior, machine-readable JSON outputs continue to surface the actual resolved `session_id`, and unknown labels, ambiguous labels (more than one persisted session sharing the same label), and malformed selectors (`label:` with no name) all fail cleanly with distinct diagnostics; activity-based newest-first ordering is unchanged and mixed labeled/unlabeled stores stay backward-compatible
- [x] CLI session-unlabel for persisted sessions (`session-unlabel <id>`, `session-unlabel latest`, and `session-unlabel label:<name>`) that removes only the persisted `label` metadata field while preserving the existing `session_id`, leaving transcript entries and ordering untouched, and not bumping `updated_at_ms` so newest-first ordering stays activity-based; emits a deterministic `{ unlabeled_session_id, removed_label }` shape, fails cleanly for unknown sessions/selectors and for attempts to unlabel a session that is already unlabeled, and keeps older unlabeled sessions backward-compatible by not serializing a null/empty label field after removal
- [x] CLI session-retag for persisted sessions (`session-retag <id> <label>`, `session-retag latest <label>`, and `session-retag label:<old-name> <new-name>`) that atomically replaces the persisted `label` metadata field while preserving the existing `session_id`, leaving transcript entries and ordering untouched, and not bumping `updated_at_ms` so newest-first ordering stays activity-based; emits a deterministic `{ retagged_session_id, previous_label, applied_label }` shape, fails cleanly for unknown sessions/selectors, empty/whitespace-only labels, attempts to retag a session that carries no label, and attempts where the requested label normalizes to the same effective value already present, and keeps older unlabeled sessions backward-compatible by only serializing the label field when present
- [x] CLI session-pin / session-unpin for persisted sessions (`session-pin <id>`, `session-pin latest`, `session-pin label:<name>`, `session-unpin <id>`, `session-unpin latest`, and `session-unpin label:<name>`) that toggle a deterministic `pinned` flag on session metadata while preserving the existing `session_id`, leaving transcript entries and ordering untouched, and not bumping `updated_at_ms` so newest-first ordering stays activity-based; emits deterministic `{ pinned_session_id, pinned }` / `{ unpinned_session_id, pinned }` shapes, keeps older unpinned sessions backward-compatible by only serializing the `pinned` field when the session is actually pinned, surfaces the `pinned` flag through `sessions`, `session-show`, `session-export`, `session-compare`, and `session-labels`, and makes `session-prune --keep <count>` skip pinned sessions — apply newest-first ordering only across the unpinned subset and report rescued pins via a new `pinned_preserved_count` and `pinned_preserved` pair on the prune output
