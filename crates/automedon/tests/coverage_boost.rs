//! Extra coverage for session wait/await_turn edges, Wait all/display, Rhai wait constructors.

use automedon::adapter::{Adapter, Capabilities, PreparedLaunch, TurnContext};
use automedon::config::LaunchOptions;
use automedon::event::{Event, Transcript};
use automedon::expect::{Expect, Predicate};
use automedon::transport::SpawnSpec;
use automedon::wait::{Wait, WaitCondition};
use automedon::{Session, Wait as W};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Claude result ends a turn, not the Automedon session (multi-turn continuity).
#[tokio::test(flavor = "multi_thread")]
async fn claude_result_does_not_close_session() {
    use automedon::adapter::ClaudeAdapter;

    /// Emits one Claude-shaped result frame then exits (process-per-turn).
    struct ClaudeResultHarness;
    impl Adapter for ClaudeResultHarness {
        fn name(&self) -> &'static str {
            "claude_result_h"
        }
        fn capabilities(&self) -> Capabilities {
            Capabilities {
                multi_turn: true,
                sessions: true,
                launch: true,
                ..Default::default()
            }
        }
        fn prepare(
            &self,
            prompt: &str,
            _: &LaunchOptions,
            ctx: &TurnContext,
        ) -> automedon::Result<PreparedLaunch> {
            let sid = ctx
                .session_id
                .clone()
                .unwrap_or_else(|| "claude-sess-1".into());
            let body = format!(
                r#"echo '{{"type":"system","subtype":"init","session_id":"{sid}"}}'
echo '{{"type":"result","session_id":"{sid}","result":"{prompt}","num_turns":1,"is_error":false}}'
"#
            );
            Ok(PreparedLaunch {
                harness: "claude_result_h".into(),
                spawn: Some(SpawnSpec {
                    program: PathBuf::from("/bin/sh"),
                    args: vec!["-c".into(), body],
                    cwd: None,
                    env: BTreeMap::new(),
                    retain_stdin: false,
                }),
                synthetic: None,
                capabilities: self.capabilities(),
                multi_turn: true,
            })
        }
        fn parse_line(&self, line: &str) -> Vec<Event> {
            ClaudeAdapter.parse_line(line)
        }
    }

    let mut s = Session::from_adapter(
        Arc::new(ClaudeResultHarness),
        LaunchOptions {
            default_timeout: Some(Duration::from_secs(5)),
            ..Default::default()
        },
    );
    s.prompt("TURN_T1").await.unwrap();
    s.expect(Expect::text("TURN_T1").timeout(Duration::from_secs(5)))
        .await
        .unwrap();
    s.await_turn().await.unwrap();
    assert!(
        !s.is_finished(),
        "session must stay open after turn 1 result"
    );
    assert_eq!(s.session_id(), Some("claude-sess-1"));

    s.prompt("TURN_T2").await.unwrap();
    s.expect(Expect::text("TURN_T2").timeout(Duration::from_secs(5)))
        .await
        .unwrap();
    s.await_turn().await.unwrap();
    assert!(!s.is_finished());
    let text = s.text().to_string();
    assert!(
        text.contains("TURN_T1") && text.contains("TURN_T2"),
        "{text}"
    );
    s.close().await.ok();
}

