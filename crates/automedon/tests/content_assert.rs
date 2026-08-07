//! Tool input/output content waits on the harness event stream.

use automedon::adapter::{Adapter, MockAdapter, TurnContext};
use automedon::config::LaunchOptions;
use automedon::event::{Event, Transcript};
use automedon::{Expect, Session, Wait};
use serde_json::json;
use std::time::Duration;

#[test]
fn tool_input_and_result_output_predicates() {
    let t = Transcript::default();
    let call = Event::ToolCall {
        id: "1".into(),
        name: "write_file".into(),
        input: json!({ "path": "src/lib.rs", "content": "fn fib(n: i32) -> i32 { n }" }),
    };
    assert!(Expect::tool_input("write_file", "fib").matches(&call, &t, 0));
    assert!(Expect::tool_input("write_file", "src/lib.rs").matches(&call, &t, 0));
    assert!(!Expect::tool_input("write_file", "missing").matches(&call, &t, 0));
    assert!(!Expect::tool_input("other", "fib").matches(&call, &t, 0));

    let res = Event::ToolResult {
        id: "1".into(),
        name: "write_file".into(),
        output: "wrote 24 bytes to src/lib.rs".into(),
        is_error: false,
    };
    assert!(Expect::tool_result_contains("write_file", "24 bytes").matches(&res, &t, 0));
    assert!(Expect::tool_result_error("write_file", false).matches(&res, &t, 0));
    assert!(!Expect::tool_result_error("write_file", true).matches(&res, &t, 0));
    assert!(!Expect::tool_result_contains("write_file", "failed").matches(&res, &t, 0));

    assert!(Wait::tool_input("write_file", "fib").matches(&call, &t, 0));
    assert!(Wait::tool_result_contains("write_file", "src/lib.rs").matches(&res, &t, 0));
}

#[tokio::test(flavor = "multi_thread")]
async fn mock_tools_session_content_waits() {
    let mut s = Session::builder("mock")
        .extra("scenario", json!("tools"))
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    s.prompt("list workspace").await.unwrap();
    s.expect(Expect::tool_input("list_dir", ".").timeout(Duration::from_secs(3)))
        .await
        .unwrap();
    s.expect(
        Expect::tool_result_contains("list_dir", "Cargo.toml").timeout(Duration::from_secs(3)),
    )
    .await
    .unwrap();
    s.expect(Expect::tool_input("read_file", "Cargo.toml").timeout(Duration::from_secs(3)))
        .await
        .unwrap();
    s.expect(Expect::text("listed").timeout(Duration::from_secs(3)))
        .await
        .unwrap();
    s.close().await.ok();
}

#[test]
fn rhai_content_waits_on_stream() {
    let script = r#"
        let s = launch("mock", #{ scenario: "tools", timeout_ms: 5_000 });
        s.prompt("list");
        s.expect(tool_input("list_dir", "."));
        s.expect(tool_result_contains("list_dir", "src/"));
        s.wait(wait_tool_input("read_file", "Cargo.toml"));
        s.wait(wait_tool_result_contains("read_file", "workspace"));
        s.expect(text("listed"));
        s.close();
        "CONTENT_OK"
    "#;
    let res = automedon::dsl::eval_str(script).expect("eval");
    assert!(
        res.value.to_string().contains("CONTENT_OK"),
        "{:?}",
        res.value
    );
}

#[test]
fn mock_prepare_tools_has_list_dir_input() {
    let a = MockAdapter;
    let mut extra = std::collections::BTreeMap::new();
    extra.insert("scenario".into(), json!("tools"));
    let events = a
        .prepare(
            "x",
            &LaunchOptions {
                extra,
                ..Default::default()
            },
            &TurnContext {
                turn: 1,
                ..Default::default()
            },
        )
        .unwrap()
        .synthetic
        .unwrap();
    assert!(events.iter().any(|e| matches!(
        e,
        Event::ToolCall { name, input, .. }
            if name == "list_dir" && input.to_string().contains(".")
    )));
}
