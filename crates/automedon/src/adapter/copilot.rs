//! GitHub Copilot CLI specialized driver.

use super::{
    base_env, resolve_bin, shared_parse, Adapter, Capabilities, PreparedLaunch, TurnContext,
};
use crate::config::LaunchOptions;
use crate::error::Result;
use crate::event::Event;
use crate::transport::SpawnSpec;

#[derive(Debug, Default, Clone)]
pub struct CopilotAdapter;

impl Adapter for CopilotAdapter {
    fn name(&self) -> &'static str {
        "copilot"
    }

    fn capabilities(&self) -> Capabilities {
        // Multi-turn via --resume when SessionInfo is parsed from the Resume footer.
        // ACP prepare path available via extra.acp.
        Capabilities {
            launch: true,
            multi_turn: true,
            sessions: true,
            yolo: true,
            permissions_preflight: true,
            permissions: false,
            permissions_interactive: false,
            acp: true,
            ..Default::default()
        }
    }

    fn prepare(
        &self,
        prompt: &str,
        opts: &LaunchOptions,
        ctx: &TurnContext,
    ) -> Result<PreparedLaunch> {
        let program = resolve_bin(opts, "copilot");
        let use_acp = opts
            .extra
            .get("acp")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Preferred multi-turn path: ACP keeps one process across prompts.
        if use_acp {
            return Ok(PreparedLaunch {
                harness: "copilot".into(),
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

        // Non-interactive: copilot -p/--prompt. Resume when session id known.
        let mut args = vec!["-p".into(), prompt.to_string()];
        if opts.yolo {
            args.push("--allow-all-tools".into());
            args.push("--allow-all-paths".into());
        }
        if let Some(model) = &opts.model {
            args.push("--model".into());
            args.push(model.clone());
        }
        if ctx.turn > 1 {
            if let Some(id) = ctx.session_id.as_ref().filter(|s| !s.is_empty()) {
                args.push("--resume".into());
                args.push(id.clone());
            } else {
                args.push("--continue".into());
            }
        }

        Ok(PreparedLaunch {
            harness: "copilot".into(),
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
        // Footer: `Resume     copilot --resume=<uuid>` (plain text, not JSON).
        if let Some(id) = extract_resume_id(line) {
            return vec![
                Event::SessionInfo {
                    id,
                    label: Some("copilot".into()),
                },
                Event::TextDelta {
                    text: format!("{line}\n"),
                },
            ];
        }
        match serde_json::from_str(line) {
            Ok(v) => shared_parse::parse_common_json(&v, "copilot"),
            Err(_) => vec![Event::TextDelta {
                text: format!("{line}\n"),
            }],
        }
    }
}

/// Parse `Resume … --resume=<id>` or `--resume <id>` from Copilot CLI footer lines.
fn extract_resume_id(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    if !lower.contains("resume") {
        return None;
    }
    // Prefer `--resume=<id>`
    if let Some(idx) = line.find("--resume=") {
        let rest = &line[idx + "--resume=".len()..];
        let id = rest
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_matches(|c: char| c == '"' || c == '\'');
        if !id.is_empty() {
            return Some(id.to_string());
        }
    }
    // `--resume <id>`
    let mut parts = line.split_whitespace();
    while let Some(p) = parts.next() {
        if p == "--resume" {
            if let Some(id) = parts.next() {
                let id = id.trim_matches(|c: char| c == '"' || c == '\'');
                if !id.is_empty() {
                    return Some(id.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_resume_footer() {
        let id =
            extract_resume_id("Resume     copilot --resume=a15c9384-9de2-4eb1-88d7-fa86d83b4860")
                .unwrap();
        assert_eq!(id, "a15c9384-9de2-4eb1-88d7-fa86d83b4860");
    }
}
