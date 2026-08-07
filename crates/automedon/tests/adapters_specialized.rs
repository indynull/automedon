//! Prepare/parse unit tests for Tier A/B specialized drivers (fixture frames only).

use automedon::adapter::{
    acp, Adapter, AiderAdapter, ClaudeAdapter, CodexAdapter, CopilotAdapter, CursorAdapter,
    GeminiAdapter, GrokAdapter, OpenCodeAdapter, PiAdapter, TurnContext,
};
use automedon::config::LaunchOptions;
use automedon::event::Event;
use serde_json::json;
use std::path::PathBuf;

fn ctx_turn(turn: u64, session: Option<&str>) -> TurnContext {
    TurnContext {
        turn,
        session_id: session.map(str::to_string),
        ..Default::default()
    }
}

#[test]
fn codex_prepare_exec_and_acp() {
    let a = CodexAdapter;
    let p = a
        .prepare("hi", &LaunchOptions::default(), &ctx_turn(1, None))
        .unwrap();
    let args = p.spawn.as_ref().unwrap().args.clone();
    assert!(args.iter().any(|x| x == "exec"));
    assert!(args.iter().any(|x| x == "--json"));

    let mut opts = LaunchOptions::default();
    opts.extra.insert("acp".into(), json!(true));
    let p = a.prepare("hi", &opts, &ctx_turn(1, None)).unwrap();
    let spawn = p.spawn.unwrap();
    assert!(spawn.retain_stdin);
    // ACP uses community adapter package via npx by default.
    assert!(
        spawn.args.iter().any(|x| x.contains("codex-acp"))
            || spawn.program.ends_with("npx")
            || spawn.program.as_os_str() == "npx"
    );
}

