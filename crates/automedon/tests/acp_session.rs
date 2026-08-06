//! Session ACP client path with a fixture agent (not mock scenario theater).

use automedon::adapter::{Adapter, Capabilities, PreparedLaunch, TurnContext};
use automedon::config::LaunchOptions;
use automedon::event::Event;
use automedon::transport::SpawnSpec;
use automedon::{Expect, Session, Wait};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Minimal ACP-speaking fixture: initialize → session/new → prompt responses.
struct FakeAcpAdapter {
    script: PathBuf,
}

impl Adapter for FakeAcpAdapter {
    fn name(&self) -> &'static str {
        "fake_acp"
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            launch: true,
            multi_turn: true,
            stream_tools: true,
            acp: true,
            sessions: true,
            ..Default::default()
        }
    }
    fn prepare(
        &self,
        _prompt: &str,
        opts: &LaunchOptions,
        _ctx: &TurnContext,
    ) -> automedon::Result<PreparedLaunch> {
        let use_acp = opts
            .extra
            .get("acp")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        assert!(use_acp, "fixture expects acp");
        Ok(PreparedLaunch {
            harness: "fake_acp".into(),
            spawn: Some(SpawnSpec {
                program: PathBuf::from("/bin/sh"),
                args: vec![self.script.display().to_string()],
                cwd: None,
                env: BTreeMap::new(),
                retain_stdin: true,
            }),
            synthetic: None,
            capabilities: self.capabilities(),
            multi_turn: true,
        })
    }
    fn parse_line(&self, line: &str) -> Vec<Event> {
        automedon::adapter::acp::parse_line(line)
    }
}

fn write_fixture_script(dir: &std::path::Path) -> PathBuf {
    let path = dir.join("fake_acp.sh");
    // Read JSON-RPC lines; reply with canned ACP frames.
    std::fs::write(
        &path,
        r#"#!/bin/sh
# Fake ACP agent for unit tests
while IFS= read -r line; do
  case "$line" in
    *'"method":"initialize"'*|*'"method": "initialize"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
      ;;
    *'"method":"authenticate"'*|*'"method": "authenticate"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{}}'
      ;;
    *'"method":"session/new"'*|*'"method": "session/new"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":3,"result":{"sessionId":"sess-fixture"}}'
      ;;
    *'"method":"session/prompt"'*|*'"method": "session/prompt"'*)
      # Extract prompt text roughly
      if echo "$line" | grep -q 'T1'; then
        printf '%s\n' '{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"ACP_T1"}}}}'
        printf '%s\n' '{"jsonrpc":"2.0","id":4,"result":{"stopReason":"end_turn"}}'
      elif echo "$line" | grep -q 'T2'; then
        printf '%s\n' '{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"ACP_T2"}}}}'
        printf '%s\n' '{"jsonrpc":"2.0","id":5,"result":{"stopReason":"end_turn"}}'
      else
        printf '%s\n' '{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"tool_call","toolCallId":"t1","title":"list_dir","rawInput":{}}}}'
        printf '%s\n' '{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"TOOL_DONE"}}}}'
        printf '%s\n' '{"jsonrpc":"2.0","id":6,"result":{"stopReason":"end_turn"}}'
      fi
      ;;
    *)
      printf '%s\n' '{"jsonrpc":"2.0","id":0,"result":{}}'
      ;;
  esac
done
"#,
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755));
    }
    path
}

#[tokio::test(flavor = "multi_thread")]
async fn session_acp_multi_turn_and_tool_wait() {
    let dir = tempfile_dir();
    let script = write_fixture_script(&dir);
    let adapter = Arc::new(FakeAcpAdapter { script });
    let mut opts = LaunchOptions::default();
    opts.extra.insert("acp".into(), serde_json::json!(true));
    opts.default_timeout = Some(Duration::from_secs(10));
    let mut s = Session::from_adapter(adapter, opts);

    s.prompt("say T1").await.unwrap();
    s.expect(Expect::text("ACP_T1").timeout(Duration::from_secs(5)))
        .await
        .unwrap();
    s.await_turn().await.unwrap();

    s.prompt("say T2").await.unwrap();
    s.expect(Expect::text("ACP_T2").timeout(Duration::from_secs(5)))
        .await
        .unwrap();
    s.await_turn().await.unwrap();

    s.prompt("tools please").await.unwrap();
    s.wait(Wait::tool_any().timeout(Duration::from_secs(5)))
        .await
        .unwrap();
    s.expect(Expect::text("TOOL_DONE").timeout(Duration::from_secs(5)))
        .await
        .unwrap();
    s.await_turn().await.unwrap();

    assert!(s.session_id().is_some() || s.text().contains("ACP"));
    assert!(!s.transcript().tools().is_empty());
    s.close().await.unwrap();
}

fn tempfile_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "automedon-acp-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Agent that fails session/new — exercises error path.
#[tokio::test(flavor = "multi_thread")]
async fn session_acp_session_new_error() {
    let dir = tempfile_dir();
    let path = dir.join("bad_acp.sh");
    std::fs::write(
        &path,
        r#"#!/bin/sh
while IFS= read -r line; do
  case "$line" in
    *initialize*) printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}' ;;
    *authenticate*) printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{}}' ;;
    *session/new*) printf '%s\n' '{"jsonrpc":"2.0","id":3,"error":{"message":"no session"}}' ;;
    *) printf '%s\n' '{"jsonrpc":"2.0","id":0,"result":{}}' ;;
  esac
done
"#,
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755));
    }
    let adapter = Arc::new(FakeAcpAdapter { script: path });
    let mut opts = LaunchOptions::default();
    opts.extra.insert("acp".into(), serde_json::json!(true));
    opts.default_timeout = Some(Duration::from_secs(5));
    let mut s = Session::from_adapter(adapter, opts);
    let err = s.prompt("x").await.unwrap_err();
    assert!(
        err.to_string().contains("no session") || err.to_string().contains("acp"),
        "{err}"
    );
}
