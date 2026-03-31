# Claude Code Rust Port - Implementation Plan

## Active Goal

Rewrite the current repository into a Rust-native harness runtime that learns from Claude Code architecture and produces reusable runtime components for Horizon.

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
- [x] `tools`
- [x] `commands`
- [x] `session-show <id>`

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
- Keep donor value for Horizon in mind at all times

## Immediate Next Slice

1. add example-driven CLI docs regression tests if the visible CLI surface grows beyond the current baseline
2. expand README/ARCHITECTURE examples if the CLI surface changes
3. keep each follow-up slice tied to a GitHub issue and PR
4. prioritize slices that improve primary Claude Code CLI usability, not just internal donor value
