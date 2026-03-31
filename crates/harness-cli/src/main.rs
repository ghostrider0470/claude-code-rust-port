use clap::{Parser, Subcommand};
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
            let matches = engine.route(&prompt);
            println!("{}", serde_json::to_string_pretty(&matches).unwrap());
        }
        CliCommand::Bootstrap { prompt } => {
            let report = engine.bootstrap(&prompt).unwrap();
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        CliCommand::Tools => {
            println!(
                "{}",
                serde_json::to_string_pretty(engine.tools.list()).unwrap()
            );
        }
        CliCommand::Commands => {
            println!(
                "{}",
                serde_json::to_string_pretty(engine.commands.list()).unwrap()
            );
        }
        CliCommand::SessionShow { id } => {
            let session = engine.load_session(&id).unwrap();
            println!("{}", serde_json::to_string_pretty(&session).unwrap());
        }
    }
}
