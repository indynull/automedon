//! **Wait** handlers for harness stream events.
//!
//! Block until a
//! condition matches the live (or buffered) event stream — tools, hooks, text,
//! permissions, plan/goal, turn complete, etc.
//!
//! ```no_run
//! use automedon::{Session, Wait};
//! # async fn demo(s: &mut Session) -> automedon::Result<()> {
//! s.wait(Wait::hook("PreToolUse")).await?;
//! s.wait(Wait::tool("run_terminal_command")).await?;
//! s.wait(Wait::hook_finished("PostToolUse").timeout(std::time::Duration::from_secs(60))).await?;
//! s.wait(Wait::any([Wait::permission(), Wait::text("DONE")])).await?;
//! # Ok(())
//! # }
//! ```

use std::fmt;
use std::time::Duration;

use crate::error::{Error, Result};
use crate::event::{Event, Transcript};
use crate::expect::{Expect, Predicate};

/// Explicit wait (timeout + condition).
#[derive(Debug, Clone)]
pub struct Wait {
    pub condition: WaitCondition,
    pub timeout: Duration,
}

/// What to wait for on the harness event stream.
#[derive(Debug, Clone)]
pub enum WaitCondition {
    /// Single stream predicate (text, tool, hook, …).
    On(Predicate),
    /// First of several conditions.
    Any(Vec<WaitCondition>),
    /// All conditions must match (against the same event, or cumulative
    /// transcript window since the wait started — see [`WaitCondition::matches`]).
    All(Vec<WaitCondition>),
}

impl Wait {
    pub fn new(condition: WaitCondition) -> Self {
        Self {
            condition,
            timeout: Duration::from_secs(120),
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// From an existing [`Expect`] (shares the same predicates).
    pub fn on(exp: Expect) -> Self {
        Self {
            condition: WaitCondition::On(exp.predicate),
            timeout: exp.timeout,
        }
    }

    pub fn any(waits: impl IntoIterator<Item = Wait>) -> Self {
        let mut timeout = Duration::from_secs(120);
        let mut conds = Vec::new();
        for w in waits {
            timeout = timeout.max(w.timeout);
            conds.push(w.condition);
        }
        Self {
            condition: WaitCondition::Any(conds),
            timeout,
        }
    }

    pub fn all(waits: impl IntoIterator<Item = Wait>) -> Self {
        let mut timeout = Duration::from_secs(120);
        let mut conds = Vec::new();
        for w in waits {
            timeout = timeout.max(w.timeout);
            conds.push(w.condition);
        }
        Self {
            condition: WaitCondition::All(conds),
            timeout,
        }
    }

    // --- convenience constructors (DOM-wait analog) ---

    pub fn text(needle: impl Into<String>) -> Self {
        Self::on(Expect::text(needle))
    }

    pub fn thinking(needle: impl Into<String>) -> Self {
        Self::on(Expect::thinking(needle))
    }

    pub fn tool(name: impl Into<String>) -> Self {
        Self::on(Expect::tool(name))
    }

    pub fn tool_any() -> Self {
        Self::on(Expect::tool_any())
    }

    pub fn tool_result(name: impl Into<String>) -> Self {
        Self::on(Expect::tool_result(name))
    }

    pub fn permission() -> Self {
        Self::on(Expect::permission())
    }

    pub fn hook(name: impl Into<String>) -> Self {
        Self::on(Expect::hook(name))
    }

    pub fn hook_any() -> Self {
        Self::on(Expect::hook_any())
    }

    pub fn hook_started(name: impl Into<String>) -> Self {
        Self::on(Expect::hook_started(name))
    }

    pub fn hook_finished(name: impl Into<String>) -> Self {
        Self::on(Expect::hook_finished(name))
    }

    pub fn hook_phase(name: impl Into<String>, phase: impl Into<String>) -> Self {
        Self::on(Expect::hook_phase(name, phase))
    }

    pub fn plan() -> Self {
        Self::on(Expect::plan())
    }

    pub fn plan_summary(needle: impl Into<String>) -> Self {
        Self::on(Expect::plan_summary(needle))
    }

    pub fn plan_resolved(approved: bool) -> Self {
        Self::on(Expect::plan_resolved(approved))
    }

    pub fn goal() -> Self {
        Self::on(Expect::goal())
    }

    pub fn goal_title(needle: impl Into<String>) -> Self {
        Self::on(Expect::goal_title(needle))
    }

    pub fn goal_progress() -> Self {
        Self::on(Expect::goal_progress())
    }

    pub fn goal_completed(success: bool) -> Self {
        Self::on(Expect::goal_completed(success))
    }

    pub fn turn_complete() -> Self {
        Self::on(Expect::turn_complete())
    }

    pub fn done() -> Self {
        Self::on(Expect::done())
    }

    pub fn process_exit() -> Self {
        Self::on(Expect::process_exit())
    }

    pub fn session_info() -> Self {
        Self::on(Expect::session_info())
    }

    pub fn raw(channel: impl Into<String>, needle: impl Into<String>) -> Self {
        Self::on(Expect::raw(channel, needle))
    }

    pub fn matches(&self, event: &Event, transcript: &Transcript, since: usize) -> bool {
        self.condition.matches(event, transcript, since)
    }

    pub fn into_expect(self) -> Expect {
        Expect {
            predicate: self.condition.into_predicate(),
            timeout: self.timeout,
        }
    }
}

impl fmt::Display for Wait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wait({}) timeout={:?}", self.condition, self.timeout)
    }
}

impl WaitCondition {
    pub fn matches(&self, event: &Event, transcript: &Transcript, since: usize) -> bool {
        match self {
            WaitCondition::On(p) => p.matches(event, transcript, since),
            WaitCondition::Any(cs) => cs.iter().any(|c| c.matches(event, transcript, since)),
            // All: every sub-condition is true for this event or the window since `since`.
            WaitCondition::All(cs) => cs.iter().all(|c| c.matches(event, transcript, since)),
        }
    }

    fn into_predicate(self) -> Predicate {
        match self {
            WaitCondition::On(p) => p,
            WaitCondition::Any(cs) => {
                Predicate::Any(cs.into_iter().map(WaitCondition::into_predicate).collect())
            }
            WaitCondition::All(cs) => {
                Predicate::All(cs.into_iter().map(WaitCondition::into_predicate).collect())
            }
        }
    }
}

impl fmt::Display for WaitCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WaitCondition::On(p) => write!(f, "{p}"),
            WaitCondition::Any(cs) => write!(f, "any({} conds)", cs.len()),
            WaitCondition::All(cs) => write!(f, "all({} conds)", cs.len()),
        }
    }
}

/// Map wait failures to expect-timeout errors (same wire type for scripts).
pub fn wait_timeout(wait: &Wait) -> Error {
    Error::ExpectTimeout {
        timeout: wait.timeout,
        predicate: wait.to_string(),
    }
}

/// Validate a wait has a positive timeout.
pub fn check_wait(wait: &Wait) -> Result<()> {
    if wait.timeout.is_zero() {
        return Err(Error::Other("wait timeout must be > 0".into()));
    }
    Ok(())
}
