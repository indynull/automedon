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
    /// List built-in harness adapters.
    Adapters,
    /// One-shot prompt against a harness (Rust path, no script file).
    Shot {
        /// Harness name: grok, pi, claude, mock, generic.
        harness: String,
        /// Prompt text.
        prompt: String,
        /// Auto-approve tools where supported.
        #[arg(long)]
        yolo: bool,
        /// Model id.
        #[arg(long)]
        model: Option<String>,
        /// Working directory.
        #[arg(long)]
        cwd: Option<PathBuf>,
        /// Mock scenario (mock adapter only): echo | tools | think | error.
        #[arg(long)]
        scenario: Option<String>,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(e) = try_main().await {
        eprintln!("automedon: {e:#}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

async fn try_main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Adapters => {
            println!("# product harnesses (mock is test-only; generic is escape hatch)");
            for name in automedon::adapter::product_names() {
                let a = automedon::resolve(name)?;
                let c = a.capabilities();
                println!(
                    "{name:10} launch={} multi_turn={} stream_tools={} sessions={} acp={} yolo={}",
                    c.launch, c.multi_turn, c.stream_tools, c.sessions, c.acp, c.yolo
                );
            }
            println!("# infrastructure");
            for name in ["mock", "generic"] {
                let a = automedon::resolve(name)?;
                let c = a.capabilities();
                println!("{name:10} in_process={} launch={}", c.in_process, c.launch);
            }
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
            scenario,
        } => {
            let mut opts = automedon::LaunchOptions {
                yolo,
                model,
                cwd,
                ..Default::default()
            };
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
