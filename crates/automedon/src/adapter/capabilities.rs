//! Honest capability bits — advertise only what is proven for that adapter.

use serde::{Deserialize, Serialize};

/// Per-adapter feature flags (GOAL.md bitset).
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
    /// Baseline for offline mock-like product shaping tests — not for live product adapters.
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
            // Do not imply interactive mid-flight control.
            permissions: false,
            permissions_interactive: false,
            ..Default::default()
        }
    }
}
