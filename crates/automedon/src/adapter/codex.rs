//! OpenAI Codex CLI specialized driver.
//!
//! `codex exec --json` NDJSON; multi-turn via `codex exec resume <thread_id>`.
//! Optional ACP prepare via `extra.acp`.

use serde_json::Value;

use super::{
    base_env, resolve_bin, shared_parse, Adapter, Capabilities, PreparedLaunch, TurnContext,
};
use crate::config::LaunchOptions;
use crate::error::Result;
use crate::event::Event;
use crate::transport::SpawnSpec;

#[derive(Debug, Default, Clone)]
pub struct CodexAdapter;

impl Adapter for CodexAdapter {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            launch: true,
            multi_turn: true,
            stream_tools: true,
            wait_hooks: true,
            hooks: true,
            sessions: true,
            streaming_json: true,
            yolo: true,
            permissions_preflight: true,
            // ACP via community @agentclientprotocol/codex-acp is prepare-only until live-proven.
            acp: false,
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
        if opts
            .extra
            .get("acp")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            return Err(crate::error::Error::Other(
                "codex ACP is not implemented for live drive (community package path removed; use exec --json)"
                    .into(),
            ));
        }

        let program = resolve_bin(opts, "codex");
        // First turn: `codex exec --json [opts] <prompt>`
        // Multi-turn: `codex exec resume --json [opts] <session_id|--last> <prompt>`
        // (see `codex exec resume --help`)
        let mut args = vec!["exec".into()];
        if ctx.turn > 1 {
            args.push("resume".into());
        }
        args.push("--json".into());
        if opts.yolo {
            args.push("--dangerously-bypass-approvals-and-sandbox".into());
        }
        if let Some(model) = &opts.model {
            args.push("--model".into());
            args.push(model.clone());
        } else if let Some(model) = opts.extra.get("model").and_then(|v| v.as_str()) {
            args.push("--model".into());
            args.push(model.into());
        }
        if ctx.turn <= 1 {
            if let Some(cwd) = &opts.cwd {
                args.push("--cd".into());
                args.push(cwd.display().to_string());
            }
            if let Some(sandbox) = opts.extra.get("sandbox").and_then(|v| v.as_str()) {
                args.push("--sandbox".into());
                args.push(sandbox.into());
            }
        }
        if ctx.turn > 1 {
            if let Some(id) = ctx.session_id.as_ref().filter(|s| !s.is_empty()) {
                args.push(id.clone());
            } else {
                args.push("--last".into());
            }
        }
        args.push(prompt.to_string());

        Ok(PreparedLaunch {
            harness: "codex".into(),
            spawn: Some(SpawnSpec {
                program,
                args,
                cwd: opts.cwd.clone(),
                env: base_env(opts),
                retain_stdin: false,
            }),
            synthetic: None,
            capabilities: self.capabilities(),
            multi_turn: true,
        })
    }

    fn parse_line(&self, line: &str) -> Vec<Event> {
        parse_agent_ndjson(line, "codex")
    }
}

pub(crate) fn parse_agent_ndjson(line: &str, channel: &str) -> Vec<Event> {
    let line = line.trim();
    if line.is_empty() {
        return Vec::new();
    }
    let Ok(v) = serde_json::from_str::<Value>(line) else {
        return vec![Event::Raw {
            channel: channel.into(),
            line: line.to_string(),
        }];
    };
    shared_parse::parse_common_json(&v, channel)
}
