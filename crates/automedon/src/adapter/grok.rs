//! Grok Build TUI adapter — headless `-p` with streaming-json; multi-turn via `--resume`.

use serde_json::Value;

use super::{base_env, resolve_bin, Adapter, Capabilities, PreparedLaunch, TurnContext};
use crate::config::LaunchOptions;
use crate::error::Result;
use crate::event::Event;
use crate::transport::SpawnSpec;

#[derive(Debug, Default, Clone)]
pub struct GrokAdapter;

impl Adapter for GrokAdapter {
    fn name(&self) -> &'static str {
        "grok"
    }

    fn capabilities(&self) -> Capabilities {
        // Headless streaming-json multi-turn + resume; ACP via `grok agent stdio`.
        // No interactive mid-flight permission/plan encode on this path.
        Capabilities {
            launch: true,
            multi_turn: true,
            stream_tools: true,
            sessions: true,
            streaming_json: true,
            yolo: true,
            permissions_preflight: true,
            permissions: false,
            permissions_interactive: false,
            acp: true,
            ..Default::default()
        }
    }

    fn prepare(
        &self,
        prompt: &str,
        opts: &LaunchOptions,
        ctx: &TurnContext,
    ) -> Result<PreparedLaunch> {
        let program = resolve_bin(opts, "grok");
        let use_acp = opts
            .extra
            .get("acp")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if use_acp {
            let mut env = base_env(opts);
            env.insert("AUTOMEDON_ACP_PROMPT".into(), prompt.to_string());
            return Ok(PreparedLaunch {
                harness: "grok".into(),
                spawn: Some(SpawnSpec {
                    program,
                    args: vec!["agent".into(), "stdio".into()],
                    cwd: opts.cwd.clone(),
                    env,
                    retain_stdin: true,
                }),
                synthetic: None,
                capabilities: self.capabilities(),
                multi_turn: true,
            });
        }

        let mut args = vec![
            "-p".into(),
            prompt.to_string(),
            "--output-format".into(),
            "streaming-json".into(),
        ];

        // Multi-turn: --resume <id> when known; else --continue for turn ≥ 2 (most recent cwd session).
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
            .or_else(|| opts.extra.get("session_id"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        {
            // Explicit resume/session on first prompt when operator supplies an id.
            if opts.extra.contains_key("resume") {
                args.push("--resume".into());
                args.push(id.into());
            } else {
                args.push("--session-id".into());
                args.push(id.into());
            }
        }

        if opts.yolo {
            args.push("--always-approve".into());
        }
        if let Some(model) = &opts.model {
            args.push("-m".into());
            args.push(model.clone());
        }
        if let Some(cwd) = &opts.cwd {
            args.push("--cwd".into());
            args.push(cwd.display().to_string());
        }
        if let Some(v) = opts.extra.get("max_turns").and_then(|v| v.as_u64()) {
            args.push("--max-turns".into());
            args.push(v.to_string());
        }
        if let Some(tools) = opts.extra.get("tools").and_then(|v| v.as_str()) {
            args.push("--tools".into());
            args.push(tools.into());
        }
        if let Some(tools) = opts.extra.get("disallowed_tools").and_then(|v| v.as_str()) {
            args.push("--disallowed-tools".into());
            args.push(tools.into());
        }
        if let Some(effort) = opts.extra.get("effort").and_then(|v| v.as_str()) {
            args.push("--effort".into());
            args.push(effort.into());
        }
        if opts
            .extra
            .get("include_partial")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            args.push("--include-partial-messages".into());
            if let Some(i) = args.iter().position(|a| a == "streaming-json") {
                args[i] = "streaming-messages-json".into();
            }
        }

        let multi_turn = opts
            .extra
            .get("multi_turn")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        Ok(PreparedLaunch {
            harness: "grok".into(),
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
            "text" => {
                let text = value
                    .get("data")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string();
                if text.is_empty() {
                    Vec::new()
                } else {
                    vec![Event::TextDelta { text }]
                }
            }
            "thought" | "thinking" => {
                let text = value
                    .get("data")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string();
                if text.is_empty() {
                    Vec::new()
                } else {
                    vec![Event::ThinkingDelta { text }]
                }
            }
            "tool_call" | "tool_use" => {
                let id = value
                    .get("id")
                    .or_else(|| value.get("toolCallId"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                // Live Grok streaming-json uses toolName/title; older shapes use name/tool.
                let name = value
                    .get("toolName")
                    .or_else(|| value.get("name"))
                    .or_else(|| value.get("tool"))
                    .or_else(|| value.get("title"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let input = value
                    .get("rawInput")
                    .or_else(|| value.get("input"))
                    .or_else(|| value.get("arguments"))
                    .cloned()
                    .unwrap_or(Value::Null);
                vec![Event::ToolCall { id, name, input }]
            }
            "tool_call_update" => {
                // Completed tool updates carry stdout in content / rawOutput.
                let status = value.get("status").and_then(|v| v.as_str()).unwrap_or("");
                if status != "completed" && status != "failed" && status != "error" {
                    return Vec::new();
                }
                let id = value
                    .get("toolCallId")
                    .or_else(|| value.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = value
                    .get("toolName")
                    .or_else(|| value.get("name"))
                    .or_else(|| value.get("title"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let output = value
                    .get("content")
                    .and_then(|c| c.as_array())
                    .and_then(|arr| {
                        arr.iter().find_map(|item| {
                            item.get("content")
                                .and_then(|inner| inner.get("text"))
                                .and_then(|t| t.as_str())
                                .map(str::to_string)
                        })
                    })
                    .or_else(|| {
                        value.get("rawOutput").map(|v| match v {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                    })
                    .unwrap_or_default();
                let is_error = status == "failed" || status == "error";
                vec![Event::ToolResult {
                    id,
                    name,
                    output,
                    is_error,
                }]
            }
            "tool_result" => {
                let id = value
                    .get("id")
                    .or_else(|| value.get("toolCallId"))
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
                    .get("output")
                    .or_else(|| value.get("data"))
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .unwrap_or_default();
                let is_error = value
                    .get("is_error")
                    .or_else(|| value.get("isError"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                vec![Event::ToolResult {
                    id,
                    name,
                    output,
                    is_error,
                }]
            }
            "usage" => {
                let u = value.get("usage").unwrap_or(value);
                vec![Event::Usage {
                    input_tokens: u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                    output_tokens: u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                    total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                    cost_usd: value
                        .get("total_cost_usd")
                        .and_then(|v| v.as_f64())
                        .or_else(|| u.get("total_cost_usd").and_then(|v| v.as_f64())),
                }]
            }
            "end" => {
                let reason = value
                    .get("stopReason")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                let mut events = Vec::new();
                if let Some(sid) = value
                    .get("sessionId")
                    .or_else(|| value.get("session_id"))
                    .and_then(|v| v.as_str())
                {
                    events.push(Event::SessionInfo {
                        id: sid.to_string(),
                        label: None,
                    });
                }
                // Plan / goal hooks when present on end or nested payload.
                if let Some(plan) = value.get("plan") {
                    if let Some(summary) = plan.get("summary").and_then(|s| s.as_str()) {
                        events.push(Event::PlanPresented {
                            id: plan
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("plan")
                                .to_string(),
                            summary: summary.to_string(),
                        });
                    }
                }
                if let Some(goal) = value.get("goal") {
                    if let Some(title) = goal.get("title").and_then(|s| s.as_str()) {
                        events.push(Event::GoalStarted {
                            id: goal
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("goal")
                                .to_string(),
                            title: title.to_string(),
                        });
                    }
                }
                events.push(Event::TurnComplete {
                    turn: value.get("num_turns").and_then(|v| v.as_u64()).unwrap_or(1),
                    stop_reason: reason,
                });
                // Multi-turn: do not emit Done here — process exit is ProcessExit.
                // One-shot callers still finish when the process exits.
                events
            }
            "error" => {
                let message = value
                    .get("message")
                    .or_else(|| value.get("data"))
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .unwrap_or_else(|| value.to_string());
                vec![Event::Error { message }]
            }
            // Best-effort hook shapes (harness-native or future streaming-json).
            "hook" | "hook_start" | "hook_started" => vec![Event::HookStarted {
                id: value
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("hook")
                    .to_string(),
                name: value
                    .get("name")
                    .or_else(|| value.get("hook"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                phase: value
                    .get("phase")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                detail: value
                    .get("detail")
                    .cloned()
                    .or_else(|| value.get("data").cloned().filter(|d| !d.is_null())),
            }],
            "hook_end" | "hook_finished" | "hook_complete" => vec![Event::HookFinished {
                id: value
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("hook")
                    .to_string(),
                name: value
                    .get("name")
                    .or_else(|| value.get("hook"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                phase: value
                    .get("phase")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                ok: value
                    .get("ok")
                    .or_else(|| value.get("success"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                detail: value
                    .get("detail")
                    .or_else(|| value.get("message"))
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    }),
            }],
            _ => vec![Event::Raw {
                channel: "grok".into(),
                line: value.to_string(),
            }],
        }
    }
}

/// Public helper for tests: build argv for a multi-turn resume shot.
pub fn grok_prepare_args(
    prompt: &str,
    opts: &LaunchOptions,
    ctx: &TurnContext,
) -> Result<Vec<String>> {
    let prepared = GrokAdapter.prepare(prompt, opts, ctx)?;
    Ok(prepared.spawn.map(|s| s.args).unwrap_or_default())
}
