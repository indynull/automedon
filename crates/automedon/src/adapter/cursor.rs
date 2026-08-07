//! Cursor agent CLI specialized driver (`cursor-agent`; bare `agent` is last resort).
//!
//! Prefer `cursor-agent` over bare `agent` — Grok Build also installs an `agent` binary.
//! Headless: `-p` + `--output-format stream-json`. Multi-turn: `--resume` / `--continue`.
//!
//! Live stream shapes (2026 cursor-agent): `system` / `thinking` / `assistant` /
//! `tool_call` (`editToolCall`, `shellToolCall`, …) / `result`.

use std::path::PathBuf;

use serde_json::Value;

use super::{
    base_env, resolve_bin, shared_parse, Adapter, Capabilities, PreparedLaunch, TurnContext,
};
use crate::config::LaunchOptions;
use crate::error::Result;
use crate::event::Event;
use crate::transport::SpawnSpec;

#[derive(Debug, Default, Clone)]
pub struct CursorAdapter;

impl Adapter for CursorAdapter {
    fn name(&self) -> &'static str {
        "cursor"
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
        let (program, mut args) = resolve_cursor_bin(opts, prompt);

        // cursor-agent: -p/--print + --output-format stream-json
        if !args.iter().any(|a| a == "--print" || a == "-p") {
            args.insert(0, "--print".into());
        }
        args.push("--output-format".into());
        args.push("stream-json".into());
        // Partial streaming emits both token deltas and a full assistant frame, which
        // doubles transcript text (MARKERMARKER). Off by default; set stream_partial: true
        // when you want live token deltas.
        if opts
            .extra
            .get("stream_partial")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            args.push("--stream-partial-output".into());
        }
        if opts.yolo {
            // --yolo is alias for --force on current agent builds.
            if !args.iter().any(|a| a == "--force" || a == "--yolo") {
                args.push("--force".into());
            }
        }
        if ctx.turn > 1 {
            if let Some(id) = ctx.session_id.as_ref().filter(|s| !s.is_empty()) {
                args.push("--resume".into());
                args.push(id.clone());
            } else {
                args.push("--continue".into());
            }
        } else if let Some(id) = opts.extra.get("resume").and_then(|v| v.as_str()) {
            args.push("--resume".into());
            args.push(id.into());
        }
        if let Some(model) = &opts.model {
            args.push("--model".into());
            args.push(model.clone());
        }

        Ok(PreparedLaunch {
            harness: "cursor".into(),
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
        // Plain-text auth errors
        if line.contains("Authentication required")
            || line.contains("agent login")
            || line.contains("cursor-agent login")
        {
            return vec![Event::Error {
                message: line.to_string(),
            }];
        }
        match serde_json::from_str::<Value>(line) {
            Ok(v) => parse_cursor_json(&v),
            Err(_) => vec![Event::Raw {
                channel: "cursor".into(),
                line: line.to_string(),
            }],
        }
    }
}