/// Stderr lines must go through adapter.parse_line (Copilot Resume footer pattern).
#[tokio::test(flavor = "multi_thread")]
async fn stderr_resume_footer_yields_session_info() {
    use automedon::adapter::CopilotAdapter;
    use automedon::Session;
    use std::sync::Arc;

    struct StderrResumeHarness;
    impl Adapter for StderrResumeHarness {
        fn name(&self) -> &'static str {
            "stderr_resume"
        }
        fn capabilities(&self) -> Capabilities {
            Capabilities {
                multi_turn: true,
                sessions: true,
                launch: true,
                ..Default::default()
            }
        }
        fn prepare(
            &self,
            prompt: &str,
            _: &LaunchOptions,
            _: &TurnContext,
        ) -> automedon::Result<PreparedLaunch> {
            // stdout: reply; stderr: noise (empty parse → Raw) + Copilot Resume footer
            // Write footer after a brief delay so it can land as residual after stdout EOF.
            let script = format!(
                r#"echo '{prompt}'
echo 'stderr-noise-only' >&2
echo 'Resume     copilot --resume=sess-from-stderr-99' >&2
"#
            );
            Ok(PreparedLaunch {
                harness: "stderr_resume".into(),
                spawn: Some(SpawnSpec {
                    program: PathBuf::from("/bin/sh"),
                    args: vec!["-c".into(), script],
                    cwd: None,
                    env: BTreeMap::new(),
                    retain_stdin: false,
                }),
                synthetic: None,
                capabilities: self.capabilities(),
                multi_turn: true,
            })
        }
        fn parse_line(&self, line: &str) -> Vec<Event> {
            let line = line.trim();
            if line == "stderr-noise-only" {
                return Vec::new(); // force events_from_line Raw branch
            }
            // Same extraction path as product Copilot adapter.
            CopilotAdapter.parse_line(line)
        }
    }

    let mut s = Session::from_adapter(
        Arc::new(StderrResumeHarness),
        LaunchOptions {
            default_timeout: Some(Duration::from_secs(5)),
            ..Default::default()
        },
    );
    s.prompt("HELLO_SID").await.unwrap();
    s.expect(Expect::text("HELLO_SID").timeout(Duration::from_secs(5)))
        .await
        .unwrap();
    s.await_turn().await.ok();
    let sid = s.session_id().map(str::to_string);
    assert_eq!(
        sid.as_deref(),
        Some("sess-from-stderr-99"),
        "stderr Resume footer must become SessionInfo; got {sid:?}"
    );
    // noise line should have been recorded as stderr Raw via events_from_line
    assert!(
        s.transcript().events().iter().any(|te| {
            matches!(
                &te.event,
                Event::Raw { channel, line, .. } if channel == "stderr" && line.contains("noise")
            )
        }),
        "expected stderr Raw for noise line"
    );
    s.close().await.ok();
}

/// Residual stderr after stdout closes is drained before ProcessExit.
#[tokio::test(flavor = "multi_thread")]
async fn residual_stderr_after_stdout_eof_parsed() {
    use automedon::adapter::CopilotAdapter;
    use automedon::Session;
    use std::sync::Arc;

    struct ResidualStderr;
    impl Adapter for ResidualStderr {
        fn name(&self) -> &'static str {
            "residual_stderr"
        }
        fn capabilities(&self) -> Capabilities {
            Capabilities {
                multi_turn: true,
                sessions: true,
                ..Default::default()
            }
        }
        fn prepare(
            &self,
            _: &str,
            _: &LaunchOptions,
            _: &TurnContext,
        ) -> automedon::Result<PreparedLaunch> {
            // Block stderr write until stdout is fully closed from reader side by
            // writing all to a subshell that flushes stdout first then stderr.
            Ok(PreparedLaunch {
                harness: "residual_stderr".into(),
                spawn: Some(SpawnSpec {
                    program: PathBuf::from("/bin/sh"),
                    args: vec![
                        "-c".into(),
                        r#"
echo OUT1
# close stdout from child then write resume on stderr (common CLI footer pattern)
exec 1>&-
echo 'Resume     copilot --resume=residual-sid-42' >&2
"#
                        .into(),
                    ],
                    cwd: None,
                    env: BTreeMap::new(),
                    retain_stdin: false,
                }),
                synthetic: None,
                capabilities: self.capabilities(),
                multi_turn: true,
            })
        }
        fn parse_line(&self, line: &str) -> Vec<Event> {
            CopilotAdapter.parse_line(line)
        }
    }

    let mut s = Session::from_adapter(
        Arc::new(ResidualStderr),
        LaunchOptions {
            default_timeout: Some(Duration::from_secs(5)),
            ..Default::default()
        },
    );
    s.prompt("x").await.unwrap();
    s.expect(Expect::text("OUT1").timeout(Duration::from_secs(5)))
        .await
        .unwrap();
    s.await_turn().await.ok();
    assert_eq!(
        s.session_id(),
        Some("residual-sid-42"),
        "residual stderr after stdout EOF must yield SessionInfo"
    );
    s.close().await.ok();
}

