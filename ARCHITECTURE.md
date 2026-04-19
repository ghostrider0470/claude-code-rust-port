# Claude Code Rust Port Architecture

## Purpose

This repository exists to study the exposed Claude Code architecture, learn from its harness design, and build an original Rust-native implementation that can later donate real subsystems into Horizon/Rune.

This is **not** a provenance-preserving fork and **not** a file-for-file transliteration exercise. The target is a usable agent runtime with strong types, explicit state, structured events, and reusable boundaries.

## Source Material Stance

We are using the exposed Claude Code codebase as **reference material for architectural learning**. The goals are:

- understand the runtime shape
- identify useful harness primitives
- rebuild those ideas in an original Rust design
- extract reusable pieces for Horizon

We are **not** optimizing for source-level parity, path parity, or direct lineage.

## What We Learn From The Reference

From the current mirrored workspace and snapshot metadata, the exposed system appears to center around several key surfaces:

### 1. Query/session engine
Observed from mirrored files such as:
- `QueryEngine.ts`
- `query.ts`
- `history.ts`
- `projectOnboardingState.ts`
- `session_store`-style Python mirror code

Likely responsibilities:
- turn processing
- session state
- prompt handling
- transcript/history management
- usage accounting
- stopping conditions

### 2. Command system
Observed from:
- `commands.ts`
- 200+ command snapshot entries
- command subdirectories in archived surface metadata

Likely responsibilities:
- command registration
- command lookup
- command-specific handlers
- built-in and extension command surfaces

### 3. Tool system
Observed from:
- `Tool.ts`
- `tools.ts`
- 184 tool snapshot entries
- tool-specific UI and helper modules

Likely responsibilities:
- tool metadata and dispatch
- permission-aware execution
- execution result formatting
- tool-specific state/display helpers

### 4. Runtime/bootstrap flow
Observed from:
- `main.tsx`
- `setup.ts`
- `dialogLaunchers.tsx`
- `replLauncher.tsx`
- `bootstrap/*`
- `remote/*`

Likely responsibilities:
- startup checks
- runtime mode selection
- local vs remote execution flow
- UI/runtime initialization

### 5. Context and permission shaping
Observed from:
- `context.ts`
- `permissions.py` mirror
- `prefetch.py` mirror
- `system_init.py` mirror

Likely responsibilities:
- assembling execution context
- deciding allowed tool behavior
- preparing initial runtime state

### 6. Multi-surface architecture
Archived metadata shows a very broad codebase with directories for:
- assistant
- bridge
- buddy
- cli
- coordinator
- plugins
- services
- skills
- state
- server
- voice
- remote
- upstreamproxy

This suggests the original system is not just a simple REPL; it is a broad harness/runtime with UI, command, tool, remote, and orchestration concerns.

## Rust Port Goal

Build a **Rust-first harness runtime** with clean, typed modules that can eventually donate components into Rune. The first target is not feature completeness. The first target is to correctly establish the core runtime skeleton.

## Design Principles

### Rust-native first
- strong data types
- explicit ownership of state
- serializable session/event models
- deterministic ranking and execution behavior
- clear separation between pure logic and IO

### Runtime over replica
- keep useful concepts
- redesign weak abstractions freely
- avoid fake shims that exist only to mimic structure

### Reusable donor architecture
The runtime should be built so these parts can later be transplanted into Horizon/Rune:
- event model
- transcript/session store
- command registry
- tool registry
- permissions layer
- router
- runtime execution loop

### Inspectable state
- all important actions should emit typed events
- sessions should persist to disk in a readable structured format
- the CLI should be able to inspect runtime state directly

## Proposed Rust Workspace Shape

Initial workspace:

```text
.
├── Cargo.toml
├── crates/
│   ├── harness-core/
│   ├── harness-session/
│   ├── harness-tools/
│   ├── harness-commands/
│   ├── harness-runtime/
│   └── harness-cli/
└── docs/
```

If needed, the first slice can start with fewer crates, but the boundaries should still reflect these domains.

## Core Domain Model

### harness-core
Shared types:
- `SessionId`
- `TurnIndex`
- `Prompt`
- `UsageSummary`
- `RuntimeError`
- `ToolName`
- `CommandName`
- `MatchScore`

