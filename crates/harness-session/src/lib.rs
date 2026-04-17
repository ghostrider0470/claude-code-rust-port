use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use harness_core::{Prompt, RuntimeError, SessionId, TurnIndex, UsageSummary};
use serde::{Deserialize, Serialize};

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_millis() as u64
}

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
    #[serde(default = "current_timestamp_ms")]
    pub created_at_ms: u64,
    #[serde(default = "current_timestamp_ms")]
    pub updated_at_ms: u64,
    pub messages: Vec<Prompt>,
    pub usage: UsageSummary,
}

impl Default for SessionState {
    fn default() -> Self {
        let now = current_timestamp_ms();
        Self {
            session_id: SessionId::new(),
            created_at_ms: now,
            updated_at_ms: now,
            messages: Vec::new(),
            usage: UsageSummary::default(),
        }
    }
}

impl SessionState {
    pub fn touch(&mut self) {
        self.updated_at_ms = current_timestamp_ms();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionListing {
    pub session_id: SessionId,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub message_count: usize,
    pub persisted_path: String,
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

    pub fn latest(&self) -> Result<SessionState, RuntimeError> {
        let latest = self
            .list()?
            .into_iter()
            .next()
            .ok_or_else(|| RuntimeError::SessionNotFound("latest".to_string()))?;
        self.load(&latest.session_id.to_string())
    }

    pub fn list(&self) -> Result<Vec<SessionListing>, RuntimeError> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        let entries = fs::read_dir(&self.root).map_err(|err| RuntimeError::Io(err.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|err| RuntimeError::Io(err.to_string()))?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }

            let body = fs::read_to_string(&path).map_err(|err| RuntimeError::Io(err.to_string()))?;
            let session: SessionState = serde_json::from_str(&body)
                .map_err(|err| RuntimeError::Serialization(err.to_string()))?;

            sessions.push(SessionListing {
                session_id: session.session_id,
                created_at_ms: session.created_at_ms,
                updated_at_ms: session.updated_at_ms,
                message_count: session.messages.len(),
                persisted_path: path.display().to_string(),
            });
        }

        sessions.sort_by(|left, right| {
            right
                .updated_at_ms
                .cmp(&left.updated_at_ms)
                .then_with(|| right.created_at_ms.cmp(&left.created_at_ms))
                .then_with(|| left.session_id.to_string().cmp(&right.session_id.to_string()))
                .then_with(|| left.persisted_path.cmp(&right.persisted_path))
        });

        Ok(sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::{SessionState, SessionStore, TranscriptStore};
    use harness_core::{Prompt, SessionId};
    use std::collections::BTreeMap;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_session_root() -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("harness-session-tests-{nonce}"))
    }

