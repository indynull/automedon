//! Push line coverage of shipped library paths toward complete.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use automedon::adapter::{
    known_names, pi_prepare_args, resolve, Adapter, AdapterKind, ClaudeAdapter, GenericAdapter,
    GrokAdapter, MockAdapter, PiAdapter, TurnContext,
};
use automedon::transport::{spawn_process, SpawnSpec};
use automedon::{run, Error, Event, Expect, LaunchOptions, Predicate, Session, Transcript};
use serde_json::json;

fn fixture_harness() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/fake_harness.sh")
}

// --- config / error / registry ---

#[test]
fn launch_options_builder_and_serde() {
    let opts = LaunchOptions::default()
        .cwd("/tmp")
        .bin("custom-bin")
        .model("m1")
        .yolo(true)
        .env("K", "V")
        .extra("max_turns", json!(3))
        .timeout(Duration::from_secs(9));
    assert!(opts.yolo);
    assert_eq!(opts.model.as_deref(), Some("m1"));
    assert_eq!(
        opts.default_timeout_or(Duration::from_secs(1)),
        Duration::from_secs(9)
    );
    let s = serde_json::to_string(&opts).unwrap();
    let back: LaunchOptions = serde_json::from_str(&s).unwrap();
    assert_eq!(back.model.as_deref(), Some("m1"));
    let bare = LaunchOptions::default();
    assert_eq!(
        bare.default_timeout_or(Duration::from_millis(5)),
        Duration::from_millis(5)
    );
    let none_json = r#"{"yolo":false,"env":{},"extra":{}}"#;
    let bare2: LaunchOptions = serde_json::from_str(none_json).unwrap();
    // serialize None default_timeout
    let s2 = serde_json::to_string(&bare2).unwrap();
    assert!(s2.contains("null") || s2.contains("default_timeout"));
}

#[test]
fn error_display_and_from() {
    let e = Error::UnknownAdapter("x".into());
    assert!(e.to_string().contains("unknown"));
    let _ = Error::HarnessNotFound("bin".into());
    let _ = Error::SessionFinished;
    let _ = Error::NoActiveTurn;
    let _ = Error::ExpectTimeout {
        timeout: Duration::from_secs(1),
        predicate: "p".into(),
    };
    let _ = Error::ExpectFailed("f".into());
    let _ = Error::ProcessFailed {
        code: Some(1),
        stderr: "e".into(),
    };
    let _ = Error::Script("s".into());
    let _ = Error::Other("o".into());
    let io: Error = std::io::Error::other("io").into();
    assert!(io.to_string().contains("I/O"));
    let j: Error = serde_json::from_str::<serde_json::Value>("not-json")
        .unwrap_err()
        .into();
    assert!(j.to_string().contains("JSON"));
    let a: Error = anyhow::anyhow!("aw").into();
    assert!(a.to_string().contains("aw"));
}

#[test]
fn registry_and_capabilities() {
    for name in known_names() {
        let a = resolve(name).unwrap();
        assert_eq!(a.name(), *name);
        let _ = a.capabilities();
    }
    assert!(resolve("nope").is_err());
    assert_eq!(AdapterKind::parse("grok").unwrap().as_str(), "grok");
    assert_eq!(AdapterKind::parse("grok-build").unwrap().as_str(), "grok");
    assert_eq!(AdapterKind::parse("pi-mono").unwrap().as_str(), "pi");
    assert_eq!(
        AdapterKind::parse("claude-code").unwrap().as_str(),
        "claude"
    );
    assert_eq!(AdapterKind::parse("test").unwrap().as_str(), "mock");
    assert_eq!(AdapterKind::parse("raw").unwrap().as_str(), "generic");
    assert_eq!(AdapterKind::parse("anthropic").unwrap().as_str(), "claude");
    assert!(AdapterKind::parse("zzz").is_err());
    for k in [
        AdapterKind::Claude,
        AdapterKind::Codex,
        AdapterKind::Gemini,
        AdapterKind::OpenCode,
        AdapterKind::Grok,
        AdapterKind::Cursor,
        AdapterKind::Aider,
        AdapterKind::Pi,
        AdapterKind::Copilot,
        AdapterKind::Mock,
        AdapterKind::Generic,
    ] {
        let _ = k.as_str();
        let _ = automedon::adapter::registry(k);
        assert_eq!(
            k.is_product(),
            !matches!(k, AdapterKind::Mock | AdapterKind::Generic)
        );
    }
    assert_eq!(AdapterKind::parse("codex").unwrap().as_str(), "codex");
    assert_eq!(AdapterKind::parse("gemini-cli").unwrap().as_str(), "gemini");
    assert_eq!(
        AdapterKind::parse("antigravity").unwrap().as_str(),
        "gemini"
    );
    assert_eq!(AdapterKind::parse("opencode").unwrap().as_str(), "opencode");
    assert_eq!(
        AdapterKind::parse("cursor-agent").unwrap().as_str(),
        "cursor"
    );
    assert_eq!(AdapterKind::parse("aider").unwrap().as_str(), "aider");
    assert_eq!(AdapterKind::parse("copilot").unwrap().as_str(), "copilot");
    assert!(!automedon::adapter::product_names().is_empty());
}

