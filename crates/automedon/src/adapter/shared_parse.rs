//! Shared NDJSON / agent event shapes across CLIs.

use serde_json::Value;

use crate::event::Event;

/// Map common agent stream JSON into normalized events.
pub fn parse_common_json(value: &Value, channel: &str) -> Vec<Event> {
    let ty = value
        .get("type")
        .or_else(|| value.get("method"))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    // ACP-ish session/update notifications
    if ty.contains("session/update") || ty == "session_update" {
        return parse_session_update(value);
    }

    match ty {
        "text" | "agent_message" | "message" => {
            // OpenCode nests text under part.text; others use top-level text/data/content.
            let text = value
                .get("data")
                .or_else(|| value.get("text"))
                .or_else(|| value.get("content"))
                .and_then(|d| d.as_str())
                .or_else(|| value.pointer("/part/text").and_then(|t| t.as_str()))
                .unwrap_or("")
                .to_string();
            if text.is_empty() {
                Vec::new()
            } else {
                vec![Event::TextDelta { text }]
            }
        }
        "thought" | "thinking" | "reasoning" => {
            let text = value
                .get("data")
                .or_else(|| value.get("text"))
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();
            if text.is_empty() {
                Vec::new()
            } else {
                vec![Event::ThinkingDelta { text }]
            }
        }
        "tool_call" | "tool_use" | "function_call" => {
            vec![Event::ToolCall {
                id: value
                    .get("id")
                    .or_else(|| value.get("toolCallId"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                name: value
                    .get("name")
                    .or_else(|| value.get("tool"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                input: value
                    .get("input")
                    .or_else(|| value.get("arguments"))
                    .or_else(|| value.get("args"))
                    .cloned()
                    .unwrap_or(Value::Null),
            }]
        }
        "tool_result" | "function_call_output" => {
            vec![Event::ToolResult {
                id: value
                    .get("id")
                    .or_else(|| value.get("toolCallId"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                name: value
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                output: value
                    .get("output")
                    .or_else(|| value.get("result"))
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .unwrap_or_default(),
                is_error: value
                    .get("is_error")
                    .or_else(|| value.get("isError"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            }]
        }
        "session" => {
            let id = value
                .get("id")
                .or_else(|| value.get("sessionId"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if id.is_empty() {
                vec![Event::Raw {
                    channel: channel.into(),
                    line: value.to_string(),
                }]
            } else {
                vec![Event::SessionInfo { id, label: None }]
            }
        }
        "turn_complete" | "result" | "end" | "agent_end" => {
            let mut out = Vec::new();
            if let Some(sid) = value
                .get("sessionId")
                .or_else(|| value.get("session_id"))
                .and_then(|v| v.as_str())
            {
                out.push(Event::SessionInfo {
                    id: sid.to_string(),
                    label: None,
                });
            }
            if let Some(text) = value.get("result").and_then(|v| v.as_str()) {
                if !text.is_empty() {
                    out.push(Event::TextDelta {
                        text: text.to_string(),
                    });
                }
            }
            out.push(Event::TurnComplete {
                turn: value.get("num_turns").and_then(|v| v.as_u64()).unwrap_or(1),
                stop_reason: value
                    .get("stopReason")
                    .or_else(|| value.get("subtype"))
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
            });
            out
        }
        "error" => vec![Event::Error {
            message: value
                .get("message")
                .or_else(|| value.get("error"))
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .unwrap_or_else(|| value.to_string()),
        }],
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
            detail: value.get("detail").cloned(),
        }],
        "hook_end" | "hook_finished" => vec![Event::HookFinished {
            id: value
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("hook")
                .to_string(),
            name: value
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            phase: value
                .get("phase")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            ok: value.get("ok").and_then(|v| v.as_bool()).unwrap_or(true),
            detail: None,
        }],
        "permission_request" | "permission" => vec![Event::PermissionRequest {
            id: value
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("perm")
                .to_string(),
            tool: value
                .get("tool")
                .or_else(|| value.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            detail: value
                .get("detail")
                .or_else(|| value.get("command"))
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .unwrap_or_default(),
        }],
        "agent_start" | "turn_start" | "turn.started" => vec![Event::TurnStart { turn: 0 }],
        // OpenAI Codex CLI (`codex exec --json`)
        "thread.started" => {
            let id = value
                .get("thread_id")
                .or_else(|| value.get("threadId"))
                .or_else(|| value.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if id.is_empty() {
                Vec::new()
            } else {
                vec![Event::SessionInfo {
                    id,
                    label: Some("codex-thread".into()),
                }]
            }
        }
        "turn.completed" => {
            let mut out = Vec::new();
            if let Some(sid) = value
                .get("thread_id")
                .or_else(|| value.get("sessionId"))
                .and_then(|v| v.as_str())
            {
                out.push(Event::SessionInfo {
                    id: sid.to_string(),
                    label: None,
                });
            }
            out.push(Event::TurnComplete {
                turn: value.get("turn").and_then(|v| v.as_u64()).unwrap_or(1),
                stop_reason: value
                    .get("stop_reason")
                    .or_else(|| value.get("stopReason"))
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
            });
            out
        }
        "item.completed" | "item.started" => parse_codex_item(value, ty == "item.completed"),
        // OpenCode `run --format json`
        "step_start" | "step-start" => {
            let mut out = Vec::new();
            if let Some(sid) = value
                .get("sessionID")
                .or_else(|| value.get("sessionId"))
                .or_else(|| value.get("session_id"))
                .and_then(|v| v.as_str())
            {
                out.push(Event::SessionInfo {
                    id: sid.to_string(),
                    label: Some("opencode".into()),
                });
            }
            out.push(Event::TurnStart { turn: 0 });
            out
        }
        "step_finish" | "step-finish" | "step_end" => vec![Event::TurnComplete {
            turn: 1,
            stop_reason: value
                .get("reason")
                .and_then(|v| v.as_str())
                .map(str::to_string),
        }],
        // Claude Code stream-json system init (even when auth fails)
        "system" => {
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
                    channel: channel.into(),
                    line: value.to_string(),
                });
            }
            out
        }
        // Unknown types: do not scavenge session_id (avoids pinning resume to error-frame noise).
        // Known frames that carry session id are handled above (system, session, result, thread.*, step_*).
        _ => vec![Event::Raw {
            channel: channel.into(),
            line: value.to_string(),
        }],
    }
}

fn parse_codex_item(value: &Value, completed: bool) -> Vec<Event> {
    let item = value.get("item").unwrap_or(value);
    let item_ty = item
        .get("type")
        .or_else(|| item.get("item_type"))
        .and_then(|t| t.as_str())
        .unwrap_or("");
    let id = item
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    match item_ty {
        "command_execution" | "command" | "tool_call" | "function_call" => {
            let name = item
                .get("command")
                .or_else(|| item.get("name"))
                .or_else(|| item.get("tool"))
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .unwrap_or_else(|| item_ty.to_string());
            if completed {
                vec![Event::ToolResult {
                    id,
                    name,
                    output: item
                        .get("aggregated_output")
                        .or_else(|| item.get("output"))
                        .or_else(|| item.get("result"))
                        .map(|v| match v {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .unwrap_or_default(),
                    is_error: item
                        .get("exit_code")
                        .and_then(|c| c.as_i64())
                        .is_some_and(|c| c != 0)
                        || item.get("status").and_then(|s| s.as_str()) == Some("failed"),
                }]
            } else {
                vec![Event::ToolCall {
                    id,
                    name,
                    input: item
                        .get("command")
                        .or_else(|| item.get("arguments"))
                        .cloned()
                        .unwrap_or(Value::Null),
                }]
            }
        }
        // Live Codex stream (2026): file writes/edits as file_change items, not command_execution.
        "file_change" | "file_edit" | "apply_patch" => {
            let name = item_ty.to_string();
            let input = item
                .get("changes")
                .or_else(|| item.get("path"))
                .cloned()
                .unwrap_or_else(|| item.clone());
            if completed {
                let failed = item.get("status").and_then(|s| s.as_str()) == Some("failed");
                vec![Event::ToolResult {
                    id,
                    name,
                    output: match &input {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    },
                    is_error: failed,
                }]
            } else {
                vec![Event::ToolCall { id, name, input }]
            }
        }
        "error" => vec![Event::Error {
            message: item
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("item error")
                .to_string(),
        }],
        "agent_message" | "message" => {
            let text = item
                .get("text")
                .or_else(|| item.get("content"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            if text.is_empty() {
                Vec::new()
            } else {
                vec![Event::TextDelta { text }]
            }
        }
        _ => {
            if completed {
                vec![Event::Raw {
                    channel: "codex-item".into(),
                    line: value.to_string(),
                }]
            } else {
                Vec::new()
            }
        }
    }
}

fn parse_session_update(value: &Value) -> Vec<Event> {
    let params = value.get("params").unwrap_or(value);
    let update = params.get("update").unwrap_or(params);
    let uty = update
        .get("sessionUpdate")
        .or_else(|| update.get("type"))
        .and_then(|t| t.as_str())
        .unwrap_or("");
    match uty {
        "agent_message_chunk" | "message" | "agent_thought_chunk" => {
            let text = update
                .get("content")
                .and_then(|c| c.get("text"))
                .or_else(|| update.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            if text.is_empty() {
                Vec::new()
            } else if uty.contains("thought") {
                vec![Event::ThinkingDelta { text }]
            } else {
                vec![Event::TextDelta { text }]
            }
        }
        "tool_call" | "tool_call_update" => {
            let name = update
                .get("title")
                .or_else(|| update.get("name"))
                .or_else(|| update.get("toolName"))
                .and_then(|v| v.as_str())
                .unwrap_or("tool")
                .to_string();
            let id = update
                .get("toolCallId")
                .or_else(|| update.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if uty == "tool_call_update"
                && update
                    .get("status")
                    .and_then(|s| s.as_str())
                    .is_some_and(|s| s == "completed" || s == "failed")
            {
                vec![Event::ToolResult {
                    id,
                    name,
                    output: update
                        .get("content")
                        .map(|c| c.to_string())
                        .unwrap_or_default(),
                    is_error: update.get("status").and_then(|s| s.as_str()) == Some("failed"),
                }]
            } else {
                vec![Event::ToolCall {
                    id,
                    name,
                    input: update.get("rawInput").cloned().unwrap_or(Value::Null),
                }]
            }
        }
        _ => vec![Event::Raw {
            channel: "acp".into(),
            line: value.to_string(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn text_and_thinking_variants() {
        let e = parse_common_json(&json!({"type":"agent_message","text":"a"}), "t");
        assert!(matches!(e.first(), Some(Event::TextDelta { text }) if text == "a"));
        let e = parse_common_json(&json!({"type":"thinking","data":"z"}), "t");
        assert!(matches!(e.first(), Some(Event::ThinkingDelta { text }) if text == "z"));
        assert!(parse_common_json(&json!({"type":"text","data":""}), "t").is_empty());
    }

    #[test]
    fn tools_session_error_hooks_permission() {
        let e = parse_common_json(
            &json!({"type":"tool_use","id":"1","name":"bash","arguments":{"x":1}}),
            "t",
        );
        assert!(matches!(e.first(), Some(Event::ToolCall { name, .. }) if name == "bash"));
        let e = parse_common_json(
            &json!({"type":"function_call_output","toolCallId":"1","result":"ok","isError":false}),
            "t",
        );
        assert!(matches!(e.first(), Some(Event::ToolResult { output, .. }) if output == "ok"));
        let e = parse_common_json(&json!({"type":"session","sessionId":"s9"}), "t");
        assert!(matches!(e.first(), Some(Event::SessionInfo { id, .. }) if id == "s9"));
        let e = parse_common_json(&json!({"type":"error","error":{"code":1}}), "t");
        assert!(matches!(e.first(), Some(Event::Error { .. })));
        let e = parse_common_json(
            &json!({"type":"hook_started","id":"h","name":"PreToolUse","phase":"pre"}),
            "t",
        );
        assert!(matches!(e.first(), Some(Event::HookStarted { name, .. }) if name == "PreToolUse"));
        let e = parse_common_json(
            &json!({"type":"hook_finished","id":"h","name":"Post","ok":false}),
            "t",
        );
        assert!(matches!(
            e.first(),
            Some(Event::HookFinished { ok: false, .. })
        ));
        let e = parse_common_json(
            &json!({"type":"permission_request","id":"p","tool":"bash","command":"ls"}),
            "t",
        );
        assert!(matches!(e.first(), Some(Event::PermissionRequest { tool, .. }) if tool == "bash"));
        let e = parse_common_json(&json!({"type":"turn_start"}), "t");
        assert!(matches!(e.first(), Some(Event::TurnStart { .. })));
        let e = parse_common_json(&json!({"type":"weird"}), "t");
        assert!(matches!(e.first(), Some(Event::Raw { .. })));
        let e = parse_common_json(&json!({"type":"thread.started","thread_id":"th1"}), "codex");
        assert!(matches!(e.first(), Some(Event::SessionInfo { id, .. }) if id == "th1"));
        let e = parse_common_json(&json!({"type":"step_start","sessionID":"ses1"}), "opencode");
        assert!(e
            .iter()
            .any(|x| matches!(x, Event::SessionInfo { id, .. } if id == "ses1")));
        let e = parse_common_json(
            &json!({"type":"system","session_id":"cl1","subtype":"init"}),
            "claude",
        );
        assert!(matches!(e.first(), Some(Event::SessionInfo { id, .. }) if id == "cl1"));
    }

    #[test]
    fn exhaust_common_type_strings() {
        // Drive every named arm at least once for line coverage.
        let samples = [
            r#"{"type":"agent_message","text":"a"}"#,
            r#"{"type":"message","content":"b"}"#,
            r#"{"type":"thought","data":"t"}"#,
            r#"{"type":"reasoning","text":"r"}"#,
            r#"{"type":"function_call","id":"1","tool":"x","args":{}}"#,
            r#"{"type":"function_call_output","toolCallId":"1","output":"o","is_error":true}"#,
            r#"{"type":"session","id":"s"}"#,
            r#"{"type":"session","version":1}"#,
            r#"{"type":"end","session_id":"s","result":"ok"}"#,
            r#"{"type":"agent_end"}"#,
            r#"{"type":"error","message":"e"}"#,
            r#"{"type":"hook","id":"h","hook":"Pre","phase":"p","detail":1}"#,
            r#"{"type":"hook_end","id":"h","name":"Post","phase":"p","ok":true}"#,
            r#"{"type":"permission","id":"p","name":"bash","detail":{"c":1}}"#,
            r#"{"type":"agent_start"}"#,
            r#"{"type":"turn.started"}"#,
            r#"{"type":"thread.started","threadId":"T"}"#,
            r#"{"type":"step_finish"}"#,
            r#"{"type":"step-finish"}"#,
            r#"{"type":"system","sessionId":"S"}"#,
            r#"{"type":"item.started","item":{"id":"i","type":"tool_call","tool":"t","args":{}}}"#,
            r#"{"type":"item.completed","item":{"id":"i","type":"command_execution","command":"c","result":"r","exit_code":0}}"#,
            r#"{"type":"item.completed","item":{"id":"i","type":"agent_message","text":""}}"#,
            r#"{"method":"session/update","params":{"update":{"sessionUpdate":"agent_thought_chunk","text":"th"}}}"#,
            r#"{"method":"session/update","params":{"update":{"sessionUpdate":"tool_call","toolCallId":"t","title":"bash","rawInput":{}}}}"#,
            r#"{"method":"session/update","params":{"update":{"sessionUpdate":"tool_call_update","toolCallId":"t","title":"bash","status":"completed","content":"ok"}}}"#,
            r#"{"method":"session/update","params":{"update":{"sessionUpdate":"tool_call_update","toolCallId":"t","title":"bash","status":"failed","content":"no"}}}"#,
            r#"{"method":"session/update","params":{"update":{"sessionUpdate":"other"}}}"#,
        ];
        for s in samples {
            let v: Value = serde_json::from_str(s).unwrap();
            let _ = parse_common_json(&v, "cov");
        }
    }

    #[test]
    fn codex_opencode_item_and_step_variants() {
        let e = parse_common_json(&json!({"type":"turn.started"}), "codex");
        assert!(matches!(e.first(), Some(Event::TurnStart { .. })));
        let e = parse_common_json(
            &json!({"type":"turn.completed","thread_id":"th","turn":3,"stop_reason":"end"}),
            "codex",
        );
        assert!(e.iter().any(|x| matches!(x, Event::TurnComplete { .. })));
        assert!(e
            .iter()
            .any(|x| matches!(x, Event::SessionInfo { id, .. } if id == "th")));
        let e = parse_common_json(
            &json!({
                "type":"item.started",
                "item":{"id":"i1","type":"function_call","name":"bash","arguments":{"c":"ls"}}
            }),
            "codex",
        );
        assert!(matches!(e.first(), Some(Event::ToolCall { name, .. }) if name == "bash"));
        let e = parse_common_json(
            &json!({
                "type":"item.completed",
                "item":{"id":"i1","type":"command","command":"ls","exit_code":1,"aggregated_output":"err","status":"failed"}
            }),
            "codex",
        );
        assert!(matches!(
            e.first(),
            Some(Event::ToolResult { is_error: true, .. })
        ));
        let e = parse_common_json(
            &json!({"type":"item.completed","item":{"id":"i","type":"message","content":"hi"}}),
            "codex",
        );
        assert!(matches!(e.first(), Some(Event::TextDelta { text }) if text == "hi"));
        // Live codex exec: file_change items (path in changes[]) map to tools.
        let e = parse_common_json(
            &json!({
                "type":"item.started",
                "item":{
                    "id":"item_1",
                    "type":"file_change",
                    "changes":[{"path":"/tmp/note.txt","kind":"add"}],
                    "status":"in_progress"
                }
            }),
            "codex",
        );
        assert!(
            matches!(e.first(), Some(Event::ToolCall { name, input, .. })
                if name == "file_change" && input.to_string().contains("note.txt")),
            "{e:?}"
        );
        let e = parse_common_json(
            &json!({
                "type":"item.completed",
                "item":{
                    "id":"item_1",
                    "type":"file_change",
                    "changes":[{"path":"/tmp/note.txt","kind":"add"}],
                    "status":"completed"
                }
            }),
            "codex",
        );
        assert!(
            matches!(e.first(), Some(Event::ToolResult { name, is_error: false, .. })
                if name == "file_change"),
            "{e:?}"
        );
        let e = parse_common_json(
            &json!({"type":"item.completed","item":{"id":"i","type":"error","message":"boom"}}),
            "codex",
        );
        assert!(matches!(e.first(), Some(Event::Error { .. })));
        let e = parse_common_json(
            &json!({"type":"item.completed","item":{"id":"i","type":"other"}}),
            "codex",
        );
        assert!(matches!(e.first(), Some(Event::Raw { .. })));
        let e = parse_common_json(
            &json!({"type":"item.started","item":{"id":"i","type":"other"}}),
            "codex",
        );
        assert!(e.is_empty());
        let e = parse_common_json(&json!({"type":"thread.started"}), "codex");
        assert!(e.is_empty());
        let e = parse_common_json(&json!({"type":"step-start","sessionId":"s2"}), "opencode");
        assert!(e
            .iter()
            .any(|x| matches!(x, Event::SessionInfo { id, .. } if id == "s2")));
        let e = parse_common_json(&json!({"type":"step_end","reason":"done"}), "opencode");
        assert!(matches!(e.first(), Some(Event::TurnComplete { .. })));
        let e = parse_common_json(&json!({"type":"system"}), "claude");
        assert!(matches!(e.first(), Some(Event::Raw { .. })));
        let e = parse_common_json(&json!({"type":"unknown_evt","session_id":"sid-x"}), "x");
        // Unknown types must not scavenge session_id into SessionInfo.
        assert!(e.iter().all(|x| !matches!(x, Event::SessionInfo { .. })));
        assert!(matches!(e.first(), Some(Event::Raw { .. })));
    }

    #[test]
    fn result_and_session_update() {
        let e = parse_common_json(
            &json!({"type":"result","sessionId":"s","result":"done","stopReason":"end"}),
            "t",
        );
        assert!(e.iter().any(|x| matches!(x, Event::TurnComplete { .. })));
        let e = parse_common_json(
            &json!({
                "method":"session/update",
                "params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"text":"hi"}}}
            }),
            "t",
        );
        assert!(matches!(e.first(), Some(Event::TextDelta { text }) if text == "hi"));
        let e = parse_common_json(
            &json!({
                "method":"session/update",
                "params":{"update":{"sessionUpdate":"agent_thought_chunk","text":"think"}}
            }),
            "t",
        );
        assert!(matches!(e.first(), Some(Event::ThinkingDelta { text }) if text == "think"));
        let e = parse_common_json(
            &json!({
                "method":"session/update",
                "params":{"update":{
                    "sessionUpdate":"tool_call",
                    "toolCallId":"t1",
                    "title":"run",
                    "rawInput":{"a":1}
                }}
            }),
            "t",
        );
        assert!(matches!(e.first(), Some(Event::ToolCall { name, .. }) if name == "run"));
        let e = parse_common_json(
            &json!({
                "method":"session/update",
                "params":{"update":{
                    "sessionUpdate":"tool_call_update",
                    "toolCallId":"t1",
                    "name":"run",
                    "status":"completed",
                    "content":"out"
                }}
            }),
            "t",
        );
        assert!(matches!(
            e.first(),
            Some(Event::ToolResult {
                is_error: false,
                ..
            })
        ));
        let e = parse_common_json(
            &json!({
                "method":"session/update",
                "params":{"update":{
                    "sessionUpdate":"tool_call_update",
                    "id":"t1",
                    "status":"failed"
                }}
            }),
            "t",
        );
        assert!(matches!(
            e.first(),
            Some(Event::ToolResult { is_error: true, .. })
        ));
        let e = parse_common_json(
            &json!({"method":"session/update","params":{"update":{"sessionUpdate":"other"}}}),
            "t",
        );
        assert!(matches!(e.first(), Some(Event::Raw { channel, .. }) if channel == "acp"));
    }

    #[test]
    fn empty_thinking_session_permission_detail() {
        assert!(parse_common_json(&json!({"type":"thinking","data":""}), "t").is_empty());
        let e = parse_common_json(&json!({"type":"session"}), "t");
        assert!(matches!(e.first(), Some(Event::Raw { .. })));
        let e = parse_common_json(&json!({"type":"end","result":"out"}), "t");
        assert!(e
            .iter()
            .any(|x| matches!(x, Event::TextDelta { text } if text == "out")));
        let e = parse_common_json(
            &json!({"type":"permission","id":"p","tool":"t","command":{"x":1}}),
            "t",
        );
        assert!(matches!(e.first(), Some(Event::PermissionRequest { .. })));
        assert!(parse_common_json(
            &json!({"method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"text":""}}}}),
            "t",
        )
        .is_empty());
    }
}