### harness-session
Session and transcript concerns:
- `SessionState`
- `SessionListing`
- `TranscriptEntry`
- `TranscriptStore`
- `TranscriptRecord` (on-disk transcript format: `session_id`, `created_at_ms`, `updated_at_ms`, ordered `entries`)
- `SessionExport` (deterministic export bundle: `exported_session_id`, `session`, `transcript`; bundles session state with its transcript for archival, sharing, or debugging; the same shape is the accepted input for `session-import`, closing the loop between export and import)
- `SessionImport` (deterministic import result: `imported_session_id`, `session_path`, `transcript_path`; records that a bundle was round-tripped back into the store and which local files were written)
- `SessionFindResult` (deterministic search result: `session_id`, `created_at_ms`, `updated_at_ms`, `message_count`, `persisted_path`, and a `matches` array of `{ turn_index, prompt }` entries; results are produced by `SessionStore::find` in the existing newest-first session ordering and contain only sessions whose persisted transcripts contain the query)
- `SessionComparison` (deterministic comparison bundle: `left_session_id`, `right_session_id`, `left`, `right`, `differences`; `differences` carries a `same_session` flag plus signed `created_at_ms_delta`, `updated_at_ms_delta`, `message_count_delta`, and `transcript_entry_count_delta` computed as `right - left` so the direction of the comparison is preserved)
- `SessionRename` (deterministic rename result: `renamed_session_id`, `applied_label`; records that a persisted session now carries a trimmed, non-empty human-readable label in its metadata, while leaving the session id, transcript, and activity metadata untouched)
- `SessionUnlabel` (deterministic unlabel result: `unlabeled_session_id`, `removed_label`; records that the persisted label was cleared from a session while the session id, transcript, and activity metadata remain untouched; persisted JSON no longer carries the label field afterwards so backward compatibility with older unlabeled sessions is preserved)
- `SessionSelector` (deterministic CLI selector parser with three forms: `Latest`, `Label(<name>)`, and `Id(<raw>)`; `SessionStore::resolve_selector` and `SessionStore::resolve_label` resolve a parsed selector to a single persisted `session_id` against on-disk state, with `SessionNotFound` for unknown labels, `AmbiguousLabel` when more than one persisted session shares a label, and `MalformedSelector` for `label:` with no name; raw ids return verbatim so callers retain `SessionNotFound` semantics from their own load step)
- optional `label` field on `SessionState` stored only once a session has been renamed, serialized with `skip_serializing_if = "Option::is_none"` so older unlabeled sessions remain byte-compatible with existing persisted JSON and existing inspection output
- `SessionStore`
- recency metadata for persisted sessions (`created_at_ms`) plus activity metadata (`updated_at_ms`) that bumps on resume so `latest` follows the most recently active session
- compaction policy
- disk persistence/load/list/latest/resume-aware ordering
- sibling transcript file per session at `<session-id>.transcript.json`, rewritten on bootstrap and resume; transcript files are excluded from session listings so session and transcript inspection stay independent

### harness-tools
Tool concerns:
- `ToolDefinition`
- `ToolCategory`
- `ToolCapability`
- `ToolRegistry`
- `ToolExecutor` trait
- `PermissionPolicy`
- `ToolResult`

### harness-commands
Command concerns:
- `CommandDefinition`
- `CommandRegistry`
- `CommandExecutor` trait
- `CommandResult`
- lookup/filter/search APIs

### harness-runtime
Execution concerns:
- `Router`
- `RoutedMatch`
- `RuntimeEngine`
- `BootstrapReport`
- `TurnProcessor`
- structured event emission