fn parse_cursor_json(value: &Value) -> Vec<Event> {
    let ty = value.get("type").and_then(|t| t.as_str()).unwrap_or("");
    match ty {
        "system" => {
            // init carries session_id for multi-turn resume
            if let Some(sid) = value
                .get("session_id")
                .or_else(|| value.get("sessionId"))
                .and_then(|s| s.as_str())
                .filter(|s| !s.is_empty())
            {
                vec![Event::SessionInfo {
                    id: sid.to_string(),
                    label: Some("cursor".into()),
                }]
            } else {
                Vec::new()
            }
        }
        "thinking" => {
            let text = value
                .get("text")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            if text.is_empty() {
                Vec::new()
            } else {
                vec![Event::ThinkingDelta { text }]
            }
        }
        "assistant" => {
            // Streamed content blocks: [{ "type":"text", "text":"..." }, ...]
            let mut out = Vec::new();
            if let Some(sid) = value
                .get("session_id")
                .and_then(|s| s.as_str())
                .filter(|s| !s.is_empty())
            {
                out.push(Event::SessionInfo {
                    id: sid.to_string(),
                    label: Some("cursor".into()),
                });
            }
            if let Some(arr) = value.pointer("/message/content").and_then(|c| c.as_array()) {
                for block in arr {
                    if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                        if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                            if !t.is_empty() {
                                out.push(Event::TextDelta {
                                    text: t.to_string(),
                                });
                            }
                        }
                    }
                }
            }
            out
        }
        "tool_call" => parse_cursor_tool_call(value),
        "result" => {
            // Final frame: session id + turn complete.
            // Do not re-emit `result` string as TextDelta — assistant already streamed text
            // (re-emitting doubles markers like CURSOR_WS_T1CURSOR_WS_T1).
            let mut out = Vec::new();
            if let Some(sid) = value
                .get("session_id")
                .or_else(|| value.get("sessionId"))
                .and_then(|s| s.as_str())
                .filter(|s| !s.is_empty())
            {
                out.push(Event::SessionInfo {
                    id: sid.to_string(),
                    label: Some("cursor".into()),
                });
            }
            if value.get("is_error").and_then(|b| b.as_bool()) == Some(true) {
                let msg = value
                    .get("result")
                    .and_then(|r| r.as_str())
                    .unwrap_or("cursor result error")
                    .to_string();
                out.push(Event::Error { message: msg });
            }
            out.push(Event::TurnComplete {
                turn: 1,
                stop_reason: value
                    .get("subtype")
                    .and_then(|s| s.as_str())
                    .map(str::to_string),
            });
            out
        }
        "user" => Vec::new(),
        _ => shared_parse::parse_common_json(value, "cursor"),
    }
}

/// Cursor nests tools as `tool_call: { editToolCall: { args, result }, toolCallId, ... }`.
fn parse_cursor_tool_call(value: &Value) -> Vec<Event> {
    let subtype = value.get("subtype").and_then(|s| s.as_str()).unwrap_or("");
    let id = value
        .get("call_id")
        .or_else(|| value.get("callId"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            value
                .pointer("/tool_call/toolCallId")
                .and_then(|v| v.as_str())
        })
        .unwrap_or("")
        .to_string();

    let Some(tc) = value.get("tool_call") else {
        return vec![Event::Raw {
            channel: "cursor".into(),
            line: value.to_string(),
        }];
    };

    let mut out = Vec::new();
    if let Some(sid) = value
        .get("session_id")
        .and_then(|s| s.as_str())
        .filter(|s| !s.is_empty())
    {
        out.push(Event::SessionInfo {
            id: sid.to_string(),
            label: Some("cursor".into()),
        });
    }

    let (name, body) =
        extract_cursor_tool_body(tc).unwrap_or_else(|| ("unknown".to_string(), tc.clone()));
    let input = body.get("args").cloned().unwrap_or(Value::Null);

    let completed = subtype == "completed" || body.get("result").is_some();
    if completed {
        let result = body.get("result").unwrap_or(&Value::Null);
        let is_error = result.get("failure").is_some()
            || result
                .pointer("/success/exitCode")
                .and_then(|c| c.as_i64())
                .is_some_and(|c| c != 0);
        let output = if let Some(s) = result
            .pointer("/success/stdout")
            .or_else(|| result.pointer("/success/interleavedOutput"))
            .or_else(|| result.pointer("/success/message"))
            .or_else(|| result.pointer("/success/diffString"))
            .or_else(|| result.pointer("/success/afterFullFileContent"))
            .and_then(|v| v.as_str())
        {
            s.to_string()
        } else {
            result.to_string()
        };
        out.push(Event::ToolResult {
            id,
            name,
            output,
            is_error,
        });
    } else {
        out.push(Event::ToolCall { id, name, input });
    }
    out
}

