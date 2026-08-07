//! GitHub Copilot CLI specialized driver.
//!
//! Headless: `copilot -p/--prompt` with `--output-format json` (JSONL).
//! Multi-turn: `--resume=<id>` / `--continue`; session id from final `result.sessionId`
//! and legacy text footer `Resume … --resume=<id>`.
//! Optional ACP: `--acp`.

use serde_json::Value;

use super::{
    base_env, resolve_bin, shared_parse, Adapter, Capabilities, PreparedLaunch, TurnContext,
};
use crate::config::LaunchOptions;
use crate::error::Result;
use crate::event::Event;
use crate::transport::SpawnSpec;

#[derive(Debug, Default, Clone)]
pub struct CopilotAdapter;

impl Adapter for CopilotAdapter {
    fn name(&self) -> &'static str {
        "copilot"
    }

    fn capabilities(&self) -> Capabilities {
        // JSONL stream includes assistant message deltas and toolRequests when present.
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
            permissions: false,
            permissions_interactive: false,
            // First-party: `copilot --acp` (Agent Client Protocol server).
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
        let program = resolve_bin(opts, "copilot");
        let use_acp = opts
            .extra
            .get("acp")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if use_acp {
            return Ok(PreparedLaunch {
                harness: "copilot".into(),
                spawn: Some(SpawnSpec {
                    program,
                    args: vec!["--acp".into()],
                    cwd: opts.cwd.clone(),
                    env: base_env(opts),
                    retain_stdin: true,
                }),
                synthetic: None,
                capabilities: self.capabilities(),
                multi_turn: true,
            });
        }

        // Prefer JSONL for structured text/tools/session (`--output-format json`).
        let mut args = vec![
            "-p".into(),
            prompt.to_string(),
            "--output-format".into(),
            "json".into(),
        ];
        if opts.yolo {
            // CLI: --allow-all ≡ tools + paths + urls; --yolo is an alias on recent builds.
            args.push("--allow-all".into());
        }
        if let Some(model) = &opts.model {
            args.push("--model".into());
            args.push(model.clone());
        }
        if ctx.turn > 1 {
            if let Some(id) = ctx.session_id.as_ref().filter(|s| !s.is_empty()) {
                // Flag form used in product help: --resume=<id>
                args.push(format!("--resume={id}"));
            } else {
                args.push("--continue".into());
            }
        } else if let Some(id) = opts.extra.get("resume").and_then(|v| v.as_str()) {
            args.push(format!("--resume={id}"));
        }

        Ok(PreparedLaunch {
            harness: "copilot".into(),
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
        // Legacy plain-text footer (text output mode).
        if let Some(id) = extract_resume_id(line) {
            return vec![
                Event::SessionInfo {
                    id,
                    label: Some("copilot".into()),
                },
                Event::TextDelta {
                    text: format!("{line}\n"),
                },
            ];
        }
        match serde_json::from_str::<Value>(line) {
            Ok(v) => parse_copilot_json(&v),
            Err(_) => vec![Event::TextDelta {
                text: format!("{line}\n"),
            }],
        }
    }
}

fn parse_copilot_json(value: &Value) -> Vec<Event> {
    let ty = value.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let data = value.get("data").unwrap_or(value);

    match ty {
        // Live JSONL always streams deltas then a final assistant.message with full content.
        // Emit TextDelta only from deltas so text is not doubled (HI_ONLYHI_ONLY).
        "assistant.message_delta" => {
            let text = data
                .get("deltaContent")
                .or_else(|| data.get("delta"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            if text.is_empty() {
                Vec::new()
            } else {
                vec![Event::TextDelta { text }]
            }
        }
        "assistant.message" => {
            // Full `content` already arrived via message_delta — only surface toolRequests here.
            let mut out = Vec::new();
            if let Some(arr) = data.get("toolRequests").and_then(|a| a.as_array()) {
                for tr in arr {
                    let id = tr
                        .get("id")
                        .or_else(|| tr.get("toolCallId"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = tr
                        .get("name")
                        .or_else(|| tr.get("toolName"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let input = tr
                        .get("arguments")
                        .or_else(|| tr.get("input"))
                        .or_else(|| tr.get("args"))
                        .cloned()
                        .unwrap_or(Value::Null);
                    out.push(Event::ToolCall { id, name, input });
                }
            }
            out
        }
        "assistant.reasoning_delta" => {
            let text = data
                .get("deltaContent")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            if text.is_empty() {
                Vec::new()
            } else {
                vec![Event::ThinkingDelta { text }]
            }
        }
        // Full reasoning body follows deltas — do not re-emit as ThinkingDelta.
        "assistant.reasoning" => Vec::new(),
        "assistant.turn_start" => vec![Event::TurnStart {
            turn: data
                .get("turnId")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        }],
        // Live order: turn_end then result (with sessionId). TurnComplete only on result
        // so SessionInfo is applied before await_turn returns (see Session::await_turn).
        "assistant.turn_end" => Vec::new(),
        // Final frame: session id for multi-turn resume, then terminal turn complete.
        "result" => {
            let mut out = Vec::new();
            if let Some(sid) = value
                .get("sessionId")
                .or_else(|| data.get("sessionId"))
                .and_then(|v| v.as_str())
            {
                out.push(Event::SessionInfo {
                    id: sid.to_string(),
                    label: Some("copilot".into()),
                });
            }
            let code = value
                .get("exitCode")
                .or_else(|| data.get("exitCode"))
                .and_then(|v| v.as_i64());
            if code.is_some_and(|c| c != 0) {
                out.push(Event::Error {
                    message: format!("copilot exitCode={code:?}"),
                });
            }
            out.push(Event::TurnComplete {
                turn: 1,
                stop_reason: code.map(|c| format!("exit:{c}")),
            });
            out
        }
        "tool.execution_start" | "tool_call" | "tool.start" => {
            let id = data
                .get("id")
                .or_else(|| data.get("toolCallId"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let name = data
                .get("name")
                .or_else(|| data.get("toolName"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let input = data
                .get("arguments")
                .or_else(|| data.get("input"))
                .cloned()
                .unwrap_or(Value::Null);
            shared_parse::tool_start_events(id, name, input, "tool")
        }
        "tool.execution_end"
        | "tool.execution_complete"
        | "tool_result"
        | "tool.end"
        | "tool.complete" => {
            let id = data
                .get("id")
                .or_else(|| data.get("toolCallId"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let name = data
                .get("name")
                .or_else(|| data.get("toolName"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let output = data
                .get("output")
                .or_else(|| data.get("result").and_then(|r| r.get("content")))
                .or_else(|| data.get("result"))
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .unwrap_or_default();
            let is_error = data
                .get("isError")
                .or_else(|| data.get("is_error"))
                .and_then(|v| v.as_bool())
                .unwrap_or_else(|| data.get("success").and_then(|s| s.as_bool()) == Some(false));
            shared_parse::tool_end_events(id, name, output, is_error, "tool")
        }
        "error" | "session.error" => vec![Event::Error {
            message: data
                .get("message")
                .or_else(|| value.get("message"))
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .unwrap_or_else(|| value.to_string()),
        }],
        // Noise: MCP/skills/status — ignore as Raw only if nothing useful.
        t if t.starts_with("session.")
            || t.starts_with("mcp.")
            || t == "model.call_start"
            || t == "assistant.idle"
            || t == "assistant.message_start"
            || t == "user.message" =>
        {
            Vec::new()
        }
        _ => shared_parse::parse_common_json(value, "copilot"),
    }
}

/// Parse `Resume … --resume=<id>` or `--resume <id>` from Copilot CLI footer lines.
fn extract_resume_id(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    if !lower.contains("resume") {
        return None;
    }
    if let Some(idx) = line.find("--resume=") {
        let rest = &line[idx + "--resume=".len()..];
        let id = rest
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_matches(|c: char| c == '"' || c == '\'');
        if !id.is_empty() {
            return Some(id.to_string());
        }
    }
    let mut parts = line.split_whitespace();
    while let Some(p) = parts.next() {
        if p == "--resume" {
            if let Some(id) = parts.next() {
                let id = id.trim_matches(|c: char| c == '"' || c == '\'');
                if !id.is_empty() {
                    return Some(id.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_resume_footer() {
        let id =
            extract_resume_id("Resume     copilot --resume=a15c9384-9de2-4eb1-88d7-fa86d83b4860")
                .unwrap();
        assert_eq!(id, "a15c9384-9de2-4eb1-88d7-fa86d83b4860");
    }

    #[test]
    fn parse_jsonl_message_and_result_session() {
        let a = CopilotAdapter;
        let ev = a.parse_line(r#"{"type":"assistant.message_delta","data":{"deltaContent":"HI"}}"#);
        assert!(matches!(ev.first(), Some(Event::TextDelta { text }) if text == "HI"));
        // Full message content must not re-emit text (would double HI).
        let ev = a.parse_line(
            r#"{"type":"assistant.message","data":{"content":"HI","toolRequests":[]}}"#,
        );
        assert!(ev.iter().all(|e| !matches!(e, Event::TextDelta { .. })));
        let ev = a.parse_line(
            r#"{"type":"result","sessionId":"a81b42ef-a1ea-4b38-93de-8f8bf1287571","exitCode":0}"#,
        );
        assert!(ev.iter().any(
            |e| matches!(e, Event::SessionInfo { id, .. } if id == "a81b42ef-a1ea-4b38-93de-8f8bf1287571")
        ));
        assert!(ev.iter().any(|e| matches!(e, Event::TurnComplete { .. })));
    }

    #[test]
    fn live_order_deltas_then_result_one_turn_complete_and_session() {
        // Shape from copilot --output-format json capture (delta → message → turn_end → result).
        let a = CopilotAdapter;
        let mut text = String::new();
        let mut turn_completes = 0usize;
        let mut session: Option<String> = None;
        let lines = [
            r#"{"type":"assistant.message_delta","data":{"deltaContent":"HI_ONLY"}}"#,
            r#"{"type":"assistant.message","data":{"content":"HI_ONLY","toolRequests":[]}}"#,
            r#"{"type":"assistant.turn_end","data":{"turnId":"0"}}"#,
            r#"{"type":"result","sessionId":"a81b42ef-a1ea-4b38-93de-8f8bf1287571","exitCode":0}"#,
        ];
        for line in lines {
            for e in a.parse_line(line) {
                match e {
                    Event::TextDelta { text: t } => text.push_str(&t),
                    Event::TurnComplete { .. } => turn_completes += 1,
                    Event::SessionInfo { id, .. } => session = Some(id),
                    _ => {}
                }
            }
        }
        assert_eq!(text, "HI_ONLY", "must not double full message content");
        assert_eq!(
            turn_completes, 1,
            "TurnComplete only from result, not turn_end"
        );
        assert_eq!(
            session.as_deref(),
            Some("a81b42ef-a1ea-4b38-93de-8f8bf1287571")
        );
    }

    #[test]
    fn tool_requests_and_execution_frames() {
        let a = CopilotAdapter;
        let ev = a.parse_line(
            r#"{"type":"assistant.message","data":{"content":"running","toolRequests":[{"id":"t1","name":"bash","arguments":{"c":"ls"}}]}}"#,
        );
        assert!(
            ev.iter().any(
                |e| matches!(e, Event::ToolCall { id, name, .. } if id == "t1" && name == "bash")
            ),
            "{ev:?}"
        );
        assert!(
            !ev.iter().any(|e| matches!(e, Event::TextDelta { .. })),
            "message content must not TextDelta when tools present either"
        );
        let ev = a.parse_line(
            r#"{"type":"tool.execution_start","data":{"id":"t1","name":"bash","arguments":{"c":"ls"}}}"#,
        );
        assert!(ev
            .iter()
            .any(|e| matches!(e, Event::HookStarted { name, .. } if name == "PreToolUse")));
        assert!(ev
            .iter()
            .any(|e| matches!(e, Event::ToolCall { name, .. } if name == "bash")));
        let ev = a.parse_line(
            r#"{"type":"tool.execution_end","data":{"id":"t1","name":"bash","output":"ok","isError":false}}"#,
        );
        assert!(ev.iter().any(|e| matches!(
            e,
            Event::ToolResult {
                is_error: false,
                output,
                ..
            } if output == "ok"
        )));
        assert!(ev
            .iter()
            .any(|e| matches!(e, Event::HookFinished { name, .. } if name == "PostToolUse")));
    }

    #[test]
    fn reasoning_delta_not_doubled_by_full_reasoning() {
        let a = CopilotAdapter;
        let mut thinking = String::new();
        for line in [
            r#"{"type":"assistant.reasoning_delta","data":{"deltaContent":"think "}}"#,
            r#"{"type":"assistant.reasoning_delta","data":{"deltaContent":"more"}}"#,
            r#"{"type":"assistant.reasoning","data":{"content":"think more","reasoningText":"think more"}}"#,
        ] {
            for e in a.parse_line(line) {
                if let Event::ThinkingDelta { text } = e {
                    thinking.push_str(&text);
                }
            }
        }
        assert_eq!(thinking, "think more");
    }

    #[test]
    fn prepare_json_format_and_resume() {
        let a = CopilotAdapter;
        let opts = LaunchOptions {
            yolo: true,
            ..Default::default()
        };
        let p = a
            .prepare(
                "hi",
                &opts,
                &TurnContext {
                    turn: 2,
                    session_id: Some("sess-1".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        let args = p.spawn.unwrap().args;
        assert!(args
            .windows(2)
            .any(|w| w[0] == "--output-format" && w[1] == "json"));
        assert!(args.iter().any(|a| a == "--resume=sess-1"));
        assert!(args.iter().any(|a| a == "--allow-all"));
    }
}
