//! Session: multi-turn harness control, expect, permissions, plan/goal.

use std::sync::Arc;
use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::time::{timeout, Instant};

use crate::adapter::{
    acp, mock_permission_continue, mock_plan_continue, resolve, Adapter, Capabilities, TurnContext,
};
use crate::config::LaunchOptions;
use crate::error::{Error, Result};
use crate::event::{Event, Transcript};
use crate::expect::Expect;
use crate::transport::{spawn_process, ChildIo};
use crate::wait::{check_wait, wait_needs_hooks, wait_needs_tools, Wait};

/// High-level handle — multi-turn session against one harness adapter.
pub struct Session {
    adapter: Arc<dyn Adapter>,
    opts: LaunchOptions,
    transcript: Transcript,
    default_timeout: Duration,
    child: Option<ChildIo>,
    /// Session fully closed.
    closed: bool,
    /// Current turn still streaming (live child or synthetic pending).
    turn_active: bool,
    /// Adapter reported multi_turn for the active/last prepare.
    multi_turn: bool,
    harness: String,
    turn: u64,
    /// Expect only matches events at or after this index.
    expect_cursor: usize,
    history_prompts: Vec<String>,
    /// Last user prompt (for mock permission/plan continue).
    last_prompt: Option<String>,
    /// Synthetic event queue for in-process adapters (multi-turn / approve).
    synthetic_queue: Vec<Event>,
    pending_permission: Option<String>,
    pending_plan: Option<String>,
    /// ACP stdio mode: long-lived process, JSON-RPC prompts.
    acp_mode: bool,
    /// Next JSON-RPC request id.
    acp_rpc_id: u64,
    /// ACP session id from session/new.
    acp_session_id: Option<String>,
    /// Outstanding session/prompt request id (await_turn ends on its result).
    acp_pending_prompt_id: Option<u64>,
    /// Last advertised capabilities from prepare (or adapter default at build).
    caps: Capabilities,
}

/// Result of a completed turn or one-shot run.
#[derive(Debug, Clone)]
pub struct RunResult {
    pub text: String,
    pub turn_text: String,
    pub thinking: String,
    pub code: Option<i32>,
    pub events: usize,
    pub session_id: Option<String>,
}

impl Session {
    pub fn builder(harness: impl AsRef<str>) -> SessionBuilder {
        SessionBuilder {
            harness: harness.as_ref().to_string(),
            opts: LaunchOptions::default(),
        }
    }

    pub fn from_adapter(adapter: Arc<dyn Adapter>, opts: LaunchOptions) -> Self {
        let default_timeout = opts.default_timeout_or(Duration::from_secs(120));
        let harness = adapter.name().to_string();
        let caps = adapter.capabilities();
        let multi_turn = caps.multi_turn;
        Self {
            adapter,
            opts,
            transcript: Transcript::default(),
            default_timeout,
            child: None,
            closed: false,
            turn_active: false,
            multi_turn,
            harness,
            turn: 0,
            expect_cursor: 0,
            history_prompts: Vec::new(),
            last_prompt: None,
            synthetic_queue: Vec::new(),
            pending_permission: None,
            pending_plan: None,
            acp_mode: false,
            acp_rpc_id: 1,
            acp_session_id: None,
            acp_pending_prompt_id: None,
            caps,
        }
    }

    /// Last known capabilities (from adapter / last prepare).
    pub fn capabilities(&self) -> &Capabilities {
        &self.caps
    }

    fn require_cap(&self, feature: &str, ok: bool) -> Result<()> {
        if ok {
            Ok(())
        } else {
            Err(Error::Other(format!(
                "capability not supported on {}: {feature}",
                self.harness
            )))
        }
    }

    pub fn harness(&self) -> &str {
        &self.harness
    }

    pub fn transcript(&self) -> &Transcript {
        &self.transcript
    }

    pub fn text(&self) -> &str {
        self.transcript.text()
    }

    pub fn turn_text(&self) -> &str {
        self.transcript.turn_text()
    }

    pub fn thinking(&self) -> &str {
        self.transcript.thinking()
    }

