//! Structural check: offline full-surface example calls every Rhai driver method
//! registered in the DSL (method-name coverage of the operator surface).

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

/// Tokens that must appear as calls in `examples/mock/full_driver_surface.rhai`.
/// Keep in sync with `register_fn` names in `dsl/engine.rs` (session + free constructors).
const REQUIRED_TOKENS: &[&str] = &[
    // free: launch + asserts
    "launch(",
    "assert_contains(",
    "assert_true(",
    // session methods
    "prompt(",
    "expect(",
    "wait(",
    "wait_for(",
    "await_turn(",
    ".run(",
    "approve(",
    "deny(",
    "approve_plan(",
    "reject_plan(",
    "drain(",
    "close(",
    ".text()",
    "turn_text(",
    "thinking(",
    "session_id(",
    "harness(",
    "finished(",
    "turn(",
    "tool_names(",
    // expect constructors
    "text(",
    "thinking(",
    "tool(",
    "tool_any(",
    "tool_input(",
    "tool_result(",
    "tool_result_contains(",
    "tool_result_error(",
    "permission(",
    "hook(",
    "hook_any(",
    "hook_started(",
    "hook_finished(",
    "hook_phase(",
    "plan(",
    "plan_summary(",
    "plan_resolved(",
    "goal(",
    "goal_title(",
    "goal_progress(",
    "goal_completed(",
    "session_info(",
    "turn_complete(",
    "process_exit(",
    "done(",
    "timeout_ms(",
    // wait constructors
    "wait_text(",
    "wait_tool(",
    "wait_tool_any(",
    "wait_tool_result(",
    "wait_tool_input(",
    "wait_tool_result_contains(",
    "wait_tool_result_error(",
    "wait_permission(",
    "wait_hook(",
    "wait_hook_any(",
    "wait_hook_started(",
    "wait_hook_finished(",
    "wait_hook_phase(",
    "wait_plan(",
    "wait_goal(",
    "wait_turn_complete(",
    "wait_done(",
    "wait_timeout_ms(",
];

#[test]
fn full_driver_surface_script_calls_every_registered_method() {
    let path = workspace_root().join("examples/mock/full_driver_surface.rhai");
    let src =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let mut missing = Vec::new();
    for tok in REQUIRED_TOKENS {
        if !src.contains(tok) {
            missing.push(*tok);
        }
    }
    assert!(
        missing.is_empty(),
        "examples/mock/full_driver_surface.rhai missing tokens: {missing:?}\n\
         Update the script or REQUIRED_TOKENS when the DSL surface changes."
    );
}

#[test]
fn full_driver_surface_script_runs_offline() {
    let path = workspace_root().join("examples/mock/full_driver_surface.rhai");
    assert!(path.is_file(), "missing {}", path.display());
    let res = automedon::dsl::run_script(&path).expect("run_script full_driver_surface");
    let value = res.value.to_string();
    assert!(
        value.contains("FULL_DRIVER_SURFACE_OK"),
        "unexpected script result value={value:?} debug={res:?}"
    );
}
