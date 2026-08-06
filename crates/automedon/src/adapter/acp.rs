//! Agent Client Protocol helpers (JSON-RPC 2.0 over stdio).
//!
//! Session drives the handshake when `LaunchOptions.extra.acp` is true:
//! `initialize` → `authenticate` (optional) → `session/new` → `session/prompt` per turn.

use serde_json::{json, Value};

use crate::event::Event;

use super::shared_parse;

/// Build a JSON-RPC request line (no trailing newline).
pub fn request(id: u64, method: &str, params: Value) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    })
    .to_string()
}

/// Build a JSON-RPC notification (no id).
pub fn notification(method: &str, params: Value) -> String {
    json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
    })
    .to_string()
}

pub fn initialize_params(client_name: &str) -> Value {
    json!({
        "protocolVersion": 1,
        "clientInfo": { "name": client_name, "version": env!("CARGO_PKG_VERSION") },
        "capabilities": {}
    })
}

pub fn authenticate_params(method_id: &str) -> Value {
    json!({ "methodId": method_id })
}

/// Grok (and similar) require `mcpServers` on session/new.
pub fn session_new_params(cwd: Option<&str>) -> Value {
    let mut p = json!({ "mcpServers": [] });
    if let Some(c) = cwd {
        p["cwd"] = json!(c);
    }
    p
}

pub fn session_prompt_params(session_id: &str, text: &str) -> Value {
    json!({
        "sessionId": session_id,
        "prompt": [{ "type": "text", "text": text }]
    })
}

/// Parse one ACP / agent NDJSON line into normalized events.
///
/// Also maps JSON-RPC **results** for `session/prompt` (matched by caller via id)
/// into `TurnComplete`, and extracts `sessionId` from `session/new` results.
pub fn parse_line(line: &str) -> Vec<Event> {
    let line = line.trim();
    if line.is_empty() {
        return Vec::new();
    }
    let Ok(v) = serde_json::from_str::<Value>(line) else {
        return vec![Event::Raw {
            channel: "acp".into(),
            line: line.to_string(),
        }];
    };
    parse_value(&v)
}

