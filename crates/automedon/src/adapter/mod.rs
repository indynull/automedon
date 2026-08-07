//! Harness adapters: map each CLI's quirks into the normalized event model.
//!
//! Specialized adapters: one module per product harness.
//! `mock` is test infrastructure only; `generic` is an escape hatch.

pub mod acp;
mod aider;
mod capabilities;
mod claude;
mod codex;
mod copilot;
mod cursor;
mod gemini;
mod generic;
mod grok;
mod mock;
mod opencode;
mod pi;
mod registry;
pub mod shared_parse;

use serde_json::Value;

use crate::config::LaunchOptions;
use crate::error::Result;
use crate::event::Event;
use crate::transport::SpawnSpec;

pub use aider::AiderAdapter;
pub use capabilities::Capabilities;
pub use claude::ClaudeAdapter;
pub use codex::CodexAdapter;
pub use copilot::CopilotAdapter;
pub use cursor::CursorAdapter;
pub use gemini::GeminiAdapter;
pub use generic::GenericAdapter;
pub use grok::{grok_prepare_args, GrokAdapter};
pub use mock::{mock_permission_continue, mock_plan_continue, MockAdapter};
pub use opencode::OpenCodeAdapter;
pub use pi::{pi_prepare_args, PiAdapter};
pub use registry::{known_names, product_names, registry, resolve, AdapterKind};

/// Context carried across multi-turn prompts on one Automedon session.
#[derive(Debug, Clone, Default)]
pub struct TurnContext {
    /// 1-based turn number about to run.
    pub turn: u64,
    /// Harness session id from a prior turn (resume/continue).
    pub session_id: Option<String>,
    /// Prior user prompts in this Automedon session.
    pub history_prompts: Vec<String>,
    /// Accumulated assistant text so far (mock continuity).
    pub history_text: String,
    /// Pending permission id awaiting approve/deny (mock).
    pub pending_permission: Option<String>,
    /// Pending plan id awaiting approve/deny (mock).
    pub pending_plan: Option<String>,
}

/// Built command line + how to parse each stdout line.
pub struct PreparedLaunch {
    pub harness: String,
    pub spawn: Option<SpawnSpec>,
    /// In-process event sequence for mock / synthetic adapters.
    pub synthetic: Option<Vec<Event>>,
    pub capabilities: Capabilities,
    /// When true, process exit ends only the turn — not the whole session.
    pub multi_turn: bool,
}

/// Adapter contract: prepare a launch and parse harness-native lines.
pub trait Adapter: Send + Sync {
    fn name(&self) -> &'static str;

    fn capabilities(&self) -> Capabilities;

    /// Prepare the first or a follow-up turn.
    fn prepare(
        &self,
        prompt: &str,
        opts: &LaunchOptions,
        ctx: &TurnContext,
    ) -> Result<PreparedLaunch>;

    /// Parse one stdout line into zero or more normalized events.
    fn parse_line(&self, line: &str) -> Vec<Event>;

    /// Optional: map structured JSON objects (already parsed).
    fn parse_json(&self, value: &Value) -> Vec<Event> {
        let _ = value;
        Vec::new()
    }

    /// Encode a permission decision for the live child (stdin line), if supported.
    fn encode_permission(&self, id: &str, allowed: bool) -> Option<String> {
        let _ = (id, allowed);
        None
    }

    /// Encode a plan approve/deny for the live child, if supported.
    fn encode_plan_resolve(&self, id: &str, approved: bool) -> Option<String> {
        let _ = (id, approved);
        None
    }

    /// Extract session id from a parsed event when the harness publishes one.
    fn session_id_from_event(&self, event: &Event) -> Option<String> {
        match event {
            Event::SessionInfo { id, .. } => Some(id.clone()),
            _ => None,
        }
    }
}

/// Shared helper: resolve binary from opts or PATH name.
pub(crate) fn resolve_bin(opts: &LaunchOptions, default_name: &str) -> std::path::PathBuf {
    opts.bin
        .clone()
        .unwrap_or_else(|| std::path::PathBuf::from(default_name))
}

pub(crate) fn base_env(opts: &LaunchOptions) -> std::collections::BTreeMap<String, String> {
    let mut env = opts.env.clone();
    env.entry("NO_COLOR".into()).or_insert_with(|| "1".into());
    env.entry("FORCE_COLOR".into())
        .or_insert_with(|| "0".into());
    env
}
