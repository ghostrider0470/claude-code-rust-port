# Claude Code Rust Port - Implementation Plan

## Active Goal

Build a Rust-native Claude Code-style CLI/runtime that Hamza can use as a primary tool, while keeping the underlying components clean enough to donate useful pieces into Horizon later.

## Phase 1 - Architecture Capture

- [x] Review mirrored workspace and snapshot metadata
- [x] Identify major architectural surfaces
- [x] Write `ARCHITECTURE.md`
- [x] Write this implementation plan

## Phase 2 - Rust Workspace Bootstrap

- [x] Create workspace `Cargo.toml`
- [x] Create crates:
  - [x] `harness-core`
  - [x] `harness-session`
  - [x] `harness-tools`
  - [x] `harness-commands`
  - [x] `harness-runtime`
  - [x] `harness-cli`
- [x] Add shared dependencies (`serde`, `serde_json`, `clap`, `thiserror`, etc.)
- [x] Ensure `cargo check` passes

## Phase 3 - Core Types

- [x] Implement ids and metadata types
- [x] Implement usage/accounting types
- [x] Implement error model
- [x] Implement event enums

## Phase 4 - Session + Transcript

- [x] `SessionState`
- [x] `TranscriptEntry`
- [x] append/replay/compact
- [x] persistence to disk
- [x] session reload

## Phase 5 - Registries

- [x] Tool registry with metadata
- [x] Command registry with metadata
- [x] Search/filter/list APIs
- [x] Permission policy layer

## Phase 6 - Router + Runtime

- [x] Prompt tokenization
- [x] Match scoring
- [x] Deterministic ranking
- [x] Turn processor
- [x] Event emission
- [x] Session persistence after turn

## Phase 7 - CLI

