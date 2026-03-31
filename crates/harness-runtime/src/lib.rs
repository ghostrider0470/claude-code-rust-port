use harness_commands::{CommandRegistry, CommandResult};
use harness_core::{PermissionDenial, RuntimeEvent};
use harness_session::{SessionState, SessionStore, TranscriptStore};
use harness_tools::{PermissionPolicy, ToolRegistry, ToolResult};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoutedMatch {
    pub kind: String,
    pub name: String,
    pub score: usize,
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
            "commands={} tools={} denied_prefixes=bash",
            self.commands.list().len(),
            self.tools.list().len()
        )
    }

    pub fn route(&self, prompt: &str) -> Vec<RoutedMatch> {
        let tokens: Vec<String> = prompt
            .split(|c: char| c.is_whitespace() || c == '/' || c == '-')
            .filter(|token| !token.is_empty())
            .map(|token| token.to_ascii_lowercase())
            .collect();

        let mut matches = Vec::new();

        for command in self.commands.list() {
            let haystacks = [
                command.name.to_ascii_lowercase(),
                command.description.to_ascii_lowercase(),
            ];
            let score = tokens
                .iter()
                .filter(|token| haystacks.iter().any(|hay| hay.contains(token.as_str())))
                .count();
            if score > 0 {
                matches.push(RoutedMatch {
                    kind: "command".into(),
                    name: command.name.clone(),
                    score,
                });
            }
        }

        for tool in self.tools.list() {
            let haystacks = [
                tool.name.to_ascii_lowercase(),
                tool.description.to_ascii_lowercase(),
            ];
            let score = tokens
                .iter()
                .filter(|token| haystacks.iter().any(|hay| hay.contains(token.as_str())))
                .count();
            if score > 0 {
                matches.push(RoutedMatch {
                    kind: "tool".into(),
                    name: tool.name.clone(),
                    score,
                });
            }
        }

        matches.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.kind.cmp(&b.kind))
                .then_with(|| a.name.cmp(&b.name))
        });
        matches
    }

    pub fn bootstrap(&self, prompt: &str) -> Result<TurnReport, String> {
        let mut session = SessionState::default();
        let mut transcript = TranscriptStore::default();
        let mut events = vec![RuntimeEvent::SessionStarted {
            session_id: session.session_id.clone(),
        }];
        events.push(RuntimeEvent::PromptReceived {
            prompt: prompt.to_string(),
        });

        let matches = self.route(prompt);
        events.push(RuntimeEvent::RouteComputed {
            match_count: matches.len(),
        });

        for matched in &matches {
            match matched.kind.as_str() {
                "command" => events.push(RuntimeEvent::CommandMatched {
                    name: matched.name.clone(),
                    score: matched.score,
                }),
                "tool" => events.push(RuntimeEvent::ToolMatched {
                    name: matched.name.clone(),
                    score: matched.score,
                }),
                _ => {}
            }
        }

        let denials: Vec<PermissionDenial> = matches
            .iter()
            .filter(|matched| matched.kind == "tool")
            .filter_map(|matched| self.permissions.denial_for(&matched.name))
            .collect();

        for denial in &denials {
            events.push(RuntimeEvent::PermissionDenied {
                subject: denial.subject.clone(),
                reason: denial.reason.clone(),
            });
        }

        let command_results: Vec<CommandResult> = matches
            .iter()
            .filter(|matched| matched.kind == "command")
            .map(|matched| {
                events.push(RuntimeEvent::CommandInvoked {
                    name: matched.name.clone(),
                });
                let result = self.commands.execute(&matched.name, prompt);
                events.push(RuntimeEvent::CommandCompleted {
                    name: matched.name.clone(),
                    handled: result.handled,
                });
                result
            })
            .collect();

        let tool_results: Vec<ToolResult> = matches
            .iter()
            .filter(|matched| matched.kind == "tool")
            .filter(|matched| self.permissions.denial_for(&matched.name).is_none())
            .map(|matched| {
                events.push(RuntimeEvent::ToolInvoked {
                    name: matched.name.clone(),
                });
                let result = self.tools.execute(&matched.name, prompt);
                events.push(RuntimeEvent::ToolCompleted {
                    name: matched.name.clone(),
                    handled: result.handled,
                });
                result
            })
            .collect();

        session.messages.push(prompt.to_string());
        session.usage = session.usage.add_turn(prompt, "turn completed");
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
