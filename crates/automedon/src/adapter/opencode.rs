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
            wait_hooks: true,
            hooks: true,
            sessions: true,
            streaming_json: true,
            yolo: true,
            permissions_preflight: true,
            // `opencode acp` prepare kept only when live-proven; demote by default.
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
                "opencode ACP is not implemented for live drive (use run --format json)".into(),
            ));
        }

        let program = resolve_bin(opts, "opencode");

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
        match serde_json::from_str::<serde_json::Value>(json_line) {
            Ok(v) => {
                let mut events = shared_parse::parse_common_json(&v, "opencode");
                // Prefer nested part payloads when common parse only returned metadata.
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
                // Tool parts under part.type == "tool" / tool-invocation.
                // Live OpenCode often sends one tool_use frame with state.completed already set.
                if let Some(part) = v.get("part") {
                    let pty = part.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    if pty.contains("tool") || pty == "tool-invocation" {
                        let id = part
                            .get("callID")
                            .or_else(|| part.get("id"))
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string();
                        let name = part
                            .get("tool")
                            .or_else(|| part.get("name"))
                            .and_then(|x| x.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let state = part.get("state");
                        // Live: state is object `{status,input,output}`; fixtures may use string.
                        let status = state
                            .and_then(|s| s.as_str())
                            .or_else(|| {
                                state.and_then(|s| s.get("status")).and_then(|x| x.as_str())
                            })
                            .or_else(|| part.get("status").and_then(|s| s.as_str()))
                            .unwrap_or("");
                        let input = state
                            .and_then(|s| s.get("input"))
                            .or_else(|| part.get("input"))
                            .or_else(|| part.get("args"))
                            .cloned()
                            .unwrap_or(serde_json::Value::Null);
                        let completed =
                            status == "completed" || status == "error" || pty.contains("result");
                        // Drop tool_use-only start events if this frame is already complete.
                        if completed {
                            events.retain(|e| match e {
                                Event::ToolCall { .. } => false,
                                Event::HookStarted { name, .. } if name == "PreToolUse" => false,
                                _ => true,
                            });
                            let output = state
                                .and_then(|s| s.get("output"))
                                .or_else(|| part.get("output"))
                                .or_else(|| part.get("result"))
                                .map(|x| match x {
                                    serde_json::Value::String(s) => s.clone(),
                                    other => other.to_string(),
                                })
                                .unwrap_or_default();
                            let is_error = status == "error"
                                || state
                                    .and_then(|s| s.get("metadata"))
                                    .and_then(|m| m.get("exit"))
                                    .and_then(|e| e.as_i64())
                                    .is_some_and(|c| c != 0);
                            events.extend(shared_parse::tool_start_events(
                                id.clone(),
                                name.clone(),
                                input,
                                "tool",
                            ));
                            events.extend(shared_parse::tool_end_events(
                                id, name, output, is_error, "tool",
                            ));
                        } else if !events.iter().any(|e| matches!(e, Event::ToolCall { .. })) {
                            events.extend(shared_parse::tool_start_events(id, name, input, "tool"));
                        }
                    }
                }
                // Always attach sessionID when present on any frame.
                if let Some(sid) = v
                    .get("sessionID")
                    .or_else(|| v.get("sessionId"))
                    .and_then(|s| s.as_str())
                {
                    if !events
                        .iter()
                        .any(|e| matches!(e, Event::SessionInfo { id, .. } if id == sid))
                    {
                        events.insert(
                            0,
                            Event::SessionInfo {
                                id: sid.to_string(),
                                label: Some("opencode".into()),
                            },
                        );
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
