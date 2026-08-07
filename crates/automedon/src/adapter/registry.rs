use std::sync::Arc;

use super::{
    Adapter, AiderAdapter, ClaudeAdapter, CodexAdapter, CopilotAdapter, CursorAdapter,
    GeminiAdapter, GenericAdapter, GrokAdapter, MockAdapter, OpenCodeAdapter, PiAdapter,
};
use crate::error::{Error, Result};

/// Product adapter kinds (Tier A/B + test/escape hatch).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterKind {
    // Tier A
    Claude,
    Codex,
    Gemini,
    OpenCode,
    Grok,
    Cursor,
    // Tier B
    Aider,
    Pi,
    Copilot,
    // Infrastructure
    Mock,
    Generic,
}

impl AdapterKind {
    pub fn parse(name: &str) -> Result<Self> {
        match name.to_ascii_lowercase().as_str() {
            "claude" | "claude-code" | "anthropic" => Ok(Self::Claude),
            "codex" | "openai-codex" => Ok(Self::Codex),
            "gemini" | "gemini-cli" | "antigravity" | "agy" => Ok(Self::Gemini),
            "opencode" | "open-code" => Ok(Self::OpenCode),
            "grok" | "grok-build" | "grokos" => Ok(Self::Grok),
            "cursor" | "cursor-agent" | "cursor-cli" => Ok(Self::Cursor),
            "aider" => Ok(Self::Aider),
            "pi" | "pi-mono" => Ok(Self::Pi),
            "copilot" | "github-copilot" | "gh-copilot" => Ok(Self::Copilot),
            "mock" | "test" => Ok(Self::Mock),
            "generic" | "raw" | "custom" => Ok(Self::Generic),
            other => Err(Error::UnknownAdapter(other.into())),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::OpenCode => "opencode",
            Self::Grok => "grok",
            Self::Cursor => "cursor",
            Self::Aider => "aider",
            Self::Pi => "pi",
            Self::Copilot => "copilot",
            Self::Mock => "mock",
            Self::Generic => "generic",
        }
    }

    /// Product harnesses only (excludes mock + generic).
    pub fn is_product(self) -> bool {
        !matches!(self, Self::Mock | Self::Generic)
    }

    /// Default CLI binary names operators should have on `PATH` (display string).
    pub fn default_binaries(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini (or agy)",
            Self::OpenCode => "opencode",
            Self::Grok => "grok",
            Self::Cursor => "cursor-agent (preferred) / agent / cursor",
            Self::Aider => "aider",
            Self::Pi => "pi",
            Self::Copilot => "copilot",
            Self::Mock => "(in-process)",
            Self::Generic => "opts.bin",
        }
    }

    /// Short multi-turn mechanism note for operators.
    pub fn multi_turn_summary(self) -> &'static str {
        match self {
            Self::Claude => "--resume / --continue (stream-json)",
            Self::Codex => "exec resume <id|--last> --json",
            Self::Gemini => "-r / resume latest (stream-json)",
            Self::OpenCode => "--session / --continue (json)",
            Self::Grok => "--resume / --continue; ACP optional",
            Self::Cursor => "--resume / --continue (stream-json)",
            Self::Aider => "chat-history restore",
            Self::Pi => "--session-id / --continue (json)",
            Self::Copilot => "--resume=<id> / --continue (json)",
            Self::Mock => "in-process scenarios",
            Self::Generic => "process-per-prompt",
        }
    }
}

pub fn registry(kind: AdapterKind) -> Arc<dyn Adapter> {
    match kind {
        AdapterKind::Claude => Arc::new(ClaudeAdapter),
        AdapterKind::Codex => Arc::new(CodexAdapter),
        AdapterKind::Gemini => Arc::new(GeminiAdapter),
        AdapterKind::OpenCode => Arc::new(OpenCodeAdapter),
        AdapterKind::Grok => Arc::new(GrokAdapter),
        AdapterKind::Cursor => Arc::new(CursorAdapter),
        AdapterKind::Aider => Arc::new(AiderAdapter),
        AdapterKind::Pi => Arc::new(PiAdapter),
        AdapterKind::Copilot => Arc::new(CopilotAdapter),
        AdapterKind::Mock => Arc::new(MockAdapter),
        AdapterKind::Generic => Arc::new(GenericAdapter),
    }
}

pub fn resolve(name: &str) -> Result<Arc<dyn Adapter>> {
    Ok(registry(AdapterKind::parse(name)?))
}

/// All registered names including mock (tests) and generic (escape hatch).
pub fn known_names() -> &'static [&'static str] {
    &[
        "claude", "codex", "gemini", "opencode", "grok", "cursor", "aider", "pi", "copilot",
        "mock", "generic",
    ]
}

/// Product adapter ids only (GOAL Tier A/B).
pub fn product_names() -> &'static [&'static str] {
    &[
        "claude", "codex", "gemini", "opencode", "grok", "cursor", "aider", "pi", "copilot",
    ]
}
