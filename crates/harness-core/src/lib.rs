use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct TurnIndex(pub usize);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Prompt(pub String);

impl Prompt {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolName(pub String);

impl ToolName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for ToolName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandName(pub String);

impl CommandName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for CommandName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct MatchScore(pub usize);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UsageSummary {
    pub input_tokens: usize,
    pub output_tokens: usize,
}

impl UsageSummary {
    pub fn add_turn(&self, prompt: &str, output: &str) -> Self {
        Self {
            input_tokens: self.input_tokens + estimate_tokens(prompt),
            output_tokens: self.output_tokens + estimate_tokens(output),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionDenial {
    pub subject: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeEvent {
    SessionStarted {
        session_id: SessionId,
    },
    SessionResumed {
        session_id: SessionId,
        turn_index: TurnIndex,
    },
    PromptReceived {
        prompt: Prompt,
    },
    RouteComputed {
        match_count: usize,
    },
    CommandMatched {
        name: CommandName,
        score: MatchScore,
    },
    ToolMatched {
        name: ToolName,
        score: MatchScore,
    },
    PermissionDenied {
        subject: String,
        reason: String,
    },
    CommandInvoked {
        name: CommandName,
    },
    CommandCompleted {
        name: CommandName,
        handled: bool,
    },
    ToolInvoked {
        name: ToolName,
    },
    ToolCompleted {
        name: ToolName,
        handled: bool,
    },
    TurnCompleted {
        stop_reason: String,
    },
    SessionPersisted {
        path: String,
    },
    TranscriptPersisted {
        path: String,
    },
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("io error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("session already exists: {0}")]
    SessionAlreadyExists(String),
    #[error("invalid session bundle: {0}")]
    InvalidBundle(String),
    #[error("invalid session label: {0}")]
    InvalidLabel(String),
    #[error("ambiguous session label: {0}")]
    AmbiguousLabel(String),
    #[error("malformed session selector: {0}")]
    MalformedSelector(String),
    #[error("session already unlabeled: {0}")]
    SessionAlreadyUnlabeled(String),
}

pub fn estimate_tokens(text: &str) -> usize {
    text.split_whitespace().count().max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_and_names_expose_string_views() {
        let prompt = Prompt::new("review this diff");
        let tool = ToolName::new("ReadFile");
        let command = CommandName::new("review");

        assert_eq!(prompt.as_str(), "review this diff");
        assert_eq!(tool.to_string(), "ReadFile");
        assert_eq!(command.to_string(), "review");
    }

    #[test]
    fn usage_summary_accumulates_estimated_tokens() {
        let usage = UsageSummary::default().add_turn("review this", "looks good");

        assert_eq!(usage.input_tokens, 2);
        assert_eq!(usage.output_tokens, 2);
    }

    #[test]
    fn estimate_tokens_never_returns_zero() {
        assert_eq!(estimate_tokens(""), 1);
        assert_eq!(estimate_tokens("one two three"), 3);
    }
}
