//! Opt-in live harness smokes. Skip unless `AUTOMEDON_LIVE_<ADAPTER>=1` and binary on PATH.
//!
//! ```text
//! AUTOMEDON_LIVE_GROK=1 cargo test -p automedon --test live_harness -- --ignored --nocapture
//! AUTOMEDON_LIVE_GROK_ACP=1 cargo test -p automedon --test live_harness live_grok_acp -- --ignored --nocapture
//! ```

use automedon::{Expect, Session, Wait};
use std::time::Duration;

fn bin_on_path(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| {
            std::env::split_paths(&p).any(|dir| {
                let candidate = dir.join(name);
                candidate.is_file()
            })
        })
        .unwrap_or(false)
}

fn live_enabled(adapter: &str) -> bool {
    let key = format!("AUTOMEDON_LIVE_{}", adapter.to_ascii_uppercase());
    matches!(
        std::env::var(&key).as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

fn skip_if(adapter: &str, bins: &[&str]) -> bool {
    if !live_enabled(adapter) {
        eprintln!(
            "skip: set AUTOMEDON_LIVE_{}=1",
            adapter.to_ascii_uppercase()
        );
        return true;
    }
    if !bins.iter().any(|b| bin_on_path(b)) {
        eprintln!("skip: none of {bins:?} on PATH");
        return true;
    }
    false
}

// --- Tier A ---

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live: set AUTOMEDON_LIVE_GROK=1"]
async fn live_grok_multi_turn() {
    if skip_if("grok", &["grok"]) {
        return;
    }
    let mut s = Session::builder("grok")
        .yolo(true)
        .timeout(Duration::from_secs(180))
        .extra("max_turns", serde_json::json!(4))
        .build()
        .expect("build");
    s.prompt("Reply with exactly: AUTOMEDON_LIVE_T1")
        .await
        .expect("t1");
    s.expect(Expect::text("AUTOMEDON_LIVE_T1").timeout(Duration::from_secs(120)))
        .await
        .expect("expect t1");
    s.await_turn().await.ok();
    s.prompt("Reply with exactly: AUTOMEDON_LIVE_T2")
        .await
        .expect("t2");
    s.expect(Expect::text("AUTOMEDON_LIVE_T2").timeout(Duration::from_secs(120)))
        .await
        .expect("expect t2");
    let text = s.text().to_string();
    eprintln!("live_grok text={text}");
    s.close().await.ok();
    assert!(text.contains("AUTOMEDON_LIVE_T1") && text.contains("AUTOMEDON_LIVE_T2"));
}

/// Full ACP client path: multi-turn + tool events on one `grok agent stdio` process.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "live ACP: set AUTOMEDON_LIVE_GROK_ACP=1"]
async fn live_grok_acp_multi_turn_and_tools() {
    if !matches!(
        std::env::var("AUTOMEDON_LIVE_GROK_ACP").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    ) || !bin_on_path("grok")
    {
        eprintln!("skip: AUTOMEDON_LIVE_GROK_ACP=1 and grok on PATH required");
        return;
    }
    let mut s = Session::builder("grok")
        .yolo(true)
        .timeout(Duration::from_secs(180))
        .extra("acp", serde_json::json!(true))
        .build()
        .expect("build");

    s.prompt("Reply with exactly: ACP_T1 and nothing else")
        .await
        .expect("prompt1");
    s.expect(Expect::text("ACP_T1").timeout(Duration::from_secs(120)))
        .await
        .expect("expect t1");
    s.await_turn().await.expect("await t1");

    s.prompt("Reply with exactly: ACP_T2")
        .await
        .expect("prompt2");
    s.expect(Expect::text("ACP_T2").timeout(Duration::from_secs(120)))
        .await
        .expect("expect t2");
    s.await_turn().await.expect("await t2");

    s.prompt(
        "Use a tool to list the current directory (list_dir or similar), then reply TOOL_DONE",
    )
    .await
    .expect("prompt tools");
    s.wait(Wait::tool_any().timeout(Duration::from_secs(120)))
        .await
        .expect("wait tool");
    s.expect(Expect::text("TOOL_DONE").timeout(Duration::from_secs(120)))
        .await
        .ok();
    s.await_turn().await.ok();

    let text = s.text().to_string();
    let tools: Vec<_> = s
        .transcript()
        .tools()
        .iter()
        .map(|t| t.name.clone())
        .collect();
    eprintln!(
        "live_acp text_len={} tools={tools:?} sample={}",
        text.len(),
        text.chars().take(200).collect::<String>()
    );
    s.close().await.ok();
    assert!(
        text.contains("ACP_T1") && text.contains("ACP_T2"),
        "multi-turn text missing: {text}"
    );
    assert!(
        !tools.is_empty(),
        "expected at least one tool call over ACP"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live: set AUTOMEDON_LIVE_CLAUDE=1"]
async fn live_claude_launch() {
    if skip_if("claude", &["claude"]) {
        return;
    }
    let mut s = Session::builder("claude")
        .yolo(true)
        .timeout(Duration::from_secs(180))
        .build()
        .expect("build");
    s.prompt("Reply with exactly: AUTOMEDON_LIVE_T1")
        .await
        .expect("prompt");
    s.expect(Expect::text("AUTOMEDON_LIVE_T1").timeout(Duration::from_secs(120)))
        .await
        .expect("expect");
    s.close().await.ok();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live: set AUTOMEDON_LIVE_CODEX=1"]
async fn live_codex_launch() {
    if skip_if("codex", &["codex"]) {
        return;
    }
    let mut s = Session::builder("codex")
        .yolo(true)
        .timeout(Duration::from_secs(180))
        .build()
        .expect("build");
    s.prompt("Reply with exactly: AUTOMEDON_LIVE_T1")
        .await
        .expect("prompt");
    s.expect(Expect::text("AUTOMEDON_LIVE_T1").timeout(Duration::from_secs(120)))
        .await
        .expect("expect");
    s.close().await.ok();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live: set AUTOMEDON_LIVE_GEMINI=1"]
async fn live_gemini_launch_and_text() {
    if skip_if("gemini", &["gemini", "agy"]) {
        return;
    }
    let mut s = Session::builder("gemini")
        .yolo(true)
        .timeout(Duration::from_secs(180))
        .build()
        .expect("build");
    s.prompt("Reply with exactly: AUTOMEDON_LIVE_T1 and nothing else")
        .await
        .expect("t1");
    s.expect(Expect::text("AUTOMEDON_LIVE_T1").timeout(Duration::from_secs(120)))
        .await
        .expect("expect t1");
    s.close().await.ok();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live: set AUTOMEDON_LIVE_OPENCODE=1"]
async fn live_opencode_launch() {
    if skip_if("opencode", &["opencode"]) {
        return;
    }
    let mut s = Session::builder("opencode")
        .yolo(true)
        .timeout(Duration::from_secs(180))
        .build()
        .expect("build");
    s.prompt("Reply with exactly: AUTOMEDON_LIVE_T1")
        .await
        .expect("prompt");
    s.expect(Expect::text("AUTOMEDON_LIVE_T1").timeout(Duration::from_secs(120)))
        .await
        .expect("expect");
    s.close().await.ok();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live: set AUTOMEDON_LIVE_CURSOR=1"]
async fn live_cursor_launch() {
    if skip_if("cursor", &["cursor-agent", "cursor"]) {
        return;
    }
    let mut s = Session::builder("cursor")
        .yolo(true)
        .timeout(Duration::from_secs(180))
        .build()
        .expect("build");
    s.prompt("Reply with exactly: AUTOMEDON_LIVE_T1")
        .await
        .expect("prompt");
    s.expect(Expect::text("AUTOMEDON_LIVE_T1").timeout(Duration::from_secs(120)))
        .await
        .expect("expect");
    s.close().await.ok();
}

// --- Tier B ---

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live: set AUTOMEDON_LIVE_PI=1"]
async fn live_pi_multi_turn() {
    if skip_if("pi", &["pi"]) {
        return;
    }
    // Default path (whatever provider is configured on the machine).
    let mut s = Session::builder("pi")
        .yolo(true)
        .timeout(Duration::from_secs(180))
        .build()
        .expect("build");
    s.prompt("Reply with exactly: AUTOMEDON_LIVE_T1")
        .await
        .expect("t1");
    s.expect(Expect::text("AUTOMEDON_LIVE_T1").timeout(Duration::from_secs(120)))
        .await
        .expect("expect t1");
    s.await_turn().await.ok();
    s.prompt("Reply with exactly: AUTOMEDON_LIVE_T2")
        .await
        .expect("t2");
    s.expect(Expect::text("AUTOMEDON_LIVE_T2").timeout(Duration::from_secs(120)))
        .await
        .expect("expect t2");
    let text = s.text().to_string();
    eprintln!("live_pi transcript_len={}", text.len());
    s.close().await.ok();
    assert!(!text.is_empty());
}

/// Pi + xAI: tool use emits general ToolCall and PreToolUse/PostToolUse hooks.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "live: set AUTOMEDON_LIVE_PI_XAI_TOOLS=1"]
async fn live_pi_xai_tools_and_hooks() {
    if !matches!(
        std::env::var("AUTOMEDON_LIVE_PI_XAI_TOOLS").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    ) || !bin_on_path("pi")
    {
        eprintln!("skip: AUTOMEDON_LIVE_PI_XAI_TOOLS=1 and pi on PATH");
        return;
    }
    let model = std::env::var("AUTOMEDON_PI_MODEL").unwrap_or_else(|_| "grok-4.5".into());
    let mut s = Session::builder("pi")
        .yolo(true)
        .model(&model)
        .timeout(Duration::from_secs(180))
        .extra("provider", serde_json::json!("xai"))
        .extra("tools", serde_json::json!("bash"))
        .build()
        .expect("build");
    s.prompt("Call the bash tool with command: echo hi. Then reply exactly: PI_TOOLS_DONE")
        .await
        .expect("prompt");
    // General assert API: tools and hooks both fire for Pi tool lifecycle.
    s.wait(
        Wait::any([Wait::hook("PreToolUse"), Wait::tool("bash")]).timeout(Duration::from_secs(120)),
    )
    .await
    .expect("wait PreToolUse or bash tool");
    s.expect(Expect::text("PI_TOOLS_DONE").timeout(Duration::from_secs(120)))
        .await
        .ok();
    let hooks = s
        .transcript()
        .events()
        .iter()
        .filter(|te| {
            matches!(
                te.event,
                automedon::Event::HookStarted { .. } | automedon::Event::HookFinished { .. }
            )
        })
        .count();
    let tools = s.transcript().tools().len();
    eprintln!("live_pi_xai_tools hooks={hooks} tools={tools}");
    s.close().await.ok();
    assert!(
        hooks > 0 || tools > 0,
        "expected tool or hook events from pi"
    );
}

/// Pi with xAI / Grok models (uses credentials already stored by `pi` for provider xai).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "live: set AUTOMEDON_LIVE_PI_XAI=1"]
async fn live_pi_xai_multi_turn() {
    if !matches!(
        std::env::var("AUTOMEDON_LIVE_PI_XAI").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    ) || !bin_on_path("pi")
    {
        eprintln!("skip: AUTOMEDON_LIVE_PI_XAI=1 and pi on PATH required");
        return;
    }
    let model = std::env::var("AUTOMEDON_PI_MODEL").unwrap_or_else(|_| "grok-4.5".into());
    let mut s = Session::builder("pi")
        .yolo(true)
        .model(&model)
        .timeout(Duration::from_secs(180))
        .extra("provider", serde_json::json!("xai"))
        .build()
        .expect("build");
    s.prompt("Reply with exactly: PI_XAI_T1").await.expect("t1");
    s.expect(Expect::text("PI_XAI_T1").timeout(Duration::from_secs(120)))
        .await
        .expect("expect t1");
    s.await_turn().await.ok();
    s.prompt("Reply with exactly: PI_XAI_T2").await.expect("t2");
    s.expect(Expect::text("PI_XAI_T2").timeout(Duration::from_secs(120)))
        .await
        .expect("expect t2");
    let text = s.text().to_string();
    eprintln!("live_pi_xai text={text}");
    s.close().await.ok();
    assert!(
        text.contains("PI_XAI_T1") && text.contains("PI_XAI_T2"),
        "multi-turn xAI text missing: {text}"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live: set AUTOMEDON_LIVE_AIDER=1"]
async fn live_aider_launch() {
    if skip_if("aider", &["aider"]) {
        return;
    }
    let mut s = Session::builder("aider")
        .timeout(Duration::from_secs(180))
        .model("xai/grok-4.5")
        .extra("no_git", serde_json::json!(true))
        .build()
        .expect("build");
    s.prompt("Reply with the word AUTOMEDON_LIVE_T1 only")
        .await
        .expect("prompt");
    s.expect(Expect::text("AUTOMEDON_LIVE_T1").timeout(Duration::from_secs(120)))
        .await
        .expect("expect");
    s.close().await.ok();
}

fn xai_key_from_env_or_pi() -> Option<String> {
    if let Ok(k) = std::env::var("XAI_API_KEY") {
        if !k.is_empty() {
            return Some(k);
        }
    }
    if !bin_on_path("pi") {
        return None;
    }
    let out = std::process::Command::new("pi")
        .args([
            "auth",
            "print-bearer-token",
            "--provider",
            "xai",
            "--model",
            "grok-4.5",
        ])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() || s.to_ascii_lowercase().contains("error") {
        None
    } else {
        Some(s)
    }
}

/// Aider + xAI multi-turn via chat-history restore.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "live: set AUTOMEDON_LIVE_AIDER_XAI=1"]
async fn live_aider_xai_multi_turn() {
    if !matches!(
        std::env::var("AUTOMEDON_LIVE_AIDER_XAI").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    ) || !bin_on_path("aider")
    {
        eprintln!("skip: AUTOMEDON_LIVE_AIDER_XAI=1 and aider on PATH required");
        return;
    }
    let Some(key) = xai_key_from_env_or_pi() else {
        eprintln!("skip: set XAI_API_KEY or install pi with xai auth");
        return;
    };
    let model = std::env::var("AUTOMEDON_AIDER_MODEL").unwrap_or_else(|_| "xai/grok-4.5".into());
    let hist = std::env::temp_dir().join(format!(
        "automedon-aider-live-{}.md",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut s = Session::builder("aider")
        .model(&model)
        .timeout(Duration::from_secs(180))
        .extra("no_git", serde_json::json!(true))
        .extra("xai_api_key", serde_json::json!(key))
        .extra(
            "chat_history_file",
            serde_json::json!(hist.display().to_string()),
        )
        .build()
        .expect("build");
    s.prompt("Reply with exactly: AIDER_T1 and nothing else")
        .await
        .expect("t1");
    s.expect(Expect::text("AIDER_T1").timeout(Duration::from_secs(120)))
        .await
        .expect("expect t1");
    s.await_turn().await.ok();
    assert!(
        s.session_id().is_some(),
        "aider should publish chat-history path as session id"
    );
    s.prompt(
        "Reply with exactly: AIDER_T2 and also write the token AIDER_T1 if you saw my previous message",
    )
    .await
    .expect("t2");
    s.expect(Expect::text("AIDER_T2").timeout(Duration::from_secs(120)))
        .await
        .expect("expect t2");
    let text = s.text().to_string();
    eprintln!(
        "live_aider_xai multi-turn text_len={} has_t1={} has_t2={}",
        text.len(),
        text.contains("AIDER_T1"),
        text.contains("AIDER_T2")
    );
    s.close().await.ok();
    let _ = std::fs::remove_file(&hist);
    assert!(
        text.contains("AIDER_T1") && text.contains("AIDER_T2"),
        "multi-turn continuity missing: {text}"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live: set AUTOMEDON_LIVE_COPILOT=1"]
async fn live_copilot_launch() {
    if skip_if("copilot", &["copilot"]) {
        return;
    }
    let mut s = Session::builder("copilot")
        .yolo(true)
        .timeout(Duration::from_secs(180))
        .build()
        .expect("build");
    s.prompt("Reply with exactly: AUTOMEDON_LIVE_T1")
        .await
        .expect("prompt");
    s.expect(Expect::text("AUTOMEDON_LIVE_T1").timeout(Duration::from_secs(120)))
        .await
        .expect("expect");
    s.close().await.ok();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live: set AUTOMEDON_LIVE_COPILOT=1"]
async fn live_copilot_multi_turn() {
    if skip_if("copilot", &["copilot"]) {
        return;
    }
    let mut s = Session::builder("copilot")
        .yolo(true)
        .timeout(Duration::from_secs(180))
        .build()
        .expect("build");
    s.prompt("Reply with exactly: COPILOT_T1 and nothing else")
        .await
        .expect("t1");
    s.expect(Expect::text("COPILOT_T1").timeout(Duration::from_secs(120)))
        .await
        .expect("expect t1");
    // Drain through process exit so stderr Resume footer is parsed → SessionInfo.
    s.await_turn().await.ok();
    let sid = s
        .session_id()
        .map(str::to_string)
        .expect("copilot multi-turn requires SessionInfo from stderr Resume footer");
    eprintln!("live_copilot multi-turn session_id={sid}");
    assert!(!sid.is_empty(), "empty session id after turn 1");
    s.prompt("You previously said COPILOT_T1. Reply with exactly: COPILOT_T2 and nothing else")
        .await
        .expect("t2");
    s.expect(Expect::text("COPILOT_T2").timeout(Duration::from_secs(120)))
        .await
        .expect("expect t2");
    let text = s.text().to_string();
    eprintln!(
        "live_copilot multi-turn session_id={sid} text_len={} has_t1={} has_t2={}",
        text.len(),
        text.contains("COPILOT_T1"),
        text.contains("COPILOT_T2")
    );
    s.close().await.ok();
    assert!(
        text.contains("COPILOT_T1") && text.contains("COPILOT_T2"),
        "multi-turn continuity missing: {text}"
    );
}