// --- expect matrix ---

#[test]
fn expect_predicates_matrix() {
    let mut t = Transcript::default();
    t.push(Event::TextDelta {
        text: "hello world".into(),
    });
    t.push(Event::ThinkingDelta {
        text: "ponder".into(),
    });
    t.push(Event::ToolCall {
        id: "1".into(),
        name: "list_dir".into(),
        input: json!({"path": "src/main.rs"}),
    });
    t.push(Event::ToolResult {
        id: "1".into(),
        name: "list_dir".into(),
        output: "ok".into(),
        is_error: false,
    });
    t.push(Event::PermissionRequest {
        id: "p".into(),
        tool: "bash".into(),
        detail: "ls".into(),
    });
    t.push(Event::PlanPresented {
        id: "pl".into(),
        summary: "do the thing".into(),
    });
    t.push(Event::PlanResolved {
        id: "pl".into(),
        approved: true,
    });
    t.push(Event::GoalStarted {
        id: "g".into(),
        title: "ship it".into(),
    });
    t.push(Event::GoalProgress {
        id: "g".into(),
        message: "mid".into(),
        percent: Some(50.0),
    });
    t.push(Event::GoalCompleted {
        id: "g".into(),
        success: true,
        message: Some("done".into()),
    });
    t.push(Event::SessionInfo {
        id: "sid".into(),
        label: Some("l".into()),
    });
    t.push(Event::TurnComplete {
        turn: 1,
        stop_reason: Some("end".into()),
    });
    t.push(Event::ProcessExit { code: Some(0) });
    t.push(Event::Done { code: Some(0) });
    t.push(Event::Raw {
        channel: "stdout".into(),
        line: "rawline".into(),
    });

    let ev_text = Event::TextDelta {
        text: "hello world".into(),
    };
    assert!(Expect::text("hello").matches(&ev_text, &t, 0));
    assert!(Expect::text_regex("h.*o").unwrap().matches(&ev_text, &t, 0));
    assert!(Expect::text_regex("(").is_err());
    assert!(Expect::thinking("pond").matches(
        &Event::ThinkingDelta {
            text: "ponder".into()
        },
        &t,
        0
    ));
    assert!(Expect::tool("list_dir").matches(
        &Event::ToolCall {
            id: "1".into(),
            name: "list_dir".into(),
            input: json!({"path": "src/main.rs"}),
        },
        &t,
        0
    ));
    assert!(Expect::tool_any().matches(
        &Event::ToolCall {
            id: "1".into(),
            name: "x".into(),
            input: json!(null),
        },
        &t,
        0
    ));
    assert!(Expect::tool_input("list_dir", "main.rs").matches(
        &Event::ToolCall {
            id: "1".into(),
            name: "list_dir".into(),
            input: json!({"path": "src/main.rs"}),
        },
        &t,
        0
    ));
    assert!(Expect::tool_result("list_dir").matches(
        &Event::ToolResult {
            id: "1".into(),
            name: "list_dir".into(),
            output: "ok".into(),
            is_error: false,
        },
        &t,
        0
    ));
    assert!(Expect::permission().matches(
        &Event::PermissionRequest {
            id: "p".into(),
            tool: "bash".into(),
            detail: "ls".into(),
        },
        &t,
        0
    ));
    assert!(Expect::plan().matches(
        &Event::PlanPresented {
            id: "pl".into(),
            summary: "do the thing".into(),
        },
        &t,
        0
    ));
    assert!(Expect::plan_summary("thing").matches(
        &Event::PlanPresented {
            id: "pl".into(),
            summary: "do the thing".into(),
        },
        &t,
        0
    ));
    assert!(Expect::plan_resolved(true).matches(
        &Event::PlanResolved {
            id: "pl".into(),
            approved: true,
        },
        &t,
        0
    ));
    assert!(Expect::goal().matches(
        &Event::GoalStarted {
            id: "g".into(),
            title: "ship it".into(),
        },
        &t,
        0
    ));
    assert!(Expect::goal_title("ship").matches(
        &Event::GoalStarted {
            id: "g".into(),
            title: "ship it".into(),
        },
        &t,
        0
    ));
    assert!(Expect::goal_progress().matches(
        &Event::GoalProgress {
            id: "g".into(),
            message: "mid".into(),
            percent: Some(50.0),
        },
        &t,
        0
    ));
    assert!(Expect::goal_completed(true).matches(
        &Event::GoalCompleted {
            id: "g".into(),
            success: true,
            message: None,
        },
        &t,
        0
    ));
    assert!(Expect::session_info().matches(
        &Event::SessionInfo {
            id: "sid".into(),
            label: None,
        },
        &t,
        0
    ));
    assert!(Expect::turn_complete().matches(
        &Event::TurnComplete {
            turn: 1,
            stop_reason: None,
        },
        &t,
        0
    ));
    assert!(Expect::process_exit().matches(&Event::ProcessExit { code: None }, &t, 0));
    assert!(Expect::done().matches(&Event::Done { code: None }, &t, 0));
    assert!(Expect::raw("stdout", "raw").matches(
        &Event::Raw {
            channel: "stdout".into(),
            line: "rawline".into(),
        },
        &t,
        0
    ));
    let any = Expect::new(Predicate::Any(vec![
        Predicate::Done,
        Predicate::TurnComplete,
    ]));
    assert!(any.matches(
        &Event::TurnComplete {
            turn: 1,
            stop_reason: None
        },
        &t,
        0
    ));
    let all = Expect::new(Predicate::All(vec![Predicate::Done]));
    assert!(all.matches(&Event::Done { code: Some(0) }, &t, 0));
    let _ = format!("{}", Expect::text("x"));
    let _ = format!("{}", Expect::tool("t"));
    let _ = format!("{}", Expect::tool_result("t"));
    let _ = format!("{}", Expect::plan_summary("p"));
    let _ = format!("{}", Expect::plan_resolved(false));
    let _ = format!("{}", Expect::goal_title("g"));
    let _ = format!("{}", Expect::goal_completed(false));
    let _ = format!("{}", Expect::raw("c", "n"));
    let _ = format!("{}", any);
    let _ = format!("{}", all);
    let _ = Expect::text("x").timeout(Duration::from_millis(1));
    // value_contains branches via tool_input on nested structures
    assert!(Expect::tool_input("list_dir", "nested").matches(
        &Event::ToolCall {
            id: "2".into(),
            name: "list_dir".into(),
            input: json!([{"k": "nested"}]),
        },
        &t,
        0
    ));
    assert!(Expect::tool_input("list_dir", "42").matches(
        &Event::ToolCall {
            id: "3".into(),
            name: "list_dir".into(),
            input: json!(42),
        },
        &t,
        0
    ));
}

