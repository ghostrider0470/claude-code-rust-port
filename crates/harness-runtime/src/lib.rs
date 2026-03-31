use harness_commands::{CommandRegistry, CommandResult};
use harness_core::{
    CommandName, MatchScore, PermissionDenial, Prompt, RuntimeEvent, SessionId, ToolName,
};
use harness_session::{SessionState, SessionStore, TranscriptStore};
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

        Ok(TurnReport {
            session,
            transcript,
            matches,
            denials,
            command_results,
            tool_results,
            events,
            persisted_path: persisted_path.display().to_string(),
        })
    }

    pub fn load_session(&self, id: &str) -> Result<SessionState, String> {
        self.store.load(id).map_err(|err| err.to_string())
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
