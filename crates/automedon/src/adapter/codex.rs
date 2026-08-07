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
            sessions: true,
            streaming_json: true,
            yolo: true,
            permissions_preflight: true,
            acp: true,
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
        let use_acp = opts
            .extra
            .get("acp")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if use_acp {
            // Community ACP adapter package; Session uses JSON-RPC after spawn.
            let program = resolve_bin(opts, "npx");
            let mut args = vec!["-y".into(), "@agentclientprotocol/codex-acp".into()];
            if let Some(cwd) = &opts.cwd {
                args.push("--cwd".into());
                args.push(cwd.display().to_string());
            }
            let mut env = base_env(opts);
            env.insert("AUTOMEDON_ACP_PROMPT".into(), prompt.to_string());
            return Ok(PreparedLaunch {
                harness: "codex".into(),
                spawn: Some(SpawnSpec {
                    program,
                    args,
                    cwd: opts.cwd.clone(),
                    env,
                    retain_stdin: true,
                }),
                synthetic: None,
                capabilities: self.capabilities(),
                multi_turn: true,
            });
        }

        let program = resolve_bin(opts, "codex");
        // Multi-turn: `codex exec resume <id> --json [prompt]`
        // First turn: `codex exec --json <prompt>`
        let mut args = vec!["exec".into()];
        if ctx.turn > 1 {
            if let Some(id) = ctx.session_id.as_ref().filter(|s| !s.is_empty()) {
                args.push("resume".into());
                args.push(id.clone());
            }
        }
        // Flags before positional prompt when not using resume's default last message.
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
        if let Some(cwd) = &opts.cwd {
            args.push("--cd".into());
            args.push(cwd.display().to_string());
        }
        if let Some(sandbox) = opts.extra.get("sandbox").and_then(|v| v.as_str()) {
            args.push("--sandbox".into());
            args.push(sandbox.into());
        }
        // Prompt last (resume can omit to continue last; we always send explicit prompt).
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
