use std::fmt;
use std::time::Duration;

use regex::Regex;
use serde_json::Value;

use crate::event::{Event, Transcript};
use crate::Error;

/// Wait condition over the normalized event stream.
#[derive(Debug, Clone)]
pub struct Expect {
    pub predicate: Predicate,
    pub timeout: Duration,
}

impl Expect {
    pub fn new(predicate: Predicate) -> Self {
        Self {
            predicate,
            timeout: Duration::from_secs(120),
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn text(needle: impl Into<String>) -> Self {
        Self::new(Predicate::TextContains(needle.into()))
    }

    pub fn text_regex(pattern: &str) -> Result<Self, Error> {
        let re = Regex::new(pattern).map_err(|e| Error::Other(e.to_string()))?;
        Ok(Self::new(Predicate::TextRegex(re)))
    }

    pub fn thinking(needle: impl Into<String>) -> Self {
        Self::new(Predicate::ThinkingContains(needle.into()))
    }

    pub fn tool(name: impl Into<String>) -> Self {
        Self::new(Predicate::ToolCall {
            name: Some(name.into()),
            input_contains: None,
        })
    }

    pub fn tool_any() -> Self {
        Self::new(Predicate::ToolCall {
            name: None,
            input_contains: None,
        })
    }

    pub fn tool_input(name: impl Into<String>, fragment: impl Into<String>) -> Self {
        Self::new(Predicate::ToolCall {
            name: Some(name.into()),
            input_contains: Some(fragment.into()),
        })
    }

    pub fn tool_result(name: impl Into<String>) -> Self {
        Self::new(Predicate::ToolResult {
            name: Some(name.into()),
            is_error: None,
        })
    }

    pub fn turn_complete() -> Self {
        Self::new(Predicate::TurnComplete)
    }

    pub fn done() -> Self {
        Self::new(Predicate::Done)
    }

    pub fn process_exit() -> Self {
        Self::new(Predicate::ProcessExit)
    }

    pub fn permission() -> Self {
        Self::new(Predicate::Permission)
    }

    /// Wait for a harness hook (any phase) by name, e.g. `PreToolUse`.
    pub fn hook(name: impl Into<String>) -> Self {
        Self::new(Predicate::Hook {
            name: Some(name.into()),
            phase: None,
            finished: None,
        })
    }

    /// Wait for any hook event.
    pub fn hook_any() -> Self {
        Self::new(Predicate::Hook {
            name: None,
            phase: None,
            finished: None,
        })
    }

    /// Wait for hook start (not finished).
    pub fn hook_started(name: impl Into<String>) -> Self {
        Self::new(Predicate::Hook {
            name: Some(name.into()),
            phase: None,
            finished: Some(false),
        })
    }

    /// Wait for hook completion.
    pub fn hook_finished(name: impl Into<String>) -> Self {
        Self::new(Predicate::Hook {
            name: Some(name.into()),
            phase: None,
            finished: Some(true),
        })
    }

    /// Wait for hook with a phase string (`pre`, `post`, …).
    pub fn hook_phase(name: impl Into<String>, phase: impl Into<String>) -> Self {
        Self::new(Predicate::Hook {
            name: Some(name.into()),
            phase: Some(phase.into()),
            finished: None,
        })
    }

    pub fn plan() -> Self {
        Self::new(Predicate::PlanPresented {
            summary_contains: None,
        })
    }

    pub fn plan_summary(needle: impl Into<String>) -> Self {
        Self::new(Predicate::PlanPresented {
            summary_contains: Some(needle.into()),
        })
    }

    pub fn plan_resolved(approved: bool) -> Self {
        Self::new(Predicate::PlanResolved {
            approved: Some(approved),
        })
    }

    pub fn goal() -> Self {
        Self::new(Predicate::GoalStarted {
            title_contains: None,
        })
    }

    pub fn goal_title(needle: impl Into<String>) -> Self {
        Self::new(Predicate::GoalStarted {
            title_contains: Some(needle.into()),
        })
    }

    pub fn goal_progress() -> Self {
        Self::new(Predicate::GoalProgress)
    }

    pub fn goal_completed(success: bool) -> Self {
        Self::new(Predicate::GoalCompleted {
            success: Some(success),
        })
    }

    pub fn session_info() -> Self {
        Self::new(Predicate::SessionInfo)
    }

    pub fn raw(channel: impl Into<String>, needle: impl Into<String>) -> Self {
        Self::new(Predicate::Raw {
            channel: Some(channel.into()),
            needle: needle.into(),
        })
    }

    pub fn matches(&self, event: &Event, transcript: &Transcript, since: usize) -> bool {
        self.predicate.matches(event, transcript, since)
    }
}

impl fmt::Display for Expect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.predicate)
    }
}

