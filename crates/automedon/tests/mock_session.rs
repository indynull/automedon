use automedon::adapter::{grok_prepare_args, pi_prepare_args, TurnContext};
use automedon::{Event, Expect, LaunchOptions, Session, Transcript};

#[tokio::test(flavor = "multi_thread")]
async fn mock_echo_run() {
    let mut s = Session::builder("mock").build().unwrap();
    let result = s.run("hello").await.unwrap();
    assert!(result.turn_text.contains("ECHO:hello") || result.text.contains("ECHO:hello"));
}

#[tokio::test(flavor = "multi_thread")]
async fn mock_tools_expect() {
    let mut s = Session::builder("mock")
        .extra("scenario", serde_json::json!("tools"))
        .build()
        .unwrap();
    s.prompt("explore").await.unwrap();
    s.expect(Expect::tool("list_dir")).await.unwrap();
    s.expect(Expect::tool("read_file")).await.unwrap();
    s.expect(Expect::text("listed")).await.unwrap();
    s.expect(Expect::turn_complete()).await.unwrap();
    let tools: Vec<_> = s
        .transcript()
        .tools()
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(tools, ["list_dir", "read_file"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn mock_thinking() {
    let mut s = Session::builder("mock")
        .extra("scenario", serde_json::json!("think"))
        .build()
        .unwrap();
    s.prompt("x").await.unwrap();
    s.expect(Expect::thinking("thinking")).await.unwrap();
    s.expect(Expect::text("done:x")).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn mock_multi_turn_continuity() {
    let mut s = Session::builder("mock")
        .extra("scenario", serde_json::json!("multi"))
        .build()
        .unwrap();

    s.prompt("alpha").await.unwrap();
    s.expect(Expect::text("T1:alpha")).await.unwrap();
    s.await_turn().await.unwrap();
    assert!(!s.is_finished());
    assert!(s.text().contains("T1:alpha"));

    s.prompt("beta").await.unwrap();
    // Second turn must see first-turn history (continuity).
    s.expect(Expect::text("T2:beta")).await.unwrap();
    s.expect(Expect::text("prior=T1:alpha")).await.unwrap();
    s.await_turn().await.unwrap();

    assert!(s.text().contains("T1:alpha"));
    assert!(s.text().contains("T2:beta"));
    assert_eq!(s.turn(), 2);
    s.close().await.unwrap();
    assert!(s.is_finished());
}

/// Regression: after Session::begin_turn (prompt), stale needles must not match
/// non-text events even if the harness never emits TurnStart (Grok).
#[test]
fn expect_text_ignores_stale_turn_text_without_turn_start() {
    let mut t = Transcript::default();
    t.push(Event::TextDelta {
        text: "STALE_NEEDLE".into(),
    });
    assert!(t.turn_text().contains("STALE_NEEDLE"));
    // What Session::prompt does every turn (Grok may never emit TurnStart).
    t.begin_turn();
    assert_eq!(t.turn_text(), "");

    let spawned = Event::Spawned {
        pid: 42,
        harness: "grok".into(),
    };
    let since = t.events().len();
    assert!(
        !Expect::text("STALE_NEEDLE").matches(&spawned, &t, since),
        "Spawned must not match after begin_turn cleared turn_text"
    );
    assert!(t.text_since(since).is_empty());
}

/// Session::prompt must clear per-turn buffers even when the adapter never
/// emits TurnStart (real Grok path). Stale needles must not match non-text events.
#[tokio::test(flavor = "multi_thread")]
async fn prompt_clears_turn_text_so_stale_expect_fails() {
    let mut s = Session::builder("mock")
        .extra("scenario", serde_json::json!("multi"))
        .build()
        .unwrap();

    s.prompt("alpha").await.unwrap();
    s.expect(Expect::text("T1:alpha")).await.unwrap();
    s.await_turn().await.unwrap();
    assert!(s.text().contains("T1:alpha"));
    assert!(
        !s.turn_text().is_empty(),
        "turn1 should leave turn_text populated until next prompt"
    );

    // Next prompt resets turn aggregates immediately (before any new deltas).
    s.prompt("beta").await.unwrap();
    assert_eq!(
        s.turn_text(),
        "",
        "prompt must clear turn_text for adapters without TurnStart"
    );

    // Non-text boundary event must not match turn1 needle via leftover buffers.
    let boundary = Event::Spawned {
        pid: 1,
        harness: "grok".into(),
    };
    let since = s.transcript().events().len();
    assert!(
        !Expect::text("T1:alpha").matches(&boundary, s.transcript(), since),
        "after prompt, Spawned must not match prior-turn text"
    );

    // Real turn2 content still matches once streamed.
    s.expect(Expect::text("T2:beta")).await.unwrap();
    s.expect(Expect::text("prior=T1:alpha")).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn mock_permission_approve() {
    let mut s = Session::builder("mock")
        .extra("scenario", serde_json::json!("permission"))
        .build()
        .unwrap();
    s.prompt("secret").await.unwrap();
    s.expect(Expect::permission()).await.unwrap();
    s.approve().await.unwrap();
    s.expect(Expect::text("ran:secret")).await.unwrap();
    s.expect(Expect::turn_complete()).await.unwrap();
    assert!(s
        .transcript()
        .permissions()
        .iter()
        .any(|p| p.allowed == Some(true)));
}

#[tokio::test(flavor = "multi_thread")]
async fn mock_permission_deny() {
    let mut s = Session::builder("mock")
        .extra("scenario", serde_json::json!("permission"))
        .build()
        .unwrap();
    s.prompt("nope").await.unwrap();
    s.expect(Expect::permission()).await.unwrap();
    s.deny().await.unwrap();
    s.expect(Expect::text("denied")).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn mock_plan_approve() {
    let mut s = Session::builder("mock")
        .extra("scenario", serde_json::json!("plan"))
        .build()
        .unwrap();
    s.prompt("ship feature").await.unwrap();
    s.expect(Expect::plan_summary("ship feature"))
        .await
        .unwrap();
    s.approve_plan().await.unwrap();
    s.expect(Expect::plan_resolved(true)).await.unwrap();
    s.expect(Expect::text("executing plan")).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn wait_hooks_on_stream() {
    use automedon::Wait;

    let mut s = Session::builder("mock")
        .extra("scenario", serde_json::json!("hooks"))
        .build()
        .unwrap();
    s.prompt("hi").await.unwrap();
    s.wait(Wait::hook_started("PreToolUse")).await.unwrap();
    s.wait(Wait::tool("run_terminal_command")).await.unwrap();
    s.wait(Wait::hook_finished("PreToolUse")).await.unwrap();
    s.wait(Wait::hook_phase("PostToolUse", "post"))
        .await
        .unwrap();
    s.wait(Wait::text("hooks_done:hi")).await.unwrap();
    s.wait(Wait::turn_complete()).await.unwrap();
    assert!(s.transcript().hooks().len() >= 2);
    assert!(s
        .transcript()
        .hooks()
        .iter()
        .any(|h| h.name == "PreToolUse" && h.finished));
}

#[test]
fn wait_constructors_and_display() {
    use std::time::Duration;

    use automedon::{Wait, WaitCondition};

    let _ = Wait::text("a");
    let _ = Wait::thinking("t");
    let _ = Wait::tool("x");
    let _ = Wait::tool_any();
    let _ = Wait::tool_result("x");
    let _ = Wait::permission();
    let _ = Wait::hook("PreToolUse");
    let _ = Wait::hook_any();
    let _ = Wait::hook_started("H");
    let _ = Wait::hook_finished("H");
    let _ = Wait::hook_phase("H", "pre");
    let _ = Wait::plan();
    let _ = Wait::plan_summary("s");
    let _ = Wait::plan_resolved(true);
    let _ = Wait::goal();
    let _ = Wait::goal_title("g");
    let _ = Wait::goal_progress();
    let _ = Wait::goal_completed(false);
    let _ = Wait::turn_complete();
    let _ = Wait::done();
    let _ = Wait::process_exit();
    let _ = Wait::session_info();
    let _ = Wait::raw("stdout", "x");
    let w = Wait::all([Wait::text("a"), Wait::thinking("t")]).timeout(Duration::from_secs(1));
    let _ = format!("{w}");
    let _ = format!("{}", WaitCondition::On(automedon::Predicate::Done));
    let exp = Wait::on(Expect::text("z")).into_expect();
    assert!(format!("{exp}").contains("text"));
    assert!(exp.timeout > Duration::ZERO);
    assert!(automedon::wait::check_wait(&Wait::text("a")).is_ok());
    assert!(automedon::wait::check_wait(&Wait::text("a").timeout(Duration::ZERO)).is_err());
    let _ = automedon::wait::wait_timeout(&Wait::text("x"));
}

#[tokio::test(flavor = "multi_thread")]
async fn wait_any_permission_or_text() {
    use automedon::Wait;

    let mut s = Session::builder("mock")
        .extra("scenario", serde_json::json!("permission"))
        .build()
        .unwrap();
    s.prompt("x").await.unwrap();
    s.wait(Wait::any([Wait::permission(), Wait::text("never")]))
        .await
        .unwrap();
    s.approve().await.unwrap();
    s.wait(Wait::text("ran:x")).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn grok_parse_hook_events() {
    use automedon::{Adapter, GrokAdapter};

    let a = GrokAdapter;
    let start = a.parse_line(
        r#"{"type":"hook_start","id":"h1","name":"PreToolUse","phase":"pre","detail":{"tool":"bash"}}"#,
    );
    assert!(matches!(
        &start[0],
        automedon::Event::HookStarted { name, phase: Some(p), .. }
            if name == "PreToolUse" && p == "pre"
    ));
    let end = a
        .parse_line(r#"{"type":"hook_end","id":"h1","name":"PreToolUse","phase":"pre","ok":true}"#);
    assert!(matches!(
        &end[0],
        automedon::Event::HookFinished { ok: true, .. }
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn rhai_wait_hooks() {
    let src = r#"
        let s = launch("mock", #{ scenario: "hooks" });
        s.prompt("z");
        s.wait(wait_hook_started("PreToolUse"));
        s.wait(wait_tool("run_terminal_command"));
        s.wait(wait_hook_finished("PostToolUse"));
        s.wait(wait_text("hooks_done:z"));
        s.wait(wait_turn_complete());
        true
    "#;
    automedon::dsl::eval_str(src).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn mock_goal_lifecycle() {
    let mut s = Session::builder("mock")
        .extra("scenario", serde_json::json!("goal"))
        .build()
        .unwrap();
    s.prompt("fix CI").await.unwrap();
    s.expect(Expect::goal_title("fix CI")).await.unwrap();
    s.expect(Expect::goal_progress()).await.unwrap();
    s.expect(Expect::goal_completed(true)).await.unwrap();
    s.expect(Expect::text("goal_ok:fix CI")).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn grok_parse_streaming_json() {
    use automedon::{Adapter, GrokAdapter};

    let a = GrokAdapter;
    let events = a.parse_line(r#"{"type":"text","data":"hello"}"#);
    assert!(matches!(
        &events[0],
        automedon::Event::TextDelta { text } if text == "hello"
    ));
    let end = a.parse_line(
        r#"{"type":"end","stopReason":"end_turn","sessionId":"sid-123","num_turns":1}"#,
    );
    assert!(end
        .iter()
        .any(|e| matches!(e, automedon::Event::SessionInfo { id, .. } if id == "sid-123")));
    assert!(end
        .iter()
        .any(|e| matches!(e, automedon::Event::TurnComplete { .. })));
    // Multi-turn: end must not close the Automedon session by itself.
    assert!(!end
        .iter()
        .any(|e| matches!(e, automedon::Event::Done { .. })));
}

#[tokio::test(flavor = "multi_thread")]
async fn pi_parse_session_and_settled() {
    use automedon::{Adapter, PiAdapter};

    let a = PiAdapter;
    let sess = a.parse_line(
        r#"{"type":"session","version":3,"id":"pi-sess-9","timestamp":"t","cwd":"/tmp"}"#,
    );
    assert!(matches!(
        &sess[0],
        automedon::Event::SessionInfo { id, .. } if id == "pi-sess-9"
    ));
    let line =
        r#"{"type":"message_update","assistantMessageEvent":{"type":"text_delta","delta":"hi"}}"#;
    let events = a.parse_line(line);
    assert!(matches!(
        &events[0],
        automedon::Event::TextDelta { text } if text == "hi"
    ));
    let settled = a.parse_line(r#"{"type":"agent_settled"}"#);
    assert!(matches!(&settled[0], automedon::Event::TurnComplete { .. }));
}

#[tokio::test(flavor = "multi_thread")]
async fn grok_multi_turn_prepare_uses_resume() {
    let opts = LaunchOptions::default();
    let turn1 = TurnContext {
        turn: 1,
        session_id: None,
        ..Default::default()
    };
    let args1 = grok_prepare_args("first", &opts, &turn1).unwrap();
    assert!(args1.iter().any(|a| a == "-p"));
    assert!(args1.iter().any(|a| a == "first"));
    assert!(!args1.iter().any(|a| a == "--resume"));

    let turn2 = TurnContext {
        turn: 2,
        session_id: Some("sess-abc".into()),
        history_prompts: vec!["first".into()],
        history_text: "reply1".into(),
        ..Default::default()
    };
    let args2 = grok_prepare_args("second", &opts, &turn2).unwrap();
    assert!(
        args2
            .windows(2)
            .any(|w| w[0] == "--resume" && w[1] == "sess-abc"),
        "second turn must pass --resume <session id>, got {args2:?}"
    );
    assert!(args2.iter().any(|a| a == "second"));
    // Not a disconnected empty-history one-shot: resume flag present.
    assert_ne!(args1, args2);
}

#[tokio::test(flavor = "multi_thread")]
async fn pi_multi_turn_prepare_uses_session_or_continue() {
    let opts = LaunchOptions::default();
    let turn1 = TurnContext {
        turn: 1,
        session_id: None,
        ..Default::default()
    };
    let args1 = pi_prepare_args("first", &opts, &turn1).unwrap();
    assert!(args1.iter().any(|a| a == "-p"));
    assert!(!args1.iter().any(|a| a == "--continue"));

    let turn2 = TurnContext {
        turn: 2,
        session_id: Some("pi-uuid-1".into()),
        history_prompts: vec!["first".into()],
        history_text: "a".into(),
        ..Default::default()
    };
    let args2 = pi_prepare_args("second", &opts, &turn2).unwrap();
    assert!(
        args2
            .windows(2)
            .any(|w| w[0] == "--session-id" && w[1] == "pi-uuid-1")
            || args2.iter().any(|a| a == "--continue"),
        "second turn must resume session, got {args2:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn rhai_dsl_mock() {
    let src = r#"
        let s = launch("mock", #{ scenario: "echo" });
        s.prompt("yo");
        s.expect(text("ECHO:yo"));
        s.expect(turn_complete());
        assert_contains(s.text(), "ECHO:yo");
        s.text()
    "#;
    let result = automedon::dsl::eval_str(src).unwrap();
    assert!(result.value.to_string().contains("ECHO:yo"));
}

#[tokio::test(flavor = "multi_thread")]
async fn rhai_dsl_multi_turn() {
    let src = r#"
        let s = launch("mock", #{ scenario: "multi" });
        s.prompt("one");
        s.expect(text("T1:one"));
        s.await_turn();
        s.prompt("two");
        s.expect(text("T2:two"));
        s.expect(text("prior=T1:one"));
        s.await_turn();
        assert_contains(s.text(), "T1:one");
        assert_contains(s.text(), "T2:two");
        s.close();
        s.text()
    "#;
    let result = automedon::dsl::eval_str(src).unwrap();
    assert!(result.value.to_string().contains("prior=T1:one"));
}

#[tokio::test(flavor = "multi_thread")]
async fn rhai_dsl_permission() {
    let src = r#"
        let s = launch("mock", #{ scenario: "permission" });
        s.prompt("x");
        s.expect(permission());
        s.approve();
        s.expect(text("ran:x"));
        s.expect(turn_complete());
        true
    "#;
    automedon::dsl::eval_str(src).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn rhai_dsl_tools() {
    let src = r#"
        let s = launch("mock", #{ scenario: "tools" });
        s.prompt("x");
        s.expect(tool("list_dir"));
        s.expect(tool("read_file"));
        s.expect(turn_complete());
        let names = s.tool_names();
        assert_true(names.len() == 2);
        names
    "#;
    automedon::dsl::eval_str(src).unwrap();
}
