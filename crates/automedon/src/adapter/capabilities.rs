//! Capability bits: features this adapter's driver implements (prepare/parse/control).
//! False means Automedon does not implement the control path — not "author lacked a key".

use serde::{Deserialize, Serialize};

/// Per-adapter feature flags.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capabilities {
    pub launch: bool,
    pub multi_turn: bool,
    pub stream_tools: bool,
    pub wait_hooks: bool,
    pub permissions_preflight: bool,
    pub permissions_interactive: bool,
    pub plan_mode: bool,
    pub goals: bool,
    pub subagents: bool,
    pub sessions: bool,
    pub acp: bool,
    pub tool_filter: bool,
    pub worktree: bool,
    /// In-process mock (never a product harness).
    pub in_process: bool,
    /// Legacy aliases used by older call sites.
    pub streaming_json: bool,
    pub yolo: bool,
    pub permissions: bool,
    pub plans: bool,
    pub hooks: bool,
}

impl Capabilities {
    /// Headless product baseline: launch, multi-turn sessions, stream JSON, preflight yolo.
    /// No interactive mid-flight permission/plan unless the adapter implements encode.
    pub fn product_headless() -> Self {
        Self {
            launch: true,
            multi_turn: true,
            stream_tools: true,
            sessions: true,
            streaming_json: true,
            yolo: true,
            tool_filter: true,
            permissions_preflight: true,
            permissions: false,
            permissions_interactive: false,
            ..Default::default()
        }
    }
}