pub fn parse_value(v: &Value) -> Vec<Event> {
    // JSON-RPC response (has id + result/error, no method)
    if v.get("method").is_none() && v.get("id").is_some() {
        if let Some(err) = v.get("error") {
            return vec![Event::Error {
                message: err
                    .get("message")
                    .and_then(|m| m.as_str())
                    .map(str::to_string)
                    .unwrap_or_else(|| err.to_string()),
            }];
        }
        if let Some(result) = v.get("result") {
            let mut out = Vec::new();
            if let Some(sid) = result
                .get("sessionId")
                .or_else(|| result.get("session_id"))
                .and_then(|s| s.as_str())
            {
                out.push(Event::SessionInfo {
                    id: sid.to_string(),
                    label: None,
                });
            }
            // session/prompt result carries stopReason (turn end).
            if result.get("stopReason").is_some() || result.get("stop_reason").is_some() {
                out.push(Event::TurnComplete {
                    turn: 0,
                    stop_reason: result
                        .get("stopReason")
                        .or_else(|| result.get("stop_reason"))
                        .and_then(|s| s.as_str())
                        .map(str::to_string),
                });
            }
            return out;
        }
        return Vec::new();
    }

    let method = v.get("method").and_then(|m| m.as_str()).unwrap_or("");

    // xAI hook lifecycle (extends ACP notifications)
    if method == "_x.ai/session_notification" || method.ends_with("session_notification") {
        let update = v.get("params").and_then(|p| p.get("update")).unwrap_or(v);
        let su = update
            .get("sessionUpdate")
            .and_then(|s| s.as_str())
            .unwrap_or("");
        if su == "hook_execution" || su.contains("hook") {
            let name = update
                .get("event_name")
                .or_else(|| update.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("hook")
                .to_string();
            return vec![Event::HookStarted {
                id: update
                    .get("prompt_id")
                    .and_then(|p| p.as_str())
                    .unwrap_or("hook")
                    .to_string(),
                name,
                phase: update
                    .get("event_name")
                    .and_then(|p| p.as_str())
                    .map(str::to_string),
                detail: Some(update.clone()),
            }];
        }
        if su == "turn_completed" {
            return vec![Event::TurnComplete {
                turn: 0,
                stop_reason: update
                    .get("stop_reason")
                    .and_then(|s| s.as_str())
                    .map(str::to_string),
            }];
        }
        // Fall through: try common parse on nested update
        if update.get("sessionUpdate").is_some() {
            return shared_parse::parse_common_json(
                &json!({
                    "method": "session/update",
                    "params": { "update": update }
                }),
                "acp",
            );
        }
    }

    shared_parse::parse_common_json(v, "acp")
}

/// True when this JSON-RPC object is a response to `id`.
pub fn is_response_for(v: &Value, id: u64) -> bool {
    v.get("id")
        .and_then(|i| i.as_u64().or_else(|| i.as_i64().map(|x| x as u64)))
        == Some(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_shape() {
        let s = request(1, "initialize", initialize_params("automedon"));
        let v: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["method"], "initialize");
        assert_eq!(v["id"], 1);
    }

    #[test]
    fn session_new_requires_mcp_servers() {
        let p = session_new_params(Some("/tmp"));
        assert!(p.get("mcpServers").is_some());
        assert_eq!(p["cwd"], "/tmp");
    }

    #[test]
    fn parse_session_update_text() {
        let line = r#"{"method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"text":"hi"}}}}"#;
        let ev = parse_line(line);
        assert!(matches!(ev.first(), Some(Event::TextDelta { text }) if text == "hi"));
    }

    #[test]
    fn parse_prompt_result_turn_complete() {
        let line = r#"{"jsonrpc":"2.0","id":10,"result":{"stopReason":"end_turn","_meta":{"promptId":"p1"}}}"#;
        let ev = parse_line(line);
        assert!(ev.iter().any(|e| matches!(e, Event::TurnComplete { .. })));
    }

    #[test]
    fn parse_session_new_result() {
        let line = r#"{"jsonrpc":"2.0","id":3,"result":{"sessionId":"s-1"}}"#;
        let ev = parse_line(line);
        assert!(matches!(ev.first(), Some(Event::SessionInfo { id, .. }) if id == "s-1"));
    }

    #[test]
    fn notification_and_session_params() {
        let n = notification("session/update", json!({"x": 1}));
        assert!(n.contains("session/update"));
        let sp = session_prompt_params("sid", "hello");
        assert_eq!(sp["sessionId"], "sid");
        assert!(parse_line("").is_empty());
        assert!(matches!(
            parse_line("not-json").first(),
            Some(Event::Raw { .. })
        ));
        assert!(is_response_for(
            &json!({"jsonrpc":"2.0","id":7,"result":{}}),
            7
        ));
    }

    #[test]
    fn parse_rpc_error_and_hooks() {
        let ev = parse_line(r#"{"jsonrpc":"2.0","id":3,"error":{"message":"nope"}}"#);
        assert!(matches!(ev.first(), Some(Event::Error { message }) if message == "nope"));
        let ev = parse_line(
            r#"{"method":"_x.ai/session_notification","params":{"update":{"sessionUpdate":"hook_execution","event_name":"user_prompt_submit","prompt_id":"p1"}}}"#,
        );
        assert!(matches!(
            ev.first(),
            Some(Event::HookStarted { name, .. }) if name == "user_prompt_submit"
        ));
        let ev = parse_line(
            r#"{"method":"_x.ai/session_notification","params":{"update":{"sessionUpdate":"turn_completed","stop_reason":"end_turn"}}}"#,
        );
        assert!(matches!(ev.first(), Some(Event::TurnComplete { .. })));
        let ev = parse_line(
            r#"{"method":"_x.ai/session_notification","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"text":"z"}}}}"#,
        );
        assert!(matches!(ev.first(), Some(Event::TextDelta { text }) if text == "z"));
        assert!(authenticate_params("cached_token")["methodId"] == "cached_token");
        assert!(parse_value(&json!({"jsonrpc":"2.0","id":1,"result":{}})).is_empty());
    }
}