### harness-cli
User-facing CLI:
- `summary`
- `route <prompt>`
- `bootstrap <prompt>`
- `resume <id> <prompt>` and `resume latest <prompt>` (append a new turn to an existing persisted session; output confirms the targeted session id and appended turn index)
- `tools list`
- `commands list`
- `sessions` (newest-first)
- `session show <id>`
- `session show latest`
- `transcript show <id>` and `transcript show latest` (machine-readable JSON transcript inspection that restates the owning session id and preserves turn ordering)
- `session-export <id>` and `session-export latest` (deterministic JSON export bundle that packages session state plus transcript together; output confirms the exported session id and preserves turn ordering)
- `session-compare <left-id> <right-id>` with `latest` accepted on either side (deterministic JSON comparison bundle that identifies both compared session ids and reports signed deltas for recency/activity metadata and transcript/turn counts)
- `session-delete <id>` and `session-delete latest` (removes both the session JSON and its sibling transcript JSON; deterministic JSON output identifies the deleted session id and the removed paths, and the command fails cleanly without deleting anything else when the target session does not exist)
- `session-import <bundle-path>` (imports a persisted session bundle from a JSON file in the `session-export` shape; recreates both the session JSON and its sibling transcript JSON, preserves the imported session id, recency/activity metadata, and transcript `turn_index` ordering, and fails cleanly without overwriting unrelated persisted sessions when the bundle is invalid or the target session id already exists locally)
- `session-find <query>` (searches persisted local sessions by transcript prompt text without mutating any session state; the query is matched case-insensitively as a substring against each persisted transcript entry; output is a deterministic JSON array of result objects, ordered using the same newest-first session ordering as `sessions`, where each result identifies the matched `session_id` plus recency/activity metadata and a `matches` array of `{ turn_index, prompt }` entries; an empty query and a query with no matches both succeed cleanly with an empty array)
- `session-rename <id> <label>` and `session-rename latest <label>` (attaches a trimmed, non-empty human-readable label to persisted session metadata while preserving the existing `session_id`; the rename does not mutate transcript entries or transcript ordering and does not bump `updated_at_ms` so newest-first ordering stays activity-based; output is a deterministic `{ renamed_session_id, applied_label }` JSON shape, and empty/whitespace-only labels and unknown session ids fail cleanly)
- `session-labels` (lists every persisted session that currently carries a label without mutating any session state; output is a deterministic JSON array ordered using the same newest-first persisted-session ordering as `sessions`, and each entry exposes `label`, `session_id`, `created_at_ms`, `updated_at_ms`, `message_count`, and `persisted_path`; unlabeled sessions are omitted, duplicate labels remain visible as separate rows so ambiguity is discoverable before a `label:<name>` selector would fail, and an empty labeled store returns a clean empty JSON array)
- `session-unlabel <id>`, `session-unlabel latest`, and `session-unlabel label:<name>` (removes only the persisted `label` metadata field from a session; preserves the existing `session_id`, does not mutate transcript entries or transcript ordering, and does not bump `updated_at_ms` so newest-first ordering stays activity-based; output is a deterministic `{ unlabeled_session_id, removed_label }` JSON shape; unknown sessions/selectors fail cleanly via the shared selector machinery, and attempting to unlabel a session that carries no label surfaces as `SessionAlreadyUnlabeled` rather than a silent no-op; after removal the persisted session JSON no longer emits a `label` field so older unlabeled sessions stay byte-compatible)
- `label:<name>` selector accepted anywhere a single persisted session id is accepted (`session-show`, `transcript-show`, `resume`, `session-export`, `session-delete`, `session-fork`, `session-rename`, and either side of `session-compare`). Selector resolution is centralized at the runtime layer (`RuntimeEngine::resolve_selector`) and delegates to `SessionStore::resolve_selector` so every command path resolves selectors uniformly. Raw session ids and `latest` keep their existing behavior; machine-readable JSON outputs continue to surface the actual resolved `session_id` rather than the selector input. Failure modes are deterministic and distinct: unknown labels surface as `SessionNotFound("label:<name>")`, duplicate labels as `AmbiguousLabel`, and an empty `label:` as `MalformedSelector`. Sessions without a label are transparently skipped during label resolution so mixed labeled/unlabeled stores remain backward-compatible.

## Structured Event Model

The Rust runtime should expose a typed event stream. First event set:

- `SessionStarted`
- `SessionResumed`
- `PromptReceived`
- `RouteComputed`
- `CommandMatched`
- `ToolMatched`
- `PermissionDenied`
- `CommandInvoked`
- `CommandCompleted`
- `ToolInvoked`
- `ToolCompleted`
- `TurnCompleted`
- `SessionPersisted`
- `TranscriptPersisted`

These should be serializable with `serde`.

## MVP Scope

The first shippable Rust slice should provide:

1. a buildable Cargo workspace
2. a typed session/transcript model
3. command and tool registries with metadata
4. deterministic prompt routing
5. a runtime turn processor that emits events
6. JSON session persistence to disk
7. a CLI exposing summary, route, bootstrap, tools, commands, and session inspection

## What We Are Not Doing First

Not in the first slice:
- full remote runtime support
- UI replication
- MCP/runtime network integration
- plugin loading
- voice surfaces
- full parity with the exposed codebase
- full agent/subagent execution

Those can come later once the core architecture is real.

## How This Helps Horizon

The donor value for Horizon/Rune is in reusable primitives:
- typed events instead of ad hoc runtime strings
- session/transcript persistence with explicit structure
- cleaner command/tool registries
- explainable routing decisions
- more reliable turn processing contracts

Once stable, these can be transplanted selectively into Horizon.

## Rewrite Strategy

1. document the learned architecture
2. establish a Rust workspace
3. implement core domain types
4. implement session/transcript persistence
5. implement command/tool registries
6. implement router and runtime turn processing
7. expose a CLI
8. delete obsolete Python-first scaffolding once replacement is in place

## Success Criteria

The first rewrite milestone is successful when:
- the repository builds as a Rust workspace
- `cargo test` passes
- `cargo clippy -- -D warnings` passes
- a prompt can be routed and processed through a typed runtime path
- sessions can be persisted and reloaded
- events and registries are inspectable from the CLI

