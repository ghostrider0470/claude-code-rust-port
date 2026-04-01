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
    Tools,
    Commands,
    Sessions,
    SessionShow { id: String },
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
    use harness_session::{SessionState, SessionStore};
    use harness_tools::{PermissionPolicy, ToolRegistry};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    const README: &str = include_str!("../../../README.md");

    fn temp_session_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("harness-cli-tests-{nonce}"))
    }

    fn temp_engine(root: &PathBuf) -> RuntimeEngine {
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

    fn normalize_created_at_ms(output: &str) -> String {
        let marker = "\"created_at_ms\": ";
        if let Some(start) = output.find(marker) {
            let value_start = start + marker.len();
            let value_end = output[value_start..]
                .find(|ch: char| !ch.is_ascii_digit())
                .map(|offset| value_start + offset)
                .unwrap_or(output.len());

            let mut normalized = String::with_capacity(output.len());
            normalized.push_str(&output[..value_start]);
            normalized.push_str("<created-at-ms>");
            normalized.push_str(&output[value_end..]);
            normalized
        } else {
            output.to_string()
        }
    }

    fn normalize_bootstrap_example(output: &str, session_id: &str, root: &PathBuf) -> String {
        normalize_created_at_ms(
            &output
                .replace(session_id, "<session-id>")
                .replace(root.to_string_lossy().as_ref(), ".sessions"),
        )
    }

    fn normalize_session_output(output: &str, session_id: &str) -> String {
        normalize_created_at_ms(&output.replace(session_id, "<session-id>"))
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