#[test]
fn event_helpers_and_transcript_records() {
    let e = Event::TextDelta { text: "a".into() };
    assert_eq!(e.as_text_delta(), Some("a"));
    assert!(!e.is_session_terminal());
    assert!(!e.is_turn_boundary());
    assert!(Event::Done { code: None }.is_session_terminal());
    assert!(Event::TurnComplete {
        turn: 1,
        stop_reason: None
    }
    .is_turn_boundary());
    assert_eq!(
        Event::ToolCall {
            id: "i".into(),
            name: "n".into(),
            input: json!(null)
        }
        .tool_name(),
        Some("n")
    );
    assert_eq!(
        Event::ToolResult {
            id: "i".into(),
            name: "n".into(),
            output: "o".into(),
            is_error: true
        }
        .tool_name(),
        Some("n")
    );

    let mut t = Transcript::default();
    t.push(Event::SessionInfo {
        id: "s".into(),
        label: None,
    });
    t.push(Event::TurnStart { turn: 1 });
    t.push(Event::TextDelta { text: "hi".into() });
    t.push(Event::ThinkingDelta { text: "th".into() });
    t.push(Event::ToolCall {
        id: "c".into(),
        name: "t".into(),
        input: json!({}),
    });
    t.push(Event::ToolResult {
        id: "c".into(),
        name: "t".into(),
        output: "out".into(),
        is_error: false,
    });
    // unmatched tool result id
    t.push(Event::ToolResult {
        id: "missing".into(),
        name: "t".into(),
        output: "x".into(),
        is_error: false,
    });
    t.push(Event::PermissionRequest {
        id: "p".into(),
        tool: "x".into(),
        detail: "d".into(),
    });
    t.push(Event::PermissionResolved {
        id: "p".into(),
        allowed: false,
    });
    t.push(Event::PlanPresented {
        id: "pl".into(),
        summary: "s".into(),
    });
    t.push(Event::PlanResolved {
        id: "pl".into(),
        approved: false,
    });
    t.push(Event::GoalStarted {
        id: "g".into(),
        title: "t".into(),
    });
    t.push(Event::GoalCompleted {
        id: "g".into(),
        success: false,
        message: Some("m".into()),
    });
    t.push(Event::Usage {
        input_tokens: 1,
        output_tokens: 2,
        total_tokens: 3,
        cost_usd: Some(0.1),
    });
    t.push(Event::Error {
        message: "e".into(),
    });
    t.push(Event::PlanModeEnter {
        reason: Some("r".into()),
    });
    t.push(Event::PlanModeExit {
        reason: Some("r".into()),
    });
    t.push(Event::Spawned {
        pid: 1,
        harness: "h".into(),
    });
    assert_eq!(t.session_id(), Some("s"));
    assert_eq!(t.text(), "hi");
    assert_eq!(t.thinking(), "th");
    assert!(!t.tools().is_empty());
    assert!(!t.plans().is_empty());
    assert!(!t.goals().is_empty());
    assert!(!t.permissions().is_empty());
    assert!(!t.thinking_since(0).is_empty());
    t.begin_turn();
    assert_eq!(t.turn_text(), "");
    assert_eq!(t.turn_thinking(), "");
}

// --- adapters parse ---

