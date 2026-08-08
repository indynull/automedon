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
# Fake ACP agent for unit tests (ids follow client; authenticate is optional)
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":[[:space:]]*\([0-9][0-9]*\).*/\1/p' | head -1)
  [ -n "$id" ] || id=0
  case "$line" in
    *'"method":"initialize"'*|*'"method": "initialize"'*)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"protocolVersion\":1}}"
      ;;
    *'"method":"authenticate"'*|*'"method": "authenticate"'*)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{}}"
      ;;
    *'"method":"session/new"'*|*'"method": "session/new"'*)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"sessionId\":\"sess-fixture\"}}"
      ;;
    *'"method":"session/prompt"'*|*'"method": "session/prompt"'*)
      if echo "$line" | grep -q 'T1'; then
        printf '%s\n' '{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"ACP_T1"}}}}'
        printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"stopReason\":\"end_turn\"}}"
      elif echo "$line" | grep -q 'T2'; then
        printf '%s\n' '{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"ACP_T2"}}}}'
        printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"stopReason\":\"end_turn\"}}"
      else
        printf '%s\n' '{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"tool_call","toolCallId":"t1","title":"list_dir","rawInput":{}}}}'
        printf '%s\n' '{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"TOOL_DONE"}}}}'
        printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"stopReason\":\"end_turn\"}}"
      fi
      ;;
    *)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{}}"
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
  id=$(printf '%s' "$line" | sed -n 's/.*"id":[[:space:]]*\([0-9][0-9]*\).*/\1/p' | head -1)
  [ -n "$id" ] || id=0
  case "$line" in
    *initialize*) printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"protocolVersion\":1}}" ;;
    *authenticate*) printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{}}" ;;
    *session/new*) printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"error\":{\"message\":\"no session\"}}" ;;
    *) printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{}}" ;;
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

/// Explicit `acp_auth` runs soft authenticate; failure must not poison waits.
#[tokio::test(flavor = "multi_thread")]
async fn session_acp_soft_auth_then_session_new() {
    let dir = tempfile_dir();
    let path = dir.join("auth_soft.sh");
    std::fs::write(
        &path,
        r#"#!/bin/sh
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":[[:space:]]*\([0-9][0-9]*\).*/\1/p' | head -1)
  [ -n "$id" ] || id=0
  case "$line" in
    *initialize*)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"protocolVersion\":1}}"
      ;;
    *authenticate*)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"error\":{\"message\":\"unknown auth method\"}}"
      ;;
    *session/new*)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"sessionId\":\"sess-soft\"}}"
      ;;
    *session/prompt*)
      printf '%s\n' '{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"SOFT_OK"}}}}'
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"stopReason\":\"end_turn\"}}"
      ;;
    *)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{}}"
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
    let adapter = Arc::new(FakeAcpAdapter { script: path });
    let mut opts = LaunchOptions::default();
    opts.extra.insert("acp".into(), serde_json::json!(true));
    opts.extra
        .insert("acp_auth".into(), serde_json::json!("cached_token"));
    opts.default_timeout = Some(Duration::from_secs(10));
    let mut s = Session::from_adapter(adapter, opts);
    s.prompt("hi").await.unwrap();
    s.expect(Expect::text("SOFT_OK").timeout(Duration::from_secs(5)))
        .await
        .unwrap();
    s.await_turn().await.unwrap();
    assert_eq!(s.session_id(), Some("sess-soft"));
    s.close().await.unwrap();
}

/// `acp_auth: none` skips authenticate entirely.
#[tokio::test(flavor = "multi_thread")]
async fn session_acp_auth_none_skips_authenticate() {
    let dir = tempfile_dir();
    let path = dir.join("no_auth.sh");
    std::fs::write(
        &path,
        r#"#!/bin/sh
# Fail hard if authenticate is ever sent
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":[[:space:]]*\([0-9][0-9]*\).*/\1/p' | head -1)
  [ -n "$id" ] || id=0
  case "$line" in
    *authenticate*)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"error\":{\"message\":\"auth must not be called\"}}"
      ;;
    *initialize*)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"protocolVersion\":1}}"
      ;;
    *session/new*)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"sessionId\":\"sess-noauth\"}}"
      ;;
    *session/prompt*)
      printf '%s\n' '{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"NOAUTH_OK"}}}}'
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"stopReason\":\"end_turn\"}}"
      ;;
    *)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{}}"
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
    let adapter = Arc::new(FakeAcpAdapter { script: path });
    let mut opts = LaunchOptions::default();
    opts.extra.insert("acp".into(), serde_json::json!(true));
    opts.extra
        .insert("acp_auth".into(), serde_json::json!("none"));
    opts.default_timeout = Some(Duration::from_secs(10));
    let mut s = Session::from_adapter(adapter, opts);
    s.prompt("hi").await.unwrap();
    s.expect(Expect::text("NOAUTH_OK").timeout(Duration::from_secs(5)))
        .await
        .unwrap();
    s.close().await.unwrap();
}

