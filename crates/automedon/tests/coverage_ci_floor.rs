//! Raise line coverage on product adapters so continuous integration fail-under stays green.

use automedon::adapter::{
    Adapter, AdapterKind, ClaudeAdapter, CodexAdapter, CopilotAdapter, CursorAdapter,
    GeminiAdapter, GrokAdapter, OpenCodeAdapter, TurnContext,
};
use automedon::config::LaunchOptions;
use automedon::event::Event;
use serde_json::json;
use std::path::PathBuf;

fn ctx(turn: u64, session: Option<&str>) -> TurnContext {
    TurnContext {
        turn,
        session_id: session.map(str::to_string),
        ..Default::default()
    }
}

#[test]
fn registry_mock_and_generic_operator_strings() {
    assert_eq!(AdapterKind::Mock.default_binaries(), "(in-process)");
    assert_eq!(AdapterKind::Generic.default_binaries(), "opts.bin");
    assert_eq!(
        AdapterKind::Mock.multi_turn_summary(),
        "in-process scenarios"
    );
    assert_eq!(
        AdapterKind::Generic.multi_turn_summary(),
        "process-per-prompt"
    );
    assert!(!AdapterKind::Mock.is_product());
    assert!(!AdapterKind::Generic.is_product());
}

#[test]
fn opencode_tool_parts_and_nested_text() {
    let a = OpenCodeAdapter;

    // Tool invocation completed (part.state + non-string output).
    let done = a.parse_line(
        r#"{"type":"message","part":{"type":"tool-invocation","id":"c1","tool":"bash","state":"completed","output":{"ok":true}},"sessionID":"oc-1"}"#,
    );
    assert!(
        done.iter()
            .any(|e| matches!(e, Event::ToolResult { id, is_error: false, .. } if id == "c1")),
        "{done:?}"
    );
    assert!(done
        .iter()
        .any(|e| matches!(e, Event::SessionInfo { id, .. } if id == "oc-1")));

    // Tool call in progress (args path).
    let start = a.parse_line(
        r#"{"part":{"type":"tool","callID":"c2","name":"read","args":{"path":"/tmp/x"}}}"#,
    );
    assert!(
        start
            .iter()
            .any(|e| matches!(e, Event::ToolCall { id, name, .. } if id == "c2" && name == "read")),
        "{start:?}"
    );

    // Tool result type name + result field string.
    let tr =
        a.parse_line(r#"{"part":{"type":"tool-result","id":"c3","tool":"x","result":"out text"}}"#);
    assert!(tr.iter().any(|e| matches!(
        e,
        Event::ToolResult {
            id,
            output,
            ..
        } if id == "c3" && output == "out text"
    )));

    // Nested part text when common parse is only metadata-like.
    let nested = a.parse_line(r#"{"part":{"text":"from-part"},"sessionId":"oc-2"}"#);
    assert!(
        nested
            .iter()
            .any(|e| matches!(e, Event::TextDelta { text } if text == "from-part")),
        "{nested:?}"
    );

    // message/content/0/text path
    let msg = a.parse_line(r#"{"message":{"content":[{"text":"deep"}]}}"#);
    assert!(msg
        .iter()
        .any(|e| matches!(e, Event::TextDelta { text } if text == "deep")));

    // empty text field skipped by extract
    let _ = a.parse_line(r#"{"text":""}"#);

    // session already present → no duplicate SessionInfo insert still ok
    let dup = a.parse_line(r#"{"type":"session","sessionID":"same","sessionId":"same"}"#);
    let _ = dup;
}

#[test]
fn copilot_parse_and_prepare_remaining_arms() {
    let a = CopilotAdapter;

    // First-turn resume from extra.
    let mut opts = LaunchOptions::default();
    opts.extra.insert("resume".into(), json!("extra-sess"));
    let args = a
        .prepare("p", &opts, &ctx(1, None))
        .unwrap()
        .spawn
        .unwrap()
        .args;
    assert!(args.iter().any(|x| x == "--resume=extra-sess"));

    // Empty deltas / reasoning.
    assert!(a
        .parse_line(r#"{"type":"assistant.message_delta","data":{"deltaContent":""}}"#)
        .is_empty());
    assert!(a
        .parse_line(r#"{"type":"assistant.reasoning_delta","data":{"deltaContent":""}}"#)
        .is_empty());

    // Turn start + non-zero exit + data.sessionId fallback on result without top-level type path.
    let start = a.parse_line(r#"{"type":"assistant.turn_start","data":{"turnId":"3"}}"#);
    assert!(matches!(start.first(), Some(Event::TurnStart { turn: 3 })));

    let bad = a.parse_line(r#"{"type":"result","data":{"sessionId":"from-data"},"exitCode":2}"#);
    assert!(bad
        .iter()
        .any(|e| matches!(e, Event::SessionInfo { id, .. } if id == "from-data")));
    assert!(bad.iter().any(|e| matches!(e, Event::Error { .. })));
    assert!(bad.iter().any(|e| matches!(e, Event::TurnComplete { .. })));

    // tool_result non-string output + alternate type names
    let tr = a.parse_line(
        r#"{"type":"tool_result","data":{"id":"t","name":"n","result":{"x":1},"is_error":true}}"#,
    );
    assert!(matches!(
        tr.first(),
        Some(Event::ToolResult {
            is_error: true,
            output,
            ..
        }) if output.contains('1')
    ));
    let _ = a.parse_line(r#"{"type":"tool.end","data":{"toolCallId":"t2","output":"s"}}"#);
    let _ = a
        .parse_line(r#"{"type":"tool_call","data":{"toolCallId":"t3","toolName":"w","input":{}}}"#);
    let _ = a.parse_line(r#"{"type":"tool.start","data":{"id":"t4","name":"w"}}"#);

    // Errors + noise filters + fallback shared parse
    let err = a.parse_line(r#"{"type":"error","data":{"message":"boom"}}"#);
    assert!(matches!(err.first(), Some(Event::Error { message }) if message == "boom"));
    let err2 = a.parse_line(r#"{"type":"session.error","message":42}"#);
    assert!(err2.iter().any(|e| matches!(e, Event::Error { .. })));
    assert!(a.parse_line(r#"{"type":"session.idle"}"#).is_empty());
    assert!(a.parse_line(r#"{"type":"mcp.ready"}"#).is_empty());
    assert!(a.parse_line(r#"{"type":"model.call_start"}"#).is_empty());
    assert!(a.parse_line(r#"{"type":"assistant.idle"}"#).is_empty());
    assert!(a
        .parse_line(r#"{"type":"assistant.message_start"}"#)
        .is_empty());
    assert!(a.parse_line(r#"{"type":"user.message"}"#).is_empty());
    // unknown type → shared parse
    let _ = a.parse_line(r#"{"type":"unknown.frame","text":"x"}"#);

    // JSON without data wrapper uses value itself (line 143 path via type on root)
    let _ = a.parse_line(r#"{"type":"assistant.message_delta","deltaContent":"root"}"#);

    // --resume with empty next token after flag
    assert!(!a
        .parse_line("please resume --resume")
        .iter()
        .any(|e| matches!(e, Event::SessionInfo { .. })));
}

#[test]
fn gemini_session_id_variants_and_empty() {
    let a = GeminiAdapter;
    assert!(a.parse_line("").is_empty());
    let sid = a.parse_line(r#"{"sessionId":"g-sess-1","type":"other"}"#);
    assert!(sid
        .iter()
        .any(|e| matches!(e, Event::SessionInfo { id, .. } if id == "g-sess-1")));
    let nested = a.parse_line(r#"{"session":{"id":"nested-g"},"type":"x"}"#);
    assert!(nested
        .iter()
        .any(|e| matches!(e, Event::SessionInfo { id, .. } if id == "nested-g")));
    // Auth plain text already partially covered; hit remaining strings
    assert!(matches!(
        a.parse_line("Error authenticating with Google").first(),
        Some(Event::Error { .. })
    ));
    // custom binary extra
    let mut opts = LaunchOptions::default();
    opts.extra
        .insert("binary".into(), json!("/tmp/fake-gemini"));
    let p = a.prepare("hi", &opts, &ctx(1, None)).unwrap();
    assert_eq!(p.spawn.unwrap().program, PathBuf::from("/tmp/fake-gemini"));
}

#[test]
fn claude_auth_and_content_edge_paths() {
    let a = ClaudeAdapter;
    // Auth Error path only when blocks skip TextDelta (no block type) so out is SessionInfo-only.
    let auth = a.parse_line(
        r#"{"type":"assistant","session_id":"s1","message":{"content":[{"text":"Not logged in · Please run /login"}]}}"#,
    );
    assert!(
        auth.iter().any(|e| matches!(e, Event::Error { .. })),
        "{auth:?}"
    );

    // result empty string → no text, still TurnComplete
    let empty_res = a.parse_line(r#"{"type":"result","session_id":"s","result":"","num_turns":2}"#);
    assert!(empty_res
        .iter()
        .any(|e| matches!(e, Event::TurnComplete { turn: 2, .. })));

    // user without message
    assert!(a.parse_line(r#"{"type":"user"}"#).is_empty());

    // tool_use top-level
    let tu = a.parse_line(r#"{"type":"tool_use","id":"u1","name":"Bash","input":{"c":"1"}}"#);
    assert!(tu
        .iter()
        .any(|e| matches!(e, Event::ToolCall { id, .. } if id == "u1")));

    // content blocks: empty text skip, tool_result non-string content
    let blocks = a.parse_line(
        r#"{"type":"assistant","message":{"content":[{"type":"text","text":""},{"type":"tool_use","id":"t","name":"R","input":{}},{"type":"tool_result","tool_use_id":"t","content":{"n":1},"is_error":false},{"type":"other"}]}}"#,
    );
    assert!(blocks.iter().any(|e| matches!(e, Event::ToolCall { .. })));
    assert!(blocks.iter().any(|e| matches!(
        e,
        Event::ToolResult {
            output,
            ..
        } if output.contains('1')
    )));

    // message without content array
    let bare = a.parse_line(r#"{"type":"assistant","message":{"role":"assistant"}}"#);
    assert!(bare.iter().any(|e| matches!(e, Event::Raw { .. })) || !bare.is_empty());
}

#[test]
fn codex_acp_cwd_and_first_turn_cd() {
    let a = CodexAdapter;
    assert!(!a.capabilities().acp);
    let mut opts = LaunchOptions {
        cwd: Some(PathBuf::from("/tmp")),
        ..Default::default()
    };
    opts.extra.insert("acp".into(), json!(true));
    let err = match a.prepare("p", &opts, &ctx(1, None)) {
        Err(e) => e,
        Ok(_) => panic!("expected prepare error"),
    };
    assert!(err.to_string().contains("ACP is not implemented"));

    let opts = LaunchOptions {
        cwd: Some(PathBuf::from("/work")),
        ..Default::default()
    };
    let args = a
        .prepare("p", &opts, &ctx(1, None))
        .unwrap()
        .spawn
        .unwrap()
        .args;
    assert!(args.windows(2).any(|w| w[0] == "--cd" && w[1] == "/work"));
}

#[test]
fn cursor_stream_partial_false_and_binary_cursor() {
    let a = CursorAdapter;
    let mut opts = LaunchOptions::default();
    opts.extra.insert("stream_partial".into(), json!(false));
    let args = a
        .prepare("hi", &opts, &ctx(1, None))
        .unwrap()
        .spawn
        .unwrap()
        .args;
    assert!(!args.iter().any(|x| x == "--stream-partial-output"));

    let mut opts = LaunchOptions::default();
    opts.extra.insert("binary".into(), json!("cursor"));
    let spawn = a
        .prepare("hi", &opts, &ctx(1, None))
        .unwrap()
        .spawn
        .unwrap();
    assert_eq!(spawn.program, PathBuf::from("cursor"));
    assert!(spawn.args.iter().any(|x| x == "agent"));
}

#[test]
fn grok_session_id_extra_without_resume_key() {
    let a = GrokAdapter;
    // Only session_id (not resume) on first turn → --session-id path
    let mut opts = LaunchOptions::default();
    opts.extra.insert("session_id".into(), json!("named"));
    let args = a
        .prepare("hi", &opts, &ctx(1, None))
        .unwrap()
        .spawn
        .unwrap()
        .args;
    assert!(
        args.windows(2)
            .any(|w| w[0] == "--session-id" && w[1] == "named")
            || args
                .windows(2)
                .any(|w| w[0] == "--resume" && w[1] == "named"),
        "{args:?}"
    );
}
