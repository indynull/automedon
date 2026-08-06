use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Instant;

/// Normalized harness event stream — the common vocabulary every adapter maps into.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    Spawned {
        pid: u32,
        harness: String,
    },
    /// Harness-side session identity (for resume/continue multi-turn).
    SessionInfo {
        id: String,
        label: Option<String>,
    },
    TurnStart {
        turn: u64,
    },
    ThinkingDelta {
        text: String,
    },
    TextDelta {
        text: String,
    },
    ToolCall {
        id: String,
        name: String,
        input: Value,
    },
    ToolResult {
        id: String,
        name: String,
        output: String,
        is_error: bool,
    },
    PermissionRequest {
        id: String,
        tool: String,
        detail: String,
    },
    PermissionResolved {
        id: String,
        allowed: bool,
    },
    /// Harness hook lifecycle (PreToolUse, PostToolUse, SessionStart, Stop, …).
    HookStarted {
        id: String,
        name: String,
        /// e.g. `pre`, `post`, `session`, or harness-native phase string.
        phase: Option<String>,
        detail: Option<Value>,
    },
    HookFinished {
        id: String,
        name: String,
        phase: Option<String>,
        ok: bool,
        detail: Option<String>,
    },
    /// Enter plan / design mode (harness-specific UI or headless signal).
    PlanModeEnter {
        reason: Option<String>,
    },
    PlanPresented {
        id: String,
        summary: String,
    },
    PlanResolved {
        id: String,
        approved: bool,
    },
    PlanModeExit {
        reason: Option<String>,
    },
    GoalStarted {
        id: String,
        title: String,
    },
    GoalProgress {
        id: String,
        message: String,
        percent: Option<f64>,
    },
    GoalCompleted {
        id: String,
        success: bool,
        message: Option<String>,
    },
    Usage {
        input_tokens: u64,
        output_tokens: u64,
        total_tokens: u64,
        cost_usd: Option<f64>,
    },
    TurnComplete {
        turn: u64,
        stop_reason: Option<String>,
    },
    /// Process for this turn exited (multi-turn session may continue via resume).
    ProcessExit {
        code: Option<i32>,
    },
    /// Session fully closed (user close or terminal one-shot).
    Done {
        code: Option<i32>,
    },
    /// Unparsed line / notification kept for debugging harness quirks.
    Raw {
        channel: String,
        line: String,
    },
    Error {
        message: String,
    },
}

impl Event {
    pub fn is_session_terminal(&self) -> bool {
        matches!(self, Event::Done { .. })
    }

    pub fn is_turn_boundary(&self) -> bool {
        matches!(
            self,
            Event::TurnComplete { .. } | Event::ProcessExit { .. } | Event::Done { .. }
        )
    }

    pub fn as_text_delta(&self) -> Option<&str> {
        match self {
            Event::TextDelta { text } => Some(text),
            _ => None,
        }
    }

