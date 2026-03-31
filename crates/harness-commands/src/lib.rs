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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_registry_exposes_expected_command_names() {
        let registry = CommandRegistry::seeded();
        let names: Vec<_> = registry
            .list()
            .iter()
            .map(|command| command.name.to_string())
            .collect();

        assert_eq!(names, vec!["review", "agents", "setup"]);
    }

    #[test]
    fn execute_matches_seeded_commands_case_insensitively() {
        let registry = CommandRegistry::seeded();
        let result = registry.execute(&CommandName::new("REVIEW"), "review this PR");

        assert!(result.handled);
        assert_eq!(result.name.to_string(), "review");
        assert!(result.message.contains("review this PR"));
    }

    #[test]
    fn execute_reports_unknown_commands() {
        let registry = CommandRegistry::seeded();
        let result = registry.execute(&CommandName::new("deploy"), "ship it");

        assert!(!result.handled);
        assert_eq!(result.name.to_string(), "deploy");
        assert_eq!(result.message, "unknown command: deploy");
    }
}
