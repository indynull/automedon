use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::mpsc;

use crate::error::{Error, Result};

/// How to spawn a harness subprocess.
#[derive(Debug, Clone)]
pub struct SpawnSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: BTreeMap<String, String>,
    /// Keep stdin open for interactive / multi-turn. One-shot headless should leave this false
    /// so the child sees EOF immediately (Node CLIs often hang on an open pipe).
    pub retain_stdin: bool,
}

impl Default for SpawnSpec {
    fn default() -> Self {
        Self {
            program: PathBuf::new(),
            args: Vec::new(),
            cwd: None,
            env: BTreeMap::new(),
            retain_stdin: false,
        }
    }
}

/// Live child handles used by a session.
pub struct ChildIo {
    pub child: Child,
    pub stdin: Option<ChildStdin>,
    pub lines_rx: mpsc::Receiver<String>,
    pub stderr_rx: mpsc::Receiver<String>,
}

/// Spawn `spec`, fan out stdout/stderr lines on channels (bounded for backpressure).
pub async fn spawn_process(spec: SpawnSpec) -> Result<ChildIo> {
    let mut cmd = Command::new(&spec.program);
    cmd.args(&spec.args)
        .stdin(if spec.retain_stdin {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    if let Some(cwd) = &spec.cwd {
        cmd.current_dir(cwd);
    }
    for (k, v) in &spec.env {
        cmd.env(k, v);
    }

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::HarnessNotFound(spec.program.display().to_string())
        } else {
            Error::Io(e)
        }
    })?;

    // piped above — take is infallible for these handles
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let stdin = if spec.retain_stdin {
        child.stdin.take()
    } else {
        None
    };

    let (lines_tx, lines_rx) = mpsc::channel::<String>(4096);
    let (stderr_tx, stderr_rx) = mpsc::channel::<String>(1024);

    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if lines_tx.send(line).await.is_err() {
                break;
            }
        }
    });

    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if stderr_tx.send(line).await.is_err() {
                break;
            }
        }
    });

    Ok(ChildIo {
        child,
        stdin,
        lines_rx,
        stderr_rx,
    })
}
