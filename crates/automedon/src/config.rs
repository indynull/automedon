use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Launch options shared across adapters. Harness-specific knobs go in [`extra`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LaunchOptions {
    /// Working directory for the harness process.
    pub cwd: Option<PathBuf>,
    /// Binary override (default: adapter name on PATH).
    pub bin: Option<PathBuf>,
    /// Model id / pattern when the harness supports it.
    pub model: Option<String>,
    /// Auto-approve tools / yolo / always-approve.
    pub yolo: bool,
    /// Extra env vars.
    pub env: BTreeMap<String, String>,
    /// Default expect timeout.
    #[serde(default, with = "humantime_serde_opt")]
    pub default_timeout: Option<Duration>,
    /// Adapter-specific options (max_turns, output_format, tools, …).
    #[serde(default)]
    pub extra: BTreeMap<String, Value>,
}

impl LaunchOptions {
    pub fn cwd(mut self, path: impl Into<PathBuf>) -> Self {
        self.cwd = Some(path.into());
        self
    }

    pub fn bin(mut self, path: impl Into<PathBuf>) -> Self {
        self.bin = Some(path.into());
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn yolo(mut self, yolo: bool) -> Self {
        self.yolo = yolo;
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn extra(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = Some(timeout);
        self
    }

    pub fn default_timeout_or(&self, fallback: Duration) -> Duration {
        self.default_timeout.unwrap_or(fallback)
    }
}

/// Minimal serde helper so Duration can ride in LaunchOptions without a new dep name conflict.
mod humantime_serde_opt {
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(d) => serializer.serialize_some(&d.as_secs_f64()),
            None => serializer.serialize_none(), // covered when default_timeout is None
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Option::<f64>::deserialize(deserializer)?;
        Ok(v.map(Duration::from_secs_f64))
    }
}
