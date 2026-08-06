//! Aider specialized driver — multi-turn via chat history file restore.
//!
//! Aider’s non-interactive `--message` path processes one prompt per process.
//! Continuity across Automedon turns uses:
//! `--chat-history-file <path>` and, on turn ≥ 2, `--restore-chat-history`.

use std::path::PathBuf;

use super::{base_env, resolve_bin, Adapter, Capabilities, PreparedLaunch, TurnContext};
use crate::config::LaunchOptions;
use crate::error::Result;
use crate::event::Event;
use crate::transport::SpawnSpec;

#[derive(Debug, Default, Clone)]
pub struct AiderAdapter;

impl Adapter for AiderAdapter {
    fn name(&self) -> &'static str {
        "aider"
    }

    fn capabilities(&self) -> Capabilities {
        // Live-proven with xAI: launch, multi-turn (history restore), preflight yolo.
        Capabilities {
            launch: true,
            multi_turn: true,
            sessions: true,
            yolo: true,
            permissions_preflight: true,
            permissions: false,
            permissions_interactive: false,
            ..Default::default()
        }
    }

    fn prepare(
        &self,
        prompt: &str,
        opts: &LaunchOptions,
        ctx: &TurnContext,
    ) -> Result<PreparedLaunch> {
        let program = resolve_bin(opts, "aider");

        // Stable history file for this Automedon session (path doubles as session id).
        let history = chat_history_path(opts, ctx);
        let history_str = history.display().to_string();

        let mut args = vec![
            "--message".into(),
            prompt.to_string(),
            "--yes-always".into(),
            "--no-stream".into(),
            "--no-pretty".into(),
            "--chat-history-file".into(),
            history_str.clone(),
        ];
        // Restore prior turns once we have history (turn ≥ 2).
        if ctx.turn > 1 {
            args.push("--restore-chat-history".into());
        }

        if let Some(model) = &opts.model {
            args.push("--model".into());
            args.push(model.clone());
        } else if let Some(model) = opts.extra.get("model").and_then(|v| v.as_str()) {
            args.push("--model".into());
            args.push(model.into());
        }
        if let Some(base) = opts.extra.get("openai_api_base").and_then(|v| v.as_str()) {
            args.push("--openai-api-base".into());
            args.push(base.into());
        }
        if opts
            .extra
            .get("no_git")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
        {
            args.push("--no-git".into());
        }

        let mut env = base_env(opts);
        if let Some(key) = opts.extra.get("xai_api_key").and_then(|v| v.as_str()) {
            env.insert("XAI_API_KEY".into(), key.into());
        }
        if let Some(key) = opts.extra.get("openai_api_key").and_then(|v| v.as_str()) {
            env.insert("OPENAI_API_KEY".into(), key.into());
        }

        // Seed SessionInfo so the next turn reuses this history path as session_id.
        let synthetic = if ctx.session_id.as_ref() != Some(&history_str) {
            Some(vec![Event::SessionInfo {
                id: history_str,
                label: Some("aider-chat-history".into()),
            }])
        } else {
            None
        };

        Ok(PreparedLaunch {
            harness: "aider".into(),
            spawn: Some(SpawnSpec {
                program,
                args,
                cwd: opts.cwd.clone(),
                env,
                retain_stdin: false,
            }),
            synthetic,
            capabilities: self.capabilities(),
            multi_turn: true,
        })
    }

    fn parse_line(&self, line: &str) -> Vec<Event> {
        let line = line.trim();
        if line.is_empty() {
            return Vec::new();
        }
        // Skip noise that breaks expect on pure assistant content.
        if line.starts_with("Aider v")
            || line.starts_with("Model:")
            || line.starts_with("Git repo:")
            || line.starts_with("Repo-map:")
            || line.starts_with("Tokens:")
            || line.starts_with("Cost:")
            || line.starts_with("Warning:")
        {
            return vec![Event::Raw {
                channel: "aider".into(),
                line: line.to_string(),
            }];
        }
        vec![Event::TextDelta {
            text: format!("{line}\n"),
        }]
    }
}

fn chat_history_path(opts: &LaunchOptions, ctx: &TurnContext) -> PathBuf {
    if let Some(p) = opts
        .extra
        .get("chat_history_file")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
    {
        return PathBuf::from(p);
    }
    if let Some(id) = ctx.session_id.as_ref().filter(|s| !s.is_empty()) {
        // Prior turn stored the history path as session id.
        if id.contains('/') || id.ends_with(".md") || id.contains('\\') {
            return PathBuf::from(id);
        }
    }
    // Per-session file under temp (or cwd if provided).
    let dir = opts.cwd.clone().unwrap_or_else(std::env::temp_dir);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    dir.join(format!(".automedon-aider-{stamp}.chat.history.md"))
}
