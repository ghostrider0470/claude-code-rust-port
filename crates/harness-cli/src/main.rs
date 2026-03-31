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
        CliCommand::Tools => serde_json::to_string_pretty(engine.tools.list())
            .expect("serialize tool list"),
        CliCommand::Commands => serde_json::to_string_pretty(engine.commands.list())
            .expect("serialize command list"),
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
    use harness_commands::{CommandDefinition, CommandRegistry};
    use harness_session::{SessionState, SessionStore};
    use harness_tools::{PermissionPolicy, ToolDefinition, ToolRegistry};
    use harness_runtime::RuntimeEngine;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    #[test]
    fn summary_renders_seeded_runtime_counts() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let output = render_command(&engine, CliCommand::Summary);

        assert_eq!(output, "commands=3 tools=3 denied_prefixes=bash");

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn route_renders_seeded_runtime_matches_json() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let output = render_command(
            &engine,
            CliCommand::Route {
                prompt: "review bash".to_string(),
            },
        );
        let routed: serde_json::Value = serde_json::from_str(&output).expect("parse route output");

        assert_eq!(
            routed,
            serde_json::json!([
                {
                    "kind": "command",
                    "name": "review",
                    "score": 1
                },
                {
                    "kind": "tool",
                    "name": "Bash",
                    "score": 1
                }
            ])
        );

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn tools_renders_seeded_tool_registry_json() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let output = render_command(&engine, CliCommand::Tools);
        let tools: Vec<ToolDefinition> = serde_json::from_str(&output).expect("parse tool list");

        let names: Vec<String> = tools.into_iter().map(|tool| tool.name.0).collect();
        assert_eq!(
            names,
            vec![
                "ReadFile".to_string(),
                "EditFile".to_string(),
                "Bash".to_string()
            ]
        );

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn commands_renders_seeded_command_registry_json() {
        let root = temp_session_root();
        let engine = temp_engine(&root);

        let output = render_command(&engine, CliCommand::Commands);
        let commands: Vec<CommandDefinition> =
            serde_json::from_str(&output).expect("parse command list");

        let names: Vec<String> = commands.into_iter().map(|command| command.name.0).collect();
        assert_eq!(
            names,
            vec![
                "review".to_string(),
                "agents".to_string(),
                "setup".to_string()
            ]
        );

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn bootstrap_then_session_show_round_trips_persisted_session_json() {
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
            session
                .messages
                .into_iter()
                .map(|prompt| prompt.0)
                .collect::<Vec<_>>(),
            vec!["review bash".to_string()]
        );

        fs::remove_dir_all(&root).expect("remove temp cli test directory");
    }
}
