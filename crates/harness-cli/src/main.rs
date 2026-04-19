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
        SessionComparison, SessionDeletion, SessionExport, SessionFindResult, SessionImport,
        SessionState, SessionStore, TranscriptRecord,
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
}
