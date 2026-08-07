//! In-process mock harness — multi-turn, permissions, plan, and goal scenarios.

use serde_json::json;

use super::{Adapter, Capabilities, PreparedLaunch, TurnContext};
use crate::config::LaunchOptions;
use crate::error::Result;
use crate::event::Event;

#[derive(Debug, Default, Clone)]
pub struct MockAdapter;

impl Adapter for MockAdapter {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            launch: true,
            multi_turn: true,
            stream_tools: true,
            wait_hooks: true,
            permissions_preflight: true,
            permissions_interactive: true,
            plan_mode: true,
            goals: true,
            sessions: true,
            streaming_json: true,
            yolo: true,
            permissions: true,
            plans: true,
            hooks: true,
            in_process: true,
            ..Default::default()
        }
    }

    fn prepare(
        &self,
        prompt: &str,
        opts: &LaunchOptions,
        ctx: &TurnContext,
    ) -> Result<PreparedLaunch> {
        let scenario = opts
            .extra
            .get("scenario")
            .and_then(|v| v.as_str())
            .unwrap_or("echo");

        let events = match scenario {
            "tools" => scenario_tools(prompt, ctx),
            "error" => scenario_error(prompt, ctx),
            "think" => scenario_think(prompt, ctx),
            "multi" => scenario_multi(prompt, ctx),
            "permission" => scenario_permission(prompt, ctx, opts),
            "plan" => scenario_plan(prompt, ctx, opts),
            "goal" => scenario_goal(prompt, ctx),
            "hooks" => scenario_hooks(prompt, ctx),
            _ => scenario_echo(prompt, ctx),
        };

        Ok(PreparedLaunch {
            harness: "mock".into(),
            spawn: None,
            synthetic: Some(events),
            capabilities: self.capabilities(),
            multi_turn: true,
        })
    }

    fn parse_line(&self, _line: &str) -> Vec<Event> {
        Vec::new()
    }

    fn encode_permission(&self, id: &str, allowed: bool) -> Option<String> {
        Some(format!(
            "{{\"type\":\"permission_response\",\"id\":\"{id}\",\"allowed\":{allowed}}}"
        ))
    }

    fn encode_plan_resolve(&self, id: &str, approved: bool) -> Option<String> {
        Some(format!(
            "{{\"type\":\"plan_response\",\"id\":\"{id}\",\"approved\":{approved}}}"
        ))
    }
}

fn session_meta(ctx: &TurnContext) -> Event {
    Event::SessionInfo {
        id: ctx
            .session_id
            .clone()
            .unwrap_or_else(|| "mock-session-1".into()),
        label: Some("mock".into()),
    }
}

fn turn_header(ctx: &TurnContext) -> Vec<Event> {
    let mut v = Vec::new();
    if ctx.turn <= 1 {
        v.push(Event::Spawned {
            pid: 0,
            harness: "mock".into(),
        });
        v.push(session_meta(ctx));
    }
    v.push(Event::TurnStart {
        turn: ctx.turn.max(1),
    });
    v
}

fn turn_footer(ctx: &TurnContext, end_session: bool) -> Vec<Event> {
    let mut v = vec![Event::TurnComplete {
        turn: ctx.turn.max(1),
        stop_reason: Some("end_turn".into()),
    }];
    if end_session {
        // In-process mock has no child process; still emit ProcessExit so
        // expect(process_exit()) / operators can exercise that constructor offline.
        v.push(Event::ProcessExit { code: Some(0) });
        v.push(Event::Done { code: Some(0) });
    }
    v
}

fn scenario_echo(prompt: &str, ctx: &TurnContext) -> Vec<Event> {
    let mut events = turn_header(ctx);
    events.push(Event::ThinkingDelta {
        text: format!("echoing: {prompt}"),
    });
    events.push(Event::TextDelta {
        text: format!("ECHO:{prompt}"),
    });
    // One-shot echo scenarios end the session; multi keeps it open.
    events.extend(turn_footer(ctx, true));
    events
}