- [x] `summary`
- [x] `route <prompt>`
- [x] `bootstrap <prompt>`
- [x] `resume <id> <prompt>` (and `resume latest <prompt>`) — append a new turn to an existing persisted session; output confirms the targeted session id, the appended turn index, and the refreshed `updated_at_ms` activity metadata
- [x] `tools`
- [x] `commands`
- [x] `session-show <id>`
- [x] `transcript-show <id>` (and `transcript-show latest`) — inspect the persisted transcript for an explicit session id or the most recently active session; output is machine-readable JSON that restates the owning `session_id`, the session's recency metadata, and the turn entries in `turn_index` order
- [x] `session-export <id>` (and `session-export latest`) — export a persisted session bundle as deterministic JSON packaging session state plus transcript together; output confirms the `exported_session_id` and preserves `turn_index` ordering so bundles can be archived, attached to bug reports, or compared across environments without manually inspecting `.sessions/` files
- [x] `session-compare <left-id> <right-id>` (with `latest` accepted on either side) — compare two persisted sessions as a single machine-readable JSON bundle that identifies both compared `session_id`s and reports signed `right - left` deltas for recency metadata (`created_at_ms_delta`, `updated_at_ms_delta`) and activity metadata (`message_count_delta`, `transcript_entry_count_delta`), plus a `same_session` flag so self-comparisons via `latest latest` are trivially recognizable
- [x] `session-delete <id>` (and `session-delete latest`) — delete a persisted session cleanly; a single call removes both the session JSON and its sibling transcript JSON, and the output is deterministic machine-readable JSON that identifies the deleted session id plus the removed paths so the deletion is trivially auditable, with a clean `SessionNotFound` failure when the target session does not exist
- [x] `session-import <bundle-path>` — restore a persisted session from a JSON bundle previously emitted by `session-export`; accepts the deterministic `{ exported_session_id, session, transcript }` shape, recreates both persisted artifacts while preserving the imported session id, recency/activity metadata, and transcript `turn_index` ordering, and fails cleanly without overwriting unrelated persisted sessions when the bundle is invalid or the target session id already exists locally
- [x] `session-find <query>` — search persisted local sessions by transcript prompt text without mutating any session state; the query is matched case-insensitively as a substring against each persisted transcript entry; output is a deterministic JSON array ordered using the existing newest-first session ordering, where each result identifies the matched `session_id` plus recency/activity metadata and a `matches` array of `{ turn_index, prompt }` entries so the command is useful from the terminal without a follow-up `transcript-show`; an empty query and a query with no matches both succeed cleanly with an empty array instead of erroring
- [x] `session-rename <id> <label>` (and `session-rename latest <label>`) — attach a trimmed, non-empty human-readable label to persisted session metadata while preserving the existing `session_id`, leaving transcript entries and ordering untouched, and not bumping `updated_at_ms` so newest-first ordering stays activity-based; output is a deterministic `{ renamed_session_id, applied_label }` JSON shape, and empty/whitespace-only labels and unknown session ids fail cleanly; older unlabeled sessions remain readable because the label field is only emitted after a session has actually been labeled
- [x] `session-labels` — list every persisted session that currently carries a label without mutating any session state; output is a deterministic JSON array ordered using the same newest-first persisted-session ordering as `sessions`, each entry exposes `label`, `session_id`, `created_at_ms`, `updated_at_ms`, `message_count`, and `persisted_path`; unlabeled sessions are omitted, duplicate labels stay visible as separate rows so ambiguity is discoverable before a `label:<name>` selector would fail with `AmbiguousLabel`, and an empty labeled store returns a clean empty JSON array
- [x] `label:<name>` selector for persisted sessions accepted anywhere a single persisted session id is accepted (`session-show`, `transcript-show`, `resume`, `session-export`, `session-delete`, `session-fork`, `session-rename`, and either side of `session-compare`); selector resolution is centralized at the runtime layer (`RuntimeEngine::resolve_selector`) and delegates to `SessionStore::resolve_selector` so every command path resolves selectors uniformly; raw session ids and `latest` keep their existing behavior, machine-readable JSON outputs continue to surface the actual resolved `session_id`, and unknown labels (`SessionNotFound("label:<name>")`), ambiguous labels (`AmbiguousLabel`), and malformed selectors (`MalformedSelector` for `label:` with no name) all fail cleanly with distinct diagnostics; activity-based newest-first ordering is unchanged and mixed labeled/unlabeled stores stay backward-compatible
- [x] `session-unlabel <id>` (and `session-unlabel latest` / `session-unlabel label:<name>`) — remove the persisted `label` metadata field from a session without disturbing `session_id`, transcript entries, transcript ordering, or `updated_at_ms`; the same shared selector machinery as every other single-session command accepts raw ids, `latest`, and `label:<name>`; output is a deterministic `{ unlabeled_session_id, removed_label }` JSON shape that identifies the resolved session id and the label that was cleared; unknown sessions/selectors surface as `SessionNotFound`, and attempting to unlabel a session that already carries no label surfaces as `SessionAlreadyUnlabeled` rather than a silent no-op; after removal the session disappears from `session-labels` while its persisted JSON no longer emits a `label` field, keeping older unlabeled sessions backward-compatible
- [x] `session-retag <id> <label>` (and `session-retag latest <label>` / `session-retag label:<old-name> <new-name>`) — atomically replace the persisted `label` metadata field on a session in one step, preserving the existing `session_id`, leaving transcript entries and ordering untouched, and not bumping `updated_at_ms` so newest-first ordering stays activity-based; the same shared selector machinery as every other single-session command accepts raw ids, `latest`, and `label:<name>`; output is a deterministic `{ retagged_session_id, previous_label, applied_label }` JSON shape that identifies the resolved session id, the label that was replaced, and the normalized label now persisted; unknown sessions/selectors surface as `SessionNotFound`, empty/whitespace-only labels surface as `InvalidLabel`, retagging an unlabeled session surfaces as `SessionAlreadyUnlabeled`, and a same-effective-label request surfaces as `SessionAlreadyLabeled` rather than a silent no-op; the persisted JSON continues to serialize the `label` field only when present so older unlabeled sessions stay backward-compatible
- [x] `session-prune --keep <count>` — bulk-remove older persisted sessions, preserving the newest `<count>` using the same newest-first ordering as `sessions` and `session-labels` (`updated_at_ms` → `created_at_ms` → `session_id` → `persisted_path`); for each pruned session both persisted artifacts are removed together — the `.sessions/<session-id>.json` file and its sibling `.sessions/<session-id>.transcript.json` — and the output is a deterministic `{ kept_count, pruned_count, removed }` JSON shape where `removed` lists each pruned session's `session_id` together with the removed session and transcript paths; preserved sessions are never mutated — their label, transcript entries, transcript ordering, and activity metadata stay exactly as they were; `--keep 0` is supported and prunes every persisted session, and a store already within the retention budget succeeds cleanly with an empty `removed` array

## Phase 8 - Cleanup

- [x] Remove obsolete Python-first scaffolding
- [x] Rewrite README for Rust-first identity
- [x] Add examples and usage docs

## Engineering Rules

- Build Rust-native modules, not Python transliterations
- Keep modules small and typed
- Prefer enums/structs over arbitrary strings
- Separate pure logic from filesystem operations
- Make runtime behavior inspectable through events
- Prioritize primary Claude Code CLI usability first, while preserving donor value for Horizon where it does not fight the product path

## Immediate Next Slice

1. expand README/ARCHITECTURE examples when the visible CLI surface changes beyond the current seeded baseline
2. keep README-backed CLI example tests updated whenever documented output changes
3. keep each follow-up slice tied to a GitHub issue and PR
4. prioritize slices that improve primary Claude Code CLI usability, not just internal donor value
