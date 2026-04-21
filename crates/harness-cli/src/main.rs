use clap::{Parser, Subcommand};
use harness_core::Prompt;
use harness_runtime::RuntimeEngine;
use harness_session::{
    DEFAULT_TRANSCRIPT_CONTEXT_WINDOW, DEFAULT_TRANSCRIPT_RANGE_COUNT,
    DEFAULT_TRANSCRIPT_TAIL_COUNT,
};

#[derive(Debug, Parser)]
#[command(name = "harness")]
#[command(about = "Rust-native harness runtime inspired by Claude Code architecture")]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    Summary,
    Route {
        prompt: String,
    },
    Bootstrap {
        prompt: String,
    },
    Resume {
        id: String,
        prompt: String,
    },
    Tools,
    Commands,
    Sessions,
    SessionShow {
        id: String,
    },
    TranscriptShow {
        id: String,
    },
    SessionExport {
        id: String,
    },
    SessionCompare {
        left: String,
        right: String,
    },
    SessionDelete {
        id: String,
    },
    SessionImport {
        path: String,
    },
    SessionFind {
        query: String,
    },
    SessionFork {
        id: String,
        prompt: String,
    },
    SessionRename {
        id: String,
        label: String,
    },
    SessionUnlabel {
        id: String,
    },
    SessionRetag {
        id: String,
        label: String,
    },
    SessionLabels,
    SessionPins,
    SessionPrune {
        #[arg(long)]
        keep: usize,
    },
    SessionPin {
        id: String,
    },
    SessionUnpin {
        id: String,
    },
    SessionSelectorCheck {
        selector: String,
    },
    TranscriptTail {
        selector: String,
        #[arg(long, default_value_t = DEFAULT_TRANSCRIPT_TAIL_COUNT)]
        count: usize,
    },
    TranscriptFind {
        selector: String,
        query: String,
    },
    TranscriptRange {
        selector: String,
        #[arg(long)]
        start: usize,
        #[arg(long, default_value_t = DEFAULT_TRANSCRIPT_RANGE_COUNT)]
        count: usize,
    },
    TranscriptContext {
        selector: String,
        #[arg(long)]
        turn: usize,
        #[arg(long, default_value_t = DEFAULT_TRANSCRIPT_CONTEXT_WINDOW)]
        before: usize,
        #[arg(long, default_value_t = DEFAULT_TRANSCRIPT_CONTEXT_WINDOW)]
        after: usize,
    },
    TranscriptTurnShow {
        selector: String,
        #[arg(long)]
        turn: usize,
    },
    TranscriptLastTurn {
        selector: String,
    },
    TranscriptFirstTurn {
        selector: String,
    },
    TranscriptEntryCount {
        selector: String,
    },
    TranscriptHasEntries {
        selector: String,
    },
    TranscriptTurnExists {
        selector: String,
        #[arg(long)]
        turn: usize,
    },
    TranscriptTurnIndexes {
        selector: String,
    },
    TranscriptTurnIndexRange {
        selector: String,
    },
    TranscriptHasTurnGaps {
        selector: String,
    },
    TranscriptMissingTurnIndexes {
        selector: String,
    },
    TranscriptTurnDensity {
        selector: String,
    },
    TranscriptGapRanges {
        selector: String,
    },
}

