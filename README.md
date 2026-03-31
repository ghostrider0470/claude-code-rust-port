# Claude Code Rust Port

Rust-first harness runtime research and implementation workspace.

## Purpose

This repository studies the exposed Claude Code architecture at the systems level and rebuilds the useful harness ideas as an original Rust-native runtime. The target is not path parity or source transliteration. The target is a typed, inspectable, reusable agent runtime that can later donate components into Horizon/Rune.

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
├── crates/
│   ├── harness-cli/
│   ├── harness-commands/
│   ├── harness-core/
│   ├── harness-runtime/
│   ├── harness-session/
│   └── harness-tools/
└── src/
    └── reference_data/
```

## Rust MVP Target

The first meaningful milestone is:

- typed core models
- session + transcript persistence
- tool and command registries
- deterministic routing
- runtime turn processor with structured events
- CLI commands for summary, route, bootstrap, tools, commands, and session inspection

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

## Rust Test Coverage Baseline

Current protected Rust surface:

- `harness-session` save/load round-trip persistence
- transcript compaction behavior in `harness-session`
- deterministic route ordering in `harness-runtime`
- bootstrap permission denial + session persistence behavior in `harness-runtime`
- CLI summary output for the seeded runtime surface
- CLI JSON inspection output for `tools` and `commands`
- CLI persisted-session round trip through `bootstrap` + `session-show`

Validation commands:

```bash
cargo test -p harness-session
cargo test -p harness-runtime
cargo test -p harness-cli
cargo test
```

More runtime and CLI coverage should continue incrementally through the active issue queue.

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
- [x] cleanup of obsolete Python-first scaffolding