fn scenario_multi(prompt: &str, ctx: &TurnContext) -> Vec<Event> {
    let mut events = turn_header(ctx);
    let turn = ctx.turn.max(1);
    if turn == 1 {
        events.push(Event::TextDelta {
            text: format!("T1:{prompt}"),
        });
        events.extend(turn_footer(ctx, false));
    } else {
        // Continuity: second turn must see first-turn history text.
        let prior = if ctx.history_text.is_empty() {
            "(none)".to_string()
        } else {
            ctx.history_text.clone()
        };
        events.push(Event::TextDelta {
            text: format!("T{turn}:{prompt}|prior={prior}"),
        });
        events.extend(turn_footer(ctx, false));
    }
    events
}

fn scenario_permission(prompt: &str, ctx: &TurnContext, opts: &LaunchOptions) -> Vec<Event> {
    let mut events = turn_header(ctx);
    let auto = opts.yolo
        || opts
            .extra
            .get("auto_approve")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

    // If a pending permission was resolved via session.approve/deny before this prepare,
    // TurnContext won't have it — Session injects PermissionResolved synthetically.
    // Here we emit the request on first prepare of the turn, then if yolo, resolve immediately.
    events.push(Event::PermissionRequest {
        id: "perm_1".into(),
        tool: "run_terminal_command".into(),
        detail: format!("bash: echo {prompt}"),
    });
    if auto {
        events.push(Event::PermissionResolved {
            id: "perm_1".into(),
            allowed: true,
        });
        events.push(Event::ToolCall {
            id: "call_perm".into(),
            name: "run_terminal_command".into(),
            input: json!({ "command": format!("echo {prompt}") }),
        });
        events.push(Event::ToolResult {
            id: "call_perm".into(),
            name: "run_terminal_command".into(),
            output: format!("{prompt}\n"),
            is_error: false,
        });
        events.push(Event::TextDelta {
            text: format!("ran:{prompt}"),
        });
        events.extend(turn_footer(ctx, true));
    } else {
        // Pause: session must call approve/deny; mock_continue fills the rest.
        // Marker event only — no turn complete yet.
    }
    events
}

/// Finish a permission turn after approve/deny (called from session).
pub fn mock_permission_continue(prompt: &str, allowed: bool) -> Vec<Event> {
    let mut events = vec![Event::PermissionResolved {
        id: "perm_1".into(),
        allowed,
    }];
    if allowed {
        events.push(Event::ToolCall {
            id: "call_perm".into(),
            name: "run_terminal_command".into(),
            input: json!({ "command": format!("echo {prompt}") }),
        });
        events.push(Event::ToolResult {
            id: "call_perm".into(),
            name: "run_terminal_command".into(),
            output: format!("{prompt}\n"),
            is_error: false,
        });
        events.push(Event::TextDelta {
            text: format!("ran:{prompt}"),
        });
    } else {
        events.push(Event::TextDelta {
            text: "denied".into(),
        });
    }
    events.push(Event::TurnComplete {
        turn: 1,
        stop_reason: Some("end_turn".into()),
    });
    events.push(Event::Done { code: Some(0) });
    events
}

fn scenario_plan(prompt: &str, ctx: &TurnContext, opts: &LaunchOptions) -> Vec<Event> {
    let mut events = turn_header(ctx);
    let auto = opts.yolo
        || opts
            .extra
            .get("auto_approve")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

    events.push(Event::PlanModeEnter {
        reason: Some("user_request".into()),
    });
    events.push(Event::PlanPresented {
        id: "plan_1".into(),
        summary: format!("Plan for: {prompt}"),
    });
    if auto {
        events.push(Event::PlanResolved {
            id: "plan_1".into(),
            approved: true,
        });
        events.push(Event::PlanModeExit {
            reason: Some("approved".into()),
        });
        events.push(Event::TextDelta {
            text: format!("executing plan for {prompt}"),
        });
        events.extend(turn_footer(ctx, true));
    }
    events
}

pub fn mock_plan_continue(prompt: &str, approved: bool) -> Vec<Event> {
    let mut events = vec![Event::PlanResolved {
        id: "plan_1".into(),
        approved,
    }];
    events.push(Event::PlanModeExit {
        reason: Some(if approved { "approved" } else { "rejected" }.into()),
    });
    if approved {
        events.push(Event::TextDelta {
            text: format!("executing plan for {prompt}"),
        });
    } else {
        events.push(Event::TextDelta {
            text: "plan rejected".into(),
        });
    }
    events.push(Event::TurnComplete {
        turn: 1,
        stop_reason: Some("end_turn".into()),
    });
    events.push(Event::Done { code: Some(0) });
    events
}

