//! Claude Code adapter — headless `claude -p` with stream-json.
//! Multi-turn via `--resume` / `--continue`. Parses system init, tools, and hooks.

use serde_json::Value;

use super::{
    base_env, resolve_bin, shared_parse, Adapter, Capabilities, PreparedLaunch, TurnContext,
};
use crate::config::LaunchOptions;
use crate::error::Result;
use crate::event::Event;
use crate::transport::SpawnSpec;

#[derive(Debug, Default, Clone)]
pub struct ClaudeAdapter;

impl Adapter for ClaudeAdapter {
    fn name(&self) -> &'static str {
        "claude"
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
            tool_filter: true,
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
        let program = resolve_bin(opts, "claude");
        // Prefer print mode: -p / --print are equivalent on recent Claude Code.
        // --print/-p + stream-json; --include-hook-events exposes hook lifecycle on the stream.
        let mut args = vec![
            "-p".into(),
            prompt.to_string(),
            "--output-format".into(),
            "stream-json".into(),
            "--verbose".into(),
            "--include-hook-events".into(),
        ];

        // Multi-turn: --resume <id> when we have a session; else --continue on turn ≥ 2.
        if ctx.turn > 1 {
            if let Some(id) = ctx.session_id.as_ref().filter(|s| !s.is_empty()) {
                args.push("--resume".into());
                args.push(id.clone());
            } else if opts
                .extra
                .get("continue")
                .and_then(|v| v.as_bool())
                .unwrap_or(true)
            {
                args.push("--continue".into());
            }
        } else if let Some(id) = opts
            .extra
            .get("resume")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        {
            args.push("--resume".into());
            args.push(id.into());
        }

        if opts.yolo {
            args.push("--dangerously-skip-permissions".into());
        }
        if let Some(mode) = opts.extra.get("permission_mode").and_then(|v| v.as_str()) {
            args.push("--permission-mode".into());
            args.push(mode.into());
        }
        if let Some(model) = &opts.model {
            args.push("--model".into());
            args.push(model.clone());
        }
        if let Some(v) = opts.extra.get("max_turns").and_then(|v| v.as_u64()) {
            args.push("--max-turns".into());
            args.push(v.to_string());
        }
        if let Some(tools) = opts.extra.get("allowed_tools").and_then(|v| v.as_str()) {
            args.push("--allowedTools".into());
            args.push(tools.into());
        }
        // Claude Code hooks: config lives in settings; optional path override.
        if let Some(settings) = opts.extra.get("settings").and_then(|v| v.as_str()) {
            args.push("--settings".into());
            args.push(settings.into());
        }
        if let Some(sid) = opts.extra.get("session_id").and_then(|v| v.as_str()) {
            args.push("--session-id".into());
            args.push(sid.into());
        }

