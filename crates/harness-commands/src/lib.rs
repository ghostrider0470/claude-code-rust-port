use harness_core::CommandName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandDefinition {
    pub name: CommandName,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandResult {
    pub name: CommandName,
    pub handled: bool,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct CommandRegistry {
    commands: Vec<CommandDefinition>,
}

impl CommandRegistry {
    pub fn seeded() -> Self {
        Self {
            commands: vec![
                CommandDefinition {
                    name: CommandName::new("review"),
                    description: "Review code or diffs".into(),
                },
                CommandDefinition {
                    name: CommandName::new("agents"),
                    description: "Inspect agent state".into(),
                },
                CommandDefinition {
                    name: CommandName::new("setup"),
                    description: "Show runtime setup state".into(),
                },
            ],
        }
    }

    pub fn list(&self) -> &[CommandDefinition] {
        &self.commands
    }

    pub fn execute(&self, name: &CommandName, prompt: &str) -> CommandResult {
        match self
            .commands
            .iter()
            .find(|command| command.name.0.eq_ignore_ascii_case(&name.0))
        {
            Some(command) => CommandResult {
                name: command.name.clone(),
                handled: true,
                message: format!(
                    "command '{}' would handle prompt {:?}",
                    command.name, prompt
                ),
            },
            None => CommandResult {
                name: name.clone(),
                handled: false,
                message: format!("unknown command: {}", name),
            },
        }
    }
}
