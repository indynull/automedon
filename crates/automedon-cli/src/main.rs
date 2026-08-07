//! `medon` — CLI for Automedon harness automation scripts.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "medon",
    about = "Drive local AI coding harness CLIs (Grok, Pi, Claude, and others)",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a Rhai (`.rhai` / `.ail`) automation script.
    Run {
        /// Path to the script.
        script: PathBuf,
        /// Print the script return value.
        #[arg(long)]
        print: bool,
    },
    /// Evaluate a short inline Rhai snippet.
    Eval {
        /// Rhai source.
        source: String,
    },
    /// List product adapters, capabilities, binaries, and multi-turn mechanisms.
    Adapters,
    /// One-shot prompt against a harness (Rust path, no script file).
    Shot {
        /// Adapter id: claude, codex, gemini, opencode, grok, cursor, aider, pi, copilot, mock, generic.
        harness: String,
        /// Prompt text.
        prompt: String,
        /// Auto-approve tools where supported (maps to product yolo/allow-all flags).
        #[arg(long)]
        yolo: bool,
        /// Model id.
        #[arg(long)]
        model: Option<String>,
        /// Working directory for the child process.
        #[arg(long)]
        cwd: Option<PathBuf>,
        /// Default wait/expect timeout in milliseconds (product CLIs often need 60_000–180_000).
        #[arg(long)]
        timeout_ms: Option<u64>,
        /// Mock scenario (mock only): echo, multi, tools, hooks, permission, plan, goal, think, error.
        #[arg(long)]
        scenario: Option<String>,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(e) = try_main().await {
        eprintln!("medon: {e:#}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn yn(b: bool) -> &'static str {
    if b {
        "yes"
    } else {
        "—"
    }
}

async fn try_main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Adapters => {
            use automedon::adapter::AdapterKind;
            println!(
                "Product adapters (driver surface — still need product CLI + auth for live runs)\n"
            );
            println!(
                "{:<10} {:<28} {:<6} {:<6} {:<6} {:<5}  MULTI-TURN",
                "NAME", "BINARY", "LAUNCH", "MULTI", "TOOLS", "ACP"
            );
            for name in automedon::adapter::product_names() {
                let kind = AdapterKind::parse(name)?;
                let a = automedon::resolve(name)?;
                let c = a.capabilities();
                println!(
                    "{name:<10} {:<28} {:<6} {:<6} {:<6} {:<5}  {}",
                    kind.default_binaries(),
                    yn(c.launch),
                    yn(c.multi_turn),
                    yn(c.stream_tools),
                    yn(c.acp),
                    kind.multi_turn_summary(),
                );
            }
            println!();
            println!("Infrastructure:");
            for name in ["mock", "generic"] {
                let kind = AdapterKind::parse(name)?;
                let a = automedon::resolve(name)?;
                let c = a.capabilities();
                println!(
                    "  {name:<8} binary={}  in_process={}  launch={}",
                    kind.default_binaries(),
                    yn(c.in_process),
                    yn(c.launch)
                );
            }
            println!();
            println!("Examples:  medon run examples/mock/smoke.rhai --print");
            println!("           medon run examples/harnesses/<name>.rhai --print");
            println!("Docs:      handbook adapters + MATRIX.md");
        }
        Commands::Run { script, print } => {
            if !script.exists() {
                bail!("script not found: {}", script.display());
            }
            let result = automedon::dsl::run_script(&script)
                .with_context(|| format!("running {}", script.display()))?;
            if print {
                println!("{}", result.value);
            }
        }
        Commands::Eval { source } => {
            let result = automedon::dsl::eval_str(&source)?;
            if !result.value.is_unit() {
                println!("{}", result.value);
            }
        }
        Commands::Shot {
            harness,
            prompt,
            yolo,
            model,
            cwd,
            timeout_ms,
            scenario,
        } => {
            let mut opts = automedon::LaunchOptions {
                yolo,
                model,
                cwd,
                ..Default::default()
            };
            if let Some(ms) = timeout_ms {
                opts.default_timeout = Some(std::time::Duration::from_millis(ms));
            }
            if let Some(sc) = scenario {
                opts.extra.insert("scenario".into(), serde_json::json!(sc));
            }
            let result = automedon::run(&harness, &prompt, opts).await?;
            println!("{}", result.text);
            if let Some(code) = result.code {
                if code != 0 {
                    bail!("harness exited with code {code}");
                }
            }
        }
    }
    Ok(())
}