/// Find nested `*ToolCall` object and a short tool name (`edit`, `shell`, …).
fn extract_cursor_tool_body(tc: &Value) -> Option<(String, Value)> {
    let obj = tc.as_object()?;
    for (key, body) in obj {
        if key.ends_with("ToolCall") && body.is_object() {
            let name = cursor_tool_name(key);
            return Some((name, body.clone()));
        }
    }
    // Fallback: toolCallId-only wrapper
    None
}

fn cursor_tool_name(key: &str) -> String {
    let base = key.strip_suffix("ToolCall").unwrap_or(key);
    // editToolCall → edit, shellToolCall → shell, readFileToolCall → readFile
    if base.is_empty() {
        return "unknown".into();
    }
    // Prefer lowercase short names for common tools
    match base {
        "edit" | "Edit" => "edit".into(),
        "shell" | "Shell" => "shell".into(),
        "read" | "Read" | "readFile" | "ReadFile" => "read".into(),
        "grep" | "Grep" => "grep".into(),
        "delete" | "Delete" => "delete".into(),
        "ls" | "Ls" | "list" | "List" => "ls".into(),
        other => {
            // camelCase → keep as-is with first letter lowercased
            let mut c = other.chars();
            match c.next() {
                None => "unknown".into(),
                Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
            }
        }
    }
}

/// Prefer explicit `extra.binary` / `opts.bin`, then `cursor-agent`, then bare `agent`, then `cursor agent`.
fn resolve_cursor_bin(opts: &LaunchOptions, prompt: &str) -> (PathBuf, Vec<String>) {
    if let Some(bin) = opts.extra.get("binary").and_then(|v| v.as_str()) {
        let program = resolve_bin(opts, bin);
        let args = if bin == "cursor" {
            vec!["agent".into(), "-p".into(), prompt.to_string()]
        } else {
            vec!["-p".into(), prompt.to_string()]
        };
        return (program, args);
    }
    if opts.bin.is_some() {
        return (
            resolve_bin(opts, "cursor-agent"),
            vec!["-p".into(), prompt.to_string()],
        );
    }
    // PATH preference: cursor-agent first (unambiguous), then bare agent, then `cursor agent`.
    for name in ["cursor-agent", "agent"] {
        if which_on_path(name) {
            return (PathBuf::from(name), vec!["-p".into(), prompt.to_string()]);
        }
    }
    (
        PathBuf::from("cursor"),
        vec!["agent".into(), "-p".into(), prompt.to_string()],
    )
}

