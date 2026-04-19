use clap::{Parser, Subcommand};
use harness_core::Prompt;
use harness_runtime::RuntimeEngine;

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
    Route { prompt: String },
    Bootstrap { prompt: String },
    Resume { id: String, prompt: String },
    Tools,
    Commands,
    Sessions,
    SessionShow { id: String },
    TranscriptShow { id: String },
    SessionExport { id: String },
    SessionCompare { left: String, right: String },
    SessionDelete { id: String },
    SessionImport { path: String },
    SessionFind { query: String },
    SessionFork { id: String, prompt: String },
    SessionRename { id: String, label: String },
    SessionUnlabel { id: String },
    SessionRetag { id: String, label: String },
    SessionLabels,
    SessionPrune {
        #[arg(long)]
        keep: usize,
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
            let transcript = engine
                .load_transcript(&id)
                .expect("load transcript by id");
            serde_json::to_string_pretty(&transcript).expect("serialize transcript")
        }
        CliCommand::SessionExport { id } => {
            let export = engine.export_session(&id).expect("export persisted session");
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
        CliCommand::SessionPrune { keep } => {
            let pruned = engine
                .prune_sessions(keep)
                .expect("prune persisted sessions");
            serde_json::to_string_pretty(&pruned).expect("serialize session prune")
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
    use super::{render_command, CliCommand};
    use harness_commands::CommandRegistry;
    use harness_runtime::RuntimeEngine;
    use harness_session::{
        SessionComparison, SessionDeletion, SessionExport, SessionFindResult, SessionFork,
        SessionImport, SessionLabelEntry, SessionPrune, SessionRename, SessionRetag, SessionState,
        SessionStore, SessionUnlabel, TranscriptRecord,
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

        assert_eq!(
            output,
            readme_output_block("route \"review bash\"", "json")
        );

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
            assert!(!std::path::Path::new(path).exists(), "removed path must not exist");
        }

        assert_eq!(
            normalize_bootstrap_example(&delete_output, &session_id, &root),
            readme_output_block("session-delete <id>", "json")
        );

        let after = engine.list_sessions().expect("list after delete");
        assert!(after.is_empty(), "session listing must be empty after delete");

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
        let bundle_path = std::env::temp_dir()
            .join(format!("harness-cli-import-bundle-{session_id}.json"));
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

        let missing = std::env::temp_dir()
            .join("harness-cli-import-bundle-definitely-missing-xyzzy.json");
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
        assert_eq!(results.len(), 1, "exactly one persisted session should match");
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
            malformed.unwrap_err().contains("malformed session selector"),
            "malformed-selector error must mention malformed selector"
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }

    #[test]
    fn session_labels_matches_readme_example_and_orders_newest_first_omits_unlabeled_and_keeps_duplicates_separate() {
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
            assert!(
                result.is_err(),
                "empty/whitespace label must fail to retag"
            );
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

        assert_eq!(
            output,
            readme_output_block("session-prune <no-op>", "json")
        );

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

        assert!(engine.list_sessions().expect("list after full prune").is_empty());

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }
}
