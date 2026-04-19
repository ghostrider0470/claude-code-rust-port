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
pub struct TranscriptRecord {
    pub session_id: SessionId,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub entries: Vec<TranscriptEntry>,
}

impl TranscriptRecord {
    pub fn from_session(session: &SessionState, transcript: &TranscriptStore) -> Self {
        Self {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: transcript.entries.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionExport {
    pub exported_session_id: SessionId,
    pub session: SessionState,
    pub transcript: TranscriptRecord,
}

impl SessionExport {
    pub fn new(session: SessionState, transcript: TranscriptRecord) -> Self {
        Self {
            exported_session_id: session.session_id.clone(),
            session,
            transcript,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionComparisonSide {
    pub session_id: SessionId,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub message_count: usize,
    pub transcript_entry_count: usize,
}

impl SessionComparisonSide {
    pub fn from_parts(session: &SessionState, transcript: &TranscriptRecord) -> Self {
        Self {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            message_count: session.messages.len(),
            transcript_entry_count: transcript.entries.len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionComparisonDifferences {
    pub same_session: bool,
    pub created_at_ms_delta: i64,
    pub updated_at_ms_delta: i64,
    pub message_count_delta: i64,
    pub transcript_entry_count_delta: i64,
}

impl SessionComparisonDifferences {
    pub fn between(left: &SessionComparisonSide, right: &SessionComparisonSide) -> Self {
        Self {
            same_session: left.session_id == right.session_id,
            created_at_ms_delta: signed_delta(left.created_at_ms, right.created_at_ms),
            updated_at_ms_delta: signed_delta(left.updated_at_ms, right.updated_at_ms),
            message_count_delta: signed_usize_delta(left.message_count, right.message_count),
            transcript_entry_count_delta: signed_usize_delta(
                left.transcript_entry_count,
                right.transcript_entry_count,
            ),
        }
    }
}

fn signed_delta(left: u64, right: u64) -> i64 {
    (right as i128 - left as i128) as i64
}

fn signed_usize_delta(left: usize, right: usize) -> i64 {
    (right as i128 - left as i128) as i64
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionComparison {
    pub left_session_id: SessionId,
    pub right_session_id: SessionId,
    pub left: SessionComparisonSide,
    pub right: SessionComparisonSide,
    pub differences: SessionComparisonDifferences,
}

impl SessionComparison {
    pub fn new(left: SessionComparisonSide, right: SessionComparisonSide) -> Self {
        let differences = SessionComparisonDifferences::between(&left, &right);
        Self {
            left_session_id: left.session_id.clone(),
            right_session_id: right.session_id.clone(),
            left,
            right,
            differences,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionDeletion {
    pub deleted_session_id: SessionId,
    pub removed_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionImport {
    pub imported_session_id: SessionId,
    pub session_path: String,
    pub transcript_path: String,
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

    pub fn save_transcript(
        &self,
        record: &TranscriptRecord,
    ) -> Result<PathBuf, RuntimeError> {
        fs::create_dir_all(&self.root).map_err(|err| RuntimeError::Io(err.to_string()))?;
        let path = self.transcript_path(&record.session_id.to_string());
        let body = serde_json::to_string_pretty(record)
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

    pub fn load_transcript(&self, session_id: &str) -> Result<TranscriptRecord, RuntimeError> {
        let path = self.transcript_path(session_id);
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

    pub fn latest_transcript(&self) -> Result<TranscriptRecord, RuntimeError> {
        let latest = self
            .list()?
            .into_iter()
            .next()
            .ok_or_else(|| RuntimeError::SessionNotFound("latest".to_string()))?;
        self.load_transcript(&latest.session_id.to_string())
    }

    pub fn transcript_path(&self, session_id: &str) -> PathBuf {
        self.root.join(format!("{session_id}.transcript.json"))
    }

    pub fn delete(&self, session_id: &str) -> Result<SessionDeletion, RuntimeError> {
        let session_path = self.root.join(format!("{}.json", session_id));
        if !session_path.exists() {
            return Err(RuntimeError::SessionNotFound(session_id.to_string()));
        }

        let session = self.load(session_id)?;

        let transcript_path = self.transcript_path(session_id);
        let mut removed_paths = Vec::new();

        fs::remove_file(&session_path).map_err(|err| RuntimeError::Io(err.to_string()))?;
        removed_paths.push(session_path.display().to_string());

        if transcript_path.exists() {
            fs::remove_file(&transcript_path).map_err(|err| RuntimeError::Io(err.to_string()))?;
            removed_paths.push(transcript_path.display().to_string());
        }

        Ok(SessionDeletion {
            deleted_session_id: session.session_id,
            removed_paths,
        })
    }

    pub fn import_bundle(
        &self,
        bundle: &SessionExport,
    ) -> Result<SessionImport, RuntimeError> {
        if bundle.exported_session_id != bundle.session.session_id {
            return Err(RuntimeError::InvalidBundle(format!(
                "exported_session_id {} does not match nested session.session_id {}",
                bundle.exported_session_id, bundle.session.session_id
            )));
        }
        if bundle.exported_session_id != bundle.transcript.session_id {
            return Err(RuntimeError::InvalidBundle(format!(
                "exported_session_id {} does not match nested transcript.session_id {}",
                bundle.exported_session_id, bundle.transcript.session_id
            )));
        }
        for (position, entry) in bundle.transcript.entries.iter().enumerate() {
            if entry.turn_index.0 != position {
                return Err(RuntimeError::InvalidBundle(format!(
                    "transcript entry at position {} declares turn_index {} (expected {})",
                    position, entry.turn_index.0, position
                )));
            }
        }

        let id = bundle.exported_session_id.to_string();
        let session_path = self.root.join(format!("{id}.json"));
        let transcript_path = self.transcript_path(&id);

        if session_path.exists() || transcript_path.exists() {
            return Err(RuntimeError::SessionAlreadyExists(id));
        }

        let saved_session_path = self.save(&bundle.session)?;
        let saved_transcript_path = self.save_transcript(&bundle.transcript)?;

        Ok(SessionImport {
            imported_session_id: bundle.session.session_id.clone(),
            session_path: saved_session_path.display().to_string(),
            transcript_path: saved_transcript_path.display().to_string(),
        })
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
            if path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.ends_with(".transcript.json"))
            {
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
    use super::{
        SessionComparison, SessionComparisonSide, SessionExport, SessionState, SessionStore,
        TranscriptEntry, TranscriptRecord, TranscriptStore,
    };
    use harness_core::{Prompt, RuntimeError, SessionId, TurnIndex};
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
    fn saves_and_loads_transcript_record_round_trip_preserving_order() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);
        let session = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_001,
            updated_at_ms: 1_700_000_000_050,
            messages: vec![Prompt::new("review bash"), Prompt::new("summary please")],
            usage: harness_core::UsageSummary {
                input_tokens: 4,
                output_tokens: 4,
            },
        };

        let mut transcript = TranscriptStore::default();
        transcript.append(Prompt::new("review bash"));
        transcript.append(Prompt::new("summary please"));

        let record = TranscriptRecord::from_session(&session, &transcript);
        let saved_path = store.save_transcript(&record).expect("save transcript");
        assert_eq!(
            saved_path,
            root.join(format!("{}.transcript.json", session.session_id))
        );

        let loaded = store
            .load_transcript(&session.session_id.to_string())
            .expect("load transcript");

        assert_eq!(loaded.session_id, session.session_id);
        assert_eq!(loaded.created_at_ms, session.created_at_ms);
        assert_eq!(loaded.updated_at_ms, session.updated_at_ms);
        let ordered: Vec<(usize, String)> = loaded
            .entries
            .iter()
            .map(|entry| (entry.turn_index.0, entry.prompt.0.clone()))
            .collect();
        assert_eq!(
            ordered,
            vec![
                (0, "review bash".to_string()),
                (1, "summary please".to_string()),
            ]
        );

        store.save(&session).expect("save session state");
        let listed_ids: Vec<String> = store
            .list()
            .expect("list persisted sessions")
            .into_iter()
            .map(|listing| listing.session_id.to_string())
            .collect();
        assert_eq!(listed_ids, vec![session.session_id.to_string()]);

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn latest_transcript_follows_most_recently_updated_session() {
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

        let mut older_transcript = TranscriptStore::default();
        older_transcript.append(Prompt::new("review bash"));
        let mut newer_transcript = TranscriptStore::default();
        newer_transcript.append(Prompt::new("summary"));

        store.save(&older).expect("save older session");
        store.save(&newer).expect("save newer session");
        store
            .save_transcript(&TranscriptRecord::from_session(&older, &older_transcript))
            .expect("save older transcript");
        store
            .save_transcript(&TranscriptRecord::from_session(&newer, &newer_transcript))
            .expect("save newer transcript");

        let latest = store.latest_transcript().expect("load latest transcript");
        assert_eq!(latest.session_id, newer.session_id);
        assert_eq!(latest.entries.len(), 1);
        assert_eq!(latest.entries[0].prompt.0, "summary");

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn session_export_bundles_session_and_transcript_and_confirms_id() {
        let session = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_001,
            updated_at_ms: 1_700_000_000_050,
            messages: vec![Prompt::new("review bash"), Prompt::new("summary please")],
            usage: harness_core::UsageSummary {
                input_tokens: 4,
                output_tokens: 4,
            },
        };

        let mut transcript = TranscriptStore::default();
        transcript.append(Prompt::new("review bash"));
        transcript.append(Prompt::new("summary please"));
        let record = TranscriptRecord::from_session(&session, &transcript);

        let export = SessionExport::new(session.clone(), record.clone());

        assert_eq!(export.exported_session_id, session.session_id);
        assert_eq!(export.session, session);
        assert_eq!(export.transcript, record);

        let serialized = serde_json::to_string(&export).expect("serialize export");
        let ordered: Vec<(usize, String)> = export
            .transcript
            .entries
            .iter()
            .map(|entry| (entry.turn_index.0, entry.prompt.0.clone()))
            .collect();
        assert_eq!(
            ordered,
            vec![
                (0, "review bash".to_string()),
                (1, "summary please".to_string()),
            ],
            "export transcript must preserve turn ordering"
        );

        let again = serde_json::to_string(&export).expect("serialize export again");
        assert_eq!(serialized, again, "export serialization should be deterministic");

        let roundtrip: SessionExport =
            serde_json::from_str(&serialized).expect("deserialize export");
        assert_eq!(roundtrip, export);
    }

    #[test]
    fn session_comparison_reports_signed_deltas_and_same_session_flag() {
        let left_state = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_000,
            updated_at_ms: 1_700_000_000_100,
            messages: vec![Prompt::new("review bash")],
            usage: harness_core::UsageSummary {
                input_tokens: 2,
                output_tokens: 2,
            },
        };
        let right_state = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_050,
            updated_at_ms: 1_700_000_000_500,
            messages: vec![
                Prompt::new("review bash"),
                Prompt::new("summary please"),
                Prompt::new("one more"),
            ],
            usage: harness_core::UsageSummary {
                input_tokens: 6,
                output_tokens: 6,
            },
        };

        let mut left_transcript = TranscriptStore::default();
        left_transcript.append(Prompt::new("review bash"));
        let mut right_transcript = TranscriptStore::default();
        right_transcript.append(Prompt::new("review bash"));
        right_transcript.append(Prompt::new("summary please"));
        right_transcript.append(Prompt::new("one more"));

        let left_record = TranscriptRecord::from_session(&left_state, &left_transcript);
        let right_record = TranscriptRecord::from_session(&right_state, &right_transcript);

        let left = SessionComparisonSide::from_parts(&left_state, &left_record);
        let right = SessionComparisonSide::from_parts(&right_state, &right_record);
        let comparison = SessionComparison::new(left.clone(), right.clone());

        assert_eq!(comparison.left_session_id, left_state.session_id);
        assert_eq!(comparison.right_session_id, right_state.session_id);
        assert_eq!(comparison.left, left);
        assert_eq!(comparison.right, right);
        assert!(!comparison.differences.same_session);
        assert_eq!(comparison.differences.created_at_ms_delta, 50);
        assert_eq!(comparison.differences.updated_at_ms_delta, 400);
        assert_eq!(comparison.differences.message_count_delta, 2);
        assert_eq!(comparison.differences.transcript_entry_count_delta, 2);

        let reversed = SessionComparison::new(right.clone(), left.clone());
        assert_eq!(reversed.differences.created_at_ms_delta, -50);
        assert_eq!(reversed.differences.updated_at_ms_delta, -400);
        assert_eq!(reversed.differences.message_count_delta, -2);
        assert_eq!(reversed.differences.transcript_entry_count_delta, -2);

        let self_compare = SessionComparison::new(left.clone(), left.clone());
        assert!(self_compare.differences.same_session);
        assert_eq!(self_compare.differences.created_at_ms_delta, 0);
        assert_eq!(self_compare.differences.updated_at_ms_delta, 0);
        assert_eq!(self_compare.differences.message_count_delta, 0);
        assert_eq!(self_compare.differences.transcript_entry_count_delta, 0);

        let serialized = serde_json::to_string(&comparison).expect("serialize comparison");
        let again = serde_json::to_string(&comparison).expect("serialize comparison again");
        assert_eq!(serialized, again, "comparison serialization should be deterministic");
        let roundtrip: SessionComparison =
            serde_json::from_str(&serialized).expect("deserialize comparison");
        assert_eq!(roundtrip, comparison);
    }

    #[test]
    fn delete_removes_session_and_transcript_and_reports_paths() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);
        let session = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_001,
            updated_at_ms: 1_700_000_000_050,
            messages: vec![Prompt::new("review bash")],
            usage: harness_core::UsageSummary {
                input_tokens: 2,
                output_tokens: 2,
            },
        };
        let mut transcript = TranscriptStore::default();
        transcript.append(Prompt::new("review bash"));
        let record = TranscriptRecord::from_session(&session, &transcript);

        let session_path = store.save(&session).expect("save session");
        let transcript_path = store.save_transcript(&record).expect("save transcript");

        let id = session.session_id.to_string();
        let deletion = store.delete(&id).expect("delete session");

        assert_eq!(deletion.deleted_session_id, session.session_id);
        assert_eq!(
            deletion.removed_paths,
            vec![
                session_path.display().to_string(),
                transcript_path.display().to_string(),
            ]
        );
        assert!(!session_path.exists());
        assert!(!transcript_path.exists());

        match store.load(&id) {
            Err(RuntimeError::SessionNotFound(missing)) => assert_eq!(missing, id),
            other => panic!("expected SessionNotFound after delete, got {other:?}"),
        }

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn delete_missing_session_errors_without_touching_sibling_sessions() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);
        let keeper = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_001,
            updated_at_ms: 1_700_000_000_050,
            messages: vec![Prompt::new("keep me")],
            usage: harness_core::UsageSummary {
                input_tokens: 1,
                output_tokens: 1,
            },
        };
        let mut keeper_transcript = TranscriptStore::default();
        keeper_transcript.append(Prompt::new("keep me"));
        let keeper_record = TranscriptRecord::from_session(&keeper, &keeper_transcript);

        let keeper_path = store.save(&keeper).expect("save keeper session");
        let keeper_transcript_path = store
            .save_transcript(&keeper_record)
            .expect("save keeper transcript");

        let missing = SessionId::new().to_string();
        match store.delete(&missing) {
            Err(RuntimeError::SessionNotFound(reported)) => assert_eq!(reported, missing),
            other => panic!("expected SessionNotFound for missing id, got {other:?}"),
        }

        assert!(keeper_path.exists(), "sibling session must not be deleted");
        assert!(
            keeper_transcript_path.exists(),
            "sibling transcript must not be deleted"
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn import_bundle_writes_session_and_transcript_preserving_id_metadata_and_turn_order() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);
        let session = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_001,
            updated_at_ms: 1_700_000_000_777,
            messages: vec![Prompt::new("review bash"), Prompt::new("summary please")],
            usage: harness_core::UsageSummary {
                input_tokens: 4,
                output_tokens: 4,
            },
        };
        let mut transcript = TranscriptStore::default();
        transcript.append(Prompt::new("review bash"));
        transcript.append(Prompt::new("summary please"));
        let record = TranscriptRecord::from_session(&session, &transcript);
        let bundle = SessionExport::new(session.clone(), record.clone());

        let imported = store.import_bundle(&bundle).expect("import bundle");

        assert_eq!(imported.imported_session_id, session.session_id);
        assert_eq!(
            imported.session_path,
            root.join(format!("{}.json", session.session_id))
                .display()
                .to_string()
        );
        assert_eq!(
            imported.transcript_path,
            root.join(format!("{}.transcript.json", session.session_id))
                .display()
                .to_string()
        );

        let reloaded = store
            .load(&session.session_id.to_string())
            .expect("reload imported session");
        assert_eq!(reloaded, session);
        assert_eq!(reloaded.created_at_ms, 1_700_000_000_001);
        assert_eq!(reloaded.updated_at_ms, 1_700_000_000_777);

        let reloaded_transcript = store
            .load_transcript(&session.session_id.to_string())
            .expect("reload imported transcript");
        assert_eq!(reloaded_transcript, record);
        let ordered: Vec<(usize, String)> = reloaded_transcript
            .entries
            .iter()
            .map(|entry| (entry.turn_index.0, entry.prompt.0.clone()))
            .collect();
        assert_eq!(
            ordered,
            vec![
                (0, "review bash".to_string()),
                (1, "summary please".to_string()),
            ]
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn import_bundle_rejects_existing_session_without_touching_siblings() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let keeper = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_001,
            updated_at_ms: 1_700_000_000_050,
            messages: vec![Prompt::new("keep me")],
            usage: harness_core::UsageSummary {
                input_tokens: 1,
                output_tokens: 1,
            },
        };
        let mut keeper_transcript = TranscriptStore::default();
        keeper_transcript.append(Prompt::new("keep me"));
        let keeper_record = TranscriptRecord::from_session(&keeper, &keeper_transcript);
        store.save(&keeper).expect("save keeper");
        store
            .save_transcript(&keeper_record)
            .expect("save keeper transcript");

        let bundle = SessionExport::new(keeper.clone(), keeper_record.clone());
        match store.import_bundle(&bundle) {
            Err(RuntimeError::SessionAlreadyExists(reported)) => {
                assert_eq!(reported, keeper.session_id.to_string());
            }
            other => panic!("expected SessionAlreadyExists, got {other:?}"),
        }

        let reloaded = store
            .load(&keeper.session_id.to_string())
            .expect("existing session must survive");
        assert_eq!(reloaded, keeper, "existing session must not be overwritten");
        let reloaded_transcript = store
            .load_transcript(&keeper.session_id.to_string())
            .expect("existing transcript must survive");
        assert_eq!(reloaded_transcript, keeper_record);

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn import_bundle_rejects_mismatched_session_ids_without_writing() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let session = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_001,
            updated_at_ms: 1_700_000_000_050,
            messages: vec![Prompt::new("one")],
            usage: harness_core::UsageSummary {
                input_tokens: 1,
                output_tokens: 1,
            },
        };
        let mut transcript = TranscriptStore::default();
        transcript.append(Prompt::new("one"));
        let mut record = TranscriptRecord::from_session(&session, &transcript);
        record.session_id = SessionId::new();
        let bundle = SessionExport::new(session.clone(), record);

        match store.import_bundle(&bundle) {
            Err(RuntimeError::InvalidBundle(_)) => {}
            other => panic!("expected InvalidBundle for mismatched transcript id, got {other:?}"),
        }

        assert!(
            !root.join(format!("{}.json", session.session_id)).exists(),
            "session JSON must not be written when bundle is invalid"
        );
        assert!(
            !root
                .join(format!("{}.transcript.json", session.session_id))
                .exists(),
            "transcript JSON must not be written when bundle is invalid"
        );

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn import_bundle_rejects_non_monotonic_turn_indexes() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let session = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_001,
            updated_at_ms: 1_700_000_000_050,
            messages: vec![Prompt::new("a"), Prompt::new("b")],
            usage: harness_core::UsageSummary {
                input_tokens: 2,
                output_tokens: 2,
            },
        };
        let record = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            entries: vec![
                TranscriptEntry {
                    turn_index: TurnIndex(1),
                    prompt: Prompt::new("a"),
                },
                TranscriptEntry {
                    turn_index: TurnIndex(2),
                    prompt: Prompt::new("b"),
                },
            ],
        };
        let bundle = SessionExport::new(session.clone(), record);

        match store.import_bundle(&bundle) {
            Err(RuntimeError::InvalidBundle(_)) => {}
            other => panic!("expected InvalidBundle for non-monotonic turn indexes, got {other:?}"),
        }

        assert!(!root.join(format!("{}.json", session.session_id)).exists());

        fs::remove_dir_all(&root).ok();
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