    pub fn session_id(&self) -> Option<&str> {
        self.transcript.session_id()
    }

    pub fn is_finished(&self) -> bool {
        self.closed
    }

    pub fn turn(&self) -> u64 {
        self.turn
    }

    /// One-shot: prompt, wait until turn (or session) settles, return aggregate.
    pub async fn run(&mut self, prompt: impl AsRef<str>) -> Result<RunResult> {
        self.prompt(prompt).await?;
        self.await_turn().await?;
        Ok(self.run_result())
    }

    /// Start a user turn (multi-turn safe). Accumulates transcript; does not wipe history.
    pub async fn prompt(&mut self, text: impl AsRef<str>) -> Result<()> {
        if self.closed {
            return Err(Error::SessionFinished);
        }
        // Finish any lingering turn stream before starting a new prompt.
        if self.turn_active {
            self.await_turn().await?;
        }

        let prompt = text.as_ref().to_string();
        self.turn += 1;
        self.last_prompt = Some(prompt.clone());

        let want_acp = self
            .opts
            .extra
            .get("acp")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // ACP multi-turn: reuse live process and send session/prompt only.
        if want_acp && self.acp_mode && self.child.is_some() && self.acp_session_id.is_some() {
            self.history_prompts.push(prompt.clone());
            self.expect_cursor = self.transcript.events().len();
            self.transcript.begin_turn();
            self.turn_active = true;
            self.multi_turn = true;
            self.push(Event::TurnStart { turn: self.turn });
            self.acp_send_prompt(&prompt).await?;
            return Ok(());
        }

        let ctx = TurnContext {
            turn: self.turn,
            session_id: self.transcript.session_id().map(str::to_string),
            history_prompts: self.history_prompts.clone(),
            history_text: self.transcript.text().to_string(),
            pending_permission: self.pending_permission.clone(),
            pending_plan: self.pending_plan.clone(),
        };

        let prepared = self.adapter.prepare(&prompt, &self.opts, &ctx)?;
        self.harness = prepared.harness.clone();
        self.multi_turn = prepared.multi_turn;
        self.caps = prepared.capabilities.clone();
        self.history_prompts.push(prompt.clone());
        // New turn: advance expect cursor and reset per-turn aggregates so
        // adapters that never emit TurnStart (Grok) cannot leave stale text
        // matching the next expect on Spawned/raw events.
        self.expect_cursor = self.transcript.events().len();
        self.transcript.begin_turn();
        self.turn_active = true;

        // Synthetic-only (mock) or seed events before a real spawn
        // (e.g. Aider injects SessionInfo with chat-history path).
        if let Some(events) = prepared.synthetic {
            if prepared.spawn.is_none() {
                self.enqueue_synthetic(events);
                return Ok(());
            }
            // Apply SessionInfo immediately so multi-turn resume has an id
            // even if the caller has not yet drained the stream.
            for ev in events {
                if matches!(ev, Event::SessionInfo { .. }) {
                    self.apply_event(ev);
                } else {
                    self.enqueue_synthetic(vec![ev]);
                }
            }
        }

        let spawn = prepared
            .spawn
            .ok_or_else(|| Error::Other("adapter produced neither spawn nor synthetic".into()))?;

        let retain = spawn.retain_stdin;
        let is_acp = want_acp && retain;

        // Drop prior child unless we are about to ACP-reuse (handled above).
        if let Some(mut io) = self.child.take() {
            let _ = io.child.kill().await;
            let _ = io.child.wait().await;
        }
        self.acp_mode = false;
        self.acp_session_id = None;
        self.acp_pending_prompt_id = None;

        let io = spawn_process(spawn).await?;
        let pid = io.child.id().unwrap_or(0);
        // Session-level turn boundary (harness may not emit TurnStart).
        self.push(Event::TurnStart { turn: self.turn });
        self.push(Event::Spawned {
            pid,
            harness: self.harness.clone(),
        });
        self.child = Some(io);

        if is_acp {
            self.acp_mode = true;
            self.multi_turn = true;
            self.acp_handshake_and_prompt(&prompt).await?;
        }
        Ok(())
    }