    #[test]
    fn saves_and_loads_session_state_round_trip() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);
        let session = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_001,
            updated_at_ms: 1_700_000_000_001,
            messages: vec![Prompt::new("review the runtime lane")],
            usage: harness_core::UsageSummary {
                input_tokens: 4,
                output_tokens: 2,
            },
        };

        let saved_path = store.save(&session).expect("save session state");
        let loaded = store
            .load(&session.session_id.to_string())
            .expect("load session state");

        assert_eq!(saved_path, root.join(format!("{}.json", session.session_id)));
        assert_eq!(loaded, session);

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn transcript_compaction_keeps_most_recent_entries() {
        let mut transcript = TranscriptStore::default();
        transcript.append(Prompt::new("first"));
        transcript.append(Prompt::new("second"));
        transcript.append(Prompt::new("third"));

        transcript.compact(2);

        let prompts: Vec<String> = transcript
            .replay()
            .into_iter()
            .map(|prompt| prompt.0)
            .collect();

        assert_eq!(prompts, vec!["second".to_string(), "third".to_string()]);
        assert!(!transcript.flushed);
    }

    #[test]
    fn lists_persisted_sessions_newest_first() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);
        let older = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_000,
            updated_at_ms: 1_700_000_000_000,
            messages: vec![Prompt::new("review bash")],
            usage: harness_core::UsageSummary {
                input_tokens: 2,
                output_tokens: 2,
            },
        };
        let newer = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_100,
            updated_at_ms: 1_700_000_000_100,
            messages: vec![Prompt::new("summary"), Prompt::new("tools")],
            usage: harness_core::UsageSummary {
                input_tokens: 2,
                output_tokens: 2,
            },
        };

        store.save(&older).expect("save older session");
        store.save(&newer).expect("save newer session");
        fs::write(root.join("notes.txt"), "ignore me").expect("write non-session fixture");

        let listed = store.list().expect("list persisted sessions");
        let listed_ids: Vec<String> = listed
            .iter()
            .map(|session| session.session_id.to_string())
            .collect();

        assert_eq!(
            listed_ids,
            vec![newer.session_id.to_string(), older.session_id.to_string()]
        );

        let by_id: BTreeMap<String, (u64, usize, String)> = listed
            .into_iter()
            .map(|session| {
                (
                    session.session_id.to_string(),
                    (
                        session.created_at_ms,
                        session.message_count,
                        session.persisted_path,
                    ),
                )
            })
            .collect();

        assert_eq!(by_id[&older.session_id.to_string()].0, older.created_at_ms);
        assert_eq!(by_id[&newer.session_id.to_string()].0, newer.created_at_ms);
        assert_eq!(by_id[&older.session_id.to_string()].1, 1);
        assert_eq!(by_id[&newer.session_id.to_string()].1, 2);
        assert_eq!(
            by_id[&older.session_id.to_string()].2,
            root.join(format!("{}.json", older.session_id))
                .display()
                .to_string()
        );
        assert_eq!(
            by_id[&newer.session_id.to_string()].2,
            root.join(format!("{}.json", newer.session_id))
                .display()
                .to_string()
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn loads_latest_persisted_session() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);
        let older = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_000,
            updated_at_ms: 1_700_000_000_000,
            messages: vec![Prompt::new("review bash")],
            usage: harness_core::UsageSummary {
                input_tokens: 2,
                output_tokens: 2,
            },
        };
        let newer = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_100,
            updated_at_ms: 1_700_000_000_100,
            messages: vec![Prompt::new("summary")],
            usage: harness_core::UsageSummary {
                input_tokens: 1,
                output_tokens: 1,
            },
        };

        store.save(&older).expect("save older session");
        store.save(&newer).expect("save newer session");

        let latest = store.latest().expect("load latest session");

        assert_eq!(latest, newer);

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn latest_follows_updated_at_when_older_session_is_resumed() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);
        let first = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_000,
            updated_at_ms: 1_700_000_000_000,
            messages: vec![Prompt::new("review bash")],
            usage: harness_core::UsageSummary {
                input_tokens: 2,
                output_tokens: 2,
            },
        };
        let second = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_100,
            updated_at_ms: 1_700_000_000_100,
            messages: vec![Prompt::new("summary")],
            usage: harness_core::UsageSummary {
                input_tokens: 1,
                output_tokens: 1,
            },
        };

        store.save(&first).expect("save first session");
        store.save(&second).expect("save second session");

        let mut resumed_first = first.clone();
        resumed_first.updated_at_ms = 1_700_000_000_500;
        resumed_first.messages.push(Prompt::new("follow up"));
        store.save(&resumed_first).expect("save resumed first session");

        let latest = store.latest().expect("load latest persisted session");
        assert_eq!(latest, resumed_first);

        let listed_ids: Vec<String> = store
            .list()
            .expect("list persisted sessions")
            .into_iter()
            .map(|session| session.session_id.to_string())
            .collect();
        assert_eq!(
            listed_ids,
            vec![
                resumed_first.session_id.to_string(),
                second.session_id.to_string(),
            ]
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }
}
