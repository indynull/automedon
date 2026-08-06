//! Generic process adapter — any argv; lines become Raw or best-effort JSON events.

use serde_json::Value;

use super::{base_env, Adapter, Capabilities, PreparedLaunch, TurnContext};
use crate::config::LaunchOptions;
use crate::error::{Error, Result};
use crate::event::Event;
use crate::transport::SpawnSpec;

#[derive(Debug, Default, Clone)]
pub struct GenericAdapter;

impl Adapter for GenericAdapter {
    fn name(&self) -> &'static str {
        "generic"
    }

    fn capabilities(&self) -> Capabilities {
        // Escape hatch only — not a product harness.
        Capabilities::default()
    }

    fn prepare(
        &self,
        prompt: &str,
        opts: &LaunchOptions,
        _ctx: &TurnContext,
    ) -> Result<PreparedLaunch> {
        let program = opts
            .bin
            .clone()
            .ok_or_else(|| Error::Other("generic adapter requires opts.bin".into()))?;

        let mut args: Vec<String> = opts
            .extra
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();

        let append_prompt = opts
            .extra
            .get("append_prompt")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        if append_prompt {
            args.push(prompt.to_string());
        }

        Ok(PreparedLaunch {
            harness: "generic".into(),
            spawn: Some(SpawnSpec {
                program,
                args,
                cwd: opts.cwd.clone(),
                env: base_env(opts),
                retain_stdin: opts
                    .extra
                    .get("retain_stdin")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            }),
            synthetic: None,
            capabilities: self.capabilities(),
            multi_turn: false,
        })
    }

    fn parse_line(&self, line: &str) -> Vec<Event> {
        let line = line.trim();
        if line.is_empty() {
            return Vec::new();
        }
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            // Best-effort: if it looks like our normalized shape or has type/text.
            if let Some(ty) = v.get("type").and_then(|t| t.as_str()) {
                match ty {
                    "text" => {
                        if let Some(t) = v.get("data").and_then(|d| d.as_str()) {
                            return vec![Event::TextDelta {
                                text: t.to_string(),
                            }];
                        }
                    }
                    "end" | "done" => return vec![Event::Done { code: Some(0) }],
                    _ => {}
                }
            }
        }
        vec![Event::Raw {
            channel: "stdout".into(),
            line: line.to_string(),
        }]
    }
}
