//! Pi coding agent — `pi -p` with `--mode json`; multi-turn via `--continue` / `--session-id`.

use serde_json::Value;

use super::{base_env, resolve_bin, Adapter, Capabilities, PreparedLaunch, TurnContext};
use crate::config::LaunchOptions;
use crate::error::Result;
use crate::event::Event;
use crate::transport::SpawnSpec;

#[derive(Debug, Default, Clone)]
pub struct PiAdapter;

impl Adapter for PiAdapter {
    fn name(&self) -> &'static str {
        "pi"
    }

    fn capabilities(&self) -> Capabilities {
        // Multi-turn sessions; tool stream maps tool lifecycle → Tool* + Hook* (Pre/PostToolUse).
        Capabilities {
            launch: true,
            multi_turn: true,
            sessions: true,
            streaming_json: true,
            stream_tools: true,
            wait_hooks: true,
            hooks: true,
            yolo: true,
            permissions_preflight: true,
            permissions: false,
            permissions_interactive: false,
            tool_filter: true,
            ..Default::default()
        }
    }

    fn prepare(
        &self,
        prompt: &str,
        opts: &LaunchOptions,
        ctx: &TurnContext,
    ) -> Result<PreparedLaunch> {
        let program = resolve_bin(opts, "pi");
        let mut args = vec![
            "-p".into(),
            prompt.to_string(),
            "--mode".into(),
            "json".into(),
        ];

        let multi_turn = opts
            .extra
            .get("multi_turn")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if multi_turn {
            if let Some(id) = ctx.session_id.as_ref().filter(|s| !s.is_empty()) {
                if ctx.turn > 1 {
                    // Resume the same project session for continuity.
                    args.push("--session-id".into());
                    args.push(id.clone());
                    // Also pass --continue when no explicit id path (belt).
                }
            } else if ctx.turn > 1 {
                args.push("--continue".into());
            }
            // Persist session so resume works; only skip when explicitly requested.
            let no_session = opts
                .extra
                .get("no_session")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if no_session && ctx.turn <= 1 && ctx.session_id.is_none() {
                args.push("--no-session".into());
            }
        } else {
            args.push("--no-session".into());
        }

        if opts.yolo {
            args.push("--approve".into());
        }
        // Prefer explicit provider; model may be bare or "provider/model".
        if let Some(provider) = opts.extra.get("provider").and_then(|v| v.as_str()) {
            args.push("--provider".into());
            args.push(provider.into());
        }
        if let Some(model) = &opts.model {
            args.push("--model".into());
            args.push(model.clone());
        } else if let Some(model) = opts.extra.get("model").and_then(|v| v.as_str()) {
            args.push("--model".into());
            args.push(model.into());
        }
        if let Some(tools) = opts.extra.get("tools").and_then(|v| v.as_str()) {
            args.push("--tools".into());
            args.push(tools.into());
        } else {
            // Default headless tool surface for agent runs (specialized default).
            // Override with extra.tools="" to disable.
            let disable = opts
                .extra
                .get("tools")
                .and_then(|v| v.as_str())
                .is_some_and(|s| s.is_empty());
            if !disable {
                args.push("--tools".into());
                args.push("bash,read,edit,write".into());
            }
        }
        if let Some(tools) = opts.extra.get("exclude_tools").and_then(|v| v.as_str()) {
            args.push("--exclude-tools".into());
            args.push(tools.into());
        }
        if let Some(thinking) = opts.extra.get("thinking").and_then(|v| v.as_str()) {
            args.push("--thinking".into());
            args.push(thinking.into());
        }
        // Specialized: load Pi extensions (hooks/tools). General hooks observe uses stream events.
        if let Some(path) = opts.extra.get("extension").and_then(|v| v.as_str()) {
            args.push("--extension".into());
            args.push(path.into());
        }
        if let Some(arr) = opts.extra.get("extensions").and_then(|v| v.as_array()) {
            for p in arr.iter().filter_map(|v| v.as_str()) {
                args.push("--extension".into());
                args.push(p.into());
            }
        }

        Ok(PreparedLaunch {
            harness: "pi".into(),
            spawn: Some(SpawnSpec {
                program,
                args,
                cwd: opts.cwd.clone(),
                env: base_env(opts),
                retain_stdin: false,
            }),
            synthetic: None,
            capabilities: self.capabilities(),
            multi_turn,
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
        let ty = value.get("type").and_then(|t| t.as_str()).unwrap_or("");
        match ty {
            "session" => {
                let id = value
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if id.is_empty() {
                    vec![Event::Raw {
                        channel: "pi.session".into(),
                        line: value.to_string(),
                    }]
                } else {
                    vec![Event::SessionInfo { id, label: None }]
                }
            }
            "turn_start" => vec![Event::TurnStart { turn: 0 }],
            "turn_end" => vec![Event::TurnComplete {
                turn: 0,
                stop_reason: None,
            }],
            "agent_start" => vec![Event::TurnStart { turn: 0 }],
            "agent_end" => Vec::new(),
            // Settled = turn finished; process may still exit → ProcessExit from transport.
            "agent_settled" => vec![Event::TurnComplete {
                turn: 0,
                stop_reason: Some("settled".into()),
            }],
            "message_update" => parse_pi_message_update(value),
            "message_end" | "message_start" => Vec::new(),
            // Pi extension lifecycle (docs: tool_call can block) + tool execution stream.
            // Normalize to general Tool* and Hook* (PreToolUse / PostToolUse).
            "tool_execution_start" | "tool_call" => {
                let id = value
                    .get("toolCallId")
                    .or_else(|| value.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = value
                    .get("toolName")
                    .or_else(|| value.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let input = value
                    .get("args")
                    .or_else(|| value.get("input"))
                    .cloned()
                    .unwrap_or(Value::Null);
                let native = ty.to_string();
                vec![
                    Event::HookStarted {
                        id: id.clone(),
                        name: "PreToolUse".into(),
                        phase: Some(native),
                        detail: Some(serde_json::json!({
                            "tool": name,
                            "input": input,
                        })),
                    },
                    Event::ToolCall { id, name, input },
                ]
            }
            "tool_execution_end" | "tool_result" => {
                let id = value
                    .get("toolCallId")
                    .or_else(|| value.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = value
                    .get("toolName")
                    .or_else(|| value.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let output = value
                    .get("result")
                    .or_else(|| value.get("output"))
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .unwrap_or_default();
                let is_error = value
                    .get("isError")
                    .or_else(|| value.get("is_error"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let native = ty.to_string();
                vec![
                    Event::ToolResult {
                        id: id.clone(),
                        name: name.clone(),
                        output: output.clone(),
                        is_error,
                    },
                    Event::HookFinished {
                        id,
                        name: "PostToolUse".into(),
                        phase: Some(native),
                        ok: !is_error,
                        detail: Some(if name.is_empty() {
                            output
                        } else {
                            format!("{name}: {output}")
                        }),
                    },
                ]
            }
            "session_start" => vec![Event::HookStarted {
                id: "session".into(),
                name: "SessionStart".into(),
                phase: value
                    .get("reason")
                    .and_then(|r| r.as_str())
                    .map(str::to_string),
                detail: Some(value.clone()),
            }],
            "session_shutdown" | "session_end" => vec![Event::HookFinished {
                id: "session".into(),
                name: "SessionEnd".into(),
                phase: None,
                ok: true,
                detail: None,
            }],
            _ => vec![Event::Raw {
                channel: "pi".into(),
                line: value.to_string(),
            }],
        }
    }
}

fn parse_pi_message_update(value: &Value) -> Vec<Event> {
    let Some(ev) = value.get("assistantMessageEvent") else {
        return Vec::new();
    };
    let ev_type = ev.get("type").and_then(|t| t.as_str()).unwrap_or("");
    match ev_type {
        "text_delta" => {
            let text = ev
                .get("delta")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();
            if text.is_empty() {
                Vec::new()
            } else {
                vec![Event::TextDelta { text }]
            }
        }
        "thinking_delta" => {
            let text = ev
                .get("delta")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();
            if text.is_empty() {
                Vec::new()
            } else {
                vec![Event::ThinkingDelta { text }]
            }
        }
        // Pi JSON: tool calls stream inside message_update.
        "toolcall_start" | "toolcall_end" => {
            let tc = ev.get("toolCall");
            let (id, name, input) = if let Some(tc) = tc {
                (
                    tc.get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    tc.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    tc.get("arguments")
                        .or_else(|| tc.get("args"))
                        .cloned()
                        .unwrap_or(Value::Null),
                )
            } else {
                // toolcall_start often embeds toolCall on partial.message content last item
                let partial = ev.get("partial").or_else(|| value.get("message"));
                let content = partial
                    .and_then(|p| p.get("content"))
                    .and_then(|c| c.as_array())
                    .and_then(|arr| {
                        arr.iter()
                            .rev()
                            .find(|x| x.get("type").and_then(|t| t.as_str()) == Some("toolCall"))
                    });
                match content {
                    Some(tc) => (
                        tc.get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        tc.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        tc.get("arguments")
                            .or_else(|| tc.get("args"))
                            .cloned()
                            .unwrap_or(Value::Null),
                    ),
                    None => return Vec::new(),
                }
            };
            // Emit on end (complete args); also on start if name known.
            if ev_type == "toolcall_start" && name == "unknown" {
                return Vec::new();
            }
            vec![
                Event::HookStarted {
                    id: id.clone(),
                    name: "PreToolUse".into(),
                    phase: Some(ev_type.to_string()),
                    detail: Some(serde_json::json!({ "tool": name, "input": input })),
                },
                Event::ToolCall { id, name, input },
            ]
        }
        _ => Vec::new(),
    }
}

/// Public helper for tests: build argv for multi-turn continue/resume.
pub fn pi_prepare_args(
    prompt: &str,
    opts: &LaunchOptions,
    ctx: &TurnContext,
) -> Result<Vec<String>> {
    let prepared = PiAdapter.prepare(prompt, opts, ctx)?;
    Ok(prepared.spawn.map(|s| s.args).unwrap_or_default())
}
