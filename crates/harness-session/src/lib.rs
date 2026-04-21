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

fn is_false(flag: &bool) -> bool {
    !*flag
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub pinned: bool,
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
            label: None,
            pinned: false,
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
    #[serde(default, skip_serializing_if = "is_false")]
    pub pinned: bool,
}

impl SessionComparisonSide {
    pub fn from_parts(session: &SessionState, transcript: &TranscriptRecord) -> Self {
        Self {
            session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            message_count: session.messages.len(),
            transcript_entry_count: transcript.entries.len(),
            pinned: session.pinned,
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
pub struct SessionFork {
    pub source_session_id: SessionId,
    pub forked_session_id: SessionId,
    pub appended_turn_index: TurnIndex,
    pub session_path: String,
    pub transcript_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRename {
    pub renamed_session_id: SessionId,
    pub applied_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionUnlabel {
    pub unlabeled_session_id: SessionId,
    pub removed_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRetag {
    pub retagged_session_id: SessionId,
    pub previous_label: String,
    pub applied_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionPruneRemoval {
    pub session_id: SessionId,
    pub session_path: String,
    pub transcript_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionPrune {
    pub kept_count: usize,
    pub pruned_count: usize,
    pub pinned_preserved_count: usize,
    pub removed: Vec<SessionPruneRemoval>,
    pub pinned_preserved: Vec<SessionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionPin {
    pub pinned_session_id: SessionId,
    pub pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionUnpin {
    pub unpinned_session_id: SessionId,
    pub pinned: bool,
}

pub fn normalize_label(raw: &str) -> Result<String, RuntimeError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(RuntimeError::InvalidLabel(
            "label must not be empty or whitespace-only".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

pub const LABEL_SELECTOR_PREFIX: &str = "label:";
pub const LATEST_SELECTOR: &str = "latest";

/// Parsed form of a single-session CLI selector. Raw ids are returned verbatim
/// (no UUID validation here — the load step surfaces unknown ids via
/// `SessionNotFound`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionSelector {
    Latest,
    Label(String),
    Id(String),
}

impl SessionSelector {
    /// Parse a CLI selector string. Recognises `latest`, `label:<name>`, and
    /// otherwise treats input as a raw id. `label:` with an empty/whitespace
    /// label is reported as a malformed selector so the failure is distinct
    /// from "unknown label" or "unknown id".
    pub fn parse(raw: &str) -> Result<Self, RuntimeError> {
        if raw == LATEST_SELECTOR {
            return Ok(Self::Latest);
        }
        if let Some(rest) = raw.strip_prefix(LABEL_SELECTOR_PREFIX) {
            let trimmed = rest.trim();
            if trimmed.is_empty() {
                return Err(RuntimeError::MalformedSelector(format!(
                    "label selector requires a non-empty label after `{LABEL_SELECTOR_PREFIX}`"
                )));
            }
            return Ok(Self::Label(trimmed.to_string()));
        }
        Ok(Self::Id(raw.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionListing {
    pub session_id: SessionId,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub message_count: usize,
    pub persisted_path: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionFindMatch {
    pub turn_index: TurnIndex,
    pub prompt: Prompt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionFindResult {
    pub session_id: SessionId,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub message_count: usize,
    pub persisted_path: String,
    pub matches: Vec<SessionFindMatch>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionLabelEntry {
    pub label: String,
    pub session_id: SessionId,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub message_count: usize,
    pub persisted_path: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionPinEntry {
    pub session_id: SessionId,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub message_count: usize,
    pub persisted_path: String,
    pub pinned: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSelectorCheck {
    pub selector: String,
    pub resolved_session_id: SessionId,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub message_count: usize,
    pub persisted_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub pinned: bool,
}

/// Default number of transcript entries returned by `tail_transcript` when the
/// caller does not specify an explicit count. Kept stable so CLI scripts can
/// rely on the documented contract.
pub const DEFAULT_TRANSCRIPT_TAIL_COUNT: usize = 10;

/// Default number of transcript entries returned by `range_transcript` when the
/// caller does not specify an explicit `--count`. Kept stable so CLI scripts
/// can rely on the documented contract.
pub const DEFAULT_TRANSCRIPT_RANGE_COUNT: usize = 10;

/// Default number of transcript entries returned on each side of the centered
/// turn by `context_transcript` when the caller does not specify an explicit
/// `--before` / `--after`. Kept stable so CLI scripts can rely on the
/// documented contract.
pub const DEFAULT_TRANSCRIPT_CONTEXT_WINDOW: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTranscriptTail {
    pub selector: String,
    pub resolved_session_id: SessionId,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub total_entries: usize,
    pub returned_entries: usize,
    pub entries: Vec<TranscriptEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTranscriptFind {
    pub selector: String,
    pub resolved_session_id: SessionId,
    pub query: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub total_entries: usize,
    pub match_count: usize,
    pub matches: Vec<SessionFindMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTranscriptRange {
    pub selector: String,
    pub resolved_session_id: SessionId,
    pub start_turn_index: usize,
    pub requested_count: usize,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub total_entries: usize,
    pub returned_entries: usize,
    pub entries: Vec<TranscriptEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTranscriptContext {
    pub selector: String,
    pub resolved_session_id: SessionId,
    pub center_turn_index: usize,
    pub requested_before: usize,
    pub requested_after: usize,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub total_entries: usize,
    pub returned_entries: usize,
    pub entries: Vec<TranscriptEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTranscriptTurnShow {
    pub selector: String,
    pub resolved_session_id: SessionId,
    pub turn_index: usize,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub total_entries: usize,
    pub entry: TranscriptEntry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTranscriptLastTurn {
    pub selector: String,
    pub resolved_session_id: SessionId,
    pub turn_index: usize,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub total_entries: usize,
    pub entry: TranscriptEntry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTranscriptFirstTurn {
    pub selector: String,
    pub resolved_session_id: SessionId,
    pub turn_index: usize,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub total_entries: usize,
    pub entry: TranscriptEntry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTranscriptEntryCount {
    pub selector: String,
    pub resolved_session_id: SessionId,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub total_entries: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTranscriptHasEntries {
    pub selector: String,
    pub resolved_session_id: SessionId,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub total_entries: usize,
    pub has_entries: bool,
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

    /// Resolve a CLI selector (`latest`, `label:<name>`, or a raw id) to a
    /// concrete persisted `session_id` string. Raw ids are returned verbatim
    /// without disk I/O so callers can still surface `SessionNotFound` from
    /// their own load step. `latest` and label selectors do touch disk because
    /// they need to scan persisted state.
    pub fn resolve_selector(&self, selector: &str) -> Result<String, RuntimeError> {
        match SessionSelector::parse(selector)? {
            SessionSelector::Latest => Ok(self.latest()?.session_id.to_string()),
            SessionSelector::Label(label) => self.resolve_label(&label),
            SessionSelector::Id(id) => Ok(id),
        }
    }

    /// Resolve a selector (raw id, `latest`, or `label:<name>`) and surface the
    /// resolved persisted session's descriptive metadata without mutating any
    /// persisted state. Useful for inspect-only CLI scripts that need to
    /// confirm which session a selector points at before running a mutating
    /// command. Preserves existing selector failure semantics: unknown
    /// id/label → `SessionNotFound`, duplicate labels → `AmbiguousLabel`,
    /// empty `label:` → `MalformedSelector`.
    pub fn check_selector(&self, selector: &str) -> Result<SessionSelectorCheck, RuntimeError> {
        let resolved_id = self.resolve_selector(selector)?;
        let session = self.load(&resolved_id)?;
        let persisted_path = self.root.join(format!("{resolved_id}.json"));
        Ok(SessionSelectorCheck {
            selector: selector.to_string(),
            resolved_session_id: session.session_id.clone(),
            created_at_ms: session.created_at_ms,
            updated_at_ms: session.updated_at_ms,
            message_count: session.messages.len(),
            persisted_path: persisted_path.display().to_string(),
            label: session.label.clone(),
            pinned: session.pinned,
        })
    }

    /// Resolve a selector (raw id, `latest`, or `label:<name>`) and return the
    /// newest `count` transcript entries for the matching persisted session in
    /// their original `turn_index` order. The persisted transcript is not
    /// mutated. `count` larger than the transcript length simply returns every
    /// available entry. Preserves existing selector failure semantics unchanged
    /// (`SessionNotFound` / `AmbiguousLabel` / `MalformedSelector`).
    pub fn tail_transcript(
        &self,
        selector: &str,
        count: usize,
    ) -> Result<SessionTranscriptTail, RuntimeError> {
        let resolved_id = self.resolve_selector(selector)?;
        let transcript = self.load_transcript(&resolved_id)?;
        let total = transcript.entries.len();
        let start = total.saturating_sub(count);
        let entries = transcript.entries[start..].to_vec();
        Ok(SessionTranscriptTail {
            selector: selector.to_string(),
            resolved_session_id: transcript.session_id,
            created_at_ms: transcript.created_at_ms,
            updated_at_ms: transcript.updated_at_ms,
            total_entries: total,
            returned_entries: entries.len(),
            entries,
        })
    }

    /// Resolve a selector (raw id, `latest`, or `label:<name>`) and search the
    /// matching persisted transcript for entries whose prompt text contains
    /// `query` case-insensitively (mirroring `find()` query semantics).
    /// Matches preserve the transcript's original `turn_index` ordering. An
    /// empty query matches zero entries. The persisted transcript is not
    /// mutated. Preserves existing selector failure semantics unchanged
    /// (`SessionNotFound` / `AmbiguousLabel` / `MalformedSelector`).
    pub fn find_in_transcript(
        &self,
        selector: &str,
        query: &str,
    ) -> Result<SessionTranscriptFind, RuntimeError> {
        let resolved_id = self.resolve_selector(selector)?;
        let transcript = self.load_transcript(&resolved_id)?;
        let total = transcript.entries.len();

        let matches: Vec<SessionFindMatch> = if query.is_empty() {
            Vec::new()
        } else {
            let needle = query.to_ascii_lowercase();
            transcript
                .entries
                .iter()
                .filter(|entry| entry.prompt.0.to_ascii_lowercase().contains(&needle))
                .map(|entry| SessionFindMatch {
                    turn_index: entry.turn_index,
                    prompt: entry.prompt.clone(),
                })
                .collect()
        };

        Ok(SessionTranscriptFind {
            selector: selector.to_string(),
            resolved_session_id: transcript.session_id,
            query: query.to_string(),
            created_at_ms: transcript.created_at_ms,
            updated_at_ms: transcript.updated_at_ms,
            total_entries: total,
            match_count: matches.len(),
            matches,
        })
    }

    /// Resolve a selector (raw id, `latest`, or `label:<name>`) and return a
    /// bounded forward slice of the matching persisted transcript beginning at
    /// `turn_index == start` and containing at most `count` entries in their
    /// original `turn_index` order. The persisted transcript is not mutated.
    /// A `start` past the end of the transcript (including on an empty
    /// transcript) returns a clean empty `entries` array rather than erroring,
    /// and a `count` larger than the number of remaining entries simply
    /// returns the available tail. Preserves existing selector failure
    /// semantics unchanged (`SessionNotFound` / `AmbiguousLabel` /
    /// `MalformedSelector`).
    pub fn range_transcript(
        &self,
        selector: &str,
        start: usize,
        count: usize,
    ) -> Result<SessionTranscriptRange, RuntimeError> {
        let resolved_id = self.resolve_selector(selector)?;
        let transcript = self.load_transcript(&resolved_id)?;
        let total = transcript.entries.len();
        let entries = if start >= total {
            Vec::new()
        } else {
            let end = start.saturating_add(count).min(total);
            transcript.entries[start..end].to_vec()
        };
        Ok(SessionTranscriptRange {
            selector: selector.to_string(),
            resolved_session_id: transcript.session_id,
            start_turn_index: start,
            requested_count: count,
            created_at_ms: transcript.created_at_ms,
            updated_at_ms: transcript.updated_at_ms,
            total_entries: total,
            returned_entries: entries.len(),
            entries,
        })
    }

    /// Resolve a selector (raw id, `latest`, or `label:<name>`) and return a
    /// bounded symmetric window around `turn` in the matching persisted
    /// transcript. The window includes the centered entry at
    /// `turn_index == turn` when present plus up to `before` preceding and
    /// `after` following entries in their original `turn_index` order. Windows
    /// that extend past transcript bounds are clipped cleanly to the available
    /// in-range entries. A `turn` past the end of the transcript, including
    /// on an empty transcript, returns an empty `entries` array rather than
    /// erroring. The persisted transcript is not mutated. Preserves existing
    /// selector failure semantics unchanged (`SessionNotFound` /
    /// `AmbiguousLabel` / `MalformedSelector`).
    pub fn context_transcript(
        &self,
        selector: &str,
        turn: usize,
        before: usize,
        after: usize,
    ) -> Result<SessionTranscriptContext, RuntimeError> {
        let resolved_id = self.resolve_selector(selector)?;
        let transcript = self.load_transcript(&resolved_id)?;
        let total = transcript.entries.len();
        let entries = if turn >= total {
            Vec::new()
        } else {
            let start = turn.saturating_sub(before);
            let end = turn.saturating_add(after).saturating_add(1).min(total);
            transcript.entries[start..end].to_vec()
        };
        Ok(SessionTranscriptContext {
            selector: selector.to_string(),
            resolved_session_id: transcript.session_id,
            center_turn_index: turn,
            requested_before: before,
            requested_after: after,
            created_at_ms: transcript.created_at_ms,
            updated_at_ms: transcript.updated_at_ms,
            total_entries: total,
            returned_entries: entries.len(),
            entries,
        })
    }

    /// Resolve a selector (raw id, `latest`, or `label:<name>`) and return the
    /// single persisted transcript entry whose `turn_index == turn` for the
    /// matching session. The persisted transcript is not mutated. Empty
    /// transcripts and out-of-range `turn` values fail cleanly and deterministically
    /// with `TranscriptTurnOutOfRange` rather than masking the miss as a
    /// silent empty result, because the command's contract is to return
    /// exactly one entry. Preserves existing selector failure semantics
    /// unchanged (`SessionNotFound` / `AmbiguousLabel` / `MalformedSelector`).
    pub fn turn_show_transcript(
        &self,
        selector: &str,
        turn: usize,
    ) -> Result<SessionTranscriptTurnShow, RuntimeError> {
        let resolved_id = self.resolve_selector(selector)?;
        let transcript = self.load_transcript(&resolved_id)?;
        let total = transcript.entries.len();
        if turn >= total {
            return Err(RuntimeError::TranscriptTurnOutOfRange(format!(
                "turn {turn} not found in transcript of length {total}"
            )));
        }
        let entry = transcript.entries[turn].clone();
        Ok(SessionTranscriptTurnShow {
            selector: selector.to_string(),
            resolved_session_id: transcript.session_id,
            turn_index: turn,
            created_at_ms: transcript.created_at_ms,
            updated_at_ms: transcript.updated_at_ms,
            total_entries: total,
            entry,
        })
    }

    /// Resolve a selector (raw id, `latest`, or `label:<name>`) and return the
    /// single persisted transcript entry with the highest available
    /// `turn_index` for the matching session. The persisted transcript is not
    /// mutated. An empty transcript has no last turn and therefore fails
    /// cleanly and deterministically with `TranscriptTurnOutOfRange` rather
    /// than masking the miss as a silent empty result, because the command's
    /// contract is to return exactly one entry. Preserves existing selector
    /// failure semantics unchanged (`SessionNotFound` / `AmbiguousLabel` /
    /// `MalformedSelector`).
    pub fn last_turn_transcript(
        &self,
        selector: &str,
    ) -> Result<SessionTranscriptLastTurn, RuntimeError> {
        let resolved_id = self.resolve_selector(selector)?;
        let transcript = self.load_transcript(&resolved_id)?;
        let total = transcript.entries.len();
        if total == 0 {
            return Err(RuntimeError::TranscriptTurnOutOfRange(format!(
                "no last turn in transcript of length {total}"
            )));
        }
        let last_index = total - 1;
        let entry = transcript.entries[last_index].clone();
        Ok(SessionTranscriptLastTurn {
            selector: selector.to_string(),
            resolved_session_id: transcript.session_id,
            turn_index: last_index,
            created_at_ms: transcript.created_at_ms,
            updated_at_ms: transcript.updated_at_ms,
            total_entries: total,
            entry,
        })
    }

    /// Resolve a selector (raw id, `latest`, or `label:<name>`) and return the
    /// single persisted transcript entry with the lowest available
    /// `turn_index` for the matching session. The persisted transcript is not
    /// mutated. An empty transcript has no first turn and therefore fails
    /// cleanly and deterministically with `TranscriptTurnOutOfRange` rather
    /// than masking the miss as a silent empty result, because the command's
    /// contract is to return exactly one entry. Preserves existing selector
    /// failure semantics unchanged (`SessionNotFound` / `AmbiguousLabel` /
    /// `MalformedSelector`).
    pub fn first_turn_transcript(
        &self,
        selector: &str,
    ) -> Result<SessionTranscriptFirstTurn, RuntimeError> {
        let resolved_id = self.resolve_selector(selector)?;
        let transcript = self.load_transcript(&resolved_id)?;
        let total = transcript.entries.len();
        if total == 0 {
            return Err(RuntimeError::TranscriptTurnOutOfRange(format!(
                "no first turn in transcript of length {total}"
            )));
        }
        let entry = transcript.entries[0].clone();
        Ok(SessionTranscriptFirstTurn {
            selector: selector.to_string(),
            resolved_session_id: transcript.session_id,
            turn_index: 0,
            created_at_ms: transcript.created_at_ms,
            updated_at_ms: transcript.updated_at_ms,
            total_entries: total,
            entry,
        })
    }

    /// Resolve a selector (raw id, `latest`, or `label:<name>`) and return a
    /// deterministic machine-readable summary of the resolved persisted
    /// transcript's length without returning any transcript entries. The
    /// persisted transcript is not mutated. An empty transcript succeeds
    /// cleanly with `total_entries: 0`. Preserves existing selector failure
    /// semantics unchanged (`SessionNotFound` / `AmbiguousLabel` /
    /// `MalformedSelector`).
    pub fn entry_count_transcript(
        &self,
        selector: &str,
    ) -> Result<SessionTranscriptEntryCount, RuntimeError> {
        let resolved_id = self.resolve_selector(selector)?;
        let transcript = self.load_transcript(&resolved_id)?;
        Ok(SessionTranscriptEntryCount {
            selector: selector.to_string(),
            resolved_session_id: transcript.session_id,
            created_at_ms: transcript.created_at_ms,
            updated_at_ms: transcript.updated_at_ms,
            total_entries: transcript.entries.len(),
        })
    }

    /// Resolve a selector (raw id, `latest`, or `label:<name>`) and return a
    /// deterministic machine-readable summary describing whether the resolved
    /// persisted transcript contains any entries, without returning the entries
    /// themselves. The persisted transcript is not mutated. Empty transcripts
    /// succeed cleanly with `total_entries: 0` and `has_entries: false`.
    /// Preserves existing selector failure semantics unchanged
    /// (`SessionNotFound` / `AmbiguousLabel` / `MalformedSelector`).
    pub fn has_entries_transcript(
        &self,
        selector: &str,
    ) -> Result<SessionTranscriptHasEntries, RuntimeError> {
        let resolved_id = self.resolve_selector(selector)?;
        let transcript = self.load_transcript(&resolved_id)?;
        let total_entries = transcript.entries.len();
        Ok(SessionTranscriptHasEntries {
            selector: selector.to_string(),
            resolved_session_id: transcript.session_id,
            created_at_ms: transcript.created_at_ms,
            updated_at_ms: transcript.updated_at_ms,
            total_entries,
            has_entries: total_entries > 0,
        })
    }

    /// Resolve a label to a single persisted `session_id`. Errors map cleanly:
    /// no match → `SessionNotFound("label:<name>")`, more than one match →
    /// `AmbiguousLabel`. Sessions without a label (older or never-renamed) are
    /// transparently skipped so mixed labeled/unlabeled stores keep working.
    pub fn resolve_label(&self, label: &str) -> Result<String, RuntimeError> {
        let trimmed = label.trim();
        if trimmed.is_empty() {
            return Err(RuntimeError::MalformedSelector(format!(
                "label selector requires a non-empty label after `{LABEL_SELECTOR_PREFIX}`"
            )));
        }

        let mut matched_ids: Vec<String> = Vec::new();
        for listing in self.list()? {
            let id = listing.session_id.to_string();
            let session = match self.load(&id) {
                Ok(state) => state,
                Err(RuntimeError::SessionNotFound(_)) => continue,
                Err(other) => return Err(other),
            };
            if session.label.as_deref() == Some(trimmed) {
                matched_ids.push(id);
            }
        }

        match matched_ids.len() {
            0 => Err(RuntimeError::SessionNotFound(format!(
                "{LABEL_SELECTOR_PREFIX}{trimmed}"
            ))),
            1 => Ok(matched_ids.remove(0)),
            n => Err(RuntimeError::AmbiguousLabel(format!(
                "label {trimmed:?} matches {n} persisted sessions"
            ))),
        }
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

    /// Preserve the newest `keep` prune-eligible (unpinned) persisted sessions
    /// using the same newest-first ordering as `list()` and remove every older
    /// unpinned persisted session together with its sibling transcript JSON.
    /// Pinned sessions are always preserved and reported under
    /// `pinned_preserved` / `pinned_preserved_count` regardless of `keep`.
    /// Preserved sessions are never mutated — their label, pinned flag,
    /// transcript entries, and activity metadata are untouched. If the store
    /// already contains `<= keep` unpinned sessions the call succeeds cleanly
    /// with an empty `removed` list. `keep == 0` is supported and prunes every
    /// unpinned persisted session.
    pub fn prune(&self, keep: usize) -> Result<SessionPrune, RuntimeError> {
        let listings = self.list()?;

        let mut pinned_preserved: Vec<SessionId> = Vec::new();
        let mut eligible: Vec<SessionListing> = Vec::new();
        for listing in listings {
            if listing.pinned {
                pinned_preserved.push(listing.session_id.clone());
            } else {
                eligible.push(listing);
            }
        }

        let kept_count = eligible.len().min(keep);

        let mut removed = Vec::new();
        for listing in eligible.into_iter().skip(kept_count) {
            let id = listing.session_id.to_string();
            let session_path = self.root.join(format!("{id}.json"));
            let transcript_path = self.transcript_path(&id);

            fs::remove_file(&session_path).map_err(|err| RuntimeError::Io(err.to_string()))?;
            if transcript_path.exists() {
                fs::remove_file(&transcript_path)
                    .map_err(|err| RuntimeError::Io(err.to_string()))?;
            }

            removed.push(SessionPruneRemoval {
                session_id: listing.session_id,
                session_path: session_path.display().to_string(),
                transcript_path: transcript_path.display().to_string(),
            });
        }

        Ok(SessionPrune {
            kept_count,
            pruned_count: removed.len(),
            pinned_preserved_count: pinned_preserved.len(),
            removed,
            pinned_preserved,
        })
    }

    /// Mark a persisted session as pinned without touching its `session_id`,
    /// `updated_at_ms`, label, messages, or transcript entries. Pinned sessions
    /// are preserved by `prune` regardless of the retention budget. Fails
    /// cleanly when the session is already pinned so the operation never
    /// silently no-ops.
    pub fn pin(&self, session_id: &str) -> Result<SessionPin, RuntimeError> {
        let mut session = self.load(session_id)?;
        if session.pinned {
            return Err(RuntimeError::SessionAlreadyPinned(
                session.session_id.to_string(),
            ));
        }
        session.pinned = true;
        self.save(&session)?;

        Ok(SessionPin {
            pinned_session_id: session.session_id,
            pinned: true,
        })
    }

    /// Clear the pinned flag on a persisted session without touching its
    /// `session_id`, `updated_at_ms`, label, messages, or transcript entries.
    /// Fails cleanly when the session is not currently pinned so the operation
    /// never silently no-ops. Keeps persisted JSON free of an explicit
    /// `pinned: false` field so older sessions stay byte-compatible.
    pub fn unpin(&self, session_id: &str) -> Result<SessionUnpin, RuntimeError> {
        let mut session = self.load(session_id)?;
        if !session.pinned {
            return Err(RuntimeError::SessionAlreadyUnpinned(
                session.session_id.to_string(),
            ));
        }
        session.pinned = false;
        self.save(&session)?;

        Ok(SessionUnpin {
            unpinned_session_id: session.session_id,
            pinned: false,
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

    pub fn fork(
        &self,
        source_session_id: &str,
        prompt: Prompt,
    ) -> Result<SessionFork, RuntimeError> {
        let source_session = self.load(source_session_id)?;
        let source_transcript = self.load_transcript(source_session_id)?;

        let forked_session_id = SessionId::new();
        let now = current_timestamp_ms();
        let appended_turn_index = TurnIndex(source_session.messages.len());

        let mut forked_messages = source_session.messages.clone();
        forked_messages.push(prompt.clone());

        let forked_session = SessionState {
            session_id: forked_session_id.clone(),
            created_at_ms: now,
            updated_at_ms: now,
            messages: forked_messages,
            usage: source_session.usage.add_turn(prompt.as_str(), "turn forked"),
            label: None,
            pinned: false,
        };

        let mut forked_entries = source_transcript.entries.clone();
        forked_entries.push(TranscriptEntry {
            turn_index: appended_turn_index,
            prompt,
        });

        let forked_transcript = TranscriptRecord {
            session_id: forked_session_id.clone(),
            created_at_ms: now,
            updated_at_ms: now,
            entries: forked_entries,
        };

        let session_path = self.save(&forked_session)?;
        let transcript_path = self.save_transcript(&forked_transcript)?;

        Ok(SessionFork {
            source_session_id: source_session.session_id,
            forked_session_id,
            appended_turn_index,
            session_path: session_path.display().to_string(),
            transcript_path: transcript_path.display().to_string(),
        })
    }

    pub fn rename(
        &self,
        session_id: &str,
        label: &str,
    ) -> Result<SessionRename, RuntimeError> {
        let applied_label = normalize_label(label)?;

        let mut session = self.load(session_id)?;
        session.label = Some(applied_label.clone());
        self.save(&session)?;

        Ok(SessionRename {
            renamed_session_id: session.session_id,
            applied_label,
        })
    }

    /// Remove the persisted `label` from a session without touching its
    /// `session_id`, transcript entries, or `updated_at_ms`. Fails cleanly
    /// when the session exists but carries no label so callers can
    /// distinguish this from "unknown id" without silently no-oping.
    pub fn unlabel(&self, session_id: &str) -> Result<SessionUnlabel, RuntimeError> {
        let mut session = self.load(session_id)?;
        let removed_label = session.label.take().ok_or_else(|| {
            RuntimeError::SessionAlreadyUnlabeled(session.session_id.to_string())
        })?;
        self.save(&session)?;

        Ok(SessionUnlabel {
            unlabeled_session_id: session.session_id,
            removed_label,
        })
    }

    /// Replace the persisted `label` on a session that already carries one.
    /// Preserves `session_id`, transcript entries/ordering, and `updated_at_ms`
    /// so newest-first ordering stays activity-based. Fails with
    /// `SessionAlreadyUnlabeled` when the session has no label to replace, and
    /// with `SessionAlreadyLabeled` when the requested label normalizes to the
    /// same effective value already persisted.
    pub fn retag(&self, session_id: &str, label: &str) -> Result<SessionRetag, RuntimeError> {
        let applied_label = normalize_label(label)?;

        let mut session = self.load(session_id)?;
        let previous_label = session
            .label
            .clone()
            .ok_or_else(|| RuntimeError::SessionAlreadyUnlabeled(session.session_id.to_string()))?;
        if previous_label == applied_label {
            return Err(RuntimeError::SessionAlreadyLabeled(format!(
                "{} already carries label {:?}",
                session.session_id, applied_label
            )));
        }
        session.label = Some(applied_label.clone());
        self.save(&session)?;

        Ok(SessionRetag {
            retagged_session_id: session.session_id,
            previous_label,
            applied_label,
        })
    }

    pub fn find(&self, query: &str) -> Result<Vec<SessionFindResult>, RuntimeError> {
        if query.is_empty() {
            return Ok(Vec::new());
        }

        let needle = query.to_ascii_lowercase();
        let mut results = Vec::new();
        for listing in self.list()? {
            let id = listing.session_id.to_string();
            let transcript = match self.load_transcript(&id) {
                Ok(record) => record,
                Err(RuntimeError::SessionNotFound(_)) => continue,
                Err(other) => return Err(other),
            };

            let matches: Vec<SessionFindMatch> = transcript
                .entries
                .iter()
                .filter(|entry| entry.prompt.0.to_ascii_lowercase().contains(&needle))
                .map(|entry| SessionFindMatch {
                    turn_index: entry.turn_index,
                    prompt: entry.prompt.clone(),
                })
                .collect();

            if matches.is_empty() {
                continue;
            }

            results.push(SessionFindResult {
                session_id: listing.session_id,
                created_at_ms: listing.created_at_ms,
                updated_at_ms: listing.updated_at_ms,
                message_count: listing.message_count,
                persisted_path: listing.persisted_path,
                matches,
                pinned: listing.pinned,
            });
        }

        Ok(results)
    }

    /// Enumerate labeled persisted sessions. Order mirrors `list()` (newest
    /// first by `updated_at_ms`, then `created_at_ms`, then `session_id`, then
    /// `persisted_path`). Sessions without a label are omitted; duplicate
    /// labels remain as separate rows so ambiguity is discoverable before a
    /// `label:<name>` selector would fail. Never mutates persisted state.
    pub fn list_labels(&self) -> Result<Vec<SessionLabelEntry>, RuntimeError> {
        let mut entries = Vec::new();
        for listing in self.list()? {
            let id = listing.session_id.to_string();
            let session = match self.load(&id) {
                Ok(state) => state,
                Err(RuntimeError::SessionNotFound(_)) => continue,
                Err(other) => return Err(other),
            };
            let Some(label) = session.label.clone() else {
                continue;
            };
            entries.push(SessionLabelEntry {
                label,
                session_id: listing.session_id,
                created_at_ms: listing.created_at_ms,
                updated_at_ms: listing.updated_at_ms,
                message_count: listing.message_count,
                persisted_path: listing.persisted_path,
                pinned: session.pinned,
            });
        }
        Ok(entries)
    }

    /// Enumerate pinned persisted sessions. Order mirrors `list()` (newest
    /// first by `updated_at_ms`, then `created_at_ms`, then `session_id`, then
    /// `persisted_path`). Unpinned sessions are omitted. Surfaces the `label`
    /// when present so the listing is useful from the terminal without a
    /// follow-up `session-show`. Never mutates persisted state.
    pub fn list_pins(&self) -> Result<Vec<SessionPinEntry>, RuntimeError> {
        let mut entries = Vec::new();
        for listing in self.list()? {
            if !listing.pinned {
                continue;
            }
            let id = listing.session_id.to_string();
            let session = match self.load(&id) {
                Ok(state) => state,
                Err(RuntimeError::SessionNotFound(_)) => continue,
                Err(other) => return Err(other),
            };
            entries.push(SessionPinEntry {
                session_id: listing.session_id,
                created_at_ms: listing.created_at_ms,
                updated_at_ms: listing.updated_at_ms,
                message_count: listing.message_count,
                persisted_path: listing.persisted_path,
                pinned: true,
                label: session.label.clone(),
            });
        }
        Ok(entries)
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
                pinned: session.pinned,
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
        SessionComparison, SessionComparisonSide, SessionExport, SessionFindResult, SessionState,
        SessionStore, TranscriptEntry, TranscriptRecord, TranscriptStore,
    };
    use super::normalize_label;
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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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
    fn find_returns_newest_first_results_with_matched_turn_indexes_and_prompts() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let older = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_000,
            updated_at_ms: 1_700_000_000_000,
            messages: vec![Prompt::new("review bash"), Prompt::new("Review summary")],
            usage: harness_core::UsageSummary {
                input_tokens: 4,
                output_tokens: 4,
            },
            label: None,
            pinned: false,
        };
        let mut older_transcript = TranscriptStore::default();
        older_transcript.append(Prompt::new("review bash"));
        older_transcript.append(Prompt::new("Review summary"));
        let older_record = TranscriptRecord::from_session(&older, &older_transcript);

        let newer = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_100,
            updated_at_ms: 1_700_000_000_100,
            messages: vec![Prompt::new("review tools")],
            usage: harness_core::UsageSummary {
                input_tokens: 2,
                output_tokens: 2,
            },
            label: None,
            pinned: false,
        };
        let mut newer_transcript = TranscriptStore::default();
        newer_transcript.append(Prompt::new("review tools"));
        let newer_record = TranscriptRecord::from_session(&newer, &newer_transcript);

        let unrelated = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_200,
            updated_at_ms: 1_700_000_000_200,
            messages: vec![Prompt::new("summary please")],
            usage: harness_core::UsageSummary {
                input_tokens: 2,
                output_tokens: 2,
            },
            label: None,
            pinned: false,
        };
        let mut unrelated_transcript = TranscriptStore::default();
        unrelated_transcript.append(Prompt::new("summary please"));
        let unrelated_record = TranscriptRecord::from_session(&unrelated, &unrelated_transcript);

        store.save(&older).expect("save older");
        store.save_transcript(&older_record).expect("save older transcript");
        store.save(&newer).expect("save newer");
        store.save_transcript(&newer_record).expect("save newer transcript");
        store.save(&unrelated).expect("save unrelated");
        store
            .save_transcript(&unrelated_record)
            .expect("save unrelated transcript");

        let results: Vec<SessionFindResult> = store.find("review").expect("find sessions");

        let ids: Vec<String> = results
            .iter()
            .map(|result| result.session_id.to_string())
            .collect();
        assert_eq!(
            ids,
            vec![newer.session_id.to_string(), older.session_id.to_string()],
            "find results must use newest-first session ordering"
        );

        let newer_result = &results[0];
        assert_eq!(newer_result.message_count, 1);
        assert_eq!(newer_result.matches.len(), 1);
        assert_eq!(newer_result.matches[0].turn_index.0, 0);
        assert_eq!(newer_result.matches[0].prompt.0, "review tools");

        let older_result = &results[1];
        assert_eq!(older_result.message_count, 2);
        let older_matches: Vec<(usize, String)> = older_result
            .matches
            .iter()
            .map(|m| (m.turn_index.0, m.prompt.0.clone()))
            .collect();
        assert_eq!(
            older_matches,
            vec![
                (0, "review bash".to_string()),
                (1, "Review summary".to_string()),
            ],
            "matches must preserve turn_index order and capture case-insensitive matches"
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn find_returns_empty_results_for_unmatched_query_and_for_empty_query() {
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
            label: None,
            pinned: false,
        };
        let mut transcript = TranscriptStore::default();
        transcript.append(Prompt::new("review bash"));
        let record = TranscriptRecord::from_session(&session, &transcript);
        store.save(&session).expect("save session");
        store.save_transcript(&record).expect("save transcript");

        let no_match = store.find("nothing-here").expect("find with no matches");
        assert!(
            no_match.is_empty(),
            "unmatched query must return an empty result set"
        );

        let empty_query = store.find("").expect("find with empty query");
        assert!(
            empty_query.is_empty(),
            "empty query must return an empty result set without crashing"
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn fork_creates_new_session_id_carries_transcript_and_appends_divergent_turn() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let source = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_001,
            updated_at_ms: 1_700_000_000_050,
            messages: vec![Prompt::new("review bash"), Prompt::new("summary please")],
            usage: harness_core::UsageSummary {
                input_tokens: 4,
                output_tokens: 4,
            },
            label: None,
            pinned: false,
        };
        let mut source_transcript = TranscriptStore::default();
        source_transcript.append(Prompt::new("review bash"));
        source_transcript.append(Prompt::new("summary please"));
        let source_record = TranscriptRecord::from_session(&source, &source_transcript);

        store.save(&source).expect("save source session");
        store
            .save_transcript(&source_record)
            .expect("save source transcript");

        let fork = store
            .fork(&source.session_id.to_string(), Prompt::new("try again"))
            .expect("fork source session");

        assert_eq!(fork.source_session_id, source.session_id);
        assert_ne!(
            fork.forked_session_id, source.session_id,
            "forked session must use a fresh session id"
        );
        assert_eq!(fork.appended_turn_index, TurnIndex(2));
        assert_eq!(
            fork.session_path,
            root.join(format!("{}.json", fork.forked_session_id))
                .display()
                .to_string()
        );
        assert_eq!(
            fork.transcript_path,
            root.join(format!("{}.transcript.json", fork.forked_session_id))
                .display()
                .to_string()
        );

        let forked_session = store
            .load(&fork.forked_session_id.to_string())
            .expect("load forked session");
        assert_eq!(
            forked_session
                .messages
                .iter()
                .map(|prompt| prompt.0.clone())
                .collect::<Vec<_>>(),
            vec![
                "review bash".to_string(),
                "summary please".to_string(),
                "try again".to_string(),
            ],
            "forked session must carry source messages then append the new prompt"
        );

        let forked_transcript = store
            .load_transcript(&fork.forked_session_id.to_string())
            .expect("load forked transcript");
        let ordered: Vec<(usize, String)> = forked_transcript
            .entries
            .iter()
            .map(|entry| (entry.turn_index.0, entry.prompt.0.clone()))
            .collect();
        assert_eq!(
            ordered,
            vec![
                (0, "review bash".to_string()),
                (1, "summary please".to_string()),
                (2, "try again".to_string()),
            ],
            "forked transcript must preserve turn ordering and append the new prompt"
        );
        assert_eq!(forked_transcript.session_id, fork.forked_session_id);

        let reloaded_source = store
            .load(&source.session_id.to_string())
            .expect("source session must survive");
        assert_eq!(reloaded_source, source, "source session must not be mutated");
        let reloaded_source_transcript = store
            .load_transcript(&source.session_id.to_string())
            .expect("source transcript must survive");
        assert_eq!(
            reloaded_source_transcript, source_record,
            "source transcript must not be mutated"
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn fork_errors_cleanly_when_source_session_is_missing() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let missing = SessionId::new().to_string();
        match store.fork(&missing, Prompt::new("forked prompt")) {
            Err(RuntimeError::SessionNotFound(reported)) => assert_eq!(reported, missing),
            other => panic!("expected SessionNotFound for missing source id, got {other:?}"),
        }

        assert!(
            !root.exists() || fs::read_dir(&root).map(|mut iter| iter.next().is_none()).unwrap_or(true),
            "no persisted artifacts should be written when fork source is missing"
        );

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
            label: None,
            pinned: false,
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
            label: None,
            pinned: false,
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

    #[test]
    fn rename_stores_label_preserves_id_transcript_and_activity_metadata() {
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
            label: None,
            pinned: false,
        };
        let mut transcript = TranscriptStore::default();
        transcript.append(Prompt::new("review bash"));
        transcript.append(Prompt::new("summary please"));
        let record = TranscriptRecord::from_session(&session, &transcript);

        store.save(&session).expect("save session");
        store.save_transcript(&record).expect("save transcript");

        let id = session.session_id.to_string();
        let renamed = store
            .rename(&id, "  runtime-review  ")
            .expect("rename persisted session");

        assert_eq!(renamed.renamed_session_id, session.session_id);
        assert_eq!(renamed.applied_label, "runtime-review");

        let reloaded = store.load(&id).expect("reload renamed session");
        assert_eq!(reloaded.session_id, session.session_id);
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));
        assert_eq!(
            reloaded.created_at_ms, session.created_at_ms,
            "rename must preserve created_at_ms"
        );
        assert_eq!(
            reloaded.updated_at_ms, session.updated_at_ms,
            "rename must preserve updated_at_ms so newest-first ordering stays activity-based"
        );
        assert_eq!(reloaded.messages, session.messages);
        assert_eq!(reloaded.usage, session.usage);

        let reloaded_transcript = store
            .load_transcript(&id)
            .expect("reload transcript after rename");
        assert_eq!(
            reloaded_transcript, record,
            "rename must not mutate transcript entries or ordering"
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn rename_rejects_empty_and_whitespace_only_labels_without_touching_store() {
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
            label: None,
            pinned: false,
        };
        store.save(&session).expect("save session");
        let id = session.session_id.to_string();

        for invalid in ["", "   ", "\t\n "] {
            match store.rename(&id, invalid) {
                Err(RuntimeError::InvalidLabel(_)) => {}
                other => panic!("expected InvalidLabel for {invalid:?}, got {other:?}"),
            }
        }

        let reloaded = store.load(&id).expect("reload session after rejected rename");
        assert!(
            reloaded.label.is_none(),
            "rejected rename must not persist a label on the session"
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn rename_missing_session_reports_session_not_found() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let missing = SessionId::new().to_string();
        match store.rename(&missing, "anything") {
            Err(RuntimeError::SessionNotFound(reported)) => assert_eq!(reported, missing),
            other => panic!("expected SessionNotFound for missing id, got {other:?}"),
        }

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn unlabel_clears_label_preserves_id_transcript_and_activity_metadata() {
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
            label: Some("runtime-review".to_string()),
            pinned: false,
        };
        let mut transcript = TranscriptStore::default();
        transcript.append(Prompt::new("review bash"));
        transcript.append(Prompt::new("summary please"));
        let record = TranscriptRecord::from_session(&session, &transcript);

        store.save(&session).expect("save session");
        store.save_transcript(&record).expect("save transcript");

        let id = session.session_id.to_string();
        let unlabeled = store.unlabel(&id).expect("unlabel persisted session");

        assert_eq!(unlabeled.unlabeled_session_id, session.session_id);
        assert_eq!(unlabeled.removed_label, "runtime-review");

        let reloaded = store.load(&id).expect("reload unlabeled session");
        assert_eq!(reloaded.session_id, session.session_id);
        assert!(
            reloaded.label.is_none(),
            "unlabel must clear the persisted label"
        );
        assert_eq!(
            reloaded.created_at_ms, session.created_at_ms,
            "unlabel must preserve created_at_ms"
        );
        assert_eq!(
            reloaded.updated_at_ms, session.updated_at_ms,
            "unlabel must preserve updated_at_ms so newest-first ordering stays activity-based"
        );
        assert_eq!(reloaded.messages, session.messages);
        assert_eq!(reloaded.usage, session.usage);

        let reloaded_transcript = store
            .load_transcript(&id)
            .expect("reload transcript after unlabel");
        assert_eq!(
            reloaded_transcript, record,
            "unlabel must not mutate transcript entries or ordering"
        );

        let persisted_body =
            fs::read_to_string(root.join(format!("{id}.json"))).expect("read session json");
        assert!(
            !persisted_body.contains("\"label\""),
            "persisted session json must not serialize a null/empty label field: {persisted_body}"
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn unlabel_already_unlabeled_session_fails_without_touching_store() {
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
            label: None,
            pinned: false,
        };
        store.save(&session).expect("save session");
        let id = session.session_id.to_string();

        match store.unlabel(&id) {
            Err(RuntimeError::SessionAlreadyUnlabeled(reported)) => {
                assert_eq!(reported, id, "error must surface the resolved session id");
            }
            other => panic!("expected SessionAlreadyUnlabeled, got {other:?}"),
        }

        let reloaded = store
            .load(&id)
            .expect("reload session after failed unlabel");
        assert!(
            reloaded.label.is_none(),
            "failed unlabel must leave the session unlabeled"
        );
        assert_eq!(
            reloaded.updated_at_ms, session.updated_at_ms,
            "failed unlabel must not bump activity metadata"
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn retag_replaces_label_preserves_id_transcript_and_activity_metadata() {
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
            label: Some("runtime-review".to_string()),
            pinned: false,
        };
        let mut transcript = TranscriptStore::default();
        transcript.append(Prompt::new("review bash"));
        transcript.append(Prompt::new("summary please"));
        let record = TranscriptRecord::from_session(&session, &transcript);

        store.save(&session).expect("save session");
        store.save_transcript(&record).expect("save transcript");

        let id = session.session_id.to_string();
        let retagged = store
            .retag(&id, "  release-candidate  ")
            .expect("retag persisted session");

        assert_eq!(retagged.retagged_session_id, session.session_id);
        assert_eq!(retagged.previous_label, "runtime-review");
        assert_eq!(retagged.applied_label, "release-candidate");

        let reloaded = store.load(&id).expect("reload retagged session");
        assert_eq!(reloaded.label.as_deref(), Some("release-candidate"));
        assert_eq!(reloaded.created_at_ms, session.created_at_ms);
        assert_eq!(
            reloaded.updated_at_ms, session.updated_at_ms,
            "retag must preserve updated_at_ms so newest-first ordering stays activity-based"
        );
        assert_eq!(reloaded.messages, session.messages);
        assert_eq!(reloaded.usage, session.usage);

        let reloaded_transcript = store
            .load_transcript(&id)
            .expect("reload transcript after retag");
        assert_eq!(
            reloaded_transcript, record,
            "retag must not mutate transcript entries or ordering"
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn retag_rejects_same_effective_label_and_unlabeled_and_invalid_inputs_without_touching_store() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let labeled = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_001,
            updated_at_ms: 1_700_000_000_050,
            messages: vec![Prompt::new("review bash")],
            usage: harness_core::UsageSummary {
                input_tokens: 2,
                output_tokens: 2,
            },
            label: Some("runtime-review".to_string()),
            pinned: false,
        };
        store.save(&labeled).expect("save labeled session");
        let labeled_id = labeled.session_id.to_string();

        // Same effective label (including surrounding whitespace that normalizes away).
        match store.retag(&labeled_id, "  runtime-review  ") {
            Err(RuntimeError::SessionAlreadyLabeled(reported)) => {
                assert!(
                    reported.contains(&labeled_id),
                    "SessionAlreadyLabeled must surface the resolved session id: {reported}"
                );
            }
            other => panic!("expected SessionAlreadyLabeled, got {other:?}"),
        }

        // Empty/whitespace-only labels surface InvalidLabel.
        for invalid in ["", "   ", "\t\n"] {
            match store.retag(&labeled_id, invalid) {
                Err(RuntimeError::InvalidLabel(_)) => {}
                other => panic!("expected InvalidLabel for {invalid:?}, got {other:?}"),
            }
        }

        // Retagging an unlabeled session surfaces SessionAlreadyUnlabeled.
        let unlabeled = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_002,
            updated_at_ms: 1_700_000_000_060,
            messages: vec![Prompt::new("review bash")],
            usage: harness_core::UsageSummary {
                input_tokens: 2,
                output_tokens: 2,
            },
            label: None,
            pinned: false,
        };
        store.save(&unlabeled).expect("save unlabeled session");
        let unlabeled_id = unlabeled.session_id.to_string();
        match store.retag(&unlabeled_id, "first-label") {
            Err(RuntimeError::SessionAlreadyUnlabeled(reported)) => {
                assert_eq!(
                    reported, unlabeled_id,
                    "SessionAlreadyUnlabeled must surface the resolved session id"
                );
            }
            other => panic!("expected SessionAlreadyUnlabeled, got {other:?}"),
        }

        // Unknown session id surfaces SessionNotFound.
        let missing = SessionId::new().to_string();
        match store.retag(&missing, "anything") {
            Err(RuntimeError::SessionNotFound(reported)) => assert_eq!(reported, missing),
            other => panic!("expected SessionNotFound for missing id, got {other:?}"),
        }

        // All failing attempts must leave persisted state untouched.
        let reloaded_labeled = store.load(&labeled_id).expect("reload labeled after failures");
        assert_eq!(reloaded_labeled.label.as_deref(), Some("runtime-review"));
        assert_eq!(reloaded_labeled.updated_at_ms, labeled.updated_at_ms);
        let reloaded_unlabeled = store
            .load(&unlabeled_id)
            .expect("reload unlabeled after failures");
        assert!(reloaded_unlabeled.label.is_none());

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn unlabel_missing_session_reports_session_not_found() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let missing = SessionId::new().to_string();
        match store.unlabel(&missing) {
            Err(RuntimeError::SessionNotFound(reported)) => assert_eq!(reported, missing),
            other => panic!("expected SessionNotFound for missing id, got {other:?}"),
        }

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn unlabeled_session_json_roundtrips_through_labeled_deserializer_for_backward_compat() {
        let legacy = r#"{
            "session_id": "11111111-1111-1111-1111-111111111111",
            "created_at_ms": 1700000000001,
            "updated_at_ms": 1700000000050,
            "messages": ["review bash"],
            "usage": { "input_tokens": 2, "output_tokens": 2 }
        }"#;
        let parsed: SessionState =
            serde_json::from_str(legacy).expect("legacy unlabeled session JSON must still parse");
        assert!(
            parsed.label.is_none(),
            "missing label field must deserialize to None"
        );

        let reserialized = serde_json::to_string(&parsed).expect("reserialize unlabeled session");
        assert!(
            !reserialized.contains("\"label\""),
            "unlabeled session must not emit a label field: {reserialized}"
        );
    }

    #[test]
    fn resolve_selector_dispatches_latest_label_and_raw_id_against_persisted_state() {
        use super::SessionSelector;

        let root = temp_session_root();
        let store = SessionStore::new(&root);

        // Two sessions: older labeled, newer unlabeled, so newest-first ordering
        // is independent of the label match.
        let labeled = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_000,
            updated_at_ms: 1_700_000_000_000,
            messages: vec![Prompt::new("review bash")],
            usage: harness_core::UsageSummary::default(),
            label: Some("runtime-review".to_string()),
            pinned: false,
        };
        let unlabeled = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_500,
            updated_at_ms: 1_700_000_000_500,
            messages: vec![Prompt::new("summary")],
            usage: harness_core::UsageSummary::default(),
            label: None,
            pinned: false,
        };
        store.save(&labeled).expect("save labeled");
        store.save(&unlabeled).expect("save unlabeled");

        // Raw id round-trips verbatim with no disk I/O assumptions.
        let raw_id = labeled.session_id.to_string();
        assert_eq!(store.resolve_selector(&raw_id).unwrap(), raw_id);

        // `latest` follows updated_at_ms, not label.
        assert_eq!(
            store.resolve_selector("latest").unwrap(),
            unlabeled.session_id.to_string()
        );

        // Label resolves to the labeled session even though it is older.
        assert_eq!(
            store.resolve_selector("label:runtime-review").unwrap(),
            labeled.session_id.to_string()
        );

        // Selector parsing covers raw + latest + label forms.
        assert_eq!(
            SessionSelector::parse("latest").unwrap(),
            SessionSelector::Latest
        );
        assert_eq!(
            SessionSelector::parse("label:foo").unwrap(),
            SessionSelector::Label("foo".to_string())
        );
        assert_eq!(
            SessionSelector::parse("abc-123").unwrap(),
            SessionSelector::Id("abc-123".to_string())
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn resolve_label_reports_unknown_ambiguous_and_malformed_selectors_cleanly() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let first = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_000,
            updated_at_ms: 1_700_000_000_000,
            messages: vec![Prompt::new("a")],
            usage: harness_core::UsageSummary::default(),
            label: Some("dup".to_string()),
            pinned: false,
        };
        let second = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_100,
            updated_at_ms: 1_700_000_000_100,
            messages: vec![Prompt::new("b")],
            usage: harness_core::UsageSummary::default(),
            label: Some("dup".to_string()),
            pinned: false,
        };
        store.save(&first).expect("save first");
        store.save(&second).expect("save second");

        // Unknown label surfaces as SessionNotFound with the `label:` prefix
        // preserved so the CLI error mentions the selector the user typed.
        match store.resolve_selector("label:missing") {
            Err(RuntimeError::SessionNotFound(reported)) => {
                assert_eq!(reported, "label:missing");
            }
            other => panic!("expected SessionNotFound for unknown label, got {other:?}"),
        }

        // Two sessions carrying the same label is an ambiguity error, not a
        // silent newest-wins.
        match store.resolve_selector("label:dup") {
            Err(RuntimeError::AmbiguousLabel(message)) => {
                assert!(message.contains("\"dup\""), "message should quote label: {message}");
                assert!(message.contains('2'), "message should mention match count: {message}");
            }
            other => panic!("expected AmbiguousLabel, got {other:?}"),
        }

        // `label:` with no name (or only whitespace) is a malformed selector,
        // distinct from "unknown label".
        for malformed in ["label:", "label:   "] {
            match store.resolve_selector(malformed) {
                Err(RuntimeError::MalformedSelector(_)) => {}
                other => panic!("expected MalformedSelector for {malformed:?}, got {other:?}"),
            }
        }

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn list_labels_orders_newest_first_omits_unlabeled_and_keeps_duplicates_as_separate_rows() {
        use super::SessionLabelEntry;

        let root = temp_session_root();
        let store = SessionStore::new(&root);

        // Three sessions with varied activity. Ordering is driven by
        // updated_at_ms → created_at_ms → session_id → persisted_path, same as
        // SessionStore::list.
        let older_labeled = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_001,
            updated_at_ms: 1_700_000_000_010,
            messages: vec![Prompt::new("review bash")],
            usage: harness_core::UsageSummary::default(),
            label: Some("runtime-review".to_string()),
            pinned: false,
        };
        let newer_labeled = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_050,
            updated_at_ms: 1_700_000_000_200,
            messages: vec![Prompt::new("summary please")],
            usage: harness_core::UsageSummary::default(),
            label: Some("release-candidate".to_string()),
            pinned: false,
        };
        let unlabeled = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_100,
            updated_at_ms: 1_700_000_000_300,
            messages: vec![Prompt::new("no label here")],
            usage: harness_core::UsageSummary::default(),
            label: None,
            pinned: false,
        };

        store.save(&older_labeled).expect("save older labeled");
        store.save(&newer_labeled).expect("save newer labeled");
        store.save(&unlabeled).expect("save unlabeled");

        let entries: Vec<SessionLabelEntry> = store.list_labels().expect("list labels");

        // Unlabeled session must be absent, and the remaining labeled sessions
        // must appear in newest-first order (newer_labeled before older_labeled).
        let ids: Vec<String> = entries
            .iter()
            .map(|entry| entry.session_id.to_string())
            .collect();
        assert_eq!(
            ids,
            vec![
                newer_labeled.session_id.to_string(),
                older_labeled.session_id.to_string(),
            ],
            "list_labels must omit unlabeled sessions and use newest-first ordering"
        );

        assert_eq!(entries[0].label, "release-candidate");
        assert_eq!(entries[0].message_count, 1);
        assert_eq!(entries[0].updated_at_ms, 1_700_000_000_200);
        assert_eq!(
            entries[0].persisted_path,
            root.join(format!("{}.json", newer_labeled.session_id))
                .display()
                .to_string()
        );

        assert_eq!(entries[1].label, "runtime-review");
        assert_eq!(entries[1].message_count, 1);

        // Verify list_labels does not mutate persisted state — reloaded state
        // must match exactly what was saved (including the unlabeled session
        // staying unlabeled).
        assert_eq!(
            store
                .load(&unlabeled.session_id.to_string())
                .expect("reload unlabeled")
                .label,
            None,
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn list_labels_keeps_duplicate_labels_as_separate_rows_in_newest_first_order() {
        use super::SessionLabelEntry;

        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let earlier = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_001,
            updated_at_ms: 1_700_000_000_100,
            messages: vec![Prompt::new("alpha")],
            usage: harness_core::UsageSummary::default(),
            label: Some("dup".to_string()),
            pinned: false,
        };
        let later = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_002,
            updated_at_ms: 1_700_000_000_300,
            messages: vec![Prompt::new("beta")],
            usage: harness_core::UsageSummary::default(),
            label: Some("dup".to_string()),
            pinned: false,
        };

        store.save(&earlier).expect("save earlier");
        store.save(&later).expect("save later");

        let entries: Vec<SessionLabelEntry> = store.list_labels().expect("list labels");

        // Both rows must appear — ambiguity is discoverable, not collapsed.
        assert_eq!(entries.len(), 2, "duplicate labels must not be collapsed");
        assert_eq!(entries[0].label, "dup");
        assert_eq!(entries[1].label, "dup");
        assert_eq!(entries[0].session_id.to_string(), later.session_id.to_string());
        assert_eq!(entries[1].session_id.to_string(), earlier.session_id.to_string());

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn list_labels_returns_empty_vector_for_missing_root_and_for_unlabeled_store() {
        // Missing root directory should not error — the store must behave the
        // same as for `list()`: cleanly empty.
        let missing_root = temp_session_root();
        let empty_store = SessionStore::new(&missing_root);
        let empty = empty_store.list_labels().expect("list labels on missing root");
        assert!(empty.is_empty(), "missing root must yield empty label listing");

        // Store with only unlabeled sessions must also yield a clean empty
        // listing rather than including the unlabeled sessions.
        let root = temp_session_root();
        let store = SessionStore::new(&root);
        let unlabeled = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_001,
            updated_at_ms: 1_700_000_000_010,
            messages: vec![Prompt::new("no label")],
            usage: harness_core::UsageSummary::default(),
            label: None,
            pinned: false,
        };
        store.save(&unlabeled).expect("save unlabeled");
        let entries = store.list_labels().expect("list labels");
        assert!(
            entries.is_empty(),
            "unlabeled-only store must yield empty label listing"
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn list_pins_orders_newest_first_omits_unpinned_and_surfaces_optional_label() {
        use super::SessionPinEntry;

        let root = temp_session_root();
        let store = SessionStore::new(&root);

        // Three sessions: older pinned + labeled, newer pinned + unlabeled,
        // and a middle unpinned one that must be omitted from the pin listing.
        let older_pinned_labeled = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_001,
            updated_at_ms: 1_700_000_000_010,
            messages: vec![Prompt::new("review bash")],
            usage: harness_core::UsageSummary::default(),
            label: Some("runtime-review".to_string()),
            pinned: true,
        };
        let middle_unpinned = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_050,
            updated_at_ms: 1_700_000_000_100,
            messages: vec![Prompt::new("unpinned middle")],
            usage: harness_core::UsageSummary::default(),
            label: Some("scratch".to_string()),
            pinned: false,
        };
        let newer_pinned_unlabeled = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_090,
            updated_at_ms: 1_700_000_000_300,
            messages: vec![Prompt::new("summary please")],
            usage: harness_core::UsageSummary::default(),
            label: None,
            pinned: true,
        };

        store.save(&older_pinned_labeled).expect("save older pinned");
        store.save(&middle_unpinned).expect("save middle unpinned");
        store.save(&newer_pinned_unlabeled).expect("save newer pinned");

        let entries: Vec<SessionPinEntry> = store.list_pins().expect("list pins");

        let ids: Vec<String> = entries
            .iter()
            .map(|entry| entry.session_id.to_string())
            .collect();
        assert_eq!(
            ids,
            vec![
                newer_pinned_unlabeled.session_id.to_string(),
                older_pinned_labeled.session_id.to_string(),
            ],
            "list_pins must omit unpinned sessions and use newest-first ordering"
        );

        assert!(entries[0].pinned);
        assert_eq!(entries[0].label, None, "unlabeled pinned session must omit label");
        assert_eq!(entries[0].message_count, 1);
        assert_eq!(entries[0].updated_at_ms, 1_700_000_000_300);
        assert_eq!(
            entries[0].persisted_path,
            root.join(format!("{}.json", newer_pinned_unlabeled.session_id))
                .display()
                .to_string()
        );

        assert!(entries[1].pinned);
        assert_eq!(
            entries[1].label.as_deref(),
            Some("runtime-review"),
            "labeled pinned session must surface label"
        );

        // list_pins must not mutate persisted state — the unpinned session
        // stays exactly as saved.
        let reloaded = store
            .load(&middle_unpinned.session_id.to_string())
            .expect("reload unpinned");
        assert!(!reloaded.pinned);
        assert_eq!(reloaded.label.as_deref(), Some("scratch"));

        // Round-trip the serialized pinned entry to confirm the deterministic
        // shape and that `label` is skipped for unlabeled pinned rows.
        let serialized = serde_json::to_string(&entries[0]).expect("serialize pin entry");
        assert!(
            !serialized.contains("\"label\""),
            "unlabeled pinned entry must not serialize `label`: {serialized}"
        );
        let round_trip: SessionPinEntry =
            serde_json::from_str(&serialized).expect("round-trip pin entry");
        assert_eq!(round_trip, entries[0]);

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn list_pins_returns_empty_vector_for_missing_root_and_for_unpinned_store() {
        // Missing root directory should not error — cleanly empty, matching
        // the behaviour of `list()` / `list_labels()`.
        let missing_root = temp_session_root();
        let empty_store = SessionStore::new(&missing_root);
        let empty = empty_store.list_pins().expect("list pins on missing root");
        assert!(empty.is_empty(), "missing root must yield empty pin listing");

        // Store with only unpinned sessions must also yield a clean empty
        // listing rather than including unpinned sessions.
        let root = temp_session_root();
        let store = SessionStore::new(&root);
        let unpinned = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_001,
            updated_at_ms: 1_700_000_000_010,
            messages: vec![Prompt::new("no pin")],
            usage: harness_core::UsageSummary::default(),
            label: Some("keep".to_string()),
            pinned: false,
        };
        store.save(&unpinned).expect("save unpinned");
        let entries = store.list_pins().expect("list pins");
        assert!(
            entries.is_empty(),
            "unpinned-only store must yield empty pin listing"
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn normalize_label_trims_and_rejects_empty_whitespace() {
        assert_eq!(normalize_label("hello").unwrap(), "hello");
        assert_eq!(normalize_label("  runtime-review  ").unwrap(), "runtime-review");
        assert!(matches!(
            normalize_label(""),
            Err(RuntimeError::InvalidLabel(_))
        ));
        assert!(matches!(
            normalize_label("   \t\n"),
            Err(RuntimeError::InvalidLabel(_))
        ));
    }

    fn seed_session(store: &SessionStore, updated_at_ms: u64, label: Option<&str>) -> SessionState {
        let session = SessionState {
            session_id: SessionId::new(),
            created_at_ms: updated_at_ms,
            updated_at_ms,
            messages: vec![Prompt::new("seed prompt")],
            usage: harness_core::UsageSummary::default(),
            label: label.map(|value| value.to_string()),
            pinned: false,
        };
        store.save(&session).expect("save seeded session");

        let transcript = TranscriptRecord {
            session_id: session.session_id.clone(),
            created_at_ms: updated_at_ms,
            updated_at_ms,
            entries: vec![TranscriptEntry {
                turn_index: TurnIndex(0),
                prompt: Prompt::new("seed prompt"),
            }],
        };
        store
            .save_transcript(&transcript)
            .expect("save seeded transcript");

        session
    }

    #[test]
    fn prune_preserves_newest_n_and_removes_older_session_and_transcript_pairs() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let oldest = seed_session(&store, 1_700_000_000_000, Some("old-label"));
        let middle = seed_session(&store, 1_700_000_000_100, None);
        let newest = seed_session(&store, 1_700_000_000_200, Some("newest-label"));

        let outcome = store.prune(2).expect("prune keep 2");

        assert_eq!(outcome.kept_count, 2);
        assert_eq!(outcome.pruned_count, 1);
        assert_eq!(outcome.removed.len(), 1);
        assert_eq!(
            outcome.removed[0].session_id.to_string(),
            oldest.session_id.to_string(),
            "prune must target the oldest session"
        );
        assert_eq!(
            outcome.removed[0].session_path,
            root.join(format!("{}.json", oldest.session_id))
                .display()
                .to_string()
        );
        assert_eq!(
            outcome.removed[0].transcript_path,
            root.join(format!("{}.transcript.json", oldest.session_id))
                .display()
                .to_string()
        );

        // Oldest artifacts are gone.
        assert!(
            !root.join(format!("{}.json", oldest.session_id)).exists(),
            "pruned session JSON must be removed"
        );
        assert!(
            !root
                .join(format!("{}.transcript.json", oldest.session_id))
                .exists(),
            "pruned transcript JSON must be removed"
        );

        // Preserved sessions are untouched: identity, label, recency, transcript
        // contents, and `turn_index` ordering all match what was saved.
        let reloaded_middle = store
            .load(&middle.session_id.to_string())
            .expect("reload middle");
        let reloaded_newest = store
            .load(&newest.session_id.to_string())
            .expect("reload newest");
        assert_eq!(reloaded_middle, middle);
        assert_eq!(reloaded_newest, newest);

        let transcript_middle = store
            .load_transcript(&middle.session_id.to_string())
            .expect("reload middle transcript");
        assert_eq!(transcript_middle.entries.len(), 1);
        assert_eq!(transcript_middle.entries[0].turn_index.0, 0);

        // Listing ordering now contains only the preserved sessions in
        // newest-first order.
        let remaining: Vec<String> = store
            .list()
            .expect("list after prune")
            .into_iter()
            .map(|entry| entry.session_id.to_string())
            .collect();
        assert_eq!(
            remaining,
            vec![
                newest.session_id.to_string(),
                middle.session_id.to_string(),
            ]
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn prune_keep_zero_removes_every_persisted_session_deterministically() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let a = seed_session(&store, 1_700_000_000_000, None);
        let b = seed_session(&store, 1_700_000_000_100, Some("labelled"));
        let c = seed_session(&store, 1_700_000_000_200, None);

        let outcome = store.prune(0).expect("prune keep 0");
        assert_eq!(outcome.kept_count, 0);
        assert_eq!(outcome.pruned_count, 3);

        // Removal order must match newest-first ordering.
        let removed_ids: Vec<String> = outcome
            .removed
            .iter()
            .map(|entry| entry.session_id.to_string())
            .collect();
        assert_eq!(
            removed_ids,
            vec![
                c.session_id.to_string(),
                b.session_id.to_string(),
                a.session_id.to_string(),
            ]
        );

        // Each removed entry carries the deterministic session/transcript paths.
        for (removal, session) in outcome.removed.iter().zip([&c, &b, &a]) {
            assert_eq!(
                removal.session_path,
                root.join(format!("{}.json", session.session_id))
                    .display()
                    .to_string()
            );
            assert_eq!(
                removal.transcript_path,
                root.join(format!("{}.transcript.json", session.session_id))
                    .display()
                    .to_string()
            );
        }

        assert!(store.list().expect("list after full prune").is_empty());

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn prune_keep_equal_or_greater_than_total_is_noop_with_empty_removed() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let a = seed_session(&store, 1_700_000_000_000, None);
        let b = seed_session(&store, 1_700_000_000_100, None);

        let equal = store.prune(2).expect("prune keep == total");
        assert_eq!(equal.kept_count, 2);
        assert_eq!(equal.pruned_count, 0);
        assert!(equal.removed.is_empty(), "keep == total must not prune");

        let greater = store.prune(5).expect("prune keep > total");
        assert_eq!(greater.kept_count, 2);
        assert_eq!(greater.pruned_count, 0);
        assert!(greater.removed.is_empty(), "keep > total must not prune");

        // Sessions still on disk and still reloadable unchanged.
        assert_eq!(store.load(&a.session_id.to_string()).unwrap(), a);
        assert_eq!(store.load(&b.session_id.to_string()).unwrap(), b);

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn prune_on_empty_store_succeeds_with_zero_kept_and_empty_removed() {
        let missing_root = temp_session_root();
        let store = SessionStore::new(&missing_root);
        let outcome = store.prune(3).expect("prune empty store");
        assert_eq!(outcome.kept_count, 0);
        assert_eq!(outcome.pruned_count, 0);
        assert!(outcome.removed.is_empty());
    }

    #[test]
    fn pin_sets_flag_preserves_id_transcript_and_activity_metadata() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let seeded = seed_session(&store, 1_700_000_000_050, None);
        let original_transcript = store
            .load_transcript(&seeded.session_id.to_string())
            .expect("load transcript");

        let outcome = store
            .pin(&seeded.session_id.to_string())
            .expect("pin session");
        assert_eq!(outcome.pinned_session_id, seeded.session_id);
        assert!(outcome.pinned, "pin result must report pinned: true");

        let reloaded = store
            .load(&seeded.session_id.to_string())
            .expect("reload pinned session");
        assert!(reloaded.pinned, "pinned flag must persist across reload");
        // Identity and activity metadata are untouched.
        assert_eq!(reloaded.session_id, seeded.session_id);
        assert_eq!(reloaded.created_at_ms, seeded.created_at_ms);
        assert_eq!(reloaded.updated_at_ms, seeded.updated_at_ms);
        assert_eq!(reloaded.messages, seeded.messages);
        assert_eq!(reloaded.usage, seeded.usage);
        assert_eq!(reloaded.label, seeded.label);
        // Transcript bytes are byte-identical.
        let after_transcript = store
            .load_transcript(&seeded.session_id.to_string())
            .expect("reload transcript");
        assert_eq!(after_transcript, original_transcript);

        // Already-pinned sessions reject a second pin without mutation.
        let second = store.pin(&seeded.session_id.to_string());
        assert!(matches!(second, Err(RuntimeError::SessionAlreadyPinned(_))));

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn unpin_clears_flag_and_omits_field_from_persisted_json() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let seeded = seed_session(&store, 1_700_000_000_050, None);
        store
            .pin(&seeded.session_id.to_string())
            .expect("pin for unpin round-trip");

        let outcome = store
            .unpin(&seeded.session_id.to_string())
            .expect("unpin session");
        assert_eq!(outcome.unpinned_session_id, seeded.session_id);
        assert!(!outcome.pinned, "unpin result must report pinned: false");

        // Backward-compatible serialization: once pin is cleared, persisted JSON
        // no longer contains a `pinned` field at all (no null, no `false`).
        let persisted_path = root.join(format!("{}.json", seeded.session_id));
        let persisted_json =
            fs::read_to_string(&persisted_path).expect("read persisted session json");
        assert!(
            !persisted_json.contains("\"pinned\""),
            "unpinned persisted JSON must not carry a pinned field: {persisted_json}"
        );

        // Unpinning an already-unpinned session is a clean, descriptive error.
        let second = store.unpin(&seeded.session_id.to_string());
        assert!(matches!(
            second,
            Err(RuntimeError::SessionAlreadyUnpinned(_))
        ));

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn unpinned_session_state_serializes_without_pinned_field_for_backward_compat() {
        let session = SessionState {
            session_id: SessionId::new(),
            created_at_ms: 1_700_000_000_000,
            updated_at_ms: 1_700_000_000_000,
            messages: vec![Prompt::new("hi")],
            usage: harness_core::UsageSummary::default(),
            label: None,
            pinned: false,
        };
        let serialized = serde_json::to_string(&session).expect("serialize session");
        assert!(
            !serialized.contains("\"pinned\""),
            "unpinned default session must not serialize `pinned`: {serialized}"
        );

        // Deserialization round-trips cleanly from legacy JSON that has no pinned field.
        let legacy = serde_json::to_string(&serde_json::json!({
            "session_id": session.session_id,
            "created_at_ms": session.created_at_ms,
            "updated_at_ms": session.updated_at_ms,
            "messages": session.messages,
            "usage": session.usage,
        }))
        .expect("serialize legacy session");
        let reloaded: SessionState =
            serde_json::from_str(&legacy).expect("deserialize legacy session");
        assert!(!reloaded.pinned, "missing pinned field must default to false");
    }

    #[test]
    fn prune_skips_pinned_sessions_and_reports_pinned_preserved_deterministically() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        // Newest-first ordering: c > b > a.
        let a = seed_session(&store, 1_700_000_000_000, None);
        let b = seed_session(&store, 1_700_000_000_100, None);
        let c = seed_session(&store, 1_700_000_000_200, None);

        // Pin the *oldest* session. Without pinning, `keep=1` would preserve
        // only `c` and delete both `a` and `b`. Pinning `a` must rescue it.
        store
            .pin(&a.session_id.to_string())
            .expect("pin oldest session");

        let outcome = store.prune(1).expect("prune keep=1 with pin");

        // Unpinned sessions: newest-first ordering keeps `c` and prunes `b`.
        assert_eq!(outcome.kept_count, 1, "kept_count counts unpinned only");
        assert_eq!(outcome.pruned_count, 1);
        assert_eq!(outcome.removed.len(), 1);
        assert_eq!(outcome.removed[0].session_id, b.session_id);

        // Pinned preservation is surfaced deterministically.
        assert_eq!(outcome.pinned_preserved_count, 1);
        assert_eq!(outcome.pinned_preserved, vec![a.session_id.clone()]);

        // Pinned oldest session is still on disk, with transcript and pin flag
        // intact; newest unpinned session is preserved; middle one is gone.
        let reloaded_pinned = store
            .load(&a.session_id.to_string())
            .expect("pinned session must survive prune");
        assert!(reloaded_pinned.pinned);
        assert_eq!(reloaded_pinned, SessionState { pinned: true, ..a.clone() });
        assert!(root
            .join(format!("{}.transcript.json", a.session_id))
            .exists());
        assert!(store.load(&c.session_id.to_string()).is_ok());
        assert!(store.load(&b.session_id.to_string()).is_err());

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn prune_within_budget_after_excluding_pins_returns_empty_removed_with_pinned_surfaced() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let a = seed_session(&store, 1_700_000_000_000, None);
        let b = seed_session(&store, 1_700_000_000_100, None);
        store
            .pin(&a.session_id.to_string())
            .expect("pin session a");
        store
            .pin(&b.session_id.to_string())
            .expect("pin session b");

        // Every session is pinned, so prune with any budget is a clean no-op on
        // removed, but both pins surface via pinned_preserved.
        let outcome = store.prune(0).expect("prune keep=0 with all pinned");
        assert_eq!(outcome.kept_count, 0);
        assert_eq!(outcome.pruned_count, 0);
        assert!(outcome.removed.is_empty());
        assert_eq!(outcome.pinned_preserved_count, 2);
        // Newest-first ordering propagates into pinned_preserved (b is newer).
        assert_eq!(
            outcome.pinned_preserved,
            vec![b.session_id.clone(), a.session_id.clone()]
        );

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn list_and_list_labels_surface_pinned_state() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        let unpinned = seed_session(&store, 1_700_000_000_000, Some("runtime-review"));
        let pinned = seed_session(&store, 1_700_000_000_100, Some("release-candidate"));
        store
            .pin(&pinned.session_id.to_string())
            .expect("pin newest");

        let listings = store.list().expect("list sessions");
        // Newest-first ordering; pinned flag surfaces per listing.
        assert_eq!(listings.len(), 2);
        assert_eq!(listings[0].session_id, pinned.session_id);
        assert!(listings[0].pinned);
        assert_eq!(listings[1].session_id, unpinned.session_id);
        assert!(!listings[1].pinned);

        let labels = store.list_labels().expect("list labels");
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].session_id, pinned.session_id);
        assert!(labels[0].pinned);
        assert!(!labels[1].pinned);

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn check_selector_resolves_raw_id_latest_and_label_and_surfaces_pinned_and_label_metadata() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        // Older unlabeled anchor; newer labeled + pinned target.
        let older = seed_session(&store, 1_700_000_000_100, None);
        let newer = seed_session(&store, 1_700_000_000_500, Some("runtime-review"));
        store
            .pin(&newer.session_id.to_string())
            .expect("pin newer for metadata surfacing");

        let raw_id = older.session_id.to_string();
        let by_id = store.check_selector(&raw_id).expect("check raw id");
        assert_eq!(by_id.selector, raw_id);
        assert_eq!(by_id.resolved_session_id, older.session_id);
        assert_eq!(by_id.message_count, older.messages.len());
        assert_eq!(by_id.created_at_ms, older.created_at_ms);
        assert_eq!(by_id.updated_at_ms, older.updated_at_ms);
        assert_eq!(
            by_id.persisted_path,
            root.join(format!("{raw_id}.json")).display().to_string()
        );
        assert!(by_id.label.is_none());
        assert!(!by_id.pinned);

        // `latest` resolves to the most recently active session (the newer one).
        let by_latest = store.check_selector("latest").expect("check latest");
        assert_eq!(by_latest.selector, "latest");
        assert_eq!(by_latest.resolved_session_id, newer.session_id);
        assert_eq!(by_latest.label.as_deref(), Some("runtime-review"));
        assert!(by_latest.pinned);

        // `label:<name>` routes through resolve_label and surfaces pinned+label.
        let by_label = store
            .check_selector("label:runtime-review")
            .expect("check label");
        assert_eq!(by_label.selector, "label:runtime-review");
        assert_eq!(by_label.resolved_session_id, newer.session_id);
        assert_eq!(by_label.label.as_deref(), Some("runtime-review"));
        assert!(by_label.pinned);

        // Inspect-only: persisted state is unchanged after the three checks.
        let newer_reloaded = store
            .load(&newer.session_id.to_string())
            .expect("reload newer");
        assert!(newer_reloaded.pinned);
        assert_eq!(newer_reloaded.label.as_deref(), Some("runtime-review"));
        assert_eq!(newer_reloaded.updated_at_ms, newer.updated_at_ms);
        let older_reloaded = store.load(&raw_id).expect("reload older");
        assert!(!older_reloaded.pinned);
        assert!(older_reloaded.label.is_none());
        assert_eq!(older_reloaded.updated_at_ms, older.updated_at_ms);

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }

    #[test]
    fn check_selector_preserves_selector_failure_semantics_for_unknown_ambiguous_and_malformed() {
        let root = temp_session_root();
        let store = SessionStore::new(&root);

        // Two sessions carrying the same label for the ambiguity case; plus an
        // untouched anchor so the store root exists.
        let _anchor = seed_session(&store, 1_700_000_000_050, None);
        let _first = seed_session(&store, 1_700_000_000_100, Some("dup"));
        let _second = seed_session(&store, 1_700_000_000_200, Some("dup"));

        // Unknown raw id surfaces SessionNotFound verbatim via load().
        let missing = "00000000-0000-0000-0000-000000000000";
        match store.check_selector(missing) {
            Err(RuntimeError::SessionNotFound(reported)) => assert_eq!(reported, missing),
            other => panic!("expected SessionNotFound for unknown id, got {other:?}"),
        }

        // Unknown label surfaces SessionNotFound with `label:` prefix preserved.
        match store.check_selector("label:missing") {
            Err(RuntimeError::SessionNotFound(reported)) => {
                assert_eq!(reported, "label:missing");
            }
            other => panic!("expected SessionNotFound for unknown label, got {other:?}"),
        }

        // Duplicate labels surface AmbiguousLabel rather than silently picking.
        match store.check_selector("label:dup") {
            Err(RuntimeError::AmbiguousLabel(message)) => {
                assert!(message.contains("\"dup\""), "message should quote label: {message}");
            }
            other => panic!("expected AmbiguousLabel, got {other:?}"),
        }

        // Empty/whitespace `label:` surfaces MalformedSelector, distinct from
        // "unknown label".
        for malformed in ["label:", "label:   "] {
            match store.check_selector(malformed) {
                Err(RuntimeError::MalformedSelector(_)) => {}
                other => panic!("expected MalformedSelector for {malformed:?}, got {other:?}"),
            }
        }

        fs::remove_dir_all(&root).expect("remove temp session test directory");
    }
}