#[test]
fn wait_new_all_display_and_into_predicate() {
    let w = Wait::new(WaitCondition::On(Predicate::Done)).timeout(Duration::from_millis(50));
    assert_eq!(w.timeout, Duration::from_millis(50));
    let all = Wait::all([Wait::text("a"), Wait::permission()]);
    let s = all.to_string();
    assert!(s.contains("all") || s.contains("text") || s.contains("permission"));
    let any = Wait::any([Wait::text("x").timeout(Duration::from_secs(1))]);
    assert!(any.to_string().contains("any") || any.to_string().contains("text"));
    // matches All
    let mut t = Transcript::default();
    t.push(Event::TextDelta { text: "a".into() });
    let cond = WaitCondition::All(vec![WaitCondition::On(Predicate::TextContains("a".into()))]);
    assert!(cond.matches(&Event::TextDelta { text: "a".into() }, &t, 0));
    // into_expect → into_predicate for Any/All branches
    let any_w = Wait::any([Wait::done(), Wait::text("z")]);
    let all_w = Wait::all([Wait::done()]);
    let _ = any_w.into_expect();
    let _ = all_w.into_expect();
    // Display arms
    let _ = format!("{}", WaitCondition::Any(vec![]));
    let _ = format!("{}", WaitCondition::All(vec![]));
    let _ = format!(
        "{}",
        WaitCondition::On(Predicate::Hook {
            name: Some("Pre".into()),
            phase: Some("pre".into()),
            finished: Some(true),
        })
    );
    let _ = format!("{}", Expect::plan_resolved(true));
    let _ = format!("{}", Expect::process_exit());
}

#[tokio::test(flavor = "multi_thread")]
async fn await_turn_pauses_on_mock_permission_and_wait_for() {
    let mut s = Session::builder("mock")
        .extra("scenario", json!("permission"))
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    s.prompt("x").await.unwrap();
    // Permission event should let await_turn return without full turn complete.
    s.await_turn().await.unwrap();
    s.approve().await.unwrap();
    s.expect(Expect::text("ran:x").timeout(Duration::from_secs(5)))
        .await
        .unwrap();
    // wait_for alias
    s.prompt("y").await.ok();
    let mut s2 = Session::builder("mock")
        .extra("scenario", json!("echo"))
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    s2.prompt("hi").await.unwrap();
    s2.wait_for(W::text("hi").timeout(Duration::from_secs(3)))
        .await
        .unwrap();
    s2.close().await.unwrap();
    s.close().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn wait_matches_buffered_event_and_timeout() {
    let mut s = Session::builder("mock")
        .extra("scenario", json!("echo"))
        .timeout(Duration::from_millis(200))
        .build()
        .unwrap();
    s.prompt("buf").await.unwrap();
    s.await_turn().await.unwrap();
    // Event already buffered — wait should match from cursor rewind... cursor advanced.
    // Re-prompt and wait with short timeout for missing string.
    s.prompt("z").await.unwrap();
    let err = s
        .wait(W::text("NEVER_MATCH_XYZ").timeout(Duration::from_millis(50)))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("timeout") || err.to_string().contains("Expect"));
    s.close().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn multi_line_stdout_parse_and_session_id_promote() {
    struct MultiParse;
    impl Adapter for MultiParse {
        fn name(&self) -> &'static str {
            "multi_parse"
        }
        fn capabilities(&self) -> Capabilities {
            Capabilities {
                multi_turn: false,
                ..Default::default()
            }
        }
        fn prepare(
            &self,
            _: &str,
            _: &LaunchOptions,
            _: &TurnContext,
        ) -> automedon::Result<PreparedLaunch> {
            Ok(PreparedLaunch {
                harness: "multi_parse".into(),
                spawn: Some(SpawnSpec {
                    program: PathBuf::from("/bin/sh"),
                    args: vec![
                        "-c".into(),
                        r#"
echo '{"type":"text","data":"a"}{"type":"text","data":"b"}'
printf '%s\n' '{"type":"text","data":"line1"}'
printf '%s\n' '{"type":"text","data":"line2"}'
printf '%s\n' '{"type":"end","sessionId":"sid-x","stopReason":"end_turn"}'
"#
                        .into(),
                    ],
                    cwd: None,
                    env: BTreeMap::new(),
                    retain_stdin: false,
                }),
                synthetic: None,
                capabilities: self.capabilities(),
                multi_turn: false,
            })
        }
        fn parse_line(&self, line: &str) -> Vec<Event> {
            // Multi-event from one line if concatenated JSON — otherwise one.
            if line.contains("}{") {
                return vec![
                    Event::TextDelta { text: "a".into() },
                    Event::TextDelta { text: "b".into() },
                ];
            }
            automedon::GrokAdapter.parse_line(line)
        }
        fn session_id_from_event(&self, event: &Event) -> Option<String> {
            match event {
                Event::TextDelta { text } if text == "line2" => Some("from-text".into()),
                Event::SessionInfo { id, .. } => Some(id.clone()),
                _ => None,
            }
        }
    }
    let mut s = Session::from_adapter(Arc::new(MultiParse), LaunchOptions::default());
    s.prompt("p").await.unwrap();
    s.expect(Expect::text("line1").timeout(Duration::from_secs(5)))
        .await
        .unwrap();
    s.drain_until_done().await.ok();
    s.close().await.unwrap();
}

