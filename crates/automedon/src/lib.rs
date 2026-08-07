//! # Automedon
//!
//! Drive local AI coding harnesses (Grok, Pi, Claude, …): session, wait, expect.
//!
//! ## Multi-turn
//!
//! ```no_run
//! use automedon::{Expect, Session};
//!
//! # async fn demo() -> automedon::Result<()> {
//! let mut s = Session::builder("mock")
//!     .extra("scenario", serde_json::json!("multi"))
//!     .build()?;
//! s.prompt("alpha").await?;
//! s.expect(Expect::text("T1:alpha")).await?;
//! s.await_turn().await?;
//! s.prompt("beta").await?;
//! s.expect(Expect::text("prior=T1:alpha")).await?;
//! s.close().await?;
//! # Ok(())
//! # }
//! ```

pub mod adapter;
pub mod config;
pub mod dsl;
pub mod error;
pub mod event;
pub mod expect;
pub mod session;
pub mod transport;
pub mod wait;

pub use adapter::{
    grok_prepare_args, known_names, pi_prepare_args, product_names, resolve, Adapter, AdapterKind,
    AiderAdapter, Capabilities, ClaudeAdapter, CodexAdapter, CopilotAdapter, CursorAdapter,
    GeminiAdapter, GenericAdapter, GrokAdapter, MockAdapter, OpenCodeAdapter, PiAdapter,
    TurnContext,
};
pub use config::LaunchOptions;
pub use error::{Error, Result};
pub use event::{
    Event, GoalRecord, HookRecord, PermissionRecord, PlanRecord, TimedEvent, ToolRecord, Transcript,
};
pub use expect::{Expect, Predicate};
pub use session::{run, RunResult, Session, SessionBuilder};
pub use wait::{Wait, WaitCondition};
