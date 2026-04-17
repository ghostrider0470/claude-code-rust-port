use harness_commands::{CommandRegistry, CommandResult};
use harness_core::{
    CommandName, MatchScore, PermissionDenial, Prompt, RuntimeEvent, SessionId, ToolName, TurnIndex,
};
use harness_session::{
    SessionListing, SessionState, SessionStore, TranscriptRecord, TranscriptStore,
};
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

    pub fn load_session(&self, id: &str) -> Result<SessionState, String> {
        if id == "latest" {
            return self.store.latest().map_err(|err| err.to_string());
        }

        self.store.load(id).map_err(|err| err.to_string())
    }

    pub fn load_transcript(&self, id: &str) -> Result<TranscriptRecord, String> {
        if id == "latest" {
            return self.store.latest_transcript().map_err(|err| err.to_string());
        }

        self.store.load_transcript(id).map_err(|err| err.to_string())
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
}
