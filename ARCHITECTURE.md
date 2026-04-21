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
- `sessions` with optional `--limit <n>` (inspect-only listing that preserves the existing newest-first ordering — `updated_at_ms` → `created_at_ms` → `session_id` → `persisted_path`; omitting `--limit` returns every persisted session unchanged; `--limit <n>` returns at most the newest `n` rows from that same ordering, `--limit 0` returns an empty array cleanly, a `--limit` larger than the store returns every available session cleanly, and the per-row JSON shape stays the same — the limited form is a slice of the existing array, not a new wrapper object; does not mutate any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata)
- `session show <id>`
- `session show latest`
- `transcript show <id>` and `transcript show latest` (machine-readable JSON transcript inspection that restates the owning session id and preserves turn ordering)
- `session-export <id>` and `session-export latest` (deterministic JSON export bundle that packages session state plus transcript together; output confirms the exported session id and preserves turn ordering)
- `session-compare <left-id> <right-id>` with `latest` accepted on either side (deterministic JSON comparison bundle that identifies both compared session ids and reports signed deltas for recency/activity metadata and transcript/turn counts)
- `session-delete <id>` and `session-delete latest` (removes both the session JSON and its sibling transcript JSON; deterministic JSON output identifies the deleted session id and the removed paths, and the command fails cleanly without deleting anything else when the target session does not exist)
- `session-import <bundle-path>` (imports a persisted session bundle from a JSON file in the `session-export` shape; recreates both the session JSON and its sibling transcript JSON, preserves the imported session id, recency/activity metadata, and transcript `turn_index` ordering, and fails cleanly without overwriting unrelated persisted sessions when the bundle is invalid or the target session id already exists locally)
- `session-find <query>` with optional `--limit <n>` (searches persisted local sessions by transcript prompt text without mutating any session state; the query is matched case-insensitively as a substring against each persisted transcript entry; output is a deterministic JSON array of result objects, ordered using the same newest-first session ordering as `sessions`, where each result identifies the matched `session_id` plus recency/activity metadata and a `matches` array of `{ turn_index, prompt }` entries; an empty query and a query with no matches both succeed cleanly with an empty array; omitting `--limit` returns every matching persisted session unchanged, `--limit <n>` returns at most the newest `n` rows from the same ordering and matched subset, `--limit 0` returns an empty array cleanly, a `--limit` larger than the matched subset returns every matching session cleanly, each retained row's `matches` array is unchanged under limiting, and the per-row JSON shape is preserved — the limited form is a slice of the existing array rather than a new wrapper object; negative and non-numeric `--limit` values fail cleanly at parse time)
- `session-rename <id> <label>` and `session-rename latest <label>` (attaches a trimmed, non-empty human-readable label to persisted session metadata while preserving the existing `session_id`; the rename does not mutate transcript entries or transcript ordering and does not bump `updated_at_ms` so newest-first ordering stays activity-based; output is a deterministic `{ renamed_session_id, applied_label }` JSON shape, and empty/whitespace-only labels and unknown session ids fail cleanly)
- `session-labels` with optional `--limit <n>` (lists every persisted session that currently carries a label without mutating any session state; output is a deterministic JSON array ordered using the same newest-first persisted-session ordering as `sessions`, and each entry exposes `label`, `session_id`, `created_at_ms`, `updated_at_ms`, `message_count`, and `persisted_path`; unlabeled sessions are omitted, duplicate labels remain visible as separate rows so ambiguity is discoverable before a `label:<name>` selector would fail, and an empty labeled store returns a clean empty JSON array; omitting `--limit` returns every labeled persisted session unchanged, `--limit <n>` returns at most the newest `n` rows from the same ordering, `--limit 0` returns an empty array cleanly, a `--limit` larger than the labeled subset returns every labeled session cleanly, and the per-row JSON shape is preserved — the limited form is a slice of the existing array, not a new wrapper object; negative and non-numeric `--limit` values fail cleanly at parse time)
- `session-unlabel <id>`, `session-unlabel latest`, and `session-unlabel label:<name>` (removes only the persisted `label` metadata field from a session; preserves the existing `session_id`, does not mutate transcript entries or transcript ordering, and does not bump `updated_at_ms` so newest-first ordering stays activity-based; output is a deterministic `{ unlabeled_session_id, removed_label }` JSON shape; unknown sessions/selectors fail cleanly via the shared selector machinery, and attempting to unlabel a session that carries no label surfaces as `SessionAlreadyUnlabeled` rather than a silent no-op; after removal the persisted session JSON no longer emits a `label` field so older unlabeled sessions stay byte-compatible)
- `session-retag <id> <label>`, `session-retag latest <label>`, and `session-retag label:<old-name> <new-name>` (atomically replaces the persisted `label` metadata field on a session that already carries one; preserves the existing `session_id`, does not mutate transcript entries or transcript ordering, and does not bump `updated_at_ms` so newest-first ordering stays activity-based; output is a deterministic `{ retagged_session_id, previous_label, applied_label }` JSON shape; unknown sessions/selectors fail cleanly via the shared selector machinery, empty/whitespace-only labels surface as `InvalidLabel`, retagging a session that has no label surfaces as `SessionAlreadyUnlabeled`, and a request whose normalized label equals the already-persisted label surfaces as `SessionAlreadyLabeled` rather than a silent no-op; the persisted JSON continues to serialize the `label` field only when present so older unlabeled sessions stay backward-compatible)
- `session-prune --keep <count>` (bulk-removes older *prune-eligible (unpinned)* persisted sessions, preserving the newest `<count>` unpinned sessions using the same newest-first ordering as `sessions` and `session-labels` — `updated_at_ms` → `created_at_ms` → `session_id` → `persisted_path` — applied only across the unpinned subset; pinned sessions are always preserved regardless of `<count>` and surface via `pinned_preserved_count` / `pinned_preserved`; for each pruned session both persisted artifacts are removed together — the `.sessions/<session-id>.json` file and its sibling `.sessions/<session-id>.transcript.json` — and the output is a deterministic `{ kept_count, pruned_count, pinned_preserved_count, removed, pinned_preserved }` JSON shape where `removed` lists each pruned session's `session_id` together with the removed session and transcript paths, and `pinned_preserved` lists every rescued pinned `session_id`; preserved sessions are never mutated (label, pinned flag, transcript entries, transcript ordering, and activity metadata all stay exactly as they were); `--keep 0` is supported and prunes every unpinned persisted session, and a store already within the retention budget after excluding pins succeeds cleanly with an empty `removed` array)
- `session-pin <id>`, `session-pin latest`, and `session-pin label:<name>` (marks a persisted session as pinned so `session-prune` always excludes it from removal regardless of `--keep`; preserves the existing `session_id`, does not mutate transcript entries or transcript ordering, and does not bump `updated_at_ms` so newest-first ordering stays activity-based; output is a deterministic `{ pinned_session_id, pinned }` JSON shape; unknown sessions/selectors fail cleanly via the shared selector machinery, and attempting to pin an already-pinned session surfaces as `SessionAlreadyPinned` rather than a silent no-op; the persisted session JSON serializes the `pinned` field only when actually pinned so older unpinned sessions stay byte-compatible; pinned state also surfaces through `sessions`, `session-show`, `session-export`, `session-compare`, and `session-labels`)
- `session-unpin <id>`, `session-unpin latest`, and `session-unpin label:<name>` (clears the pinned flag on a persisted session so it becomes eligible for `session-prune` again; preserves the existing `session_id`, does not mutate transcript entries or transcript ordering, and does not bump `updated_at_ms` so newest-first ordering stays activity-based; output is a deterministic `{ unpinned_session_id, pinned }` JSON shape; unknown sessions/selectors fail cleanly via the shared selector machinery, and attempting to unpin a session that is not pinned surfaces as `SessionAlreadyUnpinned` rather than a silent no-op; after removal the persisted JSON no longer emits a `pinned` field so older sessions stay backward-compatible)
- `session-pins` with optional `--limit <n>` (lists every persisted session that is currently pinned without mutating any session state; output is a deterministic JSON array ordered using the same newest-first persisted-session ordering as `sessions` and `session-labels` — `updated_at_ms` → `created_at_ms` → `session_id` → `persisted_path` — and each entry exposes `session_id`, `created_at_ms`, `updated_at_ms`, `message_count`, `persisted_path`, and `pinned: true`, plus `label` when the pinned session carries one so the listing is useful from the terminal without a follow-up `session-show`; unpinned sessions are omitted, duplicate labels on pinned sessions remain visible as separate rows so ambiguity is discoverable, and an empty pinned store returns a clean empty JSON array; omitting `--limit` returns every pinned persisted session unchanged, `--limit <n>` returns at most the newest `n` rows from the same ordering and filtered subset, `--limit 0` returns an empty array cleanly, a `--limit` larger than the pinned subset returns every pinned session cleanly, and the per-row JSON shape is preserved — the limited form is a slice of the existing array rather than a new wrapper object; negative and non-numeric `--limit` values fail cleanly at parse time; pairs with `session-prune --keep <count>` so the protected set can be audited before a prune runs)
- `session-selector-check <selector>` (inspect-only command that resolves a single-session selector through the shared selector-resolution path and surfaces the targeted persisted session's descriptive metadata without mutating any persisted state, transcript entry, label, pinned flag, id, path, or ordering metadata; accepts raw session ids, `latest`, and `label:<name>` — the same forms every other single-session command accepts; output is a deterministic `{ selector, resolved_session_id, created_at_ms, updated_at_ms, message_count, persisted_path, label?, pinned? }` JSON shape where `selector` echoes the raw input verbatim, `resolved_session_id` is the persisted id the selector actually maps to, `label` is only emitted when the targeted session carries one, and `pinned` is only emitted when `true`; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-tail <selector>` with optional `--count <n>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns the newest transcript entries for the resolved session in their original `turn_index` order without mutating any persisted state, transcript entry, label, pinned flag, id, path, or ordering metadata; `--count` defaults to `10` when omitted, a `--count` larger than the persisted transcript returns every available entry, and `--count 0` or an empty transcript returns an empty `entries` array cleanly; output is a deterministic `{ selector, resolved_session_id, created_at_ms, updated_at_ms, total_entries, returned_entries, entries }` JSON shape where `selector` echoes the raw input, `total_entries` is the full transcript length, `returned_entries == entries.len()`, and each entry carries at least `turn_index` and `prompt`; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-find <selector> <query>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and searches only the resolved session's persisted transcript for entries whose prompt text contains `<query>` case-insensitively as a substring, mirroring `session-find` query semantics but scoped to a single selected session; output is a deterministic `{ selector, resolved_session_id, query, created_at_ms, updated_at_ms, total_entries, match_count, matches }` JSON shape where `selector` and `query` echo the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `total_entries` is the full transcript length, `match_count == matches.len()`, and each entry in `matches` carries at least `turn_index` and `prompt` in the source transcript's `turn_index` order; an empty query and a no-match query both succeed cleanly with `match_count == 0` and an empty `matches` array; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`; does not mutate any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata)
- `transcript-range <selector> --start <turn-index> [--count <n>]` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns a bounded forward slice of the resolved session's persisted transcript beginning at `turn_index == <start>` and containing at most `<n>` entries in their original `turn_index` order; `--count` defaults to `10` when omitted; a `--count` larger than the remaining entries returns the available tail cleanly, a `--start` past the end of the transcript — including on an empty transcript — returns an empty `entries` array cleanly, and negative / non-numeric `--start` or `--count` values fail cleanly at parse time; output is a deterministic `{ selector, resolved_session_id, start_turn_index, requested_count, created_at_ms, updated_at_ms, total_entries, returned_entries, entries }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `start_turn_index` and `requested_count` echo the requested window, `total_entries` is the full transcript length, `returned_entries == entries.len()`, and each entry carries at least `turn_index` and `prompt`; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`; does not mutate any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata)
- `transcript-turn-show <selector> --turn <turn-index>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns the single persisted transcript entry whose `turn_index == <turn>` from the resolved session's transcript without mutating any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata; output is a deterministic `{ selector, resolved_session_id, turn_index, created_at_ms, updated_at_ms, total_entries, entry }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `turn_index` echoes the requested turn, `total_entries` is the full transcript length, and `entry` carries at least `turn_index` and `prompt`; because the contract is to return exactly one entry, an empty transcript and a `--turn` past the end of the transcript both fail cleanly and deterministically as `TranscriptTurnOutOfRange` rather than masking the miss as a silent empty result, and negative / non-numeric `--turn` values fail cleanly at parse time; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-last-turn <selector>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns the single persisted transcript entry with the highest available `turn_index` from the resolved session's transcript without mutating any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata; output is a deterministic `{ selector, resolved_session_id, turn_index, created_at_ms, updated_at_ms, total_entries, entry }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `turn_index` is the newest transcript `turn_index` present, `total_entries` is the full transcript length, and `entry` carries at least `turn_index` and `prompt`; because the contract is to return exactly one entry, an empty transcript has no last turn and fails cleanly and deterministically as `TranscriptTurnOutOfRange` rather than masking the miss as a silent empty result; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-first-turn <selector>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns the single persisted transcript entry with the lowest available `turn_index` from the resolved session's transcript without mutating any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata; output is a deterministic `{ selector, resolved_session_id, turn_index, created_at_ms, updated_at_ms, total_entries, entry }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `turn_index` is the oldest transcript `turn_index` present, `total_entries` is the full transcript length, and `entry` carries at least `turn_index` and `prompt`; because the contract is to return exactly one entry, an empty transcript has no first turn and fails cleanly and deterministically as `TranscriptTurnOutOfRange` rather than masking the miss as a silent empty result; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-entry-count <selector>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns a deterministic machine-readable summary of the resolved session's persisted transcript length without returning any transcript entries and without mutating any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata; output is a deterministic `{ selector, resolved_session_id, created_at_ms, updated_at_ms, total_entries }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, and `total_entries` equals the persisted transcript length; empty transcripts succeed cleanly with `total_entries: 0` because the command's contract does not require returning an entry; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-has-entries <selector>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns a deterministic machine-readable summary describing whether the resolved session's persisted transcript contains any entries without returning transcript entries and without mutating any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata; output is a deterministic `{ selector, resolved_session_id, created_at_ms, updated_at_ms, total_entries, has_entries }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `total_entries` equals the persisted transcript length, and `has_entries` is `true` exactly when `total_entries > 0`; empty transcripts succeed cleanly with `total_entries: 0` and `has_entries: false`; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-turn-exists <selector> --turn <turn-index>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns a deterministic machine-readable summary describing whether the resolved session's persisted transcript contains an entry whose `turn_index == <turn>` without returning transcript entries and without mutating any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata; output is a deterministic `{ selector, resolved_session_id, turn_index, created_at_ms, updated_at_ms, total_entries, exists }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `turn_index` echoes the requested turn, `total_entries` equals the persisted transcript length, and `exists` is `true` exactly when the transcript contains an entry whose `turn_index == <turn>`; empty transcripts and out-of-range turns succeed cleanly with `exists: false`, and negative / non-numeric `--turn` values fail cleanly at parse time; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-turn-indexes <selector>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns a deterministic machine-readable summary listing the `turn_index` values present in the resolved session's persisted transcript in ascending order without returning transcript entries and without mutating any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata; output is a deterministic `{ selector, resolved_session_id, created_at_ms, updated_at_ms, total_entries, turn_indexes }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `total_entries` equals the persisted transcript length, and `turn_indexes` is an ascending array of the `turn_index` values present in the persisted transcript; empty transcripts succeed cleanly with `total_entries: 0` and `turn_indexes: []`; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-turn-index-range <selector>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns a deterministic machine-readable summary describing the lowest and highest `turn_index` values present in the resolved session's persisted transcript without returning transcript entries and without mutating any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata; output is a deterministic `{ selector, resolved_session_id, created_at_ms, updated_at_ms, total_entries, first_turn_index, last_turn_index }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `total_entries` equals the persisted transcript length, and `first_turn_index` / `last_turn_index` are the smallest and largest present `turn_index` values in the persisted transcript; empty transcripts succeed cleanly with `total_entries: 0` and both bounds `null`; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-has-turn-gaps <selector>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns a deterministic machine-readable summary describing whether the resolved session's persisted transcript has at least one missing integer `turn_index` between the smallest and largest present `turn_index` values without returning transcript entries and without mutating any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata; output is a deterministic `{ selector, resolved_session_id, created_at_ms, updated_at_ms, total_entries, has_turn_gaps }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `total_entries` equals the persisted transcript length, and `has_turn_gaps` is `true` exactly when at least one integer `turn_index` between the smallest and largest present `turn_index` values is missing from the resolved persisted transcript; empty and single-entry transcripts succeed cleanly with `has_turn_gaps: false`; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-missing-turn-indexes <selector>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns a deterministic machine-readable summary listing every missing integer `turn_index` between the smallest and largest present `turn_index` values in the resolved session's persisted transcript in ascending order without returning transcript entries and without mutating any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata; output is a deterministic `{ selector, resolved_session_id, created_at_ms, updated_at_ms, total_entries, missing_turn_indexes }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `total_entries` equals the persisted transcript length, and `missing_turn_indexes` is an ascending array of every missing integer `turn_index` between the smallest and largest present `turn_index` values in the persisted transcript; empty, single-entry, and contiguous transcripts succeed cleanly with `missing_turn_indexes: []`; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-turn-density <selector>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns a deterministic machine-readable summary describing how densely populated the resolved session's persisted transcript's `turn_index` span is without returning transcript entries and without mutating any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata; output is a deterministic `{ selector, resolved_session_id, created_at_ms, updated_at_ms, total_entries, span_entry_count, missing_turn_count, turn_density }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `total_entries` equals the persisted transcript length, `span_entry_count` is the number of integer turn positions in the inclusive span from the smallest present `turn_index` to the largest present `turn_index`, `missing_turn_count` equals `span_entry_count - total_entries`, and `turn_density` equals `total_entries / span_entry_count` as a deterministic numeric value; empty transcripts succeed cleanly with `total_entries: 0`, `span_entry_count: 0`, `missing_turn_count: 0`, and `turn_density: 1.0`; single-entry and contiguous transcripts succeed cleanly with `missing_turn_count: 0` and `turn_density: 1.0`; gapped transcripts report `turn_density` strictly less than `1.0`; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-gap-ranges <selector>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns a deterministic machine-readable summary describing every contiguous run of missing integer `turn_index` values between the smallest and largest present `turn_index` values in the resolved session's persisted transcript, in ascending order, without returning transcript entries and without mutating any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata; output is a deterministic `{ selector, resolved_session_id, created_at_ms, updated_at_ms, total_entries, gap_ranges }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `total_entries` equals the persisted transcript length, and `gap_ranges` is an ascending array of contiguous missing-turn runs, each range carrying `{ start_turn_index, end_turn_index, missing_count }` with inclusive bounds; single missing turns collapse to a range where `start_turn_index == end_turn_index` and `missing_count == 1`; adjacent missing turns collapse into a single range and disjoint gaps produce multiple ascending ranges; empty, single-entry, and contiguous transcripts succeed cleanly with `gap_ranges: []`; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-largest-gap <selector>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns a deterministic machine-readable summary describing only the single largest contiguous run of missing integer `turn_index` values between the smallest and largest present `turn_index` values in the resolved session's persisted transcript, without returning transcript entries and without mutating any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata; output is a deterministic `{ selector, resolved_session_id, created_at_ms, updated_at_ms, total_entries, largest_gap }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `total_entries` equals the persisted transcript length, and `largest_gap` is either `null` when no gap exists or an object carrying `{ start_turn_index, end_turn_index, missing_count }` with inclusive bounds; a single missing turn collapses to a run where `start_turn_index == end_turn_index` and `missing_count == 1`; when multiple gap runs tie for the highest `missing_count`, the earliest run (lowest `start_turn_index`) wins deterministically; empty transcripts, single-entry transcripts, and contiguous transcripts succeed cleanly with `largest_gap: null`; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-smallest-gap <selector>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns a deterministic machine-readable summary describing only the single smallest contiguous run of missing integer `turn_index` values between the smallest and largest present `turn_index` values in the resolved session's persisted transcript, without returning transcript entries and without mutating any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata; output is a deterministic `{ selector, resolved_session_id, created_at_ms, updated_at_ms, total_entries, smallest_gap }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `total_entries` equals the persisted transcript length, and `smallest_gap` is either `null` when no gap exists or an object carrying `{ start_turn_index, end_turn_index, missing_count }` with inclusive bounds; a single missing turn collapses to a run where `start_turn_index == end_turn_index` and `missing_count == 1`; when multiple gap runs tie for the lowest `missing_count`, the earliest run (lowest `start_turn_index`) wins deterministically; empty transcripts, single-entry transcripts, and contiguous transcripts succeed cleanly with `smallest_gap: null`; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-gap-count <selector>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns a deterministic machine-readable count of contiguous runs of missing integer `turn_index` values between the smallest and largest present `turn_index` values in the resolved session's persisted transcript without returning the runs themselves or any transcript entries and without mutating any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata; output is a deterministic `{ selector, resolved_session_id, created_at_ms, updated_at_ms, total_entries, gap_count }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `total_entries` equals the persisted transcript length, and `gap_count` is the number of contiguous missing-`turn_index` runs — adjacent missing turns collapse into one run and disjoint runs count separately; empty, single-entry, and contiguous transcripts succeed cleanly with `gap_count: 0`; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-missing-turn-count <selector>` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns a deterministic machine-readable count of individual missing integer `turn_index` values between the smallest and largest present `turn_index` values in the resolved session's persisted transcript without returning the missing indexes themselves or any transcript entries and without mutating any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata; output is a deterministic `{ selector, resolved_session_id, created_at_ms, updated_at_ms, total_entries, missing_turn_count }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `total_entries` equals the persisted transcript length, and `missing_turn_count` is the number of individual missing integer `turn_index` positions — every missing integer between the smallest and largest present `turn_index` values contributes `1` and disjoint runs sum additively; empty, single-entry, and contiguous transcripts succeed cleanly with `missing_turn_count: 0`; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`)
- `transcript-context <selector> --turn <turn-index> [--before <n>] [--after <n>]` (inspect-only command that resolves a single-session selector — raw `session_id`, `latest`, or `label:<name>` — through the shared selector-resolution path and returns a bounded symmetric window around `turn_index == <turn>` from the resolved session's persisted transcript, including the centered entry when present plus up to `<before>` preceding and `<after>` following entries in their original `turn_index` order; `--before` and `--after` each default to `2` when omitted; a window extending past either transcript bound is clipped cleanly to the available in-range entries, a `--turn` past the end of the transcript — including on an empty transcript — returns an empty `entries` array cleanly, and negative / non-numeric `--turn`, `--before`, or `--after` values fail cleanly at parse time; output is a deterministic `{ selector, resolved_session_id, center_turn_index, requested_before, requested_after, created_at_ms, updated_at_ms, total_entries, returned_entries, entries }` JSON shape where `selector` echoes the raw input, `resolved_session_id` is the persisted id the selector actually maps to, `center_turn_index`, `requested_before`, and `requested_after` echo the requested window, `total_entries` is the full transcript length, `returned_entries == entries.len()`, and each entry carries at least `turn_index` and `prompt`; preserves existing selector failure semantics unchanged — unknown ids/labels → `SessionNotFound`, duplicate labels → `AmbiguousLabel`, empty `label:` → `MalformedSelector`; does not mutate any persisted session, transcript entry, label, pinned flag, id, path, or ordering metadata)
- `label:<name>` selector accepted anywhere a single persisted session id is accepted (`session-show`, `transcript-show`, `resume`, `session-export`, `session-delete`, `session-fork`, `session-rename`, `session-unlabel`, `session-retag`, `session-pin`, `session-unpin`, and either side of `session-compare`). Selector resolution is centralized at the runtime layer (`RuntimeEngine::resolve_selector`) and delegates to `SessionStore::resolve_selector` so every command path resolves selectors uniformly. Raw session ids and `latest` keep their existing behavior; machine-readable JSON outputs continue to surface the actual resolved `session_id` rather than the selector input. Failure modes are deterministic and distinct: unknown labels surface as `SessionNotFound("label:<name>")`, duplicate labels as `AmbiguousLabel`, and an empty `label:` as `MalformedSelector`. Sessions without a label are transparently skipped during label resolution so mixed labeled/unlabeled stores remain backward-compatible.

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

