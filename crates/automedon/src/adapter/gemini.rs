//! Google Gemini CLI specialized driver (prefers `agy` / Antigravity when present).
//! Headless stream-json; multi-turn via `-r` / resume; optional ACP prepare.

use std::path::PathBuf;

use super::{
    base_env, resolve_bin, shared_parse, Adapter, Capabilities, PreparedLaunch, TurnContext,
};
use crate::config::LaunchOptions;
use crate::error::Result;
use crate::event::Event;
use crate::transport::SpawnSpec;

#[derive(Debug, Default, Clone)]
pub struct GeminiAdapter;

impl Adapter for GeminiAdapter {
    fn name(&self) -> &'static str {
        "gemini"
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
            acp: true,
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
        let program = resolve_gemini_bin(opts);
        let use_acp = opts
            .extra
            .get("acp")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if use_acp {
            return Ok(PreparedLaunch {
                harness: "gemini".into(),
                spawn: Some(SpawnSpec {
                    program,
                    args: vec!["--acp".into()],
                    cwd: opts.cwd.clone(),
                    env: base_env(opts),
                    retain_stdin: true,
                }),
                synthetic: None,
                capabilities: self.capabilities(),
                multi_turn: true,
            });
        }

        let mut args = vec![
            "-p".into(),
            prompt.to_string(),
            "-o".into(),
            "stream-json".into(),
        ];
        if opts.yolo || opts.extra.get("approval_mode").and_then(|v| v.as_str()) == Some("yolo") {
            args.push("-y".into());
        }
        if let Some(mode) = opts.extra.get("approval_mode").and_then(|v| v.as_str()) {
            if mode != "yolo" {
                args.push("--approval-mode".into());
                args.push(mode.into());
            }
        }
        if let Some(model) = &opts.model {
            args.push("-m".into());
            args.push(model.clone());
        }
        if ctx.turn > 1 {
            if let Some(id) = ctx.session_id.as_ref().filter(|s| !s.is_empty()) {
                args.push("-r".into());
                args.push(id.clone());
            } else {
                // resume latest session when no id known
                args.push("-r".into());
                args.push(
                    opts.extra
                        .get("resume")
                        .and_then(|v| v.as_str())
                        .unwrap_or("latest")
                        .into(),
                );
            }
        } else if let Some(id) = opts.extra.get("resume").and_then(|v| v.as_str()) {
            args.push("-r".into());
            args.push(id.into());
        }
        if opts
            .extra
            .get("worktree")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            args.push("-w".into());
        }
        if let Some(tools) = opts.extra.get("allowed_tools").and_then(|v| v.as_str()) {
            args.push("--allowed-tools".into());
            args.push(tools.into());
        }

        Ok(PreparedLaunch {
            harness: "gemini".into(),
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
        // Vendor kill-switch surfaces as multi-line stack; still mark Error.
        if line.contains("IneligibleTierError") || line.contains("no longer supported") {
            return vec![Event::Error {
                message: line.to_string(),
            }];
        }
        let json_line = if let Some(idx) = line.find('{') {
            &line[idx..]
        } else {
            line
        };
        match serde_json::from_str(json_line) {
            Ok(v) => shared_parse::parse_common_json(&v, "gemini"),
            Err(_) => vec![Event::Raw {
                channel: "gemini".into(),
                line: line.to_string(),
            }],
        }
    }
}

fn resolve_gemini_bin(opts: &LaunchOptions) -> PathBuf {
    if opts.bin.is_some() {
        return resolve_bin(opts, "gemini");
    }
    if let Some(bin) = opts.extra.get("binary").and_then(|v| v.as_str()) {
        return PathBuf::from(bin);
    }
    // Prefer Antigravity when installed (`agy`), else gemini CLI.
    if which_on_path("agy") {
        return PathBuf::from("agy");
    }
    PathBuf::from("gemini")
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