#[derive(Debug, Clone)]
pub enum Predicate {
    TextContains(String),
    TextRegex(Regex),
    ThinkingContains(String),
    ToolCall {
        name: Option<String>,
        input_contains: Option<String>,
    },
    ToolResult {
        name: Option<String>,
        is_error: Option<bool>,
    },
    TurnComplete,
    Done,
    ProcessExit,
    Permission,
    /// Harness hook: name/phase optional; `finished` None=any, Some(false)=started, Some(true)=finished.
    Hook {
        name: Option<String>,
        phase: Option<String>,
        finished: Option<bool>,
    },
    PlanPresented {
        summary_contains: Option<String>,
    },
    PlanResolved {
        approved: Option<bool>,
    },
    GoalStarted {
        title_contains: Option<String>,
    },
    GoalProgress,
    GoalCompleted {
        success: Option<bool>,
    },
    SessionInfo,
    Raw {
        channel: Option<String>,
        needle: String,
    },
    Any(Vec<Predicate>),
    All(Vec<Predicate>),
}

impl fmt::Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Predicate::TextContains(s) => write!(f, "text contains {s:?}"),
            Predicate::TextRegex(r) => write!(f, "text ~ /{r}/"),
            Predicate::ThinkingContains(s) => write!(f, "thinking contains {s:?}"),
            Predicate::ToolCall {
                name,
                input_contains,
            } => write!(f, "tool call name={name:?} input~={input_contains:?}"),
            Predicate::ToolResult { name, is_error } => {
                write!(f, "tool result name={name:?} error={is_error:?}")
            }
            Predicate::TurnComplete => write!(f, "turn complete"),
            Predicate::Done => write!(f, "done"),
            Predicate::ProcessExit => write!(f, "process exit"),
            Predicate::Permission => write!(f, "permission request"),
            Predicate::Hook {
                name,
                phase,
                finished,
            } => write!(
                f,
                "hook name={name:?} phase={phase:?} finished={finished:?}"
            ),
            Predicate::PlanPresented { summary_contains } => {
                write!(f, "plan presented ~{summary_contains:?}")
            }
            Predicate::PlanResolved { approved } => {
                write!(f, "plan resolved approved={approved:?}")
            }
            Predicate::GoalStarted { title_contains } => {
                write!(f, "goal started ~{title_contains:?}")
            }
            Predicate::GoalProgress => write!(f, "goal progress"),
            Predicate::GoalCompleted { success } => write!(f, "goal completed success={success:?}"),
            Predicate::SessionInfo => write!(f, "session info"),
            Predicate::Raw { channel, needle } => {
                write!(f, "raw channel={channel:?} contains {needle:?}")
            }
            Predicate::Any(ps) => write!(f, "any({} preds)", ps.len()),
            Predicate::All(ps) => write!(f, "all({} preds)", ps.len()),
        }
    }
}

