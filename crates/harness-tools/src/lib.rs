use harness_core::PermissionDenial;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResult {
    pub name: String,
    pub handled: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PermissionPolicy {
    denied_prefixes: Vec<String>,
}

impl PermissionPolicy {
    pub fn with_denied_prefixes(prefixes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            denied_prefixes: prefixes.into_iter().map(Into::into).collect(),
        }
    }

    pub fn denial_for(&self, tool_name: &str) -> Option<PermissionDenial> {
        let lowered = tool_name.to_ascii_lowercase();
        self.denied_prefixes
            .iter()
            .find(|prefix| lowered.starts_with(&prefix.to_ascii_lowercase()))
            .map(|_| PermissionDenial {
                subject: tool_name.to_string(),
                reason: "tool blocked by permission policy".to_string(),
            })
    }
}

#[derive(Debug, Clone, Default)]
pub struct ToolRegistry {
    tools: Vec<ToolDefinition>,
}

impl ToolRegistry {
    pub fn seeded() -> Self {
        Self {
            tools: vec![
                ToolDefinition {
                    name: "ReadFile".into(),
                    description: "Read a file from disk".into(),
                },
                ToolDefinition {
                    name: "EditFile".into(),
                    description: "Edit a file on disk".into(),
                },
                ToolDefinition {
                    name: "Bash".into(),
                    description: "Execute shell commands".into(),
                },
            ],
        }
    }

    pub fn list(&self) -> &[ToolDefinition] {
        &self.tools
    }

    pub fn find(&self, query: &str) -> Vec<ToolDefinition> {
        let needle = query.to_ascii_lowercase();
        self.tools
            .iter()
            .filter(|tool| {
                tool.name.to_ascii_lowercase().contains(&needle)
                    || tool.description.to_ascii_lowercase().contains(&needle)
            })
            .cloned()
            .collect()
    }

    pub fn execute(&self, name: &str, payload: &str) -> ToolResult {
        match self
            .tools
            .iter()
            .find(|tool| tool.name.eq_ignore_ascii_case(name))
        {
            Some(tool) => ToolResult {
                name: tool.name.clone(),
                handled: true,
                message: format!("tool '{}' would handle payload {:?}", tool.name, payload),
            },
            None => ToolResult {
                name: name.to_string(),
                handled: false,
                message: format!("unknown tool: {}", name),
            },
        }
    }
}
