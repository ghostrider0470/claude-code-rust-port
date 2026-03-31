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

fn main() {
    let cli = Cli::parse();
    let engine = RuntimeEngine::default();

    match cli.command {
        CliCommand::Summary => {
            println!("{}", engine.summary());
        }
        CliCommand::Route { prompt } => {
            let matches = engine.route(&Prompt::new(prompt));
            println!(
                "{}",
                serde_json::to_string_pretty(&matches).expect("serialize route result")
            );
        }
        CliCommand::Bootstrap { prompt } => {
            let report = engine
                .bootstrap(Prompt::new(prompt))
                .expect("bootstrap runtime turn");
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("serialize bootstrap report")
            );
        }
        CliCommand::Tools => {
            println!(
                "{}",
                serde_json::to_string_pretty(engine.tools.list()).expect("serialize tool list")
            );
        }
        CliCommand::Commands => {
            println!(
                "{}",
                serde_json::to_string_pretty(engine.commands.list())
                    .expect("serialize command list")
            );
        }
        CliCommand::SessionShow { id } => {
            let session = engine.load_session(&id).expect("load session by id");
            println!(
                "{}",
                serde_json::to_string_pretty(&session).expect("serialize session")
            );
        }
    }
}
