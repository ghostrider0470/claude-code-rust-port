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