#[test]
fn grok_parse_all_shapes() {
    let a = GrokAdapter;
    assert!(a.capabilities().streaming_json);
    assert!(a.parse_line("").is_empty());
    assert!(a.parse_line("   ").is_empty());
    assert!(matches!(&a.parse_line("not-json")[0], Event::Raw { .. }));
    assert!(matches!(
        &a.parse_line(r#"{"type":"text","data":""}"#)[..],
        []
    ));
    assert!(matches!(
        &a.parse_line(r#"{"type":"text","data":"hi"}"#)[0],
        Event::TextDelta { text } if text == "hi"
    ));
    assert!(matches!(
        &a.parse_line(r#"{"type":"thought","data":"t"}"#)[0],
        Event::ThinkingDelta { .. }
    ));
    assert!(matches!(
        &a.parse_line(r#"{"type":"thinking","data":""}"#)[..],
        []
    ));
    assert!(matches!(
        &a.parse_line(r#"{"type":"tool_call","id":"1","name":"n","input":{}}"#)[0],
        Event::ToolCall { .. }
    ));
    assert!(matches!(
        &a.parse_line(r#"{"type":"tool_use","toolCallId":"1","tool":"n","arguments":{"a":1}}"#)[0],
        Event::ToolCall { .. }
    ));
    assert!(matches!(
        &a.parse_line(r#"{"type":"tool_result","id":"1","name":"n","output":"o"}"#)[0],
        Event::ToolResult { .. }
    ));
    assert!(matches!(
        &a.parse_line(r#"{"type":"tool_result","toolCallId":"1","data":{"x":1},"isError":true}"#)
            [0],
        Event::ToolResult { is_error: true, .. }
    ));
    assert!(matches!(
        &a.parse_line(r#"{"type":"usage","usage":{"input_tokens":1,"output_tokens":2,"total_tokens":3},"total_cost_usd":0.01}"#)[0],
        Event::Usage { .. }
    ));
    let end = a.parse_line(
        r#"{"type":"end","stopReason":"end_turn","sessionId":"s1","num_turns":2,"plan":{"id":"p","summary":"sum"},"goal":{"id":"g","title":"gt"}}"#,
    );
    assert!(end.iter().any(|e| matches!(e, Event::SessionInfo { .. })));
    assert!(end.iter().any(|e| matches!(e, Event::PlanPresented { .. })));
    assert!(end.iter().any(|e| matches!(e, Event::GoalStarted { .. })));
    assert!(end.iter().any(|e| matches!(e, Event::TurnComplete { .. })));
    assert!(matches!(
        &a.parse_line(r#"{"type":"error","message":"boom"}"#)[0],
        Event::Error { .. }
    ));
    assert!(matches!(
        &a.parse_line(r#"{"type":"error","data":{"c":1}}"#)[0],
        Event::Error { .. }
    ));
    // end with session_id snake_case and plan/goal missing fields
    let end2 = a.parse_line(r#"{"type":"end","session_id":"sid2","plan":{},"goal":{}}"#);
    assert!(end2.iter().any(|e| matches!(e, Event::SessionInfo { .. })));
    assert!(matches!(
        &a.parse_line(r#"{"type":"available_commands"}"#)[0],
        Event::Raw { .. }
    ));
    // tool_result with is_error false via is_error key
    assert!(matches!(
        &a.parse_line(
            r#"{"type":"tool_result","id":"1","name":"n","output":"x","is_error":false}"#
        )[0],
        Event::ToolResult {
            is_error: false,
            ..
        }
    ));
    let ctx = TurnContext {
        turn: 1,
        ..Default::default()
    };
    let opts = LaunchOptions::default()
        .yolo(true)
        .model("m")
        .cwd("/tmp")
        .extra("max_turns", json!(2))
        .extra("tools", json!("a,b"))
        .extra("disallowed_tools", json!("c"))
        .extra("effort", json!("low"))
        .extra("include_partial", json!(true))
        .extra("session_id", json!("fixed-sid"));
    let prep = a.prepare("hi", &opts, &ctx).unwrap();
    assert!(prep.spawn.is_some());
    assert!(prep.multi_turn);
    let args = prep.spawn.unwrap().args;
    assert!(args.iter().any(|x| x == "--always-approve"));
    assert!(args.iter().any(|x| x == "streaming-messages-json"));
}

#[test]
fn pi_parse_and_prepare() {
    let a = PiAdapter;
    assert!(a.parse_line("").is_empty());
    assert!(matches!(&a.parse_line("x")[0], Event::Raw { .. }));
    assert!(matches!(
        &a.parse_line(r#"{"type":"session","id":"sid"}"#)[0],
        Event::SessionInfo { id, .. } if id == "sid"
    ));
    assert!(matches!(
        &a.parse_line(r#"{"type":"session","version":1}"#)[0],
        Event::Raw { .. }
    ));
    assert!(matches!(
        &a.parse_line(r#"{"type":"turn_start"}"#)[0],
        Event::TurnStart { .. }
    ));
    assert!(matches!(
        &a.parse_line(r#"{"type":"turn_end"}"#)[0],
        Event::TurnComplete { .. }
    ));
    assert!(matches!(
        &a.parse_line(r#"{"type":"agent_start"}"#)[0],
        Event::TurnStart { .. }
    ));
    assert!(a.parse_line(r#"{"type":"agent_end"}"#).is_empty());
    assert!(matches!(
        &a.parse_line(r#"{"type":"agent_settled"}"#)[0],
        Event::TurnComplete { .. }
    ));
    assert!(a.parse_line(r#"{"type":"message_start"}"#).is_empty());
    assert!(a.parse_line(r#"{"type":"message_end"}"#).is_empty());
    assert!(matches!(
        &a.parse_line(r#"{"type":"message_update","assistantMessageEvent":{"type":"text_delta","delta":"z"}}"#)[0],
        Event::TextDelta { text } if text == "z"
    ));
    assert!(a
        .parse_line(
            r#"{"type":"message_update","assistantMessageEvent":{"type":"text_delta","delta":""}}"#
        )
        .is_empty());
    assert!(matches!(
        &a.parse_line(r#"{"type":"message_update","assistantMessageEvent":{"type":"thinking_delta","delta":"q"}}"#)[0],
        Event::ThinkingDelta { .. }
    ));
    assert!(a
        .parse_line(r#"{"type":"message_update","assistantMessageEvent":{"type":"thinking_delta","delta":""}}"#)
        .is_empty());
    assert!(a
        .parse_line(r#"{"type":"message_update","assistantMessageEvent":{"type":"other"}}"#)
        .is_empty());
    assert!(a.parse_line(r#"{"type":"message_update"}"#).is_empty());
    // tool start → HookStarted(PreToolUse) + ToolCall
    let start = a
        .parse_line(r#"{"type":"tool_execution_start","toolCallId":"1","toolName":"t","args":{}}"#);
    assert!(start
        .iter()
        .any(|e| matches!(e, Event::HookStarted { name, .. } if name == "PreToolUse")));
    assert!(start.iter().any(|e| matches!(e, Event::ToolCall { .. })));
    assert!(a
        .parse_line(r#"{"type":"tool_call","id":"1","name":"t","input":null}"#)
        .iter()
        .any(|e| matches!(e, Event::ToolCall { .. })));
    let end = a.parse_line(
        r#"{"type":"tool_execution_end","toolCallId":"1","toolName":"t","result":"ok"}"#,
    );
    assert!(end.iter().any(|e| matches!(e, Event::ToolResult { .. })));
    assert!(end
        .iter()
        .any(|e| matches!(e, Event::HookFinished { name, .. } if name == "PostToolUse")));
    let tr = a.parse_line(r#"{"type":"tool_result","id":"1","output":{"a":1},"is_error":true}"#);
    assert!(tr
        .iter()
        .any(|e| matches!(e, Event::ToolResult { is_error: true, .. })));
    assert!(matches!(
        &a.parse_line(r#"{"type":"weird"}"#)[0],
        Event::Raw { .. }
    ));

    let opts = LaunchOptions::default()
        .yolo(true)
        .model("m")
        .extra("provider", json!("openai"))
        .extra("tools", json!("a"))
        .extra("exclude_tools", json!("b"))
        .extra("thinking", json!("low"))
        .extra("multi_turn", json!(true));
    let ctx1 = TurnContext {
        turn: 1,
        ..Default::default()
    };
    let p1 = a.prepare("p", &opts, &ctx1).unwrap();
    assert!(p1.spawn.unwrap().args.iter().any(|x| x == "--approve"));
    let ctx2 = TurnContext {
        turn: 2,
        session_id: None,
        ..Default::default()
    };
    let args2 = pi_prepare_args("p2", &opts, &ctx2).unwrap();
    assert!(args2.iter().any(|x| x == "--continue"));
    let mut opts_ns = opts.clone();
    opts_ns.extra.insert("no_session".into(), json!(true));
    let pns = a.prepare("p", &opts_ns, &ctx1).unwrap();
    assert!(pns.spawn.unwrap().args.iter().any(|x| x == "--no-session"));
    let mut opts_one = LaunchOptions::default();
    opts_one.extra.insert("multi_turn".into(), json!(false));
    let pone = a.prepare("p", &opts_one, &ctx1).unwrap();
    assert!(!pone.multi_turn);
}

#[test]
fn claude_and_generic_adapters() {
    let c = ClaudeAdapter;
    assert!(c.capabilities().streaming_json);
    assert!(c.capabilities().multi_turn);
    assert!(c.parse_line("").is_empty());
    assert!(matches!(&c.parse_line("nope")[0], Event::Raw { .. }));
    assert!(matches!(
        &c.parse_line(r#"{"type":"content_block_delta","delta":{"type":"text_delta","text":"hi"}}"#)
            [0],
        Event::TextDelta { text } if text == "hi"
    ));
    // empty text_delta / thinking_delta fall through
    assert!(matches!(
        &c.parse_line(r#"{"type":"content_block_delta","delta":{"type":"text_delta","text":""}}"#)
            [0],
        Event::Raw { .. }
    ));
    assert!(matches!(
        &c.parse_line(
            r#"{"type":"content_block_delta","delta":{"type":"thinking_delta","thinking":""}}"#
        )[0],
        Event::Raw { .. }
    ));
    assert!(matches!(
        &c.parse_line(r#"{"type":"content_block_delta","delta":{"type":"thinking_delta"}}"#)[0],
        Event::Raw { .. }
    ));
    // content blocks: empty text, unknown block type
    let _ = c.parse_line(
        r#"{"type":"assistant","message":{"content":[{"type":"text","text":""},{"type":"other"}]}}"#,
    );
    assert!(matches!(
        &c.parse_line(
            r#"{"type":"content_block_delta","delta":{"type":"thinking_delta","thinking":"t"}}"#
        )[0],
        Event::ThinkingDelta { .. }
    ));
    assert!(matches!(
        &c.parse_line(
            r#"{"type":"content_block_delta","delta":{"type":"thinking_delta","text":"t2"}}"#
        )[0],
        Event::ThinkingDelta { .. }
    ));
    // turn 1 without resume
    let ctx1 = TurnContext {
        turn: 1,
        ..Default::default()
    };
    let _ = c.prepare("p", &LaunchOptions::default(), &ctx1).unwrap();
    let assistant = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"A"},{"type":"tool_use","id":"1","name":"Bash","input":{"cmd":"ls"}}]}}"#;
    let evs = c.parse_line(assistant);
    assert!(evs.iter().any(|e| matches!(e, Event::TextDelta { .. })));
    assert!(evs.iter().any(|e| matches!(e, Event::ToolCall { .. })));
    assert!(matches!(
        &c.parse_line(r#"{"type":"assistant"}"#)[0],
        Event::Raw { .. }
    ));
    let res = c.parse_line(
        r#"{"type":"result","result":"final","num_turns":2,"subtype":"success","is_error":false}"#,
    );
    assert!(res.iter().any(|e| matches!(e, Event::TextDelta { .. })));
    assert!(res.iter().any(|e| matches!(e, Event::TurnComplete { .. })));
    assert!(!res.iter().any(|e| matches!(e, Event::Done { .. })));
    let res_err = c.parse_line(r#"{"type":"result","is_error":true}"#);
    assert!(res_err
        .iter()
        .any(|e| matches!(e, Event::TurnComplete { .. })));
    assert!(!res_err.iter().any(|e| matches!(e, Event::Done { .. })));
    assert!(matches!(
        &c.parse_line(r#"{"type":"tool_use","id":"1","name":"Bash","input":{}}"#)[0],
        Event::ToolCall { .. }
    ));
    assert!(matches!(
        &c.parse_line(r#"{"type":"error","error":"e"}"#)[0],
        Event::Error { .. }
    ));
    assert!(matches!(
        &c.parse_line(r#"{"type":"error","message":{"x":1}}"#)[0],
        Event::Error { .. }
    ));
    assert!(matches!(
        &c.parse_line(r#"{"event":"other"}"#)[0],
        Event::Raw { .. }
    ));
    let ctx = TurnContext {
        turn: 2,
        session_id: Some("sid".into()),
        ..Default::default()
    };
    let opts = LaunchOptions::default()
        .yolo(true)
        .model("m")
        .extra("max_turns", json!(3))
        .extra("allowed_tools", json!("Bash"));
    let prep = c.prepare("p", &opts, &ctx).unwrap();
    let args = prep.spawn.unwrap().args;
    assert!(args.windows(2).any(|w| w[0] == "--resume" && w[1] == "sid"));

    let g = GenericAdapter;
    assert!(g.prepare("p", &LaunchOptions::default(), &ctx).is_err());
    let mut opts = LaunchOptions::default().bin("/bin/echo");
    opts.extra.insert("args".into(), json!(["-n"]));
    opts.extra.insert("append_prompt".into(), json!(true));
    opts.extra.insert("retain_stdin".into(), json!(true));
    let prep = g.prepare("hello", &opts, &ctx).unwrap();
    assert!(prep.spawn.unwrap().retain_stdin);
    assert!(g.parse_line("").is_empty());
    assert!(matches!(
        &g.parse_line(r#"{"type":"text","data":"z"}"#)[0],
        Event::TextDelta { .. }
    ));
    // text without string data → fall through to Raw
    assert!(matches!(
        &g.parse_line(r#"{"type":"text","data":1}"#)[0],
        Event::Raw { .. }
    ));
    assert!(matches!(
        &g.parse_line(r#"{"type":"text"}"#)[0],
        Event::Raw { .. }
    ));
    assert!(matches!(
        &g.parse_line(r#"{"type":"end"}"#)[0],
        Event::Done { .. }
    ));
    assert!(matches!(
        &g.parse_line(r#"{"type":"done"}"#)[0],
        Event::Done { .. }
    ));
    assert!(matches!(&g.parse_line("plain")[0], Event::Raw { .. }));
    assert!(matches!(
        &g.parse_line(r#"{"type":"other"}"#)[0],
        Event::Raw { .. }
    ));
    let mut opts2 = LaunchOptions::default().bin("/bin/true");
    opts2.extra.insert("append_prompt".into(), json!(false));
    let prep2 = g.prepare("p", &opts2, &ctx).unwrap();
    assert!(!prep2.spawn.unwrap().args.iter().any(|a| a == "p"));
}

#[test]
fn mock_scenarios_and_encode() {
    let m = MockAdapter;
    assert!(m.encode_permission("p", true).unwrap().contains("perm"));
    assert!(m.encode_plan_resolve("pl", false).unwrap().contains("plan"));
    assert!(m.parse_line("x").is_empty());
    let ctx = TurnContext {
        turn: 1,
        ..Default::default()
    };
    for sc in [
        "echo",
        "tools",
        "error",
        "think",
        "multi",
        "permission",
        "plan",
        "goal",
        "hooks",
    ] {
        let opts = LaunchOptions::default().extra("scenario", json!(sc));
        let prep = m.prepare("prompt", &opts, &ctx).unwrap();
        assert!(prep.synthetic.is_some());
    }
    let yolo = LaunchOptions::default()
        .yolo(true)
        .extra("scenario", json!("permission"));
    let prep = m.prepare("x", &yolo, &ctx).unwrap();
    assert!(prep
        .synthetic
        .unwrap()
        .iter()
        .any(|e| matches!(e, Event::PermissionResolved { allowed: true, .. })));
    let yolo_plan = LaunchOptions::default()
        .yolo(true)
        .extra("scenario", json!("plan"));
    let prep = m.prepare("x", &yolo_plan, &ctx).unwrap();
    assert!(prep
        .synthetic
        .unwrap()
        .iter()
        .any(|e| matches!(e, Event::PlanResolved { approved: true, .. })));
    let ctx2 = TurnContext {
        turn: 2,
        history_text: "prior".into(),
        ..Default::default()
    };
    let multi = LaunchOptions::default().extra("scenario", json!("multi"));
    let prep = m.prepare("b", &multi, &ctx2).unwrap();
    let text: String = prep
        .synthetic
        .unwrap()
        .iter()
        .filter_map(|e| e.as_text_delta())
        .collect();
    assert!(text.contains("prior"));
    // empty prior history on turn 2
    let ctx_empty = TurnContext {
        turn: 2,
        history_text: String::new(),
        ..Default::default()
    };
    let prep = m.prepare("b", &multi, &ctx_empty).unwrap();
    let text: String = prep
        .synthetic
        .unwrap()
        .iter()
        .filter_map(|e| e.as_text_delta())
        .collect();
    assert!(text.contains("(none)"));
}

// --- transport ---

#[tokio::test(flavor = "multi_thread")]
async fn spawn_process_stdout_stderr_and_not_found() {
    let script = fixture_harness();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755));
    }
    let mut env = BTreeMap::new();
    env.insert("NO_COLOR".into(), "1".into());
    let mut io = spawn_process(SpawnSpec {
        program: script.clone(),
        args: vec!["--stderr-note".into(), "PING".into()],
        cwd: Some(PathBuf::from("/tmp")),
        env,
        retain_stdin: false,
    })
    .await
    .unwrap();
    let mut lines = Vec::new();
    while let Some(l) = io.lines_rx.recv().await {
        lines.push(l);
    }
    let _ = io.stderr_rx.recv().await;
    let status = io.child.wait().await.unwrap();
    assert!(status.success());
    assert!(lines
        .iter()
        .any(|l| l.contains("FAKE:") || l.contains("PING")));

    let mut io2 = spawn_process(SpawnSpec {
        program: PathBuf::from("/bin/echo"),
        args: vec!["hello".into()],
        cwd: None,
        env: BTreeMap::new(),
        retain_stdin: true,
    })
    .await
    .unwrap();
    assert!(io2.stdin.is_some());
    drop(io2.stdin.take());
    let _ = io2.lines_rx.recv().await;
    let _ = io2.child.wait().await;

    let err = match spawn_process(SpawnSpec {
        program: PathBuf::from("/no/such/automedon/binary/ever"),
        args: vec![],
        cwd: None,
        env: BTreeMap::new(),
        retain_stdin: false,
    })
    .await
    {
        Ok(_) => panic!("expected harness not found"),
        Err(e) => e,
    };
    assert!(matches!(err, Error::HarnessNotFound(_)));

    let _ = SpawnSpec::default();
}

// --- session process path via grok parser + fake harness ---

#[tokio::test(flavor = "multi_thread")]
async fn session_real_process_with_fake_harness_as_generic_then_grok_bin() {
    let script = fixture_harness();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755));
    }

    // Drive Grok adapter against our fake harness binary (same NDJSON end shape).
    let mut s = Session::builder("grok")
        .bin(&script)
        .yolo(true)
        .extra("multi_turn", json!(false))
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    s.prompt("ZAP").await.unwrap();
    s.expect(Expect::text("ZAP").timeout(Duration::from_secs(5)))
        .await
        .unwrap();
    s.expect(Expect::turn_complete().timeout(Duration::from_secs(5)))
        .await
        .unwrap();
    // process exit closes non-multi_turn session
    s.drain_until_done().await.unwrap();
    assert!(s.session_id().is_some() || !s.text().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn session_edges_mock() {
    let mut s = Session::builder("mock")
        .extra("scenario", json!("error"))
        .build()
        .unwrap();
    s.prompt("x").await.unwrap();
    s.expect(Expect::new(Predicate::Any(vec![
        Predicate::Done,
        // Error is not Done; wait for done after error
    ])))
    .await
    .ok();
    let _ = s.await_turn().await;
    s.close().await.unwrap();
    assert!(s.prompt("again").await.is_err());

    let mut s = Session::builder("mock")
        .cwd(".")
        .model("m")
        .yolo(false)
        .timeout(Duration::from_secs(5))
        .extra("scenario", json!("plan"))
        .build()
        .unwrap();
    s.prompt("work").await.unwrap();
    s.expect(Expect::plan()).await.unwrap();
    s.reject_plan().await.unwrap();
    s.expect(Expect::text("plan rejected")).await.unwrap();
    s.close().await.unwrap();

    let mut s = Session::builder("mock")
        .extra("scenario", json!("permission"))
        .build()
        .unwrap();
    s.prompt("x").await.unwrap();
    s.expect(Expect::permission()).await.unwrap();
    s.deny().await.unwrap();
    s.expect(Expect::text("denied")).await.unwrap();

    let mut s = Session::builder("mock")
        .extra("scenario", json!("multi"))
        .build()
        .unwrap();
    let r = s.run("one").await.unwrap();
    assert!(r.text.contains("T1:one") || r.turn_text.contains("T1:one"));
    assert_eq!(s.turn(), 1);
    s.prompt("two").await.unwrap();
    s.await_turn().await.unwrap();
    s.drain_until_done().await.unwrap();
    s.close().await.unwrap();
    s.close().await.unwrap(); // idempotent

    let r = run(
        "mock",
        "hi",
        LaunchOptions::default().extra("scenario", json!("echo")),
    )
    .await
    .unwrap();
    assert!(r.text.contains("ECHO:hi") || r.turn_text.contains("ECHO:hi"));

    let (mut s, _) = Session::builder("mock")
        .extra("scenario", json!("echo"))
        .run("z")
        .await
        .unwrap();
    assert!(!s.harness().is_empty());
    let _ = s.thinking();
    let _ = s.transcript();
    s.close().await.unwrap();

    // expect timeout
    let mut s = Session::builder("mock")
        .extra("scenario", json!("multi"))
        .timeout(Duration::from_millis(30))
        .build()
        .unwrap();
    s.prompt("a").await.unwrap();
    s.await_turn().await.unwrap();
    let err = s
        .expect(Expect::text("NEVER_APPEARS").timeout(Duration::from_millis(40)))
        .await;
    assert!(err.is_err());

    // no pending permission
    let mut s = Session::builder("mock")
        .extra("scenario", json!("echo"))
        .build()
        .unwrap();
    assert!(s.approve().await.is_err());
    assert!(s.deny().await.is_err());
    assert!(s.approve_plan().await.is_err());
    assert!(s.reject_plan().await.is_err());
}

// --- DSL ---

#[tokio::test(flavor = "multi_thread")]
async fn rhai_dsl_surface_coverage() {
    let src = r#"
        let s = launch("mock", #{
            scenario: "goal",
            cwd: ".",
            model: "m",
            yolo: false,
            timeout_ms: 5000,
            bin: "unused"
        });
        s.prompt("g");
        s.expect(timeout_ms(goal_title("g"), 5000));
        s.expect(goal_progress());
        s.expect(goal_completed(true));
        s.expect(turn_complete());
        assert_true(s.turn() > 0);
        print(s.thinking());
        print(s.session_id());
        print(s.harness());
        print(s.turn_text());
        s.close();
        true
    "#;
    automedon::dsl::eval_str(src).unwrap();

    let src = r#"
        let s = launch("mock", #{ scenario: "plan" });
        s.prompt("p");
        s.expect(plan());
        s.reject_plan();
        s.expect(text("plan rejected"));
        s.drain();
        s.close();
        true
    "#;
    automedon::dsl::eval_str(src).unwrap();

    let src = r#"
        let s = launch("mock", #{ scenario: "permission" });
        s.prompt("x");
        s.expect(permission());
        s.deny();
        s.expect(text("denied"));
        true
    "#;
    automedon::dsl::eval_str(src).unwrap();

    let src = r#"
        let s = launch("mock");
        let out = s.run("hi");
        assert_contains(out, "ECHO");
        true
    "#;
    automedon::dsl::eval_str(src).unwrap();

    // eval_file
    let path = std::env::temp_dir().join("automedon_cov_script.rhai");
    std::fs::write(
        &path,
        r#"let s = launch("mock", #{ scenario: "echo" }); s.run("z")"#,
    )
    .unwrap();
    automedon::dsl::eval_file(&path).unwrap();
    automedon::dsl::run_script(&path).unwrap();

    assert!(automedon::dsl::eval_str("assert_true(false)").is_err());
    assert!(automedon::dsl::eval_str(r#"assert_contains("a", "zzz")"#).is_err());
    assert!(automedon::dsl::eval_str("this is not valid rhai [[[").is_err());
}

#[test]
fn resolve_adapter_default_trait_parse_json() {
    // default parse_json on trait via Generic which overrides; Mock uses default empty
    let m = MockAdapter;
    assert!(m.parse_json(&json!({})).is_empty());
    assert!(m
        .session_id_from_event(&Event::Done { code: None })
        .is_none());
    assert!(
        m.session_id_from_event(&Event::SessionInfo {
            id: "x".into(),
            label: None
        })
        .as_deref()
            == Some("x")
    );
    assert!(m.encode_permission("i", false).is_some());
    assert!(m.encode_plan_resolve("i", true).is_some());
    // Grok default encode none for plan via... Grok doesn't override encode - returns None
    let g = GrokAdapter;
    assert!(g.encode_permission("i", true).is_none());
    assert!(g.encode_plan_resolve("i", true).is_none());
}
