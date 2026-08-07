//! OpenCode specialized driver — `opencode run --format json` + optional `opencode acp`.
//! Session continuity: `--session <id>` / `--continue`.

use super::{
    base_env, resolve_bin, shared_parse, Adapter, Capabilities, PreparedLaunch, TurnContext,
};
use crate::config::LaunchOptions;
use crate::error::Result;
use crate::event::Event;
use crate::transport::SpawnSpec;

#[derive(Debug, Default, Clone)]
pub struct OpenCodeAdapter;

impl Adapter for OpenCodeAdapter {
    fn name(&self) -> &'static str {
        "opencode"
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

        let program = resolve_bin(opts, "opencode");
        if use_acp {
            return Ok(PreparedLaunch {
                harness: "opencode".into(),
                spawn: Some(SpawnSpec {
                    program,
                    args: vec!["acp".into()],
                    cwd: opts.cwd.clone(),
                    env: base_env(opts),
                    retain_stdin: true,
                }),
                synthetic: None,
                capabilities: self.capabilities(),
                multi_turn: true,
            });
        }

        // `opencode run "prompt" --format json`
        let mut args = vec!["run".into(), prompt.to_string()];
        if opts.yolo {
            // auto-approve agent tools when supported
            args.push("--auto".into());
        }
        if let Some(model) = &opts.model {
            args.push("--model".into());
            args.push(model.clone());
        } else if let Some(model) = opts.extra.get("model").and_then(|v| v.as_str()) {
            args.push("--model".into());
            args.push(model.into());
        }
        if ctx.turn > 1 {
            if let Some(id) = ctx.session_id.as_ref().filter(|s| !s.is_empty()) {
                args.push("--session".into());
                args.push(id.clone());
            } else {
                args.push("--continue".into());
            }
        } else if let Some(id) = opts.extra.get("session").and_then(|v| v.as_str()) {
            args.push("--session".into());
            args.push(id.into());
        }
        args.push("--format".into());
        args.push("json".into());

        Ok(PreparedLaunch {
            harness: "opencode".into(),
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
        let line = line.trim();
        if line.is_empty() {
            return Vec::new();
        }
        // Strip ANSI / color noise if a line still starts with '{'
        let json_line = if let Some(idx) = line.find('{') {
            &line[idx..]
        } else {
            line
        };
        match serde_json::from_str(json_line) {
            Ok(v) => {
                let mut events = shared_parse::parse_common_json(&v, "opencode");
                // OpenCode text often nested under part / message
                if events.iter().all(|e| {
                    matches!(
                        e,
                        Event::Raw { .. } | Event::SessionInfo { .. } | Event::TurnStart { .. }
                    )
                }) {
                    if let Some(text) = extract_opencode_text(&v) {
                        events.push(Event::TextDelta { text });
                    }
                }
                events
            }
            Err(_) => vec![Event::Raw {
                channel: "opencode".into(),
                line: line.to_string(),
            }],
        }
    }
}

fn extract_opencode_text(value: &serde_json::Value) -> Option<String> {
    if let Some(t) = value.get("text").and_then(|t| t.as_str()) {
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    if let Some(t) = value
        .pointer("/part/text")
        .and_then(|t| t.as_str())
        .filter(|s| !s.is_empty())
    {
        return Some(t.to_string());
    }
    if let Some(t) = value
        .pointer("/message/content/0/text")
        .and_then(|t| t.as_str())
        .filter(|s| !s.is_empty())
    {
        return Some(t.to_string());
    }
    None
}
