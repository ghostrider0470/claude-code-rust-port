use harness_commands::{CommandRegistry, CommandResult};
use harness_core::{
    CommandName, MatchScore, PermissionDenial, Prompt, RuntimeEvent, SessionId, ToolName, TurnIndex,
};
use harness_session::{
    SessionComparison, SessionComparisonSide, SessionDeletion, SessionExport, SessionFindResult,
    SessionFork, SessionImport, SessionLabelEntry, SessionListing, SessionPin, SessionPinEntry,
    SessionPrune, SessionRename, SessionRetag, SessionSelectorCheck, SessionState, SessionStore,
    SessionTranscriptContext, SessionTranscriptFind, SessionTranscriptFirstTurn,
    SessionTranscriptLastTurn, SessionTranscriptRange, SessionTranscriptTail,
    SessionTranscriptTurnShow, SessionUnlabel,
    SessionUnpin, TranscriptRecord, TranscriptStore,
};
use std::fs;
use std::path::Path;
use harness_tools::{PermissionPolicy, ToolRegistry, ToolResult};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchKind {
    Command,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoutedMatch {
    pub kind: MatchKind,
    pub name: String,
    pub score: MatchScore,
}

#[derive(Debug, Clone, Serialize)]
pub struct TurnReport {
    pub session: SessionState,
    pub transcript: TranscriptStore,
    pub matches: Vec<RoutedMatch>,
    pub denials: Vec<PermissionDenial>,
    pub command_results: Vec<CommandResult>,
    pub tool_results: Vec<ToolResult>,
    pub events: Vec<RuntimeEvent>,
    pub persisted_path: String,
    pub persisted_transcript_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResumeReport {
    pub resumed_session_id: SessionId,
    pub appended_turn_index: TurnIndex,
    pub session: SessionState,
    pub transcript: TranscriptStore,
    pub matches: Vec<RoutedMatch>,
    pub denials: Vec<PermissionDenial>,
    pub command_results: Vec<CommandResult>,
    pub tool_results: Vec<ToolResult>,
    pub events: Vec<RuntimeEvent>,
    pub persisted_path: String,
    pub persisted_transcript_path: String,
}

#[derive(Debug, Clone)]
pub struct RuntimeEngine {
    pub commands: CommandRegistry,
    pub tools: ToolRegistry,
    pub permissions: PermissionPolicy,
    pub store: SessionStore,
}

impl Default for RuntimeEngine {
    fn default() -> Self {
        Self {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(SessionStore::default_root()),
        }
    }
}

impl RuntimeEngine {
    pub fn summary(&self) -> String {
        format!(
            "commands={} tools={} denied_prefixes={}",
            self.commands.list().len(),
            self.tools.list().len(),
            self.permissions.denied_prefixes().join(",")
        )
    }

    pub fn route(&self, prompt: &Prompt) -> Vec<RoutedMatch> {
        let tokens: Vec<String> = prompt
            .as_str()
            .split(|c: char| c.is_whitespace() || c == '/' || c == '-')
            .filter(|token| !token.is_empty())
            .map(|token| token.to_ascii_lowercase())
            .collect();

        let mut matches = Vec::new();

        for command in self.commands.list() {
            let haystacks = [
                command.name.0.to_ascii_lowercase(),
                command.description.to_ascii_lowercase(),
            ];
            let score = tokens
                .iter()
                .filter(|token| haystacks.iter().any(|hay| hay.contains(token.as_str())))
                .count();
            if score > 0 {
                matches.push(RoutedMatch {
                    kind: MatchKind::Command,
                    name: command.name.to_string(),
                    score: MatchScore(score),
                });
            }
        }

        for tool in self.tools.list() {
            let haystacks = [
                tool.name.0.to_ascii_lowercase(),
                tool.description.to_ascii_lowercase(),
            ];
            let score = tokens
                .iter()
                .filter(|token| haystacks.iter().any(|hay| hay.contains(token.as_str())))
                .count();
            if score > 0 {
                matches.push(RoutedMatch {
                    kind: MatchKind::Tool,
                    name: tool.name.to_string(),
                    score: MatchScore(score),
                });
            }
        }

        matches.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| format!("{:?}", a.kind).cmp(&format!("{:?}", b.kind)))
                .then_with(|| a.name.cmp(&b.name))
        });
        matches
    }

    pub fn bootstrap(&self, prompt: Prompt) -> Result<TurnReport, String> {
        let mut session = SessionState::default();
        let mut transcript = TranscriptStore::default();
        let mut events = vec![RuntimeEvent::SessionStarted {
            session_id: SessionId(session.session_id.0),
        }];
        events.push(RuntimeEvent::PromptReceived {
            prompt: prompt.clone(),
        });

        let matches = self.route(&prompt);
        events.push(RuntimeEvent::RouteComputed {
            match_count: matches.len(),
        });

        for matched in &matches {
            match matched.kind {
                MatchKind::Command => events.push(RuntimeEvent::CommandMatched {
                    name: CommandName::new(matched.name.clone()),
                    score: matched.score,
                }),
                MatchKind::Tool => events.push(RuntimeEvent::ToolMatched {
                    name: ToolName::new(matched.name.clone()),
                    score: matched.score,
                }),
            }
        }

        let denials: Vec<PermissionDenial> = matches
            .iter()
            .filter(|matched| matched.kind == MatchKind::Tool)
            .filter_map(|matched| {
                self.permissions
                    .denial_for(&ToolName::new(matched.name.clone()))
            })
            .collect();

        for denial in &denials {
            events.push(RuntimeEvent::PermissionDenied {
                subject: denial.subject.clone(),
                reason: denial.reason.clone(),
            });
        }

        let command_results: Vec<CommandResult> = matches
            .iter()
            .filter(|matched| matched.kind == MatchKind::Command)
            .map(|matched| {
                let name = CommandName::new(matched.name.clone());
                events.push(RuntimeEvent::CommandInvoked { name: name.clone() });
                let result = self.commands.execute(&name, prompt.as_str());
                events.push(RuntimeEvent::CommandCompleted {
                    name,
                    handled: result.handled,
                });
                result
            })
            .collect();

        let tool_results: Vec<ToolResult> = matches
            .iter()
            .filter(|matched| matched.kind == MatchKind::Tool)
            .filter(|matched| {
                self.permissions
                    .denial_for(&ToolName::new(matched.name.clone()))
                    .is_none()
            })
            .map(|matched| {
                let name = ToolName::new(matched.name.clone());
                events.push(RuntimeEvent::ToolInvoked { name: name.clone() });
                let result = self.tools.execute(&name, prompt.as_str());
                events.push(RuntimeEvent::ToolCompleted {
                    name,
                    handled: result.handled,
                });
                result
            })
            .collect();

        session.messages.push(prompt.clone());
        session.usage = session.usage.add_turn(prompt.as_str(), "turn completed");
        transcript.append(prompt);
        transcript.flush();

        events.push(RuntimeEvent::TurnCompleted {
            stop_reason: "completed".into(),
        });

        let persisted_path = self.store.save(&session).map_err(|err| err.to_string())?;
        events.push(RuntimeEvent::SessionPersisted {
            path: persisted_path.display().to_string(),
        });

        let transcript_record = TranscriptRecord::from_session(&session, &transcript);
        let persisted_transcript_path = self
            .store
            .save_transcript(&transcript_record)
            .map_err(|err| err.to_string())?;
        events.push(RuntimeEvent::TranscriptPersisted {
            path: persisted_transcript_path.display().to_string(),
        });

        Ok(TurnReport {
            session,
            transcript,
            matches,
            denials,
            command_results,
            tool_results,
            events,
            persisted_path: persisted_path.display().to_string(),
            persisted_transcript_path: persisted_transcript_path.display().to_string(),
        })
    }

    pub fn resume(&self, target: &str, prompt: Prompt) -> Result<ResumeReport, String> {
        let mut session = self.load_session(target)?;
        let resumed_session_id = session.session_id.clone();

        let mut transcript = TranscriptStore::default();
        for existing in &session.messages {
            transcript.append(existing.clone());
        }

        let appended_turn_index = TurnIndex(session.messages.len());
        let mut events = vec![RuntimeEvent::SessionResumed {
            session_id: resumed_session_id.clone(),
            turn_index: appended_turn_index,
        }];
        events.push(RuntimeEvent::PromptReceived {
            prompt: prompt.clone(),
        });

        let matches = self.route(&prompt);
        events.push(RuntimeEvent::RouteComputed {
            match_count: matches.len(),
        });

        for matched in &matches {
            match matched.kind {
                MatchKind::Command => events.push(RuntimeEvent::CommandMatched {
                    name: CommandName::new(matched.name.clone()),
                    score: matched.score,
                }),
                MatchKind::Tool => events.push(RuntimeEvent::ToolMatched {
                    name: ToolName::new(matched.name.clone()),
                    score: matched.score,
                }),
            }
        }

        let denials: Vec<PermissionDenial> = matches
            .iter()
            .filter(|matched| matched.kind == MatchKind::Tool)
            .filter_map(|matched| {
                self.permissions
                    .denial_for(&ToolName::new(matched.name.clone()))
            })
            .collect();

        for denial in &denials {
            events.push(RuntimeEvent::PermissionDenied {
                subject: denial.subject.clone(),
                reason: denial.reason.clone(),
            });
        }

        let command_results: Vec<CommandResult> = matches
            .iter()
            .filter(|matched| matched.kind == MatchKind::Command)
            .map(|matched| {
                let name = CommandName::new(matched.name.clone());
                events.push(RuntimeEvent::CommandInvoked { name: name.clone() });
                let result = self.commands.execute(&name, prompt.as_str());
                events.push(RuntimeEvent::CommandCompleted {
                    name,
                    handled: result.handled,
                });
                result
            })
            .collect();

        let tool_results: Vec<ToolResult> = matches
            .iter()
            .filter(|matched| matched.kind == MatchKind::Tool)
            .filter(|matched| {
                self.permissions
                    .denial_for(&ToolName::new(matched.name.clone()))
                    .is_none()
            })
            .map(|matched| {
                let name = ToolName::new(matched.name.clone());
                events.push(RuntimeEvent::ToolInvoked { name: name.clone() });
                let result = self.tools.execute(&name, prompt.as_str());
                events.push(RuntimeEvent::ToolCompleted {
                    name,
                    handled: result.handled,
                });
                result
            })
            .collect();

        session.messages.push(prompt.clone());
        session.usage = session.usage.add_turn(prompt.as_str(), "turn completed");
        session.touch();
        transcript.append(prompt);
        transcript.flush();

        events.push(RuntimeEvent::TurnCompleted {
            stop_reason: "completed".into(),
        });

        let persisted_path = self.store.save(&session).map_err(|err| err.to_string())?;
        events.push(RuntimeEvent::SessionPersisted {
            path: persisted_path.display().to_string(),
        });

        let transcript_record = TranscriptRecord::from_session(&session, &transcript);
        let persisted_transcript_path = self
            .store
            .save_transcript(&transcript_record)
            .map_err(|err| err.to_string())?;
        events.push(RuntimeEvent::TranscriptPersisted {
            path: persisted_transcript_path.display().to_string(),
        });

        Ok(ResumeReport {
            resumed_session_id,
            appended_turn_index,
            session,
            transcript,
            matches,
            denials,
            command_results,
            tool_results,
            events,
            persisted_path: persisted_path.display().to_string(),
            persisted_transcript_path: persisted_transcript_path.display().to_string(),
        })
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionListing>, String> {
        self.store.list().map_err(|err| err.to_string())
    }

    pub fn find_sessions(&self, query: &str) -> Result<Vec<SessionFindResult>, String> {
        self.store.find(query).map_err(|err| err.to_string())
    }

    pub fn list_session_labels(&self) -> Result<Vec<SessionLabelEntry>, String> {
        self.store.list_labels().map_err(|err| err.to_string())
    }

    pub fn list_session_pins(&self) -> Result<Vec<SessionPinEntry>, String> {
        self.store.list_pins().map_err(|err| err.to_string())
    }

    /// Resolve a CLI selector (`latest`, `label:<name>`, or raw id) to the
    /// concrete persisted `session_id` string. All single-session command
    /// paths funnel through this so label support is uniform.
    pub fn resolve_selector(&self, selector: &str) -> Result<String, String> {
        self.store
            .resolve_selector(selector)
            .map_err(|err| err.to_string())
    }

    pub fn load_session(&self, id: &str) -> Result<SessionState, String> {
        let resolved = self.resolve_selector(id)?;
        self.store.load(&resolved).map_err(|err| err.to_string())
    }

    pub fn load_transcript(&self, id: &str) -> Result<TranscriptRecord, String> {
        let resolved = self.resolve_selector(id)?;
        self.store
            .load_transcript(&resolved)
            .map_err(|err| err.to_string())
    }

    pub fn export_session(&self, id: &str) -> Result<SessionExport, String> {
        let session = self.load_session(id)?;
        let transcript = self
            .store
            .load_transcript(&session.session_id.to_string())
            .map_err(|err| err.to_string())?;
        Ok(SessionExport::new(session, transcript))
    }

    pub fn compare_sessions(&self, left: &str, right: &str) -> Result<SessionComparison, String> {
        Ok(SessionComparison::new(
            self.comparison_side_for(left)?,
            self.comparison_side_for(right)?,
        ))
    }

    pub fn import_session(&self, bundle_path: &str) -> Result<SessionImport, String> {
        let path = Path::new(bundle_path);
        let body = fs::read_to_string(path).map_err(|err| {
            format!("failed to read bundle at {}: {err}", path.display())
        })?;
        let bundle: SessionExport = serde_json::from_str(&body).map_err(|err| {
            format!("failed to parse session bundle at {}: {err}", path.display())
        })?;
        self.store
            .import_bundle(&bundle)
            .map_err(|err| err.to_string())
    }

    pub fn fork_session(&self, target: &str, prompt: Prompt) -> Result<SessionFork, String> {
        let source = self.load_session(target)?;
        self.store
            .fork(&source.session_id.to_string(), prompt)
            .map_err(|err| err.to_string())
    }

    pub fn rename_session(&self, target: &str, label: &str) -> Result<SessionRename, String> {
        let source = self.load_session(target)?;
        self.store
            .rename(&source.session_id.to_string(), label)
            .map_err(|err| err.to_string())
    }

    pub fn unlabel_session(&self, target: &str) -> Result<SessionUnlabel, String> {
        let resolved = self.resolve_selector(target)?;
        self.store
            .unlabel(&resolved)
            .map_err(|err| err.to_string())
    }

    pub fn retag_session(&self, target: &str, label: &str) -> Result<SessionRetag, String> {
        let resolved = self.resolve_selector(target)?;
        self.store
            .retag(&resolved, label)
            .map_err(|err| err.to_string())
    }

    pub fn delete_session(&self, target: &str) -> Result<SessionDeletion, String> {
        let resolved = self.resolve_selector(target)?;
        self.store.delete(&resolved).map_err(|err| err.to_string())
    }

    pub fn prune_sessions(&self, keep: usize) -> Result<SessionPrune, String> {
        self.store.prune(keep).map_err(|err| err.to_string())
    }

    pub fn pin_session(&self, target: &str) -> Result<SessionPin, String> {
        let resolved = self.resolve_selector(target)?;
        self.store.pin(&resolved).map_err(|err| err.to_string())
    }

    pub fn unpin_session(&self, target: &str) -> Result<SessionUnpin, String> {
        let resolved = self.resolve_selector(target)?;
        self.store.unpin(&resolved).map_err(|err| err.to_string())
    }

    pub fn check_session_selector(
        &self,
        selector: &str,
    ) -> Result<SessionSelectorCheck, String> {
        self.store
            .check_selector(selector)
            .map_err(|err| err.to_string())
    }

    pub fn tail_session_transcript(
        &self,
        selector: &str,
        count: usize,
    ) -> Result<SessionTranscriptTail, String> {
        self.store
            .tail_transcript(selector, count)
            .map_err(|err| err.to_string())
    }

    pub fn find_in_session_transcript(
        &self,
        selector: &str,
        query: &str,
    ) -> Result<SessionTranscriptFind, String> {
        self.store
            .find_in_transcript(selector, query)
            .map_err(|err| err.to_string())
    }

    pub fn range_session_transcript(
        &self,
        selector: &str,
        start: usize,
        count: usize,
    ) -> Result<SessionTranscriptRange, String> {
        self.store
            .range_transcript(selector, start, count)
            .map_err(|err| err.to_string())
    }

    pub fn context_session_transcript(
        &self,
        selector: &str,
        turn: usize,
        before: usize,
        after: usize,
    ) -> Result<SessionTranscriptContext, String> {
        self.store
            .context_transcript(selector, turn, before, after)
            .map_err(|err| err.to_string())
    }

    pub fn turn_show_session_transcript(
        &self,
        selector: &str,
        turn: usize,
    ) -> Result<SessionTranscriptTurnShow, String> {
        self.store
            .turn_show_transcript(selector, turn)
            .map_err(|err| err.to_string())
    }

    pub fn last_turn_session_transcript(
        &self,
        selector: &str,
    ) -> Result<SessionTranscriptLastTurn, String> {
        self.store
            .last_turn_transcript(selector)
            .map_err(|err| err.to_string())
    }

    pub fn first_turn_session_transcript(
        &self,
        selector: &str,
    ) -> Result<SessionTranscriptFirstTurn, String> {
        self.store
            .first_turn_transcript(selector)
            .map_err(|err| err.to_string())
    }

    fn comparison_side_for(&self, id: &str) -> Result<SessionComparisonSide, String> {
        let session = self.load_session(id)?;
        let transcript = self
            .store
            .load_transcript(&session.session_id.to_string())
            .map_err(|err| err.to_string())?;
        Ok(SessionComparisonSide::from_parts(&session, &transcript))
    }
}

