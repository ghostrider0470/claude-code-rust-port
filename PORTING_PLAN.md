# Claude Code Rust Port - Implementation Plan

## Active Goal

Rewrite the current repository into a Rust-native harness runtime that learns from Claude Code architecture and produces reusable runtime components for Horizon.

## Phase 1 - Architecture Capture

- [x] Review mirrored workspace and snapshot metadata
- [x] Identify major architectural surfaces
- [x] Write `ARCHITECTURE.md`
- [x] Write this implementation plan

## Phase 2 - Rust Workspace Bootstrap

- [ ] Create workspace `Cargo.toml`
- [ ] Create crates:
  - [ ] `harness-core`
  - [ ] `harness-session`
  - [ ] `harness-tools`
  - [ ] `harness-commands`
  - [ ] `harness-runtime`
  - [ ] `harness-cli`
- [ ] Add shared dependencies (`serde`, `serde_json`, `clap`, `thiserror`, etc.)
- [ ] Ensure `cargo check` passes

## Phase 3 - Core Types

- [ ] Implement ids and metadata types
- [ ] Implement usage/accounting types
- [ ] Implement error model
- [ ] Implement event enums

## Phase 4 - Session + Transcript

- [ ] `SessionState`
- [ ] `TranscriptEntry`
- [ ] append/replay/compact
- [ ] persistence to disk
- [ ] session reload

## Phase 5 - Registries

- [ ] Tool registry with metadata
- [ ] Command registry with metadata
- [ ] Search/filter/list APIs
- [ ] Permission policy layer

## Phase 6 - Router + Runtime

- [ ] Prompt tokenization
- [ ] Match scoring
- [ ] Deterministic ranking
- [ ] Turn processor
- [ ] Event emission
- [ ] Session persistence after turn

## Phase 7 - CLI

- [ ] `summary`
- [ ] `route <prompt>`
- [ ] `bootstrap <prompt>`
- [ ] `tools list`
- [ ] `commands list`
- [ ] `session show <id>`

## Phase 8 - Cleanup

- [ ] Remove obsolete Python-first scaffolding
- [ ] Rewrite README for Rust-first identity
- [ ] Add examples and usage docs

## Engineering Rules

- Build Rust-native modules, not Python transliterations
- Keep modules small and typed
- Prefer enums/structs over arbitrary strings
- Separate pure logic from filesystem operations
- Make runtime behavior inspectable through events
- Keep donor value for Horizon in mind at all times

## Immediate Next Slice

1. bootstrap Cargo workspace
2. create core crates
3. implement minimal typed models
4. wire a first compiling CLI
5. validate with `cargo check`

