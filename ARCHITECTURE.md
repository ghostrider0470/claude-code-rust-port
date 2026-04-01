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
- `SessionStore`
- recency metadata for persisted sessions
- compaction policy
- disk persistence/load/list/latest

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
- `tools list`
- `commands list`
- `sessions` (newest-first)
- `session show <id>`
- `session show latest`

## Structured Event Model

The Rust runtime should expose a typed event stream. First event set:

- `SessionStarted`
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