fn which_on_path(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| {
            std::env::split_paths(&p).any(|dir| {
                let c = dir.join(name);
                c.is_file()
            })
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::TurnContext;
    use serde_json::json;

    #[test]
    fn parse_live_edit_tool_call_and_result() {
        let a = CursorAdapter;
        let start = a.parse_line(
            r#"{"type":"tool_call","subtype":"started","call_id":"c1","tool_call":{"editToolCall":{"args":{"path":"/tmp/x.txt","streamContent":"PROBE\n"}},"toolCallId":"c1"},"session_id":"s1"}"#,
        );
        assert!(
            start
                .iter()
                .any(|e| matches!(e, Event::ToolCall { name, input, .. }
                    if name == "edit" && input.to_string().contains("PROBE"))),
            "{start:?}"
        );
        assert!(start
            .iter()
            .any(|e| matches!(e, Event::SessionInfo { id, .. } if id == "s1")));

        let done = a.parse_line(
            r#"{"type":"tool_call","subtype":"completed","call_id":"c1","tool_call":{"editToolCall":{"args":{"path":"/tmp/x.txt","streamContent":"PROBE\n"},"result":{"success":{"path":"/tmp/x.txt","message":"Wrote contents","afterFullFileContent":"PROBE\n","diffString":"+PROBE"}}},"toolCallId":"c1"},"session_id":"s1"}"#,
        );
        assert!(
            done.iter().any(|e| matches!(
                e,
                Event::ToolResult {
                    name,
                    is_error: false,
                    output,
                    ..
                } if name == "edit" && (output.contains("PROBE") || output.contains("Wrote"))
            )),
            "{done:?}"
        );
    }

    #[test]
    fn parse_live_shell_tool_call() {
        let a = CursorAdapter;
        let start = a.parse_line(
            r#"{"type":"tool_call","subtype":"started","call_id":"c2","tool_call":{"shellToolCall":{"args":{"command":"echo SHELL_PROBE_OK"}},"toolCallId":"c2"},"session_id":"s2"}"#,
        );
        assert!(
            start
                .iter()
                .any(|e| matches!(e, Event::ToolCall { name, input, .. }
                    if name == "shell" && input.to_string().contains("SHELL_PROBE_OK"))),
            "{start:?}"
        );
        let done = a.parse_line(
            r#"{"type":"tool_call","subtype":"completed","call_id":"c2","tool_call":{"shellToolCall":{"args":{"command":"echo SHELL_PROBE_OK"},"result":{"success":{"exitCode":0,"stdout":"SHELL_PROBE_OK\n","interleavedOutput":"SHELL_PROBE_OK\n"}}},"toolCallId":"c2"}}"#,
        );
        assert!(
            done.iter().any(|e| matches!(
                e,
                Event::ToolResult {
                    name,
                    is_error: false,
                    output,
                    ..
                } if name == "shell" && output.contains("SHELL_PROBE_OK")
            )),
            "{done:?}"
        );
    }

    #[test]
    fn parse_system_and_result_turn_complete_no_text_double() {
        let a = CursorAdapter;
        let sys = a.parse_line(
            r#"{"type":"system","subtype":"init","session_id":"sess-abc","model":"Auto"}"#,
        );
        assert!(matches!(
            sys.first(),
            Some(Event::SessionInfo { id, .. }) if id == "sess-abc"
        ));
        let res = a.parse_line(
            r#"{"type":"result","subtype":"success","is_error":false,"result":"DONE_PROBE","session_id":"sess-abc"}"#,
        );
        assert!(res.iter().any(|e| matches!(e, Event::TurnComplete { .. })));
        assert!(
            !res.iter().any(|e| matches!(e, Event::TextDelta { .. })),
            "result must not re-emit full text: {res:?}"
        );
        assert!(res
            .iter()
            .any(|e| matches!(e, Event::SessionInfo { id, .. } if id == "sess-abc")));
    }

    #[test]
    fn parse_assistant_text_and_thinking() {
        let a = CursorAdapter;
        let th = a.parse_line(
            r#"{"type":"thinking","subtype":"delta","text":"planning","session_id":"s"}"#,
        );
        assert!(matches!(
            th.first(),
            Some(Event::ThinkingDelta { text }) if text == "planning"
        ));
        let asst = a.parse_line(
            r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"hello"}]},"session_id":"s"}"#,
        );
        assert!(
            asst.iter()
                .any(|e| matches!(e, Event::TextDelta { text } if text == "hello")),
            "{asst:?}"
        );
    }

    #[test]
    fn resolve_prefers_cursor_agent_name_when_present() {
        // unit-level: explicit binary wins
        let a = CursorAdapter;
        let mut opts = LaunchOptions::default();
        opts.extra.insert("binary".into(), json!("cursor-agent"));
        let p = a
            .prepare("hi", &opts, &TurnContext::default())
            .unwrap()
            .spawn
            .unwrap();
        assert!(
            p.program.ends_with("cursor-agent") || p.program.as_os_str() == "cursor-agent",
            "{:?}",
            p.program
        );
        assert!(p.args.iter().any(|x| x == "-p" || x == "--print"));
        assert!(p
            .args
            .windows(2)
            .any(|w| w[0] == "--output-format" && w[1] == "stream-json"));
    }

    #[test]
    fn prepare_yolo_resume_continue_model_and_partial() {
        let a = CursorAdapter;
        let mut opts = LaunchOptions {
            yolo: true,
            model: Some("sonnet".into()),
            ..Default::default()
        };
        opts.extra.insert("binary".into(), json!("cursor-agent"));
        opts.extra.insert("stream_partial".into(), json!(true));
        let args = a
            .prepare("hi", &opts, &TurnContext::default())
            .unwrap()
            .spawn
            .unwrap()
            .args;
        assert!(args.iter().any(|x| x == "--force"));
        assert!(args.iter().any(|x| x == "--stream-partial-output"));
        assert!(args
            .windows(2)
            .any(|w| w[0] == "--model" && w[1] == "sonnet"));

        let mut ctx = TurnContext {
            turn: 2,
            session_id: Some("sess-1".into()),
            ..Default::default()
        };
        let args = a.prepare("again", &opts, &ctx).unwrap().spawn.unwrap().args;
        assert!(args
            .windows(2)
            .any(|w| w[0] == "--resume" && w[1] == "sess-1"));

        ctx.session_id = None;
        let args = a.prepare("again", &opts, &ctx).unwrap().spawn.unwrap().args;
        assert!(args.iter().any(|x| x == "--continue"));

        opts.extra.insert("resume".into(), json!("from-extra"));
        let args = a
            .prepare("hi", &opts, &TurnContext::default())
            .unwrap()
            .spawn
            .unwrap()
            .args;
        assert!(args
            .windows(2)
            .any(|w| w[0] == "--resume" && w[1] == "from-extra"));
    }

    #[test]
    fn parse_edges_auth_raw_error_result_and_names() {
        let a = CursorAdapter;
        assert!(matches!(
            a.parse_line("Authentication required. Please run 'agent login'")
                .first(),
            Some(Event::Error { .. })
        ));
        assert!(matches!(
            a.parse_line("not-json-line").first(),
            Some(Event::Raw { .. })
        ));
        assert!(a.parse_line("").is_empty());
        assert!(a
            .parse_line(r#"{"type":"thinking","subtype":"delta","text":""}"#)
            .is_empty());
        assert!(a.parse_line(r#"{"type":"user","message":{}}"#).is_empty());
        assert!(a
            .parse_line(r#"{"type":"system","subtype":"init"}"#)
            .is_empty());
        let err = a.parse_line(
            r#"{"type":"result","subtype":"error","is_error":true,"result":"boom","session_id":"s"}"#,
        );
        assert!(err.iter().any(|e| matches!(e, Event::Error { .. })));
        assert!(err.iter().any(|e| matches!(e, Event::TurnComplete { .. })));

        let bare = a.parse_line(r#"{"type":"tool_call","subtype":"started","call_id":"c"}"#);
        assert!(
            bare.iter().any(|e| matches!(e, Event::Raw { .. }))
                || bare
                    .iter()
                    .any(|e| matches!(e, Event::ToolCall { name, .. } if name == "unknown")),
            "{bare:?}"
        );

        assert_eq!(cursor_tool_name("editToolCall"), "edit");
        assert_eq!(cursor_tool_name("shellToolCall"), "shell");
        assert_eq!(cursor_tool_name("readFileToolCall"), "read");
        assert_eq!(cursor_tool_name("grepToolCall"), "grep");
        assert_eq!(cursor_tool_name("deleteToolCall"), "delete");
        assert_eq!(cursor_tool_name("lsToolCall"), "ls");
        assert_eq!(cursor_tool_name("weirdThingToolCall"), "weirdThing");
        assert_eq!(cursor_tool_name("ToolCall"), "unknown");
    }

    #[test]
    fn prepare_cursor_ide_binary_uses_agent_subcommand() {
        let a = CursorAdapter;
        let mut opts = LaunchOptions::default();
        opts.extra.insert("binary".into(), json!("cursor"));
        let p = a
            .prepare("hi", &opts, &TurnContext::default())
            .unwrap()
            .spawn
            .unwrap();
        assert_eq!(p.program, PathBuf::from("cursor"));
        assert!(p.args.iter().any(|x| x == "agent"));
    }
}