    async fn acp_write_line(&mut self, line: &str) -> Result<()> {
        let io = self
            .child
            .as_mut()
            .ok_or_else(|| Error::Other("acp: no child process".into()))?;
        let stdin = io
            .stdin
            .as_mut()
            .ok_or_else(|| Error::Other("acp: stdin not retained".into()))?;
        stdin.write_all(line.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        Ok(())
    }

    fn acp_next_id(&mut self) -> u64 {
        let id = self.acp_rpc_id;
        self.acp_rpc_id = self.acp_rpc_id.saturating_add(1);
        id
    }

    /// initialize → authenticate → session/new → session/prompt.
    async fn acp_handshake_and_prompt(&mut self, prompt: &str) -> Result<()> {
        let init_id = self.acp_next_id();
        self.acp_write_line(&acp::request(
            init_id,
            "initialize",
            acp::initialize_params("automedon"),
        ))
        .await?;
        self.acp_wait_rpc(init_id).await?;

        // Optional authenticate. Default is skip: many agents (OpenCode, etc.)
        // do not accept Grok's `cached_token` and injecting a soft-failed Error
        // event would poison later waits. Set extra.acp_auth to force a method.
        let auth_method = self
            .opts
            .extra
            .get("acp_auth")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty() && *s != "none" && *s != "skip")
            .map(str::to_string);
        if let Some(method) = auth_method {
            let auth_id = self.acp_next_id();
            self.acp_write_line(&acp::request(
                auth_id,
                "authenticate",
                acp::authenticate_params(&method),
            ))
            .await?;
            // Soft: do not fail the session if auth RPC errors.
            let _ = self.acp_wait_rpc_soft(auth_id).await;
        }

        let new_id = self.acp_next_id();
        // OpenCode and others require a string cwd on session/new.
        let cwd = self
            .opts
            .cwd
            .as_ref()
            .map(|p| p.display().to_string())
            .or_else(|| {
                std::env::current_dir()
                    .ok()
                    .map(|p| p.display().to_string())
            })
            .unwrap_or_else(|| ".".into());
        self.acp_write_line(&acp::request(
            new_id,
            "session/new",
            acp::session_new_params(Some(cwd.as_str())),
        ))
        .await?;
        self.acp_wait_rpc(new_id).await?;
        if self.acp_session_id.is_none() {
            if let Some(sid) = self.transcript.session_id() {
                self.acp_session_id = Some(sid.to_string());
            }
        }
        if self.acp_session_id.is_none() {
            return Err(Error::Other(
                "acp: session/new did not return sessionId".into(),
            ));
        }
        self.acp_send_prompt(prompt).await
    }

    async fn acp_send_prompt(&mut self, prompt: &str) -> Result<()> {
        let sid = self
            .acp_session_id
            .clone()
            .ok_or_else(|| Error::Other("acp: missing session id".into()))?;
        let id = self.acp_next_id();
        self.acp_pending_prompt_id = Some(id);
        self.acp_write_line(&acp::request(
            id,
            "session/prompt",
            acp::session_prompt_params(&sid, prompt),
        ))
        .await
    }

    /// Read stdout until a JSON-RPC response for `id` arrives; apply all events.
    async fn acp_wait_rpc(&mut self, id: u64) -> Result<()> {
        self.acp_wait_rpc_inner(id, false).await
    }

    /// Like [`Self::acp_wait_rpc`] but RPC errors do not fail and Error events are not applied.
    async fn acp_wait_rpc_soft(&mut self, id: u64) -> Result<()> {
        self.acp_wait_rpc_inner(id, true).await
    }

