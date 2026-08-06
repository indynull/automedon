//! Process transport: spawn a harness, stream stdout lines as bytes, capture stderr.

mod process;

pub use process::{spawn_process, ChildIo, SpawnSpec};