    pub fn tool_name(&self) -> Option<&str> {
        match self {
            Event::ToolCall { name, .. } | Event::ToolResult { name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn hook_name(&self) -> Option<&str> {
        match self {
            Event::HookStarted { name, .. } | Event::HookFinished { name, .. } => Some(name),
            _ => None,
        }
    }
}

/// Timestamped event for the session ring buffer.
#[derive(Debug, Clone)]
pub struct TimedEvent {
    pub at: Instant,
    pub event: Event,
}

/// Accumulated transcript view over a session.
#[derive(Debug, Default, Clone)]
pub struct Transcript {
    events: Vec<TimedEvent>,
    text: String,
    thinking: String,
    turn_text: String,
    turn_thinking: String,
    tools: Vec<ToolRecord>,
    session_id: Option<String>,
    plans: Vec<PlanRecord>,
    goals: Vec<GoalRecord>,
    permissions: Vec<PermissionRecord>,
    hooks: Vec<HookRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRecord {
    pub id: String,
    pub name: String,
    pub input: Value,
    pub output: Option<String>,
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanRecord {
    pub id: String,
    pub summary: String,
    pub approved: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalRecord {
    pub id: String,
    pub title: String,
    pub success: Option<bool>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRecord {
    pub id: String,
    pub tool: String,
    pub detail: String,
    pub allowed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookRecord {
    pub id: String,
    pub name: String,
    pub phase: Option<String>,
    pub finished: bool,
    pub ok: Option<bool>,
}

impl Transcript {
    pub fn push(&mut self, event: Event) {
        match &event {
            Event::SessionInfo { id, .. } => {
                self.session_id = Some(id.clone());
            }
            Event::TurnStart { .. } => {
                self.begin_turn();
            }
            Event::TextDelta { text } => {
                self.text.push_str(text);
                self.turn_text.push_str(text);
            }
            Event::ThinkingDelta { text } => {
                self.thinking.push_str(text);
                self.turn_thinking.push_str(text);
            }
            Event::ToolCall { id, name, input } => {
                self.tools.push(ToolRecord {
                    id: id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                    output: None,
                    is_error: None,
                });
            }
            Event::ToolResult {
                id,
                output,
                is_error,
                ..
            } => {
                if let Some(t) = self.tools.iter_mut().rev().find(|t| t.id == *id) {
                    t.output = Some(output.clone());
                    t.is_error = Some(*is_error);
                }
            }
            Event::PermissionRequest { id, tool, detail } => {
                self.permissions.push(PermissionRecord {
                    id: id.clone(),
                    tool: tool.clone(),
                    detail: detail.clone(),
                    allowed: None,
                });
            }
            Event::PermissionResolved { id, allowed } => {
                if let Some(p) = self.permissions.iter_mut().rev().find(|p| p.id == *id) {
                    p.allowed = Some(*allowed);
                }
            }
            Event::HookStarted {
                id, name, phase, ..
            } => {
                self.hooks.push(HookRecord {
                    id: id.clone(),
                    name: name.clone(),
                    phase: phase.clone(),
                    finished: false,
                    ok: None,
                });
            }
            Event::HookFinished { id, ok, .. } => {
                if let Some(h) = self.hooks.iter_mut().rev().find(|h| h.id == *id) {
                    h.finished = true;
                    h.ok = Some(*ok);
                }
            }
            Event::PlanPresented { id, summary } => {
                self.plans.push(PlanRecord {
                    id: id.clone(),
                    summary: summary.clone(),
                    approved: None,
                });
            }
            Event::PlanResolved { id, approved } => {
                if let Some(p) = self.plans.iter_mut().rev().find(|p| p.id == *id) {
                    p.approved = Some(*approved);
                }
            }
            Event::GoalStarted { id, title } => {
                self.goals.push(GoalRecord {
                    id: id.clone(),
                    title: title.clone(),
                    success: None,
                    message: None,
                });
            }
            Event::GoalCompleted {
                id,
                success,
                message,
            } => {
                if let Some(g) = self.goals.iter_mut().rev().find(|g| g.id == *id) {
                    g.success = Some(*success);
                    g.message = message.clone();
                }
            }
            _ => {}
        }
        self.events.push(TimedEvent {
            at: Instant::now(),
            event,
        });
    }

    pub fn events(&self) -> &[TimedEvent] {
        &self.events
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn turn_text(&self) -> &str {
        &self.turn_text
    }

    pub fn thinking(&self) -> &str {
        &self.thinking
    }

    pub fn turn_thinking(&self) -> &str {
        &self.turn_thinking
    }

    pub fn tools(&self) -> &[ToolRecord] {
        &self.tools
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    pub fn plans(&self) -> &[PlanRecord] {
        &self.plans
    }

    pub fn goals(&self) -> &[GoalRecord] {
        &self.goals
    }

    pub fn permissions(&self) -> &[PermissionRecord] {
        &self.permissions
    }

    pub fn hooks(&self) -> &[HookRecord] {
        &self.hooks
    }

    /// Text deltas only from events in `[from, events.len())`.
    pub fn text_since(&self, from: usize) -> String {
        self.events[from.min(self.events.len())..]
            .iter()
            .filter_map(|te| te.event.as_text_delta())
            .collect()
    }

    /// Thinking deltas only from events in `[from, events.len())`.
    pub fn thinking_since(&self, from: usize) -> String {
        self.events[from.min(self.events.len())..]
            .iter()
            .filter_map(|te| match &te.event {
                Event::ThinkingDelta { text } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Reset per-turn buffers (call at each user prompt, even if the harness
    /// never emits `TurnStart` — e.g. Grok streaming-json).
    pub fn begin_turn(&mut self) {
        self.turn_text.clear();
        self.turn_thinking.clear();
    }
}