impl Predicate {
    pub fn matches(&self, event: &Event, transcript: &Transcript, since: usize) -> bool {
        match self {
            // Current-turn only: event delta, cursor window, and turn_text.
            // turn_text is safe because Session::prompt always begin_turn() — do not
            // use full transcript (that would re-match prior turns).
            Predicate::TextContains(s) => {
                event
                    .as_text_delta()
                    .is_some_and(|t| t.contains(s.as_str()))
                    || transcript.text_since(since).contains(s.as_str())
                    || transcript.turn_text().contains(s.as_str())
            }
            Predicate::TextRegex(re) => {
                event.as_text_delta().is_some_and(|t| re.is_match(t))
                    || re.is_match(&transcript.text_since(since))
                    || re.is_match(transcript.turn_text())
            }
            Predicate::ThinkingContains(s) => match event {
                Event::ThinkingDelta { text } => text.contains(s.as_str()),
                _ => {
                    transcript.thinking_since(since).contains(s.as_str())
                        || transcript.turn_thinking().contains(s.as_str())
                }
            },
            Predicate::ToolCall {
                name,
                input_contains,
            } => match event {
                Event::ToolCall { name: n, input, .. } => {
                    name.as_ref().is_none_or(|want| want == n)
                        && input_contains
                            .as_ref()
                            .is_none_or(|frag| value_contains(input, frag))
                }
                _ => false,
            },
            Predicate::ToolResult { name, is_error } => match event {
                Event::ToolResult {
                    name: n,
                    is_error: err,
                    ..
                } => {
                    name.as_ref().is_none_or(|want| want == n)
                        && is_error.is_none_or(|want| want == *err)
                }
                _ => false,
            },
            Predicate::TurnComplete => matches!(event, Event::TurnComplete { .. }),
            Predicate::Done => matches!(event, Event::Done { .. }),
            Predicate::ProcessExit => {
                matches!(event, Event::ProcessExit { .. } | Event::Done { .. })
            }
            Predicate::Permission => matches!(event, Event::PermissionRequest { .. }),
            Predicate::Hook {
                name,
                phase,
                finished,
            } => match event {
                Event::HookStarted {
                    name: n, phase: p, ..
                } => {
                    finished.is_none_or(|f| !f)
                        && name.as_ref().is_none_or(|want| want == n)
                        && phase.as_ref().is_none_or(|want| {
                            p.as_ref().is_some_and(|pp| pp.eq_ignore_ascii_case(want))
                        })
                }
                Event::HookFinished {
                    name: n, phase: p, ..
                } => {
                    finished.is_none_or(|f| f)
                        && name.as_ref().is_none_or(|want| want == n)
                        && phase.as_ref().is_none_or(|want| {
                            p.as_ref().is_some_and(|pp| pp.eq_ignore_ascii_case(want))
                        })
                }
                _ => false,
            },
            Predicate::PlanPresented { summary_contains } => match event {
                Event::PlanPresented { summary, .. } => summary_contains
                    .as_ref()
                    .is_none_or(|n| summary.contains(n.as_str())),
                _ => false,
            },
            Predicate::PlanResolved { approved } => match event {
                Event::PlanResolved { approved: a, .. } => approved.is_none_or(|want| want == *a),
                _ => false,
            },
            Predicate::GoalStarted { title_contains } => match event {
                Event::GoalStarted { title, .. } => title_contains
                    .as_ref()
                    .is_none_or(|n| title.contains(n.as_str())),
                _ => false,
            },
            Predicate::GoalProgress => matches!(event, Event::GoalProgress { .. }),
            Predicate::GoalCompleted { success } => match event {
                Event::GoalCompleted { success: s, .. } => success.is_none_or(|want| want == *s),
                _ => false,
            },
            Predicate::SessionInfo => matches!(event, Event::SessionInfo { .. }),
            Predicate::Raw { channel, needle } => match event {
                Event::Raw { channel: ch, line } => {
                    channel.as_ref().is_none_or(|want| want == ch) && line.contains(needle.as_str())
                }
                _ => false,
            },
            Predicate::Any(preds) => preds.iter().any(|p| p.matches(event, transcript, since)),
            Predicate::All(preds) => preds.iter().all(|p| p.matches(event, transcript, since)),
        }
    }
}

fn value_contains(v: &Value, frag: &str) -> bool {
    match v {
        Value::String(s) => s.contains(frag),
        Value::Array(a) => a.iter().any(|x| value_contains(x, frag)),
        Value::Object(m) => m
            .iter()
            .any(|(k, x)| k.contains(frag) || value_contains(x, frag)),
        other => other.to_string().contains(frag),
    }
}

/// Builder helpers re-exported at crate root for fluent scripts.
pub mod prelude {
    pub use super::Expect;
}