#[cfg(test)]
mod tests {
    use super::{MatchKind, RuntimeEngine};
    use harness_commands::CommandRegistry;
    use harness_core::{Prompt, RuntimeEvent};
    use harness_session::SessionStore;
    use harness_tools::{PermissionPolicy, ToolRegistry};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_session_root() -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("harness-runtime-tests-{nonce}"))
    }

    #[test]
    fn route_orders_matches_deterministically() {
        let engine = RuntimeEngine::default();

        let matches = engine.route(&Prompt::new("review bash file"));
        let ordered: Vec<(MatchKind, String, usize)> = matches
            .into_iter()
            .map(|matched| (matched.kind, matched.name, matched.score.0))
            .collect();

        assert_eq!(
            ordered,
            vec![
                (MatchKind::Command, "review".to_string(), 1),
                (MatchKind::Tool, "Bash".to_string(), 1),
                (MatchKind::Tool, "EditFile".to_string(), 1),
                (MatchKind::Tool, "ReadFile".to_string(), 1),
            ]
        );
    }

    #[test]
    fn resume_appends_to_existing_session_and_emits_resume_event() {
        use harness_core::{Prompt, RuntimeEvent, TurnIndex};

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let bootstrap = engine
            .bootstrap(Prompt::new("review bash"))
            .expect("bootstrap session");
        let original_id = bootstrap.session.session_id.to_string();
        let original_updated = bootstrap.session.updated_at_ms;

        std::thread::sleep(std::time::Duration::from_millis(2));

        let resumed = engine
            .resume(&original_id, Prompt::new("summary please"))
            .expect("resume existing session");

        assert_eq!(resumed.resumed_session_id.to_string(), original_id);
        assert_eq!(resumed.appended_turn_index, TurnIndex(1));
        assert_eq!(resumed.session.session_id.to_string(), original_id);
        assert_eq!(
            resumed
                .session
                .messages
                .iter()
                .map(|prompt| prompt.0.clone())
                .collect::<Vec<_>>(),
            vec!["review bash".to_string(), "summary please".to_string()]
        );
        assert_eq!(resumed.session.created_at_ms, bootstrap.session.created_at_ms);
        assert!(resumed.session.updated_at_ms >= original_updated);

        assert!(resumed.events.iter().any(|event| matches!(
            event,
            RuntimeEvent::SessionResumed { session_id, turn_index }
                if session_id.to_string() == original_id && *turn_index == TurnIndex(1)
        )));
        assert!(resumed.transcript.flushed);
        assert_eq!(resumed.transcript.entries.len(), 2);

        let reloaded = engine
            .load_session(&original_id)
            .expect("reload resumed session");
        assert_eq!(reloaded, resumed.session);

        let latest = engine
            .load_session("latest")
            .expect("latest should resolve to resumed session");
        assert_eq!(latest.session_id.to_string(), original_id);

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn resume_latest_targets_most_recently_active_session() {
        use harness_core::Prompt;

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let first = engine
            .bootstrap(Prompt::new("review bash"))
            .expect("bootstrap first session");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let _second = engine
            .bootstrap(Prompt::new("summary"))
            .expect("bootstrap second session");
        std::thread::sleep(std::time::Duration::from_millis(2));

        let resumed = engine
            .resume(&first.session.session_id.to_string(), Prompt::new("follow up"))
            .expect("resume first session");
        std::thread::sleep(std::time::Duration::from_millis(2));

        let resumed_via_latest = engine
            .resume("latest", Prompt::new("one more"))
            .expect("resume via latest");

        assert_eq!(
            resumed_via_latest.resumed_session_id,
            resumed.resumed_session_id,
            "latest should point at most recently updated session"
        );

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn bootstrap_persists_transcript_and_resume_extends_it_in_order() {
        use harness_core::{Prompt, RuntimeEvent};

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let bootstrap = engine
            .bootstrap(Prompt::new("review bash"))
            .expect("bootstrap session");
        let id = bootstrap.session.session_id.to_string();

        assert!(bootstrap
            .persisted_transcript_path
            .ends_with(&format!("{id}.transcript.json")));
        assert!(bootstrap.events.iter().any(|event| matches!(
            event,
            RuntimeEvent::TranscriptPersisted { path }
                if path == &bootstrap.persisted_transcript_path
        )));

        let loaded = engine
            .load_transcript(&id)
            .expect("load transcript by id");
        assert_eq!(loaded.session_id.to_string(), id);
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].prompt.0, "review bash");
        assert_eq!(loaded.entries[0].turn_index.0, 0);

        std::thread::sleep(std::time::Duration::from_millis(2));
        let resumed = engine
            .resume(&id, Prompt::new("summary please"))
            .expect("resume session");
        assert!(resumed
            .persisted_transcript_path
            .ends_with(&format!("{id}.transcript.json")));

        let after_resume = engine
            .load_transcript(&id)
            .expect("reload transcript after resume");
        let ordered: Vec<(usize, String)> = after_resume
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
        assert_eq!(after_resume.session_id.to_string(), id);
        assert_eq!(after_resume.updated_at_ms, resumed.session.updated_at_ms);

        let latest = engine
            .load_transcript("latest")
            .expect("load latest transcript");
        assert_eq!(latest.session_id.to_string(), id);
        assert_eq!(latest.entries.len(), 2);

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn export_session_bundles_persisted_state_and_transcript_for_id_and_latest() {
        use harness_core::Prompt;

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let bootstrap = engine
            .bootstrap(Prompt::new("review bash"))
            .expect("bootstrap session");
        let id = bootstrap.session.session_id.to_string();

        std::thread::sleep(std::time::Duration::from_millis(2));
        let resumed = engine
            .resume(&id, Prompt::new("summary please"))
            .expect("resume session");

        let export = engine.export_session(&id).expect("export by id");
        assert_eq!(export.exported_session_id.to_string(), id);
        assert_eq!(export.session, resumed.session);
        assert_eq!(export.transcript.session_id.to_string(), id);
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
            ]
        );

        let latest_export = engine.export_session("latest").expect("export latest");
        assert_eq!(latest_export, export, "`latest` must resolve to the same bundle");

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn compare_sessions_reports_signed_deltas_between_persisted_sessions_and_supports_latest() {
        use harness_core::Prompt;

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let first = engine
            .bootstrap(Prompt::new("review bash"))
            .expect("bootstrap first session");
        let first_id = first.session.session_id.to_string();

        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = engine
            .bootstrap(Prompt::new("summary"))
            .expect("bootstrap second session");
        let second_id = second.session.session_id.to_string();

        std::thread::sleep(std::time::Duration::from_millis(2));
        let resumed_second = engine
            .resume(&second_id, Prompt::new("follow up"))
            .expect("resume second session");

        let comparison = engine
            .compare_sessions(&first_id, &second_id)
            .expect("compare two persisted sessions");

        assert_eq!(comparison.left_session_id.to_string(), first_id);
        assert_eq!(comparison.right_session_id.to_string(), second_id);
        assert_eq!(comparison.left.message_count, 1);
        assert_eq!(comparison.right.message_count, 2);
        assert_eq!(comparison.left.transcript_entry_count, 1);
        assert_eq!(comparison.right.transcript_entry_count, 2);
        assert!(!comparison.differences.same_session);
        assert_eq!(comparison.differences.message_count_delta, 1);
        assert_eq!(comparison.differences.transcript_entry_count_delta, 1);
        assert!(comparison.differences.updated_at_ms_delta >= 0);
        assert_eq!(
            comparison.right.updated_at_ms,
            resumed_second.session.updated_at_ms
        );

        let latest_right = engine
            .compare_sessions(&first_id, "latest")
            .expect("compare with latest on right side");
        assert_eq!(latest_right.right_session_id.to_string(), second_id);
        assert_eq!(latest_right, comparison);

        let latest_left = engine
            .compare_sessions("latest", &first_id)
            .expect("compare with latest on left side");
        assert_eq!(latest_left.left_session_id.to_string(), second_id);
        assert_eq!(latest_left.right_session_id.to_string(), first_id);
        assert_eq!(
            latest_left.differences.message_count_delta,
            -comparison.differences.message_count_delta
        );

        let self_compare = engine
            .compare_sessions(&first_id, &first_id)
            .expect("compare session with itself");
        assert!(self_compare.differences.same_session);
        assert_eq!(self_compare.differences.message_count_delta, 0);
        assert_eq!(self_compare.differences.updated_at_ms_delta, 0);

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn delete_session_removes_session_and_transcript_for_explicit_id_and_fails_when_missing() {
        use harness_core::Prompt;

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let bootstrap = engine
            .bootstrap(Prompt::new("review bash"))
            .expect("bootstrap session");
        let id = bootstrap.session.session_id.to_string();

        let deletion = engine
            .delete_session(&id)
            .expect("delete persisted session by id");

        assert_eq!(deletion.deleted_session_id.to_string(), id);
        assert_eq!(
            deletion.removed_paths,
            vec![
                bootstrap.persisted_path.clone(),
                bootstrap.persisted_transcript_path.clone(),
            ]
        );

        assert!(engine.load_session(&id).is_err());
        assert!(engine.load_transcript(&id).is_err());
        assert!(engine.list_sessions().expect("list").is_empty());

        let second_attempt = engine.delete_session(&id);
        assert!(
            second_attempt.is_err(),
            "deleting an already-deleted session must fail cleanly"
        );

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn delete_session_latest_targets_most_recently_active_and_preserves_siblings() {
        use harness_core::Prompt;

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let first = engine
            .bootstrap(Prompt::new("review bash"))
            .expect("bootstrap first session");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = engine
            .bootstrap(Prompt::new("summary"))
            .expect("bootstrap second session");
        let first_id = first.session.session_id.to_string();
        let second_id = second.session.session_id.to_string();

        let deletion = engine
            .delete_session("latest")
            .expect("delete latest persisted session");

        assert_eq!(
            deletion.deleted_session_id.to_string(),
            second_id,
            "`latest` must resolve to the most recently active session"
        );
        assert_eq!(
            deletion.removed_paths,
            vec![
                second.persisted_path.clone(),
                second.persisted_transcript_path.clone(),
            ]
        );

        assert!(engine.load_session(&second_id).is_err());
        let surviving = engine
            .load_session(&first_id)
            .expect("untouched session must still load");
        assert_eq!(surviving, first.session);
        let remaining_ids: Vec<String> = engine
            .list_sessions()
            .expect("list after delete")
            .into_iter()
            .map(|listing| listing.session_id.to_string())
            .collect();
        assert_eq!(remaining_ids, vec![first_id]);

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn import_session_round_trips_export_into_a_fresh_store_and_rejects_duplicates() {
        use harness_core::Prompt;
        use std::path::Path;

        let source_root = temp_session_root();
        let source_engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&source_root),
        };

        let bootstrap = source_engine
            .bootstrap(Prompt::new("review bash"))
            .expect("bootstrap session");
        let id = bootstrap.session.session_id.to_string();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let _ = source_engine
            .resume(&id, Prompt::new("summary please"))
            .expect("resume session");

        let export = source_engine.export_session(&id).expect("export session");

        let bundle_path = std::env::temp_dir().join(format!("harness-import-bundle-{id}.json"));
        let body = serde_json::to_string_pretty(&export).expect("serialize bundle");
        fs::write(&bundle_path, &body).expect("write bundle");

        let target_root = temp_session_root();
        let target_engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&target_root),
        };

        let imported = target_engine
            .import_session(bundle_path.to_str().expect("bundle path utf8"))
            .expect("import session bundle");
        assert_eq!(imported.imported_session_id.to_string(), id);
        assert_eq!(
            imported.session_path,
            target_root.join(format!("{id}.json")).display().to_string()
        );
        assert_eq!(
            imported.transcript_path,
            target_root
                .join(format!("{id}.transcript.json"))
                .display()
                .to_string()
        );
        assert!(Path::new(&imported.session_path).exists());
        assert!(Path::new(&imported.transcript_path).exists());

        let reloaded = target_engine
            .load_session(&id)
            .expect("reload imported session");
        assert_eq!(reloaded, export.session);
        let reloaded_transcript = target_engine
            .load_transcript(&id)
            .expect("reload imported transcript");
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

        let duplicate = target_engine.import_session(bundle_path.to_str().unwrap());
        assert!(duplicate.is_err(), "duplicate import must fail cleanly");
        let err = duplicate.unwrap_err();
        assert!(
            err.contains("session already exists"),
            "duplicate-import error should mention existing session, got: {err}"
        );

        fs::remove_file(&bundle_path).ok();
        fs::remove_dir_all(&source_root).ok();
        fs::remove_dir_all(&target_root).ok();
    }

    #[test]
    fn import_session_reports_missing_bundle_path_cleanly() {
        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let missing =
            std::env::temp_dir().join("harness-import-bundle-definitely-missing-xyzzy.json");
        let _ = fs::remove_file(&missing);
        let result = engine.import_session(missing.to_str().unwrap());
        assert!(result.is_err(), "missing bundle path must fail");
        let err = result.unwrap_err();
        assert!(
            err.contains("failed to read bundle"),
            "error should describe read failure, got: {err}"
        );

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn find_sessions_returns_newest_first_matches_and_handles_empty_results() {
        use harness_core::Prompt;

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let first = engine
            .bootstrap(Prompt::new("review bash"))
            .expect("bootstrap first session");
        let first_id = first.session.session_id.to_string();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = engine
            .bootstrap(Prompt::new("summary please"))
            .expect("bootstrap second session");
        let second_id = second.session.session_id.to_string();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let _ = engine
            .resume(&first_id, Prompt::new("review tools"))
            .expect("resume first session");

        let results = engine
            .find_sessions("review")
            .expect("find sessions by review query");
        let ids: Vec<String> = results
            .iter()
            .map(|result| result.session_id.to_string())
            .collect();
        assert_eq!(
            ids,
            vec![first_id.clone()],
            "find must report only sessions whose transcripts contain the query, newest-first"
        );
        let matches: Vec<(usize, String)> = results[0]
            .matches
            .iter()
            .map(|m| (m.turn_index.0, m.prompt.0.clone()))
            .collect();
        assert_eq!(
            matches,
            vec![
                (0, "review bash".to_string()),
                (1, "review tools".to_string()),
            ],
            "matches must preserve turn_index ordering across resume-appended turns"
        );

        let summary_results = engine
            .find_sessions("summary")
            .expect("find sessions by summary query");
        let summary_ids: Vec<String> = summary_results
            .iter()
            .map(|result| result.session_id.to_string())
            .collect();
        assert_eq!(summary_ids, vec![second_id]);

        let empty = engine
            .find_sessions("definitely-not-present")
            .expect("find sessions with no matches");
        assert!(
            empty.is_empty(),
            "an unmatched query must succeed cleanly with an empty result set"
        );

        let empty_query = engine
            .find_sessions("")
            .expect("find sessions with empty query");
        assert!(
            empty_query.is_empty(),
            "an empty query must succeed cleanly with an empty result set"
        );

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn fork_session_creates_fresh_session_id_preserves_source_and_supports_latest() {
        use harness_core::{Prompt, TurnIndex};

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let bootstrap = engine
            .bootstrap(Prompt::new("review bash"))
            .expect("bootstrap session");
        let source_id = bootstrap.session.session_id.to_string();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let _ = engine
            .resume(&source_id, Prompt::new("summary please"))
            .expect("resume source session");

        let fork = engine
            .fork_session(&source_id, Prompt::new("try again"))
            .expect("fork source session by id");

        assert_eq!(fork.source_session_id.to_string(), source_id);
        assert_ne!(
            fork.forked_session_id.to_string(),
            source_id,
            "forked session id must differ from source"
        );
        assert_eq!(fork.appended_turn_index, TurnIndex(2));
        assert!(
            std::path::Path::new(&fork.session_path).exists(),
            "forked session file must be written"
        );
        assert!(
            std::path::Path::new(&fork.transcript_path).exists(),
            "forked transcript file must be written"
        );

        let forked_session = engine
            .load_session(&fork.forked_session_id.to_string())
            .expect("load forked session");
        let forked_messages: Vec<String> = forked_session
            .messages
            .iter()
            .map(|prompt| prompt.0.clone())
            .collect();
        assert_eq!(
            forked_messages,
            vec![
                "review bash".to_string(),
                "summary please".to_string(),
                "try again".to_string(),
            ]
        );

        let forked_transcript = engine
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
            ]
        );

        let source_session = engine
            .load_session(&source_id)
            .expect("source session must still load");
        let source_messages: Vec<String> = source_session
            .messages
            .iter()
            .map(|prompt| prompt.0.clone())
            .collect();
        assert_eq!(
            source_messages,
            vec!["review bash".to_string(), "summary please".to_string()],
            "source session must not be mutated by the fork"
        );
        let source_transcript = engine
            .load_transcript(&source_id)
            .expect("source transcript must still load");
        let source_ordered: Vec<(usize, String)> = source_transcript
            .entries
            .iter()
            .map(|entry| (entry.turn_index.0, entry.prompt.0.clone()))
            .collect();
        assert_eq!(
            source_ordered,
            vec![
                (0, "review bash".to_string()),
                (1, "summary please".to_string()),
            ],
            "source transcript must not be mutated by the fork"
        );

        std::thread::sleep(std::time::Duration::from_millis(2));
        let latest_fork = engine
            .fork_session("latest", Prompt::new("via latest"))
            .expect("fork via latest selector");
        assert_eq!(
            latest_fork.source_session_id.to_string(),
            fork.forked_session_id.to_string(),
            "`latest` must resolve to the most recently active persisted session (the prior fork)"
        );
        assert_ne!(
            latest_fork.forked_session_id, fork.forked_session_id,
            "latest-fork must produce a fresh session id"
        );

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn fork_session_reports_missing_source_cleanly() {
        use harness_core::{Prompt, SessionId};

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let missing = SessionId::new().to_string();
        let result = engine.fork_session(&missing, Prompt::new("will not land"));
        assert!(result.is_err(), "missing source id must fail");
        assert!(
            engine
                .list_sessions()
                .expect("list sessions after failed fork")
                .is_empty(),
            "no persisted sessions should exist after a failed fork"
        );

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn bootstrap_emits_denial_and_persists_session_state() {
        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let report = engine
            .bootstrap(Prompt::new("review bash"))
            .expect("bootstrap runtime turn");
        let reloaded = engine
            .load_session(&report.session.session_id.to_string())
            .expect("reload persisted session");

        assert_eq!(report.command_results.len(), 1);
        assert_eq!(report.command_results[0].name.0, "review");
        assert!(report.command_results[0].handled);

        assert_eq!(report.denials.len(), 1);
        assert_eq!(report.denials[0].subject, "Bash");
        assert_eq!(report.tool_results.len(), 0);

        assert!(report.transcript.flushed);
        assert_eq!(report.session.messages, vec![Prompt::new("review bash")]);
        assert_eq!(reloaded, report.session);
        assert!(report.persisted_path.starts_with(root.to_string_lossy().as_ref()));

        assert!(report.events.iter().any(|event| matches!(
            event,
            RuntimeEvent::PermissionDenied { subject, reason }
                if subject == "Bash" && reason == "tool blocked by permission policy"
        )));
        assert!(report.events.iter().any(|event| matches!(
            event,
            RuntimeEvent::SessionPersisted { path }
                if path == &report.persisted_path
        )));

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn rename_session_applies_label_preserves_id_and_transcript_and_supports_latest() {
        use harness_core::Prompt;

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let bootstrap = engine
            .bootstrap(Prompt::new("review bash"))
            .expect("bootstrap session");
        let id = bootstrap.session.session_id.to_string();
        let original_updated = bootstrap.session.updated_at_ms;
        let original_messages = bootstrap.session.messages.clone();
        let transcript_before = engine
            .load_transcript(&id)
            .expect("load transcript before rename");

        let renamed = engine
            .rename_session(&id, "  runtime-review  ")
            .expect("rename by explicit id");
        assert_eq!(renamed.renamed_session_id.to_string(), id);
        assert_eq!(renamed.applied_label, "runtime-review");

        let reloaded = engine
            .load_session(&id)
            .expect("reload renamed session");
        assert_eq!(reloaded.session_id.to_string(), id);
        assert_eq!(reloaded.label.as_deref(), Some("runtime-review"));
        assert_eq!(reloaded.messages, original_messages);
        assert_eq!(
            reloaded.updated_at_ms, original_updated,
            "rename must not bump activity metadata"
        );

        let transcript_after = engine
            .load_transcript(&id)
            .expect("load transcript after rename");
        assert_eq!(
            transcript_after, transcript_before,
            "rename must not mutate transcript entries or ordering"
        );

        let second = engine
            .bootstrap(Prompt::new("summary please"))
            .expect("bootstrap second session");
        let second_id = second.session.session_id.to_string();
        let latest_rename = engine
            .rename_session("latest", "second-label")
            .expect("rename via latest selector");
        assert_eq!(
            latest_rename.renamed_session_id.to_string(),
            second_id,
            "`latest` must resolve to the most recently active persisted session"
        );
        assert_eq!(latest_rename.applied_label, "second-label");

        let first_after = engine
            .load_session(&id)
            .expect("first session must still load");
        assert_eq!(
            first_after.label.as_deref(),
            Some("runtime-review"),
            "first session's label must survive a rename of a different session"
        );

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn label_selectors_target_persisted_sessions_across_load_show_compare_and_delete() {
        use harness_core::Prompt;

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let alpha = engine
            .bootstrap(Prompt::new("review bash"))
            .expect("bootstrap alpha session");
        let alpha_id = alpha.session.session_id.to_string();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let beta = engine
            .bootstrap(Prompt::new("summary please"))
            .expect("bootstrap beta session");
        let beta_id = beta.session.session_id.to_string();

        // Label only the older session so latest != labeled — proves the
        // selectors resolve independently and not via newest-first fallback.
        engine
            .rename_session(&alpha_id, "alpha-label")
            .expect("label alpha session");

        // load_session via label resolves to the labeled (older) session.
        let loaded = engine
            .load_session("label:alpha-label")
            .expect("load by label selector");
        assert_eq!(loaded.session_id.to_string(), alpha_id);

        // Raw id targeting still works unchanged after labeling exists.
        let raw_loaded = engine
            .load_session(&beta_id)
            .expect("load by raw id still works");
        assert_eq!(raw_loaded.session_id.to_string(), beta_id);

        // compare_sessions accepts label on one side and latest on the other,
        // and the resulting JSON identifies the actual resolved session_ids
        // rather than the selector strings.
        let comparison = engine
            .compare_sessions("label:alpha-label", "latest")
            .expect("label-vs-latest compare");
        assert_eq!(comparison.left_session_id.to_string(), alpha_id);
        assert_eq!(comparison.right_session_id.to_string(), beta_id);
        assert!(!comparison.differences.same_session);

        // delete_session via label resolves to the labeled session and leaves
        // the unlabeled sibling intact.
        let deletion = engine
            .delete_session("label:alpha-label")
            .expect("delete by label");
        assert_eq!(deletion.deleted_session_id.to_string(), alpha_id);
        assert!(engine.load_session(&alpha_id).is_err());
        assert_eq!(
            engine
                .load_session(&beta_id)
                .expect("beta must survive")
                .session_id
                .to_string(),
            beta_id,
        );

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn label_selector_failures_are_clean_for_unknown_ambiguous_and_malformed_inputs() {
        use harness_core::Prompt;

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        // Unknown label fails cleanly and leaves no persisted state behind.
        let unknown = engine.load_session("label:missing");
        assert!(unknown.is_err(), "unknown label must fail");
        assert!(
            unknown.unwrap_err().contains("label:missing"),
            "error should preserve the selector the user typed"
        );

        // Two sessions sharing one label is an ambiguity error.
        let one = engine
            .bootstrap(Prompt::new("alpha"))
            .expect("bootstrap one");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let two = engine
            .bootstrap(Prompt::new("beta"))
            .expect("bootstrap two");
        engine
            .rename_session(&one.session.session_id.to_string(), "shared")
            .expect("label one");
        engine
            .rename_session(&two.session.session_id.to_string(), "shared")
            .expect("label two");
        let ambiguous = engine.load_session("label:shared");
        assert!(ambiguous.is_err(), "ambiguous label must fail");
        let err = ambiguous.unwrap_err();
        assert!(
            err.contains("ambiguous session label"),
            "error should mention ambiguity, got: {err}"
        );

        // `label:` with no name is a malformed selector — distinct from
        // unknown-label so the CLI can surface the right diagnosis.
        let malformed = engine.load_session("label:");
        assert!(malformed.is_err(), "malformed selector must fail");
        assert!(
            malformed.unwrap_err().contains("malformed session selector"),
            "error should mention malformed selector"
        );

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn list_session_labels_returns_newest_first_entries_and_omits_unlabeled_sessions() {
        use harness_core::Prompt;

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        // Three bootstraps in time order. Only two get labeled, and the newest
        // of those is the one we label second.
        let older = engine.bootstrap(Prompt::new("alpha")).expect("bootstrap alpha");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let middle = engine.bootstrap(Prompt::new("beta")).expect("bootstrap beta");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newest = engine
            .bootstrap(Prompt::new("gamma"))
            .expect("bootstrap gamma");

        engine
            .rename_session(&older.session.session_id.to_string(), "runtime-review")
            .expect("label older");
        engine
            .rename_session(&newest.session.session_id.to_string(), "release-candidate")
            .expect("label newest");

        let entries = engine.list_session_labels().expect("list labels");

        let ids: Vec<String> = entries
            .iter()
            .map(|entry| entry.session_id.to_string())
            .collect();
        assert_eq!(
            ids,
            vec![
                newest.session.session_id.to_string(),
                older.session.session_id.to_string(),
            ],
            "labels must be listed in newest-first order and the unlabeled session omitted"
        );
        assert_eq!(entries[0].label, "release-candidate");
        assert_eq!(entries[1].label, "runtime-review");

        // middle must have no label and must be absent from the listing.
        assert!(
            !ids.contains(&middle.session.session_id.to_string()),
            "unlabeled middle session must be omitted"
        );
        assert!(
            engine
                .load_session(&middle.session.session_id.to_string())
                .expect("reload middle")
                .label
                .is_none(),
            "listing must not mutate persisted label state"
        );

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn list_session_labels_keeps_duplicate_labels_separate_and_returns_empty_for_unlabeled_store() {
        use harness_core::Prompt;

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        // With no labeled sessions yet, the listing must be cleanly empty —
        // even when unlabeled sessions exist.
        let _ = engine.bootstrap(Prompt::new("unlabeled")).expect("bootstrap");
        let empty = engine.list_session_labels().expect("list empty");
        assert!(
            empty.is_empty(),
            "unlabeled-only store must yield empty label listing"
        );

        // Two sessions sharing the same label must both appear — ambiguity is
        // discoverable, not collapsed.
        let first = engine.bootstrap(Prompt::new("alpha")).expect("bootstrap alpha");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = engine.bootstrap(Prompt::new("beta")).expect("bootstrap beta");

        engine
            .rename_session(&first.session.session_id.to_string(), "dup")
            .expect("label first");
        engine
            .rename_session(&second.session.session_id.to_string(), "dup")
            .expect("label second");

        let entries = engine.list_session_labels().expect("list labels");
        assert_eq!(entries.len(), 2, "duplicate labels must not be collapsed");
        assert!(entries.iter().all(|entry| entry.label == "dup"));
        assert_eq!(
            entries[0].session_id.to_string(),
            second.session.session_id.to_string(),
            "newest duplicate label must appear first"
        );
        assert_eq!(
            entries[1].session_id.to_string(),
            first.session.session_id.to_string(),
            "older duplicate label must appear second"
        );

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn list_session_pins_orders_newest_first_omits_unpinned_and_surfaces_label() {
        use harness_core::Prompt;

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        // Empty store must return a clean empty array without erroring.
        let empty = engine.list_session_pins().expect("list pins empty");
        assert!(empty.is_empty(), "empty store must yield empty pin listing");

        // Three bootstraps in time order. Only the older and the newest get
        // pinned; the middle one stays unpinned and must be omitted.
        let older = engine.bootstrap(Prompt::new("alpha")).expect("bootstrap alpha");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let middle = engine.bootstrap(Prompt::new("beta")).expect("bootstrap beta");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newest = engine
            .bootstrap(Prompt::new("gamma"))
            .expect("bootstrap gamma");

        // Label only the older pinned session so we can verify optional
        // `label` surfacing alongside an unlabeled pinned session.
        engine
            .rename_session(&older.session.session_id.to_string(), "runtime-review")
            .expect("label older");
        engine
            .pin_session(&older.session.session_id.to_string())
            .expect("pin older");
        engine
            .pin_session(&newest.session.session_id.to_string())
            .expect("pin newest");

        let entries = engine.list_session_pins().expect("list pins");

        let ids: Vec<String> = entries
            .iter()
            .map(|entry| entry.session_id.to_string())
            .collect();
        assert_eq!(
            ids,
            vec![
                newest.session.session_id.to_string(),
                older.session.session_id.to_string(),
            ],
            "pins must be listed in newest-first order and the unpinned session omitted"
        );
        assert!(
            entries.iter().all(|entry| entry.pinned),
            "every surfaced pin row must report pinned: true"
        );
        assert_eq!(
            entries[0].label, None,
            "unlabeled pinned session must not surface a label"
        );
        assert_eq!(
            entries[1].label.as_deref(),
            Some("runtime-review"),
            "labeled pinned session must surface the label"
        );

        // Unpinned session must not appear and must remain unmutated.
        assert!(
            !ids.contains(&middle.session.session_id.to_string()),
            "unpinned middle session must be omitted"
        );
        let reloaded_middle = engine
            .load_session(&middle.session.session_id.to_string())
            .expect("reload middle");
        assert!(!reloaded_middle.pinned, "listing must not mutate persisted pinned state");

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn rename_session_rejects_invalid_labels_and_missing_targets_cleanly() {
        use harness_core::{Prompt, SessionId};

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let bootstrap = engine
            .bootstrap(Prompt::new("review bash"))
            .expect("bootstrap session");
        let id = bootstrap.session.session_id.to_string();

        for invalid in ["", "   ", "\t\n"] {
            let result = engine.rename_session(&id, invalid);
            assert!(result.is_err(), "empty/whitespace label {invalid:?} must fail");
            assert!(
                result.unwrap_err().contains("invalid session label"),
                "invalid label error should mention label"
            );
        }
        assert!(
            engine
                .load_session(&id)
                .expect("reload after rejected rename")
                .label
                .is_none(),
            "rejected rename must not persist a label"
        );

        let missing = SessionId::new().to_string();
        let missing_result = engine.rename_session(&missing, "whatever");
        assert!(missing_result.is_err(), "unknown session id must fail");
        assert!(
            missing_result.unwrap_err().contains("session not found"),
            "missing target error should mention session not found"
        );

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn prune_sessions_delegates_to_store_and_preserves_newest_n_via_bootstrap_pipeline() {
        use harness_core::Prompt;

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let first = engine.bootstrap(Prompt::new("first")).expect("bootstrap 1");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = engine.bootstrap(Prompt::new("second")).expect("bootstrap 2");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let third = engine.bootstrap(Prompt::new("third")).expect("bootstrap 3");

        // Keep the newest two sessions. Only the oldest (`first`) should be pruned.
        let outcome = engine.prune_sessions(2).expect("prune keep 2");
        assert_eq!(outcome.kept_count, 2);
        assert_eq!(outcome.pruned_count, 1);
        assert_eq!(outcome.removed.len(), 1);
        assert_eq!(
            outcome.removed[0].session_id.to_string(),
            first.session.session_id.to_string()
        );

        let remaining: Vec<String> = engine
            .list_sessions()
            .expect("list after prune")
            .into_iter()
            .map(|entry| entry.session_id.to_string())
            .collect();
        assert_eq!(
            remaining,
            vec![
                third.session.session_id.to_string(),
                second.session.session_id.to_string(),
            ],
            "newest-first ordering must be preserved after prune"
        );

        // Preserved session content (and transcript) is untouched.
        let reloaded = engine
            .load_session(&second.session.session_id.to_string())
            .expect("reload preserved");
        assert_eq!(reloaded.messages.len(), 1);

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn pin_session_accepts_explicit_id_latest_and_label_selectors() {
        use harness_core::Prompt;

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let older = engine.bootstrap(Prompt::new("older")).expect("bootstrap older");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let newer = engine.bootstrap(Prompt::new("newer")).expect("bootstrap newer");
        let older_id = older.session.session_id.to_string();
        let newer_id = newer.session.session_id.to_string();

        // Label the older session so `label:` selector resolution is exercised.
        engine
            .rename_session(&older_id, "pin-by-label")
            .expect("label older session");

        // 1) Pin by explicit id.
        let by_id = engine.pin_session(&older_id).expect("pin by id");
        assert_eq!(by_id.pinned_session_id.to_string(), older_id);
        assert!(by_id.pinned);
        engine
            .unpin_session(&older_id)
            .expect("unpin to reuse for latest path");

        // 2) Pin by `latest`.
        let by_latest = engine.pin_session("latest").expect("pin latest");
        assert_eq!(by_latest.pinned_session_id.to_string(), newer_id);
        assert!(by_latest.pinned);
        engine.unpin_session("latest").expect("unpin latest");

        // 3) Pin by `label:<name>` resolves to the labeled (older) session.
        let by_label = engine
            .pin_session("label:pin-by-label")
            .expect("pin by label");
        assert_eq!(by_label.pinned_session_id.to_string(), older_id);
        let unpin_by_label = engine
            .unpin_session("label:pin-by-label")
            .expect("unpin by label");
        assert_eq!(unpin_by_label.unpinned_session_id.to_string(), older_id);
        assert!(!unpin_by_label.pinned);

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }

    #[test]
    fn prune_sessions_skips_pinned_entries_via_runtime_pipeline() {
        use harness_core::Prompt;

        let root = temp_session_root();
        let engine = RuntimeEngine {
            commands: CommandRegistry::seeded(),
            tools: ToolRegistry::seeded(),
            permissions: PermissionPolicy::with_denied_prefixes(["bash"]),
            store: SessionStore::new(&root),
        };

        let first = engine.bootstrap(Prompt::new("first")).expect("bootstrap 1");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = engine.bootstrap(Prompt::new("second")).expect("bootstrap 2");
        std::thread::sleep(std::time::Duration::from_millis(2));
        let third = engine.bootstrap(Prompt::new("third")).expect("bootstrap 3");

        // Pin the oldest session. With keep=1, normally only `third` would survive;
        // the pin must rescue `first` too, and `second` must still be pruned
        // because it is the oldest remaining unpinned session.
        engine
            .pin_session(&first.session.session_id.to_string())
            .expect("pin oldest");

        let outcome = engine.prune_sessions(1).expect("prune keep 1 with pin");
        assert_eq!(outcome.kept_count, 1);
        assert_eq!(outcome.pruned_count, 1);
        assert_eq!(
            outcome.removed[0].session_id.to_string(),
            second.session.session_id.to_string()
        );
        assert_eq!(outcome.pinned_preserved_count, 1);
        assert_eq!(
            outcome.pinned_preserved,
            vec![first.session.session_id.clone()]
        );

        let remaining: Vec<String> = engine
            .list_sessions()
            .expect("list after prune")
            .into_iter()
            .map(|entry| entry.session_id.to_string())
            .collect();
        assert_eq!(
            remaining,
            vec![
                third.session.session_id.to_string(),
                first.session.session_id.to_string(),
            ],
            "newest-first ordering across survivors, pinned first-session rescued"
        );

        fs::remove_dir_all(&root).expect("remove temp runtime test directory");
    }
}