#[test]
fn codex_parse_tool_and_end() {
    let a = CodexAdapter;
    let ev = a.parse_line(r#"{"type":"tool_call","id":"1","name":"bash","input":{"c":"ls"}}"#);
    assert!(matches!(ev.first(), Some(Event::ToolCall { name, .. }) if name == "bash"));
    let ev = a.parse_line(r#"{"type":"result","sessionId":"s1","result":"done"}"#);
    assert!(ev
        .iter()
        .any(|e| matches!(e, Event::SessionInfo { id, .. } if id == "s1")));
    assert!(ev.iter().any(|e| matches!(e, Event::TurnComplete { .. })));
    // Real codex exec --json frames (incl. auth-fail)
    let th = a.parse_line(r#"{"type":"thread.started","thread_id":"th-abc"}"#);
    assert!(th
        .iter()
        .any(|e| matches!(e, Event::SessionInfo { id, .. } if id == "th-abc")));
    let ts = a.parse_line(r#"{"type":"turn.started"}"#);
    assert!(matches!(ts.first(), Some(Event::TurnStart { .. })));
    let item = a.parse_line(
        r#"{"type":"item.completed","item":{"id":"i1","type":"command_execution","command":"ls","exit_code":0,"aggregated_output":"ok"}}"#,
    );
    assert!(item
        .iter()
        .any(|e| matches!(e, Event::ToolResult { name, .. } if name.contains("ls") || name == "command_execution" || !name.is_empty())));
}

#[test]
fn gemini_prepare_stream_resume_acp() {
    let a = GeminiAdapter;
    let mut opts = LaunchOptions {
        yolo: true,
        model: Some("gemini-x".into()),
        ..Default::default()
    };
    let p = a.prepare("p", &opts, &ctx_turn(2, Some("sess-9"))).unwrap();
    let args = p.spawn.unwrap().args;
    assert!(args.iter().any(|x| x == "-p"));
    assert!(args.iter().any(|x| x == "stream-json" || x == "-o"));
    assert!(args.windows(2).any(|w| w[0] == "-r" && w[1] == "sess-9"));
    assert!(args.iter().any(|x| x == "-y"));

    opts.extra.insert("acp".into(), json!(true));
    let p = a.prepare("p", &opts, &ctx_turn(1, None)).unwrap();
    assert!(p.spawn.unwrap().args.iter().any(|x| x == "--acp"));
}

#[test]
fn gemini_parse_common() {
    let a = GeminiAdapter;
    let ev = a.parse_line(r#"{"type":"text","data":"hello"}"#);
    assert!(matches!(ev.first(), Some(Event::TextDelta { text }) if text == "hello"));
    let ev = a.parse_line("not-json");
    assert!(matches!(ev.first(), Some(Event::Raw { .. })));
}

#[test]
fn opencode_prepare_and_parse() {
    let a = OpenCodeAdapter;
    let p = a
        .prepare(
            "build it",
            &LaunchOptions {
                yolo: true,
                ..Default::default()
            },
            &ctx_turn(2, Some("oc1")),
        )
        .unwrap();
    let args = p.spawn.unwrap().args;
    assert_eq!(args[0], "run");
    assert!(args.iter().any(|x| x == "--auto"));
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--session" && w[1] == "oc1"));
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--format" && w[1] == "json"));

    let ev = a.parse_line(r#"{"type":"error","message":"boom"}"#);
    assert!(matches!(ev.first(), Some(Event::Error { message }) if message == "boom"));
    // Real OpenCode step_start carries sessionID
    let step =
        a.parse_line(r#"{"type":"step_start","sessionID":"ses_abc","part":{"type":"step-start"}}"#);
    assert!(step
        .iter()
        .any(|e| matches!(e, Event::SessionInfo { id, .. } if id == "ses_abc")));
}

#[test]
fn cursor_prepare_resume() {
    let a = CursorAdapter;
    let p = a
        .prepare("x", &LaunchOptions::default(), &ctx_turn(2, Some("c1")))
        .unwrap();
    let spawn = p.spawn.unwrap();
    let prog = spawn.program.to_string_lossy();
    // PATH order: agent → cursor-agent → cursor
    assert!(
        prog.ends_with("agent")
            || prog.ends_with("cursor-agent")
            || prog == "agent"
            || prog == "cursor-agent"
            || prog == "cursor",
        "unexpected cursor binary {prog}"
    );
    assert!(spawn
        .args
        .windows(2)
        .any(|w| w[0] == "--resume" && w[1] == "c1"));
    assert!(spawn.args.iter().any(|x| x == "stream-json"));

    let mut opts = LaunchOptions::default();
    opts.extra.insert("binary".into(), json!("cursor"));
    let p = a.prepare("x", &opts, &ctx_turn(1, None)).unwrap();
    let args = p.spawn.unwrap().args;
    assert_eq!(args[0], "agent");
}

#[test]
fn aider_prepare_message_text_parse() {
    let a = AiderAdapter;
    let mut opts = LaunchOptions {
        model: Some("xai/grok-4.5".into()),
        ..Default::default()
    };
    opts.extra.insert(
        "chat_history_file".into(),
        json!("/tmp/automedon-aider-test.history.md"),
    );
    let p = a.prepare("fix", &opts, &ctx_turn(1, None)).unwrap();
    let args = p.spawn.unwrap().args;
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--message" && w[1] == "fix"));
    assert!(args.iter().any(|x| x == "--yes-always"));
    assert!(args.iter().any(|x| x == "--no-git"));
    assert!(args.iter().any(|x| x == "--chat-history-file"));
    assert!(!args.iter().any(|x| x == "--restore-chat-history"));
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--model" && w[1] == "xai/grok-4.5"));
    assert!(p.synthetic.is_some()); // SessionInfo with history path
    let p2 = a
        .prepare(
            "again",
            &opts,
            &ctx_turn(2, Some("/tmp/automedon-aider-test.history.md")),
        )
        .unwrap();
    assert!(p2
        .spawn
        .unwrap()
        .args
        .iter()
        .any(|x| x == "--restore-chat-history"));
    let ev = a.parse_line("edited foo.py");
    assert!(matches!(ev.first(), Some(Event::TextDelta { text }) if text.contains("edited")));
    assert!(matches!(
        a.parse_line("Aider v0.86.2").first(),
        Some(Event::Raw { .. })
    ));
}

#[test]
fn pi_prepare_xai_provider_model() {
    let a = PiAdapter;
    let mut opts = LaunchOptions {
        model: Some("grok-4.5".into()),
        ..Default::default()
    };
    opts.extra.insert("provider".into(), json!("xai"));
    opts.extra.insert("extension".into(), json!("/tmp/ext.ts"));
    let p = a.prepare("hi", &opts, &ctx_turn(1, None)).unwrap();
    let args = p.spawn.unwrap().args;
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--provider" && w[1] == "xai"));
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--model" && w[1] == "grok-4.5"));
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--extension" && w[1] == "/tmp/ext.ts"));
}

#[test]
fn pi_tool_lifecycle_maps_to_hooks_and_tools() {
    let a = PiAdapter;
    // Live xAI/json path: toolcall_end inside message_update
    let line = r#"{"type":"message_update","assistantMessageEvent":{"type":"toolcall_end","toolCall":{"type":"toolCall","id":"t1","name":"bash","arguments":{"command":"ls"}}}}"#;
    let pre = a.parse_line(line);
    assert!(pre.iter().any(|e| matches!(
        e,
        Event::HookStarted { name, .. } if name == "PreToolUse"
    )));
    assert!(pre
        .iter()
        .any(|e| matches!(e, Event::ToolCall { name, .. } if name == "bash")));
    // Older top-level shapes
    let pre2 = a.parse_line(
        r#"{"type":"tool_call","toolCallId":"t1","toolName":"bash","args":{"command":"ls"}}"#,
    );
    assert!(pre2
        .iter()
        .any(|e| matches!(e, Event::ToolCall { name, .. } if name == "bash")));
    let post = a.parse_line(
        r#"{"type":"tool_result","toolCallId":"t1","toolName":"bash","result":"ok","isError":false}"#,
    );
    assert!(post
        .iter()
        .any(|e| matches!(e, Event::HookFinished { name, ok: true, .. } if name == "PostToolUse")));
}

#[test]
fn copilot_prepare_acp_and_prompt() {
    let a = CopilotAdapter;
    let p = a
        .prepare(
            "hi",
            &LaunchOptions {
                yolo: true,
                ..Default::default()
            },
            &ctx_turn(1, None),
        )
        .unwrap();
    let args = p.spawn.unwrap().args;
    assert!(args.windows(2).any(|w| w[0] == "-p" && w[1] == "hi"));
    assert!(args.iter().any(|x| x == "--allow-all"));
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--output-format" && w[1] == "json"));

    let mut opts = LaunchOptions::default();
    opts.extra.insert("acp".into(), json!(true));
    let p = a.prepare("hi", &opts, &ctx_turn(1, None)).unwrap();
    assert!(p.spawn.unwrap().retain_stdin);

    let p = a
        .prepare(
            "again",
            &LaunchOptions::default(),
            &ctx_turn(2, Some("sess-cp")),
        )
        .unwrap();
    let args = p.spawn.unwrap().args;
    assert!(args.iter().any(|a| a == "--resume=sess-cp"));
}

#[test]
fn copilot_parse_resume_footer() {
    let a = CopilotAdapter;
    let ev = a.parse_line("Resume     copilot --resume=a15c9384-9de2-4eb1-88d7-fa86d83b4860");
    assert!(ev
        .iter()
        .any(|e| matches!(e, Event::SessionInfo { id, .. } if id == "a15c9384-9de2-4eb1-88d7-fa86d83b4860")));
    assert!(a.capabilities().multi_turn);
    assert!(a.capabilities().launch);
}

#[test]
fn grok_acp_prepare() {
    let a = GrokAdapter;
    let mut opts = LaunchOptions::default();
    opts.extra.insert("acp".into(), json!(true));
    let p = a.prepare("hello", &opts, &ctx_turn(1, None)).unwrap();
    let spawn = p.spawn.unwrap();
    assert_eq!(spawn.args, vec!["agent".to_string(), "stdio".to_string()]);
    assert!(spawn.retain_stdin);
    assert_eq!(
        spawn.env.get("AUTOMEDON_ACP_PROMPT").map(String::as_str),
        Some("hello")
    );
}

#[test]
fn grok_parse_live_tool_name_and_result() {
    let a = GrokAdapter;
    // Live Grok streaming-json: toolName + rawInput (not name/input).
    let start = a.parse_line(
        r#"{"type":"tool_call","toolCallId":"c1","title":"run_terminal_command","toolName":"run_terminal_command","rawInput":{"command":"echo hi"},"status":"pending"}"#,
    );
    assert!(
        matches!(start.first(), Some(Event::ToolCall { name, .. }) if name == "run_terminal_command")
    );
    assert!(matches!(
        start.first(),
        Some(Event::ToolCall { input, .. }) if input.get("command").and_then(|v| v.as_str()) == Some("echo hi")
    ));
    let mid = a.parse_line(
        r#"{"type":"tool_call_update","toolCallId":"c1","status":"in_progress","content":[]}"#,
    );
    assert!(mid.is_empty());
    let done = a.parse_line(
        r#"{"type":"tool_call_update","toolCallId":"c1","status":"completed","content":[{"type":"content","content":{"type":"text","text":"hi\n"}}]}"#,
    );
    assert!(matches!(
        done.first(),
        Some(Event::ToolResult { output, is_error: false, .. }) if output.contains("hi")
    ));
}

#[test]
fn claude_prepare_resume_tools() {
    let a = ClaudeAdapter;
    let mut opts = LaunchOptions {
        yolo: true,
        ..Default::default()
    };
    opts.extra
        .insert("allowed_tools".into(), json!("Bash,Read"));
    opts.extra.insert("max_turns".into(), json!(3));
    let p = a.prepare("q", &opts, &ctx_turn(2, Some("cl1"))).unwrap();
    let args = p.spawn.unwrap().args;
    assert!(args.windows(2).any(|w| w[0] == "--resume" && w[1] == "cl1"));
    assert!(args.iter().any(|x| x == "--dangerously-skip-permissions"));
    assert!(args.iter().any(|x| x == "--include-hook-events"));
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--allowedTools" && w[1] == "Bash,Read"));
    // turn ≥ 2 without session → --continue
    let p2 = a
        .prepare("q", &LaunchOptions::default(), &ctx_turn(2, None))
        .unwrap();
    assert!(p2.spawn.unwrap().args.iter().any(|x| x == "--continue"));
}

#[test]
fn claude_parse_system_init_session_and_auth_fail() {
    let a = ClaudeAdapter;
    let init = a.parse_line(
        r#"{"type":"system","subtype":"init","session_id":"cabe3680-1b1b-454a-9e79-b7ec64dae50f","apiKeySource":"none"}"#,
    );
    assert!(init.iter().any(
        |e| matches!(e, Event::SessionInfo { id, .. } if id == "cabe3680-1b1b-454a-9e79-b7ec64dae50f")
    ));
    let res = a.parse_line(
        r#"{"type":"result","session_id":"cabe3680-1b1b-454a-9e79-b7ec64dae50f","is_error":true,"result":"Not logged in · Please run /login","subtype":"success"}"#,
    );
    assert!(res.iter().any(|e| matches!(e, Event::Error { .. })));
    assert!(res.iter().any(|e| matches!(e, Event::SessionInfo { .. })));
    assert!(res.iter().any(|e| matches!(e, Event::TurnComplete { .. })));
}

#[test]
fn claude_prepare_extras_and_rich_parse() {
    let a = ClaudeAdapter;
    let mut opts = LaunchOptions {
        model: Some("opus".into()),
        ..Default::default()
    };
    opts.extra.insert("permission_mode".into(), json!("plan"));
    opts.extra.insert("settings".into(), json!("/tmp/s.json"));
    opts.extra.insert("session_id".into(), json!("named-sess"));
    opts.extra.insert("resume".into(), json!("from-extra"));
    let p = a.prepare("q", &opts, &ctx_turn(1, None)).unwrap();
    let args = p.spawn.unwrap().args;
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--permission-mode" && w[1] == "plan"));
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--settings" && w[1] == "/tmp/s.json"));
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--session-id" && w[1] == "named-sess"));
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--resume" && w[1] == "from-extra"));
    assert!(args.windows(2).any(|w| w[0] == "--model" && w[1] == "opus"));

    // system without session_id
    assert!(matches!(
        a.parse_line(r#"{"type":"system","subtype":"other"}"#)
            .first(),
        Some(Event::Raw { .. })
    ));
    // assistant with session + auth text
    let auth = a.parse_line(
        r#"{"type":"assistant","session_id":"s1","message":{"content":[{"type":"text","text":"Not logged in · Please run /login"}]}}"#,
    );
    assert!(auth.iter().any(|e| matches!(e, Event::SessionInfo { .. })));
    assert!(auth
        .iter()
        .any(|e| matches!(e, Event::Error { .. }) || matches!(e, Event::TextDelta { .. })));
    // tool_result top-level + user tool_result block
    let tr = a.parse_line(
        r#"{"type":"tool_result","tool_use_id":"t1","name":"Bash","content":"out","is_error":false}"#,
    );
    assert!(tr.iter().any(|e| matches!(e, Event::ToolResult { .. })));
    assert!(tr.iter().any(|e| matches!(e, Event::HookFinished { .. })));
    let user = a.parse_line(
        r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t2","content":"x","is_error":true}]}}"#,
    );
    assert!(user
        .iter()
        .any(|e| matches!(e, Event::ToolResult { is_error: true, .. })));
    assert!(user.iter().any(
        |e| matches!(e, Event::HookFinished { name, ok: false, .. } if name == "PostToolUse")
    ));
    // Live stream-json: tools live in assistant content blocks.
    let asst_tool = a.parse_line(
        r#"{"type":"assistant","session_id":"s2","message":{"content":[{"type":"tool_use","id":"tu1","name":"Bash","input":{"command":"echo hi"}}]}}"#,
    );
    assert!(asst_tool
        .iter()
        .any(|e| matches!(e, Event::HookStarted { name, .. } if name == "PreToolUse")));
    assert!(asst_tool
        .iter()
        .any(|e| matches!(e, Event::ToolCall { name, .. } if name == "Bash")));
    // hooks via shared path
    let _ = a.parse_line(r#"{"type":"hook_started","id":"h","name":"PreToolUse"}"#);
    let _ = a.parse_line(r#"{"type":"hook_finished","id":"h","name":"PostToolUse","ok":true}"#);
    let ok_res = a.parse_line(
        r#"{"type":"result","result":"done","num_turns":1,"is_error":false,"session_id":"s9"}"#,
    );
    assert!(ok_res.iter().any(|e| matches!(e, Event::TextDelta { .. })));
    assert!(ok_res
        .iter()
        .any(|e| matches!(e, Event::TurnComplete { .. })));
    assert!(!ok_res.iter().any(|e| matches!(e, Event::Done { .. })));
}

#[test]
fn codex_opencode_cursor_gemini_frame_matrix() {
    let codex = CodexAdapter;
    let _ = codex.parse_line(r#"{"type":"turn.completed","thread_id":"th","turn":2}"#);
    let _ = codex.parse_line(
        r#"{"type":"item.started","item":{"id":"i","type":"command_execution","command":"pwd"}}"#,
    );
    let _ = codex.parse_line(
        r#"{"type":"item.completed","item":{"id":"i","type":"error","message":"nope"}}"#,
    );
    let _ = codex.parse_line(
        r#"{"type":"item.completed","item":{"id":"i","type":"agent_message","text":"hi"}}"#,
    );
    let _ =
        codex.parse_line(r#"{"type":"item.completed","item":{"id":"i","type":"unknown_kind"}}"#);
    let mut opts = LaunchOptions::default();
    opts.extra
        .insert("sandbox".into(), json!("workspace-write"));
    opts.extra.insert("model".into(), json!("o3"));
    let p = codex
        .prepare("x", &opts, &ctx_turn(1, None))
        .unwrap()
        .spawn
        .unwrap()
        .args;
    assert!(p.windows(2).any(|w| w[0] == "--sandbox"));
    assert!(p.windows(2).any(|w| w[0] == "--model" && w[1] == "o3"));

    let oc = OpenCodeAdapter;
    let _ = oc.parse_line(r#"{"type":"step_finish","reason":"stop"}"#);
    let _ = oc.parse_line(r#"prefix noise {"type":"text","text":"hello"}"#);
    assert!(oc
        .parse_line(r#"{"type":"text","text":"hello"}"#)
        .iter()
        .any(|e| matches!(e, Event::TextDelta { text } if text == "hello")));
    let _ = oc.parse_line(r#"{"type":"step_start","part":{"text":"nested"}}"#);
    let mut oopts = LaunchOptions::default();
    oopts.extra.insert("session".into(), json!("pre"));
    let args = oc
        .prepare("p", &oopts, &ctx_turn(1, None))
        .unwrap()
        .spawn
        .unwrap()
        .args;
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--session" && w[1] == "pre"));
    let cont = oc
        .prepare("p", &LaunchOptions::default(), &ctx_turn(2, None))
        .unwrap()
        .spawn
        .unwrap()
        .args;
    assert!(cont.iter().any(|x| x == "--continue"));

    let cur = CursorAdapter;
    assert!(matches!(
        cur.parse_line("Error: Authentication required. Please run 'agent login'")
            .first(),
        Some(Event::Error { .. })
    ));
    let mut copts = LaunchOptions::default();
    copts.extra.insert("resume".into(), json!("r1"));
    let args = cur
        .prepare("x", &copts, &ctx_turn(1, None))
        .unwrap()
        .spawn
        .unwrap()
        .args;
    assert!(args.windows(2).any(|w| w[0] == "--resume" && w[1] == "r1"));
    let cont = cur
        .prepare("x", &LaunchOptions::default(), &ctx_turn(2, None))
        .unwrap()
        .spawn
        .unwrap()
        .args;
    assert!(cont.iter().any(|x| x == "--continue"));
    copts.extra.insert("binary".into(), json!("agent"));
    let _ = cur.prepare("x", &copts, &ctx_turn(1, None)).unwrap();

    let gem = GeminiAdapter;
    assert!(matches!(
        gem.parse_line("Error authenticating: IneligibleTierError: free tier ended")
            .first(),
        Some(Event::Error { .. })
    ));
    let mut gopts = LaunchOptions {
        model: Some("g1".into()),
        ..Default::default()
    };
    gopts.extra.insert("approval_mode".into(), json!("default"));
    gopts.extra.insert("worktree".into(), json!(true));
    gopts.extra.insert("allowed_tools".into(), json!("read"));
    gopts.extra.insert("resume".into(), json!("latest"));
    let args = gem
        .prepare("p", &gopts, &ctx_turn(1, None))
        .unwrap()
        .spawn
        .unwrap()
        .args;
    assert!(args.iter().any(|x| x == "-w"));
    assert!(args.windows(2).any(|w| w[0] == "--allowed-tools"));
    let t2 = gem
        .prepare("p", &LaunchOptions::default(), &ctx_turn(2, None))
        .unwrap()
        .spawn
        .unwrap()
        .args;
    assert!(t2.windows(2).any(|w| w[0] == "-r"));
    gopts.extra.insert("binary".into(), json!("gemini"));
    let _ = gem.prepare("p", &gopts, &ctx_turn(1, None)).unwrap();
    let _ = gem.parse_line(r#"noise {"type":"text","data":"g"}"#);
    // explicit bin override
    let with_bin = LaunchOptions {
        bin: Some(PathBuf::from("gemini")),
        ..Default::default()
    };
    let _ = gem.prepare("p", &with_bin, &ctx_turn(1, None)).unwrap();
    // turn 2 with session id
    let _ = gem
        .prepare("p", &LaunchOptions::default(), &ctx_turn(2, Some("gsess")))
        .unwrap();
    // approval yolo via extra
    let mut gy = LaunchOptions::default();
    gy.extra.insert("approval_mode".into(), json!("yolo"));
    let _ = gem.prepare("p", &gy, &ctx_turn(1, None)).unwrap();

    // cursor with explicit bin
    let cb = LaunchOptions {
        bin: Some(PathBuf::from("agent")),
        ..Default::default()
    };
    let _ = cur.prepare("x", &cb, &ctx_turn(1, None)).unwrap();
    assert!(matches!(
        cur.parse_line("please agent login now").first(),
        Some(Event::Error { .. })
    ));

    // opencode nested text extract paths
    let _ = oc.parse_line(r#"{"type":"x","part":{"text":"from-part"}}"#);
    let _ = oc.parse_line(r#"{"type":"x","message":{"content":[{"text":"from-msg"}]}}"#);
    let _ = oc.parse_line("not-json-at-all");
}

#[test]
fn pi_prepare_session_id() {
    let a = PiAdapter;
    let p = a
        .prepare(
            "again",
            &LaunchOptions::default(),
            &ctx_turn(2, Some("pi-sess")),
        )
        .unwrap();
    let args = p.spawn.unwrap().args;
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--session-id" && w[1] == "pi-sess"));
}

#[test]
fn acp_helpers_and_product_names() {
    let line = acp::request(1, "initialize", acp::initialize_params("automedon"));
    assert!(line.contains("initialize"));
    let n = acp::notification(
        "session/update",
        json!({"update": {"sessionUpdate": "agent_message_chunk", "content": {"text": "z"}}}),
    );
    let ev = acp::parse_line(&n);
    assert!(matches!(ev.first(), Some(Event::TextDelta { text }) if text == "z"));
    assert!(automedon::product_names().contains(&"grok"));
    assert!(!automedon::product_names().contains(&"mock"));
}

#[test]
fn codex_resume_model_cwd_paths() {
    let a = CodexAdapter;
    let opts = LaunchOptions {
        yolo: true,
        model: Some("gpt-x".into()),
        cwd: Some(PathBuf::from("/tmp")),
        ..Default::default()
    };
    let p = a.prepare("go", &opts, &ctx_turn(2, Some("sess"))).unwrap();
    let args = p.spawn.unwrap().args;
    assert!(args
        .iter()
        .any(|x| x == "--dangerously-bypass-approvals-and-sandbox"));
    assert!(args.iter().any(|x| x == "resume"));
    assert!(args.iter().any(|x| x == "sess"));
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--model" && w[1] == "gpt-x"));
    // `codex exec resume` has no --cd; cwd applies on first exec turn only.
    assert!(args.iter().any(|x| x == "--json"));
    assert!(matches!(a.parse_line(""), x if x.is_empty()));
    assert!(matches!(a.parse_line("plain"), x if matches!(x.first(), Some(Event::Raw { .. }))));
}

#[test]
fn gemini_approval_worktree_tools() {
    let a = GeminiAdapter;
    let mut opts = LaunchOptions::default();
    opts.extra.insert("approval_mode".into(), json!("plan"));
    opts.extra.insert("worktree".into(), json!(true));
    opts.extra.insert("allowed_tools".into(), json!("read"));
    opts.extra.insert("resume".into(), json!("latest"));
    let p = a.prepare("q", &opts, &ctx_turn(1, None)).unwrap();
    let args = p.spawn.unwrap().args;
    assert!(args.iter().any(|x| x == "--approval-mode"));
    assert!(args.iter().any(|x| x == "-w"));
    assert!(args.iter().any(|x| x == "--allowed-tools"));
    assert!(args.windows(2).any(|w| w[0] == "-r" && w[1] == "latest"));
}

#[test]
fn opencode_acp_and_model() {
    let a = OpenCodeAdapter;
    let mut opts = LaunchOptions {
        model: Some("m1".into()),
        ..Default::default()
    };
    opts.extra.insert("acp".into(), json!(true));
    let p = a.prepare("x", &opts, &ctx_turn(1, None)).unwrap();
    let spawn = p.spawn.unwrap();
    assert!(spawn.retain_stdin);
    assert_eq!(spawn.args, vec!["acp".to_string()]);
    let opts = LaunchOptions {
        model: Some("m1".into()),
        ..Default::default()
    };
    let p = a.prepare("x", &opts, &ctx_turn(1, None)).unwrap();
    assert!(p
        .spawn
        .unwrap()
        .args
        .windows(2)
        .any(|w| w[0] == "--model" && w[1] == "m1"));
    assert!(matches!(a.parse_line(""), x if x.is_empty()));
}

#[test]
fn cursor_yolo_model_and_parse() {
    let a = CursorAdapter;
    let opts = LaunchOptions {
        yolo: true,
        model: Some("composer".into()),
        ..Default::default()
    };
    let p = a.prepare("x", &opts, &ctx_turn(1, None)).unwrap();
    let args = p.spawn.unwrap().args;
    assert!(
        args.iter().any(|x| x == "--yolo" || x == "--force"),
        "expected yolo/force flag, got {args:?}"
    );
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--model" && w[1] == "composer"));
    assert!(matches!(
        a.parse_line(r#"{"type":"text","data":"c"}"#).first(),
        Some(Event::TextDelta { text }) if text == "c"
    ));
    assert!(matches!(a.parse_line(""), x if x.is_empty()));
    assert!(matches!(a.parse_line("x").first(), Some(Event::Raw { .. })));
}

#[test]
fn copilot_model_and_parse_text() {
    let a = CopilotAdapter;
    let opts = LaunchOptions {
        model: Some("gpt".into()),
        ..Default::default()
    };
    let p = a.prepare("hi", &opts, &ctx_turn(1, None)).unwrap();
    assert!(p
        .spawn
        .unwrap()
        .args
        .windows(2)
        .any(|w| w[0] == "--model" && w[1] == "gpt"));
    // Live JSONL shapes (captured from copilot --output-format json).
    assert!(matches!(
        a.parse_line(r#"{"type":"assistant.message_delta","data":{"deltaContent":"z"}}"#)
            .first(),
        Some(Event::TextDelta { text }) if text == "z"
    ));
    assert!(matches!(
        a.parse_line("plain line").first(),
        Some(Event::TextDelta { text }) if text.contains("plain")
    ));
    let res = a.parse_line(
        r#"{"type":"result","sessionId":"a81b42ef-a1ea-4b38-93de-8f8bf1287571","exitCode":0}"#,
    );
    assert!(res.iter().any(
        |e| matches!(e, Event::SessionInfo { id, .. } if id == "a81b42ef-a1ea-4b38-93de-8f8bf1287571")
    ));
    assert!(matches!(a.parse_line(""), x if x.is_empty()));
}

#[test]
fn capabilities_product_headless() {
    let c = automedon::Capabilities::product_headless();
    assert!(c.launch && c.multi_turn && c.stream_tools && c.streaming_json);
}

#[test]
fn aider_empty_line() {
    assert!(AiderAdapter.parse_line("").is_empty());
}

#[test]
fn aider_extra_model_base_keys_and_history_from_session() {
    let a = AiderAdapter;
    let mut opts = LaunchOptions::default();
    // model from extra (not LaunchOptions.model)
    opts.extra.insert("model".into(), json!("xai/grok-4.5"));
    opts.extra
        .insert("openai_api_base".into(), json!("https://api.x.ai/v1"));
    opts.extra.insert("xai_api_key".into(), json!("xai-test"));
    opts.extra.insert("openai_api_key".into(), json!("sk-test"));
    opts.extra.insert("no_git".into(), json!(false));
    let p = a.prepare("m", &opts, &ctx_turn(1, None)).unwrap();
    let spawn = p.spawn.unwrap();
    assert!(spawn
        .args
        .windows(2)
        .any(|w| w[0] == "--model" && w[1] == "xai/grok-4.5"));
    assert!(spawn
        .args
        .windows(2)
        .any(|w| w[0] == "--openai-api-base" && w[1].contains("x.ai")));
    assert!(!spawn.args.iter().any(|x| x == "--no-git"));
    assert_eq!(
        spawn.env.get("XAI_API_KEY").map(String::as_str),
        Some("xai-test")
    );
    assert_eq!(
        spawn.env.get("OPENAI_API_KEY").map(String::as_str),
        Some("sk-test")
    );

    // session_id path with .md reuses history file
    let p2 = a
        .prepare(
            "t2",
            &LaunchOptions::default(),
            &ctx_turn(2, Some("/var/tmp/prior.history.md")),
        )
        .unwrap();
    let args = p2.spawn.unwrap().args;
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--chat-history-file" && w[1] == "/var/tmp/prior.history.md"));
    assert!(args.iter().any(|x| x == "--restore-chat-history"));

    // windows-style path
    let p3 = a
        .prepare(
            "t3",
            &LaunchOptions::default(),
            &ctx_turn(2, Some(r"C:\tmp\h.md")),
        )
        .unwrap();
    assert!(p3
        .spawn
        .unwrap()
        .args
        .iter()
        .any(|x| x.contains("h.md") || x.contains(r"C:\tmp")));

    // noise lines → Raw
    for noise in [
        "Model: x",
        "Git repo: y",
        "Repo-map: z",
        "Tokens: 1",
        "Cost: 0",
        "Warning: w",
    ] {
        assert!(
            matches!(a.parse_line(noise).first(), Some(Event::Raw { .. })),
            "{noise}"
        );
    }
}

#[test]
fn pi_extensions_session_hooks_and_toolcall_partial() {
    let a = PiAdapter;
    let mut opts = LaunchOptions {
        model: None,
        ..Default::default()
    };
    opts.extra.insert("model".into(), json!("grok-4.5"));
    opts.extra
        .insert("extensions".into(), json!(["/ext/a.js", "/ext/b.js"]));
    opts.extra.insert("extension".into(), json!("/ext/c.js"));
    opts.extra.insert("multi_turn".into(), json!(false));
    let p = a.prepare("p", &opts, &ctx_turn(1, None)).unwrap();
    let args = p.spawn.unwrap().args;
    assert!(args.iter().any(|x| x == "--no-session"));
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--model" && w[1] == "grok-4.5"));
    assert_eq!(args.iter().filter(|x| *x == "--extension").count(), 3);

    // session lifecycle hooks
    let start = a.parse_line(r#"{"type":"session_start","reason":"startup"}"#);
    assert!(start
        .iter()
        .any(|e| matches!(e, Event::HookStarted { name, .. } if name == "SessionStart")));
    let end = a.parse_line(r#"{"type":"session_end"}"#);
    assert!(end
        .iter()
        .any(|e| matches!(e, Event::HookFinished { name, .. } if name == "SessionEnd")));
    let shut = a.parse_line(r#"{"type":"session_shutdown"}"#);
    assert!(shut
        .iter()
        .any(|e| matches!(e, Event::HookFinished { name, .. } if name == "SessionEnd")));

    // toolcall_start without top-level toolCall — pull from partial.message content
    let nested = r#"{
      "type":"message_update",
      "assistantMessageEvent":{
        "type":"toolcall_start",
        "partial":{
          "content":[
            {"type":"text","text":"x"},
            {"type":"toolCall","id":"tc1","name":"bash","arguments":{"cmd":"true"}}
          ]
        }
      }
    }"#;
    let ev = a.parse_line(nested);
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::ToolCall { name, .. } if name == "bash")),
        "{ev:?}"
    );

    // continue on turn>1 without session id
    let p2 = a
        .prepare("again", &LaunchOptions::default(), &ctx_turn(2, None))
        .unwrap();
    assert!(p2.spawn.unwrap().args.iter().any(|x| x == "--continue"));
}

#[test]
fn copilot_continue_and_resume_space_form() {
    let a = CopilotAdapter;
    // turn>1 without session → --continue
    let p = a
        .prepare("again", &LaunchOptions::default(), &ctx_turn(2, None))
        .unwrap();
    assert!(p.spawn.unwrap().args.iter().any(|x| x == "--continue"));

    // --resume <id> space form (not equals)
    let ev = a.parse_line("Resume copilot --resume deadbeef-1234");
    assert!(ev
        .iter()
        .any(|e| matches!(e, Event::SessionInfo { id, .. } if id == "deadbeef-1234")));
    // empty id / bare resume → no SessionInfo
    assert!(!a
        .parse_line("Resume something --resume=")
        .iter()
        .any(|e| matches!(e, Event::SessionInfo { .. })));
    assert!(!a
        .parse_line("resume flag only --resume")
        .iter()
        .any(|e| matches!(e, Event::SessionInfo { .. })));
    // no resume → None path covered by plain text
    assert!(matches!(
        a.parse_line("hello world").first(),
        Some(Event::TextDelta { .. })
    ));
}

#[test]
fn opencode_continue_without_session() {
    let a = OpenCodeAdapter;
    let p = a
        .prepare("x", &LaunchOptions::default(), &ctx_turn(2, None))
        .unwrap();
    let args = p.spawn.unwrap().args;
    // either --continue or session flag when no id
    assert!(
        args.iter().any(|x| x == "--continue" || x == "--session")
            || args.iter().any(|x| x.contains("continue")),
        "{args:?}"
    );
    // model from extra
    let mut opts = LaunchOptions::default();
    opts.extra.insert("model".into(), json!("xai/grok-4.5"));
    let p = a.prepare("m", &opts, &ctx_turn(1, None)).unwrap();
    assert!(p
        .spawn
        .unwrap()
        .args
        .windows(2)
        .any(|w| w[0] == "--model" && w[1] == "xai/grok-4.5"));
    assert!(matches!(a.parse_line("").as_slice(), []));
    assert!(matches!(
        a.parse_line("not-json-line").first(),
        Some(Event::Raw { .. })
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn session_capabilities_and_aider_synthetic_session_info() {
    use automedon::Session;
    use std::sync::Arc;
    use std::time::Duration;

    // Product adapter with no interactive plan → fail closed (require_cap false branch).
    let mut s = Session::builder("copilot")
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    assert!(s.capabilities().launch);
    assert!(!s.capabilities().permissions_interactive);
    let err = s.approve().await.unwrap_err();
    assert!(
        err.to_string().contains("capability not supported"),
        "{err}"
    );
    let err = s.approve_plan().await.unwrap_err();
    assert!(
        err.to_string().contains("capability not supported"),
        "{err}"
    );
    s.close().await.ok();

    // Aider prepare injects SessionInfo synthetic alongside spawn — session applies it.
    let mut opts = LaunchOptions {
        bin: Some(PathBuf::from("/bin/echo")),
        model: Some("xai/grok-4.5".into()),
        default_timeout: Some(Duration::from_secs(5)),
        ..Default::default()
    };
    opts.extra.insert(
        "chat_history_file".into(),
        json!("/tmp/automedon-cov-aider.history.md"),
    );
    let mut s = Session::from_adapter(Arc::new(AiderAdapter), opts);
    // prompt will spawn /bin/echo with many args (echo ignores); synthetic SessionInfo applied
    let _ = s.prompt("hello-from-cov").await;
    // session id may be history path if synthetic applied
    let sid = s.session_id().map(str::to_string);
    assert!(
        sid.as_deref()
            .is_some_and(|p| p.contains("automedon-cov-aider") || p.contains("history")),
        "expected history session id, got {sid:?}"
    );
    s.close().await.ok();
}

/// Offline fixtures derived from real CLI stream captures (2026-08-07 host inventory).
#[test]
fn real_stream_fixtures_claude_codex_opencode_copilot_pi() {
    // Claude Code stream-json (auth-fail still emits system init + result).
    let claude = ClaudeAdapter;
    let init = claude.parse_line(
        r#"{"type":"system","subtype":"init","session_id":"06b815c5-636a-4834-9da8-1ef62d2927cd","apiKeySource":"none"}"#,
    );
    assert!(init.iter().any(
        |e| matches!(e, Event::SessionInfo { id, .. } if id == "06b815c5-636a-4834-9da8-1ef62d2927cd")
    ));
    let res = claude.parse_line(
        r#"{"type":"result","session_id":"06b815c5-636a-4834-9da8-1ef62d2927cd","is_error":true,"result":"Not logged in · Please run /login","subtype":"success","num_turns":1}"#,
    );
    assert!(res.iter().any(|e| matches!(e, Event::Error { .. })));
    assert!(res.iter().any(|e| matches!(e, Event::TurnComplete { .. })));
    assert!(claude
        .prepare("q", &LaunchOptions::default(), &ctx_turn(1, None))
        .unwrap()
        .spawn
        .unwrap()
        .args
        .iter()
        .any(|a| a == "--include-hook-events"));

    // Codex exec --json
    let codex = CodexAdapter;
    let th = codex.parse_line(
        r#"{"type":"thread.started","thread_id":"019fd9f5-c0b7-77c2-b57f-99880d00fd83"}"#,
    );
    assert!(th.iter().any(
        |e| matches!(e, Event::SessionInfo { id, .. } if id == "019fd9f5-c0b7-77c2-b57f-99880d00fd83")
    ));
    let err = codex.parse_line(
        r#"{"type":"error","message":"Reconnecting... 2/5 (unexpected status 401 Unauthorized)"}"#,
    );
    assert!(matches!(err.first(), Some(Event::Error { .. })));
    let p = codex
        .prepare("hi", &LaunchOptions::default(), &ctx_turn(2, None))
        .unwrap();
    let args = p.spawn.unwrap().args;
    assert!(args.iter().any(|a| a == "resume"));
    assert!(args.iter().any(|a| a == "--last"));

    // OpenCode run --format json
    let oc = OpenCodeAdapter;
    let start = oc.parse_line(
        r#"{"type":"step_start","sessionID":"ses_02609fb1fffeo0q04BdpZ7eR2p","part":{"type":"step-start"}}"#,
    );
    assert!(start.iter().any(
        |e| matches!(e, Event::SessionInfo { id, .. } if id == "ses_02609fb1fffeo0q04BdpZ7eR2p")
    ));
    let text = oc.parse_line(
        r#"{"type":"text","sessionID":"ses_02609fb1fffeo0q04BdpZ7eR2p","part":{"type":"text","text":"Hi"}}"#,
    );
    assert!(text
        .iter()
        .any(|e| matches!(e, Event::TextDelta { text } if text == "Hi")));

    // Copilot --output-format json (real order: delta → message → turn_end → result).
    let cp = CopilotAdapter;
    let mut joined = String::new();
    let mut n_tc = 0usize;
    let mut sid = None;
    for line in [
        r#"{"type":"assistant.message_delta","data":{"deltaContent":"HI_ONLY"}}"#,
        r#"{"type":"assistant.message","data":{"content":"HI_ONLY","toolRequests":[{"id":"t1","name":"bash","arguments":{"c":"echo"}}]}}"#,
        r#"{"type":"assistant.turn_end","data":{"turnId":"0"}}"#,
        r#"{"type":"result","sessionId":"a81b42ef-a1ea-4b38-93de-8f8bf1287571","exitCode":0}"#,
    ] {
        for e in cp.parse_line(line) {
            match e {
                Event::TextDelta { text } => joined.push_str(&text),
                Event::TurnComplete { .. } => n_tc += 1,
                Event::SessionInfo { id, .. } => sid = Some(id),
                Event::ToolCall { name, .. } => assert_eq!(name, "bash"),
                _ => {}
            }
        }
    }
    assert_eq!(joined, "HI_ONLY");
    assert_eq!(n_tc, 1);
    assert_eq!(sid.as_deref(), Some("a81b42ef-a1ea-4b38-93de-8f8bf1287571"));

    // Pi --mode json
    let pi = PiAdapter;
    let sess = pi.parse_line(
        r#"{"type":"session","version":3,"id":"019fd9f6-543a-7d1b-869d-8a50f8a6f208"}"#,
    );
    assert!(sess.iter().any(
        |e| matches!(e, Event::SessionInfo { id, .. } if id == "019fd9f6-543a-7d1b-869d-8a50f8a6f208")
    ));
    let msg = pi.parse_line(
        r#"{"type":"message_update","assistantMessageEvent":{"type":"text_delta","delta":"hi"}}"#,
    );
    assert!(matches!(msg.first(), Some(Event::TextDelta { text }) if text == "hi"));
}

#[test]
fn grok_continue_when_no_session_id() {
    let a = GrokAdapter;
    let p = a
        .prepare("again", &LaunchOptions::default(), &ctx_turn(2, None))
        .unwrap();
    assert!(p.spawn.unwrap().args.iter().any(|x| x == "--continue"));
}
