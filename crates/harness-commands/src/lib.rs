use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandDefinition {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandResult {
    pub name: String,
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
                    name: "review".into(),
                    description: "Review code or diffs".into(),
                },
                CommandDefinition {
                    name: "agents".into(),
                    description: "Inspect agent state".into(),
                },
                CommandDefinition {
                    name: "setup".into(),
                    description: "Show runtime setup state".into(),
                },
            ],
        }
    }

    pub fn list(&self) -> &[CommandDefinition] {
        &self.commands
    }

    pub fn find(&self, query: &str) -> Vec<CommandDefinition> {
        let needle = query.to_ascii_lowercase();
        self.commands
            .iter()
            .filter(|command| {
                command.name.to_ascii_lowercase().contains(&needle)
                    || command.description.to_ascii_lowercase().contains(&needle)
            })
            .cloned()
            .collect()
    }

    pub fn execute(&self, name: &str, prompt: &str) -> CommandResult {
        match self
            .commands
            .iter()
            .find(|command| command.name.eq_ignore_ascii_case(name))
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
                name: name.to_string(),
                handled: false,
                message: format!("unknown command: {}", name),
            },
        }
    }
}
