//! Rhai scripting surface — the smart-developer DSL for driving harnesses.
//!
//! ```rhai
//! let s = launch("mock", #{ scenario: "tools" });
//! s.prompt("list files");
//! s.expect(tool("list_dir"));
//! s.expect(text("listed"));
//! s.expect(done());
//! print(s.text());
//! ```

mod engine;

pub use engine::{eval_file, eval_str, run_script, ScriptResult};