    async fn acp_wait_rpc_inner(&mut self, id: u64, soft: bool) -> Result<()> {
        let deadline = Instant::now() + self.default_timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(Error::Other(format!(
                    "acp: timeout waiting for rpc id={id}"
                )));
            }
            let line = {
                let io = self
                    .child
                    .as_mut()
                    .ok_or_else(|| Error::Other("acp: child gone".into()))?;
                match timeout(remaining, io.lines_rx.recv()).await {
                    Ok(Some(l)) => l,
                    Ok(None) => {
                        return Err(Error::Other("acp: stdout closed during handshake".into()))
                    }
                    Err(_) => {
                        return Err(Error::Other(format!(
                            "acp: timeout waiting for rpc id={id}"
                        )))
                    }
                }
            };
            let events = if self.acp_mode {
                acp::parse_line(&line)
            } else {
                self.adapter.parse_line(&line)
            };
            let mut matched = false;
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                matched = acp::is_response_for(&v, id);
                if matched {
                    if let Some(err) = v.get("error") {
                        let msg = err
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("rpc error");
                        if soft {
                            return Ok(());
                        }
                        if !msg.is_empty() {
                            return Err(Error::Other(format!("acp rpc {id}: {msg}")));
                        }
                    }
                }
            }
            if events.is_empty() {
                if matched {
                    return Ok(());
                }
                // still apply raw for debugging visibility
                self.apply_event(Event::Raw {
                    channel: "acp".into(),
                    line,
                });
            } else {
                for ev in events {
                    // Soft RPC: never stash Error into the transcript.
                    if soft && matches!(ev, Event::Error { .. }) {
                        if matched {
                            return Ok(());
                        }
                        continue;
                    }
                    if let Event::SessionInfo { id: ref sid, .. } = ev {
                        self.acp_session_id = Some(sid.clone());
                    }
                    self.apply_event(ev);
                }
            }
            if matched {
                return Ok(());
            }
        }
    }

    /// Wait until the current turn completes (`TurnComplete` or session `Done`).
    pub async fn await_turn(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }
        if !self.turn_active && self.child.is_none() && self.synthetic_queue.is_empty() {
            return Ok(());
        }
        loop {
            if self.closed {
                self.turn_active = false;
                return Ok(());
            }
            match self.next_event().await {
                Ok(ev) => {
                    if matches!(ev, Event::TurnComplete { .. } | Event::Done { .. }) {
                        self.turn_active = false;
                        // Keep expect_cursor consistent for follow-up expects.
                        self.expect_cursor = self.transcript.events().len();
                        return Ok(());
                    }
                    // Paused on permission/plan without turn complete.
                    if matches!(
                        ev,
                        Event::PermissionRequest { .. } | Event::PlanPresented { .. }
                    ) && self.synthetic_queue.is_empty()
                        && self.child.is_none()
                    {
                        return Ok(());
                    }
                }
                Err(Error::SessionFinished) => {
                    self.turn_active = false;
                    return Ok(());
                }
                Err(Error::Other(msg)) if msg.contains("turn paused") => {
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Wait until `expect` matches a new event (from expect_cursor).
    pub async fn expect(&mut self, mut exp: Expect) -> Result<Event> {
        if exp.timeout == Duration::from_secs(120) && self.default_timeout != exp.timeout {
            exp.timeout = self.default_timeout;
        }
        self.wait(Wait::on(exp)).await
    }

    /// Wait for a harness stream condition (hooks, tools, text, …).
    ///
    /// Same cursor semantics as [`Self::expect`]: only events at or after
    /// `expect_cursor` count, so multi-turn waits do not re-match prior turns.
    ///
    /// After each stream line is applied, the full transcript from the cursor
    /// is re-scanned. Adapters may emit several events per line (e.g. Pi
    /// `tool_execution_start` → HookStarted + ToolCall); only the last is
    /// returned from [`Self::next_event`], so matching only that last event
    /// would skip HookStarted / earlier siblings.
    pub async fn wait(&mut self, mut wait: Wait) -> Result<Event> {
        check_wait(&wait)?;
        if wait_needs_tools(&wait) {
            self.require_cap("stream_tools", self.caps.stream_tools)?;
        }
        if wait_needs_hooks(&wait) {
            self.require_cap(
                "wait_hooks",
                self.caps.wait_hooks || self.caps.hooks || self.caps.in_process,
            )?;
        }
        if wait.timeout == Duration::from_secs(120) && self.default_timeout != wait.timeout {
            wait.timeout = self.default_timeout;
        }

        let since = self.expect_cursor;
        let deadline = Instant::now() + wait.timeout;

        loop {
            // Scan buffered events (includes multi-event siblings applied via
            // take_last_event, not only the event returned by next_event).
            for (i, te) in self
                .transcript
                .events()
                .iter()
                .enumerate()
                .skip(self.expect_cursor)
            {
                if wait.matches(&te.event, &self.transcript, since) {
                    self.expect_cursor = i + 1;
                    return Ok(te.event.clone());
                }
                // Fail closed on harness Error so auth/stream failures do not hang.
                if matches!(&te.event, Event::Error { .. })
                    && !wait.matches(&te.event, &self.transcript, since)
                {
                    if let Event::Error { message } = &te.event {
                        return Err(Error::ExpectFailed(format!(
                            "harness error while waiting for {wait}: {message}"
                        )));
                    }
                }
            }

            if self.closed && self.synthetic_queue.is_empty() && self.child.is_none() {
                return Err(Error::ExpectTimeout {
                    timeout: wait.timeout,
                    predicate: wait.to_string(),
                });
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(Error::ExpectTimeout {
                    timeout: wait.timeout,
                    predicate: wait.to_string(),
                });
            }

            match timeout(remaining, self.next_event()).await {
                Err(_) => {
                    return Err(Error::ExpectTimeout {
                        timeout: wait.timeout,
                        predicate: wait.to_string(),
                    });
                }
                Ok(Err(Error::SessionFinished)) => {
                    // Final scan already ran at loop top; nothing left to read.
                    return Err(Error::ExpectTimeout {
                        timeout: wait.timeout,
                        predicate: wait.to_string(),
                    });
                }
                Ok(Err(e)) => return Err(e),
                Ok(Ok(event)) => {
                    if matches!(&event, Event::Error { .. })
                        && !wait.matches(&event, &self.transcript, since)
                    {
                        if let Event::Error { message } = &event {
                            return Err(Error::ExpectFailed(format!(
                                "harness error while waiting for {wait}: {message}"
                            )));
                        }
                    }
                    // Re-scan at top of loop so side-applied siblings match.
                    continue;
                }
            }
        }
    }

    /// Alias for [`Self::wait`].
    pub async fn wait_for(&mut self, wait: Wait) -> Result<Event> {
        self.wait(wait).await
    }

    /// Approve a pending permission request (mock or stdin-capable adapters).
    pub async fn approve(&mut self) -> Result<()> {
        self.require_cap(
            "permissions_interactive",
            self.caps.permissions_interactive || self.caps.in_process,
        )?;
        self.resolve_permission(true).await
    }

    /// Deny a pending permission request.
    pub async fn deny(&mut self) -> Result<()> {
        self.require_cap(
            "permissions_interactive",
            self.caps.permissions_interactive || self.caps.in_process,
        )?;
        self.resolve_permission(false).await
    }

    /// Approve a pending plan.
    pub async fn approve_plan(&mut self) -> Result<()> {
        self.require_cap(
            "plan_mode",
            self.caps.plan_mode || self.caps.plans || self.caps.in_process,
        )?;
        self.resolve_plan(true).await
    }

    /// Reject a pending plan.
    pub async fn reject_plan(&mut self) -> Result<()> {
        self.require_cap(
            "plan_mode",
            self.caps.plan_mode || self.caps.plans || self.caps.in_process,
        )?;
        self.resolve_plan(false).await
    }

    async fn resolve_permission(&mut self, allowed: bool) -> Result<()> {
        let id = self
            .pending_permission
            .clone()
            .or_else(|| {
                self.transcript
                    .permissions()
                    .iter()
                    .rev()
                    .find(|p| p.allowed.is_none())
                    .map(|p| p.id.clone())
            })
            .ok_or_else(|| Error::Other("no pending permission".into()))?;

        if let Some(line) = self.adapter.encode_permission(&id, allowed) {
            if let Some(io) = self.child.as_mut() {
                if let Some(stdin) = io.stdin.as_mut() {
                    stdin.write_all(line.as_bytes()).await?;
                    stdin.write_all(b"\n").await?;
                    stdin.flush().await?;
                }
            }
        }

        // Mock in-process: inject continuation events.
        if self.adapter.name() == "mock" {
            let prompt = self.last_prompt.clone().unwrap_or_default();
            let cont = mock_permission_continue(&prompt, allowed);
            self.pending_permission = None;
            self.enqueue_synthetic(cont);
            self.turn_active = true;
        } else {
            self.push(Event::PermissionResolved { id, allowed });
            self.pending_permission = None;
        }
        Ok(())
    }

    async fn resolve_plan(&mut self, approved: bool) -> Result<()> {
        let id = self
            .pending_plan
            .clone()
            .or_else(|| {
                self.transcript
                    .plans()
                    .iter()
                    .rev()
                    .find(|p| p.approved.is_none())
                    .map(|p| p.id.clone())
            })
            .ok_or_else(|| Error::Other("no pending plan".into()))?;

        if let Some(line) = self.adapter.encode_plan_resolve(&id, approved) {
            if let Some(io) = self.child.as_mut() {
                if let Some(stdin) = io.stdin.as_mut() {
                    stdin.write_all(line.as_bytes()).await?;
                    stdin.write_all(b"\n").await?;
                    stdin.flush().await?;
                }
            }
        }

        if self.adapter.name() == "mock" {
            let prompt = self.last_prompt.clone().unwrap_or_default();
            let cont = mock_plan_continue(&prompt, approved);
            self.pending_plan = None;
            self.enqueue_synthetic(cont);
            self.turn_active = true;
        } else {
            self.push(Event::PlanResolved { id, approved });
            self.pending_plan = None;
        }
        Ok(())
    }

    /// Drain until session Done (or current process ends for one-shot).
    pub async fn drain_until_done(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }
        while !self.closed {
            if !self.turn_active && self.child.is_none() && self.synthetic_queue.is_empty() {
                // Multi-turn idle: no more events without another prompt.
                if self.multi_turn {
                    break;
                }
                self.push(Event::Done { code: Some(0) });
                self.closed = true;
                break;
            }
            match self.next_event().await {
                Ok(_) => {}
                Err(Error::SessionFinished) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        if let Some(mut io) = self.child.take() {
            let _ = io.child.kill().await;
            let _ = io.child.wait().await;
        }
        self.synthetic_queue.clear();
        if !self.closed {
            self.push(Event::Done { code: Some(0) });
            self.closed = true;
        }
        self.turn_active = false;
        Ok(())
    }

    fn run_result(&self) -> RunResult {
        RunResult {
            text: self.transcript.text().to_string(),
            turn_text: self.transcript.turn_text().to_string(),
            thinking: self.transcript.thinking().to_string(),
            code: self.last_exit_code(),
            events: self.transcript.events().len(),
            session_id: self.transcript.session_id().map(str::to_string),
        }
    }

    fn enqueue_synthetic(&mut self, events: Vec<Event>) {
        self.synthetic_queue.extend(events);
    }

    /// Parse a harness line (stdout or stderr). Empty parse → `Raw` with `channel`.
    /// Harnesses like Copilot put resume footers on stderr; those must still go through
    /// the adapter so `SessionInfo` is extracted.
    fn events_from_line(&self, line: &str, raw_channel: &str) -> Vec<Event> {
        let parsed = if self.acp_mode {
            acp::parse_line(line)
        } else {
            self.adapter.parse_line(line)
        };
        if parsed.is_empty() {
            vec![Event::Raw {
                channel: raw_channel.into(),
                line: line.to_string(),
            }]
        } else {
            parsed
        }
    }

    /// Apply all but the last event; return the last (for the single-event next_event contract).
    /// Callers must pass a non-empty list (`events_from_line` always yields at least one).
    fn take_last_event(&mut self, mut events: Vec<Event>) -> Event {
        if events.len() == 1 {
            return events.pop().unwrap();
        }
        let last = events.pop().expect("take_last_event: non-empty");
        for ev in events {
            self.apply_event(ev);
        }
        last
    }

    async fn next_event(&mut self) -> Result<Event> {
        // Prefer synthetic queue (mock multi-turn / permission continue).
        if let Some(event) = self.synthetic_queue.first().cloned() {
            self.synthetic_queue.remove(0);
            self.apply_event(event.clone());
            return Ok(event);
        }

        if self.child.is_none() {
            if self.closed {
                return Err(Error::SessionFinished);
            }
            // Synthetic turn finished without Done.
            if !self.turn_active {
                return Err(Error::SessionFinished);
            }
            // No more events mid-turn (e.g. waiting for permission).
            return Err(Error::Other(
                "turn paused (awaiting approve/deny or idle)".into(),
            ));
        }

        loop {
            // Poll without holding a long-lived `child` borrow across `self` methods.
            enum Polled {
                Stdout(String),
                StdoutClosed,
                Stderr(String),
                StderrClosed,
            }
            let polled = {
                let io = self.child.as_mut().unwrap();
                tokio::select! {
                    line = io.lines_rx.recv() => match line {
                        Some(l) => Polled::Stdout(l),
                        None => Polled::StdoutClosed,
                    },
                    err_line = io.stderr_rx.recv() => match err_line {
                        Some(l) => Polled::Stderr(l),
                        None => Polled::StderrClosed,
                    },
                }
            };

            let event = match polled {
                Polled::StderrClosed => continue,
                Polled::Stdout(line) => {
                    // ACP: parse JSON-RPC + session/update; mark turn done on prompt result.
                    let mut parsed = if self.acp_mode {
                        acp::parse_line(&line)
                    } else {
                        self.adapter.parse_line(&line)
                    };
                    if self.acp_mode {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                            if let Some(pid) = self.acp_pending_prompt_id {
                                if acp::is_response_for(&v, pid) {
                                    self.acp_pending_prompt_id = None;
                                    if !parsed
                                        .iter()
                                        .any(|e| matches!(e, Event::TurnComplete { .. }))
                                    {
                                        parsed.push(Event::TurnComplete {
                                            turn: self.turn,
                                            stop_reason: v
                                                .get("result")
                                                .and_then(|r| r.get("stopReason"))
                                                .and_then(|s| s.as_str())
                                                .map(str::to_string),
                                        });
                                    }
                                }
                            }
                            if let Some(sid) = v
                                .get("result")
                                .and_then(|r| r.get("sessionId"))
                                .and_then(|s| s.as_str())
                            {
                                self.acp_session_id = Some(sid.to_string());
                            }
                        }
                    }
                    if parsed.is_empty() {
                        Event::Raw {
                            channel: if self.acp_mode { "acp" } else { "stdout" }.into(),
                            line,
                        }
                    } else {
                        self.take_last_event(parsed)
                    }
                }
                Polled::Stderr(line) => {
                    let events = self.events_from_line(&line, "stderr");
                    self.take_last_event(events)
                }
                Polled::StdoutClosed => {
                    // Wait for the process to exit so footers written after stdout closes
                    // (Copilot Resume on stderr) are fully produced, then drain stderr until
                    // the reader task closes the channel (not a fixed idle timeout).
                    let code = if let Some(io) = self.child.as_mut() {
                        io.child.wait().await?.code()
                    } else {
                        None
                    };
                    let mut residual_lines = Vec::new();
                    if let Some(io) = self.child.as_mut() {
                        while let Ok(err_line) = io.stderr_rx.try_recv() {
                            residual_lines.push(err_line);
                        }
                        // Bound total wait so a stuck stderr task cannot hang the session forever.
                        let deadline = Instant::now() + Duration::from_secs(5);
                        loop {
                            let remaining = deadline.saturating_duration_since(Instant::now());
                            if remaining.is_zero() {
                                break;
                            }
                            match timeout(remaining, io.stderr_rx.recv()).await {
                                Ok(Some(line)) => residual_lines.push(line),
                                Ok(None) => break, // channel closed
                                Err(_) => break,   // overall deadline
                            }
                        }
                    }
                    let mut residual: Vec<Event> = Vec::new();
                    for err_line in residual_lines {
                        residual.extend(self.events_from_line(&err_line, "stderr"));
                    }
                    if residual.is_empty() {
                        Event::ProcessExit { code }
                    } else {
                        residual.push(Event::ProcessExit { code });
                        let first = residual.remove(0);
                        self.synthetic_queue.extend(residual);
                        first
                    }
                }
            };

            self.apply_event(event.clone());

            if matches!(event, Event::ProcessExit { .. }) {
                if let Some(mut io) = self.child.take() {
                    match timeout(Duration::from_secs(2), io.child.wait()).await {
                        Ok(Ok(_)) => {}
                        _ => {
                            let _ = io.child.kill().await;
                            let _ = io.child.wait().await;
                        }
                    }
                }
                // One-shot / non multi-turn: process exit closes session.
                if !self.multi_turn {
                    if !self.closed {
                        self.push(Event::Done {
                            code: match &event {
                                Event::ProcessExit { code } => *code,
                                _ => None,
                            },
                        });
                        self.closed = true;
                    }
                    self.turn_active = false;
                } else {
                    // Multi-turn: turn ends on process exit if no TurnComplete yet.
                    self.turn_active = false;
                }
            }

            if matches!(event, Event::Done { .. }) {
                self.closed = true;
                self.turn_active = false;
                if let Some(mut io) = self.child.take() {
                    let _ = timeout(Duration::from_secs(2), io.child.wait()).await;
                }
            }

            return Ok(event);
        }
    }

    fn apply_event(&mut self, event: Event) {
        match &event {
            Event::PermissionRequest { id, .. } => {
                self.pending_permission = Some(id.clone());
            }
            Event::PlanPresented { id, .. } => {
                self.pending_plan = Some(id.clone());
            }
            Event::PermissionResolved { .. } => {
                self.pending_permission = None;
            }
            Event::PlanResolved { .. } => {
                self.pending_plan = None;
            }
            Event::Done { .. } => {
                self.closed = true;
                self.turn_active = false;
            }
            Event::TurnComplete { .. } => {
                self.turn_active = false;
            }
            _ => {}
        }
        // Promote non-SessionInfo events that still carry a session id.
        if !matches!(event, Event::SessionInfo { .. }) {
            if let Some(id) = self.adapter.session_id_from_event(&event) {
                if self.transcript.session_id().is_none() {
                    self.transcript.push(Event::SessionInfo { id, label: None });
                }
            }
        }
        self.push(event);
    }

    fn push(&mut self, event: Event) {
        self.transcript.push(event);
    }

    fn last_exit_code(&self) -> Option<i32> {
        self.transcript
            .events()
            .iter()
            .rev()
            .find_map(|te| match &te.event {
                Event::Done { code } | Event::ProcessExit { code } => *code,
                _ => None,
            })
    }
}

pub struct SessionBuilder {
    harness: String,
    opts: LaunchOptions,
}

impl SessionBuilder {
    pub fn opts(mut self, opts: LaunchOptions) -> Self {
        self.opts = opts;
        self
    }

    pub fn cwd(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.opts.cwd = Some(path.into());
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.opts.model = Some(model.into());
        self
    }

    pub fn yolo(mut self, yolo: bool) -> Self {
        self.opts.yolo = yolo;
        self
    }

    pub fn bin(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.opts.bin = Some(path.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.opts.default_timeout = Some(timeout);
        self
    }

    pub fn extra(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.opts.extra.insert(key.into(), value);
        self
    }

    pub fn build(self) -> Result<Session> {
        let adapter = resolve(&self.harness)?;
        Ok(Session::from_adapter(adapter, self.opts))
    }

    pub async fn run(self, prompt: impl AsRef<str>) -> Result<(Session, RunResult)> {
        let mut session = self.build()?;
        let result = session.run(prompt).await?;
        Ok((session, result))
    }
}

pub async fn run(
    harness: impl AsRef<str>,
    prompt: impl AsRef<str>,
    opts: LaunchOptions,
) -> Result<RunResult> {
    let mut session = Session::builder(harness).opts(opts).build()?;
    session.run(prompt).await
}