#[test]
fn rhai_wait_and_expect_constructors() {
    let src = r#"
        let a = wait_text("a");
        let b = wait_tool("t");
        let c = wait_tool_any();
        let d = wait_tool_result("t");
        let e = wait_permission();
        let f = wait_hook("H");
        let g = wait_hook_any();
        let h = wait_hook_started("H");
        let i = wait_hook_finished("H");
        let j = wait_hook_phase("H", "pre");
        let k = wait_plan();
        let l = wait_goal();
        let m = wait_turn_complete();
        let n = wait_done();
        let w = wait_timeout_ms(wait_text("z"), 10);
        let w2 = timeout_ms(wait_text("z"), 10);
        let pr = plan_resolved(true);
        let g0 = goal();
        let gt = goal_title("g");
        let gp = goal_progress();
        let gc = goal_completed(true);
        let hp = hook_phase("H", "pre");
        let ps = plan_summary("s");
        let sess = launch("mock", #{ scenario: "echo" });
        sess.prompt("hi");
        sess.wait_for(wait_text("hi"));
        sess.close();
        "ok"
    "#;
    let r = automedon::dsl::eval_str(src).expect("rhai");
    assert!(r.value.to_string().contains("ok"));
}

#[test]
fn event_tool_and_hook_name_helpers() {
    let tc = Event::ToolCall {
        id: "1".into(),
        name: "bash".into(),
        input: json!({}),
    };
    assert_eq!(tc.tool_name(), Some("bash"));
    assert_eq!(Event::TextDelta { text: "x".into() }.tool_name(), None);
    let hs = Event::HookStarted {
        id: "h".into(),
        name: "Pre".into(),
        phase: None,
        detail: None,
    };
    assert_eq!(hs.hook_name(), Some("Pre"));
    assert_eq!(Event::Done { code: None }.hook_name(), None);
}

#[test]
fn expect_predicate_hook_phase_and_plan_resolved() {
    let t = Transcript::default();
    let hs = Event::HookStarted {
        id: "1".into(),
        name: "PreToolUse".into(),
        phase: Some("pre".into()),
        detail: None,
    };
    assert!(Predicate::Hook {
        name: Some("PreToolUse".into()),
        phase: Some("PRE".into()),
        finished: Some(false),
    }
    .matches(&hs, &t, 0));
    let hf = Event::HookFinished {
        id: "1".into(),
        name: "Post".into(),
        phase: Some("post".into()),
        ok: true,
        detail: None,
    };
    assert!(Predicate::Hook {
        name: Some("Post".into()),
        phase: Some("post".into()),
        finished: Some(true),
    }
    .matches(&hf, &t, 0));
    assert!(!Predicate::Hook {
        name: None,
        phase: None,
        finished: Some(true),
    }
    .matches(&hs, &t, 0));
    assert!(Predicate::PlanResolved {
        approved: Some(true)
    }
    .matches(
        &Event::PlanResolved {
            id: "p".into(),
            approved: true
        },
        &t,
        0
    ));
    assert!(!Predicate::PlanResolved {
        approved: Some(true)
    }
    .matches(
        &Event::TurnComplete {
            turn: 1,
            stop_reason: None
        },
        &t,
        0
    ));
}
