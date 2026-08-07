//! Cursor agent CLI specialized driver (`agent` / `cursor-agent`).
//!
//! Headless: `-p` + `--output-format stream-json`. Multi-turn: `--resume` / `--continue`.

use std::path::PathBuf;

use super::{
    base_env, resolve_bin, shared_parse, Adapter, Capabilities, PreparedLaunch, TurnContext,
};
use crate::config::LaunchOptions;
use crate::error::Result;
use crate::event::Event;
use crate::transport::SpawnSpec;

#[derive(Debug, Default, Clone)]
pub struct CursorAdapter;

impl Adapter for CursorAdapter {
    fn name(&self) -> &'static str {
        "cursor"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            launch: true,
            multi_turn: true,
            stream_tools: true,
            sessions: true,
            streaming_json: true,
            yolo: true,
            permissions_preflight: true,
            permissions: false,
            permissions_interactive: false,
            ..Default::default()
        }
    }

    fn prepare(
        &self,
        prompt: &str,
        opts: &LaunchOptions,
        ctx: &TurnContext,
    ) -> Result<PreparedLaunch> {
        let (program, mut args) = resolve_cursor_bin(opts, prompt);

        args.push("--output-format".into());
        args.push("stream-json".into());
        if opts.yolo {
            // Cursor agent force/yolo-style flags (version-dependent; both common).
            if !args.iter().any(|a| a == "--force" || a == "--yolo") {
                args.push("--force".into());
            }
        }
        if ctx.turn > 1 {
            if let Some(id) = ctx.session_id.as_ref().filter(|s| !s.is_empty()) {
                args.push("--resume".into());
                args.push(id.clone());
            } else {
                args.push("--continue".into());
            }
        } else if let Some(id) = opts.extra.get("resume").and_then(|v| v.as_str()) {
            args.push("--resume".into());
            args.push(id.into());
        }
        if let Some(model) = &opts.model {
            args.push("--model".into());
            args.push(model.clone());
        }

        Ok(PreparedLaunch {
            harness: "cursor".into(),
            spawn: Some(SpawnSpec {
                program,
                args,
                cwd: opts.cwd.clone(),
                env: base_env(opts),
                retain_stdin: false,
            }),
            synthetic: None,
            capabilities: self.capabilities(),
            multi_turn: true,
        })
    }

    fn parse_line(&self, line: &str) -> Vec<Event> {
        let line = line.trim();
        if line.is_empty() {
            return Vec::new();
        }
        // Plain-text auth errors
        if line.contains("Authentication required") || line.contains("agent login") {
            return vec![Event::Error {
                message: line.to_string(),
            }];
        }
        match serde_json::from_str(line) {
            Ok(v) => shared_parse::parse_common_json(&v, "cursor"),
            Err(_) => vec![Event::Raw {
                channel: "cursor".into(),
                line: line.to_string(),
            }],
        }
    }
}

/// Prefer explicit `extra.binary`, then `cursor-agent`, then `agent`, then `cursor agent`.
fn resolve_cursor_bin(opts: &LaunchOptions, prompt: &str) -> (PathBuf, Vec<String>) {
    if let Some(bin) = opts.extra.get("binary").and_then(|v| v.as_str()) {
        let program = resolve_bin(opts, bin);
        let args = if bin == "cursor" {
            vec!["agent".into(), "-p".into(), prompt.to_string()]
        } else {
            vec!["-p".into(), prompt.to_string()]
        };
        return (program, args);
    }
    if opts.bin.is_some() {
        return (
            resolve_bin(opts, "agent"),
            vec!["-p".into(), prompt.to_string()],
        );
    }
    // PATH preference: agent → cursor-agent → cursor
    for name in ["agent", "cursor-agent"] {
        if which_on_path(name) {
            return (PathBuf::from(name), vec!["-p".into(), prompt.to_string()]);
        }
    }
    (
        PathBuf::from("cursor"),
        vec!["agent".into(), "-p".into(), prompt.to_string()],
    )
}

fn which_on_path(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| {
            std::env::split_paths(&p).any(|dir| {
                let c = dir.join(name);
                c.is_file()
            })
        })
        .unwrap_or(false)
}
