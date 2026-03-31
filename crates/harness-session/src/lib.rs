use std::fs;
use std::path::PathBuf;

use harness_core::{Prompt, RuntimeError, SessionId, TurnIndex, UsageSummary};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptEntry {
    pub turn_index: TurnIndex,
    pub prompt: Prompt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TranscriptStore {
    pub entries: Vec<TranscriptEntry>,
    pub flushed: bool,
}

impl TranscriptStore {
    pub fn append(&mut self, prompt: Prompt) {
        let turn_index = TurnIndex(self.entries.len());
        self.entries.push(TranscriptEntry { turn_index, prompt });
        self.flushed = false;
    }

    pub fn replay(&self) -> Vec<Prompt> {
        self.entries
            .iter()
            .map(|entry| entry.prompt.clone())
            .collect()
    }

    pub fn compact(&mut self, keep_last: usize) {
        if self.entries.len() > keep_last {
            let start = self.entries.len() - keep_last;
            self.entries = self.entries[start..].to_vec();
        }
    }

    pub fn flush(&mut self) {
        self.flushed = true;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: SessionId,
    pub messages: Vec<Prompt>,
    pub usage: UsageSummary,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            session_id: SessionId::new(),
            messages: Vec::new(),
            usage: UsageSummary::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionStore {
    root: PathBuf,
}

impl SessionStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn default_root() -> PathBuf {
        PathBuf::from(".sessions")
    }

    pub fn save(&self, session: &SessionState) -> Result<PathBuf, RuntimeError> {
        fs::create_dir_all(&self.root).map_err(|err| RuntimeError::Io(err.to_string()))?;
        let path = self.root.join(format!("{}.json", session.session_id));
        let body = serde_json::to_string_pretty(session)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        fs::write(&path, body).map_err(|err| RuntimeError::Io(err.to_string()))?;
        Ok(path)
    }

    pub fn load(&self, session_id: &str) -> Result<SessionState, RuntimeError> {
        let path = self.root.join(format!("{}.json", session_id));
        if !path.exists() {
            return Err(RuntimeError::SessionNotFound(session_id.to_string()));
        }
        let body = fs::read_to_string(&path).map_err(|err| RuntimeError::Io(err.to_string()))?;
        serde_json::from_str(&body).map_err(|err| RuntimeError::Serialization(err.to_string()))
    }
}
