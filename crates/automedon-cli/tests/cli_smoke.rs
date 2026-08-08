use assert_cmd::Command;
use predicates::prelude::*;

fn automedon() -> Command {
    Command::cargo_bin("automedon").unwrap()
}

#[test]
fn adapters_lists_mock() {
    automedon()
        .arg("adapters")
        .assert()
        .success()
        .stdout(predicate::str::contains("NAME"))
        .stdout(predicate::str::contains("BINARY"))
        .stdout(predicate::str::contains("MULTI-TURN"))
        .stdout(predicate::str::contains("grok"))
        .stdout(predicate::str::contains("claude"))
        .stdout(predicate::str::contains("mock"))
        .stdout(predicate::str::contains("examples/harnesses"));
}

#[test]
fn adapters_lists_default_binaries() {
    let out = automedon().arg("adapters").output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    // Operator-facing binary column from AdapterKind::default_binaries.
    for needle in ["claude", "codex", "copilot", "cursor-agent", "gemini"] {
        assert!(s.contains(needle), "missing {needle} in:\n{s}");
    }
}

#[test]
fn shot_mock_echo() {
    automedon()
        .args(["shot", "mock", "hello", "--scenario", "echo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ECHO:hello"));
}

#[test]
fn run_multi_turn_example() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let script = root.join("examples/mock/multi_turn.rhai");
    automedon()
        .current_dir(&root)
        .args(["run", script.to_str().unwrap(), "--print"])
        .assert()
        .success()
        .stdout(predicate::str::contains("MULTI_TURN_OK"));
}

#[test]
fn eval_snippet() {
    automedon()
        .args([
            "eval",
            r#"let s = launch("mock", #{ scenario: "echo" }); s.run("z")"#,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("ECHO:z"));
}

#[test]
fn run_missing_script_fails() {
    automedon()
        .args(["run", "/no/such/script.rhai"])
        .assert()
        .failure();
}

#[test]
fn help_works() {
    automedon().arg("--help").assert().success();
}