fn render_command(engine: &RuntimeEngine, command: CliCommand) -> String {
    match command {
        CliCommand::Summary => engine.summary(),
        CliCommand::Route { prompt } => {
            let matches = engine.route(&Prompt::new(prompt));
            serde_json::to_string_pretty(&matches).expect("serialize route result")
        }
        CliCommand::Bootstrap { prompt } => {
            let report = engine
                .bootstrap(Prompt::new(prompt))
                .expect("bootstrap runtime turn");
            serde_json::to_string_pretty(&report).expect("serialize bootstrap report")
        }
        CliCommand::Resume { id, prompt } => {
            let report = engine
                .resume(&id, Prompt::new(prompt))
                .expect("resume persisted session");
            serde_json::to_string_pretty(&report).expect("serialize resume report")
        }
        CliCommand::Tools => {
            serde_json::to_string_pretty(engine.tools.list()).expect("serialize tool list")
        }
        CliCommand::Commands => {
            serde_json::to_string_pretty(engine.commands.list()).expect("serialize command list")
        }
        CliCommand::Sessions => {
            let sessions = engine.list_sessions().expect("list persisted sessions");
            serde_json::to_string_pretty(&sessions).expect("serialize session list")
        }
        CliCommand::SessionShow { id } => {
            let session = engine.load_session(&id).expect("load session by id");
            serde_json::to_string_pretty(&session).expect("serialize session")
        }
        CliCommand::TranscriptShow { id } => {
            let transcript = engine.load_transcript(&id).expect("load transcript by id");
            serde_json::to_string_pretty(&transcript).expect("serialize transcript")
        }
        CliCommand::SessionExport { id } => {
            let export = engine
                .export_session(&id)
                .expect("export persisted session");
            serde_json::to_string_pretty(&export).expect("serialize session export")
        }
        CliCommand::SessionCompare { left, right } => {
            let comparison = engine
                .compare_sessions(&left, &right)
                .expect("compare persisted sessions");
            serde_json::to_string_pretty(&comparison).expect("serialize session comparison")
        }
        CliCommand::SessionDelete { id } => {
            let deletion = engine
                .delete_session(&id)
                .expect("delete persisted session");
            serde_json::to_string_pretty(&deletion).expect("serialize session deletion")
        }
        CliCommand::SessionImport { path } => {
            let imported = engine
                .import_session(&path)
                .expect("import persisted session bundle");
            serde_json::to_string_pretty(&imported).expect("serialize session import")
        }
        CliCommand::SessionFind { query } => {
            let results = engine
                .find_sessions(&query)
                .expect("search persisted sessions");
            serde_json::to_string_pretty(&results).expect("serialize session find results")
        }
        CliCommand::SessionFork { id, prompt } => {
            let fork = engine
                .fork_session(&id, Prompt::new(prompt))
                .expect("fork persisted session");
            serde_json::to_string_pretty(&fork).expect("serialize session fork")
        }
        CliCommand::SessionRename { id, label } => {
            let renamed = engine
                .rename_session(&id, &label)
                .expect("rename persisted session");
            serde_json::to_string_pretty(&renamed).expect("serialize session rename")
        }
        CliCommand::SessionUnlabel { id } => {
            let unlabeled = engine
                .unlabel_session(&id)
                .expect("unlabel persisted session");
            serde_json::to_string_pretty(&unlabeled).expect("serialize session unlabel")
        }
        CliCommand::SessionRetag { id, label } => {
            let retagged = engine
                .retag_session(&id, &label)
                .expect("retag persisted session");
            serde_json::to_string_pretty(&retagged).expect("serialize session retag")
        }
        CliCommand::SessionLabels => {
            let labels = engine
                .list_session_labels()
                .expect("list persisted session labels");
            serde_json::to_string_pretty(&labels).expect("serialize session labels")
        }
        CliCommand::SessionPins => {
            let pins = engine
                .list_session_pins()
                .expect("list persisted session pins");
            serde_json::to_string_pretty(&pins).expect("serialize session pins")
        }
        CliCommand::SessionPrune { keep } => {
            let pruned = engine
                .prune_sessions(keep)
                .expect("prune persisted sessions");
            serde_json::to_string_pretty(&pruned).expect("serialize session prune")
        }
        CliCommand::SessionPin { id } => {
            let pinned = engine.pin_session(&id).expect("pin persisted session");
            serde_json::to_string_pretty(&pinned).expect("serialize session pin")
        }
        CliCommand::SessionUnpin { id } => {
            let unpinned = engine.unpin_session(&id).expect("unpin persisted session");
            serde_json::to_string_pretty(&unpinned).expect("serialize session unpin")
        }
        CliCommand::SessionSelectorCheck { selector } => {
            let check = engine
                .check_session_selector(&selector)
                .expect("check persisted session selector");
            serde_json::to_string_pretty(&check).expect("serialize session selector check")
        }
        CliCommand::TranscriptTail { selector, count } => {
            let tail = engine
                .tail_session_transcript(&selector, count)
                .expect("tail persisted session transcript");
            serde_json::to_string_pretty(&tail).expect("serialize transcript tail")
        }
        CliCommand::TranscriptFind { selector, query } => {
            let find = engine
                .find_in_session_transcript(&selector, &query)
                .expect("search persisted session transcript");
            serde_json::to_string_pretty(&find).expect("serialize transcript find")
        }
        CliCommand::TranscriptRange {
            selector,
            start,
            count,
        } => {
            let range = engine
                .range_session_transcript(&selector, start, count)
                .expect("range persisted session transcript");
            serde_json::to_string_pretty(&range).expect("serialize transcript range")
        }
        CliCommand::TranscriptContext {
            selector,
            turn,
            before,
            after,
        } => {
            let context = engine
                .context_session_transcript(&selector, turn, before, after)
                .expect("context persisted session transcript");
            serde_json::to_string_pretty(&context).expect("serialize transcript context")
        }
        CliCommand::TranscriptTurnShow { selector, turn } => {
            let turn_show = engine
                .turn_show_session_transcript(&selector, turn)
                .expect("turn-show persisted session transcript");
            serde_json::to_string_pretty(&turn_show).expect("serialize transcript turn-show")
        }
        CliCommand::TranscriptLastTurn { selector } => {
            let last_turn = engine
                .last_turn_session_transcript(&selector)
                .expect("last-turn persisted session transcript");
            serde_json::to_string_pretty(&last_turn).expect("serialize transcript last-turn")
        }
        CliCommand::TranscriptFirstTurn { selector } => {
            let first_turn = engine
                .first_turn_session_transcript(&selector)
                .expect("first-turn persisted session transcript");
            serde_json::to_string_pretty(&first_turn).expect("serialize transcript first-turn")
        }
        CliCommand::TranscriptEntryCount { selector } => {
            let entry_count = engine
                .entry_count_session_transcript(&selector)
                .expect("entry-count persisted session transcript");
            serde_json::to_string_pretty(&entry_count).expect("serialize transcript entry-count")
        }
        CliCommand::TranscriptHasEntries { selector } => {
            let has_entries = engine
                .has_entries_session_transcript(&selector)
                .expect("has-entries persisted session transcript");
            serde_json::to_string_pretty(&has_entries).expect("serialize transcript has-entries")
        }
        CliCommand::TranscriptTurnExists { selector, turn } => {
            let turn_exists = engine
                .turn_exists_session_transcript(&selector, turn)
                .expect("turn-exists persisted session transcript");
            serde_json::to_string_pretty(&turn_exists).expect("serialize transcript turn-exists")
        }
        CliCommand::TranscriptTurnIndexes { selector } => {
            let turn_indexes = engine
                .turn_indexes_session_transcript(&selector)
                .expect("turn-indexes persisted session transcript");
            serde_json::to_string_pretty(&turn_indexes).expect("serialize transcript turn-indexes")
        }
        CliCommand::TranscriptTurnIndexRange { selector } => {
            let turn_range = engine
                .turn_range_session_transcript(&selector)
                .expect("turn-index-range persisted session transcript");
            serde_json::to_string_pretty(&turn_range)
                .expect("serialize transcript turn-index-range")
        }
        CliCommand::TranscriptHasTurnGaps { selector } => {
            let has_turn_gaps = engine
                .has_turn_gaps_session_transcript(&selector)
                .expect("has-turn-gaps persisted session transcript");
            serde_json::to_string_pretty(&has_turn_gaps)
                .expect("serialize transcript has-turn-gaps")
        }
        CliCommand::TranscriptMissingTurnIndexes { selector } => {
            let missing = engine
                .missing_turn_indexes_session_transcript(&selector)
                .expect("missing-turn-indexes persisted session transcript");
            serde_json::to_string_pretty(&missing)
                .expect("serialize transcript missing-turn-indexes")
        }
        CliCommand::TranscriptTurnDensity { selector } => {
            let density = engine
                .turn_density_session_transcript(&selector)
                .expect("turn-density persisted session transcript");
            serde_json::to_string_pretty(&density)
                .expect("serialize transcript turn-density")
        }
        CliCommand::TranscriptGapRanges { selector } => {
            let gap_ranges = engine
                .gap_ranges_session_transcript(&selector)
                .expect("gap-ranges persisted session transcript");
            serde_json::to_string_pretty(&gap_ranges)
                .expect("serialize transcript gap-ranges")
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let engine = RuntimeEngine::default();

    println!("{}", render_command(&engine, cli.command));
}

#[cfg(test)]
mod tests {
    use super::{render_command, Cli, CliCommand};
    use harness_commands::CommandRegistry;
    use harness_runtime::RuntimeEngine;
    use harness_session::{
        SessionComparison, SessionDeletion, SessionExport, SessionFindResult, SessionFork,
        SessionImport, SessionLabelEntry, SessionPin, SessionPinEntry, SessionPrune, SessionRename,
        SessionRetag, SessionSelectorCheck, SessionState, SessionStore, SessionTranscriptContext,
        SessionTranscriptEntryCount, SessionTranscriptFind, SessionTranscriptFirstTurn,
        SessionTranscriptGapRange, SessionTranscriptGapRanges, SessionTranscriptHasEntries,
        SessionTranscriptHasTurnGaps, SessionTranscriptLastTurn,
        SessionTranscriptMissingTurnIndexes, SessionTranscriptRange, SessionTranscriptTail,
        SessionTranscriptTurnDensity, SessionTranscriptTurnExists, SessionTranscriptTurnIndexes,
        SessionTranscriptTurnRange, SessionTranscriptTurnShow, SessionUnlabel, SessionUnpin,
        TranscriptEntry, TranscriptRecord,
        DEFAULT_TRANSCRIPT_CONTEXT_WINDOW, DEFAULT_TRANSCRIPT_RANGE_COUNT,
        DEFAULT_TRANSCRIPT_TAIL_COUNT,
    };
    use harness_tools::{PermissionPolicy, ToolRegistry};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    const README: &str = include_str!("../../../README.md");

    fn temp_session_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("harness-cli-tests-{nonce}"))
    }

    fn temp_engine(root: &Path) -> RuntimeEngine {
        RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(root),
        }
    }

    fn normalized_readme() -> String {
        README.replace("\r\n", "\n")
    }

    fn readme_output_block(heading: &str, language: &str) -> String {
        let readme = normalized_readme();
        let marker = format!("### `{heading}`");
        let section = readme
            .split(&marker)
            .nth(1)
            .and_then(|after_heading| after_heading.split("\n### ").next())
            .expect("README section should exist");
        let fence = format!("```{language}\n");
        let after_fence = section
            .split(&fence)
            .nth(1)
            .expect("README output block should exist");

        after_fence
            .split("\n```")
            .next()
            .expect("README output fence should close")
            .to_string()
    }

    fn normalize_timestamp_field(output: &str, field: &str, placeholder: &str) -> String {
        let marker = format!("\"{field}\": ");
        let mut remaining = output;
        let mut normalized = String::with_capacity(output.len());
        while let Some(start) = remaining.find(&marker) {
            let value_start = start + marker.len();
            let value_end = remaining[value_start..]
                .find(|ch: char| !ch.is_ascii_digit())
                .map(|offset| value_start + offset)
                .unwrap_or(remaining.len());

            normalized.push_str(&remaining[..value_start]);
            normalized.push_str(placeholder);
            remaining = &remaining[value_end..];
        }
        normalized.push_str(remaining);
        normalized
    }

    fn normalize_signed_number_field(output: &str, field: &str, placeholder: &str) -> String {
        let marker = format!("\"{field}\": ");
        let mut remaining = output;
        let mut normalized = String::with_capacity(output.len());
        while let Some(start) = remaining.find(&marker) {
            let value_start = start + marker.len();
            let scan_start = if remaining[value_start..].starts_with('-') {
                value_start + 1
            } else {
                value_start
            };
            let value_end = remaining[scan_start..]
                .find(|ch: char| !ch.is_ascii_digit())
                .map(|offset| scan_start + offset)
                .unwrap_or(remaining.len());

            normalized.push_str(&remaining[..value_start]);
            normalized.push_str(placeholder);
            remaining = &remaining[value_end..];
        }
        normalized.push_str(remaining);
        normalized
    }

    fn normalize_comparison_output(
        output: &str,
        left_session_id: &str,
        right_session_id: &str,
    ) -> String {
        let replaced = output
            .replace(left_session_id, "<left-session-id>")
            .replace(right_session_id, "<right-session-id>");
        let timestamps = normalize_timestamps(&replaced);
        let with_created_delta = normalize_signed_number_field(
            &timestamps,
            "created_at_ms_delta",
            "<created-at-ms-delta>",
        );
        normalize_signed_number_field(
            &with_created_delta,
            "updated_at_ms_delta",
            "<updated-at-ms-delta>",
        )
    }

    fn normalize_timestamps(output: &str) -> String {
        let stage_one = normalize_timestamp_field(output, "created_at_ms", "<created-at-ms>");
        normalize_timestamp_field(&stage_one, "updated_at_ms", "<updated-at-ms>")
    }

    fn normalize_bootstrap_example(output: &str, session_id: &str, root: &Path) -> String {
        normalize_timestamps(
            &output
                .replace(session_id, "<session-id>")
                .replace(root.to_string_lossy().as_ref(), ".sessions"),
        )
    }

    fn normalize_session_output(output: &str, session_id: &str) -> String {
        normalize_timestamps(&output.replace(session_id, "<session-id>"))
    }

    fn normalize_fork_output(
        output: &str,
        source_session_id: &str,
        forked_session_id: &str,
        root: &Path,
    ) -> String {
        let replaced = output
            .replace(root.to_string_lossy().as_ref(), ".sessions")
            .replace(forked_session_id, "<forked-session-id>")
            .replace(source_session_id, "<source-session-id>");
        normalize_timestamps(&replaced)
    }

    #[test]
    fn summary_matches_readme_example() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let output = render_command(&engine, CliCommand::Summary);

        assert_eq!(output, readme_output_block("summary", "text"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn route_matches_readme_example() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let output = render_command(
            &engine,
            CliCommand::Route {
                prompt: "review bash".to_string(),
            },
        );

        assert_eq!(output, readme_output_block("route \"review bash\"", "json"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn tools_match_readme_example() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let output = render_command(&engine, CliCommand::Tools);

        assert_eq!(output, readme_output_block("tools", "json"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn commands_match_readme_example() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let output = render_command(&engine, CliCommand::Commands);

        assert_eq!(output, readme_output_block("commands", "json"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn sessions_match_readme_example() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let sessions_output = render_command(&engine, CliCommand::Sessions);

        assert_eq!(
            normalize_bootstrap_example(&sessions_output, &session_id, &root),
            readme_output_block("sessions", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn bootstrap_and_session_show_match_readme_examples() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        assert_eq!(
            normalize_bootstrap_example(&bootstrap_output, &session_id, &root),
            readme_output_block("bootstrap \"review bash\"", "json")
        );

        let session_output = render_command(
            &engine,
            CliCommand::SessionShow {
                id: session_id.clone(),
            },
        );
        let session: SessionState =
            serde_json::from_str(&session_output).expect("parse session-show output");

        assert_eq!(session.session_id.to_string(), session_id);
        assert_eq!(
            normalize_session_output(&session_output, &session_id),
            readme_output_block("session-show <id>", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn resume_matches_readme_example_and_targets_existing_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let resume_output = render_command(
            &engine,
            CliCommand::Resume {
                id: session_id.clone(),
                prompt: "review summary".to_string(),
            },
        );

        let resume_json: serde_json::Value =
            serde_json::from_str(&resume_output).expect("parse resume report");
        assert_eq!(
            resume_json["resumed_session_id"].as_str(),
            Some(session_id.as_str()),
            "resume report must confirm the targeted session id"
        );
        assert_eq!(
            resume_json["appended_turn_index"].as_u64(),
            Some(1),
            "resume report must expose the appended turn index"
        );

        assert_eq!(
            normalize_bootstrap_example(&resume_output, &session_id, &root),
            readme_output_block("resume <id> \"review summary\"", "json")
        );

        let reloaded_output = render_command(
            &engine,
            CliCommand::SessionShow {
                id: session_id.clone(),
            },
        );
        let reloaded: SessionState =
            serde_json::from_str(&reloaded_output).expect("parse reloaded session");
        let reloaded_messages: Vec<String> = reloaded
            .messages
            .iter()
            .map(|prompt| prompt.0.clone())
            .collect();
        assert_eq!(
            reloaded_messages,
            vec!["review bash".to_string(), "review summary".to_string()],
            "resume must append to the existing persisted session"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_show_matches_readme_example_and_confirms_session_id() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let transcript_output = render_command(
            &engine,
            CliCommand::TranscriptShow {
                id: session_id.clone(),
            },
        );

        let transcript: TranscriptRecord =
            serde_json::from_str(&transcript_output).expect("parse transcript-show output");
        assert_eq!(
            transcript.session_id.to_string(),
            session_id,
            "transcript output must confirm the targeted session id"
        );
        let ordered: Vec<(usize, String)> = transcript
            .entries
            .iter()
            .map(|entry| (entry.turn_index.0, entry.prompt.0.clone()))
            .collect();
        assert_eq!(
            ordered,
            vec![(0, "review bash".to_string())],
            "transcript output must preserve turn ordering"
        );

        assert_eq!(
            normalize_session_output(&transcript_output, &session_id),
            readme_output_block("transcript-show <id>", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_show_latest_matches_readme_example() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let latest_output = render_command(
            &engine,
            CliCommand::TranscriptShow {
                id: "latest".to_string(),
            },
        );

        let latest: TranscriptRecord =
            serde_json::from_str(&latest_output).expect("parse transcript-show latest output");
        assert_eq!(latest.session_id.to_string(), session_id);

        assert_eq!(
            normalize_session_output(&latest_output, &session_id),
            readme_output_block("transcript-show latest", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_export_matches_readme_example_and_confirms_session_id() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let export_output = render_command(
            &engine,
            CliCommand::SessionExport {
                id: session_id.clone(),
            },
        );

        let export: SessionExport =
            serde_json::from_str(&export_output).expect("parse session-export output");
        assert_eq!(
            export.exported_session_id.to_string(),
            session_id,
            "export output must confirm the targeted session id"
        );
        assert_eq!(
            export.session.session_id.to_string(),
            session_id,
            "nested session id must match the exported id"
        );
        assert_eq!(
            export.transcript.session_id.to_string(),
            session_id,
            "nested transcript session id must match the exported id"
        );
        let ordered: Vec<(usize, String)> = export
            .transcript
            .entries
            .iter()
            .map(|entry| (entry.turn_index.0, entry.prompt.0.clone()))
            .collect();
        assert_eq!(
            ordered,
            vec![(0, "review bash".to_string())],
            "exported transcript must preserve turn ordering"
        );

        assert_eq!(
            normalize_session_output(&export_output, &session_id),
            readme_output_block("session-export <id>", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_export_latest_matches_readme_example() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let latest_output = render_command(
            &engine,
            CliCommand::SessionExport {
                id: "latest".to_string(),
            },
        );

        let latest: SessionExport =
            serde_json::from_str(&latest_output).expect("parse session-export latest output");
        assert_eq!(latest.exported_session_id.to_string(), session_id);

        assert_eq!(
            normalize_session_output(&latest_output, &session_id),
            readme_output_block("session-export latest", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_compare_matches_readme_example_and_identifies_both_sessions() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let left_bootstrap = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let left_json: serde_json::Value =
            serde_json::from_str(&left_bootstrap).expect("parse left bootstrap report");
        let left_id = left_json["session"]["session_id"]
            .as_str()
            .expect("left session id")
            .to_string();

        std::thread::sleep(std::time::Duration::from_millis(2));

        let right_bootstrap = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "summary please".to_string(),
            },
        );
        let right_json: serde_json::Value =
            serde_json::from_str(&right_bootstrap).expect("parse right bootstrap report");
        let right_id = right_json["session"]["session_id"]
            .as_str()
            .expect("right session id")
            .to_string();

        let compare_output = render_command(
            &engine,
            CliCommand::SessionCompare {
                left: left_id.clone(),
                right: right_id.clone(),
            },
        );

        let comparison: SessionComparison =
            serde_json::from_str(&compare_output).expect("parse session-compare output");
        assert_eq!(
            comparison.left_session_id.to_string(),
            left_id,
            "left_session_id must match the requested left target"
        );
        assert_eq!(
            comparison.right_session_id.to_string(),
            right_id,
            "right_session_id must match the requested right target"
        );
        assert!(!comparison.differences.same_session);
        assert_eq!(comparison.differences.message_count_delta, 0);
        assert_eq!(comparison.differences.transcript_entry_count_delta, 0);

        assert_eq!(
            normalize_comparison_output(&compare_output, &left_id, &right_id),
            readme_output_block("session-compare <left-id> <right-id>", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_compare_latest_latest_matches_readme_example_and_is_self_comparison() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let compare_output = render_command(
            &engine,
            CliCommand::SessionCompare {
                left: "latest".to_string(),
                right: "latest".to_string(),
            },
        );

        let comparison: SessionComparison =
            serde_json::from_str(&compare_output).expect("parse session-compare latest output");
        assert_eq!(comparison.left_session_id.to_string(), session_id);
        assert_eq!(comparison.right_session_id.to_string(), session_id);
        assert!(comparison.differences.same_session);
        assert_eq!(comparison.differences.created_at_ms_delta, 0);
        assert_eq!(comparison.differences.updated_at_ms_delta, 0);
        assert_eq!(comparison.differences.message_count_delta, 0);
        assert_eq!(comparison.differences.transcript_entry_count_delta, 0);

        assert_eq!(
            normalize_session_output(&compare_output, &session_id),
            readme_output_block("session-compare latest latest", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_delete_matches_readme_example_and_removes_persisted_artifacts() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let delete_output = render_command(
            &engine,
            CliCommand::SessionDelete {
                id: session_id.clone(),
            },
        );

        let deletion: SessionDeletion =
            serde_json::from_str(&delete_output).expect("parse session-delete output");
        assert_eq!(deletion.deleted_session_id.to_string(), session_id);
        assert_eq!(deletion.removed_paths.len(), 2);
        for path in &deletion.removed_paths {
            assert!(
                !std::path::Path::new(path).exists(),
                "removed path must not exist"
            );
        }

        assert_eq!(
            normalize_bootstrap_example(&delete_output, &session_id, &root),
            readme_output_block("session-delete <id>", "json")
        );

        let after = engine.list_sessions().expect("list after delete");
        assert!(
            after.is_empty(),
            "session listing must be empty after delete"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_delete_latest_matches_readme_example_and_targets_most_recent() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let latest_output = render_command(
            &engine,
            CliCommand::SessionDelete {
                id: "latest".to_string(),
            },
        );

        let deletion: SessionDeletion =
            serde_json::from_str(&latest_output).expect("parse session-delete latest output");
        assert_eq!(deletion.deleted_session_id.to_string(), session_id);

        assert_eq!(
            normalize_bootstrap_example(&latest_output, &session_id, &root),
            readme_output_block("session-delete latest", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_import_matches_readme_example_and_restores_bundle_into_fresh_store() {
        let source_root = temp_session_root();
        let source_engine = temp_engine(&source_root);
        let bootstrap_output = render_command(
            &source_engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let export_output = render_command(
            &source_engine,
            CliCommand::SessionExport {
                id: session_id.clone(),
            },
        );
        let bundle_path =
            std::env::temp_dir().join(format!("harness-cli-import-bundle-{session_id}.json"));
        fs::write(&bundle_path, &export_output).expect("write bundle file");

        let target_root = temp_session_root();
        let target_engine = temp_engine(&target_root);

        let import_output = render_command(
            &target_engine,
            CliCommand::SessionImport {
                path: bundle_path.to_string_lossy().into_owned(),
            },
        );

        let imported: SessionImport =
            serde_json::from_str(&import_output).expect("parse session-import output");
        assert_eq!(
            imported.imported_session_id.to_string(),
            session_id,
            "import output must confirm the imported session id"
        );
        assert_eq!(
            imported.session_path,
            target_root
                .join(format!("{session_id}.json"))
                .display()
                .to_string()
        );
        assert_eq!(
            imported.transcript_path,
            target_root
                .join(format!("{session_id}.transcript.json"))
                .display()
                .to_string()
        );
        assert!(Path::new(&imported.session_path).exists());
        assert!(Path::new(&imported.transcript_path).exists());

        let reloaded = target_engine
            .load_session(&session_id)
            .expect("reload imported session");
        assert_eq!(reloaded.session_id.to_string(), session_id);
        let reloaded_transcript = target_engine
            .load_transcript(&session_id)
            .expect("reload imported transcript");
        let ordered: Vec<(usize, String)> = reloaded_transcript
            .entries
            .iter()
            .map(|entry| (entry.turn_index.0, entry.prompt.0.clone()))
            .collect();
        assert_eq!(
            ordered,
            vec![(0, "review bash".to_string())],
            "imported transcript must preserve turn ordering"
        );

        assert_eq!(
            normalize_bootstrap_example(&import_output, &session_id, &target_root),
            readme_output_block("session-import <bundle-path>", "json")
        );

        fs::remove_file(&bundle_path).ok();
        fs::remove_dir_all(&source_root).ok();
        fs::remove_dir_all(&target_root).ok();
    }

    #[test]
    fn session_import_fails_cleanly_for_missing_bundle_without_persisting_anything() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let missing =
            std::env::temp_dir().join("harness-cli-import-bundle-definitely-missing-xyzzy.json");
        let _ = fs::remove_file(&missing);

        let result = engine.import_session(missing.to_string_lossy().as_ref());
        assert!(result.is_err(), "missing bundle path must fail");
        assert!(
            engine
                .list_sessions()
                .expect("list sessions after failed import")
                .is_empty(),
            "no persisted sessions should be written when import fails"
        );

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn session_find_matches_readme_example_and_identifies_matched_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let find_output = render_command(
            &engine,
            CliCommand::SessionFind {
                query: "review".to_string(),
            },
        );

        let results: Vec<SessionFindResult> =
            serde_json::from_str(&find_output).expect("parse session-find output");
        assert_eq!(
            results.len(),
            1,
            "exactly one persisted session should match"
        );
        assert_eq!(results[0].session_id.to_string(), session_id);
        assert_eq!(results[0].matches.len(), 1);
        assert_eq!(results[0].matches[0].turn_index.0, 0);
        assert_eq!(results[0].matches[0].prompt.0, "review bash");

        assert_eq!(
            normalize_bootstrap_example(&find_output, &session_id, &root),
            readme_output_block("session-find <query>", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_find_empty_result_matches_readme_example() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _ = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );

        let find_output = render_command(
            &engine,
            CliCommand::SessionFind {
                query: "definitely-not-present".to_string(),
            },
        );

        let results: Vec<SessionFindResult> =
            serde_json::from_str(&find_output).expect("parse session-find empty output");
        assert!(results.is_empty(), "no matches should yield an empty array");

        assert_eq!(
            find_output,
            readme_output_block("session-find <unmatched-query>", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_fork_matches_readme_example_and_creates_fresh_session_id() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let source_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let fork_output = render_command(
            &engine,
            CliCommand::SessionFork {
                id: source_id.clone(),
                prompt: "try again".to_string(),
            },
        );

        let fork: SessionFork =
            serde_json::from_str(&fork_output).expect("parse session-fork output");
        assert_eq!(
            fork.source_session_id.to_string(),
            source_id,
            "source_session_id must match the targeted source"
        );
        assert_ne!(
            fork.forked_session_id.to_string(),
            source_id,
            "forked session id must differ from source"
        );
        assert_eq!(fork.appended_turn_index.0, 1);
        assert_eq!(
            fork.session_path,
            root.join(format!("{}.json", fork.forked_session_id))
                .display()
                .to_string()
        );
        assert_eq!(
            fork.transcript_path,
            root.join(format!("{}.transcript.json", fork.forked_session_id))
                .display()
                .to_string()
        );
        assert!(Path::new(&fork.session_path).exists());
        assert!(Path::new(&fork.transcript_path).exists());

        let forked: SessionState = engine
            .load_session(&fork.forked_session_id.to_string())
            .expect("reload forked session");
        let forked_messages: Vec<String> = forked
            .messages
            .iter()
            .map(|prompt| prompt.0.clone())
            .collect();
        assert_eq!(
            forked_messages,
            vec!["review bash".to_string(), "try again".to_string()],
            "forked session must carry source messages then append the new prompt"
        );

        let source: SessionState = engine
            .load_session(&source_id)
            .expect("source session must still load");
        let source_messages: Vec<String> = source
            .messages
            .iter()
            .map(|prompt| prompt.0.clone())
            .collect();
        assert_eq!(
            source_messages,
            vec!["review bash".to_string()],
            "source session must not be mutated by fork"
        );

        let forked_id = fork.forked_session_id.to_string();
        assert_eq!(
            normalize_fork_output(&fork_output, &source_id, &forked_id, &root),
            readme_output_block("session-fork <source-session-id> \"try again\"", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_fork_latest_matches_readme_example_and_targets_most_recent() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let source_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let latest_output = render_command(
            &engine,
            CliCommand::SessionFork {
                id: "latest".to_string(),
                prompt: "try again".to_string(),
            },
        );

        let fork: SessionFork =
            serde_json::from_str(&latest_output).expect("parse session-fork latest output");
        assert_eq!(fork.source_session_id.to_string(), source_id);
        assert_ne!(fork.forked_session_id.to_string(), source_id);

        let forked_id = fork.forked_session_id.to_string();
        assert_eq!(
            normalize_fork_output(&latest_output, &source_id, &forked_id, &root),
            readme_output_block("session-fork latest \"try again\"", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_rename_matches_readme_example_and_persists_label_without_mutating_transcript() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let transcript_before = engine
            .load_transcript(&session_id)
            .expect("load transcript before rename");
        let session_before = engine
            .load_session(&session_id)
            .expect("load session before rename");

        let rename_output = render_command(
            &engine,
            CliCommand::SessionRename {
                id: session_id.clone(),
                label: "runtime-review".to_string(),
            },
        );

        let renamed: SessionRename =
            serde_json::from_str(&rename_output).expect("parse session-rename output");
        assert_eq!(renamed.renamed_session_id.to_string(), session_id);
        assert_eq!(renamed.applied_label, "runtime-review");

        let reloaded = engine
            .load_session(&session_id)
            .expect("reload session after rename");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));
        assert_eq!(
            reloaded.updated_at_ms, session_before.updated_at_ms,
            "rename must not bump activity metadata"
        );
        assert_eq!(reloaded.messages, session_before.messages);

        let transcript_after = engine
            .load_transcript(&session_id)
            .expect("reload transcript after rename");
        assert_eq!(
            transcript_after, transcript_before,
            "rename must not mutate transcript entries or ordering"
        );

        assert_eq!(
            normalize_session_output(&rename_output, &session_id),
            readme_output_block("session-rename <id> <label>", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_rename_latest_matches_readme_example_and_targets_most_recent() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let latest_output = render_command(
            &engine,
            CliCommand::SessionRename {
                id: "latest".to_string(),
                label: "runtime-review".to_string(),
            },
        );

        let renamed: SessionRename =
            serde_json::from_str(&latest_output).expect("parse session-rename latest output");
        assert_eq!(renamed.renamed_session_id.to_string(), session_id);
        assert_eq!(renamed.applied_label, "runtime-review");

        assert_eq!(
            normalize_session_output(&latest_output, &session_id),
            readme_output_block("session-rename latest <label>", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_rename_rejects_invalid_label_and_unknown_id_without_touching_other_sessions() {
        use harness_core::SessionId;

        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        for invalid in ["", "   ", "\t\n"] {
            let result = engine.rename_session(&session_id, invalid);
            assert!(result.is_err(), "empty/whitespace label must fail");
        }

        let missing = SessionId::new().to_string();
        assert!(
            engine.rename_session(&missing, "anything").is_err(),
            "unknown session id must fail"
        );

        let reloaded = engine
            .load_session(&session_id)
            .expect("reload session after failed renames");
        assert!(
            reloaded.label.is_none(),
            "rejected renames must not persist a label"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_rename_preserves_unlabeled_json_shape_for_session_show() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let unlabeled_show = render_command(
            &engine,
            CliCommand::SessionShow {
                id: session_id.clone(),
            },
        );
        assert!(
            !unlabeled_show.contains("\"label\""),
            "unlabeled session-show output must not emit the label field: {unlabeled_show}"
        );

        let _ = render_command(
            &engine,
            CliCommand::SessionRename {
                id: session_id.clone(),
                label: "runtime-review".to_string(),
            },
        );

        let labeled_show = render_command(
            &engine,
            CliCommand::SessionShow {
                id: session_id.clone(),
            },
        );
        assert!(
            labeled_show.contains("\"label\": \"runtime-review\""),
            "labeled session-show output must expose the label field: {labeled_show}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn label_selector_targets_persisted_session_via_session_show_and_emits_resolved_id_in_json() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let _ = render_command(
            &engine,
            CliCommand::SessionRename {
                id: session_id.clone(),
                label: "runtime-review".to_string(),
            },
        );

        // Raw-id targeting must keep working unchanged after the rename.
        let by_raw_id = render_command(
            &engine,
            CliCommand::SessionShow {
                id: session_id.clone(),
            },
        );
        let raw_state: SessionState =
            serde_json::from_str(&by_raw_id).expect("parse raw-id session-show");
        assert_eq!(raw_state.session_id.to_string(), session_id);

        // Label selector returns the same persisted session and the JSON
        // identifies the actual resolved session_id, not the label string.
        let by_label = render_command(
            &engine,
            CliCommand::SessionShow {
                id: "label:runtime-review".to_string(),
            },
        );
        let labeled_state: SessionState =
            serde_json::from_str(&by_label).expect("parse label session-show");
        assert_eq!(labeled_state.session_id.to_string(), session_id);
        assert!(
            !by_label.contains("label:runtime-review"),
            "JSON output must not echo the selector string in place of the resolved id: {by_label}"
        );
        assert!(
            by_label.contains(&session_id),
            "JSON output must surface the resolved session_id: {by_label}"
        );

        // session-compare also accepts the label selector on either side and
        // resolves to real session_ids in the bundle.
        let second_bootstrap = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "summary please".to_string(),
            },
        );
        let second_json: serde_json::Value =
            serde_json::from_str(&second_bootstrap).expect("parse second bootstrap");
        let second_id = second_json["session"]["session_id"]
            .as_str()
            .expect("second session id")
            .to_string();

        let compare_output = render_command(
            &engine,
            CliCommand::SessionCompare {
                left: "label:runtime-review".to_string(),
                right: "latest".to_string(),
            },
        );
        let comparison: SessionComparison =
            serde_json::from_str(&compare_output).expect("parse compare output");
        assert_eq!(comparison.left_session_id.to_string(), session_id);
        assert_eq!(comparison.right_session_id.to_string(), second_id);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn label_selector_unknown_and_ambiguous_inputs_fail_cleanly_via_runtime_layer() {
        use harness_core::Prompt;

        let root = temp_session_root();
        let engine = temp_engine(&root);

        // Unknown label surfaces from the engine before any persisted state is
        // touched. We test via the engine API to assert on the error string
        // shape — render_command panics on error by design.
        let unknown = engine.load_session("label:missing");
        assert!(unknown.is_err(), "unknown label must fail cleanly");
        assert!(
            unknown.unwrap_err().contains("label:missing"),
            "unknown-label error must echo the typed selector"
        );

        // Two persisted sessions sharing a label is ambiguous.
        let one = engine
            .bootstrap(Prompt::new("alpha"))
            .expect("bootstrap one");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let two = engine
            .bootstrap(Prompt::new("beta"))
            .expect("bootstrap two");
        engine
            .rename_session(&one.session.session_id.to_string(), "dup")
            .expect("label one");
        engine
            .rename_session(&two.session.session_id.to_string(), "dup")
            .expect("label two");
        let ambiguous = engine.load_session("label:dup");
        assert!(ambiguous.is_err(), "ambiguous label must fail cleanly");
        assert!(
            ambiguous.unwrap_err().contains("ambiguous session label"),
            "ambiguous-label error must mention ambiguity"
        );

        // `label:` with no name is malformed.
        let malformed = engine.load_session("label:");
        assert!(malformed.is_err(), "malformed selector must fail cleanly");
        assert!(
            malformed
                .unwrap_err()
                .contains("malformed session selector"),
            "malformed-selector error must mention malformed selector"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_labels_matches_readme_example_and_orders_newest_first_omits_unlabeled_and_keeps_duplicates_separate(
    ) {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        // Three persisted sessions: only two end up labeled, and the newest
        // of those is labeled second so the listing must flip activity order.
        let older_out = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let older_id = serde_json::from_str::<serde_json::Value>(&older_out)
            .expect("parse older bootstrap")["session"]["session_id"]
            .as_str()
            .expect("older session id")
            .to_string();
        std::thread::sleep(std::time::Duration::from_millis(2));

        let middle_out = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "no label here".to_string(),
            },
        );
        let middle_id = serde_json::from_str::<serde_json::Value>(&middle_out)
            .expect("parse middle bootstrap")["session"]["session_id"]
            .as_str()
            .expect("middle session id")
            .to_string();
        std::thread::sleep(std::time::Duration::from_millis(2));

        let newest_out = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "summary please".to_string(),
            },
        );
        let newest_id = serde_json::from_str::<serde_json::Value>(&newest_out)
            .expect("parse newest bootstrap")["session"]["session_id"]
            .as_str()
            .expect("newest session id")
            .to_string();

        // Only label the older and the newest; middle stays unlabeled.
        let _ = render_command(
            &engine,
            CliCommand::SessionRename {
                id: older_id.clone(),
                label: "runtime-review".to_string(),
            },
        );
        let _ = render_command(
            &engine,
            CliCommand::SessionRename {
                id: newest_id.clone(),
                label: "release-candidate".to_string(),
            },
        );

        let labels_output = render_command(&engine, CliCommand::SessionLabels);

        let entries: Vec<SessionLabelEntry> =
            serde_json::from_str(&labels_output).expect("parse session-labels output");
        assert_eq!(entries.len(), 2, "only labeled sessions must appear");
        assert_eq!(
            entries[0].session_id.to_string(),
            newest_id,
            "newest labeled session must come first"
        );
        assert_eq!(entries[0].label, "release-candidate");
        assert_eq!(entries[1].session_id.to_string(), older_id);
        assert_eq!(entries[1].label, "runtime-review");
        assert!(
            !entries
                .iter()
                .any(|entry| entry.session_id.to_string() == middle_id),
            "unlabeled session must be omitted"
        );

        // The README example shows the simpler single-labeled-session case, so
        // re-run against a fresh store to compare deterministic output.
        fs::remove_dir_all(&root).expect("remove temp cli test directory");
        let readme_root = temp_session_root();
        let readme_engine = temp_engine(&readme_root);
        let bootstrap_output = render_command(
            &readme_engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let session_id = serde_json::from_str::<serde_json::Value>(&bootstrap_output)
            .expect("parse bootstrap")["session"]["session_id"]
            .as_str()
            .expect("session id")
            .to_string();
        let _ = render_command(
            &readme_engine,
            CliCommand::SessionRename {
                id: session_id.clone(),
                label: "runtime-review".to_string(),
            },
        );
        let labels_output = render_command(&readme_engine, CliCommand::SessionLabels);

        assert_eq!(
            normalize_bootstrap_example(&labels_output, &session_id, &readme_root),
            readme_output_block("session-labels", "json")
        );

        fs::remove_dir_all(&readme_root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_labels_keeps_duplicate_labels_as_separate_rows() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first_out = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "alpha".to_string(),
            },
        );
        let first_id = serde_json::from_str::<serde_json::Value>(&first_out)
            .expect("parse first bootstrap")["session"]["session_id"]
            .as_str()
            .expect("first session id")
            .to_string();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second_out = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "beta".to_string(),
            },
        );
        let second_id = serde_json::from_str::<serde_json::Value>(&second_out)
            .expect("parse second bootstrap")["session"]["session_id"]
            .as_str()
            .expect("second session id")
            .to_string();

        let _ = render_command(
            &engine,
            CliCommand::SessionRename {
                id: first_id.clone(),
                label: "dup".to_string(),
            },
        );
        let _ = render_command(
            &engine,
            CliCommand::SessionRename {
                id: second_id.clone(),
                label: "dup".to_string(),
            },
        );

        let labels_output = render_command(&engine, CliCommand::SessionLabels);
        let entries: Vec<SessionLabelEntry> =
            serde_json::from_str(&labels_output).expect("parse session-labels output");
        assert_eq!(
            entries.len(),
            2,
            "duplicate labels must stay as separate rows, not be collapsed"
        );
        assert!(entries.iter().all(|entry| entry.label == "dup"));
        assert_eq!(entries[0].session_id.to_string(), second_id);
        assert_eq!(entries[1].session_id.to_string(), first_id);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_labels_empty_store_matches_readme_empty_example() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        // Truly empty store: no persisted sessions at all.
        let output = render_command(&engine, CliCommand::SessionLabels);
        assert_eq!(
            output,
            readme_output_block("session-labels <empty-store>", "json"),
            "empty store must emit the README-backed empty JSON array"
        );

        // Store with only unlabeled persisted sessions must behave identically.
        let _ = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "no label".to_string(),
            },
        );
        let output = render_command(&engine, CliCommand::SessionLabels);
        assert_eq!(
            output,
            readme_output_block("session-labels <empty-store>", "json"),
            "unlabeled-only store must also emit the README-backed empty JSON array"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_pins_orders_newest_first_omits_unpinned_and_surfaces_optional_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        // Three persisted sessions. Only the older and newest get pinned, and
        // only the older carries a label so the listing proves:
        //   - newest-first ordering across pinned rows
        //   - omission of the unpinned middle session
        //   - optional `label` surfacing (unlabeled pinned row omits `label`)
        let older_out = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let older_id = serde_json::from_str::<serde_json::Value>(&older_out)
            .expect("parse older bootstrap")["session"]["session_id"]
            .as_str()
            .expect("older session id")
            .to_string();
        std::thread::sleep(std::time::Duration::from_millis(2));

        let middle_out = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "unpinned middle".to_string(),
            },
        );
        let middle_id = serde_json::from_str::<serde_json::Value>(&middle_out)
            .expect("parse middle bootstrap")["session"]["session_id"]
            .as_str()
            .expect("middle session id")
            .to_string();
        std::thread::sleep(std::time::Duration::from_millis(2));

        let newest_out = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "summary please".to_string(),
            },
        );
        let newest_id = serde_json::from_str::<serde_json::Value>(&newest_out)
            .expect("parse newest bootstrap")["session"]["session_id"]
            .as_str()
            .expect("newest session id")
            .to_string();

        let _ = render_command(
            &engine,
            CliCommand::SessionRename {
                id: older_id.clone(),
                label: "runtime-review".to_string(),
            },
        );
        let _ = render_command(
            &engine,
            CliCommand::SessionPin {
                id: older_id.clone(),
            },
        );
        let _ = render_command(
            &engine,
            CliCommand::SessionPin {
                id: newest_id.clone(),
            },
        );

        let pins_output = render_command(&engine, CliCommand::SessionPins);
        let entries: Vec<SessionPinEntry> =
            serde_json::from_str(&pins_output).expect("parse session-pins output");
        assert_eq!(entries.len(), 2, "only pinned sessions must appear");
        assert_eq!(
            entries[0].session_id.to_string(),
            newest_id,
            "newest pinned session must come first"
        );
        assert!(entries[0].pinned, "pinned row must report pinned: true");
        assert_eq!(
            entries[0].label, None,
            "unlabeled pinned session must omit `label`"
        );
        assert_eq!(entries[1].session_id.to_string(), older_id);
        assert!(entries[1].pinned);
        assert_eq!(
            entries[1].label.as_deref(),
            Some("runtime-review"),
            "labeled pinned session must surface `label`"
        );
        assert!(
            !entries
                .iter()
                .any(|entry| entry.session_id.to_string() == middle_id),
            "unpinned session must be omitted"
        );

        // The README example shows a single labeled pinned session, so re-run
        // against a fresh store to compare deterministic output.
        fs::remove_dir_all(&root).expect("remove temp cli test directory");
        let readme_root = temp_session_root();
        let readme_engine = temp_engine(&readme_root);
        let bootstrap_output = render_command(
            &readme_engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let session_id = serde_json::from_str::<serde_json::Value>(&bootstrap_output)
            .expect("parse bootstrap")["session"]["session_id"]
            .as_str()
            .expect("session id")
            .to_string();
        let _ = render_command(
            &readme_engine,
            CliCommand::SessionRename {
                id: session_id.clone(),
                label: "runtime-review".to_string(),
            },
        );
        let _ = render_command(
            &readme_engine,
            CliCommand::SessionPin {
                id: session_id.clone(),
            },
        );
        let pins_output = render_command(&readme_engine, CliCommand::SessionPins);

        assert_eq!(
            normalize_bootstrap_example(&pins_output, &session_id, &readme_root),
            readme_output_block("session-pins", "json")
        );

        fs::remove_dir_all(&readme_root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_pins_empty_store_matches_readme_empty_example() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        // Truly empty store: no persisted sessions at all.
        let output = render_command(&engine, CliCommand::SessionPins);
        assert_eq!(
            output,
            readme_output_block("session-pins <empty-store>", "json"),
            "empty store must emit the README-backed empty JSON array"
        );

        // Store with only unpinned persisted sessions must behave identically.
        let _ = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "no pin".to_string(),
            },
        );
        let output = render_command(&engine, CliCommand::SessionPins);
        assert_eq!(
            output,
            readme_output_block("session-pins <empty-store>", "json"),
            "unpinned-only store must also emit the README-backed empty JSON array"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_unlabel_explicit_id_matches_readme_example_and_clears_label_without_touching_transcript(
    ) {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let _ = render_command(
            &engine,
            CliCommand::SessionRename {
                id: session_id.clone(),
                label: "runtime-review".to_string(),
            },
        );

        let session_before = engine
            .load_session(&session_id)
            .expect("load session after rename");
        let transcript_before = engine
            .load_transcript(&session_id)
            .expect("load transcript after rename");
        assert_eq!(session_before.label.as_deref(), Some("runtime-review"));

        let unlabel_output = render_command(
            &engine,
            CliCommand::SessionUnlabel {
                id: session_id.clone(),
            },
        );

        let unlabeled: SessionUnlabel =
            serde_json::from_str(&unlabel_output).expect("parse session-unlabel output");
        assert_eq!(unlabeled.unlabeled_session_id.to_string(), session_id);
        assert_eq!(unlabeled.removed_label, "runtime-review");

        let reloaded = engine
            .load_session(&session_id)
            .expect("reload session after unlabel");
        assert!(
            reloaded.label.is_none(),
            "unlabel must clear the persisted label"
        );
        assert_eq!(
            reloaded.updated_at_ms, session_before.updated_at_ms,
            "unlabel must not bump activity metadata"
        );
        assert_eq!(reloaded.messages, session_before.messages);

        let transcript_after = engine
            .load_transcript(&session_id)
            .expect("reload transcript after unlabel");
        assert_eq!(
            transcript_after, transcript_before,
            "unlabel must not mutate transcript entries or ordering"
        );

        let persisted_body = fs::read_to_string(root.join(format!("{session_id}.json")))
            .expect("read persisted session json");
        assert!(
            !persisted_body.contains("\"label\""),
            "unlabeled session must not serialize a null/empty label field: {persisted_body}"
        );

        assert_eq!(
            normalize_session_output(&unlabel_output, &session_id),
            readme_output_block("session-unlabel <id>", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_unlabel_latest_selector_resolves_to_most_recent_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();
        let _ = render_command(
            &engine,
            CliCommand::SessionRename {
                id: session_id.clone(),
                label: "runtime-review".to_string(),
            },
        );

        let latest_output = render_command(
            &engine,
            CliCommand::SessionUnlabel {
                id: "latest".to_string(),
            },
        );
        let result: SessionUnlabel =
            serde_json::from_str(&latest_output).expect("parse latest unlabel output");
        assert_eq!(result.unlabeled_session_id.to_string(), session_id);
        assert_eq!(result.removed_label, "runtime-review");

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_unlabel_label_selector_resolves_to_labeled_session_and_disappears_from_session_labels(
    ) {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();
        let _ = render_command(
            &engine,
            CliCommand::SessionRename {
                id: session_id.clone(),
                label: "runtime-review".to_string(),
            },
        );

        let transcript_before = engine
            .load_transcript(&session_id)
            .expect("load transcript before label-selector unlabel");

        // The labeled session shows up in session-labels before unlabel.
        let labels_before = render_command(&engine, CliCommand::SessionLabels);
        let labels_before_array: Vec<SessionLabelEntry> =
            serde_json::from_str(&labels_before).expect("parse session-labels before");
        assert_eq!(labels_before_array.len(), 1);
        assert_eq!(labels_before_array[0].label, "runtime-review");

        let label_output = render_command(
            &engine,
            CliCommand::SessionUnlabel {
                id: "label:runtime-review".to_string(),
            },
        );
        let result: SessionUnlabel =
            serde_json::from_str(&label_output).expect("parse label-selector unlabel output");
        assert_eq!(
            result.unlabeled_session_id.to_string(),
            session_id,
            "label selector must resolve to the labeled session id, not the selector string"
        );
        assert_eq!(result.removed_label, "runtime-review");
        assert!(
            !label_output.contains("label:runtime-review"),
            "JSON output must surface the resolved session id, not the selector string: {label_output}"
        );

        // The unlabeled session disappears from session-labels.
        let labels_after = render_command(&engine, CliCommand::SessionLabels);
        assert_eq!(
            labels_after.trim(),
            "[]",
            "unlabeled session must disappear from session-labels"
        );

        // Transcript and session content are unchanged aside from the cleared label.
        let transcript_after = engine
            .load_transcript(&session_id)
            .expect("reload transcript after unlabel");
        assert_eq!(
            transcript_after, transcript_before,
            "unlabel must not mutate transcript entries or ordering"
        );
        let reloaded = engine
            .load_session(&session_id)
            .expect("reload session after unlabel");
        assert!(reloaded.label.is_none());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_unlabel_unknown_session_and_already_unlabeled_fail_cleanly_without_touching_store() {
        use harness_core::SessionId;

        let root = temp_session_root();
        let engine = temp_engine(&root);

        // Seed one labeled and one unlabeled session so we can prove neither
        // is mutated by a failing unlabel.
        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let missing = SessionId::new().to_string();
        let unknown_err = engine
            .unlabel_session(&missing)
            .expect_err("unknown session id must fail");
        assert!(
            unknown_err.contains("session not found"),
            "unknown id must surface SessionNotFound diagnostic: {unknown_err}"
        );

        let already_err = engine
            .unlabel_session(&session_id)
            .expect_err("already-unlabeled session must fail");
        assert!(
            already_err.contains("session already unlabeled"),
            "already-unlabeled session must surface the distinct diagnostic: {already_err}"
        );
        assert!(
            already_err.contains(&session_id),
            "already-unlabeled error must surface the resolved session id: {already_err}"
        );

        // The seeded session is still loadable and still unlabeled; failed
        // unlabel must not mutate anything.
        let reloaded = engine
            .load_session(&session_id)
            .expect("reload session after failed unlabel");
        assert!(reloaded.label.is_none());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_retag_explicit_id_matches_readme_example_and_replaces_label_without_touching_transcript(
    ) {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let _ = render_command(
            &engine,
            CliCommand::SessionRename {
                id: session_id.clone(),
                label: "runtime-review".to_string(),
            },
        );

        let session_before = engine
            .load_session(&session_id)
            .expect("load session after rename");
        let transcript_before = engine
            .load_transcript(&session_id)
            .expect("load transcript after rename");
        assert_eq!(session_before.label.as_deref(), Some("runtime-review"));

        let retag_output = render_command(
            &engine,
            CliCommand::SessionRetag {
                id: session_id.clone(),
                label: "release-candidate".to_string(),
            },
        );

        let retagged: SessionRetag =
            serde_json::from_str(&retag_output).expect("parse session-retag output");
        assert_eq!(retagged.retagged_session_id.to_string(), session_id);
        assert_eq!(retagged.previous_label, "runtime-review");
        assert_eq!(retagged.applied_label, "release-candidate");

        let reloaded = engine
            .load_session(&session_id)
            .expect("reload session after retag");
        assert_eq!(reloaded.label.as_deref(), Some("release-candidate"));
        assert_eq!(
            reloaded.updated_at_ms, session_before.updated_at_ms,
            "retag must not bump activity metadata"
        );
        assert_eq!(
            reloaded.created_at_ms, session_before.created_at_ms,
            "retag must not rewrite creation metadata"
        );
        assert_eq!(reloaded.messages, session_before.messages);

        let transcript_after = engine
            .load_transcript(&session_id)
            .expect("reload transcript after retag");
        assert_eq!(
            transcript_after, transcript_before,
            "retag must not mutate transcript entries or ordering"
        );

        assert_eq!(
            normalize_session_output(&retag_output, &session_id),
            readme_output_block("session-retag <id> <label>", "json")
        );

        // session-labels reflects the new label and no longer surfaces the old one.
        let labels_output = render_command(&engine, CliCommand::SessionLabels);
        let labels: Vec<SessionLabelEntry> =
            serde_json::from_str(&labels_output).expect("parse session-labels");
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].label, "release-candidate");
        assert_eq!(labels[0].session_id.to_string(), session_id);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_retag_latest_selector_targets_most_recent_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let _ = render_command(
            &engine,
            CliCommand::SessionRename {
                id: session_id.clone(),
                label: "runtime-review".to_string(),
            },
        );

        let latest_output = render_command(
            &engine,
            CliCommand::SessionRetag {
                id: "latest".to_string(),
                label: "release-candidate".to_string(),
            },
        );

        let retagged: SessionRetag =
            serde_json::from_str(&latest_output).expect("parse session-retag latest output");
        assert_eq!(retagged.retagged_session_id.to_string(), session_id);
        assert_eq!(retagged.previous_label, "runtime-review");
        assert_eq!(retagged.applied_label, "release-candidate");

        assert_eq!(
            normalize_session_output(&latest_output, &session_id),
            readme_output_block("session-retag latest <label>", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_retag_label_selector_resolves_and_updates_session_labels_without_touching_transcript(
    ) {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let _ = render_command(
            &engine,
            CliCommand::SessionRename {
                id: session_id.clone(),
                label: "runtime-review".to_string(),
            },
        );

        let session_before = engine
            .load_session(&session_id)
            .expect("load session before label-selector retag");
        let transcript_before = engine
            .load_transcript(&session_id)
            .expect("load transcript before label-selector retag");

        let label_output = render_command(
            &engine,
            CliCommand::SessionRetag {
                id: "label:runtime-review".to_string(),
                label: "release-candidate".to_string(),
            },
        );

        let retagged: SessionRetag =
            serde_json::from_str(&label_output).expect("parse label-selector retag output");
        assert_eq!(
            retagged.retagged_session_id.to_string(),
            session_id,
            "label selector must resolve to the labeled session id, not the selector string"
        );
        assert_eq!(retagged.previous_label, "runtime-review");
        assert_eq!(retagged.applied_label, "release-candidate");
        assert!(
            !label_output.contains("label:runtime-review"),
            "JSON output must surface the resolved session id, not the selector string: {label_output}"
        );

        // session-labels reflects the new label while transcript/session content
        // and ordering stay unchanged.
        let labels_output = render_command(&engine, CliCommand::SessionLabels);
        let labels: Vec<SessionLabelEntry> =
            serde_json::from_str(&labels_output).expect("parse session-labels after retag");
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].label, "release-candidate");
        assert_eq!(labels[0].session_id.to_string(), session_id);

        let transcript_after = engine
            .load_transcript(&session_id)
            .expect("reload transcript after retag");
        assert_eq!(
            transcript_after, transcript_before,
            "retag must not mutate transcript entries or ordering"
        );

        let reloaded = engine
            .load_session(&session_id)
            .expect("reload session after retag");
        assert_eq!(reloaded.label.as_deref(), Some("release-candidate"));
        assert_eq!(
            reloaded.updated_at_ms, session_before.updated_at_ms,
            "retag must not bump activity metadata"
        );
        assert_eq!(reloaded.messages, session_before.messages);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_retag_rejects_same_effective_label_unknown_session_and_invalid_label_without_touching_store(
    ) {
        use harness_core::SessionId;

        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let _ = render_command(
            &engine,
            CliCommand::SessionRename {
                id: session_id.clone(),
                label: "runtime-review".to_string(),
            },
        );

        let session_before = engine
            .load_session(&session_id)
            .expect("load session before failed retag attempts");
        let transcript_before = engine
            .load_transcript(&session_id)
            .expect("load transcript before failed retag attempts");

        // Same effective label (including surrounding whitespace that normalizes away).
        let same_err = engine
            .retag_session(&session_id, "  runtime-review  ")
            .expect_err("same-effective-label retag must fail");
        assert!(
            same_err.contains("session already labeled"),
            "same-effective-label retag must surface SessionAlreadyLabeled diagnostic: {same_err}"
        );
        assert!(
            same_err.contains(&session_id),
            "same-effective-label error must surface the resolved session id: {same_err}"
        );

        // Empty/whitespace-only labels.
        for invalid in ["", "   ", "\t\n"] {
            let result = engine.retag_session(&session_id, invalid);
            assert!(result.is_err(), "empty/whitespace label must fail to retag");
        }

        // Unknown session id.
        let missing = SessionId::new().to_string();
        let unknown_err = engine
            .retag_session(&missing, "anything")
            .expect_err("unknown session id must fail");
        assert!(
            unknown_err.contains("session not found"),
            "unknown id must surface SessionNotFound diagnostic: {unknown_err}"
        );

        // The seeded labeled session is untouched by every failure above.
        let reloaded = engine
            .load_session(&session_id)
            .expect("reload session after failed retags");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));
        assert_eq!(reloaded.updated_at_ms, session_before.updated_at_ms);
        assert_eq!(reloaded.messages, session_before.messages);
        let transcript_after = engine
            .load_transcript(&session_id)
            .expect("reload transcript after failed retags");
        assert_eq!(transcript_after, transcript_before);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_show_latest_matches_readme_example() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let bootstrap_output = render_command(
            &engine,
            CliCommand::Bootstrap {
                prompt: "review bash".to_string(),
            },
        );
        let bootstrap_json: serde_json::Value =
            serde_json::from_str(&bootstrap_output).expect("parse bootstrap report");
        let session_id = bootstrap_json["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string();

        let latest_output = render_command(
            &engine,
            CliCommand::SessionShow {
                id: "latest".to_string(),
            },
        );

        assert_eq!(
            normalize_session_output(&latest_output, &session_id),
            readme_output_block("session-show latest", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    fn bootstrap_session_id(engine: &RuntimeEngine, prompt: &str) -> String {
        let output = render_command(
            engine,
            CliCommand::Bootstrap {
                prompt: prompt.to_string(),
            },
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&output).expect("parse bootstrap report");
        parsed["session"]["session_id"]
            .as_str()
            .expect("session id in bootstrap output")
            .to_string()
    }

    #[test]
    fn session_prune_matches_readme_example_and_preserves_newest_and_removes_older_artifacts() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let older_id = bootstrap_session_id(&engine, "older prompt");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer_id = bootstrap_session_id(&engine, "newer prompt");

        let output = render_command(&engine, CliCommand::SessionPrune { keep: 1 });

        // Newer session wins newest-first ordering; older one is pruned.
        let prune: SessionPrune =
            serde_json::from_str(&output).expect("parse session-prune output");
        assert_eq!(prune.kept_count, 1);
        assert_eq!(prune.pruned_count, 1);
        assert_eq!(prune.removed.len(), 1);
        assert_eq!(prune.removed[0].session_id.to_string(), older_id);
        assert_eq!(
            prune.removed[0].session_path,
            root.join(format!("{older_id}.json")).display().to_string()
        );
        assert_eq!(
            prune.removed[0].transcript_path,
            root.join(format!("{older_id}.transcript.json"))
                .display()
                .to_string()
        );

        // README regression guard: normalize the generated pruned id and the
        // temp root back to the documented placeholders before comparing.
        let normalized = output
            .replace(&older_id, "<pruned-session-id>")
            .replace(root.to_string_lossy().as_ref(), ".sessions");
        assert_eq!(
            normalized,
            readme_output_block("session-prune --keep <count>", "json")
        );

        // Pruned artifacts are gone from disk and from the subsequent listing,
        // while the preserved session stays newest-first.
        assert!(!root.join(format!("{older_id}.json")).exists());
        assert!(!root.join(format!("{older_id}.transcript.json")).exists());

        let remaining: Vec<String> = engine
            .list_sessions()
            .expect("list after prune")
            .into_iter()
            .map(|entry| entry.session_id.to_string())
            .collect();
        assert_eq!(remaining, vec![newer_id.clone()]);

        // Preserved session still loads with transcript entries intact.
        let transcript = engine
            .load_transcript(&newer_id)
            .expect("load preserved transcript");
        assert_eq!(transcript.entries.len(), 1);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_prune_noop_matches_readme_example_when_store_is_within_retention() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let kept_id = bootstrap_session_id(&engine, "only prompt");

        let output = render_command(&engine, CliCommand::SessionPrune { keep: 10 });
        let prune: SessionPrune =
            serde_json::from_str(&output).expect("parse session-prune no-op output");
        assert_eq!(prune.kept_count, 1);
        assert_eq!(prune.pruned_count, 0);
        assert!(prune.removed.is_empty());

        assert_eq!(output, readme_output_block("session-prune <no-op>", "json"));

        // Nothing was removed: the session is still loadable with its
        // transcript intact.
        let remaining: Vec<String> = engine
            .list_sessions()
            .expect("list after no-op prune")
            .into_iter()
            .map(|entry| entry.session_id.to_string())
            .collect();
        assert_eq!(remaining, vec![kept_id.clone()]);
        assert_eq!(
            engine
                .load_transcript(&kept_id)
                .expect("load transcript after no-op prune")
                .entries
                .len(),
            1
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_prune_keep_zero_removes_every_persisted_session_via_cli_dispatch() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let a = bootstrap_session_id(&engine, "first");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = bootstrap_session_id(&engine, "second");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let c = bootstrap_session_id(&engine, "third");

        let output = render_command(&engine, CliCommand::SessionPrune { keep: 0 });
        let prune: SessionPrune =
            serde_json::from_str(&output).expect("parse session-prune keep=0 output");

        assert_eq!(prune.kept_count, 0);
        assert_eq!(prune.pruned_count, 3);
        let removed_ids: Vec<String> = prune
            .removed
            .iter()
            .map(|entry| entry.session_id.to_string())
            .collect();
        assert_eq!(removed_ids, vec![c.clone(), b.clone(), a.clone()]);

        // Every removed entry reports the deterministic session/transcript
        // path pair for its id.
        for (removal, id) in prune.removed.iter().zip([&c, &b, &a]) {
            assert_eq!(
                removal.session_path,
                root.join(format!("{id}.json")).display().to_string()
            );
            assert_eq!(
                removal.transcript_path,
                root.join(format!("{id}.transcript.json"))
                    .display()
                    .to_string()
            );
        }

        assert!(engine
            .list_sessions()
            .expect("list after full prune")
            .is_empty());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_pin_explicit_id_matches_readme_example_and_persists_pinned_flag() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "pin me");
        let output = render_command(&engine, CliCommand::SessionPin { id: id.clone() });

        // Machine-readable JSON exposes the resolved session id and the pinned state.
        let pin: SessionPin = serde_json::from_str(&output).expect("parse session-pin output");
        assert_eq!(pin.pinned_session_id.to_string(), id);
        assert!(pin.pinned);
        // Deterministic README block (pinned `true`, resolved id placeholder).
        assert_eq!(
            output.replace(&id, "<session-id>"),
            readme_output_block("session-pin <id>", "json")
        );

        // The persisted session now carries `pinned: true` and preserves everything else.
        let reloaded = engine.load_session(&id).expect("reload pinned");
        assert!(reloaded.pinned);
        assert_eq!(reloaded.messages.len(), 1);
        // Transcript is untouched.
        let transcript = engine.load_transcript(&id).expect("reload transcript");
        assert_eq!(transcript.entries.len(), 1);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_pin_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer");

        let output = render_command(
            &engine,
            CliCommand::SessionPin {
                id: "latest".to_string(),
            },
        );
        let pin: SessionPin =
            serde_json::from_str(&output).expect("parse session-pin latest output");
        assert_eq!(pin.pinned_session_id.to_string(), newer);
        assert!(pin.pinned);

        // Newest-first ordering must be preserved — pin does NOT bump updated_at_ms.
        let listed: Vec<String> = engine
            .list_sessions()
            .expect("list after pin")
            .into_iter()
            .map(|entry| entry.session_id.to_string())
            .collect();
        assert_eq!(listed.first().map(String::as_str), Some(newer.as_str()));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_pin_and_unpin_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "pin by label");
        engine
            .rename_session(&id, "to-pin")
            .expect("attach label before pin");

        let pin_output = render_command(
            &engine,
            CliCommand::SessionPin {
                id: "label:to-pin".to_string(),
            },
        );
        let pin: SessionPin =
            serde_json::from_str(&pin_output).expect("parse session-pin label output");
        assert_eq!(pin.pinned_session_id.to_string(), id);
        assert!(pin.pinned);

        let unpin_output = render_command(
            &engine,
            CliCommand::SessionUnpin {
                id: "label:to-pin".to_string(),
            },
        );
        let unpin: SessionUnpin =
            serde_json::from_str(&unpin_output).expect("parse session-unpin label output");
        assert_eq!(unpin.unpinned_session_id.to_string(), id);
        assert!(!unpin.pinned);

        // After unpin the persisted JSON is backward-compatible (no `pinned` key).
        let persisted_path = root.join(format!("{id}.json"));
        let persisted = std::fs::read_to_string(&persisted_path).expect("read persisted json");
        assert!(
            !persisted.contains("\"pinned\""),
            "unpinned persisted JSON must not serialize `pinned`: {persisted}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_unpin_explicit_id_matches_readme_example_and_clears_pinned_flag() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "pin cycle");
        engine.pin_session(&id).expect("pin for unpin fixture");

        let output = render_command(&engine, CliCommand::SessionUnpin { id: id.clone() });
        let unpin: SessionUnpin =
            serde_json::from_str(&output).expect("parse session-unpin output");
        assert_eq!(unpin.unpinned_session_id.to_string(), id);
        assert!(!unpin.pinned);
        assert_eq!(
            output.replace(&id, "<session-id>"),
            readme_output_block("session-unpin <id>", "json")
        );

        // Session still loads, pinned flag is cleared, transcript untouched.
        let reloaded = engine.load_session(&id).expect("reload unpinned");
        assert!(!reloaded.pinned);
        let transcript = engine.load_transcript(&id).expect("reload transcript");
        assert_eq!(transcript.entries.len(), 1);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_prune_with_pinned_session_preserves_pin_and_surfaces_pinned_preserved() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let pinned_id = bootstrap_session_id(&engine, "keep me");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let middle_id = bootstrap_session_id(&engine, "middle");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newest_id = bootstrap_session_id(&engine, "newest");

        engine
            .pin_session(&pinned_id)
            .expect("pin oldest before prune");

        let output = render_command(&engine, CliCommand::SessionPrune { keep: 1 });
        let prune: SessionPrune =
            serde_json::from_str(&output).expect("parse prune-with-pin output");

        // Retention budget applied only to unpinned sessions: newest survives,
        // middle is pruned, pinned oldest is rescued via pinned_preserved.
        assert_eq!(prune.kept_count, 1);
        assert_eq!(prune.pruned_count, 1);
        assert_eq!(prune.removed[0].session_id.to_string(), middle_id);
        assert_eq!(prune.pinned_preserved_count, 1);
        assert_eq!(
            prune.pinned_preserved,
            vec![
                engine
                    .load_session(&pinned_id)
                    .expect("load pinned")
                    .session_id
            ]
        );

        // Survivors: newest + pinned-oldest. Middle is gone from disk and from listing.
        let remaining: Vec<String> = engine
            .list_sessions()
            .expect("list after pin-aware prune")
            .into_iter()
            .map(|entry| entry.session_id.to_string())
            .collect();
        assert_eq!(remaining, vec![newest_id.clone(), pinned_id.clone()]);
        assert!(!root.join(format!("{middle_id}.json")).exists());
        assert!(!root.join(format!("{middle_id}.transcript.json")).exists());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_selector_check_raw_id_matches_readme_example_and_leaves_store_untouched() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "inspect raw id");
        let before = engine.load_session(&id).expect("reload before check");
        let before_transcript = engine.load_transcript(&id).expect("reload transcript");

        let output = render_command(
            &engine,
            CliCommand::SessionSelectorCheck {
                selector: id.clone(),
            },
        );
        let check: SessionSelectorCheck =
            serde_json::from_str(&output).expect("parse session-selector-check output");

        assert_eq!(check.selector, id);
        assert_eq!(check.resolved_session_id.to_string(), id);
        assert_eq!(check.message_count, 1);
        assert_eq!(check.created_at_ms, before.created_at_ms);
        assert_eq!(check.updated_at_ms, before.updated_at_ms);
        assert_eq!(
            check.persisted_path,
            root.join(format!("{id}.json")).display().to_string()
        );
        assert!(check.label.is_none());
        assert!(!check.pinned);

        let normalized = normalize_timestamps(
            &output
                .replace(&id, "<session-id>")
                .replace(root.to_string_lossy().as_ref(), ".sessions"),
        );
        assert_eq!(
            normalized,
            readme_output_block("session-selector-check <selector>", "json")
        );

        // Check does not mutate any persisted state.
        let after = engine.load_session(&id).expect("reload after check");
        assert_eq!(after, before);
        let after_transcript = engine
            .load_transcript(&id)
            .expect("reload transcript after");
        assert_eq!(after_transcript, before_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_selector_check_latest_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer");

        let output = render_command(
            &engine,
            CliCommand::SessionSelectorCheck {
                selector: "latest".to_string(),
            },
        );
        let check: SessionSelectorCheck =
            serde_json::from_str(&output).expect("parse session-selector-check latest output");

        assert_eq!(check.selector, "latest");
        assert_eq!(check.resolved_session_id.to_string(), newer);
        assert!(check.label.is_none());
        assert!(!check.pinned);

        // Ordering unchanged after a selector check.
        let listed: Vec<String> = engine
            .list_sessions()
            .expect("list after selector check")
            .into_iter()
            .map(|entry| entry.session_id.to_string())
            .collect();
        assert_eq!(listed.first().map(String::as_str), Some(newer.as_str()));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_selector_check_label_selector_surfaces_pinned_and_label_metadata() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "pinned + labeled");
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label before check");
        engine.pin_session(&id).expect("pin before check");

        let output = render_command(
            &engine,
            CliCommand::SessionSelectorCheck {
                selector: "label:runtime-review".to_string(),
            },
        );
        let check: SessionSelectorCheck =
            serde_json::from_str(&output).expect("parse session-selector-check label output");

        assert_eq!(check.selector, "label:runtime-review");
        assert_eq!(check.resolved_session_id.to_string(), id);
        assert_eq!(check.label.as_deref(), Some("runtime-review"));
        assert!(check.pinned);

        let normalized = normalize_timestamps(
            &output
                .replace(&id, "<session-id>")
                .replace(root.to_string_lossy().as_ref(), ".sessions"),
        );
        assert_eq!(
            normalized,
            readme_output_block(
                "session-selector-check latest` / `session-selector-check label:<name>",
                "json"
            )
        );

        // The persisted session is untouched and still pinned + labeled.
        let reloaded = engine.load_session(&id).expect("reload after check");
        assert!(reloaded.pinned);
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_selector_check_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        // Anchor a real session so the store directory exists and the
        // unknown-selector failures are surfaced by resolve_selector rather
        // than by a missing store root.
        let anchor_id = bootstrap_session_id(&engine, "anchor");

        let missing_id = "00000000-0000-0000-0000-000000000000";
        let err = engine
            .check_session_selector(missing_id)
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .check_session_selector("label:nonexistent")
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        // Anchor session remains untouched.
        assert!(engine.load_session(&anchor_id).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_selector_check_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .check_session_selector("label:duplicate")
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_selector_check_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .check_session_selector("label:")
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    fn extend_transcript(engine: &RuntimeEngine, id: &str, prompts: &[&str]) {
        for prompt in prompts {
            render_command(
                engine,
                CliCommand::Resume {
                    id: id.to_string(),
                    prompt: (*prompt).to_string(),
                },
            );
        }
    }

    #[test]
    fn transcript_tail_raw_id_truncates_to_count_and_preserves_turn_order() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(
            &engine,
            &id,
            &["second prompt", "third prompt", "fourth prompt"],
        );

        let before_transcript = engine.load_transcript(&id).expect("reload transcript");

        let output = render_command(
            &engine,
            CliCommand::TranscriptTail {
                selector: id.clone(),
                count: 2,
            },
        );
        let tail: SessionTranscriptTail =
            serde_json::from_str(&output).expect("parse transcript-tail output");

        assert_eq!(tail.selector, id);
        assert_eq!(tail.resolved_session_id.to_string(), id);
        assert_eq!(tail.total_entries, 4);
        assert_eq!(tail.returned_entries, 2);
        let returned_prompts: Vec<&str> = tail
            .entries
            .iter()
            .map(|entry| entry.prompt.0.as_str())
            .collect();
        assert_eq!(returned_prompts, vec!["third prompt", "fourth prompt"]);
        let returned_indices: Vec<usize> = tail
            .entries
            .iter()
            .map(|entry| entry.turn_index.0)
            .collect();
        assert_eq!(returned_indices, vec![2, 3]);

        let after_transcript = engine.load_transcript(&id).expect("reload after tail");
        assert_eq!(after_transcript, before_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_tail_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older transcript");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer transcript");
        extend_transcript(&engine, &newer, &["newer follow-up"]);

        let output = render_command(
            &engine,
            CliCommand::TranscriptTail {
                selector: "latest".to_string(),
                count: 1,
            },
        );
        let tail: SessionTranscriptTail =
            serde_json::from_str(&output).expect("parse transcript-tail latest output");

        assert_eq!(tail.selector, "latest");
        assert_eq!(tail.resolved_session_id.to_string(), newer);
        assert_eq!(tail.total_entries, 2);
        assert_eq!(tail.returned_entries, 1);
        assert_eq!(tail.entries.len(), 1);
        assert_eq!(tail.entries[0].turn_index.0, 1);
        assert_eq!(tail.entries[0].prompt.0, "newer follow-up");

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_tail_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "labeled transcript");
        extend_transcript(&engine, &id, &["labeled follow-up"]);
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label for tail");

        let output = render_command(
            &engine,
            CliCommand::TranscriptTail {
                selector: "label:runtime-review".to_string(),
                count: DEFAULT_TRANSCRIPT_TAIL_COUNT,
            },
        );
        let tail: SessionTranscriptTail =
            serde_json::from_str(&output).expect("parse transcript-tail label output");

        assert_eq!(tail.selector, "label:runtime-review");
        assert_eq!(tail.resolved_session_id.to_string(), id);
        assert_eq!(tail.total_entries, 2);
        assert_eq!(tail.returned_entries, 2);
        let prompts: Vec<&str> = tail
            .entries
            .iter()
            .map(|entry| entry.prompt.0.as_str())
            .collect();
        assert_eq!(prompts, vec!["labeled transcript", "labeled follow-up"]);

        let reloaded = engine.load_session(&id).expect("reload after tail");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_tail_count_larger_than_transcript_returns_all_entries() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "only prompt");

        let output = render_command(
            &engine,
            CliCommand::TranscriptTail {
                selector: id.clone(),
                count: 99,
            },
        );
        let tail: SessionTranscriptTail =
            serde_json::from_str(&output).expect("parse transcript-tail output");

        assert_eq!(tail.total_entries, 1);
        assert_eq!(tail.returned_entries, 1);
        assert_eq!(tail.entries.len(), 1);
        assert_eq!(tail.entries[0].turn_index.0, 0);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_tail_count_zero_returns_empty_entries_without_erroring() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt"]);

        let tail = engine
            .tail_session_transcript(&id, 0)
            .expect("count=0 should succeed");

        assert_eq!(tail.total_entries, 2);
        assert_eq!(tail.returned_entries, 0);
        assert!(tail.entries.is_empty());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_tail_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .tail_session_transcript("00000000-0000-0000-0000-000000000000", 5)
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .tail_session_transcript("label:nonexistent", 5)
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        assert!(engine.load_session(&anchor).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_tail_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .tail_session_transcript("label:duplicate", 5)
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_tail_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .tail_session_transcript("label:", 5)
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_find_raw_id_returns_case_insensitive_turn_ordered_matches() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "Review bash runtime");
        extend_transcript(
            &engine,
            &id,
            &["add summary", "review tools", "final polish"],
        );

        let before_transcript = engine.load_transcript(&id).expect("reload transcript");

        let output = render_command(
            &engine,
            CliCommand::TranscriptFind {
                selector: id.clone(),
                query: "REVIEW".to_string(),
            },
        );
        let find: SessionTranscriptFind =
            serde_json::from_str(&output).expect("parse transcript-find output");

        assert_eq!(find.selector, id);
        assert_eq!(find.resolved_session_id.to_string(), id);
        assert_eq!(find.query, "REVIEW");
        assert_eq!(find.total_entries, 4);
        assert_eq!(find.match_count, 2);
        let matched_prompts: Vec<&str> = find.matches.iter().map(|m| m.prompt.0.as_str()).collect();
        assert_eq!(matched_prompts, vec!["Review bash runtime", "review tools"]);
        let matched_indices: Vec<usize> = find.matches.iter().map(|m| m.turn_index.0).collect();
        assert_eq!(matched_indices, vec![0, 2]);

        let after_transcript = engine.load_transcript(&id).expect("reload after find");
        assert_eq!(after_transcript, before_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_find_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older review");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer start");
        extend_transcript(&engine, &newer, &["newer review step"]);

        let output = render_command(
            &engine,
            CliCommand::TranscriptFind {
                selector: "latest".to_string(),
                query: "review".to_string(),
            },
        );
        let find: SessionTranscriptFind =
            serde_json::from_str(&output).expect("parse transcript-find latest output");

        assert_eq!(find.selector, "latest");
        assert_eq!(find.resolved_session_id.to_string(), newer);
        assert_eq!(find.total_entries, 2);
        assert_eq!(find.match_count, 1);
        assert_eq!(find.matches.len(), 1);
        assert_eq!(find.matches[0].turn_index.0, 1);
        assert_eq!(find.matches[0].prompt.0, "newer review step");

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_find_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "labeled review");
        extend_transcript(&engine, &id, &["second review", "unrelated step"]);
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label for find");

        let output = render_command(
            &engine,
            CliCommand::TranscriptFind {
                selector: "label:runtime-review".to_string(),
                query: "review".to_string(),
            },
        );
        let find: SessionTranscriptFind =
            serde_json::from_str(&output).expect("parse transcript-find label output");

        assert_eq!(find.selector, "label:runtime-review");
        assert_eq!(find.resolved_session_id.to_string(), id);
        assert_eq!(find.total_entries, 3);
        assert_eq!(find.match_count, 2);
        let matched_prompts: Vec<&str> = find.matches.iter().map(|m| m.prompt.0.as_str()).collect();
        assert_eq!(matched_prompts, vec!["labeled review", "second review"]);
        let matched_indices: Vec<usize> = find.matches.iter().map(|m| m.turn_index.0).collect();
        assert_eq!(matched_indices, vec![0, 1]);

        let reloaded = engine.load_session(&id).expect("reload after find");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_find_no_match_query_returns_empty_matches_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt"]);

        let find = engine
            .find_in_session_transcript(&id, "definitely-not-present")
            .expect("no-match query should succeed");

        assert_eq!(find.selector, id);
        assert_eq!(find.resolved_session_id.to_string(), id);
        assert_eq!(find.query, "definitely-not-present");
        assert_eq!(find.total_entries, 2);
        assert_eq!(find.match_count, 0);
        assert!(find.matches.is_empty());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_find_empty_query_returns_empty_matches_without_erroring() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt"]);

        let find = engine
            .find_in_session_transcript(&id, "")
            .expect("empty query should succeed");

        assert_eq!(find.query, "");
        assert_eq!(find.total_entries, 2);
        assert_eq!(find.match_count, 0);
        assert!(find.matches.is_empty());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_find_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .find_in_session_transcript("00000000-0000-0000-0000-000000000000", "anything")
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .find_in_session_transcript("label:nonexistent", "anything")
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        assert!(engine.load_session(&anchor).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_find_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .find_in_session_transcript("label:duplicate", "anything")
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_find_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .find_in_session_transcript("label:", "anything")
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_range_raw_id_returns_bounded_forward_slice_in_turn_order() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(
            &engine,
            &id,
            &["second prompt", "third prompt", "fourth prompt"],
        );

        let before_transcript = engine.load_transcript(&id).expect("reload transcript");

        let output = render_command(
            &engine,
            CliCommand::TranscriptRange {
                selector: id.clone(),
                start: 1,
                count: 2,
            },
        );
        let range: SessionTranscriptRange =
            serde_json::from_str(&output).expect("parse transcript-range output");

        assert_eq!(range.selector, id);
        assert_eq!(range.resolved_session_id.to_string(), id);
        assert_eq!(range.start_turn_index, 1);
        assert_eq!(range.requested_count, 2);
        assert_eq!(range.total_entries, 4);
        assert_eq!(range.returned_entries, 2);
        let returned_prompts: Vec<&str> = range
            .entries
            .iter()
            .map(|entry| entry.prompt.0.as_str())
            .collect();
        assert_eq!(returned_prompts, vec!["second prompt", "third prompt"]);
        let returned_indices: Vec<usize> = range
            .entries
            .iter()
            .map(|entry| entry.turn_index.0)
            .collect();
        assert_eq!(returned_indices, vec![1, 2]);

        let after_transcript = engine.load_transcript(&id).expect("reload after range");
        assert_eq!(after_transcript, before_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_range_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older transcript");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer transcript");
        extend_transcript(&engine, &newer, &["newer follow-up", "newer third"]);

        let output = render_command(
            &engine,
            CliCommand::TranscriptRange {
                selector: "latest".to_string(),
                start: 1,
                count: DEFAULT_TRANSCRIPT_RANGE_COUNT,
            },
        );
        let range: SessionTranscriptRange =
            serde_json::from_str(&output).expect("parse transcript-range latest output");

        assert_eq!(range.selector, "latest");
        assert_eq!(range.resolved_session_id.to_string(), newer);
        assert_eq!(range.start_turn_index, 1);
        assert_eq!(range.requested_count, DEFAULT_TRANSCRIPT_RANGE_COUNT);
        assert_eq!(range.total_entries, 3);
        assert_eq!(range.returned_entries, 2);
        let prompts: Vec<&str> = range
            .entries
            .iter()
            .map(|entry| entry.prompt.0.as_str())
            .collect();
        assert_eq!(prompts, vec!["newer follow-up", "newer third"]);
        let indices: Vec<usize> = range
            .entries
            .iter()
            .map(|entry| entry.turn_index.0)
            .collect();
        assert_eq!(indices, vec![1, 2]);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_range_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "labeled transcript");
        extend_transcript(
            &engine,
            &id,
            &["labeled follow-up", "labeled third", "labeled fourth"],
        );
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label for range");

        let output = render_command(
            &engine,
            CliCommand::TranscriptRange {
                selector: "label:runtime-review".to_string(),
                start: 0,
                count: 2,
            },
        );
        let range: SessionTranscriptRange =
            serde_json::from_str(&output).expect("parse transcript-range label output");

        assert_eq!(range.selector, "label:runtime-review");
        assert_eq!(range.resolved_session_id.to_string(), id);
        assert_eq!(range.start_turn_index, 0);
        assert_eq!(range.requested_count, 2);
        assert_eq!(range.total_entries, 4);
        assert_eq!(range.returned_entries, 2);
        let prompts: Vec<&str> = range
            .entries
            .iter()
            .map(|entry| entry.prompt.0.as_str())
            .collect();
        assert_eq!(prompts, vec!["labeled transcript", "labeled follow-up"]);

        let reloaded = engine.load_session(&id).expect("reload after range");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_range_partial_tail_returns_available_remaining_entries() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt", "third prompt"]);

        let range = engine
            .range_session_transcript(&id, 2, 5)
            .expect("partial tail range should succeed");

        assert_eq!(range.start_turn_index, 2);
        assert_eq!(range.requested_count, 5);
        assert_eq!(range.total_entries, 3);
        assert_eq!(range.returned_entries, 1);
        assert_eq!(range.entries.len(), 1);
        assert_eq!(range.entries[0].turn_index.0, 2);
        assert_eq!(range.entries[0].prompt.0, "third prompt");

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_range_out_of_range_start_returns_empty_entries_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt"]);

        let range = engine
            .range_session_transcript(&id, 42, 5)
            .expect("out-of-range start should succeed");

        assert_eq!(range.start_turn_index, 42);
        assert_eq!(range.requested_count, 5);
        assert_eq!(range.total_entries, 2);
        assert_eq!(range.returned_entries, 0);
        assert!(range.entries.is_empty());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_range_empty_transcript_returns_empty_entries_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist empty session");
        let empty_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: Vec::new(),
        };
        engine
            .store
            .save_transcript(&empty_transcript)
            .expect("persist empty transcript");

        let range = engine
            .range_session_transcript(&session_id, 0, DEFAULT_TRANSCRIPT_RANGE_COUNT)
            .expect("empty transcript should succeed");

        assert_eq!(range.start_turn_index, 0);
        assert_eq!(range.requested_count, DEFAULT_TRANSCRIPT_RANGE_COUNT);
        assert_eq!(range.total_entries, 0);
        assert_eq!(range.returned_entries, 0);
        assert!(range.entries.is_empty());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_range_count_zero_returns_empty_entries_without_erroring() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt"]);

        let range = engine
            .range_session_transcript(&id, 0, 0)
            .expect("count=0 should succeed");

        assert_eq!(range.start_turn_index, 0);
        assert_eq!(range.requested_count, 0);
        assert_eq!(range.total_entries, 2);
        assert_eq!(range.returned_entries, 0);
        assert!(range.entries.is_empty());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_range_invalid_count_is_rejected_by_clap_parse() {
        use clap::Parser;

        let err = Cli::try_parse_from([
            "harness",
            "transcript-range",
            "some-id",
            "--start",
            "0",
            "--count",
            "-1",
        ])
        .expect_err("negative --count must fail at parse time");
        let rendered = err.to_string();
        assert!(
            rendered.contains("--count") || rendered.contains("-1"),
            "expected parse error to mention the invalid --count value, got: {rendered}"
        );

        let err = Cli::try_parse_from([
            "harness",
            "transcript-range",
            "some-id",
            "--start",
            "0",
            "--count",
            "not-a-number",
        ])
        .expect_err("non-numeric --count must fail at parse time");
        let rendered = err.to_string();
        assert!(
            rendered.contains("--count") || rendered.contains("not-a-number"),
            "expected parse error to mention the invalid --count value, got: {rendered}"
        );
    }

    #[test]
    fn transcript_range_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .range_session_transcript("00000000-0000-0000-0000-000000000000", 0, 5)
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .range_session_transcript("label:nonexistent", 0, 5)
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        assert!(engine.load_session(&anchor).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_range_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .range_session_transcript("label:duplicate", 0, 5)
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_range_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .range_session_transcript("label:", 0, 5)
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_context_raw_id_returns_centered_window_in_turn_order() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(
            &engine,
            &id,
            &[
                "second prompt",
                "third prompt",
                "fourth prompt",
                "fifth prompt",
            ],
        );

        let before_transcript = engine.load_transcript(&id).expect("reload transcript");

        let output = render_command(
            &engine,
            CliCommand::TranscriptContext {
                selector: id.clone(),
                turn: 2,
                before: 1,
                after: 1,
            },
        );
        let context: SessionTranscriptContext =
            serde_json::from_str(&output).expect("parse transcript-context output");

        assert_eq!(context.selector, id);
        assert_eq!(context.resolved_session_id.to_string(), id);
        assert_eq!(context.center_turn_index, 2);
        assert_eq!(context.requested_before, 1);
        assert_eq!(context.requested_after, 1);
        assert_eq!(context.total_entries, 5);
        assert_eq!(context.returned_entries, 3);
        let returned_prompts: Vec<&str> = context
            .entries
            .iter()
            .map(|entry| entry.prompt.0.as_str())
            .collect();
        assert_eq!(
            returned_prompts,
            vec!["second prompt", "third prompt", "fourth prompt"]
        );
        let returned_indices: Vec<usize> = context
            .entries
            .iter()
            .map(|entry| entry.turn_index.0)
            .collect();
        assert_eq!(returned_indices, vec![1, 2, 3]);

        let after_transcript = engine.load_transcript(&id).expect("reload after context");
        assert_eq!(after_transcript, before_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_context_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older transcript");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer transcript");
        extend_transcript(
            &engine,
            &newer,
            &["newer follow-up", "newer third", "newer fourth"],
        );

        let output = render_command(
            &engine,
            CliCommand::TranscriptContext {
                selector: "latest".to_string(),
                turn: 1,
                before: DEFAULT_TRANSCRIPT_CONTEXT_WINDOW,
                after: DEFAULT_TRANSCRIPT_CONTEXT_WINDOW,
            },
        );
        let context: SessionTranscriptContext =
            serde_json::from_str(&output).expect("parse transcript-context latest output");

        assert_eq!(context.selector, "latest");
        assert_eq!(context.resolved_session_id.to_string(), newer);
        assert_eq!(context.center_turn_index, 1);
        assert_eq!(context.requested_before, DEFAULT_TRANSCRIPT_CONTEXT_WINDOW);
        assert_eq!(context.requested_after, DEFAULT_TRANSCRIPT_CONTEXT_WINDOW);
        assert_eq!(context.total_entries, 4);
        assert_eq!(context.returned_entries, 4);
        let indices: Vec<usize> = context
            .entries
            .iter()
            .map(|entry| entry.turn_index.0)
            .collect();
        assert_eq!(indices, vec![0, 1, 2, 3]);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_context_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "labeled transcript");
        extend_transcript(
            &engine,
            &id,
            &["labeled follow-up", "labeled third", "labeled fourth"],
        );
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label for context");

        let output = render_command(
            &engine,
            CliCommand::TranscriptContext {
                selector: "label:runtime-review".to_string(),
                turn: 2,
                before: 1,
                after: 1,
            },
        );
        let context: SessionTranscriptContext =
            serde_json::from_str(&output).expect("parse transcript-context label output");

        assert_eq!(context.selector, "label:runtime-review");
        assert_eq!(context.resolved_session_id.to_string(), id);
        assert_eq!(context.center_turn_index, 2);
        assert_eq!(context.requested_before, 1);
        assert_eq!(context.requested_after, 1);
        assert_eq!(context.total_entries, 4);
        assert_eq!(context.returned_entries, 3);
        let prompts: Vec<&str> = context
            .entries
            .iter()
            .map(|entry| entry.prompt.0.as_str())
            .collect();
        assert_eq!(
            prompts,
            vec!["labeled follow-up", "labeled third", "labeled fourth"]
        );

        let reloaded = engine.load_session(&id).expect("reload after context");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_context_clips_start_boundary_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt", "third prompt"]);

        let context = engine
            .context_session_transcript(&id, 0, 5, 1)
            .expect("start-boundary context should succeed");

        assert_eq!(context.center_turn_index, 0);
        assert_eq!(context.requested_before, 5);
        assert_eq!(context.requested_after, 1);
        assert_eq!(context.total_entries, 3);
        assert_eq!(context.returned_entries, 2);
        let indices: Vec<usize> = context
            .entries
            .iter()
            .map(|entry| entry.turn_index.0)
            .collect();
        assert_eq!(indices, vec![0, 1]);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_context_clips_end_boundary_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt", "third prompt"]);

        let context = engine
            .context_session_transcript(&id, 2, 1, 5)
            .expect("end-boundary context should succeed");

        assert_eq!(context.center_turn_index, 2);
        assert_eq!(context.requested_before, 1);
        assert_eq!(context.requested_after, 5);
        assert_eq!(context.total_entries, 3);
        assert_eq!(context.returned_entries, 2);
        let indices: Vec<usize> = context
            .entries
            .iter()
            .map(|entry| entry.turn_index.0)
            .collect();
        assert_eq!(indices, vec![1, 2]);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_context_empty_transcript_returns_empty_entries_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist empty session");
        let empty_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: Vec::new(),
        };
        engine
            .store
            .save_transcript(&empty_transcript)
            .expect("persist empty transcript");

        let context = engine
            .context_session_transcript(
                &session_id,
                0,
                DEFAULT_TRANSCRIPT_CONTEXT_WINDOW,
                DEFAULT_TRANSCRIPT_CONTEXT_WINDOW,
            )
            .expect("empty transcript should succeed");

        assert_eq!(context.center_turn_index, 0);
        assert_eq!(context.total_entries, 0);
        assert_eq!(context.returned_entries, 0);
        assert!(context.entries.is_empty());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_context_out_of_range_turn_returns_empty_entries_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt"]);

        let context = engine
            .context_session_transcript(&id, 42, 1, 1)
            .expect("out-of-range turn should succeed");

        assert_eq!(context.center_turn_index, 42);
        assert_eq!(context.total_entries, 2);
        assert_eq!(context.returned_entries, 0);
        assert!(context.entries.is_empty());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_context_invalid_before_is_rejected_by_clap_parse() {
        use clap::Parser;

        let err = Cli::try_parse_from([
            "harness",
            "transcript-context",
            "some-id",
            "--turn",
            "0",
            "--before",
            "-1",
        ])
        .expect_err("negative --before must fail at parse time");
        let rendered = err.to_string();
        assert!(
            rendered.contains("--before") || rendered.contains("-1"),
            "expected parse error to mention the invalid --before value, got: {rendered}"
        );

        let err = Cli::try_parse_from([
            "harness",
            "transcript-context",
            "some-id",
            "--turn",
            "0",
            "--before",
            "not-a-number",
        ])
        .expect_err("non-numeric --before must fail at parse time");
        let rendered = err.to_string();
        assert!(
            rendered.contains("--before") || rendered.contains("not-a-number"),
            "expected parse error to mention the invalid --before value, got: {rendered}"
        );
    }

    #[test]
    fn transcript_context_invalid_after_is_rejected_by_clap_parse() {
        use clap::Parser;

        let err = Cli::try_parse_from([
            "harness",
            "transcript-context",
            "some-id",
            "--turn",
            "0",
            "--after",
            "-1",
        ])
        .expect_err("negative --after must fail at parse time");
        let rendered = err.to_string();
        assert!(
            rendered.contains("--after") || rendered.contains("-1"),
            "expected parse error to mention the invalid --after value, got: {rendered}"
        );

        let err = Cli::try_parse_from([
            "harness",
            "transcript-context",
            "some-id",
            "--turn",
            "0",
            "--after",
            "not-a-number",
        ])
        .expect_err("non-numeric --after must fail at parse time");
        let rendered = err.to_string();
        assert!(
            rendered.contains("--after") || rendered.contains("not-a-number"),
            "expected parse error to mention the invalid --after value, got: {rendered}"
        );
    }

    #[test]
    fn transcript_context_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .context_session_transcript("00000000-0000-0000-0000-000000000000", 0, 1, 1)
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .context_session_transcript("label:nonexistent", 0, 1, 1)
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        assert!(engine.load_session(&anchor).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_context_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .context_session_transcript("label:duplicate", 0, 1, 1)
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_context_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .context_session_transcript("label:", 0, 1, 1)
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_show_raw_id_returns_exact_entry_and_leaves_transcript_untouched() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt", "third prompt"]);

        let before_transcript = engine.load_transcript(&id).expect("reload transcript");

        let output = render_command(
            &engine,
            CliCommand::TranscriptTurnShow {
                selector: id.clone(),
                turn: 1,
            },
        );
        let turn_show: SessionTranscriptTurnShow =
            serde_json::from_str(&output).expect("parse transcript-turn-show output");

        assert_eq!(turn_show.selector, id);
        assert_eq!(turn_show.resolved_session_id.to_string(), id);
        assert_eq!(turn_show.turn_index, 1);
        assert_eq!(turn_show.total_entries, 3);
        assert_eq!(turn_show.entry.turn_index.0, 1);
        assert_eq!(turn_show.entry.prompt.0, "second prompt");

        let after_transcript = engine.load_transcript(&id).expect("reload after turn-show");
        assert_eq!(after_transcript, before_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_show_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older transcript");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer transcript");
        extend_transcript(&engine, &newer, &["newer follow-up", "newer third"]);

        let output = render_command(
            &engine,
            CliCommand::TranscriptTurnShow {
                selector: "latest".to_string(),
                turn: 2,
            },
        );
        let turn_show: SessionTranscriptTurnShow =
            serde_json::from_str(&output).expect("parse transcript-turn-show latest output");

        assert_eq!(turn_show.selector, "latest");
        assert_eq!(turn_show.resolved_session_id.to_string(), newer);
        assert_eq!(turn_show.turn_index, 2);
        assert_eq!(turn_show.total_entries, 3);
        assert_eq!(turn_show.entry.turn_index.0, 2);
        assert_eq!(turn_show.entry.prompt.0, "newer third");

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_show_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "labeled transcript");
        extend_transcript(&engine, &id, &["labeled follow-up", "labeled third"]);
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label for turn-show");

        let output = render_command(
            &engine,
            CliCommand::TranscriptTurnShow {
                selector: "label:runtime-review".to_string(),
                turn: 0,
            },
        );
        let turn_show: SessionTranscriptTurnShow =
            serde_json::from_str(&output).expect("parse transcript-turn-show label output");

        assert_eq!(turn_show.selector, "label:runtime-review");
        assert_eq!(turn_show.resolved_session_id.to_string(), id);
        assert_eq!(turn_show.turn_index, 0);
        assert_eq!(turn_show.total_entries, 3);
        assert_eq!(turn_show.entry.turn_index.0, 0);
        assert_eq!(turn_show.entry.prompt.0, "labeled transcript");

        let reloaded = engine.load_session(&id).expect("reload after turn-show");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_show_empty_transcript_surfaces_turn_out_of_range() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist empty session");
        let empty_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: Vec::new(),
        };
        engine
            .store
            .save_transcript(&empty_transcript)
            .expect("persist empty transcript");

        let err = engine
            .turn_show_session_transcript(&session_id, 0)
            .expect_err("empty transcript should fail");
        assert!(
            err.contains("transcript turn out of range"),
            "expected turn out of range error, got: {err}"
        );
        assert!(
            err.contains("length 0"),
            "expected error to mention transcript length 0, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_show_out_of_range_turn_surfaces_turn_out_of_range() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt"]);

        let err = engine
            .turn_show_session_transcript(&id, 42)
            .expect_err("out-of-range turn should fail");
        assert!(
            err.contains("transcript turn out of range"),
            "expected turn out of range error, got: {err}"
        );
        assert!(
            err.contains("turn 42") && err.contains("length 2"),
            "expected error to mention requested turn and length, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_show_invalid_turn_is_rejected_by_clap_parse() {
        use clap::Parser;

        let err =
            Cli::try_parse_from(["harness", "transcript-turn-show", "some-id", "--turn", "-1"])
                .expect_err("negative --turn must fail at parse time");
        let rendered = err.to_string();
        assert!(
            rendered.contains("--turn") || rendered.contains("-1"),
            "expected parse error to mention the invalid --turn value, got: {rendered}"
        );

        let err = Cli::try_parse_from([
            "harness",
            "transcript-turn-show",
            "some-id",
            "--turn",
            "not-a-number",
        ])
        .expect_err("non-numeric --turn must fail at parse time");
        let rendered = err.to_string();
        assert!(
            rendered.contains("--turn") || rendered.contains("not-a-number"),
            "expected parse error to mention the invalid --turn value, got: {rendered}"
        );
    }

    #[test]
    fn transcript_turn_show_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .turn_show_session_transcript("00000000-0000-0000-0000-000000000000", 0)
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .turn_show_session_transcript("label:nonexistent", 0)
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        assert!(engine.load_session(&anchor).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_show_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .turn_show_session_transcript("label:duplicate", 0)
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_show_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .turn_show_session_transcript("label:", 0)
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_last_turn_raw_id_returns_newest_entry_and_leaves_transcript_untouched() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt", "third prompt"]);

        let before_transcript = engine.load_transcript(&id).expect("reload transcript");

        let output = render_command(
            &engine,
            CliCommand::TranscriptLastTurn {
                selector: id.clone(),
            },
        );
        let last_turn: SessionTranscriptLastTurn =
            serde_json::from_str(&output).expect("parse transcript-last-turn output");

        assert_eq!(last_turn.selector, id);
        assert_eq!(last_turn.resolved_session_id.to_string(), id);
        assert_eq!(last_turn.turn_index, 2);
        assert_eq!(last_turn.total_entries, 3);
        assert_eq!(last_turn.entry.turn_index.0, 2);
        assert_eq!(last_turn.entry.prompt.0, "third prompt");

        let after_transcript = engine.load_transcript(&id).expect("reload after last-turn");
        assert_eq!(after_transcript, before_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_last_turn_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older transcript");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer transcript");
        extend_transcript(&engine, &newer, &["newer follow-up", "newer third"]);

        let output = render_command(
            &engine,
            CliCommand::TranscriptLastTurn {
                selector: "latest".to_string(),
            },
        );
        let last_turn: SessionTranscriptLastTurn =
            serde_json::from_str(&output).expect("parse transcript-last-turn latest output");

        assert_eq!(last_turn.selector, "latest");
        assert_eq!(last_turn.resolved_session_id.to_string(), newer);
        assert_eq!(last_turn.turn_index, 2);
        assert_eq!(last_turn.total_entries, 3);
        assert_eq!(last_turn.entry.turn_index.0, 2);
        assert_eq!(last_turn.entry.prompt.0, "newer third");

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_last_turn_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "labeled transcript");
        extend_transcript(&engine, &id, &["labeled follow-up", "labeled third"]);
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label for last-turn");

        let output = render_command(
            &engine,
            CliCommand::TranscriptLastTurn {
                selector: "label:runtime-review".to_string(),
            },
        );
        let last_turn: SessionTranscriptLastTurn =
            serde_json::from_str(&output).expect("parse transcript-last-turn label output");

        assert_eq!(last_turn.selector, "label:runtime-review");
        assert_eq!(last_turn.resolved_session_id.to_string(), id);
        assert_eq!(last_turn.turn_index, 2);
        assert_eq!(last_turn.total_entries, 3);
        assert_eq!(last_turn.entry.turn_index.0, 2);
        assert_eq!(last_turn.entry.prompt.0, "labeled third");

        let reloaded = engine.load_session(&id).expect("reload after last-turn");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_last_turn_single_entry_transcript_returns_that_entry() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "only prompt");

        let output = render_command(
            &engine,
            CliCommand::TranscriptLastTurn {
                selector: id.clone(),
            },
        );
        let last_turn: SessionTranscriptLastTurn =
            serde_json::from_str(&output).expect("parse transcript-last-turn single output");

        assert_eq!(last_turn.turn_index, 0);
        assert_eq!(last_turn.total_entries, 1);
        assert_eq!(last_turn.entry.turn_index.0, 0);
        assert_eq!(last_turn.entry.prompt.0, "only prompt");

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_last_turn_empty_transcript_surfaces_turn_out_of_range() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist empty session");
        let empty_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: Vec::new(),
        };
        engine
            .store
            .save_transcript(&empty_transcript)
            .expect("persist empty transcript");

        let err = engine
            .last_turn_session_transcript(&session_id)
            .expect_err("empty transcript should fail");
        assert!(
            err.contains("transcript turn out of range"),
            "expected turn out of range error, got: {err}"
        );
        assert!(
            err.contains("length 0"),
            "expected error to mention transcript length 0, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_last_turn_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .last_turn_session_transcript("00000000-0000-0000-0000-000000000000")
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .last_turn_session_transcript("label:nonexistent")
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        assert!(engine.load_session(&anchor).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_last_turn_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .last_turn_session_transcript("label:duplicate")
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_last_turn_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .last_turn_session_transcript("label:")
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_first_turn_raw_id_returns_oldest_entry_and_leaves_transcript_untouched() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt", "third prompt"]);

        let before_transcript = engine.load_transcript(&id).expect("reload transcript");

        let output = render_command(
            &engine,
            CliCommand::TranscriptFirstTurn {
                selector: id.clone(),
            },
        );
        let first_turn: SessionTranscriptFirstTurn =
            serde_json::from_str(&output).expect("parse transcript-first-turn output");

        assert_eq!(first_turn.selector, id);
        assert_eq!(first_turn.resolved_session_id.to_string(), id);
        assert_eq!(first_turn.turn_index, 0);
        assert_eq!(first_turn.total_entries, 3);
        assert_eq!(first_turn.entry.turn_index.0, 0);
        assert_eq!(first_turn.entry.prompt.0, "first prompt");

        let after_transcript = engine
            .load_transcript(&id)
            .expect("reload after first-turn");
        assert_eq!(after_transcript, before_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_first_turn_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older transcript");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer first");
        extend_transcript(&engine, &newer, &["newer follow-up", "newer third"]);

        let output = render_command(
            &engine,
            CliCommand::TranscriptFirstTurn {
                selector: "latest".to_string(),
            },
        );
        let first_turn: SessionTranscriptFirstTurn =
            serde_json::from_str(&output).expect("parse transcript-first-turn latest output");

        assert_eq!(first_turn.selector, "latest");
        assert_eq!(first_turn.resolved_session_id.to_string(), newer);
        assert_eq!(first_turn.turn_index, 0);
        assert_eq!(first_turn.total_entries, 3);
        assert_eq!(first_turn.entry.turn_index.0, 0);
        assert_eq!(first_turn.entry.prompt.0, "newer first");

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_first_turn_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "labeled first");
        extend_transcript(&engine, &id, &["labeled follow-up", "labeled third"]);
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label for first-turn");

        let output = render_command(
            &engine,
            CliCommand::TranscriptFirstTurn {
                selector: "label:runtime-review".to_string(),
            },
        );
        let first_turn: SessionTranscriptFirstTurn =
            serde_json::from_str(&output).expect("parse transcript-first-turn label output");

        assert_eq!(first_turn.selector, "label:runtime-review");
        assert_eq!(first_turn.resolved_session_id.to_string(), id);
        assert_eq!(first_turn.turn_index, 0);
        assert_eq!(first_turn.total_entries, 3);
        assert_eq!(first_turn.entry.turn_index.0, 0);
        assert_eq!(first_turn.entry.prompt.0, "labeled first");

        let reloaded = engine.load_session(&id).expect("reload after first-turn");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_first_turn_single_entry_transcript_returns_that_entry() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "only prompt");

        let output = render_command(
            &engine,
            CliCommand::TranscriptFirstTurn {
                selector: id.clone(),
            },
        );
        let first_turn: SessionTranscriptFirstTurn =
            serde_json::from_str(&output).expect("parse transcript-first-turn single output");

        assert_eq!(first_turn.turn_index, 0);
        assert_eq!(first_turn.total_entries, 1);
        assert_eq!(first_turn.entry.turn_index.0, 0);
        assert_eq!(first_turn.entry.prompt.0, "only prompt");

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_first_turn_empty_transcript_surfaces_turn_out_of_range() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist empty session");
        let empty_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: Vec::new(),
        };
        engine
            .store
            .save_transcript(&empty_transcript)
            .expect("persist empty transcript");

        let err = engine
            .first_turn_session_transcript(&session_id)
            .expect_err("empty transcript should fail");
        assert!(
            err.contains("transcript turn out of range"),
            "expected turn out of range error, got: {err}"
        );
        assert!(
            err.contains("length 0"),
            "expected error to mention transcript length 0, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_first_turn_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .first_turn_session_transcript("00000000-0000-0000-0000-000000000000")
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .first_turn_session_transcript("label:nonexistent")
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        assert!(engine.load_session(&anchor).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_first_turn_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .first_turn_session_transcript("label:duplicate")
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_first_turn_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .first_turn_session_transcript("label:")
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_entry_count_raw_id_returns_length_and_leaves_transcript_untouched() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt", "third prompt"]);

        let before_transcript = engine.load_transcript(&id).expect("reload transcript");
        let before_session = engine.load_session(&id).expect("reload session");

        let output = render_command(
            &engine,
            CliCommand::TranscriptEntryCount {
                selector: id.clone(),
            },
        );
        let count: SessionTranscriptEntryCount =
            serde_json::from_str(&output).expect("parse transcript-entry-count output");

        assert_eq!(count.selector, id);
        assert_eq!(count.resolved_session_id.to_string(), id);
        assert_eq!(count.total_entries, 3);
        assert_eq!(count.created_at_ms, before_transcript.created_at_ms);
        assert_eq!(count.updated_at_ms, before_transcript.updated_at_ms);

        let after_transcript = engine
            .load_transcript(&id)
            .expect("reload after entry-count");
        assert_eq!(after_transcript, before_transcript);
        let after_session = engine.load_session(&id).expect("reload session after");
        assert_eq!(after_session, before_session);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_entry_count_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older transcript");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer first");
        extend_transcript(&engine, &newer, &["newer follow-up", "newer third"]);

        let output = render_command(
            &engine,
            CliCommand::TranscriptEntryCount {
                selector: "latest".to_string(),
            },
        );
        let count: SessionTranscriptEntryCount =
            serde_json::from_str(&output).expect("parse transcript-entry-count latest output");

        assert_eq!(count.selector, "latest");
        assert_eq!(count.resolved_session_id.to_string(), newer);
        assert_eq!(count.total_entries, 3);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_entry_count_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "labeled first");
        extend_transcript(&engine, &id, &["labeled follow-up", "labeled third"]);
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label for entry-count");

        let output = render_command(
            &engine,
            CliCommand::TranscriptEntryCount {
                selector: "label:runtime-review".to_string(),
            },
        );
        let count: SessionTranscriptEntryCount =
            serde_json::from_str(&output).expect("parse transcript-entry-count label output");

        assert_eq!(count.selector, "label:runtime-review");
        assert_eq!(count.resolved_session_id.to_string(), id);
        assert_eq!(count.total_entries, 3);

        let reloaded = engine.load_session(&id).expect("reload after entry-count");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_entry_count_empty_transcript_returns_zero_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist empty session");
        let empty_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: Vec::new(),
        };
        engine
            .store
            .save_transcript(&empty_transcript)
            .expect("persist empty transcript");

        let count = engine
            .entry_count_session_transcript(&session_id)
            .expect("empty transcript should succeed");
        assert_eq!(count.selector, session_id);
        assert_eq!(count.resolved_session_id.to_string(), session_id);
        assert_eq!(count.total_entries, 0);
        assert_eq!(count.created_at_ms, session.created_at_ms);
        assert_eq!(count.updated_at_ms, session.updated_at_ms);

        let after = engine
            .load_transcript(&session_id)
            .expect("reload empty transcript");
        assert_eq!(after, empty_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_entry_count_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .entry_count_session_transcript("00000000-0000-0000-0000-000000000000")
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .entry_count_session_transcript("label:nonexistent")
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        assert!(engine.load_session(&anchor).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_entry_count_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .entry_count_session_transcript("label:duplicate")
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_entry_count_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .entry_count_session_transcript("label:")
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_has_entries_raw_id_reports_true_and_leaves_persisted_state_untouched() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt"]);

        let before_transcript = engine.load_transcript(&id).expect("reload transcript");
        let before_session = engine.load_session(&id).expect("reload session");

        let output = render_command(
            &engine,
            CliCommand::TranscriptHasEntries {
                selector: id.clone(),
            },
        );
        let has_entries: SessionTranscriptHasEntries =
            serde_json::from_str(&output).expect("parse transcript-has-entries output");

        assert_eq!(has_entries.selector, id);
        assert_eq!(
            has_entries.resolved_session_id.to_string(),
            has_entries.selector
        );
        assert_eq!(has_entries.total_entries, 2);
        assert!(has_entries.has_entries);
        assert_eq!(has_entries.created_at_ms, before_transcript.created_at_ms);
        assert_eq!(has_entries.updated_at_ms, before_transcript.updated_at_ms);

        let after_transcript = engine
            .load_transcript(&has_entries.selector)
            .expect("reload after has-entries");
        assert_eq!(after_transcript, before_transcript);
        let after_session = engine
            .load_session(&has_entries.selector)
            .expect("reload session after has-entries");
        assert_eq!(after_session, before_session);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_has_entries_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older transcript");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer first");
        extend_transcript(&engine, &newer, &["newer follow-up"]);

        let output = render_command(
            &engine,
            CliCommand::TranscriptHasEntries {
                selector: "latest".to_string(),
            },
        );
        let has_entries: SessionTranscriptHasEntries =
            serde_json::from_str(&output).expect("parse transcript-has-entries latest output");

        assert_eq!(has_entries.selector, "latest");
        assert_eq!(has_entries.resolved_session_id.to_string(), newer);
        assert_eq!(has_entries.total_entries, 2);
        assert!(has_entries.has_entries);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_has_entries_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "labeled first");
        extend_transcript(&engine, &id, &["labeled follow-up"]);
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label for has-entries");

        let output = render_command(
            &engine,
            CliCommand::TranscriptHasEntries {
                selector: "label:runtime-review".to_string(),
            },
        );
        let has_entries: SessionTranscriptHasEntries =
            serde_json::from_str(&output).expect("parse transcript-has-entries label output");

        assert_eq!(has_entries.selector, "label:runtime-review");
        assert_eq!(has_entries.resolved_session_id.to_string(), id);
        assert_eq!(has_entries.total_entries, 2);
        assert!(has_entries.has_entries);

        let reloaded = engine.load_session(&id).expect("reload after has-entries");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_has_entries_empty_transcript_returns_false_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist empty session");
        let empty_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: Vec::new(),
        };
        engine
            .store
            .save_transcript(&empty_transcript)
            .expect("persist empty transcript");

        let has_entries = engine
            .has_entries_session_transcript(&session_id)
            .expect("empty transcript should succeed");
        assert_eq!(has_entries.selector, session_id);
        assert_eq!(
            has_entries.resolved_session_id.to_string(),
            has_entries.selector
        );
        assert_eq!(has_entries.total_entries, 0);
        assert!(!has_entries.has_entries);
        assert_eq!(has_entries.created_at_ms, session.created_at_ms);
        assert_eq!(has_entries.updated_at_ms, session.updated_at_ms);

        let after = engine
            .load_transcript(&has_entries.selector)
            .expect("reload empty transcript");
        assert_eq!(after, empty_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_has_entries_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .has_entries_session_transcript("00000000-0000-0000-0000-000000000000")
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .has_entries_session_transcript("label:nonexistent")
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        assert!(engine.load_session(&anchor).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_has_entries_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .has_entries_session_transcript("label:duplicate")
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_has_entries_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .has_entries_session_transcript("label:")
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_exists_raw_id_reports_true_and_leaves_persisted_state_untouched() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt", "third prompt"]);

        let before_transcript = engine.load_transcript(&id).expect("reload transcript");
        let before_session = engine.load_session(&id).expect("reload session");

        let output = render_command(
            &engine,
            CliCommand::TranscriptTurnExists {
                selector: id.clone(),
                turn: 1,
            },
        );
        let turn_exists: SessionTranscriptTurnExists =
            serde_json::from_str(&output).expect("parse transcript-turn-exists output");

        assert_eq!(turn_exists.selector, id);
        assert_eq!(
            turn_exists.resolved_session_id.to_string(),
            turn_exists.selector
        );
        assert_eq!(turn_exists.turn_index, 1);
        assert_eq!(turn_exists.total_entries, 3);
        assert!(turn_exists.exists);
        assert_eq!(turn_exists.created_at_ms, before_transcript.created_at_ms);
        assert_eq!(turn_exists.updated_at_ms, before_transcript.updated_at_ms);

        let after_transcript = engine
            .load_transcript(&turn_exists.selector)
            .expect("reload after turn-exists");
        assert_eq!(after_transcript, before_transcript);
        let after_session = engine
            .load_session(&turn_exists.selector)
            .expect("reload session after turn-exists");
        assert_eq!(after_session, before_session);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_exists_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older transcript");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer first");
        extend_transcript(&engine, &newer, &["newer follow-up"]);

        let output = render_command(
            &engine,
            CliCommand::TranscriptTurnExists {
                selector: "latest".to_string(),
                turn: 1,
            },
        );
        let turn_exists: SessionTranscriptTurnExists =
            serde_json::from_str(&output).expect("parse transcript-turn-exists latest output");

        assert_eq!(turn_exists.selector, "latest");
        assert_eq!(turn_exists.resolved_session_id.to_string(), newer);
        assert_eq!(turn_exists.turn_index, 1);
        assert_eq!(turn_exists.total_entries, 2);
        assert!(turn_exists.exists);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_exists_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "labeled first");
        extend_transcript(&engine, &id, &["labeled follow-up"]);
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label for turn-exists");

        let output = render_command(
            &engine,
            CliCommand::TranscriptTurnExists {
                selector: "label:runtime-review".to_string(),
                turn: 1,
            },
        );
        let turn_exists: SessionTranscriptTurnExists =
            serde_json::from_str(&output).expect("parse transcript-turn-exists label output");

        assert_eq!(turn_exists.selector, "label:runtime-review");
        assert_eq!(turn_exists.resolved_session_id.to_string(), id);
        assert_eq!(turn_exists.turn_index, 1);
        assert_eq!(turn_exists.total_entries, 2);
        assert!(turn_exists.exists);

        let reloaded = engine.load_session(&id).expect("reload after turn-exists");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_exists_missing_turn_returns_false_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt"]);

        let turn_exists = engine
            .turn_exists_session_transcript(&id, 42)
            .expect("out-of-range turn should succeed");
        assert_eq!(turn_exists.selector, id);
        assert_eq!(
            turn_exists.resolved_session_id.to_string(),
            turn_exists.selector
        );
        assert_eq!(turn_exists.turn_index, 42);
        assert_eq!(turn_exists.total_entries, 2);
        assert!(!turn_exists.exists);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_exists_empty_transcript_returns_false_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist empty session");
        let empty_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: Vec::new(),
        };
        engine
            .store
            .save_transcript(&empty_transcript)
            .expect("persist empty transcript");

        let turn_exists = engine
            .turn_exists_session_transcript(&session_id, 0)
            .expect("empty transcript should succeed");
        assert_eq!(turn_exists.selector, session_id);
        assert_eq!(
            turn_exists.resolved_session_id.to_string(),
            turn_exists.selector
        );
        assert_eq!(turn_exists.turn_index, 0);
        assert_eq!(turn_exists.total_entries, 0);
        assert!(!turn_exists.exists);
        assert_eq!(turn_exists.created_at_ms, empty_transcript.created_at_ms);
        assert_eq!(turn_exists.updated_at_ms, empty_transcript.updated_at_ms);

        let after = engine
            .load_transcript(&turn_exists.selector)
            .expect("reload empty transcript");
        assert_eq!(after, empty_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_exists_invalid_turn_is_rejected_by_clap_parse() {
        use clap::Parser;

        let err = Cli::try_parse_from([
            "harness",
            "transcript-turn-exists",
            "some-id",
            "--turn",
            "-1",
        ])
        .expect_err("negative --turn must fail at parse time");
        let rendered = err.to_string();
        assert!(
            rendered.contains("--turn") || rendered.contains("-1"),
            "expected parse error to mention the invalid --turn value, got: {rendered}"
        );

        let err = Cli::try_parse_from([
            "harness",
            "transcript-turn-exists",
            "some-id",
            "--turn",
            "not-a-number",
        ])
        .expect_err("non-numeric --turn must fail at parse time");
        let rendered = err.to_string();
        assert!(
            rendered.contains("--turn") || rendered.contains("not-a-number"),
            "expected parse error to mention the invalid --turn value, got: {rendered}"
        );
    }

    #[test]
    fn transcript_turn_exists_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .turn_exists_session_transcript("00000000-0000-0000-0000-000000000000", 0)
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .turn_exists_session_transcript("label:nonexistent", 0)
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        assert!(engine.load_session(&anchor).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_exists_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .turn_exists_session_transcript("label:duplicate", 0)
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_exists_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .turn_exists_session_transcript("label:", 0)
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_indexes_raw_id_returns_ascending_indexes_and_leaves_persisted_state_untouched(
    ) {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt", "third prompt"]);

        let before_transcript = engine.load_transcript(&id).expect("reload transcript");
        let before_session = engine.load_session(&id).expect("reload session");

        let output = render_command(
            &engine,
            CliCommand::TranscriptTurnIndexes {
                selector: id.clone(),
            },
        );
        let turn_indexes: SessionTranscriptTurnIndexes =
            serde_json::from_str(&output).expect("parse transcript-turn-indexes output");

        assert_eq!(turn_indexes.selector, id);
        assert_eq!(
            turn_indexes.resolved_session_id.to_string(),
            turn_indexes.selector
        );
        assert_eq!(turn_indexes.total_entries, 3);
        assert_eq!(turn_indexes.turn_indexes, vec![0, 1, 2]);
        assert_eq!(turn_indexes.created_at_ms, before_transcript.created_at_ms);
        assert_eq!(turn_indexes.updated_at_ms, before_transcript.updated_at_ms);

        let after_transcript = engine
            .load_transcript(&turn_indexes.selector)
            .expect("reload after turn-indexes");
        assert_eq!(after_transcript, before_transcript);
        let after_session = engine
            .load_session(&turn_indexes.selector)
            .expect("reload session after turn-indexes");
        assert_eq!(after_session, before_session);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_indexes_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older transcript");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer first");
        extend_transcript(&engine, &newer, &["newer follow-up"]);

        let output = render_command(
            &engine,
            CliCommand::TranscriptTurnIndexes {
                selector: "latest".to_string(),
            },
        );
        let turn_indexes: SessionTranscriptTurnIndexes =
            serde_json::from_str(&output).expect("parse transcript-turn-indexes latest output");

        assert_eq!(turn_indexes.selector, "latest");
        assert_eq!(turn_indexes.resolved_session_id.to_string(), newer);
        assert_eq!(turn_indexes.total_entries, 2);
        assert_eq!(turn_indexes.turn_indexes, vec![0, 1]);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_indexes_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "labeled first");
        extend_transcript(&engine, &id, &["labeled follow-up"]);
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label for turn-indexes");

        let output = render_command(
            &engine,
            CliCommand::TranscriptTurnIndexes {
                selector: "label:runtime-review".to_string(),
            },
        );
        let turn_indexes: SessionTranscriptTurnIndexes =
            serde_json::from_str(&output).expect("parse transcript-turn-indexes label output");

        assert_eq!(turn_indexes.selector, "label:runtime-review");
        assert_eq!(turn_indexes.resolved_session_id.to_string(), id);
        assert_eq!(turn_indexes.total_entries, 2);
        assert_eq!(turn_indexes.turn_indexes, vec![0, 1]);

        let reloaded = engine.load_session(&id).expect("reload after turn-indexes");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_indexes_empty_transcript_returns_empty_array_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist empty session");
        let empty_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: Vec::new(),
        };
        engine
            .store
            .save_transcript(&empty_transcript)
            .expect("persist empty transcript");

        let turn_indexes = engine
            .turn_indexes_session_transcript(&session_id)
            .expect("empty transcript should succeed");
        assert_eq!(turn_indexes.selector, session_id);
        assert_eq!(
            turn_indexes.resolved_session_id.to_string(),
            turn_indexes.selector
        );
        assert_eq!(turn_indexes.total_entries, 0);
        assert!(turn_indexes.turn_indexes.is_empty());
        assert_eq!(turn_indexes.created_at_ms, empty_transcript.created_at_ms);
        assert_eq!(turn_indexes.updated_at_ms, empty_transcript.updated_at_ms);

        let after = engine
            .load_transcript(&turn_indexes.selector)
            .expect("reload empty transcript");
        assert_eq!(after, empty_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_indexes_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .turn_indexes_session_transcript("00000000-0000-0000-0000-000000000000")
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .turn_indexes_session_transcript("label:nonexistent")
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        assert!(engine.load_session(&anchor).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_indexes_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .turn_indexes_session_transcript("label:duplicate")
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_indexes_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .turn_indexes_session_transcript("label:")
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_index_range_raw_id_returns_min_and_max_and_leaves_persisted_state_untouched()
    {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt", "third prompt"]);

        let before_transcript = engine.load_transcript(&id).expect("reload transcript");
        let before_session = engine.load_session(&id).expect("reload session");

        let output = render_command(
            &engine,
            CliCommand::TranscriptTurnIndexRange {
                selector: id.clone(),
            },
        );
        let turn_range: SessionTranscriptTurnRange =
            serde_json::from_str(&output).expect("parse transcript-turn-index-range output");

        assert_eq!(turn_range.selector, id);
        assert_eq!(
            turn_range.resolved_session_id.to_string(),
            turn_range.selector
        );
        assert_eq!(turn_range.total_entries, 3);
        assert_eq!(turn_range.first_turn_index, Some(0));
        assert_eq!(turn_range.last_turn_index, Some(2));
        assert_eq!(turn_range.created_at_ms, before_transcript.created_at_ms);
        assert_eq!(turn_range.updated_at_ms, before_transcript.updated_at_ms);

        let after_transcript = engine
            .load_transcript(&turn_range.selector)
            .expect("reload after turn-index-range");
        assert_eq!(after_transcript, before_transcript);
        let after_session = engine
            .load_session(&turn_range.selector)
            .expect("reload session after turn-index-range");
        assert_eq!(after_session, before_session);

        let normalized =
            normalize_timestamps(&output.replace(&turn_range.selector, "<session-id>"));
        assert_eq!(
            normalized,
            readme_output_block("transcript-turn-index-range <selector>", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_index_range_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older transcript");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer first");
        extend_transcript(&engine, &newer, &["newer follow-up"]);

        let output = render_command(
            &engine,
            CliCommand::TranscriptTurnIndexRange {
                selector: "latest".to_string(),
            },
        );
        let turn_range: SessionTranscriptTurnRange =
            serde_json::from_str(&output).expect("parse transcript-turn-index-range latest output");

        assert_eq!(turn_range.selector, "latest");
        assert_eq!(turn_range.resolved_session_id.to_string(), newer);
        assert_eq!(turn_range.total_entries, 2);
        assert_eq!(turn_range.first_turn_index, Some(0));
        assert_eq!(turn_range.last_turn_index, Some(1));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_index_range_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "labeled first");
        extend_transcript(&engine, &id, &["labeled follow-up"]);
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label for turn-index-range");

        let output = render_command(
            &engine,
            CliCommand::TranscriptTurnIndexRange {
                selector: "label:runtime-review".to_string(),
            },
        );
        let turn_range: SessionTranscriptTurnRange =
            serde_json::from_str(&output).expect("parse transcript-turn-index-range label output");

        assert_eq!(turn_range.selector, "label:runtime-review");
        assert_eq!(turn_range.resolved_session_id.to_string(), id);
        assert_eq!(turn_range.total_entries, 2);
        assert_eq!(turn_range.first_turn_index, Some(0));
        assert_eq!(turn_range.last_turn_index, Some(1));

        let reloaded = engine
            .load_session(&id)
            .expect("reload after turn-index-range");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_index_range_empty_transcript_returns_null_bounds_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist empty session");
        let empty_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: Vec::new(),
        };
        engine
            .store
            .save_transcript(&empty_transcript)
            .expect("persist empty transcript");

        let turn_range = engine
            .turn_range_session_transcript(&session_id)
            .expect("empty transcript should succeed");
        assert_eq!(turn_range.selector, session_id);
        assert_eq!(
            turn_range.resolved_session_id.to_string(),
            turn_range.selector
        );
        assert_eq!(turn_range.total_entries, 0);
        assert_eq!(turn_range.first_turn_index, None);
        assert_eq!(turn_range.last_turn_index, None);
        assert_eq!(turn_range.created_at_ms, empty_transcript.created_at_ms);
        assert_eq!(turn_range.updated_at_ms, empty_transcript.updated_at_ms);

        let after = engine
            .load_transcript(&turn_range.selector)
            .expect("reload empty transcript");
        assert_eq!(after, empty_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_index_range_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .turn_range_session_transcript("00000000-0000-0000-0000-000000000000")
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .turn_range_session_transcript("label:nonexistent")
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        assert!(engine.load_session(&anchor).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_index_range_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .turn_range_session_transcript("label:duplicate")
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_index_range_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .turn_range_session_transcript("label:")
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_has_turn_gaps_raw_id_contiguous_transcript_reports_false_and_leaves_state_untouched(
    ) {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt", "third prompt"]);

        let before_transcript = engine.load_transcript(&id).expect("reload transcript");
        let before_session = engine.load_session(&id).expect("reload session");

        let output = render_command(
            &engine,
            CliCommand::TranscriptHasTurnGaps {
                selector: id.clone(),
            },
        );
        let has_turn_gaps: SessionTranscriptHasTurnGaps =
            serde_json::from_str(&output).expect("parse transcript-has-turn-gaps output");

        assert_eq!(has_turn_gaps.selector, id);
        assert_eq!(
            has_turn_gaps.resolved_session_id.to_string(),
            has_turn_gaps.selector
        );
        assert_eq!(has_turn_gaps.total_entries, 3);
        assert!(!has_turn_gaps.has_turn_gaps);
        assert_eq!(has_turn_gaps.created_at_ms, before_transcript.created_at_ms);
        assert_eq!(has_turn_gaps.updated_at_ms, before_transcript.updated_at_ms);

        let after_transcript = engine
            .load_transcript(&has_turn_gaps.selector)
            .expect("reload after has-turn-gaps");
        assert_eq!(after_transcript, before_transcript);
        let after_session = engine
            .load_session(&has_turn_gaps.selector)
            .expect("reload session after has-turn-gaps");
        assert_eq!(after_session, before_session);

        let normalized =
            normalize_timestamps(&output.replace(&has_turn_gaps.selector, "<session-id>"));
        assert_eq!(
            normalized,
            readme_output_block("transcript-has-turn-gaps <selector>", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_has_turn_gaps_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older transcript");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer first");
        extend_transcript(&engine, &newer, &["newer follow-up"]);

        let output = render_command(
            &engine,
            CliCommand::TranscriptHasTurnGaps {
                selector: "latest".to_string(),
            },
        );
        let has_turn_gaps: SessionTranscriptHasTurnGaps =
            serde_json::from_str(&output).expect("parse transcript-has-turn-gaps latest output");

        assert_eq!(has_turn_gaps.selector, "latest");
        assert_eq!(has_turn_gaps.resolved_session_id.to_string(), newer);
        assert_eq!(has_turn_gaps.total_entries, 2);
        assert!(!has_turn_gaps.has_turn_gaps);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_has_turn_gaps_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "labeled first");
        extend_transcript(&engine, &id, &["labeled follow-up"]);
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label for has-turn-gaps");

        let output = render_command(
            &engine,
            CliCommand::TranscriptHasTurnGaps {
                selector: "label:runtime-review".to_string(),
            },
        );
        let has_turn_gaps: SessionTranscriptHasTurnGaps =
            serde_json::from_str(&output).expect("parse transcript-has-turn-gaps label output");

        assert_eq!(has_turn_gaps.selector, "label:runtime-review");
        assert_eq!(has_turn_gaps.resolved_session_id.to_string(), id);
        assert_eq!(has_turn_gaps.total_entries, 2);
        assert!(!has_turn_gaps.has_turn_gaps);

        let reloaded = engine
            .load_session(&id)
            .expect("reload after has-turn-gaps");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_has_turn_gaps_transcript_with_internal_gap_reports_true() {
        use harness_core::{Prompt, TurnIndex};

        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist seed session");

        let gap_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: vec![
                TranscriptEntry {
                    turn_index: TurnIndex(0),
                    prompt: Prompt::new("turn zero"),
                },
                TranscriptEntry {
                    turn_index: TurnIndex(2),
                    prompt: Prompt::new("turn two — gap at 1"),
                },
            ],
        };
        engine
            .store
            .save_transcript(&gap_transcript)
            .expect("persist gap transcript");

        let has_turn_gaps = engine
            .has_turn_gaps_session_transcript(&session_id)
            .expect("gap transcript should succeed");

        assert_eq!(has_turn_gaps.selector, session_id);
        assert_eq!(
            has_turn_gaps.resolved_session_id.to_string(),
            session_id
        );
        assert_eq!(has_turn_gaps.total_entries, 2);
        assert!(has_turn_gaps.has_turn_gaps);
        assert_eq!(has_turn_gaps.created_at_ms, gap_transcript.created_at_ms);
        assert_eq!(has_turn_gaps.updated_at_ms, gap_transcript.updated_at_ms);

        let after = engine
            .load_transcript(&session_id)
            .expect("reload gap transcript");
        assert_eq!(after, gap_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_has_turn_gaps_empty_transcript_returns_false_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist empty session");
        let empty_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: Vec::new(),
        };
        engine
            .store
            .save_transcript(&empty_transcript)
            .expect("persist empty transcript");

        let has_turn_gaps = engine
            .has_turn_gaps_session_transcript(&session_id)
            .expect("empty transcript should succeed");
        assert_eq!(has_turn_gaps.selector, session_id);
        assert_eq!(
            has_turn_gaps.resolved_session_id.to_string(),
            has_turn_gaps.selector
        );
        assert_eq!(has_turn_gaps.total_entries, 0);
        assert!(!has_turn_gaps.has_turn_gaps);
        assert_eq!(has_turn_gaps.created_at_ms, empty_transcript.created_at_ms);
        assert_eq!(has_turn_gaps.updated_at_ms, empty_transcript.updated_at_ms);

        let after = engine
            .load_transcript(&has_turn_gaps.selector)
            .expect("reload empty transcript");
        assert_eq!(after, empty_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_has_turn_gaps_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .has_turn_gaps_session_transcript("00000000-0000-0000-0000-000000000000")
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .has_turn_gaps_session_transcript("label:nonexistent")
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        assert!(engine.load_session(&anchor).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_has_turn_gaps_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .has_turn_gaps_session_transcript("label:duplicate")
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_has_turn_gaps_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .has_turn_gaps_session_transcript("label:")
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_missing_turn_indexes_raw_id_contiguous_transcript_reports_empty_and_leaves_state_untouched(
    ) {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt", "third prompt"]);

        let before_transcript = engine.load_transcript(&id).expect("reload transcript");
        let before_session = engine.load_session(&id).expect("reload session");

        let output = render_command(
            &engine,
            CliCommand::TranscriptMissingTurnIndexes {
                selector: id.clone(),
            },
        );
        let missing: SessionTranscriptMissingTurnIndexes =
            serde_json::from_str(&output).expect("parse transcript-missing-turn-indexes output");

        assert_eq!(missing.selector, id);
        assert_eq!(missing.resolved_session_id.to_string(), missing.selector);
        assert_eq!(missing.total_entries, 3);
        assert!(missing.missing_turn_indexes.is_empty());
        assert_eq!(missing.created_at_ms, before_transcript.created_at_ms);
        assert_eq!(missing.updated_at_ms, before_transcript.updated_at_ms);

        let after_transcript = engine
            .load_transcript(&missing.selector)
            .expect("reload after missing-turn-indexes");
        assert_eq!(after_transcript, before_transcript);
        let after_session = engine
            .load_session(&missing.selector)
            .expect("reload session after missing-turn-indexes");
        assert_eq!(after_session, before_session);

        let normalized = normalize_timestamps(&output.replace(&missing.selector, "<session-id>"));
        assert_eq!(
            normalized,
            readme_output_block("transcript-missing-turn-indexes <selector>", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_missing_turn_indexes_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older transcript");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer first");
        extend_transcript(&engine, &newer, &["newer follow-up"]);

        let output = render_command(
            &engine,
            CliCommand::TranscriptMissingTurnIndexes {
                selector: "latest".to_string(),
            },
        );
        let missing: SessionTranscriptMissingTurnIndexes = serde_json::from_str(&output)
            .expect("parse transcript-missing-turn-indexes latest output");

        assert_eq!(missing.selector, "latest");
        assert_eq!(missing.resolved_session_id.to_string(), newer);
        assert_eq!(missing.total_entries, 2);
        assert!(missing.missing_turn_indexes.is_empty());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_missing_turn_indexes_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "labeled first");
        extend_transcript(&engine, &id, &["labeled follow-up"]);
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label for missing-turn-indexes");

        let output = render_command(
            &engine,
            CliCommand::TranscriptMissingTurnIndexes {
                selector: "label:runtime-review".to_string(),
            },
        );
        let missing: SessionTranscriptMissingTurnIndexes = serde_json::from_str(&output)
            .expect("parse transcript-missing-turn-indexes label output");

        assert_eq!(missing.selector, "label:runtime-review");
        assert_eq!(missing.resolved_session_id.to_string(), id);
        assert_eq!(missing.total_entries, 2);
        assert!(missing.missing_turn_indexes.is_empty());

        let reloaded = engine
            .load_session(&id)
            .expect("reload after missing-turn-indexes");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_missing_turn_indexes_transcript_with_single_internal_gap_reports_single_index() {
        use harness_core::{Prompt, TurnIndex};

        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist seed session");

        let gap_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: vec![
                TranscriptEntry {
                    turn_index: TurnIndex(0),
                    prompt: Prompt::new("turn zero"),
                },
                TranscriptEntry {
                    turn_index: TurnIndex(2),
                    prompt: Prompt::new("turn two - gap at 1"),
                },
            ],
        };
        engine
            .store
            .save_transcript(&gap_transcript)
            .expect("persist gap transcript");

        let missing = engine
            .missing_turn_indexes_session_transcript(&session_id)
            .expect("single-gap transcript should succeed");

        assert_eq!(missing.selector, session_id);
        assert_eq!(missing.resolved_session_id.to_string(), session_id);
        assert_eq!(missing.total_entries, 2);
        assert_eq!(missing.missing_turn_indexes, vec![1]);
        assert_eq!(missing.created_at_ms, gap_transcript.created_at_ms);
        assert_eq!(missing.updated_at_ms, gap_transcript.updated_at_ms);

        let after = engine
            .load_transcript(&session_id)
            .expect("reload gap transcript");
        assert_eq!(after, gap_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_missing_turn_indexes_transcript_with_multiple_internal_gaps_reports_ascending_list(
    ) {
        use harness_core::{Prompt, TurnIndex};

        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist seed session");

        let gap_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: vec![
                TranscriptEntry {
                    turn_index: TurnIndex(1),
                    prompt: Prompt::new("turn one"),
                },
                TranscriptEntry {
                    turn_index: TurnIndex(4),
                    prompt: Prompt::new("turn four - gaps at 2, 3"),
                },
                TranscriptEntry {
                    turn_index: TurnIndex(7),
                    prompt: Prompt::new("turn seven - gaps at 5, 6"),
                },
            ],
        };
        engine
            .store
            .save_transcript(&gap_transcript)
            .expect("persist multi-gap transcript");

        let missing = engine
            .missing_turn_indexes_session_transcript(&session_id)
            .expect("multi-gap transcript should succeed");

        assert_eq!(missing.total_entries, 3);
        assert_eq!(missing.missing_turn_indexes, vec![2, 3, 5, 6]);
        assert_eq!(missing.created_at_ms, gap_transcript.created_at_ms);
        assert_eq!(missing.updated_at_ms, gap_transcript.updated_at_ms);

        let after = engine
            .load_transcript(&session_id)
            .expect("reload multi-gap transcript");
        assert_eq!(after, gap_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_missing_turn_indexes_single_entry_transcript_returns_empty() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "only prompt");

        let missing = engine
            .missing_turn_indexes_session_transcript(&id)
            .expect("single-entry transcript should succeed");
        assert_eq!(missing.total_entries, 1);
        assert!(missing.missing_turn_indexes.is_empty());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_missing_turn_indexes_empty_transcript_returns_empty_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist empty session");
        let empty_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: Vec::new(),
        };
        engine
            .store
            .save_transcript(&empty_transcript)
            .expect("persist empty transcript");

        let missing = engine
            .missing_turn_indexes_session_transcript(&session_id)
            .expect("empty transcript should succeed");
        assert_eq!(missing.selector, session_id);
        assert_eq!(missing.resolved_session_id.to_string(), missing.selector);
        assert_eq!(missing.total_entries, 0);
        assert!(missing.missing_turn_indexes.is_empty());
        assert_eq!(missing.created_at_ms, empty_transcript.created_at_ms);
        assert_eq!(missing.updated_at_ms, empty_transcript.updated_at_ms);

        let after = engine
            .load_transcript(&missing.selector)
            .expect("reload empty transcript");
        assert_eq!(after, empty_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_missing_turn_indexes_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .missing_turn_indexes_session_transcript("00000000-0000-0000-0000-000000000000")
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .missing_turn_indexes_session_transcript("label:nonexistent")
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        assert!(engine.load_session(&anchor).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_missing_turn_indexes_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .missing_turn_indexes_session_transcript("label:duplicate")
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_missing_turn_indexes_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .missing_turn_indexes_session_transcript("label:")
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_density_raw_id_contiguous_transcript_reports_full_density_and_leaves_state_untouched(
    ) {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt", "third prompt"]);

        let before_transcript = engine.load_transcript(&id).expect("reload transcript");
        let before_session = engine.load_session(&id).expect("reload session");

        let output = render_command(
            &engine,
            CliCommand::TranscriptTurnDensity {
                selector: id.clone(),
            },
        );
        let density: SessionTranscriptTurnDensity =
            serde_json::from_str(&output).expect("parse transcript-turn-density output");

        assert_eq!(density.selector, id);
        assert_eq!(density.resolved_session_id.to_string(), density.selector);
        assert_eq!(density.total_entries, 3);
        assert_eq!(density.span_entry_count, 3);
        assert_eq!(density.missing_turn_count, 0);
        assert_eq!(density.turn_density, 1.0);
        assert_eq!(density.created_at_ms, before_transcript.created_at_ms);
        assert_eq!(density.updated_at_ms, before_transcript.updated_at_ms);

        let after_transcript = engine
            .load_transcript(&density.selector)
            .expect("reload after turn-density");
        assert_eq!(after_transcript, before_transcript);
        let after_session = engine
            .load_session(&density.selector)
            .expect("reload session after turn-density");
        assert_eq!(after_session, before_session);

        let normalized = normalize_timestamps(&output.replace(&density.selector, "<session-id>"));
        assert_eq!(
            normalized,
            readme_output_block("transcript-turn-density <selector>", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_density_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older transcript");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer first");
        extend_transcript(&engine, &newer, &["newer follow-up"]);

        let output = render_command(
            &engine,
            CliCommand::TranscriptTurnDensity {
                selector: "latest".to_string(),
            },
        );
        let density: SessionTranscriptTurnDensity =
            serde_json::from_str(&output).expect("parse transcript-turn-density latest output");

        assert_eq!(density.selector, "latest");
        assert_eq!(density.resolved_session_id.to_string(), newer);
        assert_eq!(density.total_entries, 2);
        assert_eq!(density.span_entry_count, 2);
        assert_eq!(density.missing_turn_count, 0);
        assert_eq!(density.turn_density, 1.0);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_density_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "labeled first");
        extend_transcript(&engine, &id, &["labeled follow-up"]);
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label for turn-density");

        let output = render_command(
            &engine,
            CliCommand::TranscriptTurnDensity {
                selector: "label:runtime-review".to_string(),
            },
        );
        let density: SessionTranscriptTurnDensity =
            serde_json::from_str(&output).expect("parse transcript-turn-density label output");

        assert_eq!(density.selector, "label:runtime-review");
        assert_eq!(density.resolved_session_id.to_string(), id);
        assert_eq!(density.total_entries, 2);
        assert_eq!(density.span_entry_count, 2);
        assert_eq!(density.missing_turn_count, 0);
        assert_eq!(density.turn_density, 1.0);

        let reloaded = engine
            .load_session(&id)
            .expect("reload after turn-density");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_density_single_entry_transcript_reports_full_density() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "only prompt");

        let density = engine
            .turn_density_session_transcript(&id)
            .expect("single-entry transcript should succeed");
        assert_eq!(density.total_entries, 1);
        assert_eq!(density.span_entry_count, 1);
        assert_eq!(density.missing_turn_count, 0);
        assert_eq!(density.turn_density, 1.0);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_density_empty_transcript_reports_defaults_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist empty session");
        let empty_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: Vec::new(),
        };
        engine
            .store
            .save_transcript(&empty_transcript)
            .expect("persist empty transcript");

        let density = engine
            .turn_density_session_transcript(&session_id)
            .expect("empty transcript should succeed");
        assert_eq!(density.selector, session_id);
        assert_eq!(density.resolved_session_id.to_string(), density.selector);
        assert_eq!(density.total_entries, 0);
        assert_eq!(density.span_entry_count, 0);
        assert_eq!(density.missing_turn_count, 0);
        assert_eq!(density.turn_density, 1.0);
        assert_eq!(density.created_at_ms, empty_transcript.created_at_ms);
        assert_eq!(density.updated_at_ms, empty_transcript.updated_at_ms);

        let after = engine
            .load_transcript(&density.selector)
            .expect("reload empty transcript");
        assert_eq!(after, empty_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_density_gapped_transcript_reports_density_below_one() {
        use harness_core::{Prompt, TurnIndex};

        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist seed session");

        let gap_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: vec![
                TranscriptEntry {
                    turn_index: TurnIndex(1),
                    prompt: Prompt::new("turn one"),
                },
                TranscriptEntry {
                    turn_index: TurnIndex(4),
                    prompt: Prompt::new("turn four - gaps at 2, 3"),
                },
                TranscriptEntry {
                    turn_index: TurnIndex(7),
                    prompt: Prompt::new("turn seven - gaps at 5, 6"),
                },
            ],
        };
        engine
            .store
            .save_transcript(&gap_transcript)
            .expect("persist gap transcript");

        let density = engine
            .turn_density_session_transcript(&session_id)
            .expect("gapped transcript should succeed");

        assert_eq!(density.total_entries, 3);
        assert_eq!(density.span_entry_count, 7);
        assert_eq!(density.missing_turn_count, 4);
        assert!(
            density.turn_density < 1.0,
            "expected density < 1.0 for gapped transcript, got: {}",
            density.turn_density
        );
        assert_eq!(density.turn_density, 3.0_f64 / 7.0_f64);
        assert_eq!(density.created_at_ms, gap_transcript.created_at_ms);
        assert_eq!(density.updated_at_ms, gap_transcript.updated_at_ms);

        let after = engine
            .load_transcript(&session_id)
            .expect("reload gap transcript");
        assert_eq!(after, gap_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_density_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .turn_density_session_transcript("00000000-0000-0000-0000-000000000000")
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .turn_density_session_transcript("label:nonexistent")
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        assert!(engine.load_session(&anchor).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_density_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .turn_density_session_transcript("label:duplicate")
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_turn_density_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .turn_density_session_transcript("label:")
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_gap_ranges_raw_id_contiguous_transcript_reports_empty_ranges_and_leaves_state_untouched(
    ) {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "first prompt");
        extend_transcript(&engine, &id, &["second prompt", "third prompt"]);

        let before_transcript = engine.load_transcript(&id).expect("reload transcript");
        let before_session = engine.load_session(&id).expect("reload session");

        let output = render_command(
            &engine,
            CliCommand::TranscriptGapRanges {
                selector: id.clone(),
            },
        );
        let gap_ranges: SessionTranscriptGapRanges =
            serde_json::from_str(&output).expect("parse transcript-gap-ranges output");

        assert_eq!(gap_ranges.selector, id);
        assert_eq!(gap_ranges.resolved_session_id.to_string(), gap_ranges.selector);
        assert_eq!(gap_ranges.total_entries, 3);
        assert!(gap_ranges.gap_ranges.is_empty());
        assert_eq!(gap_ranges.created_at_ms, before_transcript.created_at_ms);
        assert_eq!(gap_ranges.updated_at_ms, before_transcript.updated_at_ms);

        let after_transcript = engine
            .load_transcript(&gap_ranges.selector)
            .expect("reload after gap-ranges");
        assert_eq!(after_transcript, before_transcript);
        let after_session = engine
            .load_session(&gap_ranges.selector)
            .expect("reload session after gap-ranges");
        assert_eq!(after_session, before_session);

        let normalized =
            normalize_timestamps(&output.replace(&gap_ranges.selector, "<session-id>"));
        assert_eq!(
            normalized,
            readme_output_block("transcript-gap-ranges <selector>", "json")
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_gap_ranges_latest_selector_targets_most_recently_active_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _older = bootstrap_session_id(&engine, "older transcript");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = bootstrap_session_id(&engine, "newer first");
        extend_transcript(&engine, &newer, &["newer follow-up"]);

        let output = render_command(
            &engine,
            CliCommand::TranscriptGapRanges {
                selector: "latest".to_string(),
            },
        );
        let gap_ranges: SessionTranscriptGapRanges =
            serde_json::from_str(&output).expect("parse transcript-gap-ranges latest output");

        assert_eq!(gap_ranges.selector, "latest");
        assert_eq!(gap_ranges.resolved_session_id.to_string(), newer);
        assert_eq!(gap_ranges.total_entries, 2);
        assert!(gap_ranges.gap_ranges.is_empty());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_gap_ranges_label_selector_resolves_to_labeled_session() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "labeled first");
        extend_transcript(&engine, &id, &["labeled follow-up"]);
        engine
            .rename_session(&id, "runtime-review")
            .expect("attach label for gap-ranges");

        let output = render_command(
            &engine,
            CliCommand::TranscriptGapRanges {
                selector: "label:runtime-review".to_string(),
            },
        );
        let gap_ranges: SessionTranscriptGapRanges =
            serde_json::from_str(&output).expect("parse transcript-gap-ranges label output");

        assert_eq!(gap_ranges.selector, "label:runtime-review");
        assert_eq!(gap_ranges.resolved_session_id.to_string(), id);
        assert_eq!(gap_ranges.total_entries, 2);
        assert!(gap_ranges.gap_ranges.is_empty());

        let reloaded = engine
            .load_session(&id)
            .expect("reload after gap-ranges");
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_gap_ranges_single_entry_transcript_reports_empty_ranges() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let id = bootstrap_session_id(&engine, "only prompt");

        let gap_ranges = engine
            .gap_ranges_session_transcript(&id)
            .expect("single-entry transcript should succeed");
        assert_eq!(gap_ranges.total_entries, 1);
        assert!(gap_ranges.gap_ranges.is_empty());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_gap_ranges_empty_transcript_reports_empty_ranges_cleanly() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist empty session");
        let empty_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: Vec::new(),
        };
        engine
            .store
            .save_transcript(&empty_transcript)
            .expect("persist empty transcript");

        let gap_ranges = engine
            .gap_ranges_session_transcript(&session_id)
            .expect("empty transcript should succeed");
        assert_eq!(gap_ranges.selector, session_id);
        assert_eq!(gap_ranges.resolved_session_id.to_string(), gap_ranges.selector);
        assert_eq!(gap_ranges.total_entries, 0);
        assert!(gap_ranges.gap_ranges.is_empty());
        assert_eq!(gap_ranges.created_at_ms, empty_transcript.created_at_ms);
        assert_eq!(gap_ranges.updated_at_ms, empty_transcript.updated_at_ms);

        let after = engine
            .load_transcript(&gap_ranges.selector)
            .expect("reload empty transcript");
        assert_eq!(after, empty_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_gap_ranges_single_missing_turn_collapses_to_singleton_range() {
        use harness_core::{Prompt, TurnIndex};

        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist seed session");

        let single_gap_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: vec![
                TranscriptEntry {
                    turn_index: TurnIndex(1),
                    prompt: Prompt::new("turn one"),
                },
                TranscriptEntry {
                    turn_index: TurnIndex(3),
                    prompt: Prompt::new("turn three - single gap at 2"),
                },
            ],
        };
        engine
            .store
            .save_transcript(&single_gap_transcript)
            .expect("persist single-gap transcript");

        let gap_ranges = engine
            .gap_ranges_session_transcript(&session_id)
            .expect("single-gap transcript should succeed");

        assert_eq!(gap_ranges.total_entries, 2);
        assert_eq!(
            gap_ranges.gap_ranges,
            vec![SessionTranscriptGapRange {
                start_turn_index: 2,
                end_turn_index: 2,
                missing_count: 1,
            }]
        );

        let after = engine
            .load_transcript(&session_id)
            .expect("reload single-gap transcript");
        assert_eq!(after, single_gap_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_gap_ranges_multiple_disjoint_gaps_produce_ascending_ranges() {
        use harness_core::{Prompt, TurnIndex};

        let root = temp_session_root();
        let engine = temp_engine(&root);

        let mut session = SessionState::default();
        session.messages.clear();
        let session_id = session.session_id.to_string();
        engine.store.save(&session).expect("persist seed session");

        let gap_transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: vec![
                TranscriptEntry {
                    turn_index: TurnIndex(1),
                    prompt: Prompt::new("turn one"),
                },
                TranscriptEntry {
                    turn_index: TurnIndex(4),
                    prompt: Prompt::new("turn four - gap at 2, 3"),
                },
                TranscriptEntry {
                    turn_index: TurnIndex(5),
                    prompt: Prompt::new("turn five"),
                },
                TranscriptEntry {
                    turn_index: TurnIndex(9),
                    prompt: Prompt::new("turn nine - gap at 6, 7, 8"),
                },
            ],
        };
        engine
            .store
            .save_transcript(&gap_transcript)
            .expect("persist multi-gap transcript");

        let gap_ranges = engine
            .gap_ranges_session_transcript(&session_id)
            .expect("multi-gap transcript should succeed");

        assert_eq!(gap_ranges.total_entries, 4);
        assert_eq!(
            gap_ranges.gap_ranges,
            vec![
                SessionTranscriptGapRange {
                    start_turn_index: 2,
                    end_turn_index: 3,
                    missing_count: 2,
                },
                SessionTranscriptGapRange {
                    start_turn_index: 6,
                    end_turn_index: 8,
                    missing_count: 3,
                },
            ]
        );
        assert_eq!(gap_ranges.created_at_ms, gap_transcript.created_at_ms);
        assert_eq!(gap_ranges.updated_at_ms, gap_transcript.updated_at_ms);

        let after = engine
            .load_transcript(&session_id)
            .expect("reload multi-gap transcript");
        assert_eq!(after, gap_transcript);

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_gap_ranges_unknown_id_and_label_surface_session_not_found() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .gap_ranges_session_transcript("00000000-0000-0000-0000-000000000000")
            .expect_err("unknown id should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown id, got: {err}"
        );

        let err = engine
            .gap_ranges_session_transcript("label:nonexistent")
            .expect_err("unknown label should fail");
        assert!(
            err.contains("session not found"),
            "expected session not found for unknown label, got: {err}"
        );

        assert!(engine.load_session(&anchor).is_ok());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_gap_ranges_duplicate_label_surfaces_ambiguous_label() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let first = bootstrap_session_id(&engine, "first dup");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = bootstrap_session_id(&engine, "second dup");
        engine
            .rename_session(&first, "duplicate")
            .expect("label first");
        engine
            .rename_session(&second, "duplicate")
            .expect("label second");

        let err = engine
            .gap_ranges_session_transcript("label:duplicate")
            .expect_err("duplicate label should fail");
        assert!(
            err.contains("ambiguous session label"),
            "expected ambiguous session label error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn transcript_gap_ranges_empty_label_surfaces_malformed_selector() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let _anchor = bootstrap_session_id(&engine, "anchor");

        let err = engine
            .gap_ranges_session_transcript("label:")
            .expect_err("empty label should fail");
        assert!(
            err.contains("malformed session selector"),
            "expected malformed session selector error, got: {err}"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }
}