fn scenario_goal(prompt: &str, ctx: &TurnContext) -> Vec<Event> {
    let mut events = turn_header(ctx);
    events.push(Event::GoalStarted {
        id: "goal_1".into(),
        title: prompt.to_string(),
    });
    events.push(Event::GoalProgress {
        id: "goal_1".into(),
        message: "working".into(),
        percent: Some(50.0),
    });
    events.push(Event::GoalProgress {
        id: "goal_1".into(),
        message: "finishing".into(),
        percent: Some(100.0),
    });
    events.push(Event::GoalCompleted {
        id: "goal_1".into(),
        success: true,
        message: Some(format!("done:{prompt}")),
    });
    events.push(Event::TextDelta {
        text: format!("goal_ok:{prompt}"),
    });
    events.extend(turn_footer(ctx, true));
    events
}

/// Hook lifecycle around a tool call — for `wait(hook(...))`.
fn scenario_hooks(prompt: &str, ctx: &TurnContext) -> Vec<Event> {
    let mut events = turn_header(ctx);
    events.push(Event::HookStarted {
        id: "hook_pre_1".into(),
        name: "PreToolUse".into(),
        phase: Some("pre".into()),
        detail: Some(json!({ "tool": "run_terminal_command" })),
    });
    events.push(Event::ToolCall {
        id: "call_hook".into(),
        name: "run_terminal_command".into(),
        input: json!({ "command": format!("echo {prompt}") }),
    });
    events.push(Event::HookFinished {
        id: "hook_pre_1".into(),
        name: "PreToolUse".into(),
        phase: Some("pre".into()),
        ok: true,
        detail: None,
    });
    events.push(Event::ToolResult {
        id: "call_hook".into(),
        name: "run_terminal_command".into(),
        output: format!("{prompt}\n"),
        is_error: false,
    });
    events.push(Event::HookStarted {
        id: "hook_post_1".into(),
        name: "PostToolUse".into(),
        phase: Some("post".into()),
        detail: Some(json!({ "tool": "run_terminal_command" })),
    });
    events.push(Event::HookFinished {
        id: "hook_post_1".into(),
        name: "PostToolUse".into(),
        phase: Some("post".into()),
        ok: true,
        detail: Some("ok".into()),
    });
    events.push(Event::TextDelta {
        text: format!("hooks_done:{prompt}"),
    });
    events.extend(turn_footer(ctx, true));
    events
}

fn scenario_tools(prompt: &str, ctx: &TurnContext) -> Vec<Event> {
    let mut events = turn_header(ctx);
    events.push(Event::ToolCall {
        id: "call_1".into(),
        name: "list_dir".into(),
        input: json!({ "path": "." }),
    });
    events.push(Event::ToolResult {
        id: "call_1".into(),
        name: "list_dir".into(),
        output: "src/\nCargo.toml\n".into(),
        is_error: false,
    });
    events.push(Event::ToolCall {
        id: "call_2".into(),
        name: "read_file".into(),
        input: json!({ "path": "Cargo.toml" }),
    });
    events.push(Event::ToolResult {
        id: "call_2".into(),
        name: "read_file".into(),
        output: "[workspace]\n".into(),
        is_error: false,
    });
    events.push(Event::TextDelta {
        text: format!("listed files for: {prompt}"),
    });
    events.extend(turn_footer(ctx, true));
    events
}

fn scenario_error(prompt: &str, ctx: &TurnContext) -> Vec<Event> {
    let mut events = turn_header(ctx);
    events.push(Event::Error {
        message: format!("mock error for prompt: {prompt}"),
    });
    events.push(Event::Done { code: Some(1) });
    events
}

fn scenario_think(prompt: &str, ctx: &TurnContext) -> Vec<Event> {
    let mut events = turn_header(ctx);
    for word in ["I ", "am ", "thinking ", "about ", "this."] {
        events.push(Event::ThinkingDelta {
            text: word.to_string(),
        });
    }
    events.push(Event::TextDelta {
        text: format!("done:{prompt}"),
    });
    events.extend(turn_footer(ctx, true));
    events
}
