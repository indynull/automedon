use assert_cmd::Command;
use predicates::prelude::*;

fn medon() -> Command {
    Command::cargo_bin("medon").unwrap()
}

#[test]
fn adapters_lists_mock() {
    medon()
        .arg("adapters")
        .assert()
        .success()
        .stdout(predicate::str::contains("NAME"))
        .stdout(predicate::str::contains("grok"))
        .stdout(predicate::str::contains("mock"));
}

#[test]
fn shot_mock_echo() {
    medon()
        .args(["shot", "mock", "hello", "--scenario", "echo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ECHO:hello"));
}

#[test]
fn run_multi_turn_example() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let script = root.join("examples/mock/multi_turn.rhai");
    medon()
        .current_dir(&root)
        .args(["run", script.to_str().unwrap(), "--print"])
        .assert()
        .success()
        .stdout(predicate::str::contains("MULTI_TURN_OK"));
}

#[test]
fn eval_snippet() {
    medon()
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
    medon()
        .args(["run", "/no/such/script.rhai"])
        .assert()
        .failure();
}

#[test]
fn help_works() {
    medon().arg("--help").assert().success();
}
