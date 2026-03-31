use serde::{Deserialize, Serialize};
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
    SessionStarted { session_id: SessionId },
    PromptReceived { prompt: String },
    RouteComputed { match_count: usize },
    CommandMatched { name: String, score: usize },
    ToolMatched { name: String, score: usize },
    PermissionDenied { subject: String, reason: String },
    CommandInvoked { name: String },
    CommandCompleted { name: String, handled: bool },
    ToolInvoked { name: String },
    ToolCompleted { name: String, handled: bool },
    TurnCompleted { stop_reason: String },
    SessionPersisted { path: String },
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("io error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("session not found: {0}")]
    SessionNotFound(String),
}

pub fn estimate_tokens(text: &str) -> usize {
    text.split_whitespace().count().max(1)
}