        Ok(PreparedLaunch {
            harness: "claude".into(),
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
        match serde_json::from_str::<Value>(line) {
            Ok(v) => self.parse_json(&v),
            Err(_) => vec![Event::Raw {
                channel: "stdout".into(),
                line: line.to_string(),
            }],
        }
    }

    fn parse_json(&self, value: &Value) -> Vec<Event> {
        let ty = value
            .get("type")
            .or_else(|| value.get("event"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        match ty {
            "system" => {
                // subtype init carries session_id even when not logged in
                let mut out = Vec::new();
                if let Some(sid) = value
                    .get("session_id")
                    .or_else(|| value.get("sessionId"))
                    .and_then(|v| v.as_str())
                {
                    out.push(Event::SessionInfo {
                        id: sid.to_string(),
                        label: Some("claude".into()),
                    });
                }
                if out.is_empty() {
                    out.push(Event::Raw {
                        channel: "claude".into(),
                        line: value.to_string(),
                    });
                }
                out
            }
            "assistant" | "content_block_delta" => {
                let mut out = Vec::new();
                if let Some(sid) = value
                    .get("session_id")
                    .or_else(|| value.get("sessionId"))
                    .and_then(|v| v.as_str())
                {
                    out.push(Event::SessionInfo {
                        id: sid.to_string(),
                        label: Some("claude".into()),
                    });
                }
                if let Some(delta) = value.get("delta") {
                    if delta.get("type").and_then(|t| t.as_str()) == Some("text_delta") {
                        let text = delta
                            .get("text")
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        if !text.is_empty() {
                            out.push(Event::TextDelta { text });
                        }
                    }
                    if delta.get("type").and_then(|t| t.as_str()) == Some("thinking_delta") {
                        let text = delta
                            .get("thinking")
                            .or_else(|| delta.get("text"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        if !text.is_empty() {
                            out.push(Event::ThinkingDelta { text });
                        }
                    }
                }
                if let Some(message) = value.get("message") {
                    out.extend(content_blocks_to_events(message));
                }
                // Auth-fail synthetic assistant text (still useful for Error path)
                if out.iter().all(|e| matches!(e, Event::SessionInfo { .. })) {
                    if let Some(msg) = value
                        .get("message")
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|b| b.get("text"))
                        .and_then(|t| t.as_str())
                    {
                        if msg.contains("Not logged in") || msg.contains("authentication") {
                            out.push(Event::Error {
                                message: msg.to_string(),
                            });
                        }
                    }
                }
                if out.is_empty() {
                    out.push(Event::Raw {
                        channel: "claude".into(),
                        line: value.to_string(),
                    });
                }
                out
            }
            "result" => {
                let mut events = Vec::new();
                if let Some(sid) = value
                    .get("session_id")
                    .or_else(|| value.get("sessionId"))
                    .and_then(|v| v.as_str())
                {
                    events.push(Event::SessionInfo {
                        id: sid.to_string(),
                        label: Some("claude".into()),
                    });
                }
                if let Some(text) = value.get("result").and_then(|v| v.as_str()) {
                    if !text.is_empty() {
                        if text.contains("Not logged in")
                            || value.get("is_error") == Some(&Value::Bool(true))
                        {
                            events.push(Event::Error {
                                message: text.to_string(),
                            });
                        } else {
                            events.push(Event::TextDelta {
                                text: text.to_string(),
                            });
                        }
                    }
                }
                // Turn end only — never session Done (multi-turn reuses Session across processes).
                events.push(Event::TurnComplete {
                    turn: value.get("num_turns").and_then(|v| v.as_u64()).unwrap_or(1),
                    stop_reason: value
                        .get("subtype")
                        .or_else(|| value.get("stop_reason"))
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                });
                events
            }
            "user" => {
                // tool_result blocks sometimes appear as user messages
                if let Some(message) = value.get("message") {
                    return content_blocks_to_events(message);
                }
                Vec::new()
            }
            "tool_use" | "tool_call" => {
                let id = value
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = value
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let input = value.get("input").cloned().unwrap_or(Value::Null);
                // HookStarted first so wait_hook_started then wait_tool works
                // with multi-event re-scan (same order as Pi).
                vec![
                    Event::HookStarted {
                        id: id.clone(),
                        name: "PreToolUse".into(),
                        phase: Some(ty.to_string()),
                        detail: Some(serde_json::json!({ "tool": name, "input": input })),
                    },
                    Event::ToolCall { id, name, input },
                ]
            }
            "tool_result" => {
                let id = value
                    .get("tool_use_id")
                    .or_else(|| value.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let is_error = value
                    .get("is_error")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let output = value
                    .get("content")
                    .or_else(|| value.get("output"))
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .unwrap_or_default();
                vec![
                    Event::ToolResult {
                        id: id.clone(),
                        name: value
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        output: output.clone(),
                        is_error,
                    },
                    Event::HookFinished {
                        id,
                        name: "PostToolUse".into(),
                        phase: Some("tool_result".into()),
                        ok: !is_error,
                        detail: Some(output),
                    },
                ]
            }
            "error" => vec![Event::Error {
                message: value
                    .get("error")
                    .or_else(|| value.get("message"))
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .unwrap_or_else(|| value.to_string()),
            }],
            // Hook stream events when present
            "hook_started" | "hook_start" | "hook" => {
                shared_parse::parse_common_json(value, "claude")
            }
            "hook_finished" | "hook_end" => shared_parse::parse_common_json(value, "claude"),
            _ => shared_parse::parse_common_json(value, "claude"),
        }
    }
}

fn content_blocks_to_events(message: &Value) -> Vec<Event> {
    let mut out = Vec::new();
    let Some(content) = message.get("content").and_then(|c| c.as_array()) else {
        return out;
    };
    for block in content {
        match block.get("type").and_then(|t| t.as_str()) {
            Some("text") => {
                if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                    if !t.is_empty() {
                        out.push(Event::TextDelta {
                            text: t.to_string(),
                        });
                    }
                }
            }
            Some("tool_use") => {
                // Same general lifecycle as Pi: PreToolUse then ToolCall.
                // Live stream-json puts tools in assistant content blocks, not
                // top-level tool_use frames; native hook_* lines only appear when
                // settings define hooks.
                let id = block
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = block
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let input = block.get("input").cloned().unwrap_or(Value::Null);
                out.push(Event::HookStarted {
                    id: id.clone(),
                    name: "PreToolUse".into(),
                    phase: Some("tool_use".into()),
                    detail: Some(serde_json::json!({ "tool": name, "input": input })),
                });
                out.push(Event::ToolCall { id, name, input });
            }
            Some("tool_result") => {
                let id = block
                    .get("tool_use_id")
                    .or_else(|| block.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let output = block
                    .get("content")
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .unwrap_or_default();
                let is_error = block
                    .get("is_error")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                out.push(Event::ToolResult {
                    id: id.clone(),
                    name: String::new(),
                    output: output.clone(),
                    is_error,
                });
                out.push(Event::HookFinished {
                    id,
                    name: "PostToolUse".into(),
                    phase: Some("tool_result".into()),
                    ok: !is_error,
                    detail: Some(output),
                });
            }
            _ => {}
        }
    }
    out
}