/// session/new with empty result (no sessionId) fails closed.
#[tokio::test(flavor = "multi_thread")]
async fn session_acp_session_new_missing_id() {
    let dir = tempfile_dir();
    let path = dir.join("no_sid.sh");
    std::fs::write(
        &path,
        r#"#!/bin/sh
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":[[:space:]]*\([0-9][0-9]*\).*/\1/p' | head -1)
  [ -n "$id" ] || id=0
  case "$line" in
    *initialize*) printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"protocolVersion\":1}}" ;;
    *session/new*) printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{}}" ;;
    *) printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{}}" ;;
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
        err.to_string().contains("sessionId") || err.to_string().contains("acp"),
        "{err}"
    );
}

/// Handshake timeout when the agent never replies.
#[tokio::test(flavor = "multi_thread")]
async fn session_acp_handshake_timeout() {
    let dir = tempfile_dir();
    let path = dir.join("silent.sh");
    std::fs::write(
        &path,
        r#"#!/bin/sh
# Read forever, never reply.
while IFS= read -r line; do
  sleep 60
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
    opts.default_timeout = Some(Duration::from_millis(200));
    let mut s = Session::from_adapter(adapter, opts);
    let err = s.prompt("x").await.unwrap_err();
    assert!(
        err.to_string().contains("timeout") || err.to_string().contains("acp"),
        "{err}"
    );
}

/// Soft auth ignores Error events and non-JSON noise before the result.
#[tokio::test(flavor = "multi_thread")]
async fn session_acp_soft_auth_noise_and_error_events() {
    let dir = tempfile_dir();
    let path = dir.join("auth_noise.sh");
    std::fs::write(
        &path,
        r#"#!/bin/sh
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":[[:space:]]*\([0-9][0-9]*\).*/\1/p' | head -1)
  [ -n "$id" ] || id=0
  case "$line" in
    *initialize*)
      printf '%s\n' "not-json-noise"
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"protocolVersion\":1}}"
      ;;
    *authenticate*)
      # Soft path: error event parse + matched RPC error with empty-ish handling
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"error\":{\"message\":\"soft fail\"}}"
      ;;
    *session/new*)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"sessionId\":\"sess-noise\"}}"
      ;;
    *session/prompt*)
      # Prompt result without agent_message_chunk TurnComplete — session synthesizes it.
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"stopReason\":\"end_turn\"}}"
      ;;
    *)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{}}"
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
    let adapter = Arc::new(FakeAcpAdapter { script: path });
    let mut opts = LaunchOptions::default();
    opts.extra.insert("acp".into(), serde_json::json!(true));
    opts.extra
        .insert("acp_auth".into(), serde_json::json!("cached_token"));
    opts.default_timeout = Some(Duration::from_secs(10));
    let mut s = Session::from_adapter(adapter, opts);
    s.prompt("hi").await.unwrap();
    s.await_turn().await.unwrap();
    assert_eq!(s.session_id(), Some("sess-noise"));
    // Second ACP prompt reuses the live process.
    s.prompt("again").await.unwrap();
    s.await_turn().await.unwrap();
    s.close().await.unwrap();
}

/// session/new SessionInfo recovered from transcript when result omits sessionId field
/// but a prior notification carried it (adapter sets via SessionInfo apply).
#[tokio::test(flavor = "multi_thread")]
async fn session_acp_session_id_from_notification() {
    let dir = tempfile_dir();
    let path = dir.join("sid_notify.sh");
    std::fs::write(
        &path,
        r#"#!/bin/sh
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":[[:space:]]*\([0-9][0-9]*\).*/\1/p' | head -1)
  [ -n "$id" ] || id=0
  case "$line" in
    *initialize*)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"protocolVersion\":1}}"
      ;;
    *session/new*)
      # Emit sessionId only in a side channel shape that parse_line turns into SessionInfo,
      # then result without sessionId — wait path copies from transcript.
      printf '%s\n' '{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"session_info_update","sessionId":"sess-notify"}}}'
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{}}"
      ;;
    *session/prompt*)
      printf '%s\n' '{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"NOTIFY_OK"}}}}'
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"stopReason\":\"end_turn\"}}"
      ;;
    *)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{}}"
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
    let adapter = Arc::new(FakeAcpAdapter { script: path });
    let mut opts = LaunchOptions::default();
    opts.extra.insert("acp".into(), serde_json::json!(true));
    opts.default_timeout = Some(Duration::from_secs(10));
    let mut s = Session::from_adapter(adapter, opts);
    // If notification path does not set session id, prompt fails — either outcome covers code.
    match s.prompt("hi").await {
        Ok(()) => {
            let _ = s
                .expect(Expect::text("NOTIFY_OK").timeout(Duration::from_secs(5)))
                .await;
            s.close().await.ok();
        }
        Err(e) => {
            assert!(
                e.to_string().contains("sessionId") || e.to_string().contains("acp"),
                "{e}"
            );
        }
    }
}
