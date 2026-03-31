use harness_core::{PermissionDenial, ToolName};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: ToolName,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResult {
    pub name: ToolName,
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

    pub fn denial_for(&self, tool_name: &ToolName) -> Option<PermissionDenial> {
        let lowered = tool_name.0.to_ascii_lowercase();
        self.denied_prefixes
            .iter()
            .find(|prefix| lowered.starts_with(&prefix.to_ascii_lowercase()))
            .map(|_| PermissionDenial {
                subject: tool_name.to_string(),
                reason: "tool blocked by permission policy".to_string(),
            })
    }

    pub fn denied_prefixes(&self) -> &[String] {
        &self.denied_prefixes
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
                    name: ToolName::new("ReadFile"),
                    description: "Read a file from disk".into(),
                },
                ToolDefinition {
                    name: ToolName::new("EditFile"),
                    description: "Edit a file on disk".into(),
                },
                ToolDefinition {
                    name: ToolName::new("Bash"),
                    description: "Execute shell commands".into(),
                },
            ],
        }
    }

    pub fn list(&self) -> &[ToolDefinition] {
        &self.tools
    }

    pub fn execute(&self, name: &ToolName, payload: &str) -> ToolResult {
        match self
            .tools
            .iter()
            .find(|tool| tool.name.0.eq_ignore_ascii_case(&name.0))
        {
            Some(tool) => ToolResult {
                name: tool.name.clone(),
                handled: true,
                message: format!("tool '{}' would handle payload {:?}", tool.name, payload),
            },
            None => ToolResult {
                name: name.clone(),
                handled: false,
                message: format!("unknown tool: {}", name),
            },
        }
    }
}
